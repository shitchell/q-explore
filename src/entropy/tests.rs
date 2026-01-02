//! Statistical tests for entropy quality
//!
//! Implements the "Balanced", "Uniform", and "Scattered" tests from Randonautica:
//! - Balanced (Monobit): Checks if 0s and 1s are roughly equal
//! - Uniform (Chi-Square): Checks if byte values are uniformly distributed
//! - Scattered (Runs): Checks for patterns/clusters in the bit sequence

use serde::{Deserialize, Serialize};

/// Threshold for considering a test "passed"
/// Values closer to 1.0 indicate better randomness
pub const PASS_THRESHOLD: f64 = 0.01;

/// Results of entropy quality tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyTestResults {
    /// Monobit test result (0-1, higher is better)
    /// Measures balance between 0s and 1s
    pub balanced: f64,

    /// Chi-square test result (0-1, higher is better)
    /// Measures uniformity of byte distribution
    pub uniform: f64,

    /// Runs test result (0-1, higher is better)
    /// Measures randomness of bit transitions
    pub scattered: f64,

    /// Overall quality (average of all tests)
    pub overall: f64,

    /// Number of bytes analyzed
    pub bytes_analyzed: usize,
}

impl EntropyTestResults {
    /// Check if all tests pass the threshold
    pub fn all_passed(&self) -> bool {
        self.balanced >= PASS_THRESHOLD
            && self.uniform >= PASS_THRESHOLD
            && self.scattered >= PASS_THRESHOLD
    }
}

/// Run all entropy tests on the given data
///
/// # Arguments
/// * `data` - Random bytes to test
///
/// # Returns
/// EntropyTestResults with scores for each test (0-1, higher is better)
pub fn run_all_tests(data: &[u8]) -> EntropyTestResults {
    let balanced = monobit_test(data);
    let uniform = chi_square_test(data);
    let scattered = runs_test(data);

    let overall = (balanced + uniform + scattered) / 3.0;

    EntropyTestResults {
        balanced,
        uniform,
        scattered,
        overall,
        bytes_analyzed: data.len(),
    }
}

/// Monobit (Frequency) Test - "Balanced"
///
/// Checks if the proportion of 0s and 1s is close to 50/50.
/// Returns a p-value like score (0-1, higher indicates better randomness).
pub fn monobit_test(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let total_bits = data.len() * 8;
    let ones = data.iter().map(|b| b.count_ones() as u64).sum::<u64>();

    // Expected: half ones, half zeros
    let expected = total_bits as f64 / 2.0;

    // Calculate the deviation
    let diff = (ones as f64 - expected).abs();

    // Standard deviation for binomial distribution
    let std_dev = (total_bits as f64 / 4.0).sqrt();

    // Z-score
    let z = diff / std_dev;

    // Convert to a 0-1 score using error function approximation
    // Lower z-score = better randomness = higher score
    let p = 1.0 - erf(z / std::f64::consts::SQRT_2);

    p.clamp(0.0, 1.0)
}

/// Chi-Square Test - "Uniform"
///
/// Tests if byte values (0-255) appear with uniform frequency.
/// Returns a p-value like score (0-1, higher indicates better uniformity).
pub fn chi_square_test(data: &[u8]) -> f64 {
    if data.len() < 256 {
        // Not enough data for meaningful chi-square test
        return 0.0;
    }

    // Count occurrences of each byte value
    let mut counts = [0u64; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }

    // Expected count for uniform distribution
    let expected = data.len() as f64 / 256.0;

    // Calculate chi-square statistic
    let chi_sq: f64 = counts
        .iter()
        .map(|&count| {
            let diff = count as f64 - expected;
            diff * diff / expected
        })
        .sum();

    // Degrees of freedom = 255 (256 categories - 1)
    // For df=255, mean=255, std_dev=sqrt(2*255) ≈ 22.6

    // Approximate p-value using normal approximation for large df
    let z = (chi_sq - 255.0) / (2.0 * 255.0_f64).sqrt();

    // Convert to 0-1 score (lower chi-sq deviation = better = higher score)
    let p = 1.0 - erf(z.abs() / std::f64::consts::SQRT_2);

    p.clamp(0.0, 1.0)
}

/// Runs Test - "Scattered"
///
/// Counts runs (consecutive sequences of same bit) and compares to expected.
/// Detects patterns or clustering in the bit sequence.
/// Returns a p-value like score (0-1, higher indicates better randomness).
pub fn runs_test(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let total_bits = data.len() * 8;

    // Count ones and runs
    let mut ones = 0u64;
    let mut runs = 1u64; // Start with 1 run
    let mut prev_bit = (data[0] >> 7) & 1;
    ones += prev_bit as u64;

    for (i, &byte) in data.iter().enumerate() {
        for j in (0..8).rev().skip(if i == 0 { 1 } else { 0 }) {
            let bit = (byte >> j) & 1;
            ones += bit as u64;
            if bit != prev_bit {
                runs += 1;
                prev_bit = bit;
            }
        }
    }

    let zeros = total_bits as u64 - ones;
    let n = total_bits as f64;
    let pi = ones as f64 / n;

    // Check if proportions are too extreme for the test
    if pi < 0.01 || pi > 0.99 {
        return 0.0;
    }

    // Expected number of runs
    let expected_runs = 2.0 * (ones as f64) * (zeros as f64) / n + 1.0;

    // Standard deviation of runs
    let std_runs = ((2.0 * (ones as f64) * (zeros as f64) * (2.0 * (ones as f64) * (zeros as f64) - n))
        / (n * n * (n - 1.0)))
        .sqrt();

    if std_runs == 0.0 || std_runs.is_nan() {
        return 0.0;
    }

    // Z-score
    let z = (runs as f64 - expected_runs) / std_runs;

    // Convert to 0-1 score
    let p = 1.0 - erf(z.abs() / std::f64::consts::SQRT_2);

    p.clamp(0.0, 1.0)
}

/// Error function approximation (Abramowitz and Stegun)
fn erf(x: f64) -> f64 {
    // Constants for approximation
    const A1: f64 = 0.254829592;
    const A2: f64 = -0.284496736;
    const A3: f64 = 1.421413741;
    const A4: f64 = -1.453152027;
    const A5: f64 = 1.061405429;
    const P: f64 = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + P * x);
    let y = 1.0 - (((((A5 * t + A4) * t) + A3) * t + A2) * t + A1) * t * (-x * x).exp();

    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qrng::pseudo::SeededPseudoBackend;
    use crate::qrng::QrngBackend;

    #[test]
    fn test_monobit_good_data() {
        // Good random data should pass
        let backend = SeededPseudoBackend::new(42);
        let data = backend.bytes(10000).unwrap();

        let score = monobit_test(&data);
        assert!(
            score > 0.01,
            "Good random data should pass monobit test, got {}",
            score
        );
    }

    #[test]
    fn test_monobit_bad_data() {
        // All zeros should fail
        let data = vec![0u8; 1000];
        let score = monobit_test(&data);
        assert!(
            score < 0.01,
            "All zeros should fail monobit test, got {}",
            score
        );

        // All ones should fail
        let data = vec![0xFFu8; 1000];
        let score = monobit_test(&data);
        assert!(
            score < 0.01,
            "All ones should fail monobit test, got {}",
            score
        );
    }

    #[test]
    fn test_chi_square_good_data() {
        let backend = SeededPseudoBackend::new(42);
        let data = backend.bytes(10000).unwrap();

        let score = chi_square_test(&data);
        assert!(
            score > 0.01,
            "Good random data should pass chi-square test, got {}",
            score
        );
    }

    #[test]
    fn test_chi_square_bad_data() {
        // Repeating pattern should have poor uniformity
        let data: Vec<u8> = (0..1000).map(|i| (i % 4) as u8).collect();
        let score = chi_square_test(&data);
        assert!(
            score < 0.01,
            "Repeating pattern should fail chi-square test, got {}",
            score
        );
    }

    #[test]
    fn test_runs_good_data() {
        let backend = SeededPseudoBackend::new(42);
        let data = backend.bytes(10000).unwrap();

        let score = runs_test(&data);
        assert!(
            score > 0.01,
            "Good random data should pass runs test, got {}",
            score
        );
    }

    #[test]
    fn test_runs_bad_data() {
        // Alternating bits should have too many runs
        let data: Vec<u8> = vec![0xAA; 1000]; // 10101010...
        let score = runs_test(&data);
        // This actually creates maximum runs, which is also bad
        // The test should detect this anomaly
        println!("Alternating pattern runs score: {}", score);
    }

    #[test]
    fn test_run_all_tests() {
        let backend = SeededPseudoBackend::new(42);
        let data = backend.bytes(10000).unwrap();

        let results = run_all_tests(&data);

        assert!(results.all_passed());
        assert_eq!(results.bytes_analyzed, 10000);
        assert!(results.overall > 0.0 && results.overall <= 1.0);
    }
}
