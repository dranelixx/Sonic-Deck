# External Integrations

**Analysis Date:** 2025-12-29

## APIs & External Services

**None Detected - Fully Offline Application**

SonicDeck is designed as a completely offline desktop application with no external API dependencies:
- No HTTP/REST clients
- No authentication services
- No telemetry/analytics
- No payment processors
- No cloud storage
- No CDN integrations

## Data Storage

**Local File Storage:**
- JSON files in `%LOCALAPPDATA%\com.sonicdeck.app\` (Windows)
  - `sounds.json` - Sound library with categories
  - `settings.json` - Application settings (device routing, volumes)
  - `hotkeys.json` - Global hotkey mappings
- Atomic writes via temp-file-rename pattern (`src-tauri/src/persistence.rs`)
- No database required

**Audio File Storage:**
- User-managed audio files (MP3, OGG, M4A)
- Files remain in original locations (not copied to app directory)
- LRU cache for decoded audio data (500MB in memory)

**Logging:**
- Location: `%LOCALAPPDATA%\com.sonicdeck.app\logs\sonicdeck.YYYY-MM-DD.log`
- Daily rotation with 7-day retention
- Uses `tracing` crate with `tracing-appender`

## Authentication & Identity

**Not Applicable**

- No user accounts
- No authentication required
- All data stored locally
- No cloud sync

## Monitoring & Observability

**Local Logging Only:**
- `tracing` crate for structured logging (`src-tauri/src/main.rs`)
- Levels: trace, debug, info, warn, error
- No external error tracking (no Sentry, Bugsnag, etc.)
- No analytics or telemetry

## CI/CD & Deployment

**CI Pipeline:**
- GitHub Actions - `.github/workflows/`
  - `rust.yml` - Rust quality checks (fmt, clippy, cargo check)
  - `tests.yml` - Full test suite + coverage (Rust + Frontend)
  - `frontend.yml` - Prettier, ESLint, TypeScript checks
  - `claude-code-review.yml` - AI code review (PRs to `main` only)
- No secrets required for CI (offline app)
- Coverage reports via Codecov

**Distribution:**
- MSI installer built via Tauri bundler
- No auto-update service currently
- Manual distribution via GitHub Releases

## Environment Configuration

**Development:**
- No environment variables required
- All config via TypeScript/Rust files
- Local audio devices required for testing

**Production:**
- No environment-specific configuration
- All settings stored in user's LocalAppData
- Works completely offline

## Webhooks & Callbacks

**Not Applicable**

- No incoming webhooks
- No outgoing webhooks
- No HTTP server component

## Tauri Plugins (Local System Integration)

**File System Access:**
- `tauri-plugin-fs` - Read/write audio files and settings
- `tauri-plugin-dialog` - File picker dialogs for importing sounds
- Capabilities defined in `src-tauri/capabilities/main-capability.json`

**Desktop Integration:**
- `tauri-plugin-global-shortcut` - OS-level hotkey registration
- `tauri-plugin-autostart` - Launch on system startup
- `tauri-plugin-shell` - Open external links
- `tauri-plugin-notification` - System notifications

**Audio:**
- Direct audio device access via `cpal` crate
- No audio servers (PulseAudio, JACK) required on Windows

---

*Integration audit: 2025-12-29*
*Update when adding external services*
