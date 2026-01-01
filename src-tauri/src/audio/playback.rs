//! Audio playback stream creation and sample writing
//!
//! Handles cpal stream creation with sample rate conversion using linear interpolation.

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BufferSize, Device, SampleRate, Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, error, info, trace, warn};

use super::{AudioData, AudioError};

/// Preferred buffer size for low-latency playback.
/// 256 samples @ 48kHz = ~5.3ms latency per buffer.
const PREFERRED_BUFFER_SIZE: u32 = 256;

/// Create and start a playback stream on a specific device
///
/// # LUFS Normalization
///
/// When `lufs_gain` is provided (non-1.0), it is applied multiplicatively
/// with the volume. This allows for loudness normalization across sounds.
/// The gain is calculated once at stream creation (not per-sample) for performance.
pub fn create_playback_stream(
    device: &Device,
    audio_data: Arc<AudioData>,
    volume: Arc<Mutex<f32>>,
    start_frame: Option<usize>,
    end_frame: Option<usize>,
    lufs_gain: f32,
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

    // Log if resampling is occurring (quality impact)
    if audio_data.sample_rate != output_sample_rate {
        info!(
            audio_sample_rate = audio_data.sample_rate,
            output_sample_rate = output_sample_rate,
            rate_ratio = format!("{:.4}", rate_ratio),
            "Sample rate conversion active"
        );
    }

    // Log LUFS gain if applied
    if (lufs_gain - 1.0).abs() > 0.001 {
        debug!(
            lufs_gain = format!("{:.3}", lufs_gain),
            lufs_gain_db = format!("{:.1}", 20.0 * lufs_gain.log10()),
            "LUFS normalization active"
        );
    }

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
        lufs_gain,
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
        "Playback stream created and started"
    );

    Ok(stream)
}

/// Buffer size options for fallback strategy
const FALLBACK_BUFFER_SIZES: [u32; 3] = [256, 512, 1024];

/// Build output stream with fallback to larger buffer sizes or default config.
///
/// Attempts to create a low-latency audio stream by trying multiple buffer sizes
/// in sequence: 256 → 512 → 1024 samples. If all fixed buffer sizes fail, falls
/// back to the device's default configuration.
///
/// # Arguments
///
/// * `device` - The audio output device
/// * `sample_format` - Sample format (F32, I16, or U16)
/// * `low_latency_config` - Preferred low-latency stream configuration
/// * `default_config` - Device's default configuration (fallback)
/// * `audio_data` - Decoded audio samples
/// * `sample_index` - Current playback position
/// * `volume` - Playback volume (0.0-1.0)
/// * `end_frame` - End frame for trimmed playback
/// * `channels` - Number of output channels
/// * `rate_ratio` - Sample rate conversion ratio
/// * `lufs_gain` - LUFS normalization gain multiplier (1.0 = no change)
///
/// # Returns
///
/// Returns a tuple of (Stream, buffer_size_description) on success, or AudioError
/// if all attempts fail.
///
/// # Logging
///
/// - Warns if using a fallback buffer size larger than preferred
/// - Warns if falling back to device default configuration
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
    lufs_gain: f32,
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
            lufs_gain,
        ) {
            Ok(stream) => {
                if buffer_size != PREFERRED_BUFFER_SIZE {
                    warn!(
                        buffer_size = buffer_size,
                        preferred = PREFERRED_BUFFER_SIZE,
                        "Using fallback buffer size (preferred size not supported by device)"
                    );
                }
                return Ok((stream, format!("Fixed({})", buffer_size)));
            }
            Err(e) => {
                debug!(
                    buffer_size = buffer_size,
                    error = %e,
                    "Failed to create stream with buffer size, trying next fallback"
                );
                continue;
            }
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
        lufs_gain,
    )?;

    Ok((stream, "Default".to_string()))
}

/// Try to build a stream with the given configuration.
///
/// Attempts to create a cpal output stream with the specified configuration.
/// Handles three sample formats (F32, I16, U16) and sets up audio callbacks
/// with volume control, sample rate conversion, and multi-channel support.
///
/// # Arguments
///
/// * `device` - The audio output device
/// * `sample_format` - Sample format to use (F32, I16, or U16)
/// * `config` - Stream configuration (sample rate, channels, buffer size)
/// * `audio_data` - Decoded audio samples
/// * `sample_index` - Current playback position (shared, mutable)
/// * `volume` - Playback volume (shared, mutable, 0.0-1.0)
/// * `end_frame` - End frame for trimmed playback
/// * `channels` - Number of output channels
/// * `rate_ratio` - Sample rate conversion ratio
/// * `lufs_gain` - LUFS normalization gain multiplier (1.0 = no change)
///
/// # Returns
///
/// Returns the created Stream on success, or AudioError if:
/// - The sample format is unsupported
/// - The stream build fails (device busy, invalid config, etc.)
///
/// # Audio Processing
///
/// The audio callback performs:
/// - Linear interpolation for sample rate conversion
/// - Volume scaling with square root curve
/// - LUFS normalization gain application
/// - Multi-channel mapping (silences extra output channels)
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
    lufs_gain: f32,
) -> Result<Stream, AudioError> {
    trace!(
        sample_format = ?sample_format,
        buffer_size = ?config.buffer_size,
        sample_rate = config.sample_rate.0,
        channels = config.channels,
        "Attempting stream build"
    );

    let stream = match sample_format {
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
                        lufs_gain,
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
                        lufs_gain,
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
                        lufs_gain,
                    );
                },
                |err| error!("Stream error: {}", err),
                None,
            )
            .map_err(|e| AudioError::StreamBuild(e.to_string())),
        _ => return Err(AudioError::UnsupportedFormat),
    }?;

    debug!(sample_format = ?sample_format, "Stream built successfully");
    Ok(stream)
}

/// Write audio data to f32 output buffer with resampling (linear interpolation)
#[allow(clippy::too_many_arguments)]
fn write_audio_f32(
    output: &mut [f32],
    audio_data: &AudioData,
    sample_index: &Arc<Mutex<f64>>,
    volume: f32,
    output_channels: usize,
    rate_ratio: f64,
    end_frame: usize,
    lufs_gain: f32,
) {
    let mut index = sample_index.lock().unwrap();
    let input_channels = audio_data.channels as usize;
    let max_frame = end_frame.min(audio_data.samples.len() / input_channels) as f64;

    // Apply square root volume curve with base attenuation
    // Base multiplier of 0.2 for safe default volume (20% of full amplitude)
    // LUFS gain is applied multiplicatively for loudness normalization
    let scaled_volume = volume.sqrt() * 0.2 * lufs_gain;

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
#[allow(clippy::too_many_arguments)]
fn write_audio_i16(
    output: &mut [i16],
    audio_data: &AudioData,
    sample_index: &Arc<Mutex<f64>>,
    volume: f32,
    output_channels: usize,
    rate_ratio: f64,
    end_frame: usize,
    lufs_gain: f32,
) {
    let mut index = sample_index.lock().unwrap();
    let input_channels = audio_data.channels as usize;
    let max_frame = end_frame.min(audio_data.samples.len() / input_channels) as f64;

    // Apply square root volume curve with base attenuation
    // Base multiplier of 0.2 for safe default volume (20% of full amplitude)
    // LUFS gain is applied multiplicatively for loudness normalization
    let scaled_volume = volume.sqrt() * 0.2 * lufs_gain;

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
#[allow(clippy::too_many_arguments)]
fn write_audio_u16(
    output: &mut [u16],
    audio_data: &AudioData,
    sample_index: &Arc<Mutex<f64>>,
    volume: f32,
    output_channels: usize,
    rate_ratio: f64,
    end_frame: usize,
    lufs_gain: f32,
) {
    let mut index = sample_index.lock().unwrap();
    let input_channels = audio_data.channels as usize;
    let max_frame = end_frame.min(audio_data.samples.len() / input_channels) as f64;

    // Apply square root volume curve with base attenuation
    // Base multiplier of 0.2 for safe default volume (20% of full amplitude)
    // LUFS gain is applied multiplicatively for loudness normalization
    let scaled_volume = volume.sqrt() * 0.2 * lufs_gain;

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

/// Calculate scaled volume with square root curve and base attenuation.
///
/// Applies a square root curve for more natural volume perception,
/// with a 0.2 base multiplier for safe default volume (20% of full amplitude).
///
/// # Arguments
/// * `volume` - Input volume from 0.0 to 1.0
///
/// # Returns
/// Scaled volume value (0.0 to 0.2 range)
#[inline]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn calculate_scaled_volume(volume: f32) -> f32 {
    volume.sqrt() * 0.2
}

/// Convert decibels to linear gain multiplier.
///
/// Examples:
/// - 0 dB → 1.0 (no change)
/// - +6 dB → 2.0 (double amplitude)
/// - -6 dB → 0.5 (half amplitude)
/// - +20 dB → 10.0
#[inline]
pub(crate) fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Calculate gain adjustment for LUFS normalization.
///
/// Returns linear gain multiplier to reach target loudness.
/// Gain is clamped to +/- 12 dB to prevent extreme adjustments.
///
/// # Arguments
/// * `sound_lufs` - Measured LUFS of the sound (None = no adjustment)
/// * `target_lufs` - Target loudness in LUFS (e.g., -14.0)
/// * `enabled` - Whether normalization is enabled
///
/// # Returns
/// Linear gain multiplier (1.0 = no change)
#[inline]
pub fn calculate_lufs_gain(sound_lufs: Option<f32>, target_lufs: f32, enabled: bool) -> f32 {
    if !enabled {
        return 1.0;
    }

    match sound_lufs {
        Some(lufs) => {
            // Difference in LUFS = difference in dB
            let diff_db = target_lufs - lufs;

            // Clamp to reasonable range to prevent extreme gain
            // +12 dB ≈ 4x amplification, -12 dB ≈ 0.25x
            let clamped_db = diff_db.clamp(-12.0, 12.0);

            db_to_linear(clamped_db)
        }
        None => 1.0, // No LUFS data available, no adjustment
    }
}

/// Linear interpolation between two samples.
///
/// # Arguments
/// * `sample1` - First sample value
/// * `sample2` - Second sample value
/// * `frac` - Interpolation fraction (0.0 = sample1, 1.0 = sample2)
///
/// # Returns
/// Interpolated sample value
#[inline]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lerp_sample(sample1: f32, sample2: f32, frac: f32) -> f32 {
    sample1 + (sample2 - sample1) * frac
}

#[cfg(test)]
mod tests {
    use super::*;

    // Volume curve tests
    #[test]
    fn test_volume_curve_zero() {
        let result = calculate_scaled_volume(0.0);
        assert!((result - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_volume_curve_full() {
        let result = calculate_scaled_volume(1.0);
        assert!((result - 0.2).abs() < 0.0001);
    }

    #[test]
    fn test_volume_curve_half() {
        // sqrt(0.5) * 0.2 ≈ 0.1414
        let result = calculate_scaled_volume(0.5);
        let expected = 0.5_f32.sqrt() * 0.2;
        assert!((result - expected).abs() < 0.0001);
    }

    #[test]
    fn test_volume_curve_quarter() {
        // sqrt(0.25) * 0.2 = 0.5 * 0.2 = 0.1
        let result = calculate_scaled_volume(0.25);
        assert!((result - 0.1).abs() < 0.0001);
    }

    #[test]
    fn test_volume_curve_monotonic() {
        // Volume curve should be monotonically increasing
        let mut prev = 0.0;
        for i in 0..=100 {
            let vol = i as f32 / 100.0;
            let result = calculate_scaled_volume(vol);
            assert!(result >= prev);
            prev = result;
        }
    }

    // Linear interpolation tests
    #[test]
    fn test_lerp_sample_start() {
        let result = lerp_sample(1.0, 2.0, 0.0);
        assert!((result - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_lerp_sample_end() {
        let result = lerp_sample(1.0, 2.0, 1.0);
        assert!((result - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_lerp_sample_middle() {
        let result = lerp_sample(1.0, 3.0, 0.5);
        assert!((result - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_lerp_sample_quarter() {
        let result = lerp_sample(0.0, 4.0, 0.25);
        assert!((result - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_lerp_sample_negative() {
        let result = lerp_sample(-1.0, 1.0, 0.5);
        assert!((result - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_lerp_sample_same_values() {
        let result = lerp_sample(5.0, 5.0, 0.7);
        assert!((result - 5.0).abs() < 0.0001);
    }

    // ========== Volume Engine V2 tests ==========

    #[test]
    fn test_db_to_linear_zero() {
        let gain = db_to_linear(0.0);
        assert!(
            (gain - 1.0).abs() < 0.0001,
            "0 dB should be 1.0, got {}",
            gain
        );
    }

    #[test]
    fn test_db_to_linear_positive() {
        let gain = db_to_linear(6.0);
        assert!(
            (gain - 2.0).abs() < 0.01,
            "+6 dB should be ~2.0, got {}",
            gain
        );

        let gain = db_to_linear(20.0);
        assert!(
            (gain - 10.0).abs() < 0.01,
            "+20 dB should be ~10.0, got {}",
            gain
        );
    }

    #[test]
    fn test_db_to_linear_negative() {
        let gain = db_to_linear(-6.0);
        assert!(
            (gain - 0.5).abs() < 0.01,
            "-6 dB should be ~0.5, got {}",
            gain
        );

        let gain = db_to_linear(-20.0);
        assert!(
            (gain - 0.1).abs() < 0.01,
            "-20 dB should be ~0.1, got {}",
            gain
        );
    }

    #[test]
    fn test_lufs_gain_disabled() {
        let gain = calculate_lufs_gain(Some(-20.0), -14.0, false);
        assert!(
            (gain - 1.0).abs() < 0.0001,
            "Disabled should return 1.0, got {}",
            gain
        );
    }

    #[test]
    fn test_lufs_gain_no_data() {
        let gain = calculate_lufs_gain(None, -14.0, true);
        assert!(
            (gain - 1.0).abs() < 0.0001,
            "No LUFS data should return 1.0, got {}",
            gain
        );
    }

    #[test]
    fn test_lufs_gain_quiet_sound() {
        // Sound at -20 LUFS, target -14 LUFS -> boost by 6 dB -> gain ~2.0
        let gain = calculate_lufs_gain(Some(-20.0), -14.0, true);
        let expected = db_to_linear(6.0);
        assert!(
            (gain - expected).abs() < 0.01,
            "Expected {}, got {}",
            expected,
            gain
        );
    }

    #[test]
    fn test_lufs_gain_loud_sound() {
        // Sound at -10 LUFS, target -14 LUFS -> reduce by 4 dB -> gain ~0.63
        let gain = calculate_lufs_gain(Some(-10.0), -14.0, true);
        let expected = db_to_linear(-4.0);
        assert!(
            (gain - expected).abs() < 0.01,
            "Expected {}, got {}",
            expected,
            gain
        );
    }

    #[test]
    fn test_lufs_gain_clamped_boost() {
        // Sound at -40 LUFS, target -14 LUFS -> would be +26 dB, clamped to +12 dB
        let gain = calculate_lufs_gain(Some(-40.0), -14.0, true);
        let expected = db_to_linear(12.0); // ~3.98
        assert!(
            (gain - expected).abs() < 0.01,
            "Should clamp to +12 dB ({:.2}), got {:.2}",
            expected,
            gain
        );
    }

    #[test]
    fn test_lufs_gain_clamped_cut() {
        // Sound at 0 LUFS, target -14 LUFS -> would be -14 dB, clamped to -12 dB
        let gain = calculate_lufs_gain(Some(0.0), -14.0, true);
        let expected = db_to_linear(-12.0); // ~0.25
        assert!(
            (gain - expected).abs() < 0.01,
            "Should clamp to -12 dB ({:.2}), got {:.2}",
            expected,
            gain
        );
    }

    #[test]
    fn test_lufs_gain_at_target() {
        // Sound already at target -> no adjustment
        let gain = calculate_lufs_gain(Some(-14.0), -14.0, true);
        assert!(
            (gain - 1.0).abs() < 0.0001,
            "At target should return 1.0, got {}",
            gain
        );
    }
}
