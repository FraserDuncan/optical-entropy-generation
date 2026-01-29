//! Camera input and frame handling.
//!
//! This module provides abstractions for capturing frames from a camera
//! and managing camera configuration. The camera is treated as a source
//! of raw optical data, not as a source of entropy directly.

mod camera;
mod config;
mod frame;

pub use camera::{Camera, CameraError, CameraInfo, MockCamera};
#[cfg(feature = "camera")]
pub use camera::NokhwaCamera;
pub use config::{CaptureConfig, ConfigError, FileConfig, HealthConfig, OutputConfig};
pub use frame::Frame;
