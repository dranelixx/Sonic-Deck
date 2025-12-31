//! Audio decoding using Symphonia
//!
//! Supports MP3, WAV, OGG Vorbis, and MP4/M4A formats.

use std::fs::File;
use std::time::Instant;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tracing::{debug, warn};

use super::{AudioData, AudioError};

/// Decode an audio file to raw PCM samples
pub fn decode_audio_file(file_path: &str) -> Result<AudioData, AudioError> {
    let start = Instant::now();
    debug!(file_path = %file_path, "Starting audio decode");

    let file = File::open(file_path)?;

    let media_source = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(file_path).extension() {
        hint.with_extension(ext.to_str().unwrap_or(""));
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            media_source,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| AudioError::ProbeFormat(e.to_string()))?;

    let mut format = probed.format;
    let track = format.default_track().ok_or(AudioError::NoTracks)?;

    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| AudioError::DecoderCreation(e.to_string()))?;

    let mut samples = Vec::new();
    let mut sample_rate = 48000;
    let mut channels = 2;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break
            }
            Err(e) => return Err(AudioError::PacketRead(e.to_string())),
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
                warn!("Decode error (continuing): {}", err);
                continue;
            }
            Err(e) => return Err(AudioError::Decode(e.to_string())),
        }
    }

    if samples.is_empty() {
        return Err(AudioError::NoData);
    }

    let duration_ms = start.elapsed().as_millis();
    let duration_secs = samples.len() as f64 / (sample_rate as f64 * channels as f64);
    debug!(
        file_path = %file_path,
        duration_ms = duration_ms,
        sample_count = samples.len(),
        sample_rate = sample_rate,
        channels = channels,
        audio_duration_secs = format!("{:.2}", duration_secs),
        "Audio decode complete"
    );

    Ok(AudioData {
        samples,
        sample_rate,
        channels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_fixture_path(filename: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(filename)
    }

    // ========== Error handling tests ==========

    #[test]
    fn test_decode_nonexistent_file() {
        let result = decode_audio_file("/nonexistent/path/audio.mp3");
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            AudioError::FileOpen(_) => {}
            _ => panic!("Expected AudioError::FileOpen, got {:?}", err),
        }
    }

    #[test]
    fn test_decode_invalid_file_extension() {
        // Try to decode a file with unsupported extension
        let result = decode_audio_file("/some/path/file.xyz");
        assert!(result.is_err());
    }

    // ========== MP3 format tests ==========

    #[test]
    fn test_decode_mp3_fixture() {
        let path = get_fixture_path("test_mono.mp3");
        let result = decode_audio_file(path.to_str().unwrap());

        assert!(result.is_ok(), "Failed to decode MP3: {:?}", result.err());

        let audio = result.unwrap();
        assert!(!audio.samples.is_empty(), "MP3 should have samples");
        assert_eq!(audio.sample_rate, 44100, "MP3 should be 44.1kHz");
        assert_eq!(audio.channels, 1, "MP3 should be mono");
    }

    #[test]
    fn test_decode_mp3_sample_count() {
        let path = get_fixture_path("test_mono.mp3");
        let audio = decode_audio_file(path.to_str().unwrap()).unwrap();

        // 1 second at 44.1kHz mono = ~44100 samples (with some tolerance for codec)
        let expected_samples = 44100;
        let tolerance = 5000; // Allow some variance due to encoder padding

        assert!(
            (audio.samples.len() as i32 - expected_samples as i32).abs() < tolerance,
            "Expected ~{} samples, got {}",
            expected_samples,
            audio.samples.len()
        );
    }

    // ========== OGG/Vorbis format tests ==========

    #[test]
    fn test_decode_ogg_fixture() {
        let path = get_fixture_path("test_stereo.ogg");
        let result = decode_audio_file(path.to_str().unwrap());

        assert!(result.is_ok(), "Failed to decode OGG: {:?}", result.err());

        let audio = result.unwrap();
        assert!(!audio.samples.is_empty(), "OGG should have samples");
        assert_eq!(audio.sample_rate, 48000, "OGG should be 48kHz");
        assert_eq!(audio.channels, 2, "OGG should be stereo");
    }

    #[test]
    fn test_decode_ogg_sample_count() {
        let path = get_fixture_path("test_stereo.ogg");
        let audio = decode_audio_file(path.to_str().unwrap()).unwrap();

        // 1 second at 48kHz stereo = ~96000 samples (interleaved)
        let expected_samples = 96000;
        let tolerance = 10000;

        assert!(
            (audio.samples.len() as i32 - expected_samples as i32).abs() < tolerance,
            "Expected ~{} samples, got {}",
            expected_samples,
            audio.samples.len()
        );
    }

    // ========== M4A/AAC format tests ==========

    #[test]
    fn test_decode_m4a_fixture() {
        let path = get_fixture_path("test_stereo.m4a");
        let result = decode_audio_file(path.to_str().unwrap());

        assert!(result.is_ok(), "Failed to decode M4A: {:?}", result.err());

        let audio = result.unwrap();
        assert!(!audio.samples.is_empty(), "M4A should have samples");
        assert_eq!(audio.sample_rate, 48000, "M4A should be 48kHz");
        assert_eq!(audio.channels, 2, "M4A should be stereo");
    }

    #[test]
    fn test_decode_m4a_sample_count() {
        let path = get_fixture_path("test_stereo.m4a");
        let audio = decode_audio_file(path.to_str().unwrap()).unwrap();

        // 1 second at 48kHz stereo = ~96000 samples (interleaved)
        let expected_samples = 96000;
        let tolerance = 10000;

        assert!(
            (audio.samples.len() as i32 - expected_samples as i32).abs() < tolerance,
            "Expected ~{} samples, got {}",
            expected_samples,
            audio.samples.len()
        );
    }

    // ========== Sample data validation tests ==========

    #[test]
    fn test_decoded_samples_are_normalized() {
        let path = get_fixture_path("test_mono.mp3");
        let audio = decode_audio_file(path.to_str().unwrap()).unwrap();

        // All samples should be in valid f32 range [-1.0, 1.0] (or slightly beyond for some codecs)
        for sample in &audio.samples {
            assert!(
                sample.is_finite(),
                "Sample should be finite, got {}",
                sample
            );
            assert!(
                *sample >= -2.0 && *sample <= 2.0,
                "Sample {} outside reasonable range",
                sample
            );
        }
    }

    #[test]
    fn test_all_formats_produce_consistent_data() {
        // Verify all supported formats produce AudioData with valid fields
        let formats = vec![
            ("test_mono.mp3", 44100, 1),
            ("test_stereo.ogg", 48000, 2),
            ("test_stereo.m4a", 48000, 2),
        ];

        for (filename, expected_rate, expected_channels) in formats {
            let path = get_fixture_path(filename);
            let audio = decode_audio_file(path.to_str().unwrap())
                .unwrap_or_else(|e| panic!("Failed to decode {}: {:?}", filename, e));

            assert_eq!(
                audio.sample_rate, expected_rate,
                "{} has wrong sample rate",
                filename
            );
            assert_eq!(
                audio.channels, expected_channels,
                "{} has wrong channel count",
                filename
            );
            assert!(!audio.samples.is_empty(), "{} has no samples", filename);
        }
    }
}
