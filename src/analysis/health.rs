//! Entropy health monitoring.
//!
//! Tracks entropy quality over time and implements fail-closed
//! behavior when quality degrades.

use super::{
    statistics::StatisticalTests,
    threshold::{QualityThresholds, ThresholdViolation},
};
use crate::extraction::RawBits;

/// Current health status of the entropy source.
#[derive(Debug, Clone)]
pub struct HealthMetrics {
    /// Most recent statistical test results.
    pub latest_stats: Option<StatisticalTests>,
    /// Whether the source is currently healthy.
    pub is_healthy: bool,
    /// Most recent violation, if any.
    pub last_violation: Option<ThresholdViolation>,
    /// Consecutive healthy samples.
    pub consecutive_healthy: u64,
    /// Consecutive unhealthy samples.
    pub consecutive_unhealthy: u64,
    /// Total samples analyzed.
    pub total_samples: u64,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            latest_stats: None,
            is_healthy: false, // Fail-closed: unhealthy until proven otherwise
            last_violation: None,
            consecutive_healthy: 0,
            consecutive_unhealthy: 0,
            total_samples: 0,
        }
    }
}

/// Monitors entropy health over time.
///
/// Implements fail-closed behavior: reseeding is only allowed
/// when the source has demonstrated consistent quality.
pub struct HealthMonitor {
    /// Quality thresholds.
    thresholds: QualityThresholds,
    /// Current metrics.
    metrics: HealthMetrics,
    /// Required consecutive healthy samples to become healthy.
    required_healthy_streak: u64,
}

impl HealthMonitor {
    /// Creates a new health monitor with default settings.
    pub fn new(thresholds: QualityThresholds) -> Self {
        Self {
            thresholds,
            metrics: HealthMetrics::default(),
            required_healthy_streak: 3, // Require 3 good samples
        }
    }

    /// Creates a monitor with a custom healthy streak requirement.
    pub fn with_streak_requirement(thresholds: QualityThresholds, streak: u64) -> Self {
        Self {
            thresholds,
            metrics: HealthMetrics::default(),
            required_healthy_streak: streak.max(1),
        }
    }

    /// Analyzes a sample and updates health status.
    pub fn analyze(&mut self, raw: &RawBits) -> &HealthMetrics {
        let stats = StatisticalTests::analyze(raw);
        self.metrics.total_samples += 1;

        match self.thresholds.check(&stats) {
            Ok(()) => {
                self.metrics.consecutive_healthy += 1;
                self.metrics.consecutive_unhealthy = 0;
                self.metrics.last_violation = None;

                // Become healthy after sufficient streak
                if self.metrics.consecutive_healthy >= self.required_healthy_streak {
                    if !self.metrics.is_healthy {
                        tracing::info!(
                            streak = self.metrics.consecutive_healthy,
                            "Entropy source became healthy"
                        );
                    }
                    self.metrics.is_healthy = true;
                }

                tracing::trace!(
                    bias = stats.bit_bias,
                    variance = stats.variance,
                    autocorr = stats.autocorrelation,
                    "Health check passed"
                );
            }
            Err(violation) => {
                self.metrics.consecutive_unhealthy += 1;
                self.metrics.consecutive_healthy = 0;
                self.metrics.last_violation = Some(violation.clone());

                // Immediately become unhealthy (fail-closed)
                if self.metrics.is_healthy {
                    tracing::warn!(
                        violation = %violation,
                        "Entropy source became unhealthy"
                    );
                }
                self.metrics.is_healthy = false;
            }
        }

        self.metrics.latest_stats = Some(stats);
        &self.metrics
    }

    /// Returns current health metrics.
    pub fn metrics(&self) -> &HealthMetrics {
        &self.metrics
    }

    /// Returns true if reseeding should be allowed.
    pub fn allow_reseed(&self) -> bool {
        self.metrics.is_healthy
    }

    /// Resets the monitor to initial state.
    pub fn reset(&mut self) {
        self.metrics = HealthMetrics::default();
        tracing::info!("Health monitor reset");
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new(QualityThresholds::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_good_data() -> RawBits {
        let data: Vec<u8> = (0..1000).map(|i| (i * 17 + 31) as u8).collect();
        RawBits::from_bytes(data, 1)
    }

    fn make_bad_data() -> RawBits {
        RawBits::from_bytes(vec![0xFFu8; 1000], 1)
    }

    #[test]
    fn test_starts_unhealthy() {
        let monitor = HealthMonitor::new(QualityThresholds::permissive());
        assert!(!monitor.allow_reseed());
    }

    #[test]
    fn test_becomes_healthy_after_streak() {
        let mut monitor =
            HealthMonitor::with_streak_requirement(QualityThresholds::permissive(), 2);

        // First good sample: not healthy yet
        monitor.analyze(&make_good_data());
        assert!(!monitor.allow_reseed());

        // Second good sample: now healthy
        monitor.analyze(&make_good_data());
        assert!(monitor.allow_reseed());
    }

    #[test]
    fn test_immediately_unhealthy_on_failure() {
        let mut monitor =
            HealthMonitor::with_streak_requirement(QualityThresholds::permissive(), 2);

        // Become healthy
        monitor.analyze(&make_good_data());
        monitor.analyze(&make_good_data());
        assert!(monitor.allow_reseed());

        // Single bad sample makes unhealthy (fail-closed)
        monitor.analyze(&make_bad_data());
        assert!(!monitor.allow_reseed());
    }
}
