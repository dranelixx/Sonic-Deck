//! Global hotkey management commands

use crate::hotkeys;
use crate::sounds::SoundId;

/// Load hotkey mappings from disk
#[tauri::command]
pub fn load_hotkeys(app_handle: tauri::AppHandle) -> Result<hotkeys::HotkeyMappings, String> {
    hotkeys::load(&app_handle)
}

/// Save hotkey mappings to disk
#[tauri::command]
pub fn save_hotkeys(
    mappings: hotkeys::HotkeyMappings,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    hotkeys::save(&mappings, &app_handle)
}

/// Register a global hotkey for a sound
#[tauri::command]
pub fn register_hotkey(
    hotkey: String,
    sound_id: SoundId,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Load current mappings
    let mut mappings = hotkeys::load(&app_handle)?;

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

    // Save updated mappings
    hotkeys::save(&mappings, &app_handle)?;

    tracing::info!(
        "Successfully registered global hotkey: {} -> {:?}",
        hotkey,
        sound_id
    );
    Ok(())
}

/// Unregister a global hotkey
#[tauri::command]
pub fn unregister_hotkey(hotkey: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Load current mappings
    let mut mappings = hotkeys::load(&app_handle)?;

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

    // Save updated mappings
    hotkeys::save(&mappings, &app_handle)?;

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
