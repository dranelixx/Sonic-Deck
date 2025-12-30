//! VB-Cable integration module
//!
//! Provides VB-Cable detection, installation, and Windows default audio device management.

mod default_device;
mod detection;
mod installer;

pub use default_device::{DefaultDeviceManager, SavedDefaults};
pub use detection::{detect_vb_cable, wait_for_vb_cable, VbCableInfo, VbCableStatus};
pub use installer::{cleanup_temp_files, install_vbcable};

// This is available but not currently used by commands - kept for future use
#[allow(unused_imports)]
pub use detection::is_vb_cable_installed;
