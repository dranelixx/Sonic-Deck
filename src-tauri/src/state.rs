//! In-memory application state for zero-latency hotkey handling
//!
//! All state changes are written to both in-memory state and disk for persistence.

use std::sync::{Arc, RwLock};

use crate::hotkeys::HotkeyMappings;
use crate::settings::AppSettings;
use crate::sounds::SoundLibrary;

/// Thread-safe in-memory application state
pub struct AppState {
    /// Hotkey mappings (keyboard shortcut -> sound ID)
    pub hotkeys: Arc<RwLock<HotkeyMappings>>,
    /// Sound library (categories + sounds)
    pub sounds: Arc<RwLock<SoundLibrary>>,
    /// Application settings (devices, volumes, preferences)
    pub settings: Arc<RwLock<AppSettings>>,
}

impl AppState {
    /// Initialize state by loading all data from disk
    pub fn load(app_handle: &tauri::AppHandle) -> Result<Self, String> {
        tracing::info!("Loading application state from disk");

        let hotkeys = crate::hotkeys::load(app_handle)?;
        let sounds = crate::sounds::load(app_handle)?;
        let settings = crate::settings::load(app_handle)?;

        tracing::info!(
            "State loaded: {} hotkeys, {} sounds, {} categories",
            hotkeys.mappings.len(),
            sounds.sounds.len(),
            sounds.categories.len()
        );

        Ok(Self {
            hotkeys: Arc::new(RwLock::new(hotkeys)),
            sounds: Arc::new(RwLock::new(sounds)),
            settings: Arc::new(RwLock::new(settings)),
        })
    }

    /// Get a read-locked reference to hotkey mappings
    pub fn read_hotkeys(&self) -> std::sync::RwLockReadGuard<'_, HotkeyMappings> {
        self.hotkeys
            .read()
            .expect("RwLock poisoned: hotkeys read failed")
    }

    /// Get a write-locked reference to hotkey mappings
    pub fn write_hotkeys(&self) -> std::sync::RwLockWriteGuard<'_, HotkeyMappings> {
        self.hotkeys
            .write()
            .expect("RwLock poisoned: hotkeys write failed")
    }

    /// Get a read-locked reference to sound library
    pub fn read_sounds(&self) -> std::sync::RwLockReadGuard<'_, SoundLibrary> {
        self.sounds
            .read()
            .expect("RwLock poisoned: sounds read failed")
    }

    /// Get a write-locked reference to sound library
    pub fn write_sounds(&self) -> std::sync::RwLockWriteGuard<'_, SoundLibrary> {
        self.sounds
            .write()
            .expect("RwLock poisoned: sounds write failed")
    }

    /// Get a read-locked reference to settings
    pub fn read_settings(&self) -> std::sync::RwLockReadGuard<'_, AppSettings> {
        self.settings
            .read()
            .expect("RwLock poisoned: settings read failed")
    }

    /// Get a write-locked reference to settings
    pub fn write_settings(&self) -> std::sync::RwLockWriteGuard<'_, AppSettings> {
        self.settings
            .write()
            .expect("RwLock poisoned: settings write failed")
    }

    /// Update hotkeys in memory and persist to disk
    pub fn update_and_save_hotkeys(
        &self,
        app_handle: &tauri::AppHandle,
        mappings: HotkeyMappings,
    ) -> Result<(), String> {
        // Write to disk first (fail fast if disk error)
        crate::hotkeys::save(&mappings, app_handle)?;

        // Update in-memory state
        *self.write_hotkeys() = mappings;

        tracing::debug!("Hotkeys updated in memory and persisted to disk");
        Ok(())
    }

    /// Update sound library in memory and persist to disk
    pub fn update_and_save_sounds(
        &self,
        app_handle: &tauri::AppHandle,
        library: SoundLibrary,
    ) -> Result<(), String> {
        // Write to disk first (fail fast if disk error)
        crate::sounds::save(&library, app_handle)?;

        // Update in-memory state
        *self.write_sounds() = library;

        tracing::debug!("Sound library updated in memory and persisted to disk");
        Ok(())
    }

    /// Update settings in memory and persist to disk
    pub fn update_and_save_settings(
        &self,
        app_handle: &tauri::AppHandle,
        settings: AppSettings,
    ) -> Result<(), String> {
        // Write to disk first (fail fast if disk error)
        crate::settings::save(&settings, app_handle)?;

        // Update in-memory state
        *self.write_settings() = settings;

        tracing::debug!("Settings updated in memory and persisted to disk");
        Ok(())
    }
}
