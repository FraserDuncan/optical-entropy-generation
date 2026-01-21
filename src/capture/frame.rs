//! Frame type representing a captured image with metadata.

use std::time::Instant;

/// A single captured frame from the camera.
///
/// Contains raw pixel data along with metadata needed for
/// temporal correlation analysis and debugging.
#[derive(Clone)]
pub struct Frame {
    /// Raw pixel data (grayscale or RGB depending on config).
    pixels: Vec<u8>,
    /// Frame width in pixels.
    width: u32,
    /// Frame height in pixels.
    height: u32,
    /// Capture timestamp for temporal analysis.
    timestamp: Instant,
    /// Monotonic sequence number.
    sequence: u64,
}

impl Frame {
    /// Creates a new frame with the given parameters.
    pub fn new(pixels: Vec<u8>, width: u32, height: u32, sequence: u64) -> Self {
        Self {
            pixels,
            width,
            height,
            timestamp: Instant::now(),
            sequence,
        }
    }

    /// Returns a reference to the raw pixel data.
    #[inline]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Returns the frame width.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the frame height.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the capture timestamp.
    #[inline]
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Returns the sequence number.
    #[inline]
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the total number of pixels (width * height).
    #[inline]
    pub fn pixel_count(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    /// Validates that the pixel buffer size matches dimensions.
    pub fn is_valid(&self) -> bool {
        self.pixels.len() == self.pixel_count()
    }
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("sequence", &self.sequence)
            .field("pixel_bytes", &self.pixels.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let pixels = vec![0u8; 640 * 480];
        let frame = Frame::new(pixels, 640, 480, 1);

        assert_eq!(frame.width(), 640);
        assert_eq!(frame.height(), 480);
        assert_eq!(frame.sequence(), 1);
        assert!(frame.is_valid());
    }

    #[test]
    fn test_frame_invalid_size() {
        let pixels = vec![0u8; 100]; // Wrong size
        let frame = Frame::new(pixels, 640, 480, 1);

        assert!(!frame.is_valid());
    }
}
