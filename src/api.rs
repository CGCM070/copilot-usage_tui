use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use crate::models::{UsageData, UsageStats, UsageItem, ModelUsage};

pub struct ApiClient {
    client: reqwest::Client,
    token: String,
    base_url: String,
}

impl ApiClient {
    pub fn new(token: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            token,
            base_url: "https://api.github.com".to_string(),
        })
    }

    pub async fn fetch_usage(&self, username: &str) -> Result<UsageData> {
        let url = format!(
            "{}/users/{}/settings/billing/premium_request/usage",
            self.base_url, username
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to GitHub API")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error: {} - {}", status, text);
        }

        let data: UsageData = response
            .json()
            .await
            .context("Failed to parse GitHub API response")?;

        Ok(data)
    }

    pub async fn validate_token(&self) -> Result<String> {
        let url = format!("{}/user", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to validate token")?;

        if !response.status().is_success() {
            anyhow::bail!("Invalid token or insufficient permissions");
        }

        #[derive(serde::Deserialize)]
        struct User {
            login: String,
        }

        let user: User = response.json().await?;
        Ok(user.login)
    }
}

pub fn calculate_stats(data: &UsageData) -> UsageStats {
    let total_limit = 300.0; // GitHub Copilot Pro limit
    
    let total_used: f64 = data.usage_items.iter().map(|item| item.net_quantity).sum();
    let percentage = (total_used / total_limit) * 100.0;

    // Calculate reset date (first day of next month)
    let now = Utc::now();
    let (next_year, next_month) = if now.month() == 12 {
        (now.year() + 1, 1)
    } else {
        (now.year(), now.month() + 1)
    };
    let reset_date = Utc.with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
        .earliest()
        .expect("Invalid date");

    // Group by model
    let mut model_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for item in &data.usage_items {
        let count = model_map.entry(item.model.clone()).or_insert(0.0);
        *count += item.net_quantity;
    }

    let mut models: Vec<ModelUsage> = model_map
        .into_iter()
        .map(|(name, used)| ModelUsage {
            name,
            used,
            limit: total_limit,
            percentage: (used / total_limit) * 100.0,
        })
        .collect();

    models.sort_by(|a, b| b.used.partial_cmp(&a.used).unwrap());

    // Calculate estimated cost (overages only)
    let estimated_cost = if total_used > total_limit {
        (total_used - total_limit) * 0.04
    } else {
        0.0
    };

    UsageStats {
        total_used,
        total_limit,
        percentage,
        reset_date,
        models,
        estimated_cost,
    }
}