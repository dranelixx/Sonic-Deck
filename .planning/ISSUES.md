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

### ISS-008: SHA256 checksum verification for VB-Cable download
**GitHub:** [#94](https://github.com/dranelixx/SonicDeck/issues/94)
**Discovered:** Code Review PR #92
**Priority:** Low
**Type:** Enhancement (Security)

**Description:**
VB-Cable installer downloaded via HTTPS without checksum verification. Adding SHA256 verification would provide defense-in-depth.

**Proposed solution:**
- Add `sha2` crate dependency
- Verify downloaded ZIP against known hash
- Consider: VB-Audio doesn't publish official checksums

**Files:** `src-tauri/src/vbcable/installer.rs`

---

### ISS-009: Improve ring buffer overflow logging with rate limiting
**GitHub:** [#95](https://github.com/dranelixx/SonicDeck/issues/95)
**Discovered:** Code Review PR #92
**Priority:** Low
**Type:** Enhancement

**Description:**
Current `overflow_logged` flag prevents repeat warnings but hides persistent problems. Only logs once, even if overflow continues.

**Proposed solution:**
- Replace single-shot flag with rate-limited logging (every 5 seconds)
- Persistent problems remain visible in logs
- Alternative: Reset flag when routing re-enabled

**Files:** `src-tauri/src/vbcable/microphone.rs`

---

### ISS-010: Frontend mock layer for Vite-only development
**GitHub:** [#97](https://github.com/dranelixx/SonicDeck/issues/97)
**Discovered:** Development workflow improvement
**Priority:** Low
**Type:** Enhancement (DX)

**Description:**
When developing frontend using Vite standalone (VSCode Vite extension), Tauri backend features are unavailable. All `invoke()` calls fail, making UI development slower.

**Proposed solution:**
- Create mock layer that detects `window.__TAURI__` absence
- Return fake data for audio devices, VB-Cable status, etc.
- Enable faster UI iteration without full Tauri rebuild

**Files:** `src/utils/tauriMock.ts` (new), all components using `invoke()`

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
