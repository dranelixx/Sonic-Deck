//! Error types for audio operations

use std::io;

/// Audio-related errors
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Failed to open audio file: {0}")]
    FileOpen(#[from] io::Error),

    #[error("Failed to probe audio format: {0}")]
    ProbeFormat(String),

    #[error("No audio tracks found in file")]
    NoTracks,

    #[error("Failed to create decoder: {0}")]
    DecoderCreation(String),

    #[error("Error reading audio packet: {0}")]
    PacketRead(String),

    #[error("Decoding error: {0}")]
    Decode(String),

    #[error("No audio data decoded")]
    NoData,

    #[error("Failed to enumerate audio devices: {0}")]
    DeviceEnumeration(String),

    #[error("No audio output devices found")]
    NoDevices,

    #[error("Failed to get device configuration: {0}")]
    DeviceConfig(String),

    #[error("Unsupported sample format")]
    UnsupportedFormat,

    #[error("Failed to build output stream: {0}")]
    StreamBuild(String),

    #[error("Failed to start stream: {0}")]
    StreamStart(String),

    #[error("Invalid device ID: {0}")]
    InvalidDeviceId(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}

/// Convert AudioError to String for Tauri commands
impl From<AudioError> for String {
    fn from(error: AudioError) -> Self {
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    // ========== Display trait tests for all 15 error variants ==========

    #[test]
    fn test_display_file_open() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let err = AudioError::FileOpen(io_err);
        let msg = err.to_string();
        assert!(msg.contains("Failed to open audio file"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_display_probe_format() {
        let err = AudioError::ProbeFormat("unsupported codec".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Failed to probe audio format"));
        assert!(msg.contains("unsupported codec"));
    }

    #[test]
    fn test_display_no_tracks() {
        let err = AudioError::NoTracks;
        let msg = err.to_string();
        assert!(msg.contains("No audio tracks found"));
    }

    #[test]
    fn test_display_decoder_creation() {
        let err = AudioError::DecoderCreation("codec not supported".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Failed to create decoder"));
        assert!(msg.contains("codec not supported"));
    }

    #[test]
    fn test_display_packet_read() {
        let err = AudioError::PacketRead("corrupted packet".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Error reading audio packet"));
        assert!(msg.contains("corrupted packet"));
    }

    #[test]
    fn test_display_decode() {
        let err = AudioError::Decode("decode failure".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Decoding error"));
        assert!(msg.contains("decode failure"));
    }

    #[test]
    fn test_display_no_data() {
        let err = AudioError::NoData;
        let msg = err.to_string();
        assert!(msg.contains("No audio data decoded"));
    }

    #[test]
    fn test_display_device_enumeration() {
        let err = AudioError::DeviceEnumeration("access denied".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Failed to enumerate audio devices"));
        assert!(msg.contains("access denied"));
    }

    #[test]
    fn test_display_no_devices() {
        let err = AudioError::NoDevices;
        let msg = err.to_string();
        assert!(msg.contains("No audio output devices found"));
    }

    #[test]
    fn test_display_device_config() {
        let err = AudioError::DeviceConfig("invalid sample rate".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Failed to get device configuration"));
        assert!(msg.contains("invalid sample rate"));
    }

    #[test]
    fn test_display_unsupported_format() {
        let err = AudioError::UnsupportedFormat;
        let msg = err.to_string();
        assert!(msg.contains("Unsupported sample format"));
    }

    #[test]
    fn test_display_stream_build() {
        let err = AudioError::StreamBuild("buffer size error".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Failed to build output stream"));
        assert!(msg.contains("buffer size error"));
    }

    #[test]
    fn test_display_stream_start() {
        let err = AudioError::StreamStart("device busy".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Failed to start stream"));
        assert!(msg.contains("device busy"));
    }

    #[test]
    fn test_display_invalid_device_id() {
        let err = AudioError::InvalidDeviceId("malformed-id".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Invalid device ID"));
        assert!(msg.contains("malformed-id"));
    }

    #[test]
    fn test_display_device_not_found() {
        let err = AudioError::DeviceNotFound("Speakers (High Definition Audio)".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Device not found"));
        assert!(msg.contains("Speakers (High Definition Audio)"));
    }

    // ========== From<io::Error> conversion test ==========

    #[test]
    fn test_from_io_error() {
        let io_err = IoError::new(ErrorKind::PermissionDenied, "permission denied");
        let audio_err: AudioError = io_err.into();

        match audio_err {
            AudioError::FileOpen(inner) => {
                assert_eq!(inner.kind(), ErrorKind::PermissionDenied);
            }
            _ => panic!("Expected AudioError::FileOpen"),
        }
    }

    // ========== Into<String> conversion test (for Tauri commands) ==========

    #[test]
    fn test_into_string_conversion() {
        let err = AudioError::NoDevices;
        let err_string: String = err.into();
        assert_eq!(err_string, "No audio output devices found");
    }

    #[test]
    fn test_into_string_preserves_inner_message() {
        let err = AudioError::ProbeFormat("test message".to_string());
        let err_string: String = err.into();
        assert!(err_string.contains("test message"));
    }
}
