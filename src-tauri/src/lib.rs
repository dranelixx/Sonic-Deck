//! SonicDeck - High-performance Desktop Soundboard
//!
//! Rust backend with dual-output audio routing (cpal-based implementation).

mod audio;
mod settings;
mod sounds;

use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use cpal::traits::HostTrait;
use tauri::{Emitter, State};
use tracing::{info, error};

pub use audio::{AudioDevice, AudioManager, CacheStats, DeviceId, WaveformData};
pub use settings::AppSettings;
pub use sounds::{Sound, SoundId, Category, CategoryId, SoundLibrary};

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

        let (Some(device_1), Some(device_2)) = (output_devices.get(idx1), output_devices.get(idx2)) else {
            error!("Devices not found at indices {} and {}", idx1, idx2);
            manager_inner.lock().unwrap().remove(&playback_id_clone);
            return;
        };

        // Calculate trim frames from milliseconds
        let sample_rate = audio_data.sample_rate;
        let start_frame = trim_start_ms.map(|ms| ((ms as f64 / 1000.0) * sample_rate as f64) as usize);
        let end_frame = trim_end_ms.map(|ms| ((ms as f64 / 1000.0) * sample_rate as f64) as usize);

        // Create streams with shared volume state and trim parameters
        let stream_1 = match audio::create_playback_stream(device_1, audio_data.clone(), volume_state.clone(), start_frame, end_frame) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create stream 1: {}", e);
                manager_inner.lock().unwrap().remove(&playback_id_clone);
                return;
            }
        };

        let stream_2 = match audio::create_playback_stream(device_2, audio_data.clone(), volume_state.clone(), start_frame, end_frame) {
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
            let progress_pct = ((elapsed_ms as f64 / total_sleep_ms as f64) * 100.0).min(100.0) as u8;
            let _ = app_handle.emit("playback-progress", PlaybackProgress {
                playback_id: playback_id_clone.clone(),
                elapsed_ms,
                total_ms: total_sleep_ms,
                progress_pct,
            });
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
fn stop_playback(
    playback_id: String,
    manager: State<'_, AudioManager>,
) -> Result<(), String> {
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
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "log")
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    
    if log_files.is_empty() {
        return Ok("No log files found.".to_string());
    }
    
    // Get the most recent log file (by modified time)
    let most_recent = log_files.iter()
        .max_by_key(|entry| {
            entry.metadata()
                .and_then(|m| m.modified())
                .ok()
        })
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
fn save_settings(
    settings: AppSettings,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    settings::save(&settings, &app_handle)
}

/// Get the settings file path (for debugging/info)
#[tauri::command]
fn get_settings_file_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    let path = settings::get_settings_path(&app_handle)?;
    Ok(path.to_string_lossy().to_string())
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
        None, // Don't change is_favorite here
        Some(trim_start_ms), // Update trim_start_ms
        Some(trim_end_ms),   // Update trim_end_ms
    )?;
    
    sounds::save(&library, &app_handle)?;
    Ok(sound)
}

/// Toggle favorite status of a sound
#[tauri::command]
fn toggle_favorite(
    sound_id: SoundId,
    app_handle: tauri::AppHandle,
) -> Result<Sound, String> {
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
            load_sounds,
            add_sound,
            update_sound,
            toggle_favorite,
            delete_sound,
            add_category,
            update_category,
            delete_category,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
