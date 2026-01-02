//! Server shared state
//!
//! Holds configuration and shared resources for the HTTP server.

use crate::config::Config;
use crate::qrng::{get_backend_with_key, QrngBackend};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for the HTTP server
pub struct AppState {
    /// Configuration
    pub config: Arc<RwLock<Config>>,

    /// Current QRNG backend
    backend_name: RwLock<String>,
}

impl AppState {
    /// Create new application state
    pub fn new(config: Config) -> Self {
        let backend_name = config.defaults.backend.clone();
        Self {
            config: Arc::new(RwLock::new(config)),
            backend_name: RwLock::new(backend_name),
        }
    }

    /// Get the current QRNG backend
    pub async fn get_backend(&self) -> Box<dyn QrngBackend> {
        let name = self.backend_name.read().await;
        let config = self.config.read().await;

        // Get API key for ANU backend if configured
        let api_key = if name.as_str() == "anu" && !config.api_keys.anu.is_empty() {
            Some(config.api_keys.anu.as_str())
        } else {
            None
        };

        get_backend_with_key(&name, api_key)
    }

    /// Set the current QRNG backend
    pub async fn set_backend(&self, name: &str) {
        let mut backend_name = self.backend_name.write().await;
        *backend_name = name.to_string();
    }

    /// Get current backend name
    pub async fn backend_name(&self) -> String {
        self.backend_name.read().await.clone()
    }
}
