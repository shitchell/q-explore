//! ANU Quantum Random Number Generator backend
//!
//! Uses the Australian National University's QRNG API to get truly random numbers.
//! API documentation: https://qrng.anu.edu.au/contact/api-documentation/
//!
//! Note: The ANU QRNG API has rate limits. For production use, consider
//! implementing a local cache of quantum random bytes.

use crate::error::{Error, Result};
use crate::qrng::QrngBackend;
use serde::Deserialize;

const ANU_API_URL: &str = "https://qrng.anu.edu.au/API/jsonI.php";
const MAX_BLOCK_SIZE: usize = 1024; // Maximum bytes per request

/// ANU QRNG backend
#[derive(Debug)]
pub struct AnuBackend {
    client: reqwest::blocking::Client,
    api_key: Option<String>,
}

/// ANU API response
#[derive(Debug, Deserialize)]
struct AnuResponse {
    success: bool,
    data: Option<Vec<u8>>,
    #[allow(dead_code)]
    r#type: Option<String>,
    #[allow(dead_code)]
    length: Option<usize>,
}

impl AnuBackend {
    /// Create a new ANU backend
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            api_key: None,
        }
    }

    /// Create a new ANU backend with an API key
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            api_key: Some(api_key.into()),
        }
    }

    /// Fetch random bytes from the ANU API
    fn fetch_bytes(&self, count: usize) -> Result<Vec<u8>> {
        let count = count.min(MAX_BLOCK_SIZE);

        let mut url = format!(
            "{}?length={}&type=uint8",
            ANU_API_URL, count
        );

        // Add API key if available
        if let Some(ref key) = self.api_key {
            if !key.is_empty() {
                url.push_str(&format!("&api_key={}", key));
            }
        }

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| Error::Qrng(format!("ANU API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Qrng(format!(
                "ANU API returned status: {}",
                response.status()
            )));
        }

        let anu_response: AnuResponse = response
            .json()
            .map_err(|e| Error::Qrng(format!("Failed to parse ANU response: {}", e)))?;

        if !anu_response.success {
            return Err(Error::Qrng("ANU API returned failure status".to_string()));
        }

        anu_response
            .data
            .ok_or_else(|| Error::Qrng("ANU API returned no data".to_string()))
    }
}

impl Default for AnuBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl QrngBackend for AnuBackend {
    fn name(&self) -> &'static str {
        "anu"
    }

    fn description(&self) -> &'static str {
        "Australian National University Quantum Random Number Generator"
    }

    fn bytes(&self, count: usize) -> Result<Vec<u8>> {
        if count == 0 {
            return Ok(Vec::new());
        }

        // For large requests, make multiple API calls
        let mut result = Vec::with_capacity(count);
        let mut remaining = count;

        while remaining > 0 {
            let batch_size = remaining.min(MAX_BLOCK_SIZE);
            let bytes = self.fetch_bytes(batch_size)?;
            result.extend(bytes);
            remaining = remaining.saturating_sub(batch_size);
        }

        Ok(result)
    }

    fn floats(&self, count: usize) -> Result<Vec<f64>> {
        if count == 0 {
            return Ok(Vec::new());
        }

        // Need 8 bytes per f64
        let bytes = self.bytes(count * 8)?;
        let mut floats = Vec::with_capacity(count);

        for chunk in bytes.chunks_exact(8) {
            let value = u64::from_le_bytes(chunk.try_into().unwrap());
            let float = (value as f64) / (u64::MAX as f64);
            floats.push(float);
        }

        Ok(floats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anu_backend_creation() {
        let backend = AnuBackend::new();
        assert_eq!(backend.name(), "anu");
        assert!(backend.api_key.is_none());
    }

    #[test]
    fn test_anu_backend_with_api_key() {
        let backend = AnuBackend::with_api_key("test_key");
        assert!(backend.api_key.is_some());
        assert_eq!(backend.api_key.as_deref(), Some("test_key"));
    }

    #[test]
    fn test_anu_backend_description() {
        let backend = AnuBackend::new();
        assert!(backend.description().contains("Australian National University"));
    }

    // Integration tests - these actually call the ANU API
    // They are disabled by default as they require network access
    // and may be rate-limited
    #[test]
    #[ignore = "Requires network access to ANU API"]
    fn test_anu_fetch_bytes() {
        let backend = AnuBackend::new();
        let bytes = backend.bytes(10).unwrap();
        assert_eq!(bytes.len(), 10);
    }

    #[test]
    #[ignore = "Requires network access to ANU API"]
    fn test_anu_fetch_floats() {
        let backend = AnuBackend::new();
        let floats = backend.floats(10).unwrap();
        assert_eq!(floats.len(), 10);
        for f in &floats {
            assert!(*f >= 0.0 && *f <= 1.0);
        }
    }

    #[test]
    #[ignore = "Requires network access to ANU API"]
    fn test_anu_large_request() {
        let backend = AnuBackend::new();
        // Request more than MAX_BLOCK_SIZE to test batching
        let bytes = backend.bytes(2048).unwrap();
        assert_eq!(bytes.len(), 2048);
    }
}
