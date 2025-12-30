//! VB-Cable related Tauri commands

use crate::vbcable::{
    cleanup_temp_files, detect_vb_cable, install_vbcable, DefaultDeviceManager, SavedDefaults,
    VbCableStatus,
};
use tracing::info;

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

/// Start VB-Cable installation (download + silent install)
///
/// Frontend should call save_default_audio_device BEFORE this.
/// The installation is run synchronously (blocking) - Windows will show a driver
/// approval dialog that the user must accept.
#[tauri::command]
pub fn start_vb_cable_install() -> Result<(), String> {
    info!("Starting VB-Cable installation from frontend request");
    install_vbcable()
}

/// Cleanup temporary installation files
///
/// Call this after installation to remove downloaded ZIP and extracted files.
#[tauri::command]
pub fn cleanup_vb_cable_install() {
    info!("Cleaning up VB-Cable installation files");
    cleanup_temp_files();
}

/// Open VB-Audio website (fallback if automated install fails)
#[tauri::command]
pub fn open_vb_audio_website() -> Result<(), String> {
    info!("Opening VB-Audio website in browser");
    open::that("https://vb-audio.com/Cable/").map_err(|e| format!("Failed to open browser: {}", e))
}

/// Save ALL default audio devices (render/capture, console/communications)
///
/// Call this before VB-Cable installation to preserve all user's default devices.
/// Returns a struct with all 4 device IDs.
#[tauri::command]
pub fn save_all_default_devices() -> Result<SavedDefaults, String> {
    info!("Saving all default audio devices");
    DefaultDeviceManager::save_all_defaults()
}

/// Restore ALL default audio devices
///
/// Call this after VB-Cable installation to restore all user's original defaults.
#[tauri::command]
pub fn restore_all_default_devices(saved: SavedDefaults) -> Result<(), String> {
    info!("Restoring all default audio devices");
    DefaultDeviceManager::restore_all_defaults(&saved)
}
