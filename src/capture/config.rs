//! Camera capture configuration.
//!
//! Fixed exposure and gain settings are critical for consistent
//! entropy characteristics. Auto-exposure would introduce
//! unpredictable correlations.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for camera capture.
///
/// All settings are fixed to ensure consistent entropy characteristics.
/// Auto-exposure and auto-gain are explicitly disabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Camera device index or identifier.
    pub device_id: u32,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Fixed exposure time in microseconds.
    pub exposure_us: u32,
    /// Fixed gain value (camera-specific units).
    pub gain: u32,
    /// Target frames per second.
    pub fps: u32,
    /// Use grayscale mode (recommended for entropy extraction).
    pub grayscale: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            device_id: 0,
            width: 640,
            height: 480,
            exposure_us: 10000, // 10ms
            gain: 1,
            fps: 30,
            grayscale: true,
        }
    }
}

impl CaptureConfig {
    /// Creates a new configuration with the specified dimensions.
    pub fn with_dimensions(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Validates the configuration parameters.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.width == 0 || self.height == 0 {
            return Err(ConfigError::InvalidDimensions);
        }
        if self.exposure_us == 0 {
            return Err(ConfigError::InvalidExposure);
        }
        if self.fps == 0 || self.fps > 120 {
            return Err(ConfigError::InvalidFrameRate);
        }
        Ok(())
    }
}

/// Configuration validation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid frame dimensions")]
    InvalidDimensions,
    #[error("invalid exposure time")]
    InvalidExposure,
    #[error("invalid frame rate (must be 1-120 fps)")]
    InvalidFrameRate,
    #[error("failed to read config file: {0}")]
    FileReadError(String),
    #[error("failed to parse config file: {0}")]
    ParseError(String),
}

/// Full configuration file format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileConfig {
    #[serde(default)]
    pub capture: CaptureConfig,
    #[serde(default)]
    pub health: HealthConfig,
    #[serde(default)]
    pub output: OutputConfig,
}

/// Health monitoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Minimum consecutive healthy samples before allowing reseed.
    pub min_healthy_streak: u32,
    /// Maximum bit bias allowed (0.0 to 0.5).
    pub max_bias: f64,
    /// Minimum variance required.
    pub min_variance: f64,
    /// Maximum autocorrelation allowed.
    pub max_autocorrelation: f64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            min_healthy_streak: 3,
            max_bias: 0.1,
            min_variance: 100.0,
            max_autocorrelation: 0.5,
        }
    }
}

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Run continuously (true) or process fixed number of frames (false).
    pub continuous: bool,
    /// Number of frames to process if not continuous.
    pub frame_count: u32,
    /// Metrics server port (0 to disable).
    pub metrics_port: u16,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            continuous: false,
            frame_count: 100,
            metrics_port: 9090,
        }
    }
}

impl FileConfig {
    /// Loads configuration from a TOML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::FileReadError(e.to_string()))?;
        let config: FileConfig =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        config.capture.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_valid() {
        let config = CaptureConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_zero_dimensions_invalid() {
        let mut config = CaptureConfig::default();
        config.width = 0;
        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidDimensions)
        ));
    }
}
