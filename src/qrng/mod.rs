//! QRNG (Quantum Random Number Generator) backends
//!
//! This module defines the `QrngBackend` trait and implementations for various
//! random number sources. Each backend is a single file implementing the trait.
//!
//! ## Flex Point
//! Adding a new QRNG backend requires:
//! 1. Create `src/qrng/{backend_name}.rs` implementing `QrngBackend`
//! 2. Add `pub mod {backend_name};` below
//! 3. Register in the backend registry (TODO: implement in config)

pub mod anu;
pub mod pseudo;

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Trait for quantum random number generator backends
///
/// Implementations must be thread-safe (Send + Sync) to work with async server.
pub trait QrngBackend: Send + Sync {
    /// Returns the backend name (e.g., "pseudo", "anu", "rndo")
    fn name(&self) -> &'static str;

    /// Returns a human-readable description of this backend
    fn description(&self) -> &'static str;

    /// Generate n random bytes
    ///
    /// # Arguments
    /// * `n` - Number of bytes to generate
    ///
    /// # Returns
    /// Vec of n random bytes (values 0-255)
    fn bytes(&self, n: usize) -> Result<Vec<u8>>;

    /// Generate a single random float uniformly distributed in [0.0, 1.0)
    ///
    /// Default implementation uses 4 bytes to create a u32, then divides by 2^32.
    fn float(&self) -> Result<f64> {
        let bytes = self.bytes(4)?;
        let u = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(u as f64 / 4_294_967_296.0)
    }

    /// Generate n random floats, each uniformly distributed in [0.0, 1.0)
    ///
    /// Default implementation calls float() n times. Backends may override
    /// for efficiency (e.g., batch API calls).
    fn floats(&self, n: usize) -> Result<Vec<f64>> {
        let bytes = self.bytes(n * 4)?;
        let mut result = Vec::with_capacity(n);
        for i in 0..n {
            let offset = i * 4;
            let u = u32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            result.push(u as f64 / 4_294_967_296.0);
        }
        Ok(result)
    }
}

/// Information about a backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInfo {
    /// Backend name (used in config/API)
    pub name: String,
    /// Human-readable description
    pub description: String,
}

/// Get a backend by name
///
/// Returns the pseudo backend as default if name is not recognized
pub fn get_backend(name: &str) -> Box<dyn QrngBackend> {
    match name {
        "pseudo" => Box::new(pseudo::PseudoBackend::new()),
        "anu" => Box::new(anu::AnuBackend::new()),
        _ => Box::new(pseudo::PseudoBackend::new()), // Default to pseudo
    }
}

/// Get a backend by name with optional API key
pub fn get_backend_with_key(name: &str, api_key: Option<&str>) -> Box<dyn QrngBackend> {
    match name {
        "pseudo" => Box::new(pseudo::PseudoBackend::new()),
        "anu" => {
            if let Some(key) = api_key {
                Box::new(anu::AnuBackend::with_api_key(key))
            } else {
                Box::new(anu::AnuBackend::new())
            }
        }
        _ => Box::new(pseudo::PseudoBackend::new()),
    }
}

/// List all available backends with their info
pub fn available_backends() -> Vec<BackendInfo> {
    vec![
        BackendInfo {
            name: "pseudo".to_string(),
            description: "Pseudo-random number generator (for testing)".to_string(),
        },
        BackendInfo {
            name: "anu".to_string(),
            description: "Australian National University Quantum Random Number Generator".to_string(),
        },
    ]
}
