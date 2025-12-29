//! VB-Cable detection via cpal device enumeration

use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;
use tracing::debug;

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

/// Quick check if VB-Cable is installed
///
/// Returns true if any output device contains "cable input" in its name.
pub fn is_vb_cable_installed() -> bool {
    let host = cpal::default_host();

    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.to_lowercase().contains("cable input") {
                    debug!("VB-Cable detected: {}", name);
                    return true;
                }
            }
        }
    }

    debug!("VB-Cable not detected");
    false
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
