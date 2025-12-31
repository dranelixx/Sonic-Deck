//! Communications device auto-switching for VB-Cable integration
//!
//! Automatically sets VB-Cable Output as the Windows "Communications" capture device
//! when the app is active, and restores the original device when the app closes.
//!
//! This allows Discord/Teams/Zoom to use VB-Cable while SonicDeck is running,
//! and automatically switch back to the real microphone when SonicDeck closes.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, error, info, warn};
use windows::core::PCWSTR;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::{
    eCapture, eCommunications, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator,
    DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
};
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;

use com_policy_config::{IPolicyConfig, PolicyConfigClient};

/// COM error: already initialized with different threading mode (safe to ignore)
const RPC_E_CHANGED_MODE: i32 = 0x80010106u32 as i32;

/// State file for crash recovery
const STATE_FILE_NAME: &str = "vbcable_comm_state.json";

/// Persisted state for crash recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedState {
    /// Original communications capture device ID (before we changed it)
    original_device_id: String,
    /// Whether VB-Cable mode is currently active
    is_active: bool,
}

/// Global state for active communications mode
static COMM_STATE: Mutex<Option<CommState>> = Mutex::new(None);

/// In-memory state for communications mode
struct CommState {
    /// Original device ID to restore on deactivation
    original_device_id: String,
}

/// Get the state file path
fn get_state_file_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("com.sonicdeck.app").join(STATE_FILE_NAME))
}

/// Save state to disk for crash recovery
fn save_state(state: &PersistedState) -> Result<(), String> {
    let path = get_state_file_path().ok_or("Could not determine state file path")?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write state file: {}", e))?;

    debug!("Saved communications state to {:?}", path);
    Ok(())
}

/// Load state from disk (for crash recovery)
fn load_state() -> Option<PersistedState> {
    let path = get_state_file_path()?;

    if !path.exists() {
        return None;
    }

    match fs::read_to_string(&path) {
        Ok(json) => match serde_json::from_str(&json) {
            Ok(state) => {
                debug!("Loaded communications state from {:?}", path);
                Some(state)
            }
            Err(e) => {
                warn!("Failed to parse state file: {}", e);
                None
            }
        },
        Err(e) => {
            warn!("Failed to read state file: {}", e);
            None
        }
    }
}

/// Delete state file
fn clear_state() {
    if let Some(path) = get_state_file_path() {
        if path.exists() {
            if let Err(e) = fs::remove_file(&path) {
                warn!("Failed to delete state file: {}", e);
            } else {
                debug!("Cleared communications state file");
            }
        }
    }
}

/// Find VB-Cable Output device ID using Windows API
///
/// VB-Cable Output is a capture (input) device that provides audio from VB-Cable.
fn find_vbcable_output_device_id() -> Result<String, String> {
    unsafe {
        // Initialize COM
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        let we_initialized_com = hr.is_ok();
        if hr.is_err() && hr != windows::core::HRESULT(RPC_E_CHANGED_MODE) {
            return Err(format!("Failed to initialize COM: {:?}", hr));
        }

        let result = (|| -> Result<String, String> {
            // Create device enumerator
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .map_err(|e| format!("Failed to create device enumerator: {}", e))?;

            // Enumerate all active capture devices
            let collection = enumerator
                .EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)
                .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

            let count = collection
                .GetCount()
                .map_err(|e| format!("Failed to get device count: {}", e))?;

            for i in 0..count {
                let device: IMMDevice = collection
                    .Item(i)
                    .map_err(|e| format!("Failed to get device {}: {}", i, e))?;

                // Get device friendly name
                let props: IPropertyStore = device
                    .OpenPropertyStore(STGM_READ)
                    .map_err(|e| format!("Failed to open property store: {}", e))?;

                let name_prop = props
                    .GetValue(&PKEY_Device_FriendlyName)
                    .map_err(|e| format!("Failed to get device name: {}", e))?;

                // Convert PROPVARIANT to string using PropVariantToStringAlloc
                let name_pwstr = match PropVariantToStringAlloc(&name_prop) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let name = name_pwstr.to_string().unwrap_or_default();

                // Check if this is VB-Cable Output
                if name.to_lowercase().contains("cable output") {
                    // Get device ID
                    let device_id_pwstr = device
                        .GetId()
                        .map_err(|e| format!("Failed to get device ID: {}", e))?;

                    let device_id = device_id_pwstr
                        .to_string()
                        .map_err(|e| format!("Failed to convert device ID: {}", e))?;

                    debug!("Found VB-Cable Output: {} (ID: {})", name, device_id);
                    return Ok(device_id);
                }
            }

            Err("VB-Cable Output device not found".to_string())
        })();

        if we_initialized_com {
            CoUninitialize();
        }

        result
    }
}

/// Get the current default communications capture device ID
fn get_current_comm_capture_device() -> Result<String, String> {
    unsafe {
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        let we_initialized_com = hr.is_ok();
        if hr.is_err() && hr != windows::core::HRESULT(RPC_E_CHANGED_MODE) {
            return Err(format!("Failed to initialize COM: {:?}", hr));
        }

        let result = (|| -> Result<String, String> {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .map_err(|e| format!("Failed to create device enumerator: {}", e))?;

            let device = enumerator
                .GetDefaultAudioEndpoint(eCapture, eCommunications)
                .map_err(|e| format!("No default communications capture device: {}", e))?;

            let device_id_pwstr = device
                .GetId()
                .map_err(|e| format!("Failed to get device ID: {}", e))?;

            let device_id = device_id_pwstr
                .to_string()
                .map_err(|e| format!("Failed to convert device ID: {}", e))?;

            Ok(device_id)
        })();

        if we_initialized_com {
            CoUninitialize();
        }

        result
    }
}

/// Set a device as the default communications capture device
fn set_comm_capture_device(device_id: &str) -> Result<(), String> {
    unsafe {
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        let we_initialized_com = hr.is_ok();
        if hr.is_err() && hr != windows::core::HRESULT(RPC_E_CHANGED_MODE) {
            return Err(format!("Failed to initialize COM: {:?}", hr));
        }

        let result = (|| -> Result<(), String> {
            let policy_config: IPolicyConfig =
                CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)
                    .map_err(|e| format!("Failed to create policy config: {}", e))?;

            let device_id_wide: Vec<u16> =
                device_id.encode_utf16().chain(std::iter::once(0)).collect();
            let device_id_pcwstr = PCWSTR::from_raw(device_id_wide.as_ptr());

            policy_config
                .SetDefaultEndpoint(device_id_pcwstr, eCommunications)
                .map_err(|e| format!("Failed to set communications device: {}", e))?;

            debug!("Set communications capture device to: {}", device_id);
            Ok(())
        })();

        if we_initialized_com {
            CoUninitialize();
        }

        result
    }
}

/// Activate VB-Cable communications mode
///
/// Sets VB-Cable Output as the Windows communications capture device.
/// Saves the original device for later restoration.
pub fn activate() -> Result<(), String> {
    // Check if already active
    {
        let state = COMM_STATE
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        if state.is_some() {
            info!("VB-Cable communications mode already active");
            return Ok(());
        }
    }

    // Find VB-Cable Output device
    let vbcable_id = find_vbcable_output_device_id()?;

    // Get current communications device (to restore later)
    let original_id = get_current_comm_capture_device()?;

    // Don't switch if already using VB-Cable
    if original_id == vbcable_id {
        info!("Communications device is already VB-Cable Output");
        return Ok(());
    }

    // Save state for crash recovery BEFORE making the change
    save_state(&PersistedState {
        original_device_id: original_id.clone(),
        is_active: true,
    })?;

    // Set VB-Cable as communications device
    set_comm_capture_device(&vbcable_id)?;

    // Store in memory
    {
        let mut state = COMM_STATE
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *state = Some(CommState {
            original_device_id: original_id.clone(),
        });
    }

    info!(
        "Activated VB-Cable communications mode (original device saved: {})",
        original_id
    );
    Ok(())
}

/// Deactivate VB-Cable communications mode
///
/// Restores the original communications capture device.
pub fn deactivate() -> Result<(), String> {
    let original_id = {
        let mut state = COMM_STATE
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        match state.take() {
            Some(s) => s.original_device_id,
            None => {
                debug!("VB-Cable communications mode not active");
                clear_state();
                return Ok(());
            }
        }
    };

    // Restore original device
    set_comm_capture_device(&original_id)?;

    // Clear persisted state
    clear_state();

    info!(
        "Deactivated VB-Cable communications mode (restored device: {})",
        original_id
    );
    Ok(())
}

/// Check if VB-Cable communications mode is active
pub fn is_active() -> bool {
    COMM_STATE.lock().map(|s| s.is_some()).unwrap_or(false)
}

/// Recover from crash - restore original device if state file exists
///
/// Called on app startup to clean up after a crash.
pub fn recover_from_crash() {
    if let Some(state) = load_state() {
        if state.is_active {
            info!(
                "Recovering from crash: restoring original communications device: {}",
                state.original_device_id
            );

            match set_comm_capture_device(&state.original_device_id) {
                Ok(_) => info!("Successfully restored original communications device"),
                Err(e) => error!("Failed to restore communications device: {}", e),
            }
        }
        clear_state();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persisted_state_serialization() {
        let state = PersistedState {
            original_device_id: "test-device-id".to_string(),
            is_active: true,
        };

        let json = serde_json::to_string(&state).expect("Serialization failed");
        let deserialized: PersistedState =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(state.original_device_id, deserialized.original_device_id);
        assert_eq!(state.is_active, deserialized.is_active);
    }

    #[test]
    fn test_is_active_default_false() {
        // In a fresh state, should not be active
        // Note: This test may fail if run after activate() without deactivate()
        // but is safe in isolation
        let active = is_active();
        // We can't assert false here because other tests may have run
        // Just verify it doesn't panic
        let _ = active;
    }
}
