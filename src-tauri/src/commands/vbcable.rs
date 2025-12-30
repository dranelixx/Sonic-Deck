//! VB-Cable related Tauri commands

use crate::vbcable::{
    cleanup_temp_files, detect_vb_cable, disable_routing, enable_routing, get_routing_status,
    install_vbcable, list_capture_devices, uninstall_vbcable, wait_for_vb_cable,
    DefaultDeviceManager, RestoreResult, SavedDefaults, VbCableStatus,
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
/// Returns structured result showing which devices were restored and which failed.
#[tauri::command]
pub fn restore_all_default_devices(saved: SavedDefaults) -> RestoreResult {
    info!("Restoring all default audio devices");
    DefaultDeviceManager::restore_all_defaults(&saved)
}

/// Wait for VB-Cable device to appear after installation (with smart retries)
///
/// Uses active retry logic that polls for the device up to 5 times with 1s delays.
/// Returns early as soon as device is detected, or None if not detected after retries.
#[tauri::command]
pub fn wait_for_vb_cable_device() -> Option<String> {
    info!("Waiting for VB-Cable device with retry logic...");
    wait_for_vb_cable().map(|info| info.output_device)
}

// ============================================================================
// Microphone Routing Commands
// ============================================================================

/// List available microphones (capture devices)
///
/// Returns a list of (device_id, display_name) tuples.
/// Excludes VB-Cable devices since we only want physical microphones.
#[tauri::command]
pub fn list_microphones() -> Vec<(String, String)> {
    info!("Listing available microphones");
    list_capture_devices()
}

/// Enable microphone routing to CABLE Input
///
/// Routes audio from the specified microphone to VB-Cable's CABLE Input device.
/// This allows the user's voice to be heard on Discord while using VB-Cable.
#[tauri::command]
pub fn enable_microphone_routing(microphone_id: String) -> Result<(), String> {
    info!("Enabling microphone routing for device: {}", microphone_id);
    enable_routing(&microphone_id)
}

/// Disable microphone routing
///
/// Stops routing microphone audio to CABLE Input.
#[tauri::command]
pub fn disable_microphone_routing() -> Result<(), String> {
    info!("Disabling microphone routing");
    disable_routing()
}

/// Get microphone routing status
///
/// Returns the device ID of the currently routed microphone, or None if not active.
#[tauri::command]
pub fn get_microphone_routing_status() -> Option<String> {
    get_routing_status()
}

// ============================================================================
// VB-Cable Uninstall Command
// ============================================================================

/// Uninstall VB-Cable
///
/// Downloads the installer if not cached and runs it with -u flag for uninstall.
/// Will trigger UAC prompt for admin rights.
#[tauri::command]
pub fn start_vb_cable_uninstall() -> Result<(), String> {
    info!("Starting VB-Cable uninstallation from frontend request");
    uninstall_vbcable()
}

// ============================================================================
// Sound Settings Command
// ============================================================================

/// Open Windows Sound Control Panel (mmsys.cpl)
///
/// Opens the classic Windows Sound settings where users can manage audio devices.
#[tauri::command]
pub fn open_sound_settings() -> Result<(), String> {
    info!("Opening Windows Sound settings (mmsys.cpl)");
    open::that("mmsys.cpl").map_err(|e| format!("Failed to open sound settings: {}", e))
}
