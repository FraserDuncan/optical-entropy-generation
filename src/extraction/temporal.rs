//! Temporal decorrelation via frame differencing.
//!
//! Removes static patterns by computing differences between
//! consecutive frames. Only changes between frames contribute
//! to the output, reducing the impact of fixed-pattern noise.

use crate::capture::Frame;

/// Computes differences between consecutive frames.
///
/// This reduces static patterns (dead pixels, fixed noise) and
/// emphasizes temporal changes in the optical signal.
pub struct TemporalDifferencer {
    /// Previous frame for differencing.
    previous: Option<Frame>,
}

impl TemporalDifferencer {
    pub fn new() -> Self {
        Self { previous: None }
    }

    /// Computes the absolute difference with the previous frame.
    ///
    /// Returns `None` on the first frame (no previous to compare).
    pub fn difference(&mut self, current: &Frame) -> Option<Vec<u8>> {
        let result = self.previous.as_ref().map(|prev| {
            // Compute absolute difference pixel by pixel
            current
                .pixels()
                .iter()
                .zip(prev.pixels().iter())
                .map(|(&c, &p)| c.abs_diff(p))
                .collect()
        });

        // Store current as previous for next call
        self.previous = Some(current.clone());

        result
    }

    /// Resets the differencer state.
    pub fn reset(&mut self) {
        self.previous = None;
    }

    /// Returns true if ready to produce output.
    pub fn is_primed(&self) -> bool {
        self.previous.is_some()
    }
}

impl Default for TemporalDifferencer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_frame_returns_none() {
        let mut diff = TemporalDifferencer::new();
        let frame = Frame::new(vec![100u8; 64], 8, 8, 1);

        assert!(diff.difference(&frame).is_none());
        assert!(diff.is_primed());
    }

    #[test]
    fn test_second_frame_returns_difference() {
        let mut diff = TemporalDifferencer::new();

        let frame1 = Frame::new(vec![100u8; 64], 8, 8, 1);
        let frame2 = Frame::new(vec![150u8; 64], 8, 8, 2);

        diff.difference(&frame1);
        let result = diff.difference(&frame2).unwrap();

        // All pixels should have difference of 50
        assert!(result.iter().all(|&v| v == 50));
    }

    #[test]
    fn test_identical_frames_zero_difference() {
        let mut diff = TemporalDifferencer::new();

        let frame1 = Frame::new(vec![100u8; 64], 8, 8, 1);
        let frame2 = Frame::new(vec![100u8; 64], 8, 8, 2);

        diff.difference(&frame1);
        let result = diff.difference(&frame2).unwrap();

        // Identical frames = zero difference
        assert!(result.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_reset_requires_new_prime() {
        let mut diff = TemporalDifferencer::new();

        let frame = Frame::new(vec![100u8; 64], 8, 8, 1);
        diff.difference(&frame);
        assert!(diff.is_primed());

        diff.reset();
        assert!(!diff.is_primed());
    }
}
