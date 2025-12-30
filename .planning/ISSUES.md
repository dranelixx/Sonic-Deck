# Deferred Issues

Issues discovered during execution, logged for future consideration.

## Open Issues

### ISS-001: Microphone routing sample rate mismatch handling
**GitHub:** [#82](https://github.com/dranelixx/SonicDeck/issues/82)
**Discovered:** Phase 2, Plan 4 (Microphone Routing)
**Priority:** Medium
**Type:** Enhancement

**Description:**
Current microphone routing uses input device sample rate for output stream. If CABLE Input expects a different sample rate, audio quality could degrade (pitch issues, artifacts).

**Proposed solution:**
- Detect sample rate mismatch between input and output devices
- Implement resampling (e.g., using `rubato` crate) when needed
- Or: Force both streams to common rate (48kHz)

**Files:** `src-tauri/src/vbcable/microphone.rs`

---

### ISS-003: Microphone routing buffer synchronization
**GitHub:** [#84](https://github.com/dranelixx/SonicDeck/issues/84)
**Discovered:** Phase 2, Plan 4 (Microphone Routing)
**Priority:** Low
**Type:** Enhancement

**Description:**
Current ring buffer implementation is simple and may suffer from underruns or overruns.

**Proposed solution:**
- Implement lock-free ring buffer (e.g., `ringbuf` crate)
- Add fill level tracking
- Handle underrun/overrun gracefully

**Files:** `src-tauri/src/vbcable/microphone.rs`

---

### ISS-004: Microphone routing device disconnect handling
**GitHub:** [#85](https://github.com/dranelixx/SonicDeck/issues/85)
**Discovered:** Phase 2, Plan 4 (Microphone Routing)
**Priority:** Medium
**Type:** Enhancement

**Description:**
If the microphone or VB-Cable device is disconnected while routing is active, the application may crash or fail silently.

**Proposed solution:**
- Detect device disconnect in stream error callbacks
- Show toast notification to user
- Optionally: Auto-retry when device reconnects

**Files:** `src-tauri/src/vbcable/microphone.rs`

---

### ISS-005: Microphone routing volume control
**GitHub:** [#86](https://github.com/dranelixx/SonicDeck/issues/86)
**Discovered:** Phase 2, Plan 4 (Microphone Routing)
**Priority:** Low
**Type:** Enhancement

**Description:**
No way to adjust the volume of the routed microphone audio.

**Proposed solution:**
- Add gain slider in VbCableSettings (0-200% range)
- Apply gain multiplier in the ring buffer
- Persist setting

**Files:** `src-tauri/src/vbcable/microphone.rs`, `src/components/settings/VbCableSettings.tsx`

---

### ISS-006: Microphone routing activity indicator
**GitHub:** [#87](https://github.com/dranelixx/SonicDeck/issues/87)
**Discovered:** Phase 2, Plan 4 (Microphone Routing)
**Priority:** Low
**Type:** Enhancement

**Description:**
Users cannot see if audio is actually flowing through the microphone routing.

**Proposed solution:**
- Add small VU meter or pulsing indicator in VbCableSettings
- Expose audio level from backend

**Files:** `src-tauri/src/vbcable/microphone.rs`, `src/components/settings/VbCableSettings.tsx`

---

### ISS-007: VbCableError enum for better error handling
**GitHub:** [#93](https://github.com/dranelixx/SonicDeck/issues/93)
**Discovered:** Code Review PR #92
**Priority:** Medium
**Type:** Enhancement
**Target:** v1.0-Beta

**Description:**
VB-Cable module uses `String` errors in many places. This makes error handling less precise and pattern matching difficult.

**Proposed solution:**
- Create dedicated `VbCableError` enum using `thiserror` crate
- Replace all String errors with typed variants
- Benefits: type-safe handling, better pattern matching, consistent errors

**Files:** `src-tauri/src/vbcable/*.rs`

---

## Resolved Issues

### ISS-002: Microphone routing latency optimization ✓
**GitHub:** [#83](https://github.com/dranelixx/SonicDeck/issues/83)
**Resolved:** Phase 2, Latency Fix Commit (c5e6fea)

**Solution implemented:**
- Reduced buffer from 1s to 100ms
- Added buffer prefill to prevent startup glitches

---

*Last updated: 2025-12-31*
