//! Application settings persistence
//!
//! Stores settings as JSON in the platform-specific app data directory.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

use crate::DeviceId;

/// Application settings for device routing and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Selected monitor output device ID
    pub monitor_device_id: Option<DeviceId>,
    /// Selected broadcast output device ID
    pub broadcast_device_id: Option<DeviceId>,
    /// Default volume (0.0 - 1.0)
    pub default_volume: f32,
    /// Global volume multiplier for all sounds (0.1 - 1.0), default 0.2
    #[serde(default = "default_volume_multiplier")]
    pub volume_multiplier: f32,
    /// Last used audio file path (for convenience)
    pub last_file_path: Option<String>,
}

fn default_volume_multiplier() -> f32 {
    1.0 // Default: disabled (no boost), sounds play at normal Windows volume
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            monitor_device_id: None,
            broadcast_device_id: None,
            default_volume: 0.5,
            volume_multiplier: default_volume_multiplier(),
            last_file_path: None,
        }
    }
}

/// Get the path to the settings file
pub fn get_settings_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Ensure directory exists
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    Ok(app_data_dir.join("settings.json"))
}

/// Load application settings from disk
pub fn load(app_handle: &tauri::AppHandle) -> Result<AppSettings, String> {
    let settings_path = get_settings_path(app_handle)?;

    if !settings_path.exists() {
        // Return default settings if file doesn't exist
        return Ok(AppSettings::default());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings: AppSettings =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?;

    Ok(settings)
}

/// Save application settings to disk
pub fn save(settings: &AppSettings, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let settings_path = get_settings_path(app_handle)?;

    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, json)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}
