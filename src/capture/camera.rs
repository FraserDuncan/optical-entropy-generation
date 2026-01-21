//! Camera abstraction for frame capture.
//!
//! This module provides a trait-based abstraction over camera hardware,
//! allowing for both real camera input and mock implementations for testing.

use super::{CaptureConfig, Frame};
use thiserror::Error;

/// Errors that can occur during camera operations.
#[derive(Debug, Error)]
pub enum CameraError {
    #[error("camera device not found: {0}")]
    DeviceNotFound(String),
    #[error("failed to open camera: {0}")]
    OpenFailed(String),
    #[error("failed to configure camera: {0}")]
    ConfigFailed(String),
    #[error("failed to capture frame: {0}")]
    CaptureFailed(String),
    #[error("camera not initialized")]
    NotInitialized,
}

/// Trait for camera implementations.
///
/// This abstraction allows swapping between real camera hardware
/// and mock implementations for testing.
pub trait Camera {
    /// Opens and initializes the camera with the given configuration.
    fn open(&mut self, config: &CaptureConfig) -> Result<(), CameraError>;

    /// Captures a single frame.
    fn capture(&mut self) -> Result<Frame, CameraError>;

    /// Checks if the camera is currently open.
    fn is_open(&self) -> bool;

    /// Closes the camera and releases resources.
    fn close(&mut self);
}

/// Mock camera for testing that generates synthetic frames.
#[derive(Debug, Default)]
pub struct MockCamera {
    config: Option<CaptureConfig>,
    sequence: u64,
}

impl MockCamera {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Camera for MockCamera {
    fn open(&mut self, config: &CaptureConfig) -> Result<(), CameraError> {
        config
            .validate()
            .map_err(|e| CameraError::ConfigFailed(e.to_string()))?;
        self.config = Some(config.clone());
        self.sequence = 0;
        tracing::info!("MockCamera opened with config: {:?}", config);
        Ok(())
    }

    fn capture(&mut self) -> Result<Frame, CameraError> {
        let config = self.config.as_ref().ok_or(CameraError::NotInitialized)?;

        // Generate synthetic noise pattern for testing
        let pixel_count = (config.width * config.height) as usize;
        let pixels: Vec<u8> = (0..pixel_count)
            .map(|i| {
                // Simple deterministic pattern mixed with sequence
                // NOT for entropy - only for testing frame handling
                ((i as u64 ^ self.sequence) % 256) as u8
            })
            .collect();

        self.sequence += 1;
        Ok(Frame::new(pixels, config.width, config.height, self.sequence))
    }

    fn is_open(&self) -> bool {
        self.config.is_some()
    }

    fn close(&mut self) {
        self.config = None;
        tracing::info!("MockCamera closed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_camera_lifecycle() {
        let mut camera = MockCamera::new();
        let config = CaptureConfig::default();

        assert!(!camera.is_open());

        camera.open(&config).unwrap();
        assert!(camera.is_open());

        let frame = camera.capture().unwrap();
        assert!(frame.is_valid());
        assert_eq!(frame.sequence(), 1);

        let frame2 = camera.capture().unwrap();
        assert_eq!(frame2.sequence(), 2);

        camera.close();
        assert!(!camera.is_open());
    }

    #[test]
    fn test_capture_without_open() {
        let mut camera = MockCamera::new();
        assert!(matches!(
            camera.capture(),
            Err(CameraError::NotInitialized)
        ));
    }
}
