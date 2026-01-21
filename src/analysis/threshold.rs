//! Quality thresholds for fail-closed behavior.
//!
//! Defines thresholds that trigger suspension of reseeding
//! when entropy quality degrades.

use super::statistics::StatisticalTests;
use serde::{Deserialize, Serialize};

/// Quality thresholds for entropy monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Maximum acceptable bit bias (absolute value).
    pub max_bit_bias: f64,
    /// Minimum acceptable variance.
    pub min_variance: f64,
    /// Maximum acceptable autocorrelation (absolute value).
    pub max_autocorrelation: f64,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            max_bit_bias: 0.05,       // 5% bias tolerance
            min_variance: 500.0,      // Require meaningful variation
            max_autocorrelation: 0.3, // Low correlation tolerance
        }
    }
}

impl QualityThresholds {
    /// Creates more conservative thresholds.
    pub fn conservative() -> Self {
        Self {
            max_bit_bias: 0.02,
            min_variance: 1000.0,
            max_autocorrelation: 0.1,
        }
    }

    /// Creates more permissive thresholds (for testing).
    pub fn permissive() -> Self {
        Self {
            max_bit_bias: 0.2,
            min_variance: 100.0,
            max_autocorrelation: 0.5,
        }
    }

    /// Checks statistics against thresholds.
    pub fn check(&self, stats: &StatisticalTests) -> Result<(), ThresholdViolation> {
        if stats.bit_bias.abs() > self.max_bit_bias {
            return Err(ThresholdViolation::BitBias {
                observed: stats.bit_bias,
                threshold: self.max_bit_bias,
            });
        }

        if stats.variance < self.min_variance {
            return Err(ThresholdViolation::LowVariance {
                observed: stats.variance,
                threshold: self.min_variance,
            });
        }

        if stats.autocorrelation.abs() > self.max_autocorrelation {
            return Err(ThresholdViolation::HighAutocorrelation {
                observed: stats.autocorrelation,
                threshold: self.max_autocorrelation,
            });
        }

        Ok(())
    }
}

/// Threshold violation types.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ThresholdViolation {
    #[error("bit bias {observed:.4} exceeds threshold {threshold:.4}")]
    BitBias { observed: f64, threshold: f64 },

    #[error("variance {observed:.2} below threshold {threshold:.2}")]
    LowVariance { observed: f64, threshold: f64 },

    #[error("autocorrelation {observed:.4} exceeds threshold {threshold:.4}")]
    HighAutocorrelation { observed: f64, threshold: f64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extraction::RawBits;

    #[test]
    fn test_good_data_passes() {
        let thresholds = QualityThresholds::permissive();

        // Simulated reasonable data
        let data: Vec<u8> = (0..1000).map(|i| (i * 17 + 31) as u8).collect();
        let raw = RawBits::from_bytes(data, 1);
        let stats = StatisticalTests::analyze(&raw);

        assert!(thresholds.check(&stats).is_ok());
    }

    #[test]
    fn test_biased_data_fails() {
        let thresholds = QualityThresholds::default();

        let data = vec![0xFFu8; 1000]; // All ones = biased
        let raw = RawBits::from_bytes(data, 1);
        let stats = StatisticalTests::analyze(&raw);

        assert!(matches!(
            thresholds.check(&stats),
            Err(ThresholdViolation::BitBias { .. })
        ));
    }

    #[test]
    fn test_constant_data_fails_variance() {
        let thresholds = QualityThresholds::default();

        let data = vec![0x80u8; 1000]; // Constant = zero variance
        let raw = RawBits::from_bytes(data, 1);
        let stats = StatisticalTests::analyze(&raw);

        assert!(matches!(
            thresholds.check(&stats),
            Err(ThresholdViolation::LowVariance { .. })
        ));
    }
}
