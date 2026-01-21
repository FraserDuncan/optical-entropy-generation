//! Entropy conditioning via cryptographic hashing.
//!
//! This module transforms raw extracted bits into uniformly distributed
//! entropy suitable for CSPRNG reseeding. It uses well-established
//! cryptographic hash functions to remove bias and correlations.

mod hash;
mod pool;

pub use hash::{ConditionedSeed, Conditioner, HashAlgorithm};
pub use pool::{EntropyPool, PoolConfig};
