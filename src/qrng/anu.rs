//! ANU Quantum Random Number Generator backend
//!
//! Uses the Australian National University's QRNG API to get truly random numbers.
//! API documentation: https://qrng.anu.edu.au/contact/api-documentation/
//!
//! Two tiers:
//! - Free: https://qrng.anu.edu.au/API/jsonI.php (rate limited)
//! - Paid: https://api.quantumnumbers.anu.edu.au (requires API key)
//!
//! If an API key is provided, the paid endpoint is used automatically.

use crate::error::{Error, Result};
use crate::qrng::QrngBackend;
use serde::Deserialize;
use std::sync::mpsc;
use std::thread;

const ANU_FREE_URL: &str = "https://qrng.anu.edu.au/API/jsonI.php";
const ANU_PAID_URL: &str = "https://api.quantumnumbers.anu.edu.au";
const MAX_BLOCK_SIZE: usize = 1024; // Maximum bytes per request

/// ANU QRNG backend
///
/// Note: This backend runs HTTP requests in a separate thread to avoid
/// conflicts with tokio's async runtime.
#[derive(Debug)]
pub struct AnuBackend {
    api_key: Option<String>,
}

/// Which API tier is being used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnuTier {
    Free,
    Paid,
}

/// ANU API success response for uint8 type
///
/// Example: `{"success": true, "type": "uint8", "length": "5", "data": [172, 216, 180, 138, 46]}`
#[derive(Debug, Deserialize)]
struct AnuResponse {
    success: bool,
    #[serde(default)]
    data: Option<Vec<u8>>,
    #[allow(dead_code)]
    r#type: Option<String>,
    #[allow(dead_code)]
    length: Option<String>,
    /// Present on error responses: `{"success": false, "message": "..."}`
    #[serde(default)]
    message: Option<String>,
}

impl AnuBackend {
    /// Create a new ANU backend
    pub fn new() -> Self {
        Self { api_key: None }
    }

    /// Create a new ANU backend with an API key
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Get which API tier is being used
    pub fn tier(&self) -> AnuTier {
        match &self.api_key {
            Some(key) if !key.is_empty() => AnuTier::Paid,
            _ => AnuTier::Free,
        }
    }

    /// Fetch random bytes from the ANU API
    ///
    /// Runs HTTP request in a separate thread to avoid tokio runtime conflicts.
    fn fetch_bytes(&self, count: usize) -> Result<Vec<u8>> {
        let count = count.min(MAX_BLOCK_SIZE);

        // Select endpoint based on whether we have an API key
        let (url, api_key) = match &self.api_key {
            Some(key) if !key.is_empty() => {
                // Paid endpoint uses header auth
                (
                    format!("{}?length={}&type=uint8", ANU_PAID_URL, count),
                    Some(key.clone()),
                )
            }
            _ => {
                // Free endpoint, no auth needed
                (
                    format!("{}?length={}&type=uint8", ANU_FREE_URL, count),
                    None,
                )
            }
        };

        // Run the HTTP request in a separate thread to avoid tokio runtime conflicts
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = (|| -> Result<Vec<u8>> {
                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .map_err(|e| Error::Qrng(format!("Failed to build HTTP client: {}", e)))?;

                let mut request = client.get(&url);

                // Add API key header for paid endpoint
                if let Some(key) = api_key {
                    request = request.header("x-api-key", key);
                }

                let response = request
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
                    let msg = anu_response
                        .message
                        .unwrap_or_else(|| "Unknown error".to_string());
                    return Err(Error::Qrng(format!("ANU API error: {}", msg)));
                }

                anu_response
                    .data
                    .ok_or_else(|| Error::Qrng("ANU API returned no data".to_string()))
            })();

            let _ = tx.send(result);
        });

        rx.recv()
            .map_err(|_| Error::Qrng("Failed to receive response from HTTP thread".to_string()))?
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

    #[test]
    fn test_anu_tier_free() {
        let backend = AnuBackend::new();
        assert_eq!(backend.tier(), AnuTier::Free);

        // Empty API key should also be free tier
        let backend_empty = AnuBackend::with_api_key("");
        assert_eq!(backend_empty.tier(), AnuTier::Free);
    }

    #[test]
    fn test_anu_tier_paid() {
        let backend = AnuBackend::with_api_key("my-api-key");
        assert_eq!(backend.tier(), AnuTier::Paid);
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
