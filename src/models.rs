use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Usage data from GitHub API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageData {
    pub time_period: TimePeriod,
    pub user: String,
    pub usage_items: Vec<UsageItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimePeriod {
    pub year: i32,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageItem {
    pub product: String,
    pub sku: String,
    pub model: String,
    pub unit_type: String,
    pub price_per_unit: f64,
    pub gross_quantity: f64,
    pub gross_amount: f64,
    pub discount_quantity: f64,
    pub discount_amount: f64,
    pub net_quantity: f64,
    pub net_amount: f64,
}

/// Aggregated usage statistics
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub total_used: f64,
    pub total_limit: f64,
    pub percentage: f64,
    pub reset_date: DateTime<Utc>,
    pub models: Vec<ModelUsage>,
    pub estimated_cost: f64,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct ModelUsage {
    pub name: String,
    pub used: f64,
    pub limit: f64,
    pub percentage: f64,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    pub theme: String,
    pub cache_ttl_minutes: u64,
    pub waybar_format: String,
    #[serde(default)]
    pub username: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token: String::new(),
            theme: "dark".to_string(),
            cache_ttl_minutes: 5,
            waybar_format: "{percentage}%".to_string(),
            username: None,
        }
    }
}

/// Cache entry with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub data: UsageData,
    pub timestamp: DateTime<Utc>,
}

/// Cache status for robust cache checks
#[derive(Debug, Clone)]
pub enum CacheStatus {
    /// Cache is fresh and contains valid data
    Fresh(UsageData),
    /// Cache exists but has expired
    Expired,
    /// Cache file doesn't exist or couldn't be read
    Missing,
    /// Cache file is corrupted or invalid
    Corrupted,
}

/// Waybar output format
#[derive(Debug, Clone, Serialize)]
pub struct WaybarOutput {
    pub text: String,
    pub tooltip: String,
    #[serde(rename = "class")]
    pub class: String,
}

/// Available themes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Dark,
    Light,
    Dracula,
    Nord,
    Monokai,
    Gruvbox,
}

impl Theme {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Theme::Light,
            "dracula" => Theme::Dracula,
            "nord" => Theme::Nord,
            "monokai" => Theme::Monokai,
            "gruvbox" => Theme::Gruvbox,
            _ => Theme::Dark,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Dracula => "dracula",
            Theme::Nord => "nord",
            Theme::Monokai => "monokai",
            Theme::Gruvbox => "gruvbox",
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
