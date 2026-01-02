//! Pseudo-random backend for testing
//!
//! Uses the `rand` crate's thread-local RNG. This is NOT quantum random,
//! but provides a fast, deterministic-when-seeded backend for development
//! and testing.

use crate::error::Result;
use crate::qrng::QrngBackend;
use rand::RngCore;
use std::sync::Mutex;

/// Pseudo-random number generator backend
///
/// Thread-safe wrapper around rand's ThreadRng.
pub struct PseudoBackend {
    // We use a Mutex to make the RNG Send + Sync
    // In practice, ThreadRng is already thread-local, but we need
    // to satisfy the trait bounds
    _phantom: std::marker::PhantomData<()>,
}

impl PseudoBackend {
    /// Create a new pseudo-random backend
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Default for PseudoBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl QrngBackend for PseudoBackend {
    fn name(&self) -> &'static str {
        "pseudo"
    }

    fn description(&self) -> &'static str {
        "Pseudo-random number generator (for testing)"
    }

    fn bytes(&self, n: usize) -> Result<Vec<u8>> {
        let mut bytes = vec![0u8; n];
        rand::thread_rng().fill_bytes(&mut bytes);
        Ok(bytes)
    }

    // Override for efficiency - generate all floats at once
    fn floats(&self, n: usize) -> Result<Vec<f64>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Ok((0..n).map(|_| rng.gen::<f64>()).collect())
    }
}

/// Seeded pseudo-random backend for deterministic testing
pub struct SeededPseudoBackend {
    rng: Mutex<rand::rngs::StdRng>,
}

impl SeededPseudoBackend {
    /// Create a new seeded pseudo-random backend
    ///
    /// Using the same seed will produce the same sequence of random values.
    pub fn new(seed: u64) -> Self {
        use rand::SeedableRng;
        Self {
            rng: Mutex::new(rand::rngs::StdRng::seed_from_u64(seed)),
        }
    }
}

impl QrngBackend for SeededPseudoBackend {
    fn name(&self) -> &'static str {
        "pseudo-seeded"
    }

    fn description(&self) -> &'static str {
        "Seeded pseudo-random number generator (for reproducible testing)"
    }

    fn bytes(&self, n: usize) -> Result<Vec<u8>> {
        let mut bytes = vec![0u8; n];
        let mut rng = self.rng.lock().unwrap();
        rng.fill_bytes(&mut bytes);
        Ok(bytes)
    }

    fn floats(&self, n: usize) -> Result<Vec<f64>> {
        use rand::Rng;
        let mut rng = self.rng.lock().unwrap();
        Ok((0..n).map(|_| rng.gen::<f64>()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pseudo_backend_bytes() {
        let backend = PseudoBackend::new();
        let bytes = backend.bytes(100).unwrap();
        assert_eq!(bytes.len(), 100);
    }

    #[test]
    fn test_pseudo_backend_floats() {
        let backend = PseudoBackend::new();
        let floats = backend.floats(100).unwrap();
        assert_eq!(floats.len(), 100);
        for f in &floats {
            assert!(*f >= 0.0 && *f < 1.0);
        }
    }

    #[test]
    fn test_seeded_backend_reproducible() {
        let backend1 = SeededPseudoBackend::new(42);
        let backend2 = SeededPseudoBackend::new(42);

        let bytes1 = backend1.bytes(100).unwrap();
        let bytes2 = backend2.bytes(100).unwrap();

        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_seeded_backend_floats_in_range() {
        let backend = SeededPseudoBackend::new(12345);
        let floats = backend.floats(1000).unwrap();

        for f in &floats {
            assert!(*f >= 0.0 && *f < 1.0, "Float {} out of range [0, 1)", f);
        }
    }
}
