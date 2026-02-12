use crate::models::{ModelUsage, UsageData, UsageStats};
use anyhow::{Context, Result};
use chrono::{Datelike, TimeZone, Utc};
use reqwest::header::HeaderMap;

const GITHUB_API_URL: &str = "https://api.github.com";

/// Safely extract text from response, with fallback
async fn extract_response_text(response: reqwest::Response) -> String {
    match response.text().await {
        Ok(text) => text,
        Err(e) => format!("(Failed to read response body: {})", e),
    }
}

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
            .connect_timeout(std::time::Duration::from_secs(10))
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
            .context("Failed to connect to GitHub API. Check your internet connection.")?;

        let status = response.status();

        if !status.is_success() {
            let text = extract_response_text(response).await;

            match status.as_u16() {
                401 => anyhow::bail!(
                    "Unauthorized (401). Your token is invalid or expired.\n\
                    Please run 'copilot-usage reconfigure' to set a new token."
                ),
                403 => anyhow::bail!(
                    "Access Forbidden (403). Your token lacks the 'Plan' permission.\n\
                    Please ensure you have enabled 'Plan' -> Read-only in Account permissions."
                ),
                404 => anyhow::bail!(
                    "Not Found (404). This could mean:\n\
                    1. You don't have GitHub Copilot Pro on a personal plan\n\
                    2. Your Copilot is managed through an organization\n\
                    3. The billing API is not available for your account type"
                ),
                429 => anyhow::bail!(
                    "Rate Limit Exceeded (429). GitHub API limit reached.\n\
                    Please wait a few minutes before trying again."
                ),
                500..=599 => anyhow::bail!(
                    "GitHub Server Error ({}). GitHub's API is experiencing issues.\n\
                    Please try again later. Response: {}",
                    status,
                    text
                ),
                _ => anyhow::bail!("GitHub API error ({}): {}", status, text),
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
            .context("Failed to connect to GitHub API. Check your internet connection.")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = extract_response_text(response).await;

            match status.as_u16() {
                401 => anyhow::bail!("Token unauthorized (401). Cannot determine username."),
                403 => anyhow::bail!(
                    "Token lacks 'user' permission (403). Fine-grained tokens may not support this."
                ),
                _ => anyhow::bail!("Failed to get user ({}): {}", status, text),
            }
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

    let total_used: f64 = data
        .usage_items
        .iter()
        .map(|item| item.gross_quantity)
        .sum();
    let total_billed: f64 = data.usage_items.iter().map(|item| item.net_quantity).sum();
    let percentage = (total_used / TOTAL_LIMIT) * 100.0;

    let now = Utc::now();
    let (next_year, next_month) = if now.month() == 12 {
        (now.year() + 1, 1)
    } else {
        (now.year(), now.month() + 1)
    };
    let reset_date = Utc
        .with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
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
        username: data.user.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TimePeriod, UsageItem};

    fn create_test_usage_item(model: &str, gross_quantity: f64, net_quantity: f64) -> UsageItem {
        UsageItem {
            product: "copilot".to_string(),
            sku: "premium".to_string(),
            model: model.to_string(),
            unit_type: "request".to_string(),
            price_per_unit: 0.04,
            gross_quantity,
            gross_amount: gross_quantity * 0.04,
            discount_quantity: 0.0,
            discount_amount: 0.0,
            net_quantity,
            net_amount: net_quantity * 0.04,
        }
    }

    fn create_test_usage_data(items: Vec<UsageItem>) -> UsageData {
        UsageData {
            time_period: TimePeriod {
                year: 2026,
                month: Some(2),
                day: None,
            },
            user: "testuser".to_string(),
            usage_items: items,
        }
    }

    #[test]
    fn test_calculate_stats_empty() {
        let data = create_test_usage_data(vec![]);
        let stats = calculate_stats(&data);

        assert_eq!(stats.total_used, 0.0);
        assert_eq!(stats.total_limit, 300.0);
        assert_eq!(stats.percentage, 0.0);
        assert_eq!(stats.models.len(), 0);
        assert_eq!(stats.estimated_cost, 0.0);
        assert_eq!(stats.username, "testuser");
    }

    #[test]
    fn test_calculate_stats_single_model() {
        let data = create_test_usage_data(vec![create_test_usage_item("gpt-4", 100.0, 0.0)]);
        let stats = calculate_stats(&data);

        assert_eq!(stats.total_used, 100.0);
        assert!((stats.percentage - 33.333).abs() < 0.01);
        assert_eq!(stats.models.len(), 1);
        assert_eq!(stats.models[0].name, "gpt-4");
        assert_eq!(stats.models[0].used, 100.0);
    }

    #[test]
    fn test_calculate_stats_multiple_models() {
        let data = create_test_usage_data(vec![
            create_test_usage_item("gpt-4", 100.0, 0.0),
            create_test_usage_item("claude-sonnet", 50.0, 0.0),
            create_test_usage_item("gpt-4", 25.0, 0.0), // Same model, should aggregate
        ]);
        let stats = calculate_stats(&data);

        assert_eq!(stats.total_used, 175.0);
        assert_eq!(stats.models.len(), 2);
        // Models should be sorted by usage descending
        assert_eq!(stats.models[0].name, "gpt-4");
        assert_eq!(stats.models[0].used, 125.0); // 100 + 25
        assert_eq!(stats.models[1].name, "claude-sonnet");
        assert_eq!(stats.models[1].used, 50.0);
    }

    #[test]
    fn test_calculate_stats_with_billing() {
        let data = create_test_usage_data(vec![
            create_test_usage_item("gpt-4", 350.0, 50.0), // 50 billed
        ]);
        let stats = calculate_stats(&data);

        assert_eq!(stats.total_used, 350.0);
        assert!((stats.percentage - 116.67).abs() < 0.01); // Over 100%
        assert!((stats.estimated_cost - 2.0).abs() < 0.01); // 50 * 0.04 = 2.0
    }
}
