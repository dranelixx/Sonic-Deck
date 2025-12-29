# Technology Stack

**Analysis Date:** 2025-12-29

## Languages

**Primary:**
- TypeScript 5.3.3 - All frontend application code (`package.json`)
- Rust 2021 Edition - All backend/audio processing (`src-tauri/Cargo.toml`)

**Secondary:**
- JavaScript - Build scripts, config files (`eslint.config.js`, `scripts/`)
- CSS - TailwindCSS 4 with `@theme` in `src/index.css`

## Runtime

**Environment:**
- Node.js 18+ (ESM modules) - Inferred from `package.json` type: "module"
- Tauri 2.0 WebView (Chromium-based) - `src-tauri/tauri.conf.json`
- Windows target (MSI bundler) - `src-tauri/tauri.conf.json`

**Package Manager:**
- Yarn - Lockfile `yarn.lock` present
- Cargo - Rust dependencies via `src-tauri/Cargo.toml`

## Frameworks

**Core:**
- React 19.2.3 - UI framework (`package.json`)
- Tauri 2.0 - Desktop framework with IPC (`src-tauri/Cargo.toml`)
- TailwindCSS 4.1.18 - Styling via `@tailwindcss/vite` plugin (`vite.config.ts`)

**Testing:**
- Vitest 4.0.16 - Frontend unit tests (`vitest.config.ts`)
- @testing-library/react 16.3.1 - Component testing (`package.json`)
- Cargo test - Rust unit/integration tests (inline `#[cfg(test)]` modules)
- cargo-llvm-cov - Rust code coverage (`package.json` scripts)

**Build/Dev:**
- Vite 7.3.0 - Frontend bundler (`vite.config.ts`)
- TypeScript 5.3.3 - Type checking and compilation (`tsconfig.json`)
- ESLint 9.39.2 - Linting with flat config (`eslint.config.js`)
- Prettier 3.7.4 - Code formatting (`.prettierrc`)
- Husky 9.1.7 - Git hooks (`.husky/pre-commit`)
- lint-staged 16.2.7 - Pre-commit linting (`package.json`)

## Key Dependencies

**Critical (Frontend):**
- @tauri-apps/api 2.5.0 - Tauri IPC bridge (`package.json`)
- @tauri-apps/plugin-global-shortcut 2.3.1 - Global hotkey support (`package.json`)
- @tauri-apps/plugin-autostart 2.5.1 - System autostart (`package.json`)
- @tauri-apps/plugin-shell 2.0.0 - Shell command execution (`package.json`)
- emojibase 17.0.0 + emojibase-data 17.0.0 - Emoji picker (`package.json`)

**Critical (Backend):**
- cpal 0.15 - Cross-platform audio I/O (`src-tauri/Cargo.toml`)
- symphonia 0.5 - Audio decoding: MP3, OGG/Vorbis, M4A/AAC (`src-tauri/Cargo.toml`)
- lru 0.12 - LRU cache for decoded audio (500MB default) (`src-tauri/Cargo.toml`)
- serde 1.0 + serde_json 1.0 - JSON serialization (`src-tauri/Cargo.toml`)
- tracing 0.1 + tracing-subscriber + tracing-appender - Logging (`src-tauri/Cargo.toml`)

**Infrastructure:**
- tauri-plugin-dialog 2.0 - File dialogs (`src-tauri/Cargo.toml`)
- tauri-plugin-fs 2.0 - File system access (`src-tauri/Cargo.toml`)
- tauri-plugin-notification 2.0 - System notifications (`src-tauri/Cargo.toml`)

## Configuration

**Environment:**
- No environment variables required (fully offline app)
- No `.env` files used
- Configuration stored in `%APPDATA%\com.sonicdeck.app\` (Windows)

**Build:**
- `tsconfig.json` - TypeScript compiler (ES2020 target, strict mode)
- `vite.config.ts` - Vite bundler with React + TailwindCSS plugins
- `vitest.config.ts` - Test runner (jsdom environment)
- `eslint.config.js` - ESLint 9 flat config format
- `.prettierrc` - Prettier formatting (80 chars, double quotes, semicolons)
- `src-tauri/tauri.conf.json` - Tauri app config (1200x800 window, dark theme)
- `src-tauri/capabilities/main-capability.json` - Tauri v2 permissions

**Version Management:**
- `version.json` - Single source of truth for app version
- `scripts/sync-version.js` - Auto-syncs to `package.json`, `Cargo.toml`, `tauri.conf.json`

## Platform Requirements

**Development:**
- Windows 10/11 (primary target)
- Node.js 18+ with Yarn
- Rust toolchain (stable, MSVC target)
- No Docker required
- No external dependencies

**Production:**
- Windows 10+ (MSI installer)
- No runtime dependencies (all bundled)
- Runs completely offline

---

*Stack analysis: 2025-12-29*
*Update after major dependency changes*
