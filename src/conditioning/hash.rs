//! Cryptographic hash-based entropy conditioning.
//!
//! Uses standard hash functions to transform biased, correlated
//! raw bits into uniformly distributed output.

use crate::extraction::RawBits;
use blake3::Hasher as Blake3Hasher;
use sha2::{Digest, Sha256};

/// Supported hash algorithms for conditioning.
#[derive(Debug, Clone, Copy, Default)]
pub enum HashAlgorithm {
    /// BLAKE3 - fast, secure, recommended default.
    #[default]
    Blake3,
    /// SHA-256 - widely deployed, conservative choice.
    Sha256,
}

/// Conditioned entropy output.
///
/// Fixed-size output from the conditioning hash, ready for
/// use as CSPRNG seed material.
#[derive(Clone)]
pub struct ConditionedSeed {
    /// The conditioned bytes (32 bytes for both BLAKE3 and SHA-256).
    data: [u8; 32],
    /// Source entropy estimate in bits.
    entropy_estimate: usize,
}

impl ConditionedSeed {
    /// Returns the seed bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.data
    }

    /// Returns the entropy estimate.
    #[inline]
    pub fn entropy_estimate(&self) -> usize {
        self.entropy_estimate
    }

    /// Creates a seed for testing purposes only.
    ///
    /// This bypasses the normal conditioning pipeline and should
    /// never be used in production code.
    #[cfg(test)]
    pub(crate) fn new_for_testing(data: [u8; 32], entropy_estimate: usize) -> Self {
        Self {
            data,
            entropy_estimate,
        }
    }
}

impl std::fmt::Debug for ConditionedSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConditionedSeed")
            .field("entropy_estimate", &self.entropy_estimate)
            .finish_non_exhaustive()
    }
}

/// Entropy conditioner using cryptographic hashing.
///
/// Transforms raw extracted bits into uniformly distributed
/// seed material using a cryptographic hash function.
pub struct Conditioner {
    algorithm: HashAlgorithm,
}

impl Conditioner {
    /// Creates a new conditioner with the specified algorithm.
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Self { algorithm }
    }

    /// Conditions raw bits into a fixed-size seed.
    ///
    /// The entropy estimate is conservative: we assume the raw bits
    /// contain at most 1 bit of entropy per byte of input, capped
    /// at the output size.
    pub fn condition(&self, raw: &RawBits) -> ConditionedSeed {
        let data = match self.algorithm {
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(raw.data());
                *hasher.finalize().as_bytes()
            }
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(raw.data());
                let result = hasher.finalize();
                let mut data = [0u8; 32];
                data.copy_from_slice(&result);
                data
            }
        };

        // Conservative entropy estimate: assume ~1 bit per input byte,
        // but never more than output size (256 bits).
        let entropy_estimate = raw.len().min(256);

        ConditionedSeed {
            data,
            entropy_estimate,
        }
    }
}

impl Default for Conditioner {
    fn default() -> Self {
        Self::new(HashAlgorithm::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_conditioning() {
        let conditioner = Conditioner::new(HashAlgorithm::Blake3);
        let raw = RawBits::from_bytes(vec![0x42; 1000], 1);

        let seed = conditioner.condition(&raw);
        assert_eq!(seed.as_bytes().len(), 32);
        assert_eq!(seed.entropy_estimate(), 256); // capped at output size
    }

    #[test]
    fn test_sha256_conditioning() {
        let conditioner = Conditioner::new(HashAlgorithm::Sha256);
        let raw = RawBits::from_bytes(vec![0x42; 1000], 1);

        let seed = conditioner.condition(&raw);
        assert_eq!(seed.as_bytes().len(), 32);
    }

    #[test]
    fn test_different_input_different_output() {
        let conditioner = Conditioner::default();

        let raw1 = RawBits::from_bytes(vec![0x00; 100], 1);
        let raw2 = RawBits::from_bytes(vec![0x01; 100], 1);

        let seed1 = conditioner.condition(&raw1);
        let seed2 = conditioner.condition(&raw2);

        assert_ne!(seed1.as_bytes(), seed2.as_bytes());
    }

    #[test]
    fn test_small_input_limited_entropy() {
        let conditioner = Conditioner::default();
        let raw = RawBits::from_bytes(vec![0x42; 10], 1);

        let seed = conditioner.condition(&raw);
        assert_eq!(seed.entropy_estimate(), 10); // limited by input size
    }
}
