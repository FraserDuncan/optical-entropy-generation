//! Bit harvesting and decorrelation.
//!
//! This module converts raw camera frames into decorrelated bitstreams
//! suitable for entropy conditioning. It applies temporal and spatial
//! transformations to reduce structure and correlations in the raw data.

mod bitstream;
mod spatial;
mod temporal;

pub use bitstream::RawBits;
pub use spatial::SpatialMixer;
pub use temporal::TemporalDifferencer;

use crate::capture::Frame;

/// Extracts raw bits from a sequence of frames.
///
/// Combines temporal differencing and spatial mixing to produce
/// a decorrelated bitstream from raw camera input.
pub struct Extractor {
    temporal: TemporalDifferencer,
    spatial: SpatialMixer,
}

impl Extractor {
    pub fn new() -> Self {
        Self {
            temporal: TemporalDifferencer::new(),
            spatial: SpatialMixer::new(),
        }
    }

    /// Processes a frame and returns extracted bits if ready.
    ///
    /// Returns `None` if more frames are needed (e.g., for differencing).
    pub fn process(&mut self, frame: &Frame) -> Option<RawBits> {
        // Apply temporal differencing
        let diff = self.temporal.difference(frame)?;

        // Apply spatial mixing
        let mixed = self.spatial.mix(&diff);

        Some(RawBits::from_bytes(mixed, frame.sequence()))
    }

    /// Resets internal state (e.g., after quality failure).
    pub fn reset(&mut self) {
        self.temporal.reset();
    }
}

impl Default for Extractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_needs_two_frames() {
        let mut extractor = Extractor::new();

        let frame1 = Frame::new(vec![100u8; 64], 8, 8, 1);
        let frame2 = Frame::new(vec![150u8; 64], 8, 8, 2);

        // First frame: no output (need previous for differencing)
        assert!(extractor.process(&frame1).is_none());

        // Second frame: should produce output
        let bits = extractor.process(&frame2);
        assert!(bits.is_some());
    }
}
