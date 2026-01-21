//! Spatial decorrelation via pixel mixing.
//!
//! Reduces spatial correlations (adjacent pixel similarity) by
//! XORing pixels from different regions of the frame.

/// Mixes pixels spatially to reduce local correlations.
///
/// Adjacent pixels in camera images are often correlated.
/// This mixer XORs pixels from distant regions to break
/// spatial structure.
pub struct SpatialMixer {
    /// Mixing stride (pixels apart to XOR).
    stride: usize,
}

impl SpatialMixer {
    pub fn new() -> Self {
        Self { stride: 1 }
    }

    /// Creates a mixer with a custom stride.
    pub fn with_stride(stride: usize) -> Self {
        Self {
            stride: stride.max(1),
        }
    }

    /// Mixes the input data spatially.
    ///
    /// XORs each byte with a byte `stride` positions away,
    /// wrapping around at boundaries.
    pub fn mix(&self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let len = data.len();
        let stride = self.stride % len.max(1);

        data.iter()
            .enumerate()
            .map(|(i, &byte)| {
                let partner_idx = (i + stride) % len;
                byte ^ data[partner_idx]
            })
            .collect()
    }
}

impl Default for SpatialMixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let mixer = SpatialMixer::new();
        assert!(mixer.mix(&[]).is_empty());
    }

    #[test]
    fn test_self_xor_with_stride_zero() {
        // With stride 0 (treated as 1), each byte XORs with next
        let mixer = SpatialMixer::with_stride(0);
        let data = vec![0xAA, 0x55, 0xAA, 0x55];
        let result = mixer.mix(&data);

        // 0xAA ^ 0x55 = 0xFF for all positions
        assert!(result.iter().all(|&v| v == 0xFF));
    }

    #[test]
    fn test_identical_data_zeros() {
        let mixer = SpatialMixer::new();
        let data = vec![0x42; 100];
        let result = mixer.mix(&data);

        // Identical bytes XOR to zero
        assert!(result.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_varied_data_nonzero() {
        let mixer = SpatialMixer::with_stride(7);
        let data: Vec<u8> = (0..100).collect();
        let result = mixer.mix(&data);

        // Should produce non-zero output for varied input
        assert!(result.iter().any(|&v| v != 0));
    }
}
