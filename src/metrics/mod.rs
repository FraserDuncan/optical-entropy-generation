//! Prometheus metrics exporter for entropy monitoring.
//!
//! This module provides observability into the entropy generation system
//! by exposing metrics in Prometheus format via an HTTP endpoint.
//!
//! # Metrics Exposed
//!
//! ## Health Metrics
//! - `optical_entropy_health_status` - Current health status (1=healthy, 0=unhealthy)
//! - `optical_entropy_consecutive_healthy` - Consecutive healthy samples
//! - `optical_entropy_consecutive_unhealthy` - Consecutive unhealthy samples
//! - `optical_entropy_total_samples` - Total samples analyzed
//!
//! ## Statistical Test Metrics
//! - `optical_entropy_bit_bias` - Bit bias (deviation from 0.5)
//! - `optical_entropy_variance` - Byte-level variance
//! - `optical_entropy_autocorrelation` - Lag-1 autocorrelation
//!
//! ## CSPRNG Metrics
//! - `optical_entropy_csprng_reseed_total` - Total reseeds performed
//! - `optical_entropy_csprng_bytes_since_reseed` - Bytes generated since last reseed
//!
//! ## Pool Metrics
//! - `optical_entropy_pool_size_bytes` - Current pool size in bytes
//! - `optical_entropy_pool_total_bits_added` - Total bits added to pool
//! - `optical_entropy_pool_extractions_total` - Total extractions performed
//!
//! # Example
//!
//! ```no_run
//! use optical_entropy::metrics::{MetricsRegistry, MetricsSnapshot};
//!
//! // Create a metrics registry
//! let registry = MetricsRegistry::new().expect("Failed to create registry");
//!
//! // Update metrics from system state
//! let snapshot = MetricsSnapshot {
//!     is_healthy: true,
//!     consecutive_healthy: 5,
//!     consecutive_unhealthy: 0,
//!     total_samples: 100,
//!     bit_bias: Some(0.002),
//!     variance: Some(5400.0),
//!     autocorrelation: Some(0.01),
//!     reseed_count: 3,
//!     bytes_since_reseed: 1024,
//!     pool_size_bytes: 256,
//!     pool_total_bits_added: 8192,
//!     pool_extractions: 2,
//! };
//!
//! registry.update(&snapshot);
//! ```

mod collector;
#[cfg(feature = "metrics")]
mod server;

pub use collector::{MetricsRegistry, MetricsSnapshot};
#[cfg(feature = "metrics")]
pub use server::{MetricsServer, MetricsServerConfig};
