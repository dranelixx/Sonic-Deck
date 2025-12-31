# Architecture

**Analysis Date:** 2025-12-29

## Pattern Overview

**Overall:** Desktop Application with Separated Frontend/Backend (Tauri v2)

**Key Characteristics:**
- React frontend communicates with Rust backend via IPC
- Dual audio output (monitor + broadcast) for streaming
- Zero-latency global hotkeys (handled in Rust, no IPC overhead)
- File-based persistence (JSON, no database)
- Fully offline operation

## Layers

**Presentation Layer (React):**
- Purpose: User interface and interaction
- Contains: Components, contexts, hooks, utilities
- Location: `src/`
- Depends on: Tauri IPC bridge (@tauri-apps/api)
- Used by: End users via Tauri WebView

**Command Layer (Tauri IPC):**
- Purpose: Bridge between frontend and backend
- Contains: Command handlers with `#[tauri::command]` attribute
- Location: `src-tauri/src/commands/`
- Depends on: State layer, Audio engine
- Used by: Frontend via `invoke()` calls

**Audio Engine Layer (Rust):**
- Purpose: Audio decoding, caching, playback
- Contains: AudioManager, decoder, cache, waveform generator
- Location: `src-tauri/src/audio/`
- Depends on: cpal, symphonia, lru crates
- Used by: Command layer, Hotkey handler

**State Layer (Rust):**
- Purpose: Thread-safe in-memory state management
- Contains: AppState with RwLock-protected data
- Location: `src-tauri/src/state.rs`
- Depends on: Persistence layer for loading/saving
- Used by: All command handlers, hotkey handler

**Persistence Layer (Rust):**
- Purpose: JSON file read/write with crash safety
- Contains: atomic_write(), load/save functions
- Location: `src-tauri/src/persistence.rs`, `sounds.rs`, `settings.rs`, `hotkeys.rs`
- Depends on: serde, serde_json
- Used by: State layer

## Data Flow

**Sound Playback (Button Click):**

1. User clicks SoundButton in `src/components/dashboard/SoundButton.tsx`
2. `useAudioPlayback` hook calls `invoke("play_dual_output", {...})`
3. IPC message sent to Rust backend
4. `commands/audio.rs::play_dual_output()` receives request
5. `AudioManager` checks cache, decodes if needed (`audio/decode.rs`)
6. Two cpal streams created for device1 + device2 (`audio/playback.rs`)
7. Playback runs in separate threads with volume control
8. `playback-progress` events emitted back to frontend
9. Waveform visualization updates in real-time

**Global Hotkey (Zero-Latency Path):**

1. User presses hotkey (e.g., Ctrl+1)
2. Tauri Global Shortcut Plugin triggers `handle_global_shortcut()` in `lib.rs`
3. Hotkey string normalized and looked up in `AppState.hotkeys`
4. Sound ID retrieved via read-lock (no write, ~0.1ms)
5. `play_dual_output()` called directly (NO IPC overhead)
6. Playback starts immediately (<10ms total latency)

**State Management:**
- In-memory: `Arc<RwLock<T>>` for thread-safe access
- Persistence: Atomic writes to JSON files on change
- Read locks for playback (concurrent reads allowed)
- Write locks for updates (exclusive, triggers save)

## Key Abstractions

**AudioManager:**
- Purpose: Coordinate playback lifecycle, manage cache
- Location: `src-tauri/src/audio/manager.rs`
- Pattern: Singleton-like (managed in Tauri state)
- Examples: `play()`, `stop()`, `stop_all()`, cache access

**AppState:**
- Purpose: Thread-safe global state container
- Location: `src-tauri/src/state.rs`
- Pattern: Arc<RwLock> for each domain (hotkeys, sounds, settings)
- Examples: `read_settings()`, `write_sounds()`, `read_hotkeys()`

**React Context Providers:**
- Purpose: Share state across component tree without prop drilling
- Location: `src/contexts/`
- Pattern: Context + Provider + custom hook
- Examples: `AudioContext`, `SettingsContext`, `SoundLibraryContext`

**DeviceId:**
- Purpose: Type-safe wrapper for audio device indices
- Location: `src-tauri/src/audio/mod.rs`
- Pattern: Newtype pattern with parsing methods
- Examples: `DeviceId::from_index(0)`, `device_id.index()`

**Type-Safe IDs:**
- Purpose: Prevent string confusion between IDs
- Location: `src-tauri/src/sounds.rs`
- Pattern: UUID-based wrapper types
- Examples: `SoundId::new()`, `CategoryId::new()`, `PlaybackId`

## Entry Points

**Frontend Entry:**
- Location: `src/main.tsx`
- Triggers: Tauri WebView load
- Responsibilities: Mount React app, initialize root component

**App Root:**
- Location: `src/App.tsx`
- Triggers: Component mount
- Responsibilities: Wrap with Context providers, render Dashboard/Settings

**Backend Entry:**
- Location: `src-tauri/src/main.rs`
- Triggers: App launch
- Responsibilities: Configure logging, parse CLI args, call `run()`

**App Initialization:**
- Location: `src-tauri/src/lib.rs::run()`
- Triggers: Called from main.rs
- Responsibilities: Register plugins, load state, setup hotkeys, create window

## Error Handling

**Strategy:** Errors bubble up to command handlers, logged, and returned to frontend

**Patterns:**
- Rust: `Result<T, E>` for fallible operations, `AudioError` enum for audio errors
- Frontend: try/catch around `invoke()`, Toast notifications for user feedback
- Logging: `tracing::error!()` before returning errors to frontend
- Events: `audio-decode-error` and `audio-playback-error` events for async errors

**Error Types:**
- `AudioError` - Decoding, device, playback failures (`src-tauri/src/audio/error.rs`)
- `String` - Simple error messages for commands
- Frontend catches all and shows Toast with user-friendly message

## Cross-Cutting Concerns

**Logging:**
- Framework: `tracing` crate with `tracing-subscriber`
- Location: `%LOCALAPPDATA%\com.sonicdeck.app\logs/sonicdeck.YYYY-MM-DD.log`
- Levels: trace (verbose) → debug → info → warn → error
- Format: Structured with timestamps and levels

**Validation:**
- Backend: Type-safe wrappers (DeviceId, SoundId), explicit error handling
- Frontend: TypeScript interfaces ensure correct data shapes
- File paths: No validation currently (potential security concern)

**Caching:**
- LRU cache for decoded audio (500MB default) - `src-tauri/src/audio/cache.rs`
- File modification time checked for cache invalidation
- Waveform data cached alongside audio samples

**Thread Safety:**
- `Arc<Mutex<T>>` for shared mutable state
- `Arc<RwLock<T>>` for read-heavy state (hotkeys lookup)
- Separate threads for each playback stream

---

*Architecture analysis: 2025-12-29*
*Update when major patterns change*
