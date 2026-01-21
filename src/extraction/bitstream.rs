//! Raw bitstream type for extracted entropy.

/// Raw bits extracted from camera frames.
///
/// This is the output of the extraction stage and input to conditioning.
/// The bits are decorrelated but not yet cryptographically processed.
#[derive(Clone)]
pub struct RawBits {
    /// Raw byte data.
    data: Vec<u8>,
    /// Number of source frames that contributed.
    source_frames: u64,
}

impl RawBits {
    /// Creates a new RawBits from byte data.
    pub fn from_bytes(data: Vec<u8>, source_frames: u64) -> Self {
        Self {
            data,
            source_frames,
        }
    }

    /// Returns the raw byte data.
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the number of bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the number of bits.
    #[inline]
    pub fn bit_count(&self) -> usize {
        self.data.len() * 8
    }

    /// Returns the source frame count.
    #[inline]
    pub fn source_frames(&self) -> u64 {
        self.source_frames
    }

    /// Counts the number of set bits (for bias analysis).
    pub fn popcount(&self) -> usize {
        self.data.iter().map(|b| b.count_ones() as usize).sum()
    }

    /// Calculates bit bias as deviation from 0.5.
    ///
    /// Returns a value in [-0.5, 0.5] where 0.0 is unbiased.
    pub fn bit_bias(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        let ones = self.popcount() as f64;
        let total = self.bit_count() as f64;
        (ones / total) - 0.5
    }
}

impl std::fmt::Debug for RawBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawBits")
            .field("bytes", &self.data.len())
            .field("source_frames", &self.source_frames)
            .field("bit_bias", &format!("{:.4}", self.bit_bias()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unbiased_data() {
        // Alternating bits: 0xAA = 10101010
        let data = vec![0xAA; 100];
        let bits = RawBits::from_bytes(data, 1);

        // Should be perfectly unbiased
        assert!((bits.bit_bias()).abs() < 0.001);
    }

    #[test]
    fn test_all_ones_bias() {
        let data = vec![0xFF; 100];
        let bits = RawBits::from_bytes(data, 1);

        // All ones = bias of +0.5
        assert!((bits.bit_bias() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_all_zeros_bias() {
        let data = vec![0x00; 100];
        let bits = RawBits::from_bytes(data, 1);

        // All zeros = bias of -0.5
        assert!((bits.bit_bias() + 0.5).abs() < 0.001);
    }
}
