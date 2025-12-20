//! Audio decoding using Symphonia
//!
//! Supports MP3, WAV, OGG Vorbis, and MP4/M4A formats.

use std::fs::File;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tracing::warn;

use super::{AudioData, AudioError};

/// Decode an audio file to raw PCM samples
pub fn decode_audio_file(file_path: &str) -> Result<AudioData, AudioError> {
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

    Ok(AudioData {
        samples,
        sample_rate,
        channels,
    })
}
