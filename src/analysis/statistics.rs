//! Statistical tests for entropy quality.
//!
//! These tests are sanity checks to detect obvious problems,
//! not proofs of entropy quality. Passing these tests is necessary
//! but not sufficient for good entropy.

use crate::extraction::RawBits;

/// Statistical test results.
#[derive(Debug, Clone)]
pub struct StatisticalTests {
    /// Bit bias (deviation from 0.5).
    pub bit_bias: f64,
    /// Byte-level variance.
    pub variance: f64,
    /// Lag-1 autocorrelation.
    pub autocorrelation: f64,
    /// Number of bytes analyzed.
    pub sample_size: usize,
}

impl StatisticalTests {
    /// Runs all statistical tests on the raw bits.
    pub fn analyze(raw: &RawBits) -> Self {
        let data = raw.data();

        Self {
            bit_bias: raw.bit_bias(),
            variance: Self::compute_variance(data),
            autocorrelation: Self::compute_autocorrelation(data),
            sample_size: data.len(),
        }
    }

    /// Computes the variance of byte values.
    fn compute_variance(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let n = data.len() as f64;
        let mean: f64 = data.iter().map(|&b| b as f64).sum::<f64>() / n;
        let variance: f64 = data.iter().map(|&b| (b as f64 - mean).powi(2)).sum::<f64>() / n;

        variance
    }

    /// Computes lag-1 autocorrelation.
    ///
    /// Measures correlation between consecutive bytes.
    /// High values indicate predictable patterns.
    fn compute_autocorrelation(data: &[u8]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let n = data.len() as f64;
        let mean: f64 = data.iter().map(|&b| b as f64).sum::<f64>() / n;

        let variance: f64 = data.iter().map(|&b| (b as f64 - mean).powi(2)).sum::<f64>();

        if variance == 0.0 {
            return 1.0; // All same value = perfect correlation
        }

        let covariance: f64 = data
            .windows(2)
            .map(|w| (w[0] as f64 - mean) * (w[1] as f64 - mean))
            .sum();

        covariance / variance
    }

    /// Returns true if results look reasonable (not proof of quality).
    pub fn looks_reasonable(&self) -> bool {
        // These are loose sanity checks, not security guarantees
        let bias_ok = self.bit_bias.abs() < 0.1; // Within 10% of unbiased
        let variance_ok = self.variance > 100.0; // Some variation expected
        let autocorr_ok = self.autocorrelation.abs() < 0.5; // Not highly correlated

        bias_ok && variance_ok && autocorr_ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_random_passes() {
        // Simulated "random" data (alternating pattern for predictability)
        let data: Vec<u8> = (0..1000).map(|i| (i * 17 + 31) as u8).collect();
        let raw = RawBits::from_bytes(data, 1);

        let stats = StatisticalTests::analyze(&raw);

        // Pseudo-random should have reasonable variance
        assert!(stats.variance > 100.0);
    }

    #[test]
    fn test_constant_data_fails() {
        let data = vec![0x80u8; 1000];
        let raw = RawBits::from_bytes(data, 1);

        let stats = StatisticalTests::analyze(&raw);

        // Constant data: zero variance, perfect autocorrelation
        assert_eq!(stats.variance, 0.0);
        assert!(!stats.looks_reasonable());
    }

    #[test]
    fn test_all_ones_biased() {
        let data = vec![0xFFu8; 1000];
        let raw = RawBits::from_bytes(data, 1);

        let stats = StatisticalTests::analyze(&raw);

        // All ones = maximum positive bias
        assert!((stats.bit_bias - 0.5).abs() < 0.001);
        assert!(!stats.looks_reasonable());
    }
}
