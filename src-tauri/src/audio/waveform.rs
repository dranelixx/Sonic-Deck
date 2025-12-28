//! Waveform peak generation from audio data
//!
//! Generates amplitude peaks for visualization.

use std::time::Instant;
use tracing::debug;

use super::AudioData;

/// Waveform data for visualization
#[derive(Clone, serde::Serialize)]
pub struct WaveformData {
    /// Peak amplitudes (0.0 to 1.0)
    pub peaks: Vec<f32>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Generate waveform peaks from audio data
pub fn generate_peaks(audio_data: &AudioData, num_peaks: usize) -> WaveformData {
    let start = Instant::now();
    let channels = audio_data.channels as usize;
    let total_frames = audio_data.samples.len() / channels;

    // Calculate duration
    let duration_ms = (total_frames as f64 / audio_data.sample_rate as f64 * 1000.0) as u64;

    if total_frames == 0 || num_peaks == 0 {
        return WaveformData {
            peaks: vec![],
            duration_ms,
        };
    }

    let frames_per_peak = (total_frames / num_peaks).max(1);
    let mut peaks = Vec::with_capacity(num_peaks);

    for peak_idx in 0..num_peaks {
        let start_frame = peak_idx * frames_per_peak;
        let end_frame = ((peak_idx + 1) * frames_per_peak).min(total_frames);

        let mut max_amplitude: f32 = 0.0;

        // Find the maximum absolute amplitude in this segment
        for frame in start_frame..end_frame {
            for ch in 0..channels {
                let sample_idx = frame * channels + ch;
                if sample_idx < audio_data.samples.len() {
                    let amplitude = audio_data.samples[sample_idx].abs();
                    if amplitude > max_amplitude {
                        max_amplitude = amplitude;
                    }
                }
            }
        }

        peaks.push(max_amplitude);
    }

    // Normalize peaks to 0.0-1.0 range
    let max_peak = peaks.iter().cloned().fold(0.0f32, f32::max);
    if max_peak > 0.0 {
        for peak in &mut peaks {
            *peak /= max_peak;
        }
    }

    let generation_time = start.elapsed().as_millis();
    debug!(
        duration_ms = generation_time,
        num_peaks = num_peaks,
        total_frames = total_frames,
        audio_duration_ms = duration_ms,
        "Waveform generation complete"
    );

    WaveformData { peaks, duration_ms }
}
