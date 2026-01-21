//! ChaCha-based CSPRNG with reseeding support.
//!
//! Wraps the standard ChaCha20 CSPRNG with an interface for
//! reseeding from conditioned optical entropy.

use crate::conditioning::ConditionedSeed;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use thiserror::Error;

/// Errors that can occur during reseeding.
#[derive(Debug, Error)]
pub enum ReseedingError {
    #[error("insufficient entropy: got {got} bits, need {need} bits")]
    InsufficientEntropy { got: usize, need: usize },
}

/// A reseedable CSPRNG backed by ChaCha20.
///
/// This wraps the standard ChaCha20Rng with an interface designed
/// for periodic reseeding from optical entropy. The CSPRNG is
/// initialized from OS entropy and can be reseeded with conditioned
/// optical entropy to supplement (not replace) the initial seed.
pub struct ReseedableRng {
    /// The underlying ChaCha20 CSPRNG.
    inner: ChaCha20Rng,
    /// Minimum entropy required for reseeding.
    min_entropy_bits: usize,
    /// Total reseeds performed.
    reseed_count: u64,
    /// Bytes generated since last reseed.
    bytes_since_reseed: u64,
}

impl ReseedableRng {
    /// Creates a new CSPRNG seeded from the OS entropy source.
    ///
    /// This is the recommended way to initialize the CSPRNG.
    /// Optical entropy is used to *supplement* this initial seed,
    /// not replace it.
    pub fn from_os_entropy() -> Self {
        Self {
            inner: ChaCha20Rng::from_entropy(),
            min_entropy_bits: 128,
            reseed_count: 0,
            bytes_since_reseed: 0,
        }
    }

    /// Creates a CSPRNG with a specific minimum entropy requirement.
    pub fn with_min_entropy(min_entropy_bits: usize) -> Self {
        Self {
            min_entropy_bits,
            ..Self::from_os_entropy()
        }
    }

    /// Reseeds the CSPRNG with conditioned optical entropy.
    ///
    /// The new seed is mixed with the current state rather than
    /// replacing it entirely, ensuring that compromising the
    /// optical source alone cannot predict outputs.
    pub fn reseed(&mut self, seed: &ConditionedSeed) -> Result<(), ReseedingError> {
        if seed.entropy_estimate() < self.min_entropy_bits {
            return Err(ReseedingError::InsufficientEntropy {
                got: seed.entropy_estimate(),
                need: self.min_entropy_bits,
            });
        }

        // Mix new seed with current state by XORing
        // This ensures optical entropy supplements, not replaces
        let mut current_state = [0u8; 32];
        self.inner.fill_bytes(&mut current_state);

        let mut mixed_seed = [0u8; 32];
        for i in 0..32 {
            mixed_seed[i] = current_state[i] ^ seed.as_bytes()[i];
        }

        self.inner = ChaCha20Rng::from_seed(mixed_seed);
        self.reseed_count += 1;
        self.bytes_since_reseed = 0;

        tracing::info!(
            reseed_count = self.reseed_count,
            entropy_estimate = seed.entropy_estimate(),
            "CSPRNG reseeded"
        );

        Ok(())
    }

    /// Returns the number of reseeds performed.
    pub fn reseed_count(&self) -> u64 {
        self.reseed_count
    }

    /// Returns bytes generated since last reseed.
    pub fn bytes_since_reseed(&self) -> u64 {
        self.bytes_since_reseed
    }
}

impl RngCore for ReseedableRng {
    fn next_u32(&mut self) -> u32 {
        self.bytes_since_reseed += 4;
        self.inner.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.bytes_since_reseed += 8;
        self.inner.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.bytes_since_reseed += dest.len() as u64;
        self.inner.fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.bytes_since_reseed += dest.len() as u64;
        self.inner.try_fill_bytes(dest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_seed(entropy: usize) -> ConditionedSeed {
        // Create a seed with specified entropy estimate
        // This is a test helper - in production, seeds come from conditioning
        let data = [0x42u8; 32];
        // Use a simple struct construction for testing
        unsafe { std::mem::transmute((data, entropy)) }
    }

    #[test]
    fn test_reseed_increments_count() {
        let mut rng = ReseedableRng::with_min_entropy(64);
        assert_eq!(rng.reseed_count(), 0);

        // Create a mock seed with sufficient entropy
        let seed = make_test_seed(128);
        rng.reseed(&seed).unwrap();

        assert_eq!(rng.reseed_count(), 1);
    }

    #[test]
    fn test_insufficient_entropy_rejected() {
        let mut rng = ReseedableRng::with_min_entropy(256);

        let seed = make_test_seed(128);
        let result = rng.reseed(&seed);

        assert!(matches!(
            result,
            Err(ReseedingError::InsufficientEntropy { .. })
        ));
    }

    #[test]
    fn test_bytes_since_reseed_tracking() {
        let mut rng = ReseedableRng::from_os_entropy();

        let mut buf = [0u8; 100];
        rng.fill_bytes(&mut buf);

        assert_eq!(rng.bytes_since_reseed(), 100);
    }
}
