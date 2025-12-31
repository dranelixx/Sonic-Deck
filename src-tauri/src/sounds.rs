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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // SoundId Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sound_id_new_creates_unique_ids() {
        let id1 = SoundId::new();
        let id2 = SoundId::new();
        // IDs should be different (with high probability)
        assert_ne!(id1.as_str(), id2.as_str());
    }

    #[test]
    fn test_sound_id_as_str_returns_inner_string() {
        let id = SoundId::new();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn test_sound_id_default_creates_new_id() {
        let id = SoundId::default();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn test_sound_id_equality() {
        let id1 = SoundId::new();
        let id2 = id1.clone();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_sound_id_serde_roundtrip() {
        let id = SoundId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: SoundId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    // -------------------------------------------------------------------------
    // CategoryId Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_category_id_new_creates_unique_ids() {
        let id1 = CategoryId::new();
        let id2 = CategoryId::new();
        assert_ne!(id1.as_str(), id2.as_str());
    }

    #[test]
    fn test_category_id_from_string() {
        let id = CategoryId::from_string("test-category".to_string());
        assert_eq!(id.as_str(), "test-category");
    }

    #[test]
    fn test_category_id_as_str() {
        let id = CategoryId::from_string("my-category".to_string());
        assert_eq!(id.as_str(), "my-category");
    }

    #[test]
    fn test_category_id_default() {
        let id = CategoryId::default();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn test_category_id_serde_roundtrip() {
        let id = CategoryId::from_string("test".to_string());
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: CategoryId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    // -------------------------------------------------------------------------
    // uuid_v4 Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_uuid_v4_format() {
        let uuid = uuid_v4();
        // UUID format: xxxxxxxx-xxxx-4xxx-xxxx-xxxxxxxxxxxx
        let parts: Vec<&str> = uuid.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn test_uuid_v4_version_marker() {
        let uuid = uuid_v4();
        let parts: Vec<&str> = uuid.split('-').collect();
        // Third part should start with '4' (UUID v4)
        assert!(parts[2].starts_with('4'));
    }

    #[test]
    fn test_uuid_v4_uniqueness() {
        let uuids: Vec<String> = (0..100).map(|_| uuid_v4()).collect();
        let unique_count = uuids.iter().collect::<std::collections::HashSet<_>>().len();
        // All generated UUIDs should be unique
        assert_eq!(unique_count, 100);
    }

    // -------------------------------------------------------------------------
    // SoundLibrary Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sound_library_default() {
        let library = SoundLibrary::default();
        assert_eq!(library.categories.len(), 1);
        assert_eq!(library.sounds.len(), 0);

        let default_category = &library.categories[0];
        assert_eq!(default_category.id.as_str(), "default");
        assert_eq!(default_category.name, "General");
        assert_eq!(default_category.icon, Some("🎵".to_string()));
        assert_eq!(default_category.sort_order, 0);
    }

    #[test]
    fn test_sound_library_serde_roundtrip() {
        let mut library = SoundLibrary::default();
        add_sound(
            &mut library,
            "Test Sound".to_string(),
            "/path/to/sound.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            Some("🔊".to_string()),
            Some(0.8),
        );

        let json = serde_json::to_string(&library).unwrap();
        let deserialized: SoundLibrary = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.categories.len(), 1);
        assert_eq!(deserialized.sounds.len(), 1);
        assert_eq!(deserialized.sounds[0].name, "Test Sound");
    }

    // -------------------------------------------------------------------------
    // add_sound Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_add_sound_basic() {
        let mut library = SoundLibrary::default();
        let sound = add_sound(
            &mut library,
            "My Sound".to_string(),
            "/audio/sound.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );

        assert_eq!(library.sounds.len(), 1);
        assert_eq!(sound.name, "My Sound");
        assert_eq!(sound.file_path, "/audio/sound.mp3");
        assert_eq!(sound.icon, None);
        assert_eq!(sound.volume, None);
        assert!(!sound.is_favorite);
    }

    #[test]
    fn test_add_sound_with_icon_and_volume() {
        let mut library = SoundLibrary::default();
        let sound = add_sound(
            &mut library,
            "Effect".to_string(),
            "/audio/effect.ogg".to_string(),
            CategoryId::from_string("default".to_string()),
            Some("💥".to_string()),
            Some(0.75),
        );

        assert_eq!(sound.icon, Some("💥".to_string()));
        assert_eq!(sound.volume, Some(0.75));
    }

    #[test]
    fn test_add_sound_volume_clamping() {
        let mut library = SoundLibrary::default();

        // Volume above 1.0 should be clamped
        let sound1 = add_sound(
            &mut library,
            "Loud".to_string(),
            "/loud.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            Some(1.5),
        );
        assert_eq!(sound1.volume, Some(1.0));

        // Volume below 0.0 should be clamped
        let sound2 = add_sound(
            &mut library,
            "Quiet".to_string(),
            "/quiet.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            Some(-0.5),
        );
        assert_eq!(sound2.volume, Some(0.0));
    }

    #[test]
    fn test_add_sound_multiple() {
        let mut library = SoundLibrary::default();
        add_sound(
            &mut library,
            "Sound 1".to_string(),
            "/s1.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );
        add_sound(
            &mut library,
            "Sound 2".to_string(),
            "/s2.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );
        add_sound(
            &mut library,
            "Sound 3".to_string(),
            "/s3.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );

        assert_eq!(library.sounds.len(), 3);
    }

    // -------------------------------------------------------------------------
    // update_sound Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_update_sound_name() {
        let mut library = SoundLibrary::default();
        let sound = add_sound(
            &mut library,
            "Original".to_string(),
            "/path.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );

        let updated = update_sound(
            &mut library,
            &sound.id,
            Some("Updated Name".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.file_path, "/path.mp3"); // Unchanged
    }

    #[test]
    fn test_update_sound_all_fields() {
        let mut library = SoundLibrary::default();
        let sound = add_sound(
            &mut library,
            "Original".to_string(),
            "/original.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );

        let new_category = CategoryId::from_string("new-cat".to_string());
        let updated = update_sound(
            &mut library,
            &sound.id,
            Some("New Name".to_string()),
            Some("/new/path.mp3".to_string()),
            Some(new_category.clone()),
            Some(Some("🎸".to_string())),
            Some(Some(0.9)),
            Some(true),
            Some(Some(1000)),
            Some(Some(5000)),
        )
        .unwrap();

        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.file_path, "/new/path.mp3");
        assert_eq!(updated.category_id, new_category);
        assert_eq!(updated.icon, Some("🎸".to_string()));
        assert_eq!(updated.volume, Some(0.9));
        assert!(updated.is_favorite);
        assert_eq!(updated.trim_start_ms, Some(1000));
        assert_eq!(updated.trim_end_ms, Some(5000));
    }

    #[test]
    fn test_update_sound_not_found() {
        let mut library = SoundLibrary::default();
        let fake_id = SoundId::new();

        let result = update_sound(
            &mut library,
            &fake_id,
            Some("Name".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Sound not found"));
    }

    #[test]
    fn test_update_sound_volume_clamping() {
        let mut library = SoundLibrary::default();
        let sound = add_sound(
            &mut library,
            "Test".to_string(),
            "/test.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );

        let updated = update_sound(
            &mut library,
            &sound.id,
            None,
            None,
            None,
            None,
            Some(Some(2.0)), // Should be clamped to 1.0
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(updated.volume, Some(1.0));
    }

    // -------------------------------------------------------------------------
    // delete_sound Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_delete_sound_success() {
        let mut library = SoundLibrary::default();
        let sound = add_sound(
            &mut library,
            "To Delete".to_string(),
            "/delete.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );

        assert_eq!(library.sounds.len(), 1);
        let result = delete_sound(&mut library, &sound.id);
        assert!(result.is_ok());
        assert_eq!(library.sounds.len(), 0);
    }

    #[test]
    fn test_delete_sound_not_found() {
        let mut library = SoundLibrary::default();
        let fake_id = SoundId::new();

        let result = delete_sound(&mut library, &fake_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Sound not found"));
    }

    #[test]
    fn test_delete_sound_preserves_others() {
        let mut library = SoundLibrary::default();
        let sound1 = add_sound(
            &mut library,
            "Keep 1".to_string(),
            "/keep1.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );
        let sound2 = add_sound(
            &mut library,
            "Delete".to_string(),
            "/delete.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );
        let sound3 = add_sound(
            &mut library,
            "Keep 2".to_string(),
            "/keep2.mp3".to_string(),
            CategoryId::from_string("default".to_string()),
            None,
            None,
        );

        delete_sound(&mut library, &sound2.id).unwrap();

        assert_eq!(library.sounds.len(), 2);
        assert!(library.sounds.iter().any(|s| s.id == sound1.id));
        assert!(library.sounds.iter().any(|s| s.id == sound3.id));
        assert!(!library.sounds.iter().any(|s| s.id == sound2.id));
    }

    // -------------------------------------------------------------------------
    // add_category Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_add_category_basic() {
        let mut library = SoundLibrary::default();
        let category = add_category(&mut library, "Music".to_string(), None);

        assert_eq!(library.categories.len(), 2); // Default + new
        assert_eq!(category.name, "Music");
        assert_eq!(category.icon, None);
        assert_eq!(category.sort_order, 1); // Default has 0
    }

    #[test]
    fn test_add_category_with_icon() {
        let mut library = SoundLibrary::default();
        let category = add_category(&mut library, "Effects".to_string(), Some("💥".to_string()));

        assert_eq!(category.icon, Some("💥".to_string()));
    }

    #[test]
    fn test_add_category_sort_order_increments() {
        let mut library = SoundLibrary::default();
        let cat1 = add_category(&mut library, "Cat1".to_string(), None);
        let cat2 = add_category(&mut library, "Cat2".to_string(), None);
        let cat3 = add_category(&mut library, "Cat3".to_string(), None);

        assert_eq!(cat1.sort_order, 1);
        assert_eq!(cat2.sort_order, 2);
        assert_eq!(cat3.sort_order, 3);
    }

    // -------------------------------------------------------------------------
    // update_category Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_update_category_name() {
        let mut library = SoundLibrary::default();
        let category = add_category(&mut library, "Original".to_string(), None);

        let updated = update_category(
            &mut library,
            &category.id,
            Some("Renamed".to_string()),
            None,
            None,
        )
        .unwrap();

        assert_eq!(updated.name, "Renamed");
    }

    #[test]
    fn test_update_category_icon() {
        let mut library = SoundLibrary::default();
        let category = add_category(&mut library, "Test".to_string(), None);

        let updated = update_category(
            &mut library,
            &category.id,
            None,
            Some(Some("🎵".to_string())),
            None,
        )
        .unwrap();

        assert_eq!(updated.icon, Some("🎵".to_string()));
    }

    #[test]
    fn test_update_category_sort_order() {
        let mut library = SoundLibrary::default();
        let category = add_category(&mut library, "Test".to_string(), None);

        let updated = update_category(&mut library, &category.id, None, None, Some(100)).unwrap();

        assert_eq!(updated.sort_order, 100);
    }

    #[test]
    fn test_update_category_not_found() {
        let mut library = SoundLibrary::default();
        let fake_id = CategoryId::new();

        let result = update_category(&mut library, &fake_id, Some("Name".to_string()), None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Category not found"));
    }

    // -------------------------------------------------------------------------
    // delete_category Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_delete_category_cannot_delete_default() {
        let mut library = SoundLibrary::default();
        let default_id = CategoryId::from_string("default".to_string());

        let result = delete_category(&mut library, &default_id, None);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot delete the default category"));
    }

    #[test]
    fn test_delete_category_not_found() {
        let mut library = SoundLibrary::default();
        let fake_id = CategoryId::new();

        let result = delete_category(&mut library, &fake_id, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Category not found"));
    }

    #[test]
    fn test_delete_category_deletes_sounds() {
        let mut library = SoundLibrary::default();
        let category = add_category(&mut library, "ToDelete".to_string(), None);

        // Add sounds to this category
        add_sound(
            &mut library,
            "Sound1".to_string(),
            "/s1.mp3".to_string(),
            category.id.clone(),
            None,
            None,
        );
        add_sound(
            &mut library,
            "Sound2".to_string(),
            "/s2.mp3".to_string(),
            category.id.clone(),
            None,
            None,
        );

        assert_eq!(library.sounds.len(), 2);

        // Delete category without moving sounds
        delete_category(&mut library, &category.id, None).unwrap();

        assert_eq!(library.categories.len(), 1); // Only default remains
        assert_eq!(library.sounds.len(), 0); // Sounds were deleted
    }

    #[test]
    fn test_delete_category_moves_sounds() {
        let mut library = SoundLibrary::default();
        let category = add_category(&mut library, "ToDelete".to_string(), None);
        let default_id = CategoryId::from_string("default".to_string());

        // Add sounds to the category to delete
        add_sound(
            &mut library,
            "Sound1".to_string(),
            "/s1.mp3".to_string(),
            category.id.clone(),
            None,
            None,
        );
        add_sound(
            &mut library,
            "Sound2".to_string(),
            "/s2.mp3".to_string(),
            category.id.clone(),
            None,
            None,
        );

        // Delete category but move sounds to default
        delete_category(&mut library, &category.id, Some(default_id.clone())).unwrap();

        assert_eq!(library.categories.len(), 1);
        assert_eq!(library.sounds.len(), 2); // Sounds preserved

        // All sounds should now be in default category
        for sound in &library.sounds {
            assert_eq!(sound.category_id, default_id);
        }
    }

    #[test]
    fn test_delete_category_preserves_other_sounds() {
        let mut library = SoundLibrary::default();
        let default_id = CategoryId::from_string("default".to_string());
        let category = add_category(&mut library, "ToDelete".to_string(), None);

        // Add sound to default category
        add_sound(
            &mut library,
            "DefaultSound".to_string(),
            "/default.mp3".to_string(),
            default_id.clone(),
            None,
            None,
        );

        // Add sound to category to delete
        add_sound(
            &mut library,
            "DeleteSound".to_string(),
            "/delete.mp3".to_string(),
            category.id.clone(),
            None,
            None,
        );

        assert_eq!(library.sounds.len(), 2);

        // Delete category without moving sounds
        delete_category(&mut library, &category.id, None).unwrap();

        assert_eq!(library.sounds.len(), 1);
        assert_eq!(library.sounds[0].name, "DefaultSound");
    }
}
