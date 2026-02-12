mod api;
mod cache;
mod config;
mod models;
mod themes;
mod ui;
mod waybar;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::api::{ApiClient, calculate_stats};
use crate::cache::Cache;
use crate::config::ConfigManager;
use crate::models::{Config, Theme};

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
    /// Configure settings
    Config {
        /// Edit configuration file
        #[arg(short, long)]
        edit: bool,
    },
    /// Reset configuration
    Reset,
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
        Some(Commands::Config { edit: _ }) => {
            println!("Configuration file: {}", config_manager.config_path().display());
            println!("Token: {}...", &config.token[..10.min(config.token.len())]);
            println!("Theme: {}", config.theme);
            println!("Cache TTL: {} minutes", config.cache_ttl_minutes);
            return Ok(());
        }
        Some(Commands::Reset) => {
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
            
            // Validate token and get username
            let username = api_client.validate_token().await?;
            println!("Authenticated as: {}", username.cyan());

            // Fetch usage data
            let data = api_client.fetch_usage(&username).await?;
            
            // Cache the data
            cache.set(&data)?;
            
            data
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