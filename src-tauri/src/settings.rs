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
    /// Close button behavior: true = minimize to tray, false = quit app
    #[serde(default = "default_minimize_to_tray")]
    pub minimize_to_tray: bool,
    /// Start application minimized to tray
    #[serde(default)]
    pub start_minimized: bool,
    /// Enable autostart on system boot
    #[serde(default)]
    pub autostart_enabled: bool,
    /// Microphone device ID for VB-Cable routing
    #[serde(default)]
    pub microphone_routing_device_id: Option<String>,
    /// Whether microphone routing is enabled
    #[serde(default)]
    pub microphone_routing_enabled: bool,
}

fn default_volume_multiplier() -> f32 {
    1.0 // Default: disabled (no boost), sounds play at normal Windows volume
}

fn default_minimize_to_tray() -> bool {
    true // Default: close minimizes to tray
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            monitor_device_id: None,
            broadcast_device_id: None,
            default_volume: 0.5,
            volume_multiplier: default_volume_multiplier(),
            last_file_path: None,
            minimize_to_tray: default_minimize_to_tray(),
            start_minimized: false,
            autostart_enabled: false,
            microphone_routing_device_id: None,
            microphone_routing_enabled: false,
        }
    }
}

/// Get the path to the settings file
pub fn get_settings_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_local_data_dir()
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

/// Save application settings to disk (atomic write)
pub fn save(settings: &AppSettings, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let settings_path = get_settings_path(app_handle)?;

    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    crate::persistence::atomic_write(&settings_path, &json)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Default Value Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_volume_multiplier() {
        assert_eq!(default_volume_multiplier(), 1.0);
    }

    #[test]
    fn test_default_minimize_to_tray() {
        assert!(default_minimize_to_tray());
    }

    // -------------------------------------------------------------------------
    // AppSettings::default() Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();

        assert_eq!(settings.monitor_device_id, None);
        assert_eq!(settings.broadcast_device_id, None);
        assert_eq!(settings.default_volume, 0.5);
        assert_eq!(settings.volume_multiplier, 1.0);
        assert_eq!(settings.last_file_path, None);
        assert!(settings.minimize_to_tray);
        assert!(!settings.start_minimized);
        assert!(!settings.autostart_enabled);
        assert_eq!(settings.microphone_routing_device_id, None);
        assert!(!settings.microphone_routing_enabled);
    }

    #[test]
    fn test_app_settings_default_volume_range() {
        let settings = AppSettings::default();
        assert!(settings.default_volume >= 0.0 && settings.default_volume <= 1.0);
    }

    // -------------------------------------------------------------------------
    // Serialization Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_app_settings_serde_roundtrip() {
        let settings = AppSettings::default();

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.default_volume, settings.default_volume);
        assert_eq!(deserialized.volume_multiplier, settings.volume_multiplier);
        assert_eq!(deserialized.minimize_to_tray, settings.minimize_to_tray);
    }

    #[test]
    fn test_app_settings_serde_with_devices() {
        let settings = AppSettings {
            monitor_device_id: Some(DeviceId::from_index(0)),
            broadcast_device_id: Some(DeviceId::from_index(1)),
            default_volume: 0.75,
            volume_multiplier: 0.5,
            last_file_path: Some("/path/to/file.mp3".to_string()),
            minimize_to_tray: false,
            start_minimized: true,
            autostart_enabled: true,
            microphone_routing_device_id: Some("device_2".to_string()),
            microphone_routing_enabled: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.monitor_device_id,
            Some(DeviceId::from_index(0))
        );
        assert_eq!(
            deserialized.broadcast_device_id,
            Some(DeviceId::from_index(1))
        );
        assert_eq!(deserialized.default_volume, 0.75);
        assert_eq!(deserialized.volume_multiplier, 0.5);
        assert_eq!(
            deserialized.last_file_path,
            Some("/path/to/file.mp3".to_string())
        );
        assert!(!deserialized.minimize_to_tray);
        assert!(deserialized.start_minimized);
        assert!(deserialized.autostart_enabled);
        assert_eq!(
            deserialized.microphone_routing_device_id,
            Some("device_2".to_string())
        );
        assert!(deserialized.microphone_routing_enabled);
    }

    #[test]
    fn test_app_settings_deserialize_with_defaults() {
        // Simulate JSON without optional fields (serde should use defaults)
        let json = r#"{
            "monitor_device_id": null,
            "broadcast_device_id": null,
            "default_volume": 0.5,
            "last_file_path": null
        }"#;

        let settings: AppSettings = serde_json::from_str(json).unwrap();

        // These fields should use their #[serde(default)] values
        assert_eq!(settings.volume_multiplier, 1.0);
        assert!(settings.minimize_to_tray);
        assert!(!settings.start_minimized);
        assert!(!settings.autostart_enabled);
        assert_eq!(settings.microphone_routing_device_id, None);
        assert!(!settings.microphone_routing_enabled);
    }

    #[test]
    fn test_app_settings_json_format() {
        let settings = AppSettings::default();
        let json = serde_json::to_string_pretty(&settings).unwrap();

        // Verify it's valid JSON and contains expected fields
        assert!(json.contains("monitor_device_id"));
        assert!(json.contains("broadcast_device_id"));
        assert!(json.contains("default_volume"));
        assert!(json.contains("volume_multiplier"));
        assert!(json.contains("minimize_to_tray"));
    }

    // -------------------------------------------------------------------------
    // DeviceId Integration Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_device_id_in_settings() {
        let device = DeviceId::from_index(5);
        let settings = AppSettings {
            monitor_device_id: Some(device.clone()),
            ..AppSettings::default()
        };

        assert_eq!(
            settings.monitor_device_id.as_ref().unwrap().to_string(),
            "device_5"
        );
    }
}
