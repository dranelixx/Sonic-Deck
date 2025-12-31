# Codebase Concerns

**Analysis Date:** 2025-12-29

## Tech Debt

**No TODO/FIXME Comments Found:**
- The codebase is clean of technical debt markers
- No HACK, XXX, or BUG comments detected
- Well-maintained code quality

## Known Bugs

**Race Condition in File Access (Issue #51):**
- Problem: Concurrent file access can cause corruption
- Files: `src-tauri/src/persistence.rs`, `src-tauri/src/sounds.rs`, `src-tauri/src/settings.rs`
- Status: Tracked in issue, milestone: Alpha Stability
- Current mitigation: Atomic file operations (temp-file-rename)
- Missing: File locking, multi-instance detection

**Hotkey Edge Cases (Issue #35):**
- Problem: Hotkey conflicts with other applications, modifier key edge cases
- File: `src-tauri/src/lib.rs` (hotkey handling)
- Status: Tracked in issue, milestone: Alpha Stability
- Current mitigation: Basic normalization
- Missing: Conflict detection, fallback behavior, clear error messages

## Security Considerations

**Path Traversal Risk - No Input Validation:**
- Risk: File paths passed to audio decoder are not sanitized
- Files: `src-tauri/src/audio/decode.rs:19-23`, `src-tauri/src/audio/cache.rs:69-70`
- Current mitigation: None
- Recommendations:
  - Normalize paths with `canonicalize()` before use
  - Validate paths against allowed directories
  - Reject paths with `..` or absolute paths outside app scope

**No File Size Limits:**
- Risk: Extremely large audio files could cause memory exhaustion
- Files: `src-tauri/src/audio/decode.rs`
- Current mitigation: LRU cache eviction (500MB limit)
- Recommendations: Add pre-decode file size check (e.g., max 100MB)

**Hotkey String Validation Missing:**
- Risk: Extremely long hotkey strings could cause memory issues
- File: `src-tauri/src/lib.rs:25-51` (`normalize_hotkey_string`)
- Current mitigation: None
- Recommendations: Add length limit, validate against known modifier keys

## Performance Bottlenecks

**Device Enumeration on Every Playback (Issue #38):**
- Problem: Device enumeration happens on every playback, adds 400-500ms latency
- Files: `src-tauri/src/audio/device.rs`, `src-tauri/src/commands/audio.rs`
- Status: Tracked in issue, milestone: v1.0 Beta
- Improvement path: Cache device list, invalidate on device change events

**First-Time Audio Decode:**
- Problem: Initial playback of uncached sounds requires full decode
- Files: `src-tauri/src/audio/decode.rs`, `src-tauri/src/audio/cache.rs`
- Measurement: Up to 500ms for 5-minute MP3 files
- Cause: Symphonia decode is synchronous, blocks first play
- Improvement path: Pre-decode on sound import, background warm-up

**Waveform Generation:**
- Problem: Waveform calculation CPU-intensive for large files
- Files: `src-tauri/src/audio/waveform.rs`
- Measurement: Up to 2s for 10-minute audio files
- Cause: Full file scan required for peak calculation
- Improvement path: Cache waveform data to disk, progressive loading

## Fragile Areas

**RwLock Poisoning Not Gracefully Handled:**
- File: `src-tauri/src/state.rs:48-83`
- Why fragile: Uses `.expect()` which panics if lock is poisoned
- Common failures: If any thread panics while holding lock, all subsequent accesses fail
- Safe modification: Use `.unwrap_or_else(|e| e.into_inner())` to recover from poisoning
- Test coverage: No tests for poisoned lock scenarios

**Audio Device Enumeration Silently Skips Devices:**
- File: `src-tauri/src/audio/device.rs:22-32`
- Why fragile: Devices without names are silently ignored
- Common failures: Device index mismatch if some devices skipped
- Safe modification: Log skipped devices, track actual indices
- Test coverage: Unit tests exist but don't cover edge cases

**TOCTOU Race Condition in Cache Validation:**
- File: `src-tauri/src/audio/cache.rs:74-79`
- Why fragile: File modification time checked, then file read separately
- Common failures: File changed between check and read → stale cache
- Safe modification: Lock file during validation or use file hash
- Test coverage: No tests for concurrent modification

## Scaling Limits

**Sound Library Size:**
- Current capacity: Unlimited sounds in `sounds.json`
- Limit: JSON parsing becomes slow at thousands of sounds
- Symptoms at limit: Slow app startup, UI lag
- Scaling path: Add pagination, consider SQLite for large libraries

**LRU Cache Size:**
- Current capacity: 500MB in-memory cache
- Limit: Memory exhaustion on low-RAM systems
- Symptoms at limit: OOM errors, system slowdown
- Scaling path: Make cache size configurable, add disk-backed cache

## Dependencies at Risk

**No High-Risk Dependencies Detected:**
- All major dependencies actively maintained
- cpal and symphonia are stable audio libraries
- Tauri 2.0 is production-ready

## Missing Critical Features

**No Default Device Fallback (Issue #32):**
- Problem: If configured audio device disconnected, playback fails
- Files: `src-tauri/src/commands/audio.rs:225-235`
- Status: Tracked in issue, milestone: Alpha Stability
- Current workaround: User must reconfigure settings
- Blocks: Seamless audio device hot-plugging
- Implementation complexity: Medium (add fallback logic, retry with backoff)

**No Sound Library Size Limit:**
- Problem: Unlimited sounds can cause performance issues
- Files: `src-tauri/src/sounds.rs`, `src-tauri/src/commands/sounds.rs`
- Current workaround: None
- Blocks: Reliable performance with large libraries
- Implementation complexity: Low (add count check in add_sound)

## Test Coverage Gaps

**Frontend Test Coverage Low (Issue #75):**
- What's not tested: Most React components, context providers
- Status: Tracked in issue, milestone: Alpha Stability
- Current: 6% coverage, Target: 30%
- Priority components: SoundButton, CategoryTabs, AudioDeviceSettings
- Difficulty: Requires mocking Tauri and audio contexts

**Rust Unit Test Gaps (Issue #77):**
- What's not tested: Edge cases in cache, settings, hotkeys
- Status: Tracked in issue, milestone: Alpha Stability
- Current: 48% coverage, Target: 55%
- Priority modules: cache.rs (72%→90%), settings.rs (76%→90%), hotkeys.rs (83%→95%)

**No Integration Tests for Hotkey Flow (Issue #37):**
- What's not tested: Full hotkey registration → playback path
- Files: `src-tauri/src/lib.rs` (handle_global_shortcut)
- Risk: Hotkey system could break silently
- Priority: High
- Difficulty: Requires simulating global shortcuts

**No Tests for Concurrent Playback:**
- What's not tested: Multiple sounds playing simultaneously
- Files: `src-tauri/src/audio/manager.rs`, `src-tauri/src/commands/audio.rs`
- Risk: Race conditions in multi-sound playback
- Priority: Medium
- Difficulty: Requires thread synchronization testing

## Positive Observations

The codebase is generally well-structured:
- No SQL injection (no database)
- No hardcoded secrets
- No telemetry (offline-first design)
- Tauri v2 capabilities properly configured
- Atomic writes prevent data corruption
- Good test coverage for core audio logic (45% Rust threshold)
- Clean code with no TODO/FIXME markers

---

*Concerns audit: 2025-12-29*
*Update as issues are fixed or new ones discovered*
