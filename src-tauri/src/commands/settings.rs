//! Application settings and autostart management commands

use crate::settings::{self, AppSettings};
use crate::AppState;
use tauri::State;

/// Load application settings from in-memory state
#[tauri::command]
pub fn load_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.read_settings();
    Ok(settings.clone())
}

/// Save application settings to state and disk
#[tauri::command]
pub fn save_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    state.update_and_save_settings(&app_handle, settings)
}

/// Get the settings file path (for debugging/info)
#[tauri::command]
pub fn get_settings_file_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    let path = settings::get_settings_path(&app_handle)?;
    Ok(path.to_string_lossy().to_string())
}

/// Enable autostart on system boot
#[tauri::command]
pub fn enable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        app_handle
            .autolaunch()
            .enable()
            .map_err(|e| format!("Failed to enable autostart: {}", e))?;
    }
    Ok(())
}

/// Disable autostart on system boot
#[tauri::command]
pub fn disable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        app_handle
            .autolaunch()
            .disable()
            .map_err(|e| format!("Failed to disable autostart: {}", e))?;
    }
    Ok(())
}

/// Check if autostart is enabled
#[tauri::command]
pub fn is_autostart_enabled(app_handle: tauri::AppHandle) -> Result<bool, String> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        app_handle
            .autolaunch()
            .is_enabled()
            .map_err(|e| format!("Failed to check autostart status: {}", e))
    }
    #[cfg(not(desktop))]
    Ok(false)
}
