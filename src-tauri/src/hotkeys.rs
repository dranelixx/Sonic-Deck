//! Global hotkey management and persistence

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::Manager;

use crate::SoundId;

/// Hotkey mappings: keyboard shortcut string -> sound ID
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HotkeyMappings {
    pub mappings: HashMap<String, SoundId>,
}

/// Get the path to the hotkeys file
pub fn get_hotkeys_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Ensure directory exists
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    Ok(app_data_dir.join("hotkeys.json"))
}

/// Load hotkey mappings from disk
pub fn load(app_handle: &tauri::AppHandle) -> Result<HotkeyMappings, String> {
    let hotkeys_path = get_hotkeys_path(app_handle)?;

    if !hotkeys_path.exists() {
        // Return default empty mappings if file doesn't exist
        return Ok(HotkeyMappings::default());
    }

    let content = std::fs::read_to_string(&hotkeys_path)
        .map_err(|e| format!("Failed to read hotkeys file: {}", e))?;

    let mappings: HotkeyMappings =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse hotkeys: {}", e))?;

    Ok(mappings)
}

/// Save hotkey mappings to disk (atomic write)
pub fn save(mappings: &HotkeyMappings, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let hotkeys_path = get_hotkeys_path(app_handle)?;

    let json = serde_json::to_string_pretty(mappings)
        .map_err(|e| format!("Failed to serialize hotkeys: {}", e))?;

    crate::persistence::atomic_write(&hotkeys_path, &json)?;

    tracing::debug!("Hotkey mappings saved to {:?}", hotkeys_path);
    Ok(())
}

/// Add a hotkey mapping (checks for duplicates)
pub fn add_mapping(
    mappings: &mut HotkeyMappings,
    hotkey: String,
    sound_id: SoundId,
) -> Result<(), String> {
    if mappings.mappings.contains_key(&hotkey) {
        return Err(format!("Hotkey '{}' is already assigned", hotkey));
    }

    mappings.mappings.insert(hotkey.clone(), sound_id.clone());
    tracing::info!("Added hotkey mapping: {} -> {:?}", hotkey, sound_id);
    Ok(())
}

/// Remove a hotkey mapping
pub fn remove_mapping(mappings: &mut HotkeyMappings, hotkey: &str) -> Result<(), String> {
    if mappings.mappings.remove(hotkey).is_some() {
        tracing::info!("Removed hotkey mapping: {}", hotkey);
        Ok(())
    } else {
        Err(format!("Hotkey '{}' not found", hotkey))
    }
}

/// Get the sound ID for a hotkey
pub fn get_sound_id<'a>(mappings: &'a HotkeyMappings, hotkey: &str) -> Option<&'a SoundId> {
    mappings.mappings.get(hotkey)
}

/// Get all hotkeys assigned to a specific sound
pub fn get_hotkeys_for_sound(mappings: &HotkeyMappings, sound_id: &SoundId) -> Vec<String> {
    mappings
        .mappings
        .iter()
        .filter(|(_, sid)| *sid == sound_id)
        .map(|(hotkey, _)| hotkey.clone())
        .collect()
}
