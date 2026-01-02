//! ANU Quantum Random Number Generator backend
//!
//! Uses the Australian National University's QRNG API to get truly random numbers.
//! API documentation: https://qrng.anu.edu.au/contact/api-documentation/
//!
//! Two tiers:
//! - Free: https://qrng.anu.edu.au/API/jsonI.php (rate limited, expired SSL cert)
//! - Paid: https://api.quantumnumbers.anu.edu.au (requires API key)
//!
//! Both tiers support hex16 format with size=10 for maximum throughput:
//! 1024 values × 20 bytes = 20KB per request.

use crate::constants::api::{ANU_FREE_URL, ANU_PAID_URL};
use crate::error::{Error, Result};
use crate::qrng::QrngBackend;
use serde::Deserialize;
use std::sync::mpsc;
use std::thread;

const MAX_ARRAY_LENGTH: usize = 1024; // Maximum array length per request

// hex16 with size=10 gives 40 hex chars = 20 bytes per element
const HEX16_BLOCK_SIZE: usize = 10; // Max allowed by API
const BYTES_PER_HEX16_ELEMENT: usize = HEX16_BLOCK_SIZE * 2; // 20 bytes (each block = 4 hex chars = 2 bytes)
const BYTES_PER_REQUEST: usize = MAX_ARRAY_LENGTH * BYTES_PER_HEX16_ELEMENT; // 20,480 bytes

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

/// ANU API success response for hex16 format
///
/// Example: `{"success": true, "type": "hex16", "length": "5", "data": ["b580bb5ec3bd0d97d367...", ...]}`
#[derive(Debug, Deserialize)]
struct AnuResponse {
    success: bool,
    #[serde(default)]
    data: Option<Vec<String>>,
    #[allow(dead_code)]
    r#type: Option<String>,
    #[allow(dead_code)]
    length: Option<String>, // API returns this as a string
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
    /// Uses hex16 with size=10 for maximum throughput (20,480 bytes per request).
    /// Runs HTTP request in a separate thread to avoid tokio runtime conflicts.
    fn fetch_bytes(&self, count: usize) -> Result<Vec<u8>> {
        // Calculate how many hex16 elements we need (each gives us 20 bytes with size=10)
        // Request up to MAX_ARRAY_LENGTH elements
        let element_count = ((count + BYTES_PER_HEX16_ELEMENT - 1) / BYTES_PER_HEX16_ELEMENT)
            .min(MAX_ARRAY_LENGTH);

        // Select endpoint based on whether we have an API key
        let (url, api_key) = match &self.api_key {
            Some(key) if !key.is_empty() => {
                // Paid endpoint uses header auth
                (
                    format!(
                        "{}?length={}&type=hex16&size={}",
                        ANU_PAID_URL, element_count, HEX16_BLOCK_SIZE
                    ),
                    Some(key.clone()),
                )
            }
            _ => {
                // Free endpoint, no auth needed
                (
                    format!(
                        "{}?length={}&type=hex16&size={}",
                        ANU_FREE_URL, element_count, HEX16_BLOCK_SIZE
                    ),
                    None,
                )
            }
        };

        // Run the HTTP request in a separate thread to avoid tokio runtime conflicts
        let (tx, rx) = mpsc::channel();
        let is_free_tier = api_key.is_none();

        thread::spawn(move || {
            let result = (|| -> Result<Vec<u8>> {
                let mut client_builder = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(30));

                // SECURITY WARNING: Free tier API has expired SSL cert.
                // This disables certificate verification, making MitM attacks possible.
                // For production use, upgrade to paid tier or use another backend.
                if is_free_tier {
                    eprintln!(
                        "Warning: ANU free tier has expired SSL cert - verification disabled. \
                         Consider using paid tier for production."
                    );
                    client_builder = client_builder.danger_accept_invalid_certs(true);
                }

                let client = client_builder
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

                // Convert hex strings to bytes
                // Each element is 40 hex chars = 20 bytes (with size=10)
                let hex_data = anu_response
                    .data
                    .ok_or_else(|| Error::Qrng("ANU API returned no data".to_string()))?;

                let mut bytes = Vec::with_capacity(hex_data.len() * BYTES_PER_HEX16_ELEMENT);
                for hex_str in hex_data {
                    // Parse pairs of hex characters into bytes
                    for i in (0..hex_str.len()).step_by(2) {
                        if i + 2 <= hex_str.len() {
                            let byte = u8::from_str_radix(&hex_str[i..i + 2], 16).map_err(|e| {
                                Error::Qrng(format!(
                                    "Failed to parse hex '{}': {}",
                                    &hex_str[i..i + 2],
                                    e
                                ))
                            })?;
                            bytes.push(byte);
                        }
                    }
                }

                Ok(bytes)
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
        // Each call fetches up to BYTES_PER_REQUEST bytes (20,480 with hex16 size=10)
        let mut result = Vec::with_capacity(count);
        let mut remaining = count;

        while remaining > 0 {
            let batch_size = remaining.min(BYTES_PER_REQUEST);
            let bytes = self.fetch_bytes(batch_size)?;
            result.extend(&bytes[..bytes.len().min(remaining)]);
            remaining = remaining.saturating_sub(bytes.len());
        }

        // Truncate to exact count requested (in case we got extra from u16 rounding)
        result.truncate(count);
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
    // Run with: cargo test qrng::anu -- --ignored
    // Set ANU_API_KEY env var to use paid tier (avoids rate limiting)

    /// Get backend with API key from environment if available
    fn get_test_backend() -> AnuBackend {
        match std::env::var("ANU_API_KEY") {
            Ok(key) if !key.is_empty() => AnuBackend::with_api_key(key),
            _ => AnuBackend::new(),
        }
    }

    #[test]
    #[ignore = "Requires network access to ANU API"]
    fn test_anu_fetch_bytes() {
        let backend = get_test_backend();
        let bytes = backend.bytes(10).unwrap();
        assert_eq!(bytes.len(), 10);
    }

    #[test]
    #[ignore = "Requires network access to ANU API"]
    fn test_anu_fetch_floats() {
        let backend = get_test_backend();
        let floats = backend.floats(10).unwrap();
        assert_eq!(floats.len(), 10);
        for f in &floats {
            assert!(*f >= 0.0 && *f <= 1.0);
        }
    }

    #[test]
    #[ignore = "Requires network access to ANU API"]
    fn test_anu_large_request() {
        let backend = get_test_backend();
        // Request more than BYTES_PER_REQUEST (20,480) to test batching
        let bytes = backend.bytes(25000).unwrap();
        assert_eq!(bytes.len(), 25000);
    }
}
