//! Sound library and category management commands

use crate::hotkeys;
use crate::sounds::{self, Category, CategoryId, Sound, SoundId, SoundLibrary};
use crate::AppState;
use tauri::State;
use tracing::{info, warn};

/// Load the sound library from in-memory state
#[tauri::command]
pub fn load_sounds(state: State<'_, AppState>) -> Result<SoundLibrary, String> {
    let library = state.read_sounds();
    Ok(library.clone())
}

/// Add a new sound to the library
#[tauri::command]
pub fn add_sound(
    name: String,
    file_path: String,
    category_id: CategoryId,
    icon: Option<String>,
    volume: Option<f32>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Sound, String> {
    let mut library = {
        let current = state.read_sounds();
        current.clone()
    };

    let sound = sounds::add_sound(&mut library, name, file_path, category_id, icon, volume);
    state.update_and_save_sounds(&app_handle, library)?;
    Ok(sound)
}

/// Update an existing sound
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_sound(
    sound_id: SoundId,
    name: String,
    file_path: String,
    category_id: CategoryId,
    icon: Option<String>,
    volume: Option<f32>,
    trim_start_ms: Option<u64>,
    trim_end_ms: Option<u64>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Sound, String> {
    let mut library = {
        let current = state.read_sounds();
        current.clone()
    };

    // Always update all fields when editing (simpler API)
    let sound = sounds::update_sound(
        &mut library,
        &sound_id,
        Some(name),
        Some(file_path),
        Some(category_id),
        Some(icon),
        Some(volume),
        None,                // Don't change is_favorite here
        Some(trim_start_ms), // Update trim_start_ms
        Some(trim_end_ms),   // Update trim_end_ms
    )?;

    state.update_and_save_sounds(&app_handle, library)?;
    Ok(sound)
}

/// Toggle favorite status of a sound
#[tauri::command]
pub fn toggle_favorite(
    sound_id: SoundId,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Sound, String> {
    let mut library = {
        let current = state.read_sounds();
        current.clone()
    };

    // Find the sound and toggle is_favorite
    let sound = library
        .sounds
        .iter_mut()
        .find(|s| s.id == sound_id)
        .ok_or_else(|| format!("Sound not found: {}", sound_id.as_str()))?;

    sound.is_favorite = !sound.is_favorite;
    let updated_sound = sound.clone();

    state.update_and_save_sounds(&app_handle, library)?;
    Ok(updated_sound)
}

/// Delete a sound from the library and remove associated hotkeys
#[tauri::command]
pub fn delete_sound(
    sound_id: SoundId,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // First, find and remove any hotkeys associated with this sound
    let mut mappings = {
        let current = state.read_hotkeys();
        current.clone()
    };

    let hotkeys_to_remove = hotkeys::get_hotkeys_for_sound(&mappings, &sound_id);

    if !hotkeys_to_remove.is_empty() {
        info!(
            "Removing {} hotkey(s) for deleted sound {:?}",
            hotkeys_to_remove.len(),
            sound_id
        );

        #[cfg(desktop)]
        {
            use tauri_plugin_global_shortcut::GlobalShortcutExt;

            for hotkey in &hotkeys_to_remove {
                // Unregister global shortcut
                if let Ok(shortcut) = hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
                    if let Err(e) = app_handle.global_shortcut().unregister(shortcut) {
                        warn!("Failed to unregister hotkey '{}': {}", hotkey, e);
                    }
                }

                // Remove from mappings
                if let Err(e) = hotkeys::remove_mapping(&mut mappings, hotkey) {
                    warn!("Failed to remove hotkey mapping '{}': {}", hotkey, e);
                }
            }
        }

        // Save updated hotkey mappings
        state.update_and_save_hotkeys(&app_handle, mappings)?;
    }

    // Now delete the sound
    let mut library = {
        let current = state.read_sounds();
        current.clone()
    };

    sounds::delete_sound(&mut library, &sound_id)?;
    state.update_and_save_sounds(&app_handle, library)?;

    Ok(())
}

/// Add a new category
#[tauri::command]
pub fn add_category(
    name: String,
    icon: Option<String>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Category, String> {
    let mut library = {
        let current = state.read_sounds();
        current.clone()
    };

    let category = sounds::add_category(&mut library, name, icon);
    state.update_and_save_sounds(&app_handle, library)?;
    Ok(category)
}

/// Update an existing category
#[tauri::command]
pub fn update_category(
    category_id: CategoryId,
    name: Option<String>,
    icon: Option<Option<String>>,
    sort_order: Option<i32>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Category, String> {
    let mut library = {
        let current = state.read_sounds();
        current.clone()
    };

    let category = sounds::update_category(&mut library, &category_id, name, icon, sort_order)?;
    state.update_and_save_sounds(&app_handle, library)?;
    Ok(category)
}

/// Delete a category
#[tauri::command]
pub fn delete_category(
    category_id: CategoryId,
    move_sounds_to: Option<CategoryId>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut library = {
        let current = state.read_sounds();
        current.clone()
    };

    sounds::delete_category(&mut library, &category_id, move_sounds_to)?;
    state.update_and_save_sounds(&app_handle, library)?;
    Ok(())
}
