//! Geocoding module
//!
//! Provides geocoding (location name to coordinates) and IP geolocation.

pub mod ip_location;
pub mod nominatim;

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// A geocoded location result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lng: f64,
    /// Display name (address or description)
    pub display_name: String,
}

/// Trait for geocoding backends
pub trait GeoBackend: Send + Sync {
    /// Geocode a location string to coordinates
    ///
    /// Returns the best match for the query, or None if not found
    fn geocode(&self, query: &str) -> impl std::future::Future<Output = Result<Option<GeoLocation>>> + Send;

    /// Reverse geocode coordinates to a location name
    fn reverse_geocode(&self, lat: f64, lng: f64) -> impl std::future::Future<Output = Result<Option<GeoLocation>>> + Send;
}

/// Get the default geocoding backend
pub fn get_geocoder() -> nominatim::NominatimBackend {
    nominatim::NominatimBackend::new()
}

/// Get the IP location service
pub fn get_ip_locator() -> ip_location::IpLocator {
    ip_location::IpLocator::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_location_serialization() {
        let loc = GeoLocation {
            lat: 40.7128,
            lng: -74.0060,
            display_name: "New York City".to_string(),
        };

        let json = serde_json::to_string(&loc).unwrap();
        let parsed: GeoLocation = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.lat, 40.7128);
        assert_eq!(parsed.display_name, "New York City");
    }
}
