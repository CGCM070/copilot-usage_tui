mod api;
mod cache;
mod config;
mod models;
mod themes;
mod ui;
mod waybar;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::api::{ApiClient, calculate_stats};
use crate::cache::Cache;
use crate::config::ConfigManager;
use crate::models::Theme;

#[derive(Parser)]
#[command(name = "copilot-usage_cli")]
#[command(about = "GitHub Copilot usage tracker CLI")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Force refresh data from API
    #[arg(short, long)]
    refresh: bool,

    /// Output for Waybar
    #[arg(long)]
    waybar: bool,

    /// Use specific theme
    #[arg(short, long)]
    theme: Option<String>,

    /// Show cache status
    #[arg(long)]
    cache_status: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current configuration
    Config,
    /// Reset and reconfigure settings
    Reset,
    /// Reconfigure (alias for reset)
    Reconfigure,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check if we just need cache status
    if cli.cache_status {
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
            println!("No configuration found. Run without arguments to set up.");
        }
        return Ok(());
    }

    // Setup config
    let config_manager = ConfigManager::new()?;
    let config = match config_manager.load()? {
        Some(cfg) => cfg,
        None => {
            println!("{}", "Welcome to GitHub Copilot Usage CLI!".cyan().bold());
            config_manager.setup_interactive()?
        }
    };

    // Handle subcommands
    match cli.command {
        Some(Commands::Config) => {
            println!("Configuration file: {}", config_manager.config_path().display());
            if config.token.is_empty() {
                println!("Token: {}", "(not set)".red());
            } else {
                let visible_chars = 10.min(config.token.len());
                println!("Token: {}...", &config.token[..visible_chars]);
            }
            println!("Theme: {}", config.theme);
            println!("Cache TTL: {} minutes", config.cache_ttl_minutes);
            return Ok(());
        }
        Some(Commands::Reset) | Some(Commands::Reconfigure) => {
            println!("{}", "Reconfiguring...".yellow());
            config_manager.setup_interactive()?;
            return Ok(());
        }
        None => {}
    }

    // Get data (from cache or API)
    let cache = Cache::new(config.cache_ttl_minutes)?;

    if cli.refresh {
        cache.invalidate()?;
        println!("{}", "Cache invalidated. Fetching fresh data...".yellow());
    }

    let usage_data = match cache.get()? {
        Some(data) => {
            println!("{}", "Using cached data".dimmed());
            data
        }
        None => {
            println!("{}", "Fetching data from GitHub API...".cyan());
            
            let api_client = ApiClient::new(config.token.clone())?;
            
            // Try to get username from API
            let username = match api_client.get_authenticated_user().await {
                Ok(user) => {
                    println!("Authenticated as: {}", user.cyan());
                    user
                }
                Err(_) => {
                    // Fine-grained tokens don't have access to /user endpoint
                    // Ask user for their GitHub username
                    println!("\n{}", "Could not determine username from token.".yellow());
                    println!("(Fine-grained tokens don't have access to the user endpoint)");
                    println!();
                    
                    let username: String = dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("Enter your GitHub username")
                        .interact_text()?;
                    
                    username
                }
            };

            // Fetch usage data
            match api_client.fetch_usage(&username).await {
                Ok(data) => {
                    // Cache the data
                    cache.set(&data)?;
                    data
                }
                Err(e) => {
                    let err_str = format!("{}", e);
                    
                    if err_str.contains("403") {
                        eprintln!("\n{}", "⚠️  API Access Denied! (403)".red().bold());
                        eprintln!("{}", "Your token doesn't have permission to access billing data.".red());
                        eprintln!();
                        eprintln!("{}", "Make sure your token has:".yellow().bold());
                        eprintln!("  • Account → Plan (Read) permission");
                        eprintln!();
                        eprintln!("{}", "Note: This is NOT 'Copilot Requests' permission!".yellow());
                        eprintln!("{}", "You need the 'Plan' permission specifically.".yellow());
                        eprintln!();
                        eprintln!("To fix this:");
                        eprintln!("  1. Go to https://github.com/settings/personal-access-tokens");
                        eprintln!("  2. Find your token and click 'Edit'");
                        eprintln!("  3. Under 'Account permissions', enable 'Plan' → Read-only");
                        eprintln!("  4. Save the token");
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
                        eprintln!("  1. The username '{}' doesn't exist", username);
                        eprintln!("  2. You don't have GitHub Copilot Pro on a personal plan");
                        eprintln!("  3. Your Copilot is managed through an organization");
                        eprintln!();
                    } else {
                        eprintln!("\n{}", format!("Error: {}", e).red().bold());
                    }
                    
                    std::process::exit(1);
                }
            }
        }
    };

    // Calculate stats
    let stats = calculate_stats(&usage_data);

    // Determine theme
    let theme = cli.theme.as_ref()
        .map(|t| Theme::from_str(t))
        .unwrap_or_else(|| Theme::from_str(&config.theme));

    // Output based on mode
    if cli.waybar {
        let output = waybar::generate_output(&stats, &config.waybar_format);
        println!("{}", output);
    } else {
        // Run TUI
        ui::run_ui(&stats, theme)?;
    }

    Ok(())
}