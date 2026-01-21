//! Entropy accumulation pool.
//!
//! Collects entropy from multiple extractions before conditioning,
//! ensuring sufficient entropy has been gathered before reseeding.

use super::hash::{ConditionedSeed, Conditioner, HashAlgorithm};
use crate::extraction::RawBits;

/// Configuration for the entropy pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum bits to accumulate before allowing extraction.
    pub min_bits: usize,
    /// Maximum bytes to buffer (prevents unbounded growth).
    pub max_bytes: usize,
    /// Hash algorithm for conditioning.
    pub algorithm: HashAlgorithm,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_bits: 512,        // Require 512 bits minimum
            max_bytes: 64 * 1024, // Cap at 64KB
            algorithm: HashAlgorithm::Blake3,
        }
    }
}

/// Accumulates entropy before conditioning.
///
/// The pool collects raw bits from multiple extraction cycles,
/// ensuring sufficient entropy has been gathered before producing
/// conditioned output for reseeding.
pub struct EntropyPool {
    /// Accumulated raw bytes.
    buffer: Vec<u8>,
    /// Configuration.
    config: PoolConfig,
    /// Conditioner instance.
    conditioner: Conditioner,
    /// Total bits added (for metrics).
    total_bits_added: u64,
    /// Total extractions performed.
    total_extractions: u64,
}

impl EntropyPool {
    /// Creates a new entropy pool with the given configuration.
    pub fn new(config: PoolConfig) -> Self {
        let conditioner = Conditioner::new(config.algorithm);
        Self {
            buffer: Vec::with_capacity(config.max_bytes),
            config,
            conditioner,
            total_bits_added: 0,
            total_extractions: 0,
        }
    }

    /// Adds raw bits to the pool.
    pub fn add(&mut self, raw: &RawBits) {
        let space_remaining = self.config.max_bytes.saturating_sub(self.buffer.len());
        let bytes_to_add = raw.len().min(space_remaining);

        self.buffer.extend_from_slice(&raw.data()[..bytes_to_add]);
        self.total_bits_added += (bytes_to_add * 8) as u64;

        tracing::trace!(
            bytes_added = bytes_to_add,
            pool_size = self.buffer.len(),
            "Added entropy to pool"
        );
    }

    /// Returns true if the pool has enough entropy for extraction.
    pub fn is_ready(&self) -> bool {
        self.buffer.len() * 8 >= self.config.min_bits
    }

    /// Extracts conditioned entropy from the pool.
    ///
    /// Returns `None` if insufficient entropy has been accumulated.
    /// Clears the pool after extraction.
    pub fn extract(&mut self) -> Option<ConditionedSeed> {
        if !self.is_ready() {
            tracing::debug!(
                pool_bits = self.buffer.len() * 8,
                min_bits = self.config.min_bits,
                "Pool not ready for extraction"
            );
            return None;
        }

        let raw = RawBits::from_bytes(std::mem::take(&mut self.buffer), self.total_extractions);
        let seed = self.conditioner.condition(&raw);

        self.total_extractions += 1;

        tracing::debug!(
            extraction_number = self.total_extractions,
            entropy_estimate = seed.entropy_estimate(),
            "Extracted conditioned entropy"
        );

        Some(seed)
    }

    /// Returns the current pool size in bytes.
    pub fn size_bytes(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the current pool size in bits.
    pub fn size_bits(&self) -> usize {
        self.buffer.len() * 8
    }

    /// Returns total bits ever added to the pool.
    pub fn total_bits_added(&self) -> u64 {
        self.total_bits_added
    }

    /// Returns total extractions performed.
    pub fn total_extractions(&self) -> u64 {
        self.total_extractions
    }

    /// Clears the pool without extracting.
    pub fn clear(&mut self) {
        self.buffer.clear();
        tracing::info!("Entropy pool cleared");
    }
}

impl Default for EntropyPool {
    fn default() -> Self {
        Self::new(PoolConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_not_ready_initially() {
        let pool = EntropyPool::default();
        assert!(!pool.is_ready());
    }

    #[test]
    fn test_pool_ready_after_sufficient_entropy() {
        let config = PoolConfig {
            min_bits: 80, // 10 bytes
            ..Default::default()
        };
        let mut pool = EntropyPool::new(config);

        pool.add(&RawBits::from_bytes(vec![0u8; 10], 1));
        assert!(pool.is_ready());
    }

    #[test]
    fn test_extraction_clears_pool() {
        let config = PoolConfig {
            min_bits: 80,
            ..Default::default()
        };
        let mut pool = EntropyPool::new(config);

        pool.add(&RawBits::from_bytes(vec![0u8; 20], 1));
        assert!(pool.is_ready());

        let seed = pool.extract();
        assert!(seed.is_some());
        assert!(!pool.is_ready());
        assert_eq!(pool.size_bytes(), 0);
    }

    #[test]
    fn test_max_bytes_limit() {
        let config = PoolConfig {
            min_bits: 8,
            max_bytes: 10,
            ..Default::default()
        };
        let mut pool = EntropyPool::new(config);

        // Try to add more than max
        pool.add(&RawBits::from_bytes(vec![0u8; 100], 1));

        // Should be capped at max_bytes
        assert_eq!(pool.size_bytes(), 10);
    }
}
