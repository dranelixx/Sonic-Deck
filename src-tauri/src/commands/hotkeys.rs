//! Global hotkey management commands

use crate::hotkeys;
use crate::sounds::SoundId;
use crate::AppState;
use tauri::State;

/// Load hotkey mappings from in-memory state
#[tauri::command]
pub fn load_hotkeys(state: State<'_, AppState>) -> Result<hotkeys::HotkeyMappings, String> {
    let mappings = state.read_hotkeys();
    Ok(mappings.clone())
}

/// Save hotkey mappings to state and disk
#[tauri::command]
pub fn save_hotkeys(
    mappings: hotkeys::HotkeyMappings,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    state.update_and_save_hotkeys(&app_handle, mappings)
}

/// Register a global hotkey for a sound
#[tauri::command]
pub fn register_hotkey(
    hotkey: String,
    sound_id: SoundId,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Read current mappings from state
    let mut mappings = {
        let current = state.read_hotkeys();
        current.clone()
    };

    // Add mapping (checks for duplicates)
    hotkeys::add_mapping(&mut mappings, hotkey.clone(), sound_id.clone())?;

    // Parse and register with the plugin
    let shortcut = hotkey
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map_err(|e| format!("Failed to parse hotkey '{}': {}", hotkey, e))?;

    tracing::info!("Parsed hotkey '{}' to shortcut: {:?}", hotkey, shortcut);

    app_handle
        .global_shortcut()
        .register(shortcut)
        .map_err(|e| format!("Failed to register hotkey: {}", e))?;

    // Update state and persist to disk
    state.update_and_save_hotkeys(&app_handle, mappings)?;

    tracing::info!(
        "Successfully registered global hotkey: {} -> {:?}",
        hotkey,
        sound_id
    );
    Ok(())
}

/// Unregister a global hotkey
#[tauri::command]
pub fn unregister_hotkey(
    hotkey: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Read current mappings from state
    let mut mappings = {
        let current = state.read_hotkeys();
        current.clone()
    };

    // Remove mapping
    hotkeys::remove_mapping(&mut mappings, &hotkey)?;

    // Parse and unregister from the plugin
    let shortcut = hotkey
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map_err(|e| format!("Failed to parse hotkey '{}': {}", hotkey, e))?;
    app_handle
        .global_shortcut()
        .unregister(shortcut)
        .map_err(|e| format!("Failed to unregister hotkey: {}", e))?;

    // Update state and persist to disk
    state.update_and_save_hotkeys(&app_handle, mappings)?;

    tracing::info!("Unregistered global hotkey: {}", hotkey);
    Ok(())
}

/// Check if a hotkey is currently registered
#[tauri::command]
pub fn is_hotkey_registered(hotkey: String, app_handle: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let shortcut = hotkey
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map_err(|e| format!("Failed to parse hotkey '{}': {}", hotkey, e))?;
    Ok(app_handle.global_shortcut().is_registered(shortcut))
}
