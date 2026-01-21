//! Entropy testing and health monitoring.
//!
//! This module provides statistical tests and health metrics
//! for monitoring entropy quality. These are sanity checks,
//! not cryptographic proofs of entropy.

mod health;
mod statistics;
mod threshold;

pub use health::{HealthMetrics, HealthMonitor};
pub use statistics::StatisticalTests;
pub use threshold::{QualityThresholds, ThresholdViolation};
