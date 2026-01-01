//! Audio module for Sonic Deck
//!
//! Provides dual-output audio routing with cpal-based playback and caching.

mod cache;
mod decode;
mod device;
mod error;
mod manager;
mod playback;
mod waveform;

pub use cache::CacheStats;
pub use device::enumerate_devices;
pub use error::AudioError;
pub use manager::{AudioManager, SoundState};
pub use playback::{calculate_lufs_gain, create_playback_stream};
pub use waveform::{generate_peaks, WaveformData};

use serde::{Deserialize, Serialize};

/// Type-safe device identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceId(String);

impl DeviceId {
    /// Create a new device ID from an index
    pub fn from_index(index: usize) -> Self {
        Self(format!("device_{}", index))
    }

    /// Parse the device index from the ID
    pub fn index(&self) -> Result<usize, AudioError> {
        self.0
            .strip_prefix("device_")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| AudioError::InvalidDeviceId(self.0.clone()))
    }

    /// Get the raw string ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents an audio output device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Unique identifier for the device
    pub id: DeviceId,
    /// Human-readable name of the device
    pub name: String,
    /// Whether this is the system default device
    pub is_default: bool,
}

/// Holds audio data decoded from file
#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    /// Integrated loudness (LUFS), None if not yet calculated
    pub lufs: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_id_from_index() {
        let id = DeviceId::from_index(0);
        assert_eq!(id.as_str(), "device_0");

        let id2 = DeviceId::from_index(42);
        assert_eq!(id2.as_str(), "device_42");
    }

    #[test]
    fn test_device_id_index_parsing() {
        let id = DeviceId::from_index(5);
        let index = id.index().unwrap();
        assert_eq!(index, 5);
    }

    #[test]
    fn test_device_id_index_parsing_large() {
        let id = DeviceId::from_index(999);
        let index = id.index().unwrap();
        assert_eq!(index, 999);
    }

    #[test]
    fn test_device_id_invalid_format() {
        let id = DeviceId("invalid".to_string());
        let result = id.index();
        assert!(result.is_err());

        if let Err(AudioError::InvalidDeviceId(s)) = result {
            assert_eq!(s, "invalid");
        } else {
            panic!("Expected InvalidDeviceId error");
        }
    }

    #[test]
    fn test_device_id_invalid_prefix() {
        let id = DeviceId("audio_0".to_string());
        assert!(id.index().is_err());
    }

    #[test]
    fn test_device_id_invalid_number() {
        let id = DeviceId("device_abc".to_string());
        assert!(id.index().is_err());
    }

    #[test]
    fn test_device_id_display() {
        let id = DeviceId::from_index(7);
        assert_eq!(format!("{}", id), "device_7");
    }

    #[test]
    fn test_device_id_equality() {
        let id1 = DeviceId::from_index(3);
        let id2 = DeviceId::from_index(3);
        let id3 = DeviceId::from_index(4);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_device_id_clone() {
        let id1 = DeviceId::from_index(5);
        let id2 = id1.clone();
        assert_eq!(id1, id2);
    }
}
