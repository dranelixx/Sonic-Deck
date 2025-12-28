# AGENTS.md - SonicDeck Project Instructions

This file contains project-specific instructions for AI Agents when working on the SonicDeck codebase.

---

## Project Overview

SonicDeck is a high-performance desktop soundboard application built with:

- **Frontend**: React 19.2.3 + TypeScript 5.3.3 + Vite 7.3.0 + TailwindCSS 3.4.0
- **Backend**: Tauri v2 + Rust (cpal + symphonia for audio)
- **Key Features**: Dual-audio routing, sound library management, waveform visualization, audio trimming

### Tech Stack

#### Frontend

- **Build**: Vite 7.3.0, TypeScript 5.3.3, PostCSS + Autoprefixer
- **UI**: React 19.2.3, TailwindCSS 3.4.0, Emojibase 17.0.0 (emoji picker)
- **State**: React hooks (useState, useEffect, custom hooks) - state managed via React Context API
- **Quality**: ESLint 9.39.2, Prettier 3.7.4, Husky 9.1.7, lint-staged 16.2.7

#### Backend (Rust 2021)

- **Core**: Tauri 2.0, serde (serialization)
- **Audio**: cpal (I/O), symphonia (decoding: MP3, OGG/Vorbis, M4A/AAC via isomp4)
- **Logging**: tracing, tracing-subscriber, tracing-appender
- **Plugins**: tauri-plugin-dialog, tauri-plugin-shell, tauri-plugin-global-shortcut, tauri-plugin-autostart

---

## Code Style & Conventions

### TypeScript/React

- Functional components with hooks only (no classes)
- Const arrow functions for component definitions
- TypeScript strict mode - explicit types required
- **Naming**:
  - Components: PascalCase (`SoundButton.tsx`)
  - Hooks: camelCase with `use` prefix (`useAudioPlayback.ts`)
  - Types: PascalCase (`Sound`, `Category`)
  - Utils: kebab-case (`hotkeyDisplay.ts`)

### Rust

- `cargo fmt` standard formatting
- `snake_case` for functions/variables
- `///` doc comments for public functions
- `///` doc comments for complex private functions (>20 lines or non-trivial logic)
- Explicit error handling - avoid `unwrap()` in production
- Use `tracing` for logging (see Logging section)

### General

- Line length: 100 characters max (Prettier/ESLint enforced)
- Indentation: 2 spaces (TS/TSX), 4 spaces (Rust)
- **Conventional Commits** (enforced by pre-commit hooks):
  - `feat:` - New features
  - `fix:` - Bug fixes
  - `refactor:` - Code refactoring
  - `docs:` - Documentation
  - `chore:` - Maintenance
  - `test:` - Tests

- **Commit Message Format** (Problem/Solution structure):

  ```
  fix: short description

  Problem:
  - bullet point

  Solution:
  - bullet point

  Backend (Rust):
  - specific changes

  Frontend (React):
  - specific changes
  ```

  Omit Backend/Frontend sections if not applicable.

---

## Development Workflow

### Before Making Changes

1. Read existing code - understand current implementation
2. Check context: `git log --oneline -10`
3. Verify baseline: `yarn typecheck`
4. Rust compilation: `cargo check --manifest-path src-tauri/Cargo.toml`

### Commands

- `yarn dev` - Vite dev server (frontend only)
- `yarn tauri dev` - Full app with hot reload
- `yarn typecheck` - TypeScript validation
- `yarn lint` / `yarn lint:fix` - ESLint
- `yarn format` - Prettier formatting
- `yarn tauri build` - Production build

### Git Workflow

**Branching Strategy**: Git Flow Light with `develop` branch

- **Branch Types**: `feature/description`, `fix/description`, `chore/description`, `refactor/description`
- **Never commit directly to `main` or `develop`** - always use feature branches
- **Main Branches**:
  - `main` - Production-ready releases only
  - `develop` - Integration branch for ongoing work
- **Development Workflow**:
  1. Create feature/fix branch from `develop`
  2. Make changes, commit with conventional commits
  3. Push branch and create PR to `develop`
  4. **Set PR labels** for automatic release notes categorization:
     - `feature` / `enhancement` → ✨ Features
     - `bug` / `fix` → 🐛 Bug Fixes
     - `performance` → ⚡ Performance
     - `developer-tools` / `logging` / `debugging` → 🔧 Developer Experience
     - `documentation` → 📚 Documentation
     - `chore` / `refactor` → 🔨 Maintenance
  5. CI runs: Frontend + Rust checks (no Claude Review)
  6. Merge to `develop` after checks pass
- **Release Workflow**:
  1. When ready for release: Create PR from `develop` to `main`
  2. CI runs: Frontend + Rust checks + **Claude Code Review**
  3. After Claude Review approval: Add version bump commit to PR
  4. Update `version.json` in PR, commit: `chore: bump version to vX.Y.Z`
  5. Squash merge to `main` (includes version bump)
  6. Create tag (`git tag vX.Y.Z`), push tag (`git push --tags`)
- **Interactive Rebase**: `git rebase -i develop` before PR to organize/squash commits (especially if chaotic)
- **Pre-commit hooks**: Automatic lint, format, typecheck (Husky) on every commit
- **CI/CD**:
  - Frontend/Rust checks: Run on `main`, `develop`, `fix/**`, `feature/**`, `refactor/**` branches and PRs to `main`/`develop`
  - Claude Code Review: **Only** on PRs to `main` (saves runner minutes)
  - Release workflow: Triggered by version tags, auto-generates release notes from PR labels
  - Release notes: Configured via `.github/release.yml`, categorizes PRs by labels
- **Merge Strategy**:
  - PRs to `develop`: **Merge commit** (preserves full commit history)
  - PRs to `main` (releases): **Squash merge** (1 clean commit, includes version bump)
  - Version bump: Added to PR after Claude Code Review approval, before squash merge

---

## Version Management

**Single Source of Truth**: `version.json` (root directory)

**Key Concept**: SonicDeck uses a **dual-version system** for MSI compatibility:
- **Build Version**: Numeric format (e.g., `0.7.0-0`) for MSI installers
- **Display Version**: User-friendly format (e.g., `v0.7.0-alpha`) composed from `version` + `channel` fields

**Version Mapping**: `-0` = alpha, `-1` = beta, `-2` = rc, no suffix = stable

**Automation**: `scripts/sync-version.js` auto-syncs to `package.json`, `Cargo.toml`, `tauri.conf.json`

📖 **Full details**: See `VERSION.md` for complete workflow, examples, and code usage

---

## Architecture & Project Structure

### State Management

- **React Context API** - global state managed via Context providers
- Three main contexts: `AudioContext`, `SettingsContext`, `SoundLibraryContext`
- App.tsx wraps components with Context providers
- Components consume contexts via custom hooks (`useAudio`, `useSettings`, `useSoundLibrary`)
- Local state: `useState`/`useReducer` for component-specific needs
- Side effects: `useEffect` with proper cleanup and dependencies

### Tauri Commands (Frontend ↔ Backend)

- **Commands defined in**: `src-tauri/src/commands/` (modular structure)
- **Registration**: `.invoke_handler()` in `lib.rs`
- **Frontend usage**:

  ```typescript
  import { invoke } from '@tauri-apps/api/core';
  const result = await invoke<ReturnType>('command_name', { param: value });
  ```

- **Backend pattern**: Use `#[tauri::command]` attribute
- Always handle errors on both sides

### Audio System

- **Dual Output**: Simultaneous playback to primary + secondary devices
- **Caching**: LRU cache (500MB) for decoded audio + waveform data
- **Trim System**: Non-destructive - stores trim points, applies during playback
- **Threading**: Each playback runs in dedicated thread (parallel playback)
- **Pipeline**: File → Symphonia decode → Cache → cpal stream → Dual output

### File Structure

#### Frontend (`src/`)

```text
src/
├── components/
│   ├── audio/         # FullWaveform, MiniWaveform
│   ├── categories/    # CategoryTabs
│   ├── common/        # ErrorBoundary, Toast, EmojiPicker
│   ├── dashboard/     # Dashboard, DashboardHeader, DashboardSoundGrid, SoundButton
│   ├── modals/        # HotkeyManager, SoundModal, TrimEditor
│   └── settings/      # Settings, AudioDeviceSettings, PlaybackSettings,
│                      # SystemTraySettings, SettingsAbout
├── contexts/          # AudioContext, SettingsContext, SoundLibraryContext
├── hooks/             # useAudioPlayback, useFileDrop, useHotkeyMappings
├── utils/             # hotkeyDisplay, waveformQueue
├── App.tsx            # Root component (Context provider wrapper)
├── main.tsx           # Entry point
├── types.ts           # TypeScript type definitions
└── constants.ts       # Application constants (ANIMATION_DURATIONS, DEBUG)
```

#### Backend (`src-tauri/src/`)

```text
src-tauri/src/
├── audio/             # Audio module
│   ├── manager.rs     # AudioManager (cache, playback coordination)
│   ├── playback.rs    # Playback engine (cpal streams)
│   ├── device.rs      # Device enumeration
│   ├── decode.rs      # Symphonia integration
│   ├── cache.rs       # LRU cache
│   ├── waveform.rs    # Waveform generation
│   └── error.rs       # Error types
├── commands/          # Modular command structure
│   ├── mod.rs         # Module exports
│   ├── audio.rs       # Audio-related commands
│   ├── hotkeys.rs     # Hotkey commands
│   ├── logs.rs        # Logging commands
│   ├── settings.rs    # Settings commands
│   └── sounds.rs      # Sound library commands
├── lib.rs             # App setup and command registration
├── main.rs            # App entry point
├── hotkeys.rs         # Global hotkey management
├── settings.rs        # Settings persistence
├── sounds.rs          # Sound library management
└── tray.rs            # System tray
```

#### Configuration

- `src-tauri/tauri.conf.json` - Tauri app config
- `src-tauri/capabilities/main-capability.json` - Tauri v2 permissions
- `vite.config.ts`, `tsconfig.json`, `tailwind.config.js`, `eslint.config.js`, `.prettierrc`
- `version.json` - **Centralized Version (NEW v0.7.0+)**

---

## Logging & Debugging

### Frontend Logging

- `console.log/warn/error` for debugging
- Set `DEBUG = true` in `src/constants.ts` for development mode
- Chrome DevTools available (Tauri uses Chromium)

### Rust Logging (CRITICAL - Uses `tracing`, NOT `log`)

- **Location**: `%LOCALAPPDATA%\com.sonicdeck.app\logs\sonicdeck.YYYY-MM-DD.log`
- **Daily rotation**: Last 7 days retained
- **Usage**:

  ```rust
  use tracing::{info, warn, error, debug, trace};

  info!("User initiated playback for sound: {}", sound_id);
  warn!("Device {} not found, using default", device_name);
  error!("Failed to decode audio: {:?}", err);
  debug!("Detailed flow info");
  trace!("Very verbose debugging");
  ```

- **Levels**:
  - `info`: User actions, successful operations
  - `warn`: Recoverable issues, fallbacks
  - `error`: Failures, critical issues
  - `debug`: Detailed flow (dev only)
  - `trace`: Very verbose (dev only)

### Debugging Tools

- **Verbose logging**: `RUST_LOG=debug yarn tauri dev`
- **App-specific**: `RUST_LOG=sonicdeck=trace yarn tauri dev`
- **Quick debug**: `dbg!()` macro (remove before commit)
- **Compilation check**: `cargo check`

### Audio Debugging

- Verify device selection, supported formats (MP3, OGG/Vorbis, M4A/AAC)
- Check cpal stream status, buffer underruns in logs
- Test dual-output configuration with different devices

---

## Backlog & Project Management

**TriliumNext Backlog System**:
- **Now/Next/Later** (C3uScCyjwu6O): Current sprint + upcoming work
- **High Priority** (ZTLmPPFwSqAF): Critical features
- **Medium Priority** (lS0pzzvavT0d): Standard features
- **Low Priority** (noVVr31giHQv): Nice-to-have features
- **Done Archive** (tlr8Wbb82vAK): Completed work

When starting new work, check the "Now/Next/Later" note first for current priorities.

---

## Known Issues & Roadmap

**Known Limitations**:
- No automated tests yet (manual testing required)
- Format support limited to symphonia (MP3, OGG/Vorbis, M4A/AAC)
- Global hotkeys may conflict with other apps
- Waveform generation CPU-intensive for large files

**Focus Areas for Testing**: Audio playback, device switching, file import, trim editor, hotkeys

---

## Common Tasks

### Add Tauri Command

1. Add function with `#[tauri::command]` in appropriate `src-tauri/src/commands/*.rs` file
   - Audio commands → `commands/audio.rs`
   - Settings commands → `commands/settings.rs`
   - Sound library commands → `commands/sounds.rs`
   - Hotkey commands → `commands/hotkeys.rs`
   - Logging commands → `commands/logs.rs`
2. Export function in `commands/mod.rs` via `pub use module_name::function_name;`
3. Register in `lib.rs` via `.invoke_handler(tauri::generate_handler![...])`
4. Add TypeScript types in `src/types.ts`
5. Use `invoke()` from frontend

### Add Sound Property

1. Update `Sound` interface in `src/types.ts`
2. Update Rust `Sound` struct in `src-tauri/src/sounds.rs`
3. Ensure serde serialization works
4. Update UI components
5. Update settings persistence

### Modify Audio Playback

- Changes in `src-tauri/src/audio/` must handle:
  - Dual device output
  - Volume control (primary/secondary)
  - Trim boundaries
  - Error handling (device disconnect, decode failure)
  - Proper logging

### Add Audio Format

1. Update decoder in `src-tauri/src/audio/decode.rs`
2. Add symphonia features in `Cargo.toml` if needed
3. Update file dialogs to show new format
4. Test thoroughly

### Add Hotkey Action

1. Define action in `src-tauri/src/hotkeys.rs`
2. Implement handler in hotkey event listener
3. Add UI in `src/components/modals/HotkeyManager.tsx`
4. Update `useHotkeyMappings.ts` and `src/utils/hotkeyDisplay.ts`

### Modify UI Styling

- Prefer TailwindCSS utility classes
- Add to `src/index.css` only if absolutely necessary
- Use `tailwind.config.js` for theme customization
- Follow Discord-inspired dark theme
- Use `ANIMATION_DURATIONS` from `constants.ts`

---

## Performance & Best Practices

### Frontend Performance

- Use `useMemo`, `useCallback` to prevent re-renders
- `React.memo()` for expensive components
- Debounce expensive operations (search, waveform)
- Waveform queue prevents UI blocking

### Backend Performance

- Audio decode in background threads (never block main)
- LRU cache (500MB) prevents redundant decoding
- Efficient data structures (HashMap for lookups, Vec for iteration)
- Minimize IPC calls
- Pre-compute waveforms during decode

### User Experience

- Loading states for async operations
- Clear error messages via Toast
- Consistent animation timing (`ANIMATION_DURATIONS`)
- Drag-and-drop support
- Keyboard navigation

### Accessibility

- ARIA labels for screen readers
- Keyboard-only navigation support
- Semantic HTML
- Sufficient color contrast

### Security

- Tauri v2 capabilities system (`src-tauri/capabilities/main-capability.json`)
- Validate all user inputs (frontend + backend)
- No telemetry - completely offline
- Local-only data storage

---

## Testing & Quality Assurance

### Pre-Commit Checklist (Auto-enforced by Husky)

1. `yarn typecheck` - No TypeScript errors
2. `yarn lint` - No ESLint errors
3. `cargo check --manifest-path src-tauri/Cargo.toml` - Rust compiles
4. `yarn tauri dev` - App runs
5. Manual testing of changed functionality

### Testing Resources

- `docs/TESTING_GUIDE_EN.html` / `docs/TESTING_GUIDE_DE.html`
- Windows audio setup references in `docs/` (playback, recording, microphone)
- Discord integration screenshots in `docs/`

---

## Dependencies

### Adding Dependencies

- **Frontend**: `yarn add <package>` (prod) or `yarn add -D <package>` (dev)
- **Backend**: Add to `src-tauri/Cargo.toml`
- Check license compatibility (prefer MIT/Apache-2.0)

### Updating

- `yarn upgrade-interactive` (frontend)
- `cargo update` (backend)
- Test thoroughly after updates (especially Tauri, React, audio libs)

---

## Resources

- **Tauri**: <https://v2.tauri.app/>
- **React**: <https://react.dev/>
- **TailwindCSS**: <https://tailwindcss.com/docs>
- **Rust**: <https://doc.rust-lang.org/book/>
- **cpal**: <https://docs.rs/cpal/>
- **symphonia**: <https://docs.rs/symphonia/>
- **Tauri Security**: <https://v2.tauri.app/security/>

---

## Contact

- GitHub Issues: <https://github.com/DraneLixX/SonicDeck/issues>
- Contact: Adrian Konopczynski (<adrikonop@gmail.com>)
- Review git history: `git log --all --grep="keyword"`
