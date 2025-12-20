// SonicDeck - High-performance Desktop Soundboard
// Rust Backend with Dual-Output Audio Routing (cpal-based implementation)

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender};
use std::collections::HashMap;
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream};
use tauri::{Emitter, Manager, State};

// Symphonia imports for audio decoding
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Represents an audio output device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Unique identifier for the device
    pub id: String,
    /// Human-readable name of the device
    pub name: String,
    /// Whether this is the system default device
    pub is_default: bool,
}

/// Application settings for device routing and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Selected monitor output device ID
    pub monitor_device_id: Option<String>,
    /// Selected broadcast output device ID
    pub broadcast_device_id: Option<String>,
    /// Default volume (0.0 - 1.0)
    pub default_volume: f32,
    /// Last used audio file path (for convenience)
    pub last_file_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            monitor_device_id: None,
            broadcast_device_id: None,
            default_volume: 0.5,
            last_file_path: None,
        }
    }
}

/// Holds audio data decoded from file
#[derive(Clone)]
struct AudioData {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
}

/// Manages audio playback state and active streams
pub struct AudioManager {
    /// Stop signals for active playbacks (send () to stop)
    stop_senders: Arc<Mutex<HashMap<String, Sender<()>>>>,
    /// Counter for generating unique playback IDs
    playback_counter: Arc<Mutex<u64>>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            stop_senders: Arc::new(Mutex::new(HashMap::new())),
            playback_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Generate a unique playback ID
    fn next_playback_id(&self) -> String {
        let mut counter = self.playback_counter.lock().unwrap();
        *counter += 1;
        format!("playback_{}", *counter)
    }

    /// Register a stop sender
    fn register_playback(&self, playback_id: String, sender: Sender<()>) {
        let mut senders = self.stop_senders.lock().unwrap();
        senders.insert(playback_id, sender);
    }

    /// Unregister a playback
    #[allow(dead_code)]
    fn unregister_playback(&self, playback_id: &str) {
        let mut senders = self.stop_senders.lock().unwrap();
        senders.remove(playback_id);
    }

    /// Stop all active playbacks
    pub fn stop_all(&self) {
        let mut senders = self.stop_senders.lock().unwrap();
        for (_, sender) in senders.drain() {
            let _ = sender.send(()); // Ignore errors if thread already stopped
        }
    }

    /// Signal a specific playback to stop
    pub fn signal_stop(&self, playback_id: &str) -> bool {
        let mut senders = self.stop_senders.lock().unwrap();
        if let Some(sender) = senders.remove(playback_id) {
            let _ = sender.send(());
            true
        } else {
            false
        }
    }
}

// ============================================================================
// AUDIO DECODING
// ============================================================================

/// Decode an audio file to raw PCM samples
fn decode_audio_file(file_path: &str) -> Result<AudioData, String> {
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open audio file: {}", e))?;
    
    let media_source = MediaSourceStream::new(Box::new(file), Default::default());
    
    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(file_path).extension() {
        hint.with_extension(ext.to_str().unwrap_or(""));
    }
    
    let probed = symphonia::default::get_probe()
        .format(&hint, media_source, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("Failed to probe audio format: {}", e))?;
    
    let mut format = probed.format;
    let track = format.default_track()
        .ok_or_else(|| "No audio tracks found".to_string())?;
    
    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Failed to create decoder: {}", e))?;
    
    let mut samples = Vec::new();
    let mut sample_rate = 48000;
    let mut channels = 2;
    
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(format!("Error reading packet: {}", e)),
        };
        
        if packet.track_id() != track_id {
            continue;
        }
        
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                sample_rate = spec.rate;
                channels = spec.channels.count() as u16;
                
                let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
                sample_buf.copy_interleaved_ref(decoded);
                samples.extend_from_slice(sample_buf.samples());
            }
            Err(SymphoniaError::DecodeError(err)) => {
                eprintln!("Decode error: {}", err);
                continue;
            }
            Err(e) => return Err(format!("Decoding error: {}", e)),
        }
    }
    
    if samples.is_empty() {
        return Err("No audio data decoded".to_string());
    }
    
    Ok(AudioData {
        samples,
        sample_rate,
        channels,
    })
}

// ============================================================================
// AUDIO PLAYBACK
// ============================================================================

/// Create and start a playback stream on a specific device
fn create_playback_stream(
    device: &Device,
    audio_data: Arc<AudioData>,
    volume: Arc<Mutex<f32>>,
) -> Result<Stream, String> {
    let config = device.default_output_config()
        .map_err(|e| format!("Failed to get device config: {}", e))?;
    
    let output_sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let sample_index = Arc::new(Mutex::new(0.0f64)); // Float for smooth resampling
    let sample_index_clone = sample_index.clone();
    let audio_data_clone = audio_data.clone();
    let volume_clone = volume.clone();
    
    // Calculate sample rate ratio
    let rate_ratio = audio_data.sample_rate as f64 / output_sample_rate as f64;
    
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let volume_c = volume_clone.clone();
            device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let vol = *volume_c.lock().unwrap();
                    write_audio_f32(data, &audio_data_clone, &sample_index_clone, vol, channels, rate_ratio);
                },
                |err| eprintln!("Stream error: {}", err),
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let volume_c = volume_clone.clone();
            device.build_output_stream(
                &config.into(),
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let vol = *volume_c.lock().unwrap();
                    write_audio_i16(data, &audio_data_clone, &sample_index_clone, vol, channels, rate_ratio);
                },
                |err| eprintln!("Stream error: {}", err),
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let volume_c = volume_clone.clone();
            device.build_output_stream(
                &config.into(),
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    let vol = *volume_c.lock().unwrap();
                    write_audio_u16(data, &audio_data_clone, &sample_index_clone, vol, channels, rate_ratio);
                },
                |err| eprintln!("Stream error: {}", err),
                None,
            )
        }
        _ => return Err("Unsupported sample format".to_string()),
    }
    .map_err(|e| format!("Failed to build output stream: {}", e))?;
    
    stream.play()
        .map_err(|e| format!("Failed to start stream: {}", e))?;
    
    Ok(stream)
}

/// Write audio data to f32 output buffer with resampling (linear interpolation)
fn write_audio_f32(
    output: &mut [f32],
    audio_data: &AudioData,
    sample_index: &Arc<Mutex<f64>>,
    volume: f32,
    output_channels: usize,
    rate_ratio: f64,
) {
    let mut index = sample_index.lock().unwrap();
    let input_channels = audio_data.channels as usize;
    let max_frame = (audio_data.samples.len() / input_channels) as f64;
    
    for frame in output.chunks_mut(output_channels) {
        if *index >= max_frame - 1.0 {
            // End of audio - silence
            for sample in frame.iter_mut() {
                *sample = 0.0;
            }
            continue;
        }
        
        // Linear interpolation between samples
        let frame_idx = *index as usize;
        let frac = *index - frame_idx as f64; // Fractional part for interpolation
        
        for (ch, sample) in frame.iter_mut().enumerate() {
            let ch_idx = ch % input_channels;
            let idx1 = frame_idx * input_channels + ch_idx;
            let idx2 = (frame_idx + 1) * input_channels + ch_idx;
            
            if idx2 < audio_data.samples.len() {
                // Linear interpolation: value = sample1 + (sample2 - sample1) * frac
                let sample1 = audio_data.samples[idx1];
                let sample2 = audio_data.samples[idx2];
                *sample = (sample1 + (sample2 - sample1) * frac as f32) * volume;
            } else if idx1 < audio_data.samples.len() {
                *sample = audio_data.samples[idx1] * volume;
            } else {
                *sample = 0.0;
            }
        }
        
        *index += rate_ratio;
    }
}

/// Write audio data to i16 output buffer with resampling (linear interpolation)
fn write_audio_i16(
    output: &mut [i16],
    audio_data: &AudioData,
    sample_index: &Arc<Mutex<f64>>,
    volume: f32,
    output_channels: usize,
    rate_ratio: f64,
) {
    let mut index = sample_index.lock().unwrap();
    let input_channels = audio_data.channels as usize;
    let max_frame = (audio_data.samples.len() / input_channels) as f64;
    
    for frame in output.chunks_mut(output_channels) {
        if *index >= max_frame - 1.0 {
            // End of audio - silence
            for sample in frame.iter_mut() {
                *sample = 0;
            }
            continue;
        }
        
        // Linear interpolation between samples
        let frame_idx = *index as usize;
        let frac = *index - frame_idx as f64;
        
        for (ch, sample) in frame.iter_mut().enumerate() {
            let ch_idx = ch % input_channels;
            let idx1 = frame_idx * input_channels + ch_idx;
            let idx2 = (frame_idx + 1) * input_channels + ch_idx;
            
            let value = if idx2 < audio_data.samples.len() {
                let sample1 = audio_data.samples[idx1];
                let sample2 = audio_data.samples[idx2];
                (sample1 + (sample2 - sample1) * frac as f32) * volume
            } else if idx1 < audio_data.samples.len() {
                audio_data.samples[idx1] * volume
            } else {
                0.0
            };
            *sample = (value * 32767.0) as i16;
        }
        
        *index += rate_ratio;
    }
}

/// Write audio data to u16 output buffer with resampling (linear interpolation)
fn write_audio_u16(
    output: &mut [u16],
    audio_data: &AudioData,
    sample_index: &Arc<Mutex<f64>>,
    volume: f32,
    output_channels: usize,
    rate_ratio: f64,
) {
    let mut index = sample_index.lock().unwrap();
    let input_channels = audio_data.channels as usize;
    let max_frame = (audio_data.samples.len() / input_channels) as f64;
    
    for frame in output.chunks_mut(output_channels) {
        if *index >= max_frame - 1.0 {
            // End of audio - silence
            for sample in frame.iter_mut() {
                *sample = 32768;
            }
            continue;
        }
        
        // Linear interpolation between samples
        let frame_idx = *index as usize;
        let frac = *index - frame_idx as f64;
        
        for (ch, sample) in frame.iter_mut().enumerate() {
            let ch_idx = ch % input_channels;
            let idx1 = frame_idx * input_channels + ch_idx;
            let idx2 = (frame_idx + 1) * input_channels + ch_idx;
            
            let value = if idx2 < audio_data.samples.len() {
                let sample1 = audio_data.samples[idx1];
                let sample2 = audio_data.samples[idx2];
                (sample1 + (sample2 - sample1) * frac as f32) * volume
            } else if idx1 < audio_data.samples.len() {
                audio_data.samples[idx1] * volume
            } else {
                0.0
            };
            *sample = ((value + 1.0) * 32767.5) as u16;
        }
        
        *index += rate_ratio;
    }
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Lists all available output audio devices on the system
#[tauri::command]
fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    let default_device = host.default_output_device();
    let default_name = default_device
        .as_ref()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    let output_devices = host.output_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    for (index, device) in output_devices.enumerate() {
        if let Ok(name) = device.name() {
            let device_id = format!("device_{}", index);
            let is_default = name == default_name;

            devices.push(AudioDevice {
                id: device_id,
                name,
                is_default,
            });
        }
    }

    if devices.is_empty() {
        return Err("No audio output devices found".to_string());
    }

    Ok(devices)
}

/// Plays an audio file simultaneously to two different output devices
#[tauri::command]
fn play_dual_output(
    file_path: String,
    device_id_1: String,
    device_id_2: String,
    volume: f32,
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
    let manager_inner = manager.stop_senders.clone();
    
    // Spawn dedicated playback thread (including decoding to avoid blocking UI)
    thread::spawn(move || {
        // Decode audio file in background thread
        let audio_data = match decode_audio_file(&file_path) {
            Ok(data) => Arc::new(data),
            Err(e) => {
                eprintln!("Failed to decode audio: {}", e);
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
                eprintln!("Failed to enumerate devices: {}", e);
                manager_inner.lock().unwrap().remove(&playback_id_clone);
                return;
            }
        };
        
        // Parse device indices
        let device_1_idx = device_id_1.strip_prefix("device_")
            .and_then(|s| s.parse::<usize>().ok());
        let device_2_idx = device_id_2.strip_prefix("device_")
            .and_then(|s| s.parse::<usize>().ok());

        let (Some(idx1), Some(idx2)) = (device_1_idx, device_2_idx) else {
            eprintln!("Invalid device IDs");
            manager_inner.lock().unwrap().remove(&playback_id_clone);
            return;
        };

        let (Some(device_1), Some(device_2)) = (output_devices.get(idx1), output_devices.get(idx2)) else {
            eprintln!("Devices not found");
            manager_inner.lock().unwrap().remove(&playback_id_clone);
            return;
        };
        
        // Create streams with shared volume state
        let stream_1 = match create_playback_stream(device_1, audio_data.clone(), volume_state.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to create stream 1: {}", e);
                manager_inner.lock().unwrap().remove(&playback_id_clone);
                return;
            }
        };

        let stream_2 = match create_playback_stream(device_2, audio_data.clone(), volume_state.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to create stream 2: {}", e);
                manager_inner.lock().unwrap().remove(&playback_id_clone);
                return;
            }
        };
        
        // Calculate duration
        let duration_secs = audio_data.samples.len() as f64 
            / (audio_data.sample_rate as f64 * audio_data.channels as f64);
        let total_sleep_ms = (duration_secs * 1000.0) as u64;
        
        // Wait for completion or stop signal
        let check_interval = Duration::from_millis(100);
        let mut elapsed_ms = 0u64;
        
        while elapsed_ms < total_sleep_ms {
            // Check for stop signal
            if stop_rx.try_recv().is_ok() {
                break;
            }
            
            thread::sleep(check_interval);
            elapsed_ms += 100;
        }
        
        // Clean up
        drop(stream_1);
        drop(stream_2);
        manager_inner.lock().unwrap().remove(&playback_id_clone);
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

// ============================================================================
// SETTINGS MANAGEMENT
// ============================================================================

/// Get the path to the settings file
fn get_settings_path(app_handle: tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    // Ensure directory exists
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    
    Ok(app_data_dir.join("settings.json"))
}

/// Load application settings from disk
#[tauri::command]
fn load_settings(app_handle: tauri::AppHandle) -> Result<AppSettings, String> {
    let settings_path = get_settings_path(app_handle)?;
    
    if !settings_path.exists() {
        // Return default settings if file doesn't exist
        return Ok(AppSettings::default());
    }
    
    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    
    let settings: AppSettings = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;
    
    Ok(settings)
}

/// Save application settings to disk
#[tauri::command]
fn save_settings(
    settings: AppSettings,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let settings_path = get_settings_path(app_handle)?;
    
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    std::fs::write(&settings_path, json)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    
    Ok(())
}

/// Get the settings file path (for debugging/info)
#[tauri::command]
fn get_settings_file_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    let path = get_settings_path(app_handle)?;
    Ok(path.to_string_lossy().to_string())
}

// ============================================================================
// TAURI APP INITIALIZATION
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AudioManager::new())
        .invoke_handler(tauri::generate_handler![
            list_audio_devices,
            play_dual_output,
            stop_all_audio,
            stop_playback,
            load_settings,
            save_settings,
            get_settings_file_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
