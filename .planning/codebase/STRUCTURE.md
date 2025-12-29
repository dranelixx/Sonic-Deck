# Codebase Structure

**Analysis Date:** 2025-12-29

## Directory Layout

```
SonicDeck/
├── src/                          # Frontend (React + TypeScript)
│   ├── components/              # UI components (feature-based)
│   │   ├── audio/              # Waveform visualization
│   │   ├── categories/         # Category tabs
│   │   ├── common/             # Shared components
│   │   ├── dashboard/          # Main sound grid UI
│   │   ├── modals/             # Dialog components
│   │   └── settings/           # Settings panel
│   ├── contexts/               # React Context providers
│   ├── hooks/                  # Custom React hooks
│   ├── test/                   # Test setup and mocks
│   └── utils/                  # Utility functions
├── src-tauri/                   # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── audio/             # Audio engine
│   │   └── commands/          # Tauri IPC handlers
│   └── tests/                 # Integration tests
├── scripts/                     # Build scripts
├── docs/                        # Documentation
└── .github/                     # CI/CD workflows
```

## Directory Purposes

**src/components/**
- Purpose: All React UI components
- Contains: `.tsx` files organized by feature
- Key files: `SoundButton.tsx`, `Dashboard.tsx`, `FullWaveform.tsx`
- Subdirectories:
  - `audio/` - FullWaveform.tsx, MiniWaveform.tsx
  - `categories/` - CategoryTabs.tsx
  - `common/` - ErrorBoundary.tsx, Toast.tsx, EmojiPicker.tsx
  - `dashboard/` - Dashboard.tsx, DashboardHeader.tsx, DashboardSoundGrid.tsx, SoundButton.tsx
  - `modals/` - SoundModal.tsx, TrimEditor.tsx, HotkeyManager.tsx
  - `settings/` - Settings.tsx, AudioDeviceSettings.tsx, PlaybackSettings.tsx, SystemTraySettings.tsx, SettingsAbout.tsx

**src/contexts/**
- Purpose: React Context API for global state
- Contains: Context providers with custom hooks
- Key files:
  - `AudioContext.tsx` - Audio device state
  - `SettingsContext.tsx` - App settings
  - `SoundLibraryContext.tsx` - Sounds and categories

**src/hooks/**
- Purpose: Reusable React hooks
- Contains: Custom hooks for complex logic
- Key files:
  - `useAudioPlayback.ts` - Playback control logic
  - `useHotkeyMappings.ts` - Hotkey management
  - `useFileDrop.ts` - Drag-and-drop handling

**src/utils/**
- Purpose: Pure utility functions
- Contains: Helpers without React dependencies
- Key files:
  - `hotkeyDisplay.ts` - Format hotkey strings for display
  - `waveformQueue.ts` - Async waveform generation queue

**src-tauri/src/audio/**
- Purpose: Complete audio processing engine
- Contains: Decoding, caching, playback, waveform
- Key files:
  - `mod.rs` - Public exports (DeviceId, AudioData, AudioDevice)
  - `manager.rs` - AudioManager lifecycle coordination
  - `playback.rs` - cpal stream creation
  - `decode.rs` - Symphonia audio decoding
  - `cache.rs` - LRU cache (500MB)
  - `device.rs` - Device enumeration
  - `waveform.rs` - Peak calculation
  - `error.rs` - AudioError enum

**src-tauri/src/commands/**
- Purpose: Tauri IPC command handlers
- Contains: Functions with `#[tauri::command]`
- Key files:
  - `mod.rs` - Re-exports all commands
  - `audio.rs` - play_dual_output, stop_playback, list_audio_devices
  - `sounds.rs` - load_sounds, add_sound, update_sound, delete_sound
  - `settings.rs` - load_settings, save_settings
  - `hotkeys.rs` - load_hotkeys, save_hotkeys, register/unregister
  - `logs.rs` - get_log_directory, open_log_directory

**src-tauri/src/** (root)
- Purpose: App initialization and persistence
- Contains: Entry points, state management, persistence
- Key files:
  - `main.rs` - CLI entry point, logging setup
  - `lib.rs` - Tauri app initialization, plugin setup
  - `state.rs` - AppState with RwLock containers
  - `sounds.rs` - Sound/Category structs, persistence
  - `settings.rs` - AppSettings struct, persistence
  - `hotkeys.rs` - HotkeyMappings struct, persistence
  - `persistence.rs` - Atomic file write utility
  - `tray.rs` - System tray icon

## Key File Locations

**Entry Points:**
- `src/main.tsx` - React app mount point
- `src/App.tsx` - Root component with Context wrappers
- `src-tauri/src/main.rs` - Rust entry, logging setup
- `src-tauri/src/lib.rs` - Tauri app setup, `run()` function

**Configuration:**
- `package.json` - Frontend dependencies, scripts
- `tsconfig.json` - TypeScript compiler options
- `vite.config.ts` - Vite bundler config
- `vitest.config.ts` - Test runner config
- `eslint.config.js` - ESLint rules
- `.prettierrc` - Prettier formatting
- `src-tauri/Cargo.toml` - Rust dependencies
- `src-tauri/tauri.conf.json` - Tauri app config
- `src-tauri/capabilities/main-capability.json` - Tauri v2 permissions
- `version.json` - Centralized version number

**Core Logic:**
- `src/hooks/useAudioPlayback.ts` - Frontend playback control
- `src-tauri/src/audio/manager.rs` - Backend playback coordination
- `src-tauri/src/commands/audio.rs` - Audio IPC commands
- `src-tauri/src/lib.rs` - Hotkey handler (zero-latency path)

**Testing:**
- `src/test/setup.ts` - Vitest setup, Tauri mocks
- `src/utils/*.test.ts` - Frontend unit tests
- `src-tauri/tests/` - Rust integration tests
- `src-tauri/tests/fixtures/` - Test audio files

**Documentation:**
- `README.md` - User documentation
- `AGENTS.md` - AI agent instructions (symlinked in CLAUDE.md)
- `CONTRIBUTING.md` - Contribution guidelines
- `VERSION.md` - Version management docs
- `docs/` - Additional guides and screenshots

## Naming Conventions

**Files:**
- PascalCase.tsx - React components (`SoundButton.tsx`, `Dashboard.tsx`)
- camelCase.ts - Hooks with `use` prefix (`useAudioPlayback.ts`)
- kebab-case.ts - Utilities (`hotkeyDisplay.ts`)
- UPPERCASE.md - Important docs (`README.md`, `CLAUDE.md`)
- snake_case.rs - Rust modules (`audio/manager.rs`)

**Directories:**
- kebab-case - All frontend directories (`src/components/`)
- snake_case - Rust directories (`src-tauri/src/audio/`)
- Plural for collections - `components/`, `contexts/`, `hooks/`

**Special Patterns:**
- `*.test.ts` - Test files co-located with source
- `mod.rs` - Rust module re-exports
- `index.ts` - Not used (explicit imports preferred)

## Where to Add New Code

**New React Component:**
- Implementation: `src/components/{feature}/{ComponentName}.tsx`
- Tests: `src/components/{feature}/{ComponentName}.test.tsx` (if needed)
- Export: Import directly where used

**New Custom Hook:**
- Implementation: `src/hooks/use{HookName}.ts`
- Tests: `src/hooks/use{HookName}.test.ts`
- Pattern: Return object with values and functions

**New Tauri Command:**
- Implementation: `src-tauri/src/commands/{domain}.rs`
- Export: Add to `commands/mod.rs`
- Register: Add to `lib.rs` `generate_handler![]`
- Types: Add TypeScript interface in `src/types.ts`
- Usage: `invoke<ReturnType>("command_name", { params })`

**New Audio Feature:**
- Implementation: `src-tauri/src/audio/{feature}.rs`
- Export: Add to `audio/mod.rs`
- Tests: Add `#[cfg(test)] mod tests { ... }` inline

**New Utility:**
- Frontend: `src/utils/{utility-name}.ts`
- Backend: `src-tauri/src/{utility}.rs`
- Tests: Co-located `*.test.ts` or inline `#[cfg(test)]`

**New Context:**
- Implementation: `src/contexts/{Domain}Context.tsx`
- Pattern: createContext + Provider + useHook
- Integration: Wrap in `App.tsx`

## Special Directories

**node_modules/**
- Purpose: Frontend dependencies (auto-generated)
- Source: `yarn install`
- Committed: No (gitignored)

**target/**
- Purpose: Rust build artifacts
- Source: `cargo build`
- Committed: No (gitignored)

**dist/**
- Purpose: Vite production build output
- Source: `yarn build`
- Committed: No (gitignored)

**.planning/**
- Purpose: Project planning documents (this directory)
- Source: Created by GSD commands
- Committed: Yes

**scripts/**
- Purpose: Build automation scripts
- Key files: `sync-version.js` - Version synchronization
- Committed: Yes

---

*Structure analysis: 2025-12-29*
*Update when directory structure changes*
