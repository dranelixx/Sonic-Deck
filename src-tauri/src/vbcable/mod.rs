//! VB-Cable integration module
//!
//! Provides VB-Cable detection and Windows default audio device management.

mod default_device;
mod detection;

pub use default_device::DefaultDeviceManager;
pub use detection::{detect_vb_cable, VbCableStatus};

// These are available but not currently used by commands - kept for future use
#[allow(unused_imports)]
pub use detection::{is_vb_cable_installed, VbCableInfo};
