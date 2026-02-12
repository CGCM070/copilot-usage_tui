use anyhow::Result;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::api::{ApiClient, calculate_stats};
use crate::cache::Cache;
use crate::config::ConfigManager;
use crate::models::UsageStats;
use crate::ui::state::CacheInfo;

/// Resultado de una operación async
#[derive(Debug)]
pub enum AsyncResult {
    RefreshComplete(Result<UsageStats>),
    CacheInfoReady(CacheInfo),
}

/// Maneja operaciones asíncronas en background
pub struct AsyncHandler {
    sender: Sender<AsyncResult>,
    receiver: Receiver<AsyncResult>,
}

impl AsyncHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver }
    }

    /// Intenta recibir resultado sin bloquear
    pub fn try_recv(&self) -> Option<AsyncResult> {
        self.receiver.try_recv().ok()
    }

    /// Spawn task para refrescar datos
    pub fn spawn_refresh(&self) {
        let sender = self.sender.clone();

        tokio::spawn(async move {
            let result = Self::do_refresh().await;
            let _ = sender.send(AsyncResult::RefreshComplete(result));
        });
    }

    /// Spawn task para obtener info del cache
    pub fn spawn_cache_info(&self) {
        let sender = self.sender.clone();

        tokio::spawn(async move {
            let info = Self::do_cache_info().await;
            let _ = sender.send(AsyncResult::CacheInfoReady(info));
        });
    }

    /// Implementación real del refresh
    async fn do_refresh() -> Result<UsageStats> {
        let config_manager = ConfigManager::new()?;
        let config = config_manager.load()?.unwrap();
        let cache = Cache::new(config.cache_ttl_minutes)?;

        // Invalidar cache
        cache.invalidate()?;

        // Fetch desde API
        let api_client = ApiClient::new(config.token.clone())?;

        let username = match api_client.get_authenticated_user().await {
            Ok(user) => user,
            Err(_) => {
                // Si falla, devolver error - la TUI lo manejará mostrando error dialog
                anyhow::bail!(
                    "Could not determine username from token. Please reconfigure with a valid token."
                );
            }
        };

        let usage_data = api_client.fetch_usage(&username).await?;
        cache.set(&usage_data)?;

        Ok(calculate_stats(&usage_data))
    }

    /// Implementación real de cache info
    async fn do_cache_info() -> CacheInfo {
        let config_manager = ConfigManager::new().ok();

        if let Some(config_manager) = config_manager {
            if let Ok(Some(config)) = config_manager.load() {
                let cache = Cache::new(config.cache_ttl_minutes).ok();

                if let Some(cache) = cache {
                    let last_updated = cache
                        .last_updated()
                        .ok()
                        .flatten()
                        .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string());

                    let is_fresh = cache.is_fresh();

                    return CacheInfo {
                        last_updated,
                        is_fresh,
                        ttl_minutes: config.cache_ttl_minutes,
                    };
                }
            }
        }

        // Fallback si no hay config
        CacheInfo {
            last_updated: None,
            is_fresh: false,
            ttl_minutes: 5,
        }
    }
}
