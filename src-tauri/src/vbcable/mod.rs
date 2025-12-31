//! VB-Cable integration module
//!
//! Provides VB-Cable detection, installation, Windows default audio device management,
//! microphone routing, and automatic communications device switching for Discord integration.

mod communications;
mod default_device;
mod detection;
mod installer;
mod microphone;

pub use communications::{
    activate as activate_comm_mode, deactivate as deactivate_comm_mode,
    is_active as is_comm_mode_active, recover_from_crash as recover_comm_mode,
};
pub use default_device::{DefaultDeviceManager, RestoreResult, SavedDefaults};
pub use detection::{detect_vb_cable, wait_for_vb_cable, VbCableStatus};
pub use installer::{cleanup_temp_files, install_vbcable, uninstall_vbcable};
pub use microphone::{disable_routing, enable_routing, get_routing_status, list_capture_devices};
