//! VB-Cable integration module
//!
//! Provides VB-Cable detection, installation, Windows default audio device management,
//! and microphone routing for Discord integration.

mod default_device;
mod detection;
mod installer;
mod microphone;

pub use default_device::{DefaultDeviceManager, SavedDefaults};
pub use detection::{detect_vb_cable, wait_for_vb_cable, VbCableInfo, VbCableStatus};
pub use installer::{cleanup_temp_files, install_vbcable};
pub use microphone::{disable_routing, enable_routing, get_routing_status, list_capture_devices};

// This is available but not currently used by commands - kept for future use
#[allow(unused_imports)]
pub use detection::is_vb_cable_installed;
