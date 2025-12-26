//! Sonic Deck - High-performance Desktop Soundboard
//!
//! Rust backend with dual-output audio routing (cpal-based implementation).

mod audio;
mod commands;
mod hotkeys;
mod persistence;
mod settings;
mod sounds;
mod state;
mod tray;

use tauri::Manager;
use tracing::{error, info};

pub use audio::{AudioDevice, AudioManager, CacheStats, DeviceId, WaveformData};
pub use settings::AppSettings;
pub use sounds::{Category, CategoryId, Sound, SoundId, SoundLibrary};
pub use state::AppState;
// ============================================================================
// GLOBAL SHORTCUT HANDLING
// ============================================================================

/// Normalize hotkey string to match our storage format
fn normalize_hotkey_string(hotkey: &str) -> String {
    hotkey
        .split('+')
        .map(|part| {
            let trimmed = part.trim().to_lowercase();

            // Handle modifier keys
            match trimmed.as_str() {
                "control" => "Ctrl".to_string(),
                "alt" => "Alt".to_string(),
                "shift" => "Shift".to_string(),
                "meta" => "Super".to_string(),
                // Handle NumPad keys: numpad0 -> NumPad0, numpadadd -> NumPadAdd
                key if key.starts_with("numpad") => {
                    format!("NumPad{}", capitalize_first(&key[6..]))
                }
                // Handle Digit keys: digit0 -> 0, digit1 -> 1, etc.
                key if key.starts_with("digit") => key[5..].to_string(),
                // Handle Key prefix: keya -> A, keyb -> B, etc.
                key if key.starts_with("key") => key[3..].to_uppercase(),
                // Capitalize first letter for all other keys
                other => capitalize_first(other),
            }
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Handle global shortcut events
#[cfg(desktop)]
fn handle_global_shortcut(
    app: &tauri::AppHandle,
    shortcut: &tauri_plugin_global_shortcut::Shortcut,
    event: &tauri_plugin_global_shortcut::ShortcutEvent,
) {
    use tauri_plugin_global_shortcut::ShortcutState;

    let hotkey_str = shortcut.to_string();

    // Normalize the hotkey string to match our stored format
    let normalized_hotkey = normalize_hotkey_string(&hotkey_str);

    tracing::info!(
        "Global shortcut event received: {} -> normalized: {} (state: {:?})",
        hotkey_str,
        normalized_hotkey,
        event.state
    );

    // Only handle pressed state
    if event.state != ShortcutState::Pressed {
        tracing::debug!("Ignoring non-pressed state: {:?}", event.state);
        return;
    }

    tracing::info!("Processing hotkey press: {}", normalized_hotkey);

    // Get app state (zero disk I/O)
    use tauri::Manager as TauriManager;
    let app_state = app.state::<AppState>();

    // Read hotkey mappings from in-memory state
    let mappings = app_state.read_hotkeys();
    tracing::debug!(
        "Read {} hotkey mappings from memory",
        mappings.mappings.len()
    );

    // Get sound ID for this hotkey using the normalized string
    let sound_id = match hotkeys::get_sound_id(&mappings, &normalized_hotkey) {
        Some(id) => {
            tracing::info!("Found sound mapping: '{}' -> {:?}", normalized_hotkey, id);
            id.clone()
        }
        None => {
            tracing::warn!(
                "No sound mapped to hotkey: '{}'. Available mappings:",
                normalized_hotkey
            );
            for key in mappings.mappings.keys() {
                tracing::warn!("  Available: '{}'", key);
            }
            return;
        }
    };
    drop(mappings); // Release read lock early

    // Read sound library from in-memory state
    let library = app_state.read_sounds();

    // Find the sound
    let sound = match library.sounds.iter().find(|s| s.id == sound_id) {
        Some(s) => s.clone(),
        None => {
            tracing::warn!(
                "Sound not found for hotkey: {} -> {:?}",
                hotkey_str,
                sound_id
            );
            return;
        }
    };
    drop(library); // Release read lock early

    // Read settings from in-memory state
    let settings = app_state.read_settings();
    let monitor_device = settings.monitor_device_id.clone();
    let broadcast_device = settings.broadcast_device_id.clone();
    let default_volume = settings.default_volume;
    drop(settings); // Release read lock early

    // Get device IDs
    let device1 = match monitor_device {
        Some(id) => id,
        None => {
            tracing::warn!("No monitor device configured");
            return;
        }
    };

    let device2 = match broadcast_device {
        Some(id) => id,
        None => {
            tracing::warn!("No broadcast device configured");
            return;
        }
    };

    // Determine volume
    let volume = sound.volume.unwrap_or(default_volume);

    // Get audio manager from state
    let manager = app.state::<AudioManager>();

    // Trigger playback
    match commands::play_dual_output(
        sound.file_path.clone(),
        device1,
        device2,
        volume,
        sound.trim_start_ms,
        sound.trim_end_ms,
        manager,
        app.clone(),
    ) {
        Ok(playback_id) => {
            tracing::info!(
                "Hotkey '{}' triggered sound '{}' (playback: {})",
                normalized_hotkey,
                sound.name,
                playback_id
            );
        }
        Err(e) => {
            tracing::error!("Failed to play sound from hotkey: {}", e);
        }
    }
}

/// Register all saved hotkeys on app startup
#[cfg(desktop)]
fn register_saved_hotkeys(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let mappings = hotkeys::load(app)?;

    for (hotkey, sound_id) in &mappings.mappings {
        if let Ok(shortcut) = hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
            match app.global_shortcut().register(shortcut) {
                Ok(_) => {
                    tracing::info!("Registered saved hotkey: {} -> {:?}", hotkey, sound_id);
                }
                Err(e) => {
                    tracing::error!("Failed to register saved hotkey '{}': {}", hotkey, e);
                }
            }
        } else {
            tracing::error!("Failed to parse saved hotkey: {}", hotkey);
        }
    }

    Ok(())
}

/// Clean up orphaned hotkeys (hotkeys for sounds that no longer exist)
#[cfg(desktop)]
fn cleanup_orphaned_hotkeys(app: &tauri::AppHandle) -> Result<(), String> {
    use std::collections::HashSet;
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let mut mappings = hotkeys::load(app)?;
    let library = sounds::load(app)?;

    // Create set of valid sound IDs
    let valid_ids: HashSet<_> = library.sounds.iter().map(|s| &s.id).collect();

    // Track orphaned hotkeys
    let mut orphaned = Vec::new();

    // Find orphaned hotkeys
    for (hotkey, sound_id) in &mappings.mappings {
        if !valid_ids.contains(sound_id) {
            tracing::warn!("Removing orphaned hotkey: {} -> {:?}", hotkey, sound_id);
            orphaned.push(hotkey.clone());
        }
    }

    // Remove orphaned hotkeys
    for hotkey in orphaned {
        hotkeys::remove_mapping(&mut mappings, &hotkey)?;
        if let Ok(shortcut) = hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
            let _ = app.global_shortcut().unregister(shortcut);
        }
    }

    // Save cleaned mappings
    if !mappings.mappings.is_empty() {
        hotkeys::save(&mappings, app)?;
    }

    Ok(())
}

// ============================================================================
// TAURI APP INITIALIZATION
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_audio_devices,
            commands::play_dual_output,
            commands::stop_all_audio,
            commands::stop_playback,
            commands::clear_audio_cache,
            commands::get_cache_stats,
            commands::get_logs_path,
            commands::read_logs,
            commands::clear_logs,
            commands::get_waveform,
            commands::load_settings,
            commands::save_settings,
            commands::get_settings_file_path,
            commands::enable_autostart,
            commands::disable_autostart,
            commands::is_autostart_enabled,
            commands::load_hotkeys,
            commands::save_hotkeys,
            commands::register_hotkey,
            commands::unregister_hotkey,
            commands::is_hotkey_registered,
            commands::load_sounds,
            commands::add_sound,
            commands::update_sound,
            commands::toggle_favorite,
            commands::delete_sound,
            commands::add_category,
            commands::update_category,
            commands::delete_category,
        ])
        .setup(|app| {
            // Initialize app state (load all data from disk once at startup)
            let app_state = AppState::load(app.handle())?;

            // Initialize audio manager
            let audio_manager = AudioManager::new();

            // Register state managers
            app.manage(app_state);
            app.manage(audio_manager);

            #[cfg(desktop)]
            {
                use tauri::Manager;
                use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

                // Initialize autostart plugin
                app.handle()
                    .plugin(tauri_plugin_autostart::init(
                        MacosLauncher::LaunchAgent,
                        None::<Vec<&str>>,
                    ))
                    .map_err(|e| format!("Failed to initialize autostart plugin: {}", e))?;

                // Apply saved autostart setting from in-memory state
                let state = app.state::<AppState>();
                let settings = state.read_settings();
                let autostart_enabled = settings.autostart_enabled;
                drop(settings);

                let autostart_manager = app.autolaunch();
                if autostart_enabled {
                    let _ = autostart_manager.enable();
                } else {
                    let _ = autostart_manager.disable();
                }

                // Initialize global shortcut plugin
                app.handle()
                    .plugin(
                        tauri_plugin_global_shortcut::Builder::new()
                            .with_handler(|app, shortcut, event| {
                                handle_global_shortcut(app, shortcut, &event);
                            })
                            .build(),
                    )
                    .map_err(|e| format!("Failed to initialize global shortcut plugin: {}", e))?;

                // Cleanup orphaned hotkeys
                if let Err(e) = cleanup_orphaned_hotkeys(app.handle()) {
                    error!("Failed to cleanup orphaned hotkeys: {}", e);
                }

                // Register saved hotkeys
                if let Err(e) = register_saved_hotkeys(app.handle()) {
                    error!("Failed to register saved hotkeys: {}", e);
                }

                // Initialize system tray
                if let Err(e) = tray::init(app.handle()) {
                    error!("Failed to initialize system tray: {}", e);
                }

                // Optionally start minimized (read from in-memory state)
                let state = app.state::<AppState>();
                let settings = state.read_settings();
                let start_minimized = settings.start_minimized;
                drop(settings);

                if start_minimized {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                        info!("Started minimized to tray");
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
