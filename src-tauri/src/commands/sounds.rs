//! Sound library and category management commands

use crate::sounds::{self, Category, CategoryId, Sound, SoundId, SoundLibrary};

/// Load the sound library
#[tauri::command]
pub fn load_sounds(app_handle: tauri::AppHandle) -> Result<SoundLibrary, String> {
    sounds::load(&app_handle)
}

/// Add a new sound to the library
#[tauri::command]
pub fn add_sound(
    name: String,
    file_path: String,
    category_id: CategoryId,
    icon: Option<String>,
    volume: Option<f32>,
    app_handle: tauri::AppHandle,
) -> Result<Sound, String> {
    let mut library = sounds::load(&app_handle)?;
    let sound = sounds::add_sound(&mut library, name, file_path, category_id, icon, volume);
    sounds::save(&library, &app_handle)?;
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
    app_handle: tauri::AppHandle,
) -> Result<Sound, String> {
    let mut library = sounds::load(&app_handle)?;
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

    sounds::save(&library, &app_handle)?;
    Ok(sound)
}

/// Toggle favorite status of a sound
#[tauri::command]
pub fn toggle_favorite(sound_id: SoundId, app_handle: tauri::AppHandle) -> Result<Sound, String> {
    let mut library = sounds::load(&app_handle)?;

    // Find the sound and toggle is_favorite
    let sound = library
        .sounds
        .iter_mut()
        .find(|s| s.id == sound_id)
        .ok_or_else(|| format!("Sound not found: {}", sound_id.as_str()))?;

    sound.is_favorite = !sound.is_favorite;
    let updated_sound = sound.clone();

    sounds::save(&library, &app_handle)?;
    Ok(updated_sound)
}

/// Delete a sound from the library
#[tauri::command]
pub fn delete_sound(sound_id: SoundId, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut library = sounds::load(&app_handle)?;
    sounds::delete_sound(&mut library, &sound_id)?;
    sounds::save(&library, &app_handle)?;
    Ok(())
}

/// Add a new category
#[tauri::command]
pub fn add_category(
    name: String,
    icon: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<Category, String> {
    let mut library = sounds::load(&app_handle)?;
    let category = sounds::add_category(&mut library, name, icon);
    sounds::save(&library, &app_handle)?;
    Ok(category)
}

/// Update an existing category
#[tauri::command]
pub fn update_category(
    category_id: CategoryId,
    name: Option<String>,
    icon: Option<Option<String>>,
    sort_order: Option<i32>,
    app_handle: tauri::AppHandle,
) -> Result<Category, String> {
    let mut library = sounds::load(&app_handle)?;
    let category = sounds::update_category(&mut library, &category_id, name, icon, sort_order)?;
    sounds::save(&library, &app_handle)?;
    Ok(category)
}

/// Delete a category
#[tauri::command]
pub fn delete_category(
    category_id: CategoryId,
    move_sounds_to: Option<CategoryId>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut library = sounds::load(&app_handle)?;
    sounds::delete_category(&mut library, &category_id, move_sounds_to)?;
    sounds::save(&library, &app_handle)?;
    Ok(())
}
