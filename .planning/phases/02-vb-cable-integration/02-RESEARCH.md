# Phase 2: VB-Cable Integration - Research

**Researched:** 2025-12-29
**Domain:** Windows Virtual Audio Device Integration (VB-Cable)
**Confidence:** HIGH

<research_summary>
## Summary

Researched the VB-Cable ecosystem for automatic detection and silent installation in a Tauri/Rust desktop app. VB-Cable is a kernel-level Windows audio driver that creates virtual audio devices for routing audio between applications.

**Key findings:**
1. VB-Cable supports silent installation (`-i -h` flags), but Windows will still prompt for driver approval (cannot be bypassed programmatically)
2. VB-Cable changes Windows default audio device during install - requires workaround to restore user's original default
3. Detection is straightforward via cpal device enumeration (look for "CABLE" in device name)
4. Licensing allows bundling if donationware model remains visible and attribution is included

**Primary recommendation:** Implement a 3-step flow: (1) Detect VB-Cable via cpal, (2) If missing, offer download link with clear instructions (don't auto-install due to driver approval requirement), (3) Save/restore default audio device using `com-policy-config` crate.
</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| cpal | 0.15+ | Audio device enumeration | Already in use, cross-platform, WASAPI support |
| windows-registry | 0.6+ | Registry access | Official Microsoft crate for Windows registry |
| com-policy-config | 0.6.0 | Set default audio device | Only Rust crate for IPolicyConfig interface |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| windows | 0.61+ | Windows API bindings | COM initialization, device properties |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| com-policy-config | Direct COM via windows crate | More control but more code, com-policy-config simpler |
| cpal for detection | WMI queries | cpal already in use, simpler integration |
| VB-Cable | Virtual Audio Cable (VAC) | VAC is paid ($25), VB-Cable is donationware |
| VB-Cable | Voicemeeter | Voicemeeter is more complex, overkill for simple routing |

**Installation:**
```toml
[dependencies]
com-policy-config = "0.6"
windows-registry = "0.6"

[dependencies.windows]
version = "0.61"
features = [
    "Win32_Foundation",
    "Win32_Media_Audio",
    "Win32_System_Com",
    "Win32_Devices_FunctionDiscovery",
    "Win32_UI_Shell_PropertiesSystem",
]
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/
├── audio/
│   ├── device.rs         # Existing - extend for VB-Cable detection
│   └── ...
├── vbcable/              # NEW - VB-Cable integration module
│   ├── mod.rs            # Module exports
│   ├── detection.rs      # VB-Cable detection logic
│   ├── default_device.rs # Save/restore default device
│   └── installer.rs      # Installation guidance/automation
└── commands/
    └── audio.rs          # Existing - add VB-Cable commands
```

### Pattern 1: Device Detection via Name Matching
**What:** Use cpal to enumerate devices, look for "CABLE" substring in device name
**When to use:** Checking if VB-Cable is installed
**Example:**
```rust
// Source: cpal docs + VB-Audio device naming
use cpal::traits::{DeviceTrait, HostTrait};

pub fn is_vb_cable_installed() -> bool {
    let host = cpal::default_host();

    // Check output devices for "CABLE Input"
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.to_lowercase().contains("cable input") {
                    return true;
                }
            }
        }
    }
    false
}

pub fn get_vb_cable_device() -> Option<String> {
    let host = cpal::default_host();

    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                // VB-Cable appears as "CABLE Input (VB-Audio Virtual Cable)"
                if name.to_lowercase().contains("cable input") {
                    return Some(name);
                }
            }
        }
    }
    None
}
```

### Pattern 2: Save/Restore Default Audio Device
**What:** Save current default before VB-Cable install, restore after
**When to use:** VB-Cable installation changes Windows default audio device
**Example:**
```rust
// Source: com-policy-config crate + windows crate
use com_policy_config::{IPolicyConfig, PolicyConfigClient};
use windows::Win32::Media::Audio::{
    eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize,
    CLSCTX_ALL, COINIT_MULTITHREADED,
};

pub struct DefaultDeviceManager {
    saved_device_id: Option<String>,
}

impl DefaultDeviceManager {
    pub fn save_current_default() -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)?;

            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

            let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
            let device_id = device.GetId()?.to_string()?;

            CoUninitialize();

            Ok(Self {
                saved_device_id: Some(device_id),
            })
        }
    }

    pub fn restore_default(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref device_id) = self.saved_device_id {
            unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED)?;

                let policy_config: IPolicyConfig =
                    CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)?;

                policy_config.SetDefaultEndpoint(device_id, eConsole)?;

                CoUninitialize();
            }
        }
        Ok(())
    }
}
```

### Pattern 3: Installation Flow (User-Guided)
**What:** Guide user through VB-Cable installation instead of silent auto-install
**When to use:** First-time setup or when VB-Cable not detected
**Example:**
```rust
// Frontend receives this status and shows appropriate UI
#[derive(Serialize)]
pub enum VbCableStatus {
    Installed { device_name: String },
    NotInstalled,
    InstallationPending,
}

#[tauri::command]
pub fn check_vb_cable_status() -> VbCableStatus {
    if let Some(device_name) = get_vb_cable_device() {
        VbCableStatus::Installed { device_name }
    } else {
        VbCableStatus::NotInstalled
    }
}

// Open VB-Audio download page
#[tauri::command]
pub fn open_vb_cable_download() -> Result<(), String> {
    open::that("https://vb-audio.com/Cable/")
        .map_err(|e| e.to_string())
}
```

### Anti-Patterns to Avoid
- **Auto-downloading and installing VB-Cable:** Windows requires driver approval dialog - cannot be bypassed
- **Bundling VB-Cable installer in your app:** Licensing requires donationware visibility, versioning issues
- **Ignoring default device change:** VB-Cable install changes Windows default - users get confused
- **Silent install without admin rights:** Will fail - requires elevation
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Audio device enumeration | Manual Windows API calls | cpal | Already using it, cross-platform, handles WASAPI |
| Set default audio device | Manual COM/IPolicyConfig | com-policy-config crate | Complex COM interface, already ported to Rust |
| Registry access | winreg or raw Windows API | windows-registry crate | Official Microsoft crate, safer |
| VB-Cable installer | Custom download/execute logic | Link to official download | Driver signing, versioning, licensing issues |
| Driver detection | Registry scanning | cpal device enumeration | Name matching is simpler and more reliable |

**Key insight:** VB-Cable integration is primarily about detection and user guidance, not automation. Windows security prevents fully automated driver installation, so embrace a user-guided flow.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Attempting Fully Silent Installation
**What goes wrong:** Users expect "one-click" installation but Windows blocks driver approval
**Why it happens:** Windows security requires explicit user consent for kernel drivers
**How to avoid:** Design UI that clearly explains VB-Cable is separate software requiring manual approval
**Warning signs:** User complaints about "installation failed" or "driver blocked"

### Pitfall 2: Default Audio Device Changes After Install
**What goes wrong:** After VB-Cable install, all audio routes to VB-Cable instead of speakers
**Why it happens:** Windows automatically sets newly installed audio devices as default
**How to avoid:** Save default device before suggesting install, restore after user confirms installation complete
**Warning signs:** Users reporting "no audio" after VB-Cable setup

### Pitfall 3: Memory Integrity Blocking Installation
**What goes wrong:** VB-Cable install fails on Windows 11 with obscure error
**Why it happens:** Windows 11 Core Isolation/Memory Integrity blocks unsigned kernel drivers
**How to avoid:** Detect Windows 11, show instructions to temporarily disable Memory Integrity
**Warning signs:** Error 536870330 (0xE0000246) or "driver not compatible"

### Pitfall 4: Device Enumeration Timing Issues
**What goes wrong:** VB-Cable not detected immediately after installation
**Why it happens:** Windows audio subsystem needs time to recognize new devices, may need reboot
**How to avoid:** Implement retry logic with delay, suggest reboot if device not found after installation
**Warning signs:** "VB-Cable not found" right after user completes installation

### Pitfall 5: VM/Cloud Environment Incompatibility
**What goes wrong:** VB-Cable fails in Azure VMs, Hyper-V, or other virtualized environments
**Why it happens:** Virtual audio devices conflict with virtualization layer
**How to avoid:** Detect VM environment, show warning about limited support
**Warning signs:** "LOADDRV: specified driver is not a better match" error

### Pitfall 6: Stale Device Index
**What goes wrong:** Device enumeration returns different indices after hot-plug or VB-Cable install
**Why it happens:** cpal device indices are not stable across enumerations
**How to avoid:** Use device name matching instead of index, re-enumerate when needed
**Warning signs:** Wrong device selected, audio going to unexpected output
</common_pitfalls>

<code_examples>
## Code Examples

Verified patterns from official sources:

### VB-Cable Detection (Complete)
```rust
// Source: cpal docs + VB-Audio naming convention
use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Clone)]
pub struct VbCableInfo {
    pub output_device: String,  // "CABLE Input" - for playback
    pub input_device: String,   // "CABLE Output" - for recording
}

/// Check if VB-Cable is installed and return device info
pub fn detect_vb_cable() -> Option<VbCableInfo> {
    let host = cpal::default_host();

    let mut output_device = None;
    let mut input_device = None;

    // Find output device (CABLE Input - where apps send audio)
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.to_lowercase().contains("cable input") {
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
                    input_device = Some(name);
                    break;
                }
            }
        }
    }

    match (output_device, input_device) {
        (Some(out), Some(inp)) => Some(VbCableInfo {
            output_device: out,
            input_device: inp,
        }),
        _ => None,
    }
}
```

### Device Enumeration with Retry
```rust
// Source: GitHub issue discussions on device enumeration timing
use std::thread;
use std::time::Duration;

const MAX_RETRIES: u32 = 5;
const RETRY_DELAY_MS: u64 = 1000;

pub fn wait_for_vb_cable() -> Option<VbCableInfo> {
    for attempt in 0..MAX_RETRIES {
        if let Some(info) = detect_vb_cable() {
            tracing::info!("VB-Cable detected on attempt {}", attempt + 1);
            return Some(info);
        }

        if attempt < MAX_RETRIES - 1 {
            tracing::debug!("VB-Cable not found, retrying in {}ms...", RETRY_DELAY_MS);
            thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
        }
    }

    tracing::warn!("VB-Cable not detected after {} attempts", MAX_RETRIES);
    None
}
```

### Silent Install Command (Reference Only)
```rust
// Source: VB-Audio Forum - https://forum.vb-audio.com/viewtopic.php?t=1909
// NOTE: Requires admin rights and will show Windows driver approval dialog

use std::process::Command;

/// Attempts silent VB-Cable installation
/// Returns Ok(()) if installer launched, but user must approve driver
pub fn launch_vb_cable_installer(installer_path: &str) -> Result<(), String> {
    // -i = install, -h = headless/silent mode
    let status = Command::new(installer_path)
        .args(["-i", "-h"])
        .status()
        .map_err(|e| format!("Failed to launch installer: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Installer exited with code: {:?}", status.code()))
    }
}

// For uninstall:
// VBCABLE_Setup_x64.exe -u -h
```
</code_examples>

<sota_updates>
## State of the Art (2024-2025)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Registry scanning for detection | cpal device enumeration | 2023+ | Simpler, more reliable |
| Manual COM for default device | com-policy-config crate | 2024 (v0.6.0) | Clean Rust API |
| Bundling installer | Download link + guidance | Always | Avoids licensing/signing issues |

**New tools/patterns to consider:**
- **com-policy-config 0.6.0 (July 2025):** Updated windows crate compatibility
- **windows-registry 0.6:** Official Microsoft crate for registry access
- **Windows 11 Memory Integrity:** Must be disabled for driver installation

**Deprecated/outdated:**
- **winreg crate for registry:** Use official windows-registry instead
- **Fully automated silent install:** Windows security prevents this since Win10/11
- **Device index-based selection:** Indices are unstable, use name matching
</sota_updates>

<open_questions>
## Open Questions

1. **Registry keys for VB-Cable detection**
   - What we know: VB-Audio's VBDeviceCheck tool reads driver info from registry
   - What's unclear: Exact registry keys used (not documented publicly)
   - Recommendation: Use cpal device enumeration instead - simpler and sufficient

2. **Exact error codes from silent installer**
   - What we know: `-i -h` flags work, installer returns exit code
   - What's unclear: Full list of return codes and their meanings
   - Recommendation: Check exit code, fall back to user guidance on any failure

3. **Multiple VB-Cable instances (A+B, C+D)**
   - What we know: Paid versions add more virtual cables
   - What's unclear: Whether SonicDeck benefits from multiple cables
   - Recommendation: Start with free VB-Cable, single cable sufficient for Discord routing
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [VB-Audio Official Site](https://vb-audio.com/Cable/) - VB-Cable overview, download, requirements
- [VB-Audio Licensing Page](https://vb-audio.com/Services/licensing.htm) - Bundling/distribution terms
- [cpal Documentation (Context7)](/websites/rs_cpal_0_17_0_cpal) - Device enumeration API
- [com-policy-config lib.rs](https://lib.rs/crates/com-policy-config) - Version 0.6.0, dependencies
- [windows-rs (Context7)](/microsoft/windows-rs) - Registry access patterns

### Secondary (MEDIUM confidence)
- [VB-Audio Forum: Silent Installation](https://forum.vb-audio.com/viewtopic.php?t=1909) - `-i -h` flags confirmed by maintainer
- [VB-Audio Forum: Installation Checking](https://forum.vb-audio.com/viewtopic.php?t=688) - VBDeviceCheck tool info
- [GitHub: com-policy-config](https://github.com/sidit77/com-policy-config) - Example code, caveats
- [SoundSwitch Discussion](https://github.com/Belphemur/SoundSwitch) - IPolicyConfig usage patterns

### Tertiary (LOW confidence - needs validation)
- WebSearch results on Windows 11 Memory Integrity issues - confirmed via VB-Audio Forum
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: VB-Cable virtual audio driver
- Ecosystem: Windows audio APIs, cpal, COM interfaces
- Patterns: Device detection, default device management, guided installation
- Pitfalls: Driver signing, default device changes, timing issues

**Confidence breakdown:**
- Standard stack: HIGH - verified with Context7, official crates
- Architecture: HIGH - patterns from official cpal/windows crate docs
- Pitfalls: HIGH - documented in VB-Audio forums and verified
- Code examples: HIGH - from Context7/official sources

**Research date:** 2025-12-29
**Valid until:** 2026-01-29 (30 days - stable ecosystem, no major changes expected)
</metadata>

---

*Phase: 02-vb-cable-integration*
*Research completed: 2025-12-29*
*Ready for planning: yes*
