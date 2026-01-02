//! Nominatim geocoding backend (OpenStreetMap)
//!
//! Uses the free Nominatim API for geocoding.
//! Rate limit: 1 request per second (enforced by User-Agent requirement)

use crate::constants::api::NOMINATIM_URL;
use crate::error::{Error, Result};
use crate::geo::{GeoBackend, GeoLocation};
use serde::Deserialize;

const USER_AGENT: &str = "q-explore/0.1.0";

/// Nominatim geocoding backend
#[derive(Debug, Clone)]
pub struct NominatimBackend {
    client: reqwest::Client,
}

/// Nominatim search response item
#[derive(Debug, Deserialize)]
struct NominatimResult {
    lat: String,
    lon: String,
    display_name: String,
}

impl NominatimBackend {
    /// Create a new Nominatim backend
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }

    /// Parse lat/lng strings to f64
    fn parse_coords(lat: &str, lng: &str) -> Result<(f64, f64)> {
        let lat: f64 = lat.parse().map_err(|_| {
            Error::Geo(format!("Invalid latitude: {}", lat))
        })?;
        let lng: f64 = lng.parse().map_err(|_| {
            Error::Geo(format!("Invalid longitude: {}", lng))
        })?;
        Ok((lat, lng))
    }
}

impl Default for NominatimBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoBackend for NominatimBackend {
    async fn geocode(&self, query: &str) -> Result<Option<GeoLocation>> {
        let url = format!(
            "{}/search?q={}&format=json&limit=1",
            NOMINATIM_URL,
            urlencoding::encode(query)
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Geo(format!("Nominatim request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Geo(format!(
                "Nominatim returned status: {}",
                response.status()
            )));
        }

        let results: Vec<NominatimResult> = response
            .json()
            .await
            .map_err(|e| Error::Geo(format!("Failed to parse Nominatim response: {}", e)))?;

        if let Some(result) = results.into_iter().next() {
            let (lat, lng) = Self::parse_coords(&result.lat, &result.lon)?;
            Ok(Some(GeoLocation {
                lat,
                lng,
                display_name: result.display_name,
            }))
        } else {
            Ok(None)
        }
    }

    async fn reverse_geocode(&self, lat: f64, lng: f64) -> Result<Option<GeoLocation>> {
        let url = format!(
            "{}/reverse?lat={}&lon={}&format=json",
            NOMINATIM_URL, lat, lng
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Geo(format!("Nominatim request failed: {}", e)))?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            return Err(Error::Geo(format!(
                "Nominatim returned status: {}",
                response.status()
            )));
        }

        let result: NominatimResult = response
            .json()
            .await
            .map_err(|e| Error::Geo(format!("Failed to parse Nominatim response: {}", e)))?;

        let (parsed_lat, parsed_lng) = Self::parse_coords(&result.lat, &result.lon)?;
        Ok(Some(GeoLocation {
            lat: parsed_lat,
            lng: parsed_lng,
            display_name: result.display_name,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_coords() {
        let (lat, lng) = NominatimBackend::parse_coords("40.7128", "-74.0060").unwrap();
        assert!((lat - 40.7128).abs() < 0.0001);
        assert!((lng - (-74.0060)).abs() < 0.0001);
    }

    #[test]
    fn test_parse_coords_invalid() {
        assert!(NominatimBackend::parse_coords("invalid", "0").is_err());
        assert!(NominatimBackend::parse_coords("0", "invalid").is_err());
    }

    #[test]
    fn test_backend_creation() {
        let backend = NominatimBackend::new();
        assert!(format!("{:?}", backend).contains("NominatimBackend"));
    }
}
