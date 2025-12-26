//! Sound library persistence
//!
//! Stores sounds and categories as JSON in the platform-specific app data directory.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

/// Unique identifier for a sound
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SoundId(String);

impl SoundId {
    /// Create a new unique sound ID
    pub fn new() -> Self {
        Self(uuid_v4())
    }

    /// Get the raw string ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SoundId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CategoryId(String);

impl CategoryId {
    /// Create a new unique category ID
    pub fn new() -> Self {
        Self(uuid_v4())
    }

    /// Create from a raw string (for default category)
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Get the raw string ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CategoryId {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple UUID v4 generator (no external dependency needed)
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let timestamp = now.as_nanos();
    let random_part: u64 = (timestamp as u64)
        .wrapping_mul(0x5DEECE66D)
        .wrapping_add(0xB);

    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (timestamp >> 64) as u32,
        ((timestamp >> 48) as u16),
        random_part as u16 & 0x0FFF,
        ((random_part >> 16) as u16 & 0x3FFF) | 0x8000,
        random_part >> 32
    )
}

/// A sound in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sound {
    /// Unique identifier
    pub id: SoundId,
    /// Display name
    pub name: String,
    /// Path to the audio file
    pub file_path: String,
    /// Category this sound belongs to
    pub category_id: CategoryId,
    /// Optional icon/emoji for display
    pub icon: Option<String>,
    /// Optional custom volume for this sound (0.0-1.0)
    pub volume: Option<f32>,
    /// Whether this sound is marked as favorite
    #[serde(default)]
    pub is_favorite: bool,
    /// Optional trim start time in milliseconds
    #[serde(default)]
    pub trim_start_ms: Option<u64>,
    /// Optional trim end time in milliseconds
    #[serde(default)]
    pub trim_end_ms: Option<u64>,
}

/// A category to organize sounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier
    pub id: CategoryId,
    /// Display name
    pub name: String,
    /// Optional icon/emoji for the tab
    pub icon: Option<String>,
    /// Sort order (lower = first)
    pub sort_order: i32,
}

/// Complete sound library data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundLibrary {
    /// All categories
    pub categories: Vec<Category>,
    /// All sounds
    pub sounds: Vec<Sound>,
}

impl Default for SoundLibrary {
    fn default() -> Self {
        // Create a default "General" category
        Self {
            categories: vec![Category {
                id: CategoryId::from_string("default".to_string()),
                name: "General".to_string(),
                icon: Some("🎵".to_string()),
                sort_order: 0,
            }],
            sounds: vec![],
        }
    }
}

/// Get the path to the sounds file
pub fn get_sounds_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Ensure directory exists
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    Ok(app_data_dir.join("sounds.json"))
}

/// Load sound library from disk
pub fn load(app_handle: &tauri::AppHandle) -> Result<SoundLibrary, String> {
    let sounds_path = get_sounds_path(app_handle)?;

    if !sounds_path.exists() {
        // Return default library if file doesn't exist
        return Ok(SoundLibrary::default());
    }

    let content = std::fs::read_to_string(&sounds_path)
        .map_err(|e| format!("Failed to read sounds file: {}", e))?;

    let library: SoundLibrary =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse sounds: {}", e))?;

    Ok(library)
}

/// Save sound library to disk (atomic write)
pub fn save(library: &SoundLibrary, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let sounds_path = get_sounds_path(app_handle)?;

    let json = serde_json::to_string_pretty(library)
        .map_err(|e| format!("Failed to serialize sounds: {}", e))?;

    crate::persistence::atomic_write(&sounds_path, &json)
}

// ============================================================================
// CRUD Operations
// ============================================================================

/// Add a new sound to the library
pub fn add_sound(
    library: &mut SoundLibrary,
    name: String,
    file_path: String,
    category_id: CategoryId,
    icon: Option<String>,
    volume: Option<f32>,
) -> Sound {
    let sound = Sound {
        id: SoundId::new(),
        name,
        file_path,
        category_id,
        icon,
        volume: volume.map(|v| v.clamp(0.0, 1.0)),
        is_favorite: false,
        trim_start_ms: None,
        trim_end_ms: None,
    };
    library.sounds.push(sound.clone());
    sound
}

/// Update an existing sound
#[allow(clippy::too_many_arguments)]
pub fn update_sound(
    library: &mut SoundLibrary,
    sound_id: &SoundId,
    name: Option<String>,
    file_path: Option<String>,
    category_id: Option<CategoryId>,
    icon: Option<Option<String>>,
    volume: Option<Option<f32>>,
    is_favorite: Option<bool>,
    trim_start_ms: Option<Option<u64>>,
    trim_end_ms: Option<Option<u64>>,
) -> Result<Sound, String> {
    let sound = library
        .sounds
        .iter_mut()
        .find(|s| &s.id == sound_id)
        .ok_or_else(|| format!("Sound not found: {}", sound_id.as_str()))?;

    if let Some(name) = name {
        sound.name = name;
    }
    if let Some(file_path) = file_path {
        sound.file_path = file_path;
    }
    if let Some(category_id) = category_id {
        sound.category_id = category_id;
    }
    if let Some(icon) = icon {
        sound.icon = icon;
    }
    if let Some(volume) = volume {
        sound.volume = volume.map(|v| v.clamp(0.0, 1.0));
    }
    if let Some(is_favorite) = is_favorite {
        sound.is_favorite = is_favorite;
    }
    if let Some(trim_start_ms) = trim_start_ms {
        sound.trim_start_ms = trim_start_ms;
    }
    if let Some(trim_end_ms) = trim_end_ms {
        sound.trim_end_ms = trim_end_ms;
    }

    Ok(sound.clone())
}

/// Delete a sound from the library
pub fn delete_sound(library: &mut SoundLibrary, sound_id: &SoundId) -> Result<(), String> {
    let initial_len = library.sounds.len();
    library.sounds.retain(|s| &s.id != sound_id);

    if library.sounds.len() == initial_len {
        return Err(format!("Sound not found: {}", sound_id.as_str()));
    }

    Ok(())
}

/// Add a new category
pub fn add_category(library: &mut SoundLibrary, name: String, icon: Option<String>) -> Category {
    let max_order = library
        .categories
        .iter()
        .map(|c| c.sort_order)
        .max()
        .unwrap_or(0);

    let category = Category {
        id: CategoryId::new(),
        name,
        icon,
        sort_order: max_order + 1,
    };
    library.categories.push(category.clone());
    category
}

/// Update an existing category
pub fn update_category(
    library: &mut SoundLibrary,
    category_id: &CategoryId,
    name: Option<String>,
    icon: Option<Option<String>>,
    sort_order: Option<i32>,
) -> Result<Category, String> {
    let category = library
        .categories
        .iter_mut()
        .find(|c| &c.id == category_id)
        .ok_or_else(|| format!("Category not found: {}", category_id.as_str()))?;

    if let Some(name) = name {
        category.name = name;
    }
    if let Some(icon) = icon {
        category.icon = icon;
    }
    if let Some(sort_order) = sort_order {
        category.sort_order = sort_order;
    }

    Ok(category.clone())
}

/// Delete a category and optionally move its sounds
pub fn delete_category(
    library: &mut SoundLibrary,
    category_id: &CategoryId,
    move_sounds_to: Option<CategoryId>,
) -> Result<(), String> {
    // Don't allow deleting the default category
    if category_id.as_str() == "default" {
        return Err("Cannot delete the default category".to_string());
    }

    // Check if category exists
    let exists = library.categories.iter().any(|c| &c.id == category_id);
    if !exists {
        return Err(format!("Category not found: {}", category_id.as_str()));
    }

    // Move or delete sounds in this category
    if let Some(target_category_id) = move_sounds_to {
        // Move sounds to target category
        for sound in library.sounds.iter_mut() {
            if &sound.category_id == category_id {
                sound.category_id = target_category_id.clone();
            }
        }
    } else {
        // Delete sounds in this category
        library.sounds.retain(|s| &s.category_id != category_id);
    }

    // Remove the category
    library.categories.retain(|c| &c.id != category_id);

    Ok(())
}
