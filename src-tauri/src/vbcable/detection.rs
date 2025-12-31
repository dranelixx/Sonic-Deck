//! VB-Cable detection via cpal device enumeration

use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;
use std::thread;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Information about detected VB-Cable devices
#[derive(Debug, Clone, Serialize)]
pub struct VbCableInfo {
    /// Output device name (e.g., "CABLE Input (VB-Audio Virtual Cable)")
    /// This is where apps send audio TO VB-Cable
    pub output_device: String,
    /// Input device name (e.g., "CABLE Output (VB-Audio Virtual Cable)")
    /// This is where apps receive audio FROM VB-Cable
    pub input_device: Option<String>,
}

/// VB-Cable installation status
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum VbCableStatus {
    /// VB-Cable is installed and detected
    Installed { info: VbCableInfo },
    /// VB-Cable is not installed
    NotInstalled,
}

/// Full VB-Cable detection with device info
///
/// Searches for both the output device (CABLE Input) and input device (CABLE Output).
/// Returns None if VB-Cable output device is not found.
pub fn detect_vb_cable() -> Option<VbCableInfo> {
    let host = cpal::default_host();

    let mut output_device = None;
    let mut input_device = None;

    // Find output device (CABLE Input - where apps send audio)
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.to_lowercase().contains("cable input") {
                    debug!("VB-Cable output device found: {}", name);
                    output_device = Some(name);
                    break;
                }
            }
        }
    }

    // Find input device (CABLE Output - where apps receive audio)
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.to_lowercase().contains("cable output") {
                    debug!("VB-Cable input device found: {}", name);
                    input_device = Some(name);
                    break;
                }
            }
        }
    }

    // VB-Cable output device is required, input device is optional
    output_device.map(|out| VbCableInfo {
        output_device: out,
        input_device,
    })
}

const MAX_RETRIES: u32 = 5;
const RETRY_DELAY_MS: u64 = 1000;

/// Wait for VB-Cable to appear in device list (with active retries)
///
/// Use after installation when Windows needs time to register the device.
/// Returns early as soon as device is detected, or None after all retries.
pub fn wait_for_vb_cable() -> Option<VbCableInfo> {
    for attempt in 1..=MAX_RETRIES {
        if let Some(info) = detect_vb_cable() {
            info!("VB-Cable detected on attempt {}/{}", attempt, MAX_RETRIES);
            return Some(info);
        }

        if attempt < MAX_RETRIES {
            debug!(
                "VB-Cable not found (attempt {}/{}), retrying in {}ms...",
                attempt, MAX_RETRIES, RETRY_DELAY_MS
            );
            thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
        }
    }

    warn!("VB-Cable not detected after {} attempts", MAX_RETRIES);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vb_cable_info_serialization() {
        let info = VbCableInfo {
            output_device: "CABLE Input (VB-Audio Virtual Cable)".to_string(),
            input_device: Some("CABLE Output (VB-Audio Virtual Cable)".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("CABLE Input"));
        assert!(json.contains("CABLE Output"));
    }

    #[test]
    fn test_vb_cable_status_serialization() {
        let status = VbCableStatus::NotInstalled;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("notInstalled"));

        let info = VbCableInfo {
            output_device: "CABLE Input".to_string(),
            input_device: None,
        };
        let status = VbCableStatus::Installed { info };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("installed"));
        assert!(json.contains("CABLE Input"));
    }
}
