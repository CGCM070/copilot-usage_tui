use crate::models::{CacheEntry, CacheStatus, UsageData};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub struct Cache {
    cache_path: PathBuf,
    ttl_minutes: u64,
}

impl Cache {
    pub fn new(ttl_minutes: u64) -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "copilot-usage", "copilot-usage")
            .context("Failed to determine cache directory")?;

        let cache_dir = proj_dirs.cache_dir();
        fs::create_dir_all(cache_dir)?;

        let cache_path = cache_dir.join("usage.json");

        Ok(Self {
            cache_path,
            ttl_minutes,
        })
    }

    pub fn set(&self, data: &UsageData) -> Result<()> {
        let entry = CacheEntry {
            data: data.clone(),
            timestamp: Utc::now(),
        };

        let content = serde_json::to_string_pretty(&entry)?;
        fs::write(&self.cache_path, content)?;

        Ok(())
    }

    pub fn invalidate(&self) -> Result<()> {
        if self.cache_path.exists() {
            fs::remove_file(&self.cache_path)?;
        }
        Ok(())
    }

    /// Returns detailed cache status
    pub fn status(&self) -> CacheStatus {
        if !self.cache_path.exists() {
            return CacheStatus::Missing;
        }

        let content = match fs::read_to_string(&self.cache_path) {
            Ok(c) => c,
            Err(_) => return CacheStatus::Corrupted,
        };

        let entry: CacheEntry = match serde_json::from_str(&content) {
            Ok(e) => e,
            Err(_) => return CacheStatus::Corrupted,
        };

        let age = Utc::now() - entry.timestamp;
        let ttl = Duration::minutes(self.ttl_minutes as i64);

        if age > ttl {
            CacheStatus::Expired
        } else {
            CacheStatus::Fresh(entry.data)
        }
    }

    pub fn last_updated(&self) -> Result<Option<DateTime<Utc>>> {
        if !self.cache_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.cache_path)?;
        let entry: CacheEntry = serde_json::from_str(&content)?;

        Ok(Some(entry.timestamp))
    }
}
