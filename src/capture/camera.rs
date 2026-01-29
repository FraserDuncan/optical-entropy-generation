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

/// Real camera implementation using nokhwa.
#[cfg(feature = "camera")]
pub mod real {
    use super::*;
    use nokhwa::pixel_format::RgbFormat;
    use nokhwa::utils::{
        CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
    };
    use nokhwa::Camera as NokhwaCamera_;

    /// Camera implementation using nokhwa for real hardware access.
    pub struct NokhwaCamera {
        camera: Option<NokhwaCamera_>,
        config: Option<CaptureConfig>,
        sequence: u64,
    }

    impl NokhwaCamera {
        pub fn new() -> Self {
            Self {
                camera: None,
                config: None,
                sequence: 0,
            }
        }

        /// Lists all available camera devices.
        pub fn list_devices() -> Result<Vec<CameraInfo>, CameraError> {
            let devices = nokhwa::query(nokhwa::utils::ApiBackend::Auto)
                .map_err(|e| CameraError::DeviceNotFound(e.to_string()))?;

            Ok(devices
                .into_iter()
                .map(|info| CameraInfo {
                    index: match info.index() {
                        CameraIndex::Index(i) => *i,
                        CameraIndex::String(s) => {
                            s.parse().unwrap_or(0)
                        }
                    },
                    name: info.human_name().to_string(),
                    description: info.description().to_string(),
                })
                .collect())
        }
    }

    impl Default for NokhwaCamera {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Camera for NokhwaCamera {
        fn open(&mut self, config: &CaptureConfig) -> Result<(), CameraError> {
            config
                .validate()
                .map_err(|e| CameraError::ConfigFailed(e.to_string()))?;

            let index = CameraIndex::Index(config.device_id);
            let resolution = Resolution::new(config.width, config.height);

            let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(
                CameraFormat::new(resolution, FrameFormat::RAWRGB, config.fps),
            ));

            let mut camera = NokhwaCamera_::new(index, format)
                .map_err(|e| CameraError::OpenFailed(e.to_string()))?;

            camera
                .open_stream()
                .map_err(|e| CameraError::OpenFailed(e.to_string()))?;

            tracing::info!(
                "Opened camera {} at {}x{} @ {} fps",
                config.device_id,
                config.width,
                config.height,
                config.fps
            );

            self.camera = Some(camera);
            self.config = Some(config.clone());
            self.sequence = 0;

            Ok(())
        }

        fn capture(&mut self) -> Result<Frame, CameraError> {
            let camera = self.camera.as_mut().ok_or(CameraError::NotInitialized)?;
            let config = self.config.as_ref().ok_or(CameraError::NotInitialized)?;

            let frame = camera
                .frame()
                .map_err(|e| CameraError::CaptureFailed(e.to_string()))?;

            let rgb_data = frame.decode_image::<RgbFormat>()
                .map_err(|e| CameraError::CaptureFailed(e.to_string()))?;

            // Convert to grayscale if configured
            let pixels: Vec<u8> = if config.grayscale {
                rgb_data
                    .pixels()
                    .map(|p| {
                        // Standard luminance conversion
                        let r = p[0] as f32;
                        let g = p[1] as f32;
                        let b = p[2] as f32;
                        (0.299 * r + 0.587 * g + 0.114 * b) as u8
                    })
                    .collect()
            } else {
                rgb_data.into_raw()
            };

            self.sequence += 1;

            Ok(Frame::new(
                pixels,
                config.width,
                config.height,
                self.sequence,
            ))
        }

        fn is_open(&self) -> bool {
            self.camera.is_some()
        }

        fn close(&mut self) {
            if let Some(mut camera) = self.camera.take() {
                let _ = camera.stop_stream();
            }
            self.config = None;
            tracing::info!("Camera closed");
        }
    }

    impl Drop for NokhwaCamera {
        fn drop(&mut self) {
            self.close();
        }
    }
}

/// Information about an available camera device.
#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub index: u32,
    pub name: String,
    pub description: String,
}

#[cfg(feature = "camera")]
pub use real::NokhwaCamera;

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
