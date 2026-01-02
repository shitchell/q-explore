//! Entropy quality testing
//!
//! Statistical tests to verify randomness quality of QRNG data.

pub mod tests;

pub use tests::{run_all_tests, EntropyTestResults};
