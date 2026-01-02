//! IP-based geolocation
//!
//! Uses ip-api.com for IP geolocation with file-based caching.

use crate::constants::api::IP_API_URL;
use crate::constants::cache::{IP_LOCATION_CACHE_FILE, IP_LOCATION_TTL_SECS};
use crate::error::{Error, Result};
use crate::geo::GeoLocation;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// IP location service with caching
#[derive(Debug)]
pub struct IpLocator {
    client: reqwest::Client,
    cache_path: Option<PathBuf>,
}

/// ip-api.com response
#[derive(Debug, Deserialize)]
struct IpApiResponse {
    status: String,
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    country: Option<String>,
}

/// Cached location data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedLocation {
    location: GeoLocation,
    timestamp: u64,
}

impl IpLocator {
    /// Create a new IP locator with default cache path
    pub fn new() -> Self {
        let cache_path = dirs::cache_dir()
            .map(|p| p.join("q-explore").join(IP_LOCATION_CACHE_FILE));

        Self {
            client: reqwest::Client::new(),
            cache_path,
        }
    }

    /// Create an IP locator with a specific cache path
    pub fn with_cache_path(cache_path: PathBuf) -> Self {
        Self {
            client: reqwest::Client::new(),
            cache_path: Some(cache_path),
        }
    }

    /// Create an IP locator without caching
    pub fn without_cache() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache_path: None,
        }
    }

    /// Get current location based on IP address
    pub async fn locate(&self) -> Result<GeoLocation> {
        // Check cache first
        if let Some(cached) = self.load_cache() {
            return Ok(cached);
        }

        // Fetch from API
        let location = self.fetch_location().await?;

        // Save to cache
        self.save_cache(&location);

        Ok(location)
    }

    /// Fetch location from ip-api.com
    async fn fetch_location(&self) -> Result<GeoLocation> {
        let response = self.client
            .get(IP_API_URL)
            .send()
            .await
            .map_err(|e| Error::Geo(format!("IP location request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Geo(format!(
                "IP location API returned status: {}",
                response.status()
            )));
        }

        let data: IpApiResponse = response
            .json()
            .await
            .map_err(|e| Error::Geo(format!("Failed to parse IP location response: {}", e)))?;

        if data.status != "success" {
            return Err(Error::Geo("IP location lookup failed".to_string()));
        }

        let lat = data.lat.ok_or_else(|| Error::Geo("No latitude in response".to_string()))?;
        let lng = data.lon.ok_or_else(|| Error::Geo("No longitude in response".to_string()))?;

        // Build display name from available fields
        let display_name = [data.city, data.region_name, data.country]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(", ");

        Ok(GeoLocation {
            lat,
            lng,
            display_name: if display_name.is_empty() {
                "Unknown Location".to_string()
            } else {
                display_name
            },
        })
    }

    /// Load cached location if valid
    fn load_cache(&self) -> Option<GeoLocation> {
        let cache_path = self.cache_path.as_ref()?;

        if !cache_path.exists() {
            return None;
        }

        let content = fs::read_to_string(cache_path).ok()?;
        let cached: CachedLocation = serde_json::from_str(&content).ok()?;

        // Check if cache is still valid
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs();

        if now - cached.timestamp < IP_LOCATION_TTL_SECS {
            Some(cached.location)
        } else {
            None
        }
    }

    /// Save location to cache
    fn save_cache(&self, location: &GeoLocation) {
        let Some(cache_path) = &self.cache_path else {
            return;
        };

        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let cached = CachedLocation {
            location: location.clone(),
            timestamp,
        };

        if let Ok(content) = serde_json::to_string_pretty(&cached) {
            let _ = fs::write(cache_path, content);
        }
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        if let Some(cache_path) = &self.cache_path {
            let _ = fs::remove_file(cache_path);
        }
    }

    /// Get cache duration
    pub fn cache_duration() -> Duration {
        Duration::from_secs(IP_LOCATION_TTL_SECS)
    }
}

impl Default for IpLocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ip_locator_creation() {
        let locator = IpLocator::new();
        assert!(locator.cache_path.is_some());
    }

    #[test]
    fn test_ip_locator_without_cache() {
        let locator = IpLocator::without_cache();
        assert!(locator.cache_path.is_none());
    }

    #[test]
    fn test_cache_operations() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");
        let locator = IpLocator::with_cache_path(cache_path.clone());

        // Initially no cache
        assert!(locator.load_cache().is_none());

        // Save a location
        let location = GeoLocation {
            lat: 40.7128,
            lng: -74.0060,
            display_name: "New York".to_string(),
        };
        locator.save_cache(&location);

        // Load should work now
        let loaded = locator.load_cache().unwrap();
        assert_eq!(loaded.lat, 40.7128);
        assert_eq!(loaded.display_name, "New York");

        // Clear cache
        locator.clear_cache();
        assert!(locator.load_cache().is_none());
    }

    #[test]
    fn test_cache_duration() {
        assert_eq!(IpLocator::cache_duration().as_secs(), 3600);
    }

    #[test]
    fn test_cached_location_serialization() {
        let cached = CachedLocation {
            location: GeoLocation {
                lat: 40.7128,
                lng: -74.0060,
                display_name: "NYC".to_string(),
            },
            timestamp: 1704200000,
        };

        let json = serde_json::to_string(&cached).unwrap();
        let parsed: CachedLocation = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.location.lat, 40.7128);
        assert_eq!(parsed.timestamp, 1704200000);
    }
}
