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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a test SoundId
    fn test_sound_id(id: &str) -> SoundId {
        // Use the internal constructor pattern from sounds.rs
        serde_json::from_str(&format!("\"{}\"", id)).unwrap()
    }

    // -------------------------------------------------------------------------
    // HotkeyMappings::default Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_hotkey_mappings_default() {
        let mappings = HotkeyMappings::default();
        assert!(mappings.mappings.is_empty());
    }

    // -------------------------------------------------------------------------
    // add_mapping Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_add_mapping_success() {
        let mut mappings = HotkeyMappings::default();
        let sound_id = test_sound_id("sound-1");

        let result = add_mapping(&mut mappings, "Ctrl+A".to_string(), sound_id.clone());

        assert!(result.is_ok());
        assert_eq!(mappings.mappings.len(), 1);
        assert_eq!(mappings.mappings.get("Ctrl+A"), Some(&sound_id));
    }

    #[test]
    fn test_add_mapping_multiple() {
        let mut mappings = HotkeyMappings::default();
        let sound1 = test_sound_id("sound-1");
        let sound2 = test_sound_id("sound-2");

        add_mapping(&mut mappings, "Ctrl+A".to_string(), sound1.clone()).unwrap();
        add_mapping(&mut mappings, "Ctrl+B".to_string(), sound2.clone()).unwrap();

        assert_eq!(mappings.mappings.len(), 2);
    }

    #[test]
    fn test_add_mapping_duplicate_hotkey_fails() {
        let mut mappings = HotkeyMappings::default();
        let sound1 = test_sound_id("sound-1");
        let sound2 = test_sound_id("sound-2");

        add_mapping(&mut mappings, "Ctrl+A".to_string(), sound1).unwrap();
        let result = add_mapping(&mut mappings, "Ctrl+A".to_string(), sound2);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already assigned"));
    }

    #[test]
    fn test_add_mapping_same_sound_different_hotkeys() {
        let mut mappings = HotkeyMappings::default();
        let sound = test_sound_id("sound-1");

        add_mapping(&mut mappings, "Ctrl+A".to_string(), sound.clone()).unwrap();
        let result = add_mapping(&mut mappings, "Ctrl+B".to_string(), sound.clone());

        assert!(result.is_ok());
        assert_eq!(mappings.mappings.len(), 2);
    }

    // -------------------------------------------------------------------------
    // remove_mapping Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_remove_mapping_success() {
        let mut mappings = HotkeyMappings::default();
        let sound = test_sound_id("sound-1");

        add_mapping(&mut mappings, "Ctrl+A".to_string(), sound).unwrap();
        assert_eq!(mappings.mappings.len(), 1);

        let result = remove_mapping(&mut mappings, "Ctrl+A");

        assert!(result.is_ok());
        assert!(mappings.mappings.is_empty());
    }

    #[test]
    fn test_remove_mapping_not_found() {
        let mut mappings = HotkeyMappings::default();

        let result = remove_mapping(&mut mappings, "Ctrl+X");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_remove_mapping_preserves_others() {
        let mut mappings = HotkeyMappings::default();
        let sound1 = test_sound_id("sound-1");
        let sound2 = test_sound_id("sound-2");

        add_mapping(&mut mappings, "Ctrl+A".to_string(), sound1).unwrap();
        add_mapping(&mut mappings, "Ctrl+B".to_string(), sound2.clone()).unwrap();

        remove_mapping(&mut mappings, "Ctrl+A").unwrap();

        assert_eq!(mappings.mappings.len(), 1);
        assert_eq!(mappings.mappings.get("Ctrl+B"), Some(&sound2));
    }

    // -------------------------------------------------------------------------
    // get_sound_id Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_sound_id_found() {
        let mut mappings = HotkeyMappings::default();
        let sound = test_sound_id("sound-1");

        add_mapping(&mut mappings, "Ctrl+A".to_string(), sound.clone()).unwrap();

        let result = get_sound_id(&mappings, "Ctrl+A");

        assert!(result.is_some());
        assert_eq!(result.unwrap(), &sound);
    }

    #[test]
    fn test_get_sound_id_not_found() {
        let mappings = HotkeyMappings::default();

        let result = get_sound_id(&mappings, "Ctrl+X");

        assert!(result.is_none());
    }

    // -------------------------------------------------------------------------
    // get_hotkeys_for_sound Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_hotkeys_for_sound_single() {
        let mut mappings = HotkeyMappings::default();
        let sound = test_sound_id("sound-1");

        add_mapping(&mut mappings, "Ctrl+A".to_string(), sound.clone()).unwrap();

        let hotkeys = get_hotkeys_for_sound(&mappings, &sound);

        assert_eq!(hotkeys.len(), 1);
        assert!(hotkeys.contains(&"Ctrl+A".to_string()));
    }

    #[test]
    fn test_get_hotkeys_for_sound_multiple() {
        let mut mappings = HotkeyMappings::default();
        let sound = test_sound_id("sound-1");

        add_mapping(&mut mappings, "Ctrl+A".to_string(), sound.clone()).unwrap();
        add_mapping(&mut mappings, "Ctrl+B".to_string(), sound.clone()).unwrap();
        add_mapping(&mut mappings, "Ctrl+C".to_string(), sound.clone()).unwrap();

        let hotkeys = get_hotkeys_for_sound(&mappings, &sound);

        assert_eq!(hotkeys.len(), 3);
    }

    #[test]
    fn test_get_hotkeys_for_sound_none() {
        let mappings = HotkeyMappings::default();
        let sound = test_sound_id("sound-1");

        let hotkeys = get_hotkeys_for_sound(&mappings, &sound);

        assert!(hotkeys.is_empty());
    }

    #[test]
    fn test_get_hotkeys_for_sound_filters_other_sounds() {
        let mut mappings = HotkeyMappings::default();
        let sound1 = test_sound_id("sound-1");
        let sound2 = test_sound_id("sound-2");

        add_mapping(&mut mappings, "Ctrl+A".to_string(), sound1.clone()).unwrap();
        add_mapping(&mut mappings, "Ctrl+B".to_string(), sound2).unwrap();
        add_mapping(&mut mappings, "Ctrl+C".to_string(), sound1.clone()).unwrap();

        let hotkeys = get_hotkeys_for_sound(&mappings, &sound1);

        assert_eq!(hotkeys.len(), 2);
        assert!(hotkeys.contains(&"Ctrl+A".to_string()));
        assert!(hotkeys.contains(&"Ctrl+C".to_string()));
        assert!(!hotkeys.contains(&"Ctrl+B".to_string()));
    }

    // -------------------------------------------------------------------------
    // Serialization Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_hotkey_mappings_serde_roundtrip() {
        let mut mappings = HotkeyMappings::default();
        let sound = test_sound_id("sound-1");

        add_mapping(&mut mappings, "Ctrl+Shift+A".to_string(), sound).unwrap();

        let json = serde_json::to_string(&mappings).unwrap();
        let deserialized: HotkeyMappings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.mappings.len(), 1);
        assert!(deserialized.mappings.contains_key("Ctrl+Shift+A"));
    }

    #[test]
    fn test_hotkey_mappings_empty_serde() {
        let mappings = HotkeyMappings::default();

        let json = serde_json::to_string(&mappings).unwrap();
        let deserialized: HotkeyMappings = serde_json::from_str(&json).unwrap();

        assert!(deserialized.mappings.is_empty());
    }
}
