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
        let proj_dirs = ProjectDirs::from("com", "copilot-usage", "copilot-usage")
            .context("Failed to determine config directory")?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        let config_path = config_dir.join("config.toml");

        Ok(Self { config_path })
    }

    /// Create a ConfigManager with a custom path (for testing)
    #[cfg(test)]
    pub fn with_path(config_path: PathBuf) -> Self {
        Self { config_path }
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
            .items(&[
                "dark",
                "dracula",
                "nord",
                "monokai",
                "gruvbox",
                "catppuccin",
                "onedark",
                "tokyonight",
                "solarized",
                "kanagawa",
            ])
            .interact()?;

        let themes = [
            "dark",
            "dracula",
            "nord",
            "monokai",
            "gruvbox",
            "catppuccin",
            "onedark",
            "tokyonight",
            "solarized",
            "kanagawa",
        ];

        let config = Config {
            token: token.trim().to_string(),
            theme: themes[theme_idx].to_string(),
            cache_ttl_minutes: 5,
            waybar_format: "{percentage}%".to_string(),
            username: None,
        };

        self.save(&config)?;

        println!();
        println!("{}", style("Configuration saved!").green().bold());
        println!("Location: {}", style(self.config_path.display()).dim());

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> Config {
        Config {
            token: "ghp_test123".to_string(),
            theme: "dark".to_string(),
            cache_ttl_minutes: 5,
            waybar_format: "{percentage}%".to_string(),
            username: Some("testuser".to_string()),
        }
    }

    #[test]
    fn test_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        let manager = ConfigManager::with_path(config_path);

        let result = manager.load().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let manager = ConfigManager::with_path(config_path);

        let config = create_test_config();
        manager.save(&config).unwrap();

        let loaded = manager.load().unwrap().unwrap();
        assert_eq!(loaded.token, "ghp_test123");
        assert_eq!(loaded.theme, "dark");
        assert_eq!(loaded.cache_ttl_minutes, 5);
        assert_eq!(loaded.username, Some("testuser".to_string()));
    }

    #[test]
    fn test_config_path() {
        let temp_dir = TempDir::new().unwrap();
        let expected_path = temp_dir.path().join("myconfig.toml");
        let manager = ConfigManager::with_path(expected_path.clone());

        assert_eq!(manager.config_path(), &expected_path);
    }

    #[test]
    fn test_update_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let manager = ConfigManager::with_path(config_path);

        let mut config = create_test_config();
        manager.save(&config).unwrap();

        // Update theme
        config.theme = "nord".to_string();
        manager.save(&config).unwrap();

        let loaded = manager.load().unwrap().unwrap();
        assert_eq!(loaded.theme, "nord");
    }

    #[test]
    fn test_username_optional() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let manager = ConfigManager::with_path(config_path);

        let config = Config {
            token: "ghp_test".to_string(),
            theme: "dark".to_string(),
            cache_ttl_minutes: 5,
            waybar_format: "{percentage}%".to_string(),
            username: None,
        };
        manager.save(&config).unwrap();

        let loaded = manager.load().unwrap().unwrap();
        assert!(loaded.username.is_none());
    }
}
