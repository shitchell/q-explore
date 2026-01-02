//! Configuration management
//!
//! Loads and saves configuration from XDG-compliant paths.
//! Config location: ~/.config/q-explore/config.toml

pub mod defaults;

use crate::error::{Error, Result};
use defaults::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default values for generation
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// Server settings
    #[serde(default)]
    pub server: ServerConfig,

    /// Location settings
    #[serde(default)]
    pub location: LocationConfig,

    /// URL generation settings
    #[serde(default)]
    pub url: UrlConfig,

    /// API keys for various services
    #[serde(default)]
    pub api_keys: ApiKeysConfig,
}

/// Default values for generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default QRNG backend
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Default search radius in meters
    #[serde(default = "default_radius")]
    pub radius: f64,

    /// Default number of points for analysis
    #[serde(default = "default_points")]
    pub points: usize,

    /// Default output format
    #[serde(default = "default_format")]
    pub format: String,

    /// Default anomaly type
    #[serde(rename = "type", default = "default_type")]
    pub anomaly_type: String,

    /// Default generation mode
    #[serde(default = "default_mode")]
    pub mode: String,
}

/// Server settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Shutdown timeout in seconds after last client disconnects
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout_secs: u64,
}

/// Location settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConfig {
    /// If true, --here is default when no location given
    #[serde(default)]
    pub default_here: bool,
}

/// URL generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlConfig {
    /// Default URL provider
    #[serde(default = "default_url_provider")]
    pub default: String,

    /// URL provider templates
    #[serde(default = "default_url_providers")]
    pub providers: HashMap<String, String>,
}

/// API keys for external services
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeysConfig {
    /// ANU QRNG API key
    #[serde(default)]
    pub anu: String,
}

// Default value functions for serde
fn default_backend() -> String {
    DEFAULT_BACKEND.to_string()
}
fn default_radius() -> f64 {
    DEFAULT_RADIUS
}
fn default_points() -> usize {
    DEFAULT_POINTS
}
fn default_format() -> String {
    DEFAULT_FORMAT.to_string()
}
fn default_type() -> String {
    DEFAULT_TYPE.to_string()
}
fn default_mode() -> String {
    DEFAULT_MODE.to_string()
}
fn default_host() -> String {
    DEFAULT_HOST.to_string()
}
fn default_port() -> u16 {
    DEFAULT_PORT
}
fn default_shutdown_timeout() -> u64 {
    DEFAULT_SHUTDOWN_TIMEOUT_SECS
}
fn default_url_provider() -> String {
    DEFAULT_URL_PROVIDER.to_string()
}
fn default_url_providers() -> HashMap<String, String> {
    let mut providers = HashMap::new();
    providers.insert(
        "google".to_string(),
        "https://www.google.com/maps/@{lat},{lng},15z".to_string(),
    );
    providers.insert(
        "openstreetmap".to_string(),
        "https://www.openstreetmap.org/#map=18/{lat}/{lng}".to_string(),
    );
    providers.insert(
        "apple".to_string(),
        "https://maps.apple.com/?ll={lat},{lng}".to_string(),
    );
    providers
}

// Implement Default traits
impl Default for Config {
    fn default() -> Self {
        Self {
            defaults: DefaultsConfig::default(),
            server: ServerConfig::default(),
            location: LocationConfig::default(),
            url: UrlConfig::default(),
            api_keys: ApiKeysConfig::default(),
        }
    }
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            radius: default_radius(),
            points: default_points(),
            format: default_format(),
            anomaly_type: default_type(),
            mode: default_mode(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            shutdown_timeout_secs: default_shutdown_timeout(),
        }
    }
}

impl Default for LocationConfig {
    fn default() -> Self {
        Self { default_here: false }
    }
}

impl Default for UrlConfig {
    fn default() -> Self {
        Self {
            default: default_url_provider(),
            providers: default_url_providers(),
        }
    }
}

impl Config {
    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join(APP_DIR_NAME))
            .ok_or_else(|| Error::Config("Could not determine config directory".to_string()))
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(CONFIG_FILE_NAME))
    }

    /// Load configuration from the default path
    ///
    /// Creates default config if file doesn't exist
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path).map_err(|e| {
                Error::Config(format!("Failed to read config file: {}", e))
            })?;

            toml::from_str(&content).map_err(|e| {
                Error::Config(format!("Failed to parse config file: {}", e))
            })
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to the default path
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Error::Config(format!("Failed to create config directory: {}", e))
            })?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| {
            Error::Config(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(&path, content).map_err(|e| {
            Error::Config(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    /// Get a configuration value by key path
    ///
    /// Key format: "section.key" or just "key" for top-level
    /// Returns the value as a string, or None if not found
    pub fn get(&self, key: &str) -> Option<String> {
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["defaults", "backend"] => Some(self.defaults.backend.clone()),
            ["defaults", "radius"] => Some(self.defaults.radius.to_string()),
            ["defaults", "points"] => Some(self.defaults.points.to_string()),
            ["defaults", "format"] => Some(self.defaults.format.clone()),
            ["defaults", "type"] => Some(self.defaults.anomaly_type.clone()),
            ["defaults", "mode"] => Some(self.defaults.mode.clone()),

            ["server", "host"] => Some(self.server.host.clone()),
            ["server", "port"] => Some(self.server.port.to_string()),
            ["server", "shutdown_timeout_secs"] => {
                Some(self.server.shutdown_timeout_secs.to_string())
            }

            ["location", "default_here"] => Some(self.location.default_here.to_string()),

            ["url", "default"] => Some(self.url.default.clone()),

            ["api_keys", "anu"] => Some(self.api_keys.anu.clone()),

            _ => None,
        }
    }

    /// Set a configuration value by key path
    ///
    /// Key format: "section.key"
    /// Returns error if key is invalid or value type is wrong
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["defaults", "backend"] => {
                self.defaults.backend = value.to_string();
            }
            ["defaults", "radius"] => {
                self.defaults.radius = value.parse().map_err(|_| {
                    Error::Config(format!("Invalid radius value: {}", value))
                })?;
            }
            ["defaults", "points"] => {
                self.defaults.points = value.parse().map_err(|_| {
                    Error::Config(format!("Invalid points value: {}", value))
                })?;
            }
            ["defaults", "format"] => {
                self.defaults.format = value.to_string();
            }
            ["defaults", "type"] => {
                self.defaults.anomaly_type = value.to_string();
            }
            ["defaults", "mode"] => {
                self.defaults.mode = value.to_string();
            }

            ["server", "host"] => {
                self.server.host = value.to_string();
            }
            ["server", "port"] => {
                self.server.port = value.parse().map_err(|_| {
                    Error::Config(format!("Invalid port value: {}", value))
                })?;
            }
            ["server", "shutdown_timeout_secs"] => {
                self.server.shutdown_timeout_secs = value.parse().map_err(|_| {
                    Error::Config(format!("Invalid timeout value: {}", value))
                })?;
            }

            ["location", "default_here"] => {
                self.location.default_here = value.parse().map_err(|_| {
                    Error::Config(format!("Invalid boolean value: {}", value))
                })?;
            }

            ["url", "default"] => {
                self.url.default = value.to_string();
            }

            ["api_keys", "anu"] => {
                self.api_keys.anu = value.to_string();
            }

            _ => {
                return Err(Error::Config(format!("Unknown config key: {}", key)));
            }
        }

        Ok(())
    }

    /// List all available config keys
    pub fn available_keys() -> Vec<&'static str> {
        vec![
            "defaults.backend",
            "defaults.radius",
            "defaults.points",
            "defaults.format",
            "defaults.type",
            "defaults.mode",
            "server.host",
            "server.port",
            "server.shutdown_timeout_secs",
            "location.default_here",
            "url.default",
            "api_keys.anu",
        ]
    }

    /// Format a URL using the specified provider
    ///
    /// Replaces {lat} and {lng} placeholders with actual values
    pub fn format_url(&self, provider: Option<&str>, lat: f64, lng: f64) -> Result<String> {
        let provider_name = provider.unwrap_or(&self.url.default);

        let template = self.url.providers.get(provider_name).ok_or_else(|| {
            Error::Config(format!("Unknown URL provider: {}", provider_name))
        })?;

        Ok(template
            .replace("{lat}", &lat.to_string())
            .replace("{lng}", &lng.to_string()))
    }

    /// Get server address as "host:port"
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn with_temp_config<F: FnOnce()>(f: F) {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        f();
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.defaults.backend, "pseudo");
        assert_eq!(config.defaults.radius, 3000.0);
        assert_eq!(config.defaults.points, 10_000);
        assert_eq!(config.server.port, 7878);
    }

    #[test]
    fn test_get_set() {
        let mut config = Config::default();

        assert_eq!(config.get("defaults.backend"), Some("pseudo".to_string()));

        config.set("defaults.backend", "anu").unwrap();
        assert_eq!(config.get("defaults.backend"), Some("anu".to_string()));

        config.set("defaults.radius", "5000").unwrap();
        assert_eq!(config.get("defaults.radius"), Some("5000".to_string()));
        assert_eq!(config.defaults.radius, 5000.0);
    }

    #[test]
    fn test_get_invalid_key() {
        let config = Config::default();
        assert_eq!(config.get("invalid.key"), None);
    }

    #[test]
    fn test_set_invalid_key() {
        let mut config = Config::default();
        let result = config.set("invalid.key", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_invalid_value() {
        let mut config = Config::default();
        let result = config.set("defaults.radius", "not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_url() {
        let config = Config::default();

        let url = config.format_url(Some("google"), 40.7128, -74.0060).unwrap();
        assert_eq!(url, "https://www.google.com/maps/@40.7128,-74.006,15z");

        let url = config
            .format_url(Some("openstreetmap"), 40.7128, -74.0060)
            .unwrap();
        assert_eq!(url, "https://www.openstreetmap.org/#map=18/40.7128/-74.006");
    }

    #[test]
    fn test_format_url_default_provider() {
        let config = Config::default();
        let url = config.format_url(None, 40.7128, -74.0060).unwrap();
        assert!(url.contains("google.com"));
    }

    #[test]
    fn test_format_url_unknown_provider() {
        let config = Config::default();
        let result = config.format_url(Some("unknown"), 40.7128, -74.0060);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load() {
        with_temp_config(|| {
            let mut config = Config::default();
            config.defaults.backend = "anu".to_string();
            config.defaults.radius = 5000.0;
            config.save().unwrap();

            let loaded = Config::load().unwrap();
            assert_eq!(loaded.defaults.backend, "anu");
            assert_eq!(loaded.defaults.radius, 5000.0);
        });
    }

    #[test]
    fn test_config_roundtrip() {
        // Test that a default config can be serialized and deserialized
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.defaults.backend, "pseudo");
        assert_eq!(loaded.defaults.radius, 3000.0);
        assert_eq!(loaded.server.port, 7878);
    }

    #[test]
    fn test_serialization_format() {
        let config = Config::default();
        let toml = toml::to_string_pretty(&config).unwrap();

        // Check that key sections exist
        assert!(toml.contains("[defaults]"));
        assert!(toml.contains("[server]"));
        assert!(toml.contains("[url]"));
        assert!(toml.contains("[url.providers]"));
    }

    #[test]
    fn test_server_addr() {
        let config = Config::default();
        assert_eq!(config.server_addr(), "127.0.0.1:7878");
    }

    #[test]
    fn test_available_keys() {
        let keys = Config::available_keys();
        assert!(keys.contains(&"defaults.backend"));
        assert!(keys.contains(&"server.port"));
        assert!(keys.contains(&"url.default"));
    }
}
