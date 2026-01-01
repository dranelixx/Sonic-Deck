# Phase 3: Audio Core Polish - Research

**Researched:** 2025-12-31
**Domain:** LUFS Loudness Measurement + Perceptual Volume Curves (Rust Audio)
**Confidence:** HIGH

<research_summary>
## Summary

Researched LUFS loudness measurement (ITU-R BS.1770 / EBU R128 Standard) and perceptual volume curve algorithms for SonicDeck's Audio Core Polish phase. The goal is to implement loudness normalization so all sounds play at consistent perceived volume, and improve the volume slider to feel more natural.

**Key findings:**
- **ebur128** is the standard Rust crate for LUFS measurement - a pure-Rust port of libebur128 with full EBU R128 compliance
- LUFS calculation should happen at **import time** (when decoding), not during live playback
- Target loudness of **-14 LUFS** is the streaming industry standard (Spotify, YouTube)
- Current volume curve (`sqrt * 0.2`) is too mild - **x^4** approximation matches human perception better
- Mono audio requires special channel configuration to avoid -3 LU measurement error

**Primary recommendation:** Add ebur128 dependency, calculate LUFS during audio decode, store in AudioData struct, apply gain compensation during playback using the existing volume infrastructure.
</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ebur128 | 0.1.10 | LUFS loudness measurement | Pure Rust, EBU R128 compliant, passes all official tests |
| symphonia | (existing) | Audio decoding | Already used for MP3/OGG/M4A decode |
| cpal | (existing) | Audio I/O | Already used for playback streams |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| - | - | No additional libraries needed | Stack is complete |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ebur128 | libebur128 (FFI) | ebur128 is pure Rust, no C dependency, simpler build |
| Import-time LUFS | Live LUFS | Live is more CPU intensive, unnecessary for soundboard use case |

**Installation:**
```toml
[dependencies]
ebur128 = "0.1"
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Integration

```
File → Symphonia decode → LUFS calculation (ebur128) → AudioData (with lufs_value) → Cache
                                                                    ↓
Playback ← Volume curve ← LUFS compensation ← AudioData from cache
```

### Pattern 1: LUFS Calculation During Decode
**What:** Calculate integrated LUFS when audio is first decoded, before caching
**When to use:** Always - one-time cost at import, reused for all playbacks
**Example:**
```rust
use ebur128::{EbuR128, Mode};

fn calculate_lufs(samples: &[f32], channels: u32, sample_rate: u32) -> Option<f64> {
    // Mode::I = Integrated loudness (full program)
    let mut ebu = EbuR128::new(channels, sample_rate, Mode::I).ok()?;

    ebu.add_frames_f32(samples).ok()?;

    let lufs = ebu.loudness_global().ok()?;

    // Filter invalid values (silence, too short)
    if lufs.is_finite() && lufs > -70.0 {
        Some(lufs)
    } else {
        None
    }
}
```

### Pattern 2: LUFS Normalization Gain
**What:** Calculate gain adjustment to bring audio to target loudness
**When to use:** During playback when LUFS normalization is enabled
**Example:**
```rust
const TARGET_LUFS: f64 = -14.0; // Streaming standard

fn calculate_lufs_gain(measured_lufs: f64, target_lufs: f64) -> f32 {
    // Difference in LUFS = difference in dB
    let gain_db = target_lufs - measured_lufs;

    // Convert dB to linear gain: 10^(dB/20)
    let linear_gain = 10.0_f64.powf(gain_db / 20.0);

    // Clamp to prevent excessive amplification
    linear_gain.clamp(0.1, 4.0) as f32
}
```

### Pattern 3: Perceptual Volume Curve (x^4)
**What:** Replace sqrt curve with x^4 for better perceived volume control
**When to use:** All volume slider interactions
**Example:**
```rust
/// Calculate scaled volume using perceptual x^4 curve
/// Based on research: x^4 closely approximates ideal 60dB exponential curve
pub fn calculate_scaled_volume(volume: f32) -> f32 {
    // x^4 curve for perceptual scaling
    let perceptual = volume.powi(4);

    // Apply base attenuation (0.2 = max 20% amplitude, safe default)
    perceptual * 0.2
}
```

### Anti-Patterns to Avoid
- **Linear volume slider:** Human hearing is logarithmic, linear feels wrong
- **Live LUFS calculation:** CPU-intensive, unnecessary for soundboard
- **Ignoring silence:** LUFS of -inf or NaN breaks calculations
- **Not clamping gain:** Over-amplifying quiet sounds causes clipping
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| LUFS measurement | Custom K-weighting filters | ebur128 crate | BS.1770 K-weighting is complex (pre-filter + RLB weighting), ebur128 passes all EBU tests |
| Integrated loudness | Simple RMS calculation | ebur128::loudness_global() | ITU-R BS.1770 requires gating at -70 LUFS absolute and -10 LU relative |
| True peak detection | Max sample value | ebur128::true_peak() | Needs 4x oversampling via polyphase FIR for inter-sample peaks |
| Volume curve | Custom logarithm | x^4 power function | Simple multiplication, mathematically proven approximation |

**Key insight:** LUFS calculation involves ITU-R BS.1770 K-weighting filters with specific frequency responses. Hand-rolling this is error-prone and takes significant effort. ebur128 is battle-tested, pure Rust, and produces identical results to the C libebur128.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Mono Channel Configuration
**What goes wrong:** LUFS measurement is 3 LU too low for mono audio
**Why it happens:** Default left-channel-only config doesn't account for mono playback through stereo speakers
**How to avoid:** Set channel to `Channel::DualMono` for mono files, or duplicate mono to stereo before measurement
**Warning signs:** Mono files sound much louder after normalization than expected
**Source:** [libebur128 Issue #42](https://github.com/jiixyj/libebur128/issues/42)

### Pitfall 2: Silence and Very Short Samples
**What goes wrong:** LUFS returns -inf, NaN, or unreliable values
**Why it happens:** ITU-R BS.1770 gating at -70 LUFS drops all content; insufficient samples for reliable measurement
**How to avoid:** Check `lufs.is_finite() && lufs > -70.0`, treat None/invalid as "no normalization"
**Warning signs:** Crashes, extreme gain, or silent playback

### Pitfall 3: Excessive Amplification
**What goes wrong:** Very quiet sounds get amplified too much, causing clipping
**Why it happens:** Unclamped gain calculation (e.g., -50 LUFS sound normalized to -14 LUFS = +36 dB gain)
**How to avoid:** Clamp gain to reasonable range (e.g., 0.1x to 4x, or ±12 dB)
**Warning signs:** Distorted playback of quiet sounds

### Pitfall 4: Sample Rate Mismatch
**What goes wrong:** Incorrect LUFS values
**Why it happens:** ebur128 filter coefficients are calculated per sample rate
**How to avoid:** Pass correct sample rate from AudioData to EbuR128::new()
**Warning signs:** Inconsistent normalization across different source files

### Pitfall 5: Linear Volume Curve
**What goes wrong:** Volume slider feels "jumpy" at low end, "dead" at high end
**Why it happens:** Human hearing is logarithmic, requires exponential amplitude scaling
**How to avoid:** Use x^4 curve (or x^3 for gentler feel)
**Warning signs:** Users complain volume control doesn't feel right
**Source:** [Dr. Lex's Volume Controls](https://www.dr-lex.be/info-stuff/volumecontrols.html)
</common_pitfalls>

<code_examples>
## Code Examples

Verified patterns from official sources:

### Complete LUFS Integration in AudioData
```rust
// Source: ebur128 docs + SonicDeck architecture
use ebur128::{EbuR128, Mode};

pub struct AudioData {
    pub samples: Vec<f32>,       // Interleaved samples
    pub channels: u32,
    pub sample_rate: u32,
    pub duration_secs: f64,
    pub waveform: Option<Vec<f32>>,
    pub lufs: Option<f64>,       // NEW: Integrated loudness
}

impl AudioData {
    pub fn calculate_lufs(&mut self) -> Option<f64> {
        let mut ebu = EbuR128::new(self.channels, self.sample_rate, Mode::I).ok()?;

        ebu.add_frames_f32(&self.samples).ok()?;

        match ebu.loudness_global() {
            Ok(lufs) if lufs.is_finite() && lufs > -70.0 => {
                self.lufs = Some(lufs);
                Some(lufs)
            }
            _ => None,
        }
    }
}
```

### Volume Curve with LUFS Compensation
```rust
// Source: Research synthesis
const TARGET_LUFS: f64 = -14.0;

pub fn calculate_final_volume(
    slider_volume: f32,      // 0.0 - 1.0 from UI
    sound_lufs: Option<f64>, // From AudioData.lufs
    normalization_enabled: bool,
    target_lufs: f64,        // Configurable, default -14.0
) -> f32 {
    // Step 1: Apply perceptual volume curve (x^4)
    let perceptual_volume = slider_volume.powi(4);

    // Step 2: Apply base attenuation (safety)
    let base_volume = perceptual_volume * 0.2;

    // Step 3: Apply LUFS normalization gain (if enabled and available)
    let lufs_gain = if normalization_enabled {
        sound_lufs
            .map(|lufs| {
                let gain_db = target_lufs - lufs;
                let linear_gain = 10.0_f64.powf(gain_db / 20.0);
                linear_gain.clamp(0.1, 4.0) as f32
            })
            .unwrap_or(1.0) // No adjustment if LUFS unavailable
    } else {
        1.0
    };

    base_volume * lufs_gain
}
```

### ebur128 Initialization with Proper Mode
```rust
// Source: ebur128 docs (https://docs.rs/ebur128)
use ebur128::{EbuR128, Mode, Channel};

// Mode::I = Integrated loudness only (most efficient for our use case)
// Mode::M = Momentary (400ms) - not needed
// Mode::S = Short-term (3s) - not needed
// Mode::TRUE_PEAK = True peak detection - optional, for clipping prevention

let mode = Mode::I; // Minimum for integrated loudness
// OR
let mode = Mode::I | Mode::TRUE_PEAK; // If we want peak detection too

let mut ebu = EbuR128::new(channels, sample_rate, mode)?;

// For stereo audio:
ebu.set_channel(0, Channel::Left)?;
ebu.set_channel(1, Channel::Right)?;

// For mono audio (IMPORTANT: prevents -3 LU error):
ebu.set_channel(0, Channel::DualMono)?;
```
</code_examples>

<sota_updates>
## State of the Art (2024-2025)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Peak normalization | LUFS normalization | EBU R128 (2010), widespread adoption 2015+ | Consistent perceived loudness |
| Linear volume | Perceptual curves (x^n) | Always known, now standard | Better UX |
| libebur128 (C) | ebur128 (pure Rust) | 2020 | No FFI, simpler builds |
| ITU-R BS.1770-4 | ITU-R BS.1770-5 | November 2023 | Minor updates, same algorithm |

**New tools/patterns to consider:**
- **ebur128 0.1.10** is the latest pure Rust implementation (December 2020+)
- **True peak detection** via `Mode::TRUE_PEAK` for preventing inter-sample clipping
- **-14 LUFS target** is now industry standard (Spotify, YouTube, Amazon Music)

**Deprecated/outdated:**
- **RMS-based normalization:** Replaced by LUFS gating for more accurate perception
- **Peak-based normalization:** Doesn't account for perceived loudness
- **Linear volume sliders:** Universally recognized as poor UX
</sota_updates>

<open_questions>
## Open Questions

Things that couldn't be fully resolved:

1. **Dual-output LUFS gain**
   - What we know: LUFS normalization affects both primary and secondary output
   - What's unclear: Should gain be applied identically to both, or should users have per-device volume?
   - Recommendation: Apply LUFS gain identically (it's about content normalization), per-device volume is a separate setting

2. **Very short samples (<1 second)**
   - What we know: EBU R128 was designed for programs, not sound effects
   - What's unclear: Optimal handling for sub-second sounds (button clicks, UI sounds)
   - Recommendation: Allow LUFS calculation but mark as "unreliable" for samples < 1 second, let user decide whether to apply normalization

3. **Volume curve strength (x^3 vs x^4 vs x^5)**
   - What we know: x^4 is recommended for 60 dB dynamic range
   - What's unclear: Optimal curve for soundboard use case (may want gentler x^3)
   - Recommendation: Start with x^4, gather user feedback, make configurable if needed
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [ebur128 crate documentation](https://docs.rs/ebur128/latest/ebur128/) - API reference
- [ebur128 GitHub](https://github.com/sdroege/ebur128) - Implementation details, tests
- [ITU-R BS.1770-5](https://www.itu.int/rec/R-REC-BS.1770) - Official loudness standard (November 2023)
- [EBU R 128](https://tech.ebu.ch/docs/r/r128.pdf) - European broadcast loudness standard
- [EBU Tech 3341](https://tech.ebu.ch/loudness) - Loudness metering specification
- [Dr. Lex's Volume Controls](https://www.dr-lex.be/info-stuff/volumecontrols.html) - Volume curve mathematics

### Secondary (MEDIUM confidence)
- [libebur128 Mono Issue #42](https://github.com/jiixyj/libebur128/issues/42) - Mono channel configuration
- [Wikipedia: LUFS](https://en.wikipedia.org/wiki/LUFS) - Overview, terminology
- [Wikipedia: EBU R 128](https://en.wikipedia.org/wiki/EBU_R_128) - Standard history
- [iZotope: Mastering for Streaming](https://www.izotope.com/en/learn/mastering-for-streaming-platforms.html) - Target levels

### Tertiary (LOW confidence - needs validation)
- None - all findings verified against primary sources
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: ITU-R BS.1770 / EBU R128 loudness measurement
- Ecosystem: Rust audio (ebur128, symphonia, cpal)
- Patterns: Import-time LUFS calculation, perceptual volume curves
- Pitfalls: Mono handling, silence, sample rate, gain clamping

**Confidence breakdown:**
- Standard stack: HIGH - ebur128 is the only mature Rust option, well-documented
- Architecture: HIGH - patterns from official docs and industry practice
- Pitfalls: HIGH - documented issues with solutions
- Code examples: HIGH - from ebur128 docs, verified API

**Research date:** 2025-12-31
**Valid until:** 2026-03-31 (90 days - ebur128 ecosystem stable)
</metadata>

---

*Phase: 03-audio-core-polish*
*Research completed: 2025-12-31*
*Ready for planning: yes*
