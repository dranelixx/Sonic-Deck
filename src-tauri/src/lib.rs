//! SonicDeck - High-performance Desktop Soundboard
//!
//! Rust backend with dual-output audio routing (cpal-based implementation).

mod audio;
mod hotkeys;
mod settings;
mod sounds;
mod tray;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::HostTrait;
use tauri::{Emitter, State};
use tracing::{error, info};

pub use audio::{AudioDevice, AudioManager, CacheStats, DeviceId, WaveformData};
pub use settings::AppSettings;
pub use sounds::{Category, CategoryId, Sound, SoundId, SoundLibrary};

/// Playback progress event payload
#[derive(Clone, serde::Serialize)]
struct PlaybackProgress {
    playback_id: String,
    elapsed_ms: u64,
    total_ms: u64,
    progress_pct: u8,
}

// ============================================================================
// TAURI COMMANDS - Audio
// ============================================================================

/// Lists all available output audio devices on the system
#[tauri::command]
fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    audio::enumerate_devices().map_err(Into::into)
}

/// Plays an audio file simultaneously to two different output devices
#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn play_dual_output(
    file_path: String,
    device_id_1: DeviceId,
    device_id_2: DeviceId,
    volume: f32,
    trim_start_ms: Option<u64>,
    trim_end_ms: Option<u64>,
    manager: State<'_, AudioManager>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let volume = volume.clamp(0.0, 1.0);

    // Generate playback ID
    let playback_id = manager.next_playback_id();

    // Create stop channel
    let (stop_tx, stop_rx) = mpsc::channel();

    // Register the playback
    manager.register_playback(playback_id.clone(), stop_tx);

    // Create shared volume state for dynamic control
    let volume_state = Arc::new(Mutex::new(volume));

    // Clone for the thread
    let playback_id_clone = playback_id.clone();
    let manager_inner = manager.get_stop_senders();
    let cache = manager.get_cache();

    // Spawn dedicated playback thread (including decoding to avoid blocking UI)
    thread::spawn(move || {
        // Get audio from cache or decode (cache handles the logic)
        let audio_data = match cache.lock().unwrap().get_or_decode(&file_path) {
            Ok(data) => data, // Already Arc<AudioData>
            Err(e) => {
                error!("Failed to decode audio: {}", e);
                manager_inner.lock().unwrap().remove(&playback_id_clone);
                // Emit error event
                let _ = app_handle.emit("audio-decode-error", format!("Failed to decode: {}", e));
                return;
            }
        };

        // Emit event that decoding is complete and playback is starting
        let _ = app_handle.emit("audio-decode-complete", &playback_id_clone);

        // This thread owns the streams - no Send issues!
        let host = cpal::default_host();

        let output_devices: Vec<_> = match host.output_devices() {
            Ok(devices) => devices.collect(),
            Err(e) => {
                error!("Failed to enumerate devices: {}", e);
                manager_inner.lock().unwrap().remove(&playback_id_clone);
                return;
            }
        };

        // Parse device indices
        let (idx1, idx2) = match (device_id_1.index(), device_id_2.index()) {
            (Ok(i1), Ok(i2)) => (i1, i2),
            _ => {
                error!("Invalid device IDs: {} / {}", device_id_1, device_id_2);
                manager_inner.lock().unwrap().remove(&playback_id_clone);
                return;
            }
        };

        let (Some(device_1), Some(device_2)) = (output_devices.get(idx1), output_devices.get(idx2))
        else {
            error!("Devices not found at indices {} and {}", idx1, idx2);
            manager_inner.lock().unwrap().remove(&playback_id_clone);
            return;
        };

        // Calculate trim frames from milliseconds
        let sample_rate = audio_data.sample_rate;
        let start_frame =
            trim_start_ms.map(|ms| ((ms as f64 / 1000.0) * sample_rate as f64) as usize);
        let end_frame = trim_end_ms.map(|ms| ((ms as f64 / 1000.0) * sample_rate as f64) as usize);

        // Create streams with shared volume state and trim parameters
        let stream_1 = match audio::create_playback_stream(
            device_1,
            audio_data.clone(),
            volume_state.clone(),
            start_frame,
            end_frame,
        ) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create stream 1: {}", e);
                manager_inner.lock().unwrap().remove(&playback_id_clone);
                return;
            }
        };

        let stream_2 = match audio::create_playback_stream(
            device_2,
            audio_data.clone(),
            volume_state.clone(),
            start_frame,
            end_frame,
        ) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create stream 2: {}", e);
                manager_inner.lock().unwrap().remove(&playback_id_clone);
                return;
            }
        };

        // Calculate duration (with trim)
        let total_frames = audio_data.samples.len() / audio_data.channels as usize;
        let actual_start = start_frame.unwrap_or(0);
        let actual_end = end_frame.unwrap_or(total_frames);
        let trimmed_frames = actual_end.saturating_sub(actual_start);

        let duration_secs = trimmed_frames as f64 / audio_data.sample_rate as f64;
        let total_sleep_ms = (duration_secs * 1000.0) as u64;

        // Wait for completion or stop signal, emitting progress events
        let check_interval = Duration::from_millis(50); // 50ms for smoother progress updates
        let mut elapsed_ms = 0u64;

        while elapsed_ms < total_sleep_ms {
            // Check for stop signal
            if stop_rx.try_recv().is_ok() {
                break;
            }

            thread::sleep(check_interval);
            elapsed_ms += 50;

            // Emit progress event
            let progress_pct =
                ((elapsed_ms as f64 / total_sleep_ms as f64) * 100.0).min(100.0) as u8;
            let _ = app_handle.emit(
                "playback-progress",
                PlaybackProgress {
                    playback_id: playback_id_clone.clone(),
                    elapsed_ms,
                    total_ms: total_sleep_ms,
                    progress_pct,
                },
            );
        }

        // Clean up
        drop(stream_1);
        drop(stream_2);
        manager_inner.lock().unwrap().remove(&playback_id_clone);

        // Emit playback complete event
        let _ = app_handle.emit("playback-complete", &playback_id_clone);
    });

    Ok(playback_id)
}

/// Stops all currently playing audio
#[tauri::command]
fn stop_all_audio(manager: State<'_, AudioManager>) -> Result<(), String> {
    manager.stop_all();
    Ok(())
}

/// Stops a specific playback by ID
#[tauri::command]
fn stop_playback(playback_id: String, manager: State<'_, AudioManager>) -> Result<(), String> {
    if manager.signal_stop(&playback_id) {
        Ok(())
    } else {
        Err(format!("Playback not found: {}", playback_id))
    }
}

/// Clear the audio cache (forces re-decoding on next play)
#[tauri::command]
fn clear_audio_cache(manager: State<'_, AudioManager>) -> Result<(), String> {
    manager.clear_cache();
    Ok(())
}

/// Get audio cache statistics
#[tauri::command]
fn get_cache_stats(manager: State<'_, AudioManager>) -> Result<CacheStats, String> {
    Ok(manager.cache_stats())
}

/// Get logs directory path
#[tauri::command]
fn get_logs_path() -> Result<String, String> {
    let logs_dir = dirs::data_local_dir()
        .ok_or("Could not find app data directory")?
        .join("com.sonicdeck.app")
        .join("logs");

    Ok(logs_dir.to_string_lossy().to_string())
}

/// Read the current log file
#[tauri::command]
fn read_logs() -> Result<String, String> {
    let logs_dir = dirs::data_local_dir()
        .ok_or("Could not find app data directory")?
        .join("com.sonicdeck.app")
        .join("logs");

    // Find the most recent log file
    let log_files = std::fs::read_dir(&logs_dir)
        .map_err(|e| format!("Failed to read logs directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "log")
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    if log_files.is_empty() {
        return Ok("No log files found.".to_string());
    }

    // Get the most recent log file (by modified time)
    let most_recent = log_files
        .iter()
        .max_by_key(|entry| entry.metadata().and_then(|m| m.modified()).ok())
        .ok_or("Failed to find recent log file")?;

    std::fs::read_to_string(most_recent.path())
        .map_err(|e| format!("Failed to read log file: {}", e))
}

/// Clear all log files
#[tauri::command]
fn clear_logs() -> Result<(), String> {
    let logs_dir = dirs::data_local_dir()
        .ok_or("Could not find app data directory")?
        .join("com.sonicdeck.app")
        .join("logs");

    if !logs_dir.exists() {
        return Ok(());
    }

    let log_files = std::fs::read_dir(&logs_dir)
        .map_err(|e| format!("Failed to read logs directory: {}", e))?;

    for entry in log_files.filter_map(|e| e.ok()) {
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("log") {
            std::fs::remove_file(entry.path())
                .map_err(|e| format!("Failed to delete log file: {}", e))?;
        }
    }

    info!("Log files cleared by user");
    Ok(())
}

/// Get waveform data for an audio file
#[tauri::command]
fn get_waveform(
    file_path: String,
    num_peaks: usize,
    manager: State<'_, AudioManager>,
) -> Result<WaveformData, String> {
    // Use cache to get or decode the audio
    let audio_data = manager
        .get_cache()
        .lock()
        .unwrap()
        .get_or_decode(&file_path)
        .map_err(|e| e.to_string())?;

    // Generate waveform peaks
    let waveform = audio::generate_peaks(&audio_data, num_peaks);
    Ok(waveform)
}

// ============================================================================
// TAURI COMMANDS - Settings
// ============================================================================

/// Load application settings from disk
#[tauri::command]
fn load_settings(app_handle: tauri::AppHandle) -> Result<AppSettings, String> {
    settings::load(&app_handle)
}

/// Save application settings to disk
#[tauri::command]
fn save_settings(settings: AppSettings, app_handle: tauri::AppHandle) -> Result<(), String> {
    settings::save(&settings, &app_handle)
}

/// Get the settings file path (for debugging/info)
#[tauri::command]
fn get_settings_file_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    let path = settings::get_settings_path(&app_handle)?;
    Ok(path.to_string_lossy().to_string())
}

/// Enable autostart on system boot
#[tauri::command]
fn enable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
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
fn disable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
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
fn is_autostart_enabled(app_handle: tauri::AppHandle) -> Result<bool, String> {
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

// ============================================================================
// TAURI COMMANDS - Global Hotkeys
// ============================================================================

/// Load hotkey mappings from disk
#[tauri::command]
fn load_hotkeys(app_handle: tauri::AppHandle) -> Result<hotkeys::HotkeyMappings, String> {
    hotkeys::load(&app_handle)
}

/// Save hotkey mappings to disk
#[tauri::command]
fn save_hotkeys(
    mappings: hotkeys::HotkeyMappings,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    hotkeys::save(&mappings, &app_handle)
}

/// Register a global hotkey for a sound
#[tauri::command]
fn register_hotkey(
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
fn unregister_hotkey(hotkey: String, app_handle: tauri::AppHandle) -> Result<(), String> {
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
fn is_hotkey_registered(hotkey: String, app_handle: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let shortcut = hotkey
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map_err(|e| format!("Failed to parse hotkey '{}': {}", hotkey, e))?;
    Ok(app_handle.global_shortcut().is_registered(shortcut))
}

// ============================================================================
// TAURI COMMANDS - Sound Library
// ============================================================================

/// Load the sound library
#[tauri::command]
fn load_sounds(app_handle: tauri::AppHandle) -> Result<SoundLibrary, String> {
    sounds::load(&app_handle)
}

/// Add a new sound to the library
#[tauri::command]
fn add_sound(
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
fn update_sound(
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
fn toggle_favorite(sound_id: SoundId, app_handle: tauri::AppHandle) -> Result<Sound, String> {
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
fn delete_sound(sound_id: SoundId, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut library = sounds::load(&app_handle)?;
    sounds::delete_sound(&mut library, &sound_id)?;
    sounds::save(&library, &app_handle)?;
    Ok(())
}

/// Add a new category
#[tauri::command]
fn add_category(
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
fn update_category(
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
fn delete_category(
    category_id: CategoryId,
    move_sounds_to: Option<CategoryId>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut library = sounds::load(&app_handle)?;
    sounds::delete_category(&mut library, &category_id, move_sounds_to)?;
    sounds::save(&library, &app_handle)?;
    Ok(())
}

// ============================================================================
// GLOBAL SHORTCUT HANDLING
// ============================================================================

/// Normalize hotkey string to match our storage format
fn normalize_hotkey_string(hotkey: &str) -> String {
    // Split by + and normalize each part
    let parts: Vec<&str> = hotkey.split('+').collect();
    let normalized_parts: Vec<String> = parts
        .iter()
        .map(|part| {
            let trimmed = part.trim();
            match trimmed.to_lowercase().as_str() {
                "control" => "Ctrl".to_string(),
                "alt" => "Alt".to_string(),
                "shift" => "Shift".to_string(),
                "meta" => "Super".to_string(),
                // Handle NumPad keys
                "numpad0" => "NumPad0".to_string(),
                "numpad1" => "NumPad1".to_string(),
                "numpad2" => "NumPad2".to_string(),
                "numpad3" => "NumPad3".to_string(),
                "numpad4" => "NumPad4".to_string(),
                "numpad5" => "NumPad5".to_string(),
                "numpad6" => "NumPad6".to_string(),
                "numpad7" => "NumPad7".to_string(),
                "numpad8" => "NumPad8".to_string(),
                "numpad9" => "NumPad9".to_string(),
                "numpaddecimal" => "NumPadDecimal".to_string(),
                "numpadenter" => "NumPadEnter".to_string(),
                "numpadadd" => "NumPadAdd".to_string(),
                "numpadsubtract" => "NumPadSubtract".to_string(),
                "numpadmultiply" => "NumPadMultiply".to_string(),
                "numpaddivide" => "NumPadDivide".to_string(),
                // Keep other keys as-is but capitalize first letter for consistency
                other => {
                    if !other.is_empty() {
                        let mut chars = other.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().chain(chars).collect(),
                        }
                    } else {
                        other.to_string()
                    }
                }
            }
        })
        .collect();

    normalized_parts.join("+")
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

    // Load hotkey mappings
    let mappings = match hotkeys::load(app) {
        Ok(m) => {
            tracing::info!("Loaded {} hotkey mappings", m.mappings.len());
            for (key, sound_id) in &m.mappings {
                tracing::debug!("  Mapping: '{}' -> {:?}", key, sound_id);
            }
            m
        }
        Err(e) => {
            tracing::error!("Failed to load hotkey mappings: {}", e);
            return;
        }
    };

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

    // Load sound library
    let library = match sounds::load(app) {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to load sound library: {}", e);
            return;
        }
    };

    // Find the sound
    let sound = match library.sounds.iter().find(|s| s.id == sound_id) {
        Some(s) => s,
        None => {
            tracing::warn!(
                "Sound not found for hotkey: {} -> {:?}",
                hotkey_str,
                sound_id
            );
            return;
        }
    };

    // Load settings
    let settings = match settings::load(app) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to load settings: {}", e);
            return;
        }
    };

    // Get device IDs
    let device1 = match settings.monitor_device_id {
        Some(ref id) => id.clone(),
        None => {
            tracing::warn!("No monitor device configured");
            return;
        }
    };

    let device2 = match settings.broadcast_device_id {
        Some(ref id) => id.clone(),
        None => {
            tracing::warn!("No broadcast device configured");
            return;
        }
    };

    // Determine volume
    let volume = sound.volume.unwrap_or(settings.default_volume);

    // Get audio manager from state
    use tauri::Manager as TauriManager;
    let manager = app.state::<AudioManager>();

    // Trigger playback
    match play_dual_output(
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
        .manage(AudioManager::new())
        .invoke_handler(tauri::generate_handler![
            list_audio_devices,
            play_dual_output,
            stop_all_audio,
            stop_playback,
            clear_audio_cache,
            get_cache_stats,
            get_logs_path,
            read_logs,
            clear_logs,
            get_waveform,
            load_settings,
            save_settings,
            get_settings_file_path,
            enable_autostart,
            disable_autostart,
            is_autostart_enabled,
            load_hotkeys,
            save_hotkeys,
            register_hotkey,
            unregister_hotkey,
            is_hotkey_registered,
            load_sounds,
            add_sound,
            update_sound,
            toggle_favorite,
            delete_sound,
            add_category,
            update_category,
            delete_category,
        ])
        .setup(|app| {
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

                // Apply saved autostart setting
                let settings = settings::load(app.handle()).unwrap_or_default();
                let autostart_manager = app.autolaunch();
                if settings.autostart_enabled {
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

                // Optionally start minimized
                let settings = settings::load(app.handle()).unwrap_or_default();
                if settings.start_minimized {
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
