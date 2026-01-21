//! Optical Entropy Generation Library
//!
//! A physical entropy source using optical phenomena and camera capture.
//! Provides conditioned entropy for reseeding cryptographically secure
//! pseudorandom number generators (CSPRNGs).
//!
//! # Architecture
//!
//! The system follows an explicit data flow:
//!
//! ```text
//! capture → extraction → conditioning → reseeding
//!     ↓          ↓            ↓
//!           analysis (health monitoring)
//! ```
//!
//! # Design Principles
//!
//! - **Fail-closed**: Reseeding is suspended if entropy quality degrades
//! - **Supplements OS entropy**: Does not replace system randomness
//! - **Uses standard primitives**: BLAKE3/SHA-256 for conditioning, ChaCha20 for CSPRNG
//! - **No cryptographic claims**: Statistical tests are sanity checks, not proofs
//!
//! # Example
//!
//! ```no_run
//! use optical_entropy::{
//!     capture::{MockCamera, Camera, CaptureConfig},
//!     extraction::Extractor,
//!     conditioning::EntropyPool,
//!     analysis::HealthMonitor,
//!     reseeding::ReseedableRng,
//! };
//!
//! // Initialize components
//! let mut camera = MockCamera::new();
//! camera.open(&CaptureConfig::default()).unwrap();
//!
//! let mut extractor = Extractor::new();
//! let mut pool = EntropyPool::default();
//! let mut health = HealthMonitor::default();
//! let mut rng = ReseedableRng::from_os_entropy();
//!
//! // Capture and process frames
//! for _ in 0..10 {
//!     let frame = camera.capture().unwrap();
//!
//!     if let Some(bits) = extractor.process(&frame) {
//!         health.analyze(&bits);
//!         pool.add(&bits);
//!     }
//! }
//!
//! // Reseed if healthy and pool is ready
//! if health.allow_reseed() {
//!     if let Some(seed) = pool.extract() {
//!         rng.reseed(&seed).unwrap();
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_code)]

pub mod analysis;
pub mod capture;
pub mod conditioning;
pub mod extraction;
pub mod reseeding;

// Re-export commonly used types at crate root
pub use analysis::{HealthMetrics, HealthMonitor, QualityThresholds};
pub use capture::{Camera, CaptureConfig, Frame, MockCamera};
pub use conditioning::{Conditioner, ConditionedSeed, EntropyPool, HashAlgorithm};
pub use extraction::{Extractor, RawBits};
pub use reseeding::ReseedableRng;

/// Library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
