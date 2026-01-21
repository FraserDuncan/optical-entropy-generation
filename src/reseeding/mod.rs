//! CSPRNG reseeding interface.
//!
//! This module provides a wrapper around ChaCha-based CSPRNGs
//! with support for reseeding from conditioned entropy.

mod csprng;

pub use csprng::{ReseedableRng, ReseedingError};
