//! VB-Cable related Tauri commands

use crate::vbcable::{detect_vb_cable, DefaultDeviceManager, VbCableStatus};

/// Check if VB-Cable is installed and get its status
#[tauri::command]
pub fn check_vb_cable_status() -> VbCableStatus {
    if let Some(info) = detect_vb_cable() {
        VbCableStatus::Installed { info }
    } else {
        VbCableStatus::NotInstalled
    }
}

/// Get the VB-Cable output device name if installed
///
/// Returns the device name for use in device selection dropdowns.
#[tauri::command]
pub fn get_vb_cable_device_name() -> Option<String> {
    detect_vb_cable().map(|info| info.output_device)
}

/// Save the current default audio device
///
/// Call this before VB-Cable installation to preserve the user's original default device.
/// Returns the saved device ID on success for use with restore_default_audio_device.
#[tauri::command]
pub fn save_default_audio_device() -> Result<String, String> {
    let manager = DefaultDeviceManager::save_current_default()?;
    manager
        .get_saved_device_id()
        .ok_or_else(|| "No device saved".to_string())
}

/// Restore a previously saved default audio device
///
/// Call this after VB-Cable installation to restore the user's original default device.
/// Pass the device_id returned from save_default_audio_device.
#[tauri::command]
pub fn restore_default_audio_device(device_id: String) -> Result<(), String> {
    DefaultDeviceManager::restore_device(&device_id)
}
