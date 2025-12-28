//! Audio playback stream creation and sample writing
//!
//! Handles cpal stream creation with sample rate conversion using linear interpolation.

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BufferSize, Device, SampleRate, Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, error, info, warn};

use super::{AudioData, AudioError};

/// Preferred buffer size for low-latency playback.
/// 256 samples @ 48kHz = ~5.3ms latency per buffer.
const PREFERRED_BUFFER_SIZE: u32 = 256;

/// Create and start a playback stream on a specific device
pub fn create_playback_stream(
    device: &Device,
    audio_data: Arc<AudioData>,
    volume: Arc<Mutex<f32>>,
    start_frame: Option<usize>,
    end_frame: Option<usize>,
) -> Result<Stream, AudioError> {
    let start = Instant::now();
    let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());

    debug!(device = %device_name, "Creating playback stream");

    let supported_config = device
        .default_output_config()
        .map_err(|e| AudioError::DeviceConfig(e.to_string()))?;

    let output_sample_rate = supported_config.sample_rate().0;
    let channels = supported_config.channels() as usize;
    let sample_format = supported_config.sample_format();

    // Create low-latency stream config with fixed buffer size
    let stream_config = StreamConfig {
        channels: supported_config.channels(),
        sample_rate: SampleRate(output_sample_rate),
        buffer_size: BufferSize::Fixed(PREFERRED_BUFFER_SIZE),
    };

    // Log channel mapping for multi-channel devices
    if channels > audio_data.channels as usize {
        warn!(
            "Device has {} output channels, audio has {} channels - extra channels will be silent",
            channels, audio_data.channels
        );
    }

    // Initialize sample index to start_frame (or 0)
    let start_idx = start_frame.unwrap_or(0) as f64;
    let sample_index = Arc::new(Mutex::new(start_idx));

    // Calculate end frame (or use full length)
    let max_frames = audio_data.samples.len() / audio_data.channels as usize;
    let end_idx = end_frame.unwrap_or(max_frames);
    let end_frame_arc = Arc::new(end_idx);

    // Calculate sample rate ratio for resampling
    let rate_ratio = audio_data.sample_rate as f64 / output_sample_rate as f64;

    // Try to build stream with low-latency config, fallback to default if it fails
    let (stream, used_buffer_size) = build_stream_with_fallback(
        device,
        sample_format,
        &stream_config,
        &supported_config,
        audio_data,
        sample_index,
        volume,
        end_frame_arc,
        channels,
        rate_ratio,
    )?;

    stream
        .play()
        .map_err(|e| AudioError::StreamStart(e.to_string()))?;

    let duration_ms = start.elapsed().as_millis();
    info!(
        device = %device_name,
        sample_rate = output_sample_rate,
        channels = channels,
        buffer_size = ?used_buffer_size,
        sample_format = ?sample_format,
        duration_ms = duration_ms,
        "Playback stream created"
    );

    Ok(stream)
}

/// Buffer size options for fallback strategy
const FALLBACK_BUFFER_SIZES: [u32; 3] = [256, 512, 1024];

/// Build output stream with fallback to larger buffer sizes or default config
#[allow(clippy::too_many_arguments)]
fn build_stream_with_fallback(
    device: &Device,
    sample_format: cpal::SampleFormat,
    low_latency_config: &StreamConfig,
    default_config: &cpal::SupportedStreamConfig,
    audio_data: Arc<AudioData>,
    sample_index: Arc<Mutex<f64>>,
    volume: Arc<Mutex<f32>>,
    end_frame: Arc<usize>,
    channels: usize,
    rate_ratio: f64,
) -> Result<(Stream, String), AudioError> {
    // Try each buffer size in order
    for &buffer_size in &FALLBACK_BUFFER_SIZES {
        let config = StreamConfig {
            channels: low_latency_config.channels,
            sample_rate: low_latency_config.sample_rate,
            buffer_size: BufferSize::Fixed(buffer_size),
        };

        match try_build_stream(
            device,
            sample_format,
            &config,
            audio_data.clone(),
            sample_index.clone(),
            volume.clone(),
            end_frame.clone(),
            channels,
            rate_ratio,
        ) {
            Ok(stream) => {
                if buffer_size != PREFERRED_BUFFER_SIZE {
                    warn!(
                        buffer_size = buffer_size,
                        preferred = PREFERRED_BUFFER_SIZE,
                        "Using fallback buffer size"
                    );
                }
                return Ok((stream, format!("Fixed({})", buffer_size)));
            }
            Err(_) => continue,
        }
    }

    // Final fallback: use default config
    warn!("Fixed buffer sizes failed, using device default");
    let stream = try_build_stream(
        device,
        sample_format,
        &default_config.clone().into(),
        audio_data,
        sample_index,
        volume,
        end_frame,
        channels,
        rate_ratio,
    )?;

    Ok((stream, "Default".to_string()))
}

/// Try to build a stream with the given config
#[allow(clippy::too_many_arguments)]
fn try_build_stream(
    device: &Device,
    sample_format: cpal::SampleFormat,
    config: &StreamConfig,
    audio_data: Arc<AudioData>,
    sample_index: Arc<Mutex<f64>>,
    volume: Arc<Mutex<f32>>,
    end_frame: Arc<usize>,
    channels: usize,
    rate_ratio: f64,
) -> Result<Stream, AudioError> {
    match sample_format {
        cpal::SampleFormat::F32 => device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let vol = *volume.lock().unwrap();
                    write_audio_f32(
                        data,
                        &audio_data,
                        &sample_index,
                        vol,
                        channels,
                        rate_ratio,
                        *end_frame,
                    );
                },
                |err| error!("Stream error: {}", err),
                None,
            )
            .map_err(|e| AudioError::StreamBuild(e.to_string())),
        cpal::SampleFormat::I16 => device
            .build_output_stream(
                config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let vol = *volume.lock().unwrap();
                    write_audio_i16(
                        data,
                        &audio_data,
                        &sample_index,
                        vol,
                        channels,
                        rate_ratio,
                        *end_frame,
                    );
                },
                |err| error!("Stream error: {}", err),
                None,
            )
            .map_err(|e| AudioError::StreamBuild(e.to_string())),
        cpal::SampleFormat::U16 => device
            .build_output_stream(
                config,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    let vol = *volume.lock().unwrap();
                    write_audio_u16(
                        data,
                        &audio_data,
                        &sample_index,
                        vol,
                        channels,
                        rate_ratio,
                        *end_frame,
                    );
                },
                |err| error!("Stream error: {}", err),
                None,
            )
            .map_err(|e| AudioError::StreamBuild(e.to_string())),
        _ => Err(AudioError::UnsupportedFormat),
    }
}

/// Write audio data to f32 output buffer with resampling (linear interpolation)
fn write_audio_f32(
    output: &mut [f32],
    audio_data: &AudioData,
    sample_index: &Arc<Mutex<f64>>,
    volume: f32,
    output_channels: usize,
    rate_ratio: f64,
    end_frame: usize,
) {
    let mut index = sample_index.lock().unwrap();
    let input_channels = audio_data.channels as usize;
    let max_frame = end_frame.min(audio_data.samples.len() / input_channels) as f64;

    // Apply square root volume curve with base attenuation
    // Base multiplier of 0.2 for safe default volume (20% of full amplitude)
    let scaled_volume = volume.sqrt() * 0.2;

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
            // Only map audio to channels that exist in input
            // Extra output channels (e.g., center, LFE, surround in 5.1/7.1) get silence
            // This prevents audio artifacts on multi-channel devices like Razer 7.1 headsets
            if ch >= input_channels {
                *sample = 0.0;
                continue;
            }

            let idx1 = frame_idx * input_channels + ch;
            let idx2 = (frame_idx + 1) * input_channels + ch;

            if idx2 < audio_data.samples.len() {
                // Linear interpolation: value = sample1 + (sample2 - sample1) * frac
                let sample1 = audio_data.samples[idx1];
                let sample2 = audio_data.samples[idx2];
                *sample = (sample1 + (sample2 - sample1) * frac as f32) * scaled_volume;
            } else if idx1 < audio_data.samples.len() {
                *sample = audio_data.samples[idx1] * scaled_volume;
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
    end_frame: usize,
) {
    let mut index = sample_index.lock().unwrap();
    let input_channels = audio_data.channels as usize;
    let max_frame = end_frame.min(audio_data.samples.len() / input_channels) as f64;

    // Apply square root volume curve with base attenuation
    // Base multiplier of 0.2 for safe default volume (20% of full amplitude)
    let scaled_volume = volume.sqrt() * 0.2;

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
            // Only map audio to channels that exist in input
            // Extra output channels (e.g., center, LFE, surround in 5.1/7.1) get silence
            // This prevents audio artifacts on multi-channel devices like Razer 7.1 headsets
            if ch >= input_channels {
                *sample = 0;
                continue;
            }

            let idx1 = frame_idx * input_channels + ch;
            let idx2 = (frame_idx + 1) * input_channels + ch;

            let value = if idx2 < audio_data.samples.len() {
                let sample1 = audio_data.samples[idx1];
                let sample2 = audio_data.samples[idx2];
                (sample1 + (sample2 - sample1) * frac as f32) * scaled_volume
            } else if idx1 < audio_data.samples.len() {
                audio_data.samples[idx1] * scaled_volume
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
    end_frame: usize,
) {
    let mut index = sample_index.lock().unwrap();
    let input_channels = audio_data.channels as usize;
    let max_frame = end_frame.min(audio_data.samples.len() / input_channels) as f64;

    // Apply square root volume curve with base attenuation
    // Base multiplier of 0.2 for safe default volume (20% of full amplitude)
    let scaled_volume = volume.sqrt() * 0.2;

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
            // Only map audio to channels that exist in input
            // Extra output channels (e.g., center, LFE, surround in 5.1/7.1) get silence
            // This prevents audio artifacts on multi-channel devices like Razer 7.1 headsets
            if ch >= input_channels {
                *sample = 32768; // Silence for u16 (mid-point)
                continue;
            }

            let idx1 = frame_idx * input_channels + ch;
            let idx2 = (frame_idx + 1) * input_channels + ch;

            let value = if idx2 < audio_data.samples.len() {
                let sample1 = audio_data.samples[idx1];
                let sample2 = audio_data.samples[idx2];
                (sample1 + (sample2 - sample1) * frac as f32) * scaled_volume
            } else if idx1 < audio_data.samples.len() {
                audio_data.samples[idx1] * scaled_volume
            } else {
                0.0
            };
            *sample = ((value + 1.0) * 32767.5) as u16;
        }

        *index += rate_ratio;
    }
}
