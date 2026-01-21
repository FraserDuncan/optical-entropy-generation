//! Camera input and frame handling.
//!
//! This module provides abstractions for capturing frames from a camera
//! and managing camera configuration. The camera is treated as a source
//! of raw optical data, not as a source of entropy directly.

mod camera;
mod config;
mod frame;

pub use camera::{Camera, CameraError, MockCamera};
pub use config::CaptureConfig;
pub use frame::Frame;
