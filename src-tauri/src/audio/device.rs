//! Audio device enumeration

use cpal::traits::{DeviceTrait, HostTrait};

use super::{AudioDevice, AudioError, DeviceId};

/// Lists all available output audio devices on the system
pub fn enumerate_devices() -> Result<Vec<AudioDevice>, AudioError> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    let default_device = host.default_output_device();
    let default_name = default_device
        .as_ref()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    let output_devices = host
        .output_devices()
        .map_err(|e| AudioError::DeviceEnumeration(e.to_string()))?;

    for (index, device) in output_devices.enumerate() {
        if let Ok(name) = device.name() {
            let device_id = DeviceId::from_index(index);
            let is_default = name == default_name;

            devices.push(AudioDevice {
                id: device_id,
                name,
                is_default,
            });
        }
    }

    if devices.is_empty() {
        return Err(AudioError::NoDevices);
    }

    Ok(devices)
}
