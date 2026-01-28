//! Metrics collection and registry.

use prometheus::{Gauge, IntCounter, IntGauge, Registry, TextEncoder, Encoder};
use thiserror::Error;

/// Errors that can occur during metrics operations.
#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("prometheus error: {0}")]
    Prometheus(#[from] prometheus::Error),
}

/// A snapshot of system state for metrics update.
#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot {
    /// Whether the entropy source is currently healthy.
    pub is_healthy: bool,
    /// Consecutive healthy samples.
    pub consecutive_healthy: u64,
    /// Consecutive unhealthy samples.
    pub consecutive_unhealthy: u64,
    /// Total samples analyzed.
    pub total_samples: u64,
    /// Bit bias from latest statistical test.
    pub bit_bias: Option<f64>,
    /// Variance from latest statistical test.
    pub variance: Option<f64>,
    /// Autocorrelation from latest statistical test.
    pub autocorrelation: Option<f64>,
    /// Total CSPRNG reseeds performed.
    pub reseed_count: u64,
    /// Bytes generated since last reseed.
    pub bytes_since_reseed: u64,
    /// Current entropy pool size in bytes.
    pub pool_size_bytes: usize,
    /// Total bits ever added to the pool.
    pub pool_total_bits_added: u64,
    /// Total pool extractions performed.
    pub pool_extractions: u64,
}

/// Prometheus metrics registry for entropy monitoring.
pub struct MetricsRegistry {
    registry: Registry,

    // Health metrics
    health_status: IntGauge,
    consecutive_healthy: IntGauge,
    consecutive_unhealthy: IntGauge,
    total_samples: IntCounter,

    // Statistical test metrics
    bit_bias: Gauge,
    variance: Gauge,
    autocorrelation: Gauge,

    // CSPRNG metrics
    reseed_total: IntCounter,
    bytes_since_reseed: IntGauge,

    // Pool metrics
    pool_size_bytes: IntGauge,
    pool_total_bits_added: IntCounter,
    pool_extractions_total: IntCounter,
}

impl MetricsRegistry {
    /// Creates a new metrics registry with all entropy metrics registered.
    pub fn new() -> Result<Self, MetricsError> {
        let registry = Registry::new();

        // Health metrics
        let health_status = IntGauge::new(
            "optical_entropy_health_status",
            "Current health status (1=healthy, 0=unhealthy)",
        )?;
        let consecutive_healthy = IntGauge::new(
            "optical_entropy_consecutive_healthy",
            "Number of consecutive healthy samples",
        )?;
        let consecutive_unhealthy = IntGauge::new(
            "optical_entropy_consecutive_unhealthy",
            "Number of consecutive unhealthy samples",
        )?;
        let total_samples = IntCounter::new(
            "optical_entropy_total_samples",
            "Total number of samples analyzed",
        )?;

        // Statistical test metrics
        let bit_bias = Gauge::new(
            "optical_entropy_bit_bias",
            "Bit bias from statistical test (deviation from 0.5)",
        )?;
        let variance = Gauge::new(
            "optical_entropy_variance",
            "Byte-level variance from statistical test",
        )?;
        let autocorrelation = Gauge::new(
            "optical_entropy_autocorrelation",
            "Lag-1 autocorrelation from statistical test",
        )?;

        // CSPRNG metrics
        let reseed_total = IntCounter::new(
            "optical_entropy_csprng_reseed_total",
            "Total number of CSPRNG reseeds performed",
        )?;
        let bytes_since_reseed = IntGauge::new(
            "optical_entropy_csprng_bytes_since_reseed",
            "Bytes generated since last CSPRNG reseed",
        )?;

        // Pool metrics
        let pool_size_bytes = IntGauge::new(
            "optical_entropy_pool_size_bytes",
            "Current entropy pool size in bytes",
        )?;
        let pool_total_bits_added = IntCounter::new(
            "optical_entropy_pool_total_bits_added",
            "Total bits ever added to the entropy pool",
        )?;
        let pool_extractions_total = IntCounter::new(
            "optical_entropy_pool_extractions_total",
            "Total entropy pool extractions performed",
        )?;

        // Register all metrics
        registry.register(Box::new(health_status.clone()))?;
        registry.register(Box::new(consecutive_healthy.clone()))?;
        registry.register(Box::new(consecutive_unhealthy.clone()))?;
        registry.register(Box::new(total_samples.clone()))?;
        registry.register(Box::new(bit_bias.clone()))?;
        registry.register(Box::new(variance.clone()))?;
        registry.register(Box::new(autocorrelation.clone()))?;
        registry.register(Box::new(reseed_total.clone()))?;
        registry.register(Box::new(bytes_since_reseed.clone()))?;
        registry.register(Box::new(pool_size_bytes.clone()))?;
        registry.register(Box::new(pool_total_bits_added.clone()))?;
        registry.register(Box::new(pool_extractions_total.clone()))?;

        Ok(Self {
            registry,
            health_status,
            consecutive_healthy,
            consecutive_unhealthy,
            total_samples,
            bit_bias,
            variance,
            autocorrelation,
            reseed_total,
            bytes_since_reseed,
            pool_size_bytes,
            pool_total_bits_added,
            pool_extractions_total,
        })
    }

    /// Updates all metrics from a snapshot of system state.
    pub fn update(&self, snapshot: &MetricsSnapshot) {
        // Health metrics
        self.health_status.set(if snapshot.is_healthy { 1 } else { 0 });
        self.consecutive_healthy.set(snapshot.consecutive_healthy as i64);
        self.consecutive_unhealthy.set(snapshot.consecutive_unhealthy as i64);

        // For counters, we need to increment by the difference
        let current_samples = self.total_samples.get();
        if snapshot.total_samples > current_samples {
            self.total_samples.inc_by(snapshot.total_samples - current_samples);
        }

        // Statistical test metrics (only update if present)
        if let Some(bias) = snapshot.bit_bias {
            self.bit_bias.set(bias);
        }
        if let Some(var) = snapshot.variance {
            self.variance.set(var);
        }
        if let Some(autocorr) = snapshot.autocorrelation {
            self.autocorrelation.set(autocorr);
        }

        // CSPRNG metrics
        let current_reseeds = self.reseed_total.get();
        if snapshot.reseed_count > current_reseeds {
            self.reseed_total.inc_by(snapshot.reseed_count - current_reseeds);
        }
        self.bytes_since_reseed.set(snapshot.bytes_since_reseed as i64);

        // Pool metrics
        self.pool_size_bytes.set(snapshot.pool_size_bytes as i64);

        let current_bits = self.pool_total_bits_added.get();
        if snapshot.pool_total_bits_added > current_bits {
            self.pool_total_bits_added.inc_by(snapshot.pool_total_bits_added - current_bits);
        }

        let current_extractions = self.pool_extractions_total.get();
        if snapshot.pool_extractions > current_extractions {
            self.pool_extractions_total.inc_by(snapshot.pool_extractions - current_extractions);
        }
    }

    /// Returns the underlying Prometheus registry.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Encodes all metrics in Prometheus text format.
    pub fn encode(&self) -> Result<String, MetricsError> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).into_owned())
    }
}

impl MetricsSnapshot {
    /// Creates a snapshot from the current state of entropy components.
    pub fn from_components(
        health: &crate::analysis::HealthMetrics,
        rng: &crate::reseeding::ReseedableRng,
        pool: &crate::conditioning::EntropyPool,
    ) -> Self {
        let (bit_bias, variance, autocorrelation) = health
            .latest_stats
            .as_ref()
            .map(|s| (Some(s.bit_bias), Some(s.variance), Some(s.autocorrelation)))
            .unwrap_or((None, None, None));

        Self {
            is_healthy: health.is_healthy,
            consecutive_healthy: health.consecutive_healthy,
            consecutive_unhealthy: health.consecutive_unhealthy,
            total_samples: health.total_samples,
            bit_bias,
            variance,
            autocorrelation,
            reseed_count: rng.reseed_count(),
            bytes_since_reseed: rng.bytes_since_reseed(),
            pool_size_bytes: pool.size_bytes(),
            pool_total_bits_added: pool.total_bits_added(),
            pool_extractions: pool.total_extractions(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = MetricsRegistry::new();
        assert!(registry.is_ok());
    }

    #[test]
    fn test_metrics_update() {
        let registry = MetricsRegistry::new().unwrap();

        let snapshot = MetricsSnapshot {
            is_healthy: true,
            consecutive_healthy: 5,
            consecutive_unhealthy: 0,
            total_samples: 10,
            bit_bias: Some(0.01),
            variance: Some(5000.0),
            autocorrelation: Some(0.02),
            reseed_count: 2,
            bytes_since_reseed: 1024,
            pool_size_bytes: 128,
            pool_total_bits_added: 4096,
            pool_extractions: 1,
        };

        registry.update(&snapshot);

        // Verify metrics were set
        let output = registry.encode().unwrap();
        assert!(output.contains("optical_entropy_health_status 1"));
        assert!(output.contains("optical_entropy_consecutive_healthy 5"));
        assert!(output.contains("optical_entropy_csprng_reseed_total 2"));
    }

    #[test]
    fn test_metrics_encode() {
        let registry = MetricsRegistry::new().unwrap();
        let output = registry.encode().unwrap();

        // Should contain metric names
        assert!(output.contains("optical_entropy_health_status"));
        assert!(output.contains("optical_entropy_csprng_reseed_total"));
        assert!(output.contains("optical_entropy_pool_size_bytes"));
    }
}
