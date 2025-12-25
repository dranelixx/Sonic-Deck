# Version Management

Sonic Deck uses a centralized version system to keep all version strings in sync across the project.

## How to Bump the Version

1. **Edit `version.json`** with the new version (numeric format):
   ```json
   {
     "version": "0.3.0-0",
     "prerelease": true,
     "channel": "alpha",
     "releaseDate": "2025-12-25",
     "description": "Your release notes here"
   }
   ```

2. **Commit your changes:**
   ```bash
   git add version.json
   git commit -m "chore: bump version to 0.3.0-alpha"
   git tag v0.3.0-alpha
   git push && git push --tags
   ```

3. **The pre-commit hook automatically:**
   - Runs `sync-version.js`
   - Updates `package.json`, `Cargo.toml`, `tauri.conf.json`
   - Stages the synced files
   - Everything is committed together

## Version Format (Dual-Version System)

### MSI Compatibility Requirement

**Problem:** MSI installers (Windows) only accept **numeric** pre-release identifiers. Alphanumeric versions like `0.3.0-alpha` cause build failures.

**Solution:** Sonic Deck uses a **dual-version system**:

| Component | Format | Example | Purpose |
|-----------|--------|---------|---------|
| **Build Version** | Numeric | `0.7.0-0` | MSI installer, package managers |
| **Display Version** | Alphanumeric | `v0.7.0-alpha` | User-facing (UI, GitHub releases) |

### Version Mapping

The `version` field uses **numeric** pre-release identifiers, while `channel` provides the user-friendly name:

| Phase | `version` | `channel` | MSI Installer | User Sees |
|-------|-----------|-----------|---------------|-----------|
| **Alpha** | `0.X.Y-0` | `"alpha"` | `0.X.Y-0` | `v0.X.Y-alpha` |
| **Beta** | `0.X.Y-1` | `"beta"` | `0.X.Y-1` | `v0.X.Y-beta` |
| **RC** | `0.X.Y-2` | `"rc"` | `0.X.Y-2` | `v0.X.Y-rc` |
| **Stable** | `0.X.Y` | `""` | `0.X.Y` | `v0.X.Y` |

**Examples:**
- `{ "version": "0.7.0-0", "channel": "alpha" }` → User sees `v0.7.0-alpha`
- `{ "version": "0.8.0-1", "channel": "beta" }` → User sees `v0.8.0-beta`
- `{ "version": "1.0.0", "channel": "" }` → User sees `v1.0.0`

## Files Included in Sync

- `version.json` - **Source of truth** (edit this only)
- `package.json` - Frontend version (auto-synced)
- `src-tauri/Cargo.toml` - Rust backend version (auto-synced)
- `src-tauri/tauri.conf.json` - Tauri app configuration (auto-synced)

## Using Version in Code

### Frontend (React/TypeScript)

Use the `VITE_APP_VERSION` and `VITE_APP_CHANNEL` environment variables in React components:

```typescript
// Display user-friendly version (e.g., "v0.7.0-alpha")
const appVersion = import.meta.env.VITE_APP_VERSION;
const appChannel = import.meta.env.VITE_APP_CHANNEL || "";

const displayVersion = appChannel
  ? `v${appVersion.replace(/-\d+$/, '')}-${appChannel}`
  : `v${appVersion}`;

export const MyComponent = () => (
  <p>Version: {displayVersion}</p>
);
```

**How it works:**
- Both values set at **build-time** from `version.json` in `vite.config.ts`
- `VITE_APP_VERSION` = numeric build version (e.g., `0.7.0-0`)
- `VITE_APP_CHANNEL` = alphanumeric channel (e.g., `alpha`)
- Regex `.replace(/-\d+$/, '')` removes numeric suffix → `0.7.0-0` becomes `0.7.0`
- Combined with channel → `v0.7.0-alpha` (user-friendly)
- No runtime overhead
- Automatically updated when you bump the version

### Backend (Rust)

Get the version from `tauri.conf.json` at runtime:

```rust
use tauri::Config;

let config = Config::default();
let version = &config.package.version;
println!("App version: {}", version);
```

Or use the `env!("CARGO_PKG_VERSION")` macro (set from Cargo.toml):

```rust
const VERSION: &str = env!("CARGO_PKG_VERSION");
```

## Workflow Summary

```
Edit version.json
       ↓
git commit
       ↓
Pre-commit hook runs:
  ├→ sync-version.js (updates package.json, Cargo.toml, tauri.conf.json)
  ├→ Auto-stages synced files
  └→ Pre-commit checks continue
       ↓
Single commit with all synced files
```

## Why This System?

Single source of truth prevents version mismatches across different build systems (npm, cargo, Tauri).
- ✅ **Single Source of Truth**: Only edit `version.json`
- ✅ **MSI Compatibility**: Numeric build versions work with Windows installers
- ✅ **User-Friendly Display**: Alphanumeric channel names in UI (`v0.7.0-alpha`)
- ✅ **Automatic Synchronization**: Pre-commit hook syncs all config files
- ✅ **No Manual Updates**: All version files updated automatically
- ✅ **Future-Proof**: Environment variables allow flexible version display
