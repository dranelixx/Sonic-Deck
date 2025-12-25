# Changelog

All notable changes to Sonic Deck are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0-alpha] - 2025-12-24

### Added
- Centralized version management system (single source of truth)
- Dynamic version display in app via `VITE_APP_VERSION` environment variable
- Pre-commit hook for automatic version synchronization across config files
- `sync-version.js` script for maintaining version consistency
- `VERSION.md` documentation for version management workflow

### Changed
- Build process now automatically syncs versions in `package.json`, `Cargo.toml`, and `tauri.conf.json`
- Build scripts (`yarn build`, `yarn tauri:dev`, `yarn tauri:build`) now include automatic version sync

### Fixed
- Add comprehensive error handling (try-catch) to `sync-version.js` for robust file I/O
- Fix TOML regex to target only `[package]` section, preventing accidental dependency version changes
- Fix `vite.config.ts` path bug (`../version.json` → `./version.json`)
- Fix MSI build failure by implementing dual-version system (numeric build version for MSI compatibility)

### Technical
- Implement ESM-compatible version sync script with helper functions
- Add `VITE_APP_VERSION` and `VITE_APP_CHANNEL` injection in `vite.config.ts`
- Implement dual-version system: numeric build version (MSI-compatible) + alphanumeric display version (user-friendly)
- Add smart version display logic in `SettingsAbout.tsx` (converts `0.7.0-0` + `alpha` → `v0.7.0-alpha`)
- Update pre-commit hooks to stage synced version files automatically
- Replace hardcoded version in `SettingsAbout.tsx` with environment variable

### Known Issues
- Hotkey multi-trigger on rapid key press (Fix planned for v0.7.3 or v0.8.0)
- UI click has artificial delay (Fix planned for v0.7.3)
- System tray close behavior inconsistent (Fix planned for v0.7.3)

---

## [0.6.0-beta] - 2025-12-21 (Previous Release)

### Summary
- First public beta release with core audio engine and hotkey system
- Ready for user testing and feedback

---

## [0.6.0-beta] - 2025-12-21

### Added
- Core audio engine with dual-output routing
- High-performance playback to primary and secondary audio devices
- Global hotkey system with customizable key mappings
- Sound library management with category organization
- Audio waveform visualization with interactive trim editor
- Multi-format audio support (MP3, OGG/Vorbis, M4A/AAC via symphonia)
- LRU cache (500MB) for decoded audio and waveform data
- Non-destructive audio trimming
- System tray integration
- Settings panel with audio device configuration
- Drag-and-drop file import
- Discord-inspired dark theme UI

### Technical
- Built with React 18 + TypeScript + Vite
- Tauri v2 backend with Rust
- cpal for audio I/O
- symphonia for audio decoding
- TailwindCSS for styling
- Husky pre-commit hooks for code quality

### Known Issues
- Hotkey multi-trigger on rapid key press
- UI click has artificial delay
- System tray close behavior inconsistent

### Platform Support
- Windows 10+ (fully supported)
- macOS/Linux (builds available, limited testing)

---

## [0.1.0-alpha] - 2025-12-21

### Initial Release
- Foundation release establishing core architecture
- Basic audio playback functionality
- Hotkey system foundation
- Early testing phase
