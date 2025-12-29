//! Windows default audio device save/restore functionality
//!
//! Uses the com-policy-config crate and windows crate for COM operations.
//! This is needed because VB-Cable installation changes the Windows default audio device.

use com_policy_config::{IPolicyConfig, PolicyConfigClient};
use tracing::{debug, error, info};
use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};

/// Manager for saving and restoring the Windows default audio device
#[derive(Debug, Clone)]
pub struct DefaultDeviceManager {
    saved_device_id: Option<String>,
}

impl DefaultDeviceManager {
    /// Save the current default output audio device
    ///
    /// Call this before VB-Cable installation to preserve the user's original default device.
    pub fn save_current_default() -> Result<Self, String> {
        let device_id = unsafe { get_default_device_id() }?;

        info!("Saved current default audio device: {}", device_id);

        Ok(Self {
            saved_device_id: Some(device_id),
        })
    }

    /// Get the saved device ID
    pub fn get_saved_device_id(&self) -> Option<String> {
        self.saved_device_id.clone()
    }

    /// Restore the saved device as the default
    ///
    /// Call this after VB-Cable installation completes to restore the user's original default.
    pub fn restore_default(&self) -> Result<(), String> {
        match &self.saved_device_id {
            Some(device_id) => Self::restore_device(device_id),
            None => Err("No device saved to restore".to_string()),
        }
    }

    /// Restore a specific device as the default (static method)
    ///
    /// Used when the device ID is stored externally (e.g., in frontend state).
    pub fn restore_device(device_id: &str) -> Result<(), String> {
        unsafe { set_default_device(device_id) }
    }
}

/// Get the current default output device ID (internal)
///
/// # Safety
/// Uses COM APIs which require proper initialization/cleanup.
unsafe fn get_default_device_id() -> Result<String, String> {
    // Initialize COM
    let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
    if hr.is_err() {
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

        // Get default output device (render = output, console = default role)
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|e| {
                error!("Failed to get default audio endpoint: {:?}", e);
                format!("Failed to get default audio device: {}", e)
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

        debug!("Current default device ID: {}", device_id);
        Ok(device_id)
    })();

    // Always uninitialize COM
    CoUninitialize();

    result
}

/// Set a device as the default output device (internal)
///
/// # Safety
/// Uses COM APIs which require proper initialization/cleanup.
unsafe fn set_default_device(device_id: &str) -> Result<(), String> {
    // Initialize COM
    let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
    if hr.is_err() {
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

        // Set as default for console role (main output)
        policy_config
            .SetDefaultEndpoint(device_id_pcwstr, eConsole)
            .map_err(|e| {
                error!("Failed to set default endpoint: {:?}", e);
                format!("Failed to set default audio device: {}", e)
            })?;

        info!("Restored default audio device: {}", device_id);
        Ok(())
    })();

    // Always uninitialize COM
    CoUninitialize();

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
    fn test_restore_without_saved_device() {
        let manager = DefaultDeviceManager {
            saved_device_id: None,
        };
        let result = manager.restore_default();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No device saved"));
    }
}
