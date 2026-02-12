use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::api::{ApiClient, calculate_stats};
use crate::cache::Cache;
use crate::config::ConfigManager;
use crate::models::Theme;
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

    // Comandos directos (no interactivos)
    if cli.cache_status {
        return show_cache_status().await;
    }

    // Modo Waybar
    if cli.waybar {
        return run_waybar_mode(cli.refresh).await;
    }

    match cli.command {
        Some(Commands::Config) => return show_config().await,
        Some(Commands::Reset) | Some(Commands::Reconfigure) => return reconfigure().await,
        None => {}
    }

    // Modo interactivo
    run_interactive_mode(cli).await
}

async fn run_waybar_mode(force_refresh: bool) -> Result<()> {
    let config_manager = ConfigManager::new()?;
    
    // Check config first to avoid interactive setup prompts in JSON output
    if config_manager.load()?.is_none() {
        eprintln!("Configuration missing. Run interactively first.");
        return Ok(());
    }

    match fetch_usage_data(force_refresh).await {
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

async fn run_interactive_mode(cli: Cli) -> Result<()> {
    let mut current_theme = get_initial_theme(&cli.theme)?;
    let mut force_refresh = cli.refresh;

    loop {
        let stats = fetch_usage_data(force_refresh).await?;
        force_refresh = false;

        match ui::run_ui(&stats, current_theme)? {
            Some(action) => {
                match action.as_str() {
                    "quit" => break,
                    "refresh" => {
                        println!("{}", "🔄 Refreshing data...".cyan());
                        force_refresh = true;
                    }
                    "cache" => {
                        show_cache_status().await?;
                        println!("\nPress Enter to continue...");
                        let _ = std::io::stdin().read_line(&mut String::new());
                    }
                    cmd if cmd.starts_with("theme:") => {
                        let theme_name = cmd.strip_prefix("theme:").unwrap();
                        current_theme = Theme::from_str(theme_name);
                        save_theme_preference(theme_name).await?;
                        println!("{} {}", "✓ Theme changed to:".green(), theme_name.cyan());
                    }
                    "reconfigure" => {
                        reconfigure().await?;
                        force_refresh = true;
                    }
                    _ => {}
                }
            }
            None => break,
        }
    }

    Ok(())
}

async fn show_cache_status() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    if let Some(config) = config_manager.load()? {
        let cache = Cache::new(config.cache_ttl_minutes)?;
        match cache.last_updated()? {
            Some(timestamp) => {
                println!("Cache last updated: {}", timestamp);
                if cache.is_fresh() {
                    println!("Cache status: {}", "fresh".green());
                } else {
                    println!("Cache status: {}", "expired".yellow());
                }
            }
            None => println!("Cache status: {}", "empty".red()),
        }
    } else {
        println!("No configuration found.");
    }
    Ok(())
}

async fn show_config() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let config = config_manager.load()?.unwrap_or_default();

    println!("Configuration file: {}", config_manager.config_path().display());
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

async fn reconfigure() -> Result<()> {
    println!("{}", "⚙️  Reconfiguring...".yellow());
    let config_manager = ConfigManager::new()?;
    config_manager.setup_interactive()?;
    println!("{}", "✓ Configuration updated!".green());
    Ok(())
}

fn get_initial_theme(cli_theme: &Option<String>) -> Result<Theme> {
    if let Some(theme_str) = cli_theme {
        Ok(Theme::from_str(theme_str))
    } else {
        let config_manager = ConfigManager::new()?;
        let config = config_manager.load()?.unwrap_or_default();
        Ok(Theme::from_str(&config.theme))
    }
}

async fn save_theme_preference(theme_name: &str) -> Result<()> {
    let config_manager = ConfigManager::new()?;
    if let Some(mut config) = config_manager.load()? {
        config.theme = theme_name.to_string();
        config_manager.save(&config)?;
    }
    Ok(())
}

async fn fetch_usage_data(force_refresh: bool) -> Result<crate::models::UsageStats> {
    let config_manager = ConfigManager::new()?;
    let config = match config_manager.load()? {
        Some(cfg) => cfg,
        None => {
            println!("{}", "Welcome to GitHub Copilot Usage CLI!".cyan().bold());
            config_manager.setup_interactive()?
        }
    };

    let cache = Cache::new(config.cache_ttl_minutes)?;

    if force_refresh {
        cache.invalidate()?;
    }

    let usage_data = match cache.get()? {
        Some(data) => data,
        None => {
            let api_client = ApiClient::new(config.token.clone())?;

            let username = match api_client.get_authenticated_user().await {
                Ok(user) => user,
                Err(_) => {
                    println!("\n{}", "Could not determine username from token.".yellow());
                    dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("Enter your GitHub username")
                        .interact_text()?
                }
            };

            match api_client.fetch_usage(&username).await {
                Ok(data) => {
                    cache.set(&data)?;
                    data
                }
                Err(e) => {
                    handle_api_error(&e, &config_manager).await?;
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
        eprintln!("\n{}", "⚠️  API Access Denied! (403)".red().bold());
        eprintln!("{}", "Your token doesn't have permission to access billing data.".red());
        eprintln!();
        eprintln!("{}", "Make sure your token has:".yellow().bold());
        eprintln!("  • Account → Plan (Read) permission");
        eprintln!();

        let should_reconfigure = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Reconfigure with correct token?")
            .default(true)
            .interact()?;

        if should_reconfigure {
            config_manager.setup_interactive()?;
        }
    } else if err_str.contains("404") {
        eprintln!("\n{}", "⚠️  Not Found (404)".red().bold());
        eprintln!("{}", "This could mean:".yellow());
        eprintln!("  1. User doesn't exist");
        eprintln!("  2. No GitHub Copilot Pro on personal plan");
        eprintln!("  3. Copilot managed through organization");
    }

    Ok(())
}