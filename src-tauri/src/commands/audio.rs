//! Audio-related Tauri commands
//!
//! This module contains commands for:
//! - Audio device enumeration
//! - Dual-output playback
//! - Playback control (play, stop)
//! - Audio cache management
//! - Waveform generation

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::HostTrait;
use tauri::{Emitter, State};
use tracing::{debug, error, info};

use crate::audio::{
    self, AudioDevice, AudioManager, CacheStats, DeviceId, SoundState, WaveformData,
};
use crate::state::AppState;

/// Playback progress event payload
#[derive(Clone, serde::Serialize)]
struct PlaybackProgress {
    playback_id: String,
    elapsed_ms: u64,
    total_ms: u64,
    progress_pct: u8,
}

/// Lists all available output audio devices on the system
#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    audio::enumerate_devices().map_err(Into::into)
}

/// Result of play_dual_output indicating what action was taken
#[derive(Clone, serde::Serialize)]
pub struct PlaybackResult {
    /// The playback ID (if playback started)
    pub playback_id: Option<String>,
    /// Action taken: "started", "restarted", "ignored"
    pub action: String,
    /// Previous playback ID that was stopped (if restarted)
    pub stopped_playback_id: Option<String>,
}

/// Plays an audio file simultaneously to two different output devices
///
/// # LUFS Normalization
///
/// When LUFS normalization is enabled in settings, the sound's LUFS value
/// (calculated during decode) is used to adjust playback volume to match
/// the target loudness level.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn play_dual_output(
    file_path: String,
    device_id_1: DeviceId,
    device_id_2: DeviceId,
    volume: f32,
    trim_start_ms: Option<u64>,
    trim_end_ms: Option<u64>,
    sound_id: Option<String>,
    manager: State<'_, AudioManager>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<PlaybackResult, String> {
    let volume = volume.clamp(0.0, 1.0);
    let sound_id = sound_id.unwrap_or_default();

    // Read LUFS normalization settings from AppState
    let settings = state.read_settings();
    let enable_lufs = settings.enable_lufs_normalization;
    let target_lufs = settings.target_lufs;
    drop(settings); // Release read lock early

    debug!(
        sound_id = %sound_id,
        file_path = %file_path,
        volume = volume,
        enable_lufs = enable_lufs,
        target_lufs = target_lufs,
        "Playback requested"
    );

    // Minimum time a sound must AUDIBLY play before it can be restarted
    // Lower = more responsive/snappy, but too low may cause audio glitches
    const MIN_PLAY_TIME_MS: u64 = 15;

    // Generate playback ID first
    let playback_id = manager.next_playback_id();

    // Check if this sound is already active and apply policy
    let mut stopped_playback_id: Option<String> = None;
    if !sound_id.is_empty() {
        if let Some(current_state) = manager.get_sound_state(&sound_id) {
            let current_playback_id = current_state.playback_id().to_string();

            // Check cooldown only if sound is actually playing (audible)
            let should_apply_cooldown = match &current_state {
                SoundState::Playing { started_at, .. } => {
                    (started_at.elapsed().as_millis() as u64) < MIN_PLAY_TIME_MS
                }
                SoundState::Decoding { .. } => false, // No cooldown during decoding
            };

            if should_apply_cooldown {
                debug!(
                    "Cooldown: Ignoring trigger for {} (still in cooldown)",
                    sound_id
                );
                return Ok(PlaybackResult {
                    playback_id: None,
                    action: "ignored".to_string(),
                    stopped_playback_id: None,
                });
            }

            // Restart sound (works for both Decoding and Playing states)
            info!("Restarting {} (was {})", sound_id, current_playback_id);
            manager.register_sound_decoding(sound_id.clone(), playback_id.clone());
            stopped_playback_id = Some(current_playback_id);
        } else {
            // No current playback, register as decoding
            manager.register_sound_decoding(sound_id.clone(), playback_id.clone());
        }
    }

    // Create stop channel
    let (stop_tx, stop_rx) = mpsc::channel();

    // Register the playback
    manager.register_playback(playback_id.clone(), stop_tx);

    // Create shared volume state for dynamic control
    let volume_state = Arc::new(Mutex::new(volume));

    // Clone for the thread
    let playback_id_clone = playback_id.clone();
    let manager_inner = manager.get_stop_senders();
    let active_sounds = manager.get_active_sounds();
    let cache = manager.get_cache();
    let sound_id_clone = sound_id.clone();
    let old_playback_to_stop = stopped_playback_id.clone();
    // LUFS settings are captured here (read once at playback start, not per-sample)
    let enable_lufs_clone = enable_lufs;
    let target_lufs_clone = target_lufs;

    // Spawn dedicated playback thread (including decoding to avoid blocking UI)
    thread::spawn(move || {
        let thread_start = Instant::now();

        // Helper to clean up on early return (before playback starts)
        let cleanup_early =
            |manager_inner: &Arc<Mutex<std::collections::HashMap<String, _>>>,
             active_sounds: &Arc<Mutex<std::collections::HashMap<String, SoundState>>>,
             playback_id: &str,
             sound_id: &str| {
                manager_inner.lock().unwrap().remove(playback_id);
                if !sound_id.is_empty() {
                    let mut sounds = active_sounds.lock().unwrap();
                    // Only remove if this is still our playback
                    if let Some(state) = sounds.get(sound_id) {
                        if state.playback_id() == playback_id {
                            sounds.remove(sound_id);
                        }
                    }
                }
            };

        // Get audio from cache or decode (cache handles the logic)
        let audio_data = match cache.lock().unwrap().get_or_decode(&file_path) {
            Ok(data) => data, // Already Arc<AudioData>
            Err(e) => {
                error!("Failed to decode audio: {}", e);
                cleanup_early(
                    &manager_inner,
                    &active_sounds,
                    &playback_id_clone,
                    &sound_id_clone,
                );
                // Emit error event
                if let Err(emit_err) =
                    app_handle.emit("audio-decode-error", format!("Failed to decode: {}", e))
                {
                    error!("Failed to emit decode error event: {}", emit_err);
                }
                return;
            }
        };

        // Emit event that decoding is complete and playback is starting
        if let Err(e) = app_handle.emit("audio-decode-complete", &playback_id_clone) {
            error!("Failed to emit decode complete event: {}", e);
        }

        // This thread owns the streams - no Send issues!
        let host = cpal::default_host();

        let enum_start = Instant::now();
        let output_devices: Vec<_> = match host.output_devices() {
            Ok(devices) => devices.collect(),
            Err(e) => {
                error!("Failed to enumerate devices: {}", e);
                cleanup_early(
                    &manager_inner,
                    &active_sounds,
                    &playback_id_clone,
                    &sound_id_clone,
                );
                return;
            }
        };

        let enum_duration = enum_start.elapsed().as_millis();
        debug!(
            duration_ms = enum_duration,
            device_count = output_devices.len(),
            "Device enumeration complete"
        );

        // Parse device indices
        let (idx1, idx2) = match (device_id_1.index(), device_id_2.index()) {
            (Ok(i1), Ok(i2)) => (i1, i2),
            _ => {
                let error_msg = format!("Invalid device IDs: {} / {}", device_id_1, device_id_2);
                error!("{}", error_msg);
                if let Err(e) = app_handle.emit("audio-device-error", error_msg) {
                    error!("Failed to emit device error event: {}", e);
                }
                cleanup_early(
                    &manager_inner,
                    &active_sounds,
                    &playback_id_clone,
                    &sound_id_clone,
                );
                return;
            }
        };

        let (Some(device_1), Some(device_2)) = (output_devices.get(idx1), output_devices.get(idx2))
        else {
            error!("Devices not found at indices {} and {}", idx1, idx2);
            cleanup_early(
                &manager_inner,
                &active_sounds,
                &playback_id_clone,
                &sound_id_clone,
            );
            return;
        };

        // Calculate trim frames from milliseconds
        let sample_rate = audio_data.sample_rate;
        let start_frame =
            trim_start_ms.map(|ms| ((ms as f64 / 1000.0) * sample_rate as f64) as usize);
        let end_frame = trim_end_ms.map(|ms| ((ms as f64 / 1000.0) * sample_rate as f64) as usize);

        // Calculate LUFS gain for normalization (once at stream creation, not per-sample)
        let lufs_gain =
            audio::calculate_lufs_gain(audio_data.lufs, target_lufs_clone, enable_lufs_clone);

        if enable_lufs_clone {
            debug!(
                sound_lufs = ?audio_data.lufs,
                target_lufs = target_lufs_clone,
                lufs_gain = format!("{:.3}", lufs_gain),
                "LUFS normalization applied"
            );
        }

        // Create streams with shared volume state, trim parameters, and LUFS gain
        let stream_1 = match audio::create_playback_stream(
            device_1,
            audio_data.clone(),
            volume_state.clone(),
            start_frame,
            end_frame,
            lufs_gain,
        ) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create stream 1: {}", e);
                cleanup_early(
                    &manager_inner,
                    &active_sounds,
                    &playback_id_clone,
                    &sound_id_clone,
                );
                return;
            }
        };

        let stream_2 = match audio::create_playback_stream(
            device_2,
            audio_data.clone(),
            volume_state.clone(),
            start_frame,
            end_frame,
            lufs_gain,
        ) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create stream 2: {}", e);
                cleanup_early(
                    &manager_inner,
                    &active_sounds,
                    &playback_id_clone,
                    &sound_id_clone,
                );
                return;
            }
        };

        // Streams created successfully - NOW the sound is audible!
        let streams_ready_elapsed = thread_start.elapsed().as_millis();
        info!(
            playback_id = %playback_id_clone,
            sound_id = %sound_id_clone,
            streams_ready_ms = streams_ready_elapsed,
            "Audio streams created and playing"
        );

        // Stop the old playback NOW (seamless transition, no audio gap)
        if let Some(ref old_id) = old_playback_to_stop {
            if let Some(sender) = manager_inner.lock().unwrap().remove(old_id) {
                let _ = sender.send(());
                debug!("Stopped old playback {} (new one ready)", old_id);
            }
        }

        // Transition from Decoding to Playing state
        if !sound_id_clone.is_empty() {
            let mut sounds = active_sounds.lock().unwrap();
            if let Some(state) = sounds.get(&sound_id_clone) {
                // Only update if this is still our playback
                if state.playback_id() == playback_id_clone {
                    sounds.insert(
                        sound_id_clone.clone(),
                        SoundState::Playing {
                            playback_id: playback_id_clone.clone(),
                            started_at: std::time::Instant::now(),
                        },
                    );
                    debug!(
                        "Sound {} now playing (playback {})",
                        sound_id_clone, playback_id_clone
                    );
                }
            }
        }

        // Calculate duration (with trim)
        let total_frames = audio_data.samples.len() / audio_data.channels as usize;
        let actual_start = start_frame.unwrap_or(0);
        let actual_end = end_frame.unwrap_or(total_frames);
        let trimmed_frames = actual_end.saturating_sub(actual_start);

        let duration_secs = trimmed_frames as f64 / audio_data.sample_rate as f64;
        let total_sleep_ms = (duration_secs * 1000.0) as u64;

        // Wait for completion or stop signal, emitting progress events
        let check_interval = Duration::from_millis(10); // 10ms for fast stop response
        let progress_interval = 50u64; // Emit progress every 50ms
        let mut elapsed_ms = 0u64;
        let mut last_progress_ms = 0u64;

        while elapsed_ms < total_sleep_ms {
            // Check for stop signal
            if stop_rx.try_recv().is_ok() {
                break;
            }

            thread::sleep(check_interval);
            elapsed_ms += 10;

            // Emit progress event every 50ms (not every 10ms check)
            if elapsed_ms - last_progress_ms >= progress_interval {
                last_progress_ms = elapsed_ms;
                let progress_pct =
                    ((elapsed_ms as f64 / total_sleep_ms as f64) * 100.0).min(100.0) as u8;
                if let Err(e) = app_handle.emit(
                    "playback-progress",
                    PlaybackProgress {
                        playback_id: playback_id_clone.clone(),
                        elapsed_ms,
                        total_ms: total_sleep_ms,
                        progress_pct,
                    },
                ) {
                    error!("Failed to emit progress event: {}", e);
                }
            }
        }

        // Clean up
        drop(stream_1);
        drop(stream_2);

        let total_duration_ms = thread_start.elapsed().as_millis();
        debug!(
            playback_id = %playback_id_clone,
            sound_id = %sound_id_clone,
            total_duration_ms = total_duration_ms,
            "Playback complete"
        );

        // Emit playback complete event first, so frontend knows it's done.
        // This prevents race conditions where frontend sends stop_playback
        // just before receiving this event.
        if let Err(e) = app_handle.emit("playback-complete", &playback_id_clone) {
            error!("Failed to emit playback complete event: {}", e);
        }

        // Remove from manager last
        manager_inner.lock().unwrap().remove(&playback_id_clone);

        // Remove from active sounds tracking ONLY if this playback is still the current one
        // (prevents race condition when a newer playback has already replaced us)
        if !sound_id_clone.is_empty() {
            let mut sounds = active_sounds.lock().unwrap();
            if let Some(state) = sounds.get(&sound_id_clone) {
                if state.playback_id() == playback_id_clone {
                    sounds.remove(&sound_id_clone);
                }
            }
        }
    });

    let action = if stopped_playback_id.is_some() {
        "restarted"
    } else {
        "started"
    };

    Ok(PlaybackResult {
        playback_id: Some(playback_id),
        action: action.to_string(),
        stopped_playback_id,
    })
}

/// Stops all currently playing audio
#[tauri::command]
pub fn stop_all_audio(manager: State<'_, AudioManager>) -> Result<(), String> {
    manager.stop_all();
    Ok(())
}

/// Stops a specific playback by ID
#[tauri::command]
pub fn stop_playback(playback_id: String, manager: State<'_, AudioManager>) -> Result<(), String> {
    if manager.signal_stop(&playback_id) {
        Ok(())
    } else {
        Err(format!("Playback not found: {}", playback_id))
    }
}

/// Clear the audio cache (forces re-decoding on next play)
#[tauri::command]
pub fn clear_audio_cache(manager: State<'_, AudioManager>) -> Result<(), String> {
    manager.clear_cache();
    Ok(())
}

/// Get audio cache statistics
#[tauri::command]
pub fn get_cache_stats(manager: State<'_, AudioManager>) -> Result<CacheStats, String> {
    Ok(manager.cache_stats())
}

/// Get waveform data for an audio file
#[tauri::command]
pub fn get_waveform(
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

/// Preload audio files into cache (background, non-blocking)
/// Call this when switching categories to ensure sounds are ready
#[tauri::command]
pub fn preload_sounds(file_paths: Vec<String>, manager: State<'_, AudioManager>) {
    let cache = manager.get_cache();

    // Spawn background thread to preload without blocking UI
    thread::spawn(move || {
        for path in file_paths {
            let mut cache_guard = cache.lock().unwrap();
            if cache_guard.get_or_decode(&path).is_ok() {
                debug!("Preloaded: {}", path);
            }
            // Release lock between files to not block playback
            drop(cache_guard);
        }
        debug!("Preload complete");
    });
}
