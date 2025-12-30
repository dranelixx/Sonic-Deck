# Deferred Issues

Issues discovered during execution, logged for future consideration.

## Open Issues

### ISS-001: Microphone routing sample rate mismatch handling

**Discovered:** Phase 2, Plan 4 (Microphone Routing)
**Priority:** Medium
**Type:** Enhancement

**Description:**
Current microphone routing uses input device sample rate for output stream. If CABLE Input expects a different sample rate, audio quality could degrade (pitch issues, artifacts).

**Current behavior:**
- Takes sample rate from microphone's default config
- Uses same rate for CABLE Input output stream
- No resampling if rates don't match

**Proposed solution:**
- Detect sample rate mismatch between input and output devices
- Implement resampling (e.g., using `rubato` crate) when needed
- Or: Force both streams to common rate (48kHz)

**Files:** `src-tauri/src/vbcable/microphone.rs`

---

### ISS-002: Microphone routing latency optimization

**Discovered:** Phase 2, Plan 4 (Microphone Routing)
**Priority:** Low
**Type:** Enhancement

**Description:**
Current ring buffer is sized for 1 second at 48kHz stereo (96000 samples). This may introduce noticeable latency in voice communication.

**Current behavior:**
- Ring buffer: 48000 * 2 = 96000 samples
- At 48kHz stereo, this is ~1 second of audio
- Latency could be perceptible in real-time conversation

**Proposed solution:**
- Reduce buffer size to ~50-100ms (4800-9600 samples at 48kHz)
- Implement proper buffer fill level monitoring
- Add latency configuration option in settings

**Files:** `src-tauri/src/vbcable/microphone.rs`

---

### ISS-003: Microphone routing buffer synchronization

**Discovered:** Phase 2, Plan 4 (Microphone Routing)
**Priority:** Medium
**Type:** Enhancement

**Description:**
Current ring buffer implementation is simple and may suffer from underruns (output reads faster than input writes) or overruns (input writes faster than output reads).

**Current behavior:**
- Simple ring buffer with write_pos and read_pos
- No synchronization between producer (input) and consumer (output)
- No detection of buffer underrun/overrun conditions

**Proposed solution:**
- Implement lock-free ring buffer (e.g., `ringbuf` crate)
- Add fill level tracking
- Handle underrun gracefully (output silence, log warning)
- Handle overrun gracefully (drop oldest samples, log warning)

**Files:** `src-tauri/src/vbcable/microphone.rs`

---

## Resolved Issues

(None yet)

---

*Last updated: 2025-12-30*
