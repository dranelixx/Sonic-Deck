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
use std::time::Duration;

use cpal::traits::HostTrait;
use tauri::{Emitter, State};
use tracing::error;

use crate::audio::{self, AudioDevice, AudioManager, CacheStats, DeviceId, WaveformData};

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

/// Plays an audio file simultaneously to two different output devices
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn play_dual_output(
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
