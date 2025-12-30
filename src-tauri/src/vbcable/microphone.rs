//! Microphone routing module for VB-Cable integration
//!
//! Routes microphone audio to CABLE Input so users can be heard on Discord
//! while using VB-Cable for soundboard routing.
//!
//! Audio flow: Microphone -> [This Module] -> CABLE Input -> CABLE Output -> Discord
//!
//! ## Latency
//! Uses a 100ms ring buffer for balance between latency and stability.
//! Earlier versions used 1s which caused noticeable delay (see #83).

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tracing::{debug, error, info, warn};

use crate::audio::DeviceId;

// ============================================================================
// Global Routing State
// ============================================================================

/// Global state for active microphone routing - only stores thread-safe data
static ROUTING_STATE: Mutex<Option<RoutingHandle>> = Mutex::new(None);

/// Thread-safe handle for controlling an active routing session
struct RoutingHandle {
    /// Device ID of the microphone being routed
    microphone_id: String,
    /// Signal to stop the routing thread
    stop_signal: Arc<AtomicBool>,
    /// Handle to the routing thread (for cleanup)
    _thread_handle: JoinHandle<()>,
}

// ============================================================================
// Capture Device Enumeration
// ============================================================================

/// List available capture devices (microphones)
///
/// Returns (DeviceId, display_name) pairs.
/// Excludes VB-Cable devices (CABLE Output is a recording device).
pub fn list_capture_devices() -> Vec<(String, String)> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    if let Ok(input_devices) = host.input_devices() {
        for (index, device) in input_devices.enumerate() {
            if let Ok(name) = device.name() {
                // Skip VB-Cable devices (CABLE Output appears as input device)
                if !name.to_lowercase().contains("cable") {
                    let id = DeviceId::from_index(index).to_string();
                    devices.push((id, name));
                }
            }
        }
    }

    debug!(
        "Found {} capture devices (excluding VB-Cable)",
        devices.len()
    );
    devices
}

/// Find a capture device by DeviceId
fn find_capture_device(device_id: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();

    // Parse the index from the device_id (e.g., "device_0" -> 0)
    let index = device_id
        .strip_prefix("device_")
        .and_then(|s| s.parse::<usize>().ok())?;

    host.input_devices().ok()?.nth(index)
}

/// Find CABLE Input device (output device for routing audio to VB-Cable)
fn find_cable_input_device() -> Option<cpal::Device> {
    let host = cpal::default_host();

    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.to_lowercase().contains("cable input") {
                    debug!("Found CABLE Input device: {}", name);
                    return Some(device);
                }
            }
        }
    }

    None
}

// ============================================================================
// Microphone Routing
// ============================================================================

/// Ring buffer for transferring audio between input and output streams
struct RingBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
    read_pos: usize,
    capacity: usize,
    overflow_logged: bool,
}

impl RingBuffer {
    /// Create a new ring buffer with prefilled silence
    ///
    /// Prefills half the buffer with silence so the output stream
    /// never "starves" waiting for input data. This prevents
    /// audio glitches at stream startup (industry standard practice).
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            write_pos: capacity / 2, // Start ahead to prevent underruns
            read_pos: 0,
            capacity,
            overflow_logged: false,
        }
    }

    fn write(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;

            // Detect buffer overflow (write catching up to read)
            if self.write_pos == self.read_pos && !self.overflow_logged {
                warn!("Ring buffer overflow detected - some audio data may be lost");
                self.overflow_logged = true;
            }
        }
    }

    fn read(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
    }
}

/// Enable microphone routing to CABLE Input
///
/// Captures audio from the specified microphone and routes it to CABLE Input.
/// This allows the user's voice to be mixed with soundboard audio in VB-Cable.
pub fn enable_routing(microphone_id: &str) -> Result<(), String> {
    // Check if routing is already active
    {
        let state = ROUTING_STATE
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        if let Some(existing) = state.as_ref() {
            if existing.microphone_id == microphone_id {
                info!(
                    "Microphone routing already active for device: {}",
                    microphone_id
                );
                return Ok(());
            }
            return Err(
                "Routing already active with different microphone. Disable first.".to_string(),
            );
        }
    }

    // Find microphone device
    let mic_device = find_capture_device(microphone_id)
        .ok_or_else(|| format!("Microphone device not found: {}", microphone_id))?;
    let mic_name = mic_device.name().unwrap_or_else(|_| "Unknown".to_string());
    info!("Found microphone: {}", mic_name);

    // Find CABLE Input device
    let cable_device =
        find_cable_input_device().ok_or("CABLE Input device not found. Is VB-Cable installed?")?;
    let cable_name = cable_device
        .name()
        .unwrap_or_else(|_| "Unknown".to_string());
    info!("Found CABLE Input: {}", cable_name);

    // Get supported configs
    let input_config = mic_device
        .default_input_config()
        .map_err(|e| format!("No input config for microphone: {}", e))?;
    let output_config = cable_device
        .default_output_config()
        .map_err(|e| format!("No output config for CABLE Input: {}", e))?;

    info!(
        "Input config: {} Hz, {} channels, {:?}",
        input_config.sample_rate().0,
        input_config.channels(),
        input_config.sample_format()
    );
    info!(
        "Output config: {} Hz, {} channels, {:?}",
        output_config.sample_rate().0,
        output_config.channels(),
        output_config.sample_format()
    );

    // Create stop signal
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = stop_signal.clone();

    // Store configuration for the thread
    let mic_id = microphone_id.to_string();
    let input_channels = input_config.channels();
    let output_channels = output_config.channels();
    let sample_rate = input_config.sample_rate();

    // Calculate buffer size for ~100ms latency (balance between latency and stability)
    // Formula: sample_rate * channels / 10 (100ms = 1/10 second)
    // Note: 50ms was too aggressive and caused audio glitches
    let buffer_size = (sample_rate.0 as usize * input_channels as usize / 10).max(4096);
    debug!(
        "Ring buffer size: {} samples (~100ms at {} Hz, {} ch)",
        buffer_size, sample_rate.0, input_channels
    );

    // Spawn routing thread
    let thread_handle = thread::spawn(move || {
        let ring_buffer = Arc::new(Mutex::new(RingBuffer::new(buffer_size)));
        let ring_buffer_input = ring_buffer.clone();
        let ring_buffer_output = ring_buffer;

        let stop_signal_input = stop_signal_clone.clone();
        let stop_signal_output = stop_signal_clone.clone();

        // Build input stream (capture from microphone)
        let input_stream = match mic_device.build_input_stream(
            &input_config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if stop_signal_input.load(Ordering::Relaxed) {
                    return;
                }

                if let Ok(mut buffer) = ring_buffer_input.lock() {
                    buffer.write(data);
                }
            },
            move |err| {
                error!("Input stream error: {}", err);
            },
            None,
        ) {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to build input stream: {}", e);
                return;
            }
        };

        // Build output stream config matching input sample rate
        let output_stream_config = cpal::StreamConfig {
            channels: output_channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        // Build output stream (play to CABLE Input)
        let input_ch = input_channels;
        let output_ch = output_channels;
        let output_stream = match cable_device.build_output_stream(
            &output_stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if stop_signal_output.load(Ordering::Relaxed) {
                    // Fill with silence when stopped
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }
                    return;
                }

                if let Ok(mut buffer) = ring_buffer_output.lock() {
                    // Handle channel conversion if needed
                    if input_ch == output_ch {
                        buffer.read(data);
                    } else if input_ch == 1 && output_ch == 2 {
                        // Mono to stereo: duplicate each sample
                        let mono_samples = data.len() / 2;
                        let mut mono_buffer = vec![0.0f32; mono_samples];
                        buffer.read(&mut mono_buffer);
                        for (i, &sample) in mono_buffer.iter().enumerate() {
                            data[i * 2] = sample;
                            data[i * 2 + 1] = sample;
                        }
                    } else if input_ch == 2 && output_ch == 1 {
                        // Stereo to mono: average channels
                        let stereo_samples = data.len() * 2;
                        let mut stereo_buffer = vec![0.0f32; stereo_samples];
                        buffer.read(&mut stereo_buffer);
                        for i in 0..data.len() {
                            data[i] = (stereo_buffer[i * 2] + stereo_buffer[i * 2 + 1]) * 0.5;
                        }
                    } else {
                        // Fallback: just read what we can
                        buffer.read(data);
                    }
                } else {
                    // Fill with silence if lock fails
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }
                }
            },
            move |err| {
                error!("Output stream error: {}", err);
            },
            None,
        ) {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to build output stream: {}", e);
                return;
            }
        };

        // Start both streams
        if let Err(e) = input_stream.play() {
            error!("Failed to start input stream: {}", e);
            return;
        }
        if let Err(e) = output_stream.play() {
            error!("Failed to start output stream: {}", e);
            return;
        }

        info!("Microphone routing started: {} -> CABLE Input", mic_id);

        // Keep thread alive while routing is active
        while !stop_signal_clone.load(Ordering::Relaxed) {
            thread::sleep(std::time::Duration::from_millis(100));
        }

        info!("Microphone routing thread stopping");
        // Streams are dropped here, which stops them
    });

    info!("Microphone routing enabled: {} -> {}", mic_name, cable_name);

    // Store handle
    let mut state = ROUTING_STATE
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    *state = Some(RoutingHandle {
        microphone_id: microphone_id.to_string(),
        stop_signal,
        _thread_handle: thread_handle,
    });

    Ok(())
}

/// Disable microphone routing
///
/// Stops the audio routing and releases resources.
pub fn disable_routing() -> Result<(), String> {
    let mut state = ROUTING_STATE
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    if let Some(routing) = state.take() {
        // Signal thread to stop
        routing.stop_signal.store(true, Ordering::Relaxed);
        info!(
            "Microphone routing disabled for device: {}",
            routing.microphone_id
        );
        // Thread handle is dropped here, thread will complete
    } else {
        debug!("No active microphone routing to disable");
    }

    Ok(())
}

/// Get current routing status
///
/// Returns the microphone device ID if routing is active, None otherwise.
pub fn get_routing_status() -> Option<String> {
    ROUTING_STATE
        .lock()
        .ok()
        .and_then(|state| state.as_ref().map(|s| s.microphone_id.clone()))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_prefill() {
        let buffer = RingBuffer::new(10);

        // Buffer should be prefilled with write_pos ahead of read_pos
        assert_eq!(buffer.write_pos, 5); // capacity / 2
        assert_eq!(buffer.read_pos, 0);
        assert!(!buffer.overflow_logged);

        // Buffer contains silence (0.0)
        assert!(buffer.buffer.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_ring_buffer_write_read() {
        let mut buffer = RingBuffer::new(10);

        // Skip prefilled silence first (5 samples)
        let mut prefill = [0.0; 5];
        buffer.read(&mut prefill);
        assert!(prefill.iter().all(|&x| x == 0.0));

        // Now write and read should be in sync
        buffer.write(&[1.0, 2.0, 3.0]);

        let mut output = [0.0; 3];
        buffer.read(&mut output);

        assert_eq!(output, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_ring_buffer_wrap() {
        let mut buffer = RingBuffer::new(8);

        // Skip prefilled silence (4 samples)
        let mut prefill = [0.0; 4];
        buffer.read(&mut prefill);

        // Write and read multiple times to test wrap-around
        buffer.write(&[1.0, 2.0]);
        let mut out1 = [0.0; 2];
        buffer.read(&mut out1);
        assert_eq!(out1, [1.0, 2.0]);

        buffer.write(&[3.0, 4.0, 5.0]);
        let mut out2 = [0.0; 3];
        buffer.read(&mut out2);
        assert_eq!(out2, [3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_ring_buffer_overflow_detection() {
        let mut buffer = RingBuffer::new(8);
        assert!(!buffer.overflow_logged);

        // Write enough samples to cause overflow (more than capacity)
        // Buffer starts with write_pos = 4, read_pos = 0
        // Writing 5 samples: write_pos goes 4->5->6->7->0->1
        // When write_pos becomes 0 and read_pos is still 0, overflow is detected
        buffer.write(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        // Overflow should be detected (write caught up to read)
        assert!(buffer.overflow_logged);
    }

    #[test]
    fn test_get_routing_status_none() {
        // Initially no routing should be active
        let status = get_routing_status();
        // Status could be None or Some if a previous test left it active
        // This test just verifies the function doesn't panic
        let _ = status;
    }

    #[test]
    fn test_list_capture_devices_excludes_cable() {
        // Verifies VB-Cable devices are filtered out from the list
        // (actual device count depends on system hardware)
        let devices = list_capture_devices();
        for (_, name) in &devices {
            assert!(
                !name.to_lowercase().contains("cable"),
                "VB-Cable device should be excluded: {}",
                name
            );
        }
    }
}
