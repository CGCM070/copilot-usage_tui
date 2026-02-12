use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::api::{ApiClient, calculate_stats};
use crate::cache::Cache;
use crate::config::ConfigManager;
use crate::models::{CacheStatus, Theme};
use crate::ui;
use crate::waybar;

#[derive(Parser)]
#[command(name = "copilot-usage")]
#[command(about = "GitHub Copilot Usage Tracker CLI")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Force refresh data from API
    #[arg(short, long)]
    pub refresh: bool,

    /// Output for Waybar
    #[arg(long)]
    pub waybar: bool,

    /// Use specific theme
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Show cache status
    #[arg(long)]
    pub cache_status: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show current configuration
    Config,
    /// Reset and reconfigure settings
    Reset,
    /// Reconfigure (alias for reset)
    Reconfigure,
}

/// Ejecuta la CLI y maneja los comandos
pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config_manager = ConfigManager::new()?;

    // Comandos directos (no interactivos)
    if cli.cache_status {
        return show_cache_status(&config_manager).await;
    }

    // Modo Waybar
    if cli.waybar {
        return run_waybar_mode(&config_manager, cli.refresh).await;
    }

    match cli.command {
        Some(Commands::Config) => return show_config(&config_manager).await,
        Some(Commands::Reset) | Some(Commands::Reconfigure) => {
            reconfigure(&config_manager).await?;
            // Continue to interactive mode after reconfiguration
            println!("\nLaunching dashboard...\n");
        }
        None => {}
    }

    // Modo interactivo
    run_interactive_mode(&config_manager, cli).await
}

async fn run_waybar_mode(config_manager: &ConfigManager, force_refresh: bool) -> Result<()> {
    // Check config first to avoid interactive setup prompts in JSON output
    if config_manager.load()?.is_none() {
        eprintln!("Configuration missing. Run interactively first.");
        return Ok(());
    }

    match fetch_usage_data(config_manager, force_refresh).await {
        Ok(stats) => {
            let config = config_manager.load()?.unwrap_or_default();
            let output = waybar::generate_output(&stats, &config.waybar_format);
            println!("{}", output);
        }
        Err(_) => {
            // Error is already printed to stderr by fetch_usage_data
        }
    }
    Ok(())
}

async fn run_interactive_mode(config_manager: &ConfigManager, cli: Cli) -> Result<()> {
    let mut force_refresh = cli.refresh;
    let mut current_theme: Option<Theme> = cli.theme.as_ref().map(|t| Theme::from_str(t));

    // Main loop: allows reloading stats after reconfigure
    loop {
        // Fetch data (fresh on first run if --refresh, or after reconfigure)
        let stats = fetch_usage_data(config_manager, force_refresh).await?;
        force_refresh = false;

        // Get theme: use cached value or load from config
        let theme = current_theme.unwrap_or_else(|| {
            config_manager
                .load()
                .ok()
                .flatten()
                .map(|c| Theme::from_str(&c.theme))
                .unwrap_or(Theme::Dark)
        });

        // Run UI with current stats
        match ui::run_ui(&stats, theme)? {
            None => break,
            Some(action) => match action.as_str() {
                "quit" => break,

                action if action.starts_with("theme:") => {
                    let theme_name = action.strip_prefix("theme:").unwrap();
                    current_theme = Some(Theme::from_str(theme_name));
                    save_theme_preference(config_manager, theme_name)?;
                }

                "reconfigure" => {
                    reconfigure(config_manager).await?;
                    // Reset theme to reload from new config
                    current_theme = None;
                    force_refresh = true;
                }

                _ => {}
            },
        }
    }

    Ok(())
}

async fn show_cache_status(config_manager: &ConfigManager) -> Result<()> {
    if let Some(config) = config_manager.load()? {
        let cache = Cache::new(config.cache_ttl_minutes)?;
        match cache.status() {
            CacheStatus::Fresh(_) => {
                if let Some(timestamp) = cache.last_updated()? {
                    println!("Cache last updated: {}", timestamp);
                }
                println!("Cache status: {}", "fresh".green());
            }
            CacheStatus::Expired => {
                if let Some(timestamp) = cache.last_updated()? {
                    println!("Cache last updated: {}", timestamp);
                }
                println!("Cache status: {}", "expired".yellow());
            }
            CacheStatus::Missing => {
                println!("Cache status: {}", "empty".red());
            }
            CacheStatus::Corrupted => {
                println!("Cache status: {}", "corrupted".red());
            }
        }
    } else {
        println!("No configuration found.");
    }
    Ok(())
}

async fn show_config(config_manager: &ConfigManager) -> Result<()> {
    let config = config_manager.load()?.unwrap_or_default();

    println!(
        "Configuration file: {}",
        config_manager.config_path().display()
    );
    if config.token.is_empty() {
        println!("Token: {}", "(not set)".red());
    } else {
        let visible_chars = 10.min(config.token.len());
        println!("Token: {}...", &config.token[..visible_chars]);
    }
    println!("Theme: {}", config.theme);
    println!("Cache TTL: {} minutes", config.cache_ttl_minutes);
    Ok(())
}

async fn reconfigure(config_manager: &ConfigManager) -> Result<()> {
    println!("Reconfiguring...");
    config_manager.setup_interactive()?;
    println!("Configuration updated!");
    Ok(())
}

fn save_theme_preference(config_manager: &ConfigManager, theme_name: &str) -> Result<()> {
    if let Some(mut config) = config_manager.load()? {
        config.theme = theme_name.to_string();
        config_manager.save(&config)?;
    }
    Ok(())
}

async fn fetch_usage_data(
    config_manager: &ConfigManager,
    force_refresh: bool,
) -> Result<crate::models::UsageStats> {
    let mut config = match config_manager.load()? {
        Some(cfg) => cfg,
        None => {
            println!("Welcome to GitHub Copilot Usage CLI!");
            config_manager.setup_interactive()?
        }
    };

    let cache = Cache::new(config.cache_ttl_minutes)?;

    if force_refresh {
        cache.invalidate()?;
    }

    let usage_data = match cache.status() {
        CacheStatus::Fresh(data) => data,
        _ => {
            let api_client = ApiClient::new(config.token.clone())?;

            // Try to get username: 1) from config, 2) from API, 3) prompt user
            let username = if let Some(ref cached_username) = config.username {
                cached_username.clone()
            } else {
                match api_client.get_authenticated_user().await {
                    Ok(user) => {
                        // Cache the username for future use
                        config.username = Some(user.clone());
                        config_manager.save(&config)?;
                        user
                    }
                    Err(_) => {
                        println!("\nCould not determine username from token.");
                        let user: String = dialoguer::Input::with_theme(
                            &dialoguer::theme::ColorfulTheme::default(),
                        )
                        .with_prompt("Enter your GitHub username")
                        .interact_text()?;
                        // Cache the username for future use
                        config.username = Some(user.clone());
                        config_manager.save(&config)?;
                        user
                    }
                }
            };

            match api_client.fetch_usage(&username).await {
                Ok(data) => {
                    cache.set(&data)?;
                    data
                }
                Err(e) => {
                    handle_api_error(&e, config_manager).await?;
                    return Err(e);
                }
            }
        }
    };

    Ok(calculate_stats(&usage_data))
}

async fn handle_api_error(e: &anyhow::Error, config_manager: &ConfigManager) -> Result<()> {
    let err_str = format!("{}", e);

    if err_str.contains("403") {
        eprintln!("\nAPI Access Denied! (403)");
        eprintln!("Your token doesn't have permission to access billing data.");
        eprintln!();
        eprintln!("Make sure your token has:");
        eprintln!("  • Account → Plan (Read) permission");
        eprintln!();

        let should_reconfigure =
            dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Reconfigure with correct token?")
                .default(true)
                .interact()?;

        if should_reconfigure {
            config_manager.setup_interactive()?;
        }
    } else if err_str.contains("404") {
        eprintln!("\nNot Found (404)");
        eprintln!("This could mean:");
        eprintln!("  1. User doesn't exist");
        eprintln!("  2. No GitHub Copilot Pro on personal plan");
        eprintln!("  3. Copilot managed through organization");
    }

    Ok(())
}
