//! Windows default audio device save/restore functionality
//!
//! Uses the com-policy-config crate and windows crate for COM operations.
//! This is needed because VB-Cable installation changes the Windows default audio device.
//!
//! VB-Cable changes 4 default settings:
//! - Render (output) Console: Main playback device
//! - Render (output) Communications: Device for calls/voice chat
//! - Capture (input) Console: Main recording device
//! - Capture (input) Communications: Device for calls/voice chat

use com_policy_config::{IPolicyConfig, PolicyConfigClient};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use windows::Win32::Media::Audio::{
    eCapture, eCommunications, eConsole, eRender, EDataFlow, ERole, IMMDeviceEnumerator,
    MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};

/// COM error: already initialized with different threading mode (safe to ignore)
const RPC_E_CHANGED_MODE: i32 = 0x80010106u32 as i32;

/// Saved default device settings (all 4 combinations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedDefaults {
    pub render_console: Option<String>,
    pub render_communications: Option<String>,
    pub capture_console: Option<String>,
    pub capture_communications: Option<String>,
}

/// Result of restoring default devices - shows status per device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    /// Number of devices successfully restored
    pub restored_count: u8,
    /// Number of devices that failed to restore
    pub failed_count: u8,
    /// List of failed device names with error messages
    pub failures: Vec<RestoreFailure>,
}

/// Details about a failed device restore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreFailure {
    /// Human-readable device role (e.g., "Playback Device", "Communications Microphone")
    pub device_role: String,
    /// Error message
    pub error: String,
}

/// Manager for saving and restoring the Windows default audio devices
#[derive(Debug, Clone)]
pub struct DefaultDeviceManager {
    saved_device_id: Option<String>,
}

impl DefaultDeviceManager {
    /// Save the current default output audio device (legacy - single device)
    ///
    /// Call this before VB-Cable installation to preserve the user's original default device.
    pub fn save_current_default() -> Result<Self, String> {
        let device_id = unsafe { get_default_device_id(eRender, eConsole) }?;

        info!("Saved current default audio device: {}", device_id);

        Ok(Self {
            saved_device_id: Some(device_id),
        })
    }

    /// Get the saved device ID
    pub fn get_saved_device_id(&self) -> Option<String> {
        self.saved_device_id.clone()
    }

    /// Restore a specific device as the default (static method)
    ///
    /// Used when the device ID is stored externally (e.g., in frontend state).
    pub fn restore_device(device_id: &str) -> Result<(), String> {
        unsafe { set_default_device(device_id, eRender, eConsole) }
    }

    /// Save ALL default device settings (all 4 combinations)
    pub fn save_all_defaults() -> Result<SavedDefaults, String> {
        info!("Saving all default audio devices...");

        let render_console = unsafe { get_default_device_id(eRender, eConsole) }.ok();
        let render_communications = unsafe { get_default_device_id(eRender, eCommunications) }.ok();
        let capture_console = unsafe { get_default_device_id(eCapture, eConsole) }.ok();
        let capture_communications =
            unsafe { get_default_device_id(eCapture, eCommunications) }.ok();

        debug!(
            "Saved defaults - render_console: {:?}, render_comm: {:?}, capture_console: {:?}, capture_comm: {:?}",
            render_console, render_communications, capture_console, capture_communications
        );

        Ok(SavedDefaults {
            render_console,
            render_communications,
            capture_console,
            capture_communications,
        })
    }

    /// Restore ALL default device settings
    ///
    /// Returns a structured result showing which devices were restored successfully
    /// and which failed, allowing the frontend to display specific guidance.
    pub fn restore_all_defaults(saved: &SavedDefaults) -> RestoreResult {
        info!("Restoring all default audio devices...");

        let mut restored_count: u8 = 0;
        let mut failures = Vec::new();

        if let Some(ref id) = saved.render_console {
            if let Err(e) = unsafe { set_default_device(id, eRender, eConsole) } {
                error!("Failed to restore Playback Device: {}", e);
                failures.push(RestoreFailure {
                    device_role: "Playback Device".to_string(),
                    error: e,
                });
            } else {
                restored_count += 1;
            }
        }

        if let Some(ref id) = saved.render_communications {
            if let Err(e) = unsafe { set_default_device(id, eRender, eCommunications) } {
                error!("Failed to restore Communications Playback: {}", e);
                failures.push(RestoreFailure {
                    device_role: "Communications Playback".to_string(),
                    error: e,
                });
            } else {
                restored_count += 1;
            }
        }

        if let Some(ref id) = saved.capture_console {
            if let Err(e) = unsafe { set_default_device(id, eCapture, eConsole) } {
                error!("Failed to restore Recording Device: {}", e);
                failures.push(RestoreFailure {
                    device_role: "Recording Device".to_string(),
                    error: e,
                });
            } else {
                restored_count += 1;
            }
        }

        if let Some(ref id) = saved.capture_communications {
            if let Err(e) = unsafe { set_default_device(id, eCapture, eCommunications) } {
                error!("Failed to restore Communications Microphone: {}", e);
                failures.push(RestoreFailure {
                    device_role: "Communications Microphone".to_string(),
                    error: e,
                });
            } else {
                restored_count += 1;
            }
        }

        let failed_count = failures.len() as u8;

        if failures.is_empty() {
            info!(
                "All {} default audio devices restored successfully",
                restored_count
            );
        } else {
            warn!(
                "Restored {}/{} devices. Failed: {:?}",
                restored_count,
                restored_count + failed_count,
                failures.iter().map(|f| &f.device_role).collect::<Vec<_>>()
            );
        }

        RestoreResult {
            restored_count,
            failed_count,
            failures,
        }
    }
}

/// Get the current default device ID for a specific flow and role
///
/// # Safety
/// Uses COM APIs which require proper initialization/cleanup.
unsafe fn get_default_device_id(flow: EDataFlow, role: ERole) -> Result<String, String> {
    // Initialize COM - handle case where it's already initialized by Tauri
    let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
    // RPC_E_CHANGED_MODE (0x80010106) means COM is already initialized with different mode
    // This is fine - we can still use COM, just don't uninitialize it
    let we_initialized_com = hr.is_ok();
    if hr.is_err() && hr != windows::core::HRESULT(RPC_E_CHANGED_MODE) {
        error!("COM initialization failed: {:?}", hr);
        return Err(format!("Failed to initialize COM: {:?}", hr));
    }

    let result = (|| -> Result<String, String> {
        // Create device enumerator
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| {
                error!("Failed to create device enumerator: {:?}", e);
                format!("Failed to access audio devices: {}", e)
            })?;

        // Get default device for specified flow and role
        let device = enumerator
            .GetDefaultAudioEndpoint(flow, role)
            .map_err(|e| {
                // This can fail if no device is set for this role - that's OK
                debug!(
                    "No default device for flow {:?} role {:?}: {:?}",
                    flow, role, e
                );
                format!("No default device: {}", e)
            })?;

        // Get device ID
        let device_id_pwstr = device.GetId().map_err(|e| {
            error!("Failed to get device ID: {:?}", e);
            format!("Failed to get device ID: {}", e)
        })?;

        let device_id = device_id_pwstr.to_string().map_err(|e| {
            error!("Failed to convert device ID to string: {:?}", e);
            format!("Failed to read device ID: {}", e)
        })?;

        debug!(
            "Default device for flow {:?} role {:?}: {}",
            flow, role, device_id
        );
        Ok(device_id)
    })();

    // Only uninitialize COM if we initialized it
    if we_initialized_com {
        CoUninitialize();
    }

    result
}

/// Set a device as the default for a specific role
///
/// # Safety
/// Uses COM APIs which require proper initialization/cleanup.
unsafe fn set_default_device(device_id: &str, _flow: EDataFlow, role: ERole) -> Result<(), String> {
    // Initialize COM - handle case where it's already initialized by Tauri
    let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
    // RPC_E_CHANGED_MODE (0x80010106) means COM is already initialized with different mode
    let we_initialized_com = hr.is_ok();
    if hr.is_err() && hr != windows::core::HRESULT(RPC_E_CHANGED_MODE) {
        error!("COM initialization failed: {:?}", hr);
        return Err(format!("Failed to initialize COM: {:?}", hr));
    }

    let result = (|| -> Result<(), String> {
        // Create policy config instance
        let policy_config: IPolicyConfig = CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)
            .map_err(|e| {
                error!("Failed to create policy config: {:?}", e);
                format!("Failed to access audio policy: {}", e)
            })?;

        // Convert device ID to PCWSTR
        let device_id_wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
        let device_id_pcwstr = windows::core::PCWSTR::from_raw(device_id_wide.as_ptr());

        // Set as default for specified role
        policy_config
            .SetDefaultEndpoint(device_id_pcwstr, role)
            .map_err(|e| {
                error!(
                    "Failed to set default endpoint for role {:?}: {:?}",
                    role, e
                );
                format!("Failed to set default audio device: {}", e)
            })?;

        debug!("Set default device for role {:?}: {}", role, device_id);
        Ok(())
    })();

    // Only uninitialize COM if we initialized it
    if we_initialized_com {
        CoUninitialize();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_device_manager_creation() {
        // Test that we can create a manager with None
        let manager = DefaultDeviceManager {
            saved_device_id: None,
        };
        assert!(manager.get_saved_device_id().is_none());

        // Test with a device ID
        let manager = DefaultDeviceManager {
            saved_device_id: Some("test-device-id".to_string()),
        };
        assert_eq!(
            manager.get_saved_device_id(),
            Some("test-device-id".to_string())
        );
    }

    #[test]
    fn test_default_device_manager_clone() {
        let manager = DefaultDeviceManager {
            saved_device_id: Some("device-123".to_string()),
        };
        let cloned = manager.clone();
        assert_eq!(manager.get_saved_device_id(), cloned.get_saved_device_id());
    }

    #[test]
    fn test_saved_defaults_serialization() {
        let defaults = SavedDefaults {
            render_console: Some("render-console-id".to_string()),
            render_communications: Some("render-comm-id".to_string()),
            capture_console: Some("capture-console-id".to_string()),
            capture_communications: Some("capture-comm-id".to_string()),
        };

        let json = serde_json::to_string(&defaults).expect("Serialization failed");
        let deserialized: SavedDefaults =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(defaults.render_console, deserialized.render_console);
        assert_eq!(
            defaults.render_communications,
            deserialized.render_communications
        );
        assert_eq!(defaults.capture_console, deserialized.capture_console);
        assert_eq!(
            defaults.capture_communications,
            deserialized.capture_communications
        );
    }

    #[test]
    fn test_saved_defaults_partial() {
        // Test with only some fields set (common case)
        let defaults = SavedDefaults {
            render_console: Some("main-output".to_string()),
            render_communications: None,
            capture_console: None,
            capture_communications: None,
        };

        let json = serde_json::to_string(&defaults).expect("Serialization failed");
        assert!(json.contains("render_console"));
        assert!(json.contains("main-output"));

        let deserialized: SavedDefaults =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.render_console, Some("main-output".to_string()));
        assert!(deserialized.render_communications.is_none());
    }

    #[test]
    fn test_saved_defaults_all_none() {
        let defaults = SavedDefaults {
            render_console: None,
            render_communications: None,
            capture_console: None,
            capture_communications: None,
        };

        let json = serde_json::to_string(&defaults).expect("Serialization failed");
        let deserialized: SavedDefaults =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert!(deserialized.render_console.is_none());
        assert!(deserialized.render_communications.is_none());
        assert!(deserialized.capture_console.is_none());
        assert!(deserialized.capture_communications.is_none());
    }

    #[test]
    fn test_saved_defaults_clone() {
        let defaults = SavedDefaults {
            render_console: Some("device-1".to_string()),
            render_communications: Some("device-2".to_string()),
            capture_console: None,
            capture_communications: None,
        };

        let cloned = defaults.clone();
        assert_eq!(defaults.render_console, cloned.render_console);
        assert_eq!(defaults.render_communications, cloned.render_communications);
    }

    #[test]
    fn test_restore_result_serialization() {
        let result = RestoreResult {
            restored_count: 3,
            failed_count: 1,
            failures: vec![RestoreFailure {
                device_role: "Playback Device".to_string(),
                error: "Device not found".to_string(),
            }],
        };

        let json = serde_json::to_string(&result).expect("Serialization failed");
        assert!(json.contains("restored_count"));
        assert!(json.contains("Playback Device"));

        let deserialized: RestoreResult =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.restored_count, 3);
        assert_eq!(deserialized.failed_count, 1);
        assert_eq!(deserialized.failures.len(), 1);
        assert_eq!(deserialized.failures[0].device_role, "Playback Device");
    }

    #[test]
    fn test_restore_result_all_success() {
        let result = RestoreResult {
            restored_count: 4,
            failed_count: 0,
            failures: vec![],
        };

        assert_eq!(result.restored_count, 4);
        assert!(result.failures.is_empty());
    }
}
