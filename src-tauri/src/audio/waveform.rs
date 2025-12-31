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

#[cfg(test)]
mod tests {
    use super::*;

    /// Create test AudioData with specified samples
    fn create_test_audio(samples: Vec<f32>, sample_rate: u32, channels: u16) -> AudioData {
        AudioData {
            samples,
            sample_rate,
            channels,
        }
    }

    #[test]
    fn test_generate_peaks_empty_data() {
        let audio = create_test_audio(vec![], 48000, 2);
        let waveform = generate_peaks(&audio, 100);

        assert!(waveform.peaks.is_empty());
        assert_eq!(waveform.duration_ms, 0);
    }

    #[test]
    fn test_generate_peaks_zero_peaks() {
        let audio = create_test_audio(vec![0.5, 0.5], 48000, 2);
        let waveform = generate_peaks(&audio, 0);

        assert!(waveform.peaks.is_empty());
    }

    #[test]
    fn test_generate_peaks_mono() {
        // 48000 samples @ 48kHz = 1 second
        let samples: Vec<f32> = (0..48000).map(|i| i as f32 / 48000.0).collect();
        let audio = create_test_audio(samples, 48000, 1);
        let waveform = generate_peaks(&audio, 10);

        assert_eq!(waveform.peaks.len(), 10);
        assert_eq!(waveform.duration_ms, 1000);
    }

    #[test]
    fn test_generate_peaks_stereo() {
        // 96000 samples (48000 frames * 2 channels) @ 48kHz = 1 second
        let samples: Vec<f32> = (0..96000).map(|i| i as f32 / 96000.0).collect();
        let audio = create_test_audio(samples, 48000, 2);
        let waveform = generate_peaks(&audio, 10);

        assert_eq!(waveform.peaks.len(), 10);
        assert_eq!(waveform.duration_ms, 1000);
    }

    #[test]
    fn test_generate_peaks_normalization() {
        // Create audio with known peak values
        let samples = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];
        let audio = create_test_audio(samples, 48000, 1);
        let waveform = generate_peaks(&audio, 2);

        // All peaks should be normalized to 0.0-1.0
        for peak in &waveform.peaks {
            assert!(*peak >= 0.0 && *peak <= 1.0);
        }
        // The maximum peak should be 1.0 after normalization
        let max_peak = waveform.peaks.iter().cloned().fold(0.0f32, f32::max);
        assert!((max_peak - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_generate_peaks_silent_audio() {
        // All zeros - peaks should all be 0
        let samples = vec![0.0; 1000];
        let audio = create_test_audio(samples, 48000, 1);
        let waveform = generate_peaks(&audio, 10);

        for peak in &waveform.peaks {
            assert_eq!(*peak, 0.0);
        }
    }

    #[test]
    fn test_duration_calculation() {
        // 48000 samples @ 48kHz mono = 1 second = 1000ms
        let audio = create_test_audio(vec![0.0; 48000], 48000, 1);
        let waveform = generate_peaks(&audio, 10);
        assert_eq!(waveform.duration_ms, 1000);

        // 44100 samples @ 44100Hz mono = 1 second = 1000ms
        let audio2 = create_test_audio(vec![0.0; 44100], 44100, 1);
        let waveform2 = generate_peaks(&audio2, 10);
        assert_eq!(waveform2.duration_ms, 1000);

        // 24000 samples @ 48kHz mono = 0.5 second = 500ms
        let audio3 = create_test_audio(vec![0.0; 24000], 48000, 1);
        let waveform3 = generate_peaks(&audio3, 10);
        assert_eq!(waveform3.duration_ms, 500);
    }
}
