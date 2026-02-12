use crate::models::Config;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "copilot-usage", "copilot-usage_cli")
            .context("Failed to determine config directory")?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        let config_path = config_dir.join("config.toml");

        Ok(Self { config_path })
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn load(&self) -> Result<Option<Config>> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.config_path)?;
        let config: Config = toml::from_str(&content)?;

        Ok(Some(config))
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        let content = toml::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn setup_interactive(&self) -> Result<Config> {
        use console::style;
        use dialoguer::{theme::ColorfulTheme, Input};

        println!(
            "{}",
            style("GitHub Copilot Usage CLI - Setup").bold().cyan()
        );
        println!("{}", style("=================================").cyan());
        println!();
        println!("{}", style("Please create a Personal Access Token:").dim());
        println!("1. Go to: https://github.com/settings/personal-access-tokens/new");
        println!("2. Select 'Fine-grained tokens'");
        println!("3. Resource owner: Your account");
        println!("4. Permission: Plan (Read)");
        println!();

        let token: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("GitHub Personal Access Token")
            .validate_with(|input: &String| {
                if input.starts_with("ghp_") || input.starts_with("github_pat_") {
                    Ok(())
                } else {
                    Err("Token should start with 'ghp_' or 'github_pat_'")
                }
            })
            .interact_text()?;

        let theme_idx: usize = dialoguer::Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select theme")
            .default(0)
            .items(&["dark", "light", "dracula", "nord", "monokai", "gruvbox"])
            .interact()?;

        let themes = vec!["dark", "light", "dracula", "nord", "monokai", "gruvbox"];

        let config = Config {
            token: token.trim().to_string(),
            theme: themes[theme_idx].to_string(),
            cache_ttl_minutes: 5,
            waybar_format: "{percentage}%".to_string(),
        };

        self.save(&config)?;

        println!();
        println!("{}", style("Configuration saved!").green().bold());
        println!("Location: {}", style(self.config_path.display()).dim());

        Ok(config)
    }
}
