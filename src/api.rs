use anyhow::{Context, Result};
use chrono::{Datelike, TimeZone, Utc};
use reqwest::header::HeaderMap;
use crate::models::{UsageData, UsageStats, ModelUsage};

const GITHUB_API_URL: &str = "https://api.github.com";

pub struct ApiClient {
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(token: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept",
            reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            reqwest::header::HeaderValue::from_static("2022-11-28"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("copilot-usage/0.1.0")
            .build()?;

        Ok(Self { client })
    }

    pub async fn fetch_usage(&self, username: &str) -> Result<UsageData> {
        let url = format!(
            "{}/users/{}/settings/billing/premium_request/usage",
            GITHUB_API_URL, username
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to GitHub API")?;

        let status = response.status();
        
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            
            if status == 403 {
                anyhow::bail!(
                    "Access Forbidden (403). Your token may lack the 'Plan' permission. \
                    Please ensure you have enabled 'Plan' → Read-only in Account permissions."
                );
            } else if status == 404 {
                anyhow::bail!(
                    "Not Found (404). This could mean:\n\
                    1. You don't have GitHub Copilot Pro on a personal plan\n\
                    2. Your Copilot is managed through an organization\n\
                    3. The billing API is not available for your account type"
                );
            } else {
                anyhow::bail!("GitHub API error: {} - {}", status, text);
            }
        }

        let data: UsageData = response
            .json()
            .await
            .context("Failed to parse GitHub API response")?;

        Ok(data)
    }

    pub async fn get_authenticated_user(&self) -> Result<String> {
        let url = format!("{}/user", GITHUB_API_URL);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get user info")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get user: {} - {}", status, text);
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
    const TOTAL_LIMIT: f64 = 300.0;
    const COST_PER_REQUEST: f64 = 0.04;
    
    let total_used: f64 = data.usage_items.iter().map(|item| item.gross_quantity).sum();
    let total_billed: f64 = data.usage_items.iter().map(|item| item.net_quantity).sum();
    let percentage = (total_used / TOTAL_LIMIT) * 100.0;

    let now = Utc::now();
    let (next_year, next_month) = if now.month() == 12 {
        (now.year() + 1, 1)
    } else {
        (now.year(), now.month() + 1)
    };
    let reset_date = Utc.with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
        .earliest()
        .expect("Invalid date");

    let mut model_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for item in &data.usage_items {
        *model_map.entry(item.model.clone()).or_insert(0.0) += item.gross_quantity;
    }

    let mut models: Vec<ModelUsage> = model_map
        .into_iter()
        .map(|(name, used)| ModelUsage {
            name,
            used,
            limit: TOTAL_LIMIT,
            percentage: (used / TOTAL_LIMIT) * 100.0,
        })
        .collect();

    models.sort_by(|a, b| b.used.partial_cmp(&a.used).unwrap());

    let estimated_cost = if total_billed > 0.0 {
        total_billed * COST_PER_REQUEST
    } else {
        0.0
    };

    UsageStats {
        total_used,
        total_limit: TOTAL_LIMIT,
        percentage,
        reset_date,
        models,
        estimated_cost,
    }
}