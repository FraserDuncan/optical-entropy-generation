//! ChaCha-based CSPRNG with reseeding support.
//!
//! Wraps the standard ChaCha20 CSPRNG with an interface for
//! reseeding from conditioned optical entropy.
//!
//! # Reseeding Model
//!
//! Reseeding uses BLAKE3 to mix:
//! - Previous seed material (retained across reseeds)
//! - New conditioned entropy
//! - A domain separator and reseed counter
//!
//! This follows NIST SP 800-90A style DRBG reseeding logic:
//! non-linear mixing via a cryptographic hash ensures that
//! biased or partially predictable inputs cannot degrade security.

use blake3::Hasher;
use crate::conditioning::ConditionedSeed;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use thiserror::Error;

/// Domain separator for reseeding operations.
/// Ensures the hash context is distinct from other uses.
const RESEED_DOMAIN: &[u8] = b"optical-entropy-reseed-v1";

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
///
/// # Security Model
///
/// - Initial seed comes from OS entropy (trusted)
/// - Optical entropy is mixed in via BLAKE3 (non-linear, analyzed)
/// - Previous seed material is retained and mixed with new entropy
/// - Compromising only the optical source cannot predict outputs
pub struct ReseedableRng {
    /// The underlying ChaCha20 CSPRNG.
    inner: ChaCha20Rng,
    /// Retained seed material for mixing during reseed.
    /// This is NOT the ChaCha internal state.
    seed_material: [u8; 32],
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
        // Get initial seed from OS
        let mut seed_material = [0u8; 32];
        rand_core::OsRng.fill_bytes(&mut seed_material);

        Self {
            inner: ChaCha20Rng::from_seed(seed_material),
            seed_material,
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

    /// Creates a CSPRNG from a known seed (for testing only).
    #[cfg(test)]
    pub(crate) fn from_seed_for_testing(seed: [u8; 32]) -> Self {
        Self {
            inner: ChaCha20Rng::from_seed(seed),
            seed_material: seed,
            min_entropy_bits: 128,
            reseed_count: 0,
            bytes_since_reseed: 0,
        }
    }

    /// Reseeds the CSPRNG with conditioned optical entropy.
    ///
    /// The new seed is derived by hashing together:
    /// - The previous seed material
    /// - The new conditioned entropy
    /// - A domain separator and reseed counter
    ///
    /// This ensures:
    /// - Non-linear mixing (hash, not XOR)
    /// - Bias resistance (hash output is uniform)
    /// - Forward secrecy properties are maintained
    /// - Compromising optical source alone cannot predict outputs
    pub fn reseed(&mut self, seed: &ConditionedSeed) -> Result<(), ReseedingError> {
        if seed.entropy_estimate() < self.min_entropy_bits {
            return Err(ReseedingError::InsufficientEntropy {
                got: seed.entropy_estimate(),
                need: self.min_entropy_bits,
            });
        }

        // Mix using BLAKE3:
        // new_seed = BLAKE3(domain || counter || old_seed_material || new_entropy)
        let mut hasher = Hasher::new();
        hasher.update(RESEED_DOMAIN);
        hasher.update(&self.reseed_count.to_le_bytes());
        hasher.update(&self.seed_material);
        hasher.update(seed.as_bytes());

        let new_seed_material: [u8; 32] = *hasher.finalize().as_bytes();

        // Update state
        self.seed_material = new_seed_material;
        self.inner = ChaCha20Rng::from_seed(new_seed_material);
        self.reseed_count += 1;
        self.bytes_since_reseed = 0;

        tracing::info!(
            reseed_count = self.reseed_count,
            entropy_estimate = seed.entropy_estimate(),
            "CSPRNG reseeded via BLAKE3 mixing"
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

    fn make_test_seed(data: [u8; 32], entropy: usize) -> ConditionedSeed {
        ConditionedSeed::new_for_testing(data, entropy)
    }

    #[test]
    fn test_reseed_increments_count() {
        let mut rng = ReseedableRng::with_min_entropy(64);
        assert_eq!(rng.reseed_count(), 0);

        let seed = make_test_seed([0x42u8; 32], 128);
        rng.reseed(&seed).unwrap();

        assert_eq!(rng.reseed_count(), 1);
    }

    #[test]
    fn test_insufficient_entropy_rejected() {
        let mut rng = ReseedableRng::with_min_entropy(256);

        let seed = make_test_seed([0x42u8; 32], 128);
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

    #[test]
    fn test_reseed_changes_output() {
        let initial_seed = [0x01u8; 32];
        let mut rng1 = ReseedableRng::from_seed_for_testing(initial_seed);
        let mut rng2 = ReseedableRng::from_seed_for_testing(initial_seed);

        // Before reseed: same output
        let mut out1 = [0u8; 32];
        let mut out2 = [0u8; 32];
        rng1.fill_bytes(&mut out1);
        rng2.fill_bytes(&mut out2);
        assert_eq!(out1, out2);

        // Reseed rng1 only
        let new_entropy = make_test_seed([0xAB; 32], 256);
        rng1.reseed(&new_entropy).unwrap();

        // After reseed: different output
        rng1.fill_bytes(&mut out1);
        rng2.fill_bytes(&mut out2);
        assert_ne!(out1, out2);
    }

    #[test]
    fn test_different_entropy_different_result() {
        let initial_seed = [0x01u8; 32];
        let mut rng1 = ReseedableRng::from_seed_for_testing(initial_seed);
        let mut rng2 = ReseedableRng::from_seed_for_testing(initial_seed);

        let entropy1 = make_test_seed([0xAA; 32], 256);
        let entropy2 = make_test_seed([0xBB; 32], 256);

        rng1.reseed(&entropy1).unwrap();
        rng2.reseed(&entropy2).unwrap();

        let mut out1 = [0u8; 32];
        let mut out2 = [0u8; 32];
        rng1.fill_bytes(&mut out1);
        rng2.fill_bytes(&mut out2);

        assert_ne!(out1, out2);
    }

    #[test]
    fn test_reseed_counter_affects_output() {
        // Same entropy applied at different reseed counts should differ
        let initial_seed = [0x01u8; 32];
        let mut rng1 = ReseedableRng::from_seed_for_testing(initial_seed);
        let mut rng2 = ReseedableRng::from_seed_for_testing(initial_seed);

        let entropy = make_test_seed([0xAA; 32], 256);
        let dummy = make_test_seed([0x00; 32], 256);

        // rng1: reseed once
        rng1.reseed(&entropy).unwrap();

        // rng2: reseed twice (with dummy first)
        rng2.reseed(&dummy).unwrap();
        rng2.reseed(&entropy).unwrap();

        // Even though final entropy is same, counter differs
        let mut out1 = [0u8; 32];
        let mut out2 = [0u8; 32];
        rng1.fill_bytes(&mut out1);
        rng2.fill_bytes(&mut out2);

        assert_ne!(out1, out2);
    }
}
