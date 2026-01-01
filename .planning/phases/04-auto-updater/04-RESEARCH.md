# Phase 4: Auto-Updater - Research

**Researched:** 2026-01-01
**Domain:** Tauri v2 Auto-Updater with GitHub Releases
**Confidence:** HIGH

<research_summary>
## Summary

Researched the Tauri v2 updater plugin ecosystem for implementing auto-updates in a Windows desktop app. The standard approach uses `tauri-plugin-updater` with GitHub Releases as the update endpoint, leveraging `tauri-action` for automated manifest generation.

Key finding: Tauri's updater requires its own signing mechanism (separate from Windows code signing) using Ed25519 keys. This is mandatory and cannot be disabled. The signing keys are generated via `tauri signer generate` and used to create `.sig` signature files that verify update integrity.

**Primary recommendation:** Use tauri-plugin-updater 2.0+ with tauri-action for GitHub Releases integration. Generate signing keys early, store private key securely in GitHub Secrets, and use the existing release.yml workflow as a foundation.

</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri-plugin-updater | 2.0.0+ | Update check, download, install | Official Tauri plugin, required for v2 |
| tauri-plugin-process | 2.0.0+ | App relaunch after update | Required for restart functionality |
| @tauri-apps/plugin-updater | 2.0.0+ | Frontend JS bindings | Official JS API for update UI |
| @tauri-apps/plugin-process | 2.0.0+ | Frontend relaunch | Required for `relaunch()` function |

### CI/CD
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tauri-apps/tauri-action | v0 | Build + Release automation | Generates latest.json, uploads artifacts |
| softprops/action-gh-release | v1 | GitHub Release creation | Already in use, can complement tauri-action |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| GitHub Releases | Self-hosted server | More control, but more maintenance |
| tauri-action | Manual latest.json | More work, error-prone |
| NSIS installer | WiX/MSI installer | NSIS preferred for updates (better UX) |

**Installation:**

Rust (Cargo.toml):
```toml
[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]
tauri-plugin-updater = "2.0"
tauri-plugin-process = "2.0"
```

Frontend:
```bash
yarn add @tauri-apps/plugin-updater @tauri-apps/plugin-process
```

</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Project Structure
```
src/
├── components/
│   └── UpdateNotification.tsx    # Header badge component
├── hooks/
│   └── useUpdateCheck.ts         # Update logic hook
└── contexts/
    └── UpdateContext.tsx         # Update state management

src-tauri/
├── src/
│   └── lib.rs                    # Plugin registration
├── capabilities/
│   └── main-capability.json      # updater:default permission
└── tauri.conf.json               # Updater config (endpoints, pubkey)
```

### Pattern 1: User-Controlled Update Flow
**What:** Check for updates, show notification, let user decide
**When to use:** When user control is essential (our case)
**Example:**
```typescript
// Source: tauri-apps/plugins-workspace docs
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

// Check without auto-download
const update = await check();

if (update) {
  // Show notification to user with:
  // - update.version
  // - update.body (release notes)
  // User clicks "Update Now" to trigger:
  await update.downloadAndInstall((progress) => {
    if (progress.event === 'Progress') {
      updateProgressBar(progress.data.chunkLength);
    }
  });
  await relaunch();
}
```

### Pattern 2: Background Check on App Start
**What:** Silent check at startup, non-blocking notification
**When to use:** Dezente Update-Benachrichtigung (our preference)
**Example:**
```typescript
// In App.tsx or root component
useEffect(() => {
  const checkForUpdates = async () => {
    try {
      const update = await check({ timeout: 10000 });
      if (update) {
        setUpdateAvailable(update);
      }
    } catch (error) {
      // Silently fail - don't bother user
      console.warn('Update check failed:', error);
    }
  };

  // Check after short delay to not block startup
  const timer = setTimeout(checkForUpdates, 3000);
  return () => clearTimeout(timer);
}, []);
```

### Pattern 3: Progress Tracking During Download
**What:** Show download progress to user
**When to use:** Large updates, user transparency
**Example:**
```typescript
// Source: tauri-apps/plugins-workspace docs
await update.download((event) => {
  if (event.event === 'Started') {
    setTotalSize(event.data.contentLength);
    setDownloading(true);
  } else if (event.event === 'Progress') {
    setDownloaded(prev => prev + event.data.chunkLength);
  } else if (event.event === 'Finished') {
    setDownloading(false);
    setReadyToInstall(true);
  }
});
```

### Anti-Patterns to Avoid
- **Auto-download without consent:** User explicitly wants control
- **Blocking update check on main thread:** Use async/await properly
- **No error handling for network failures:** Always wrap in try-catch
- **Ignoring Windows install behavior:** App exits during NSIS install

</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Update signature verification | Custom crypto | tauri-plugin-updater | Ed25519 signing is complex, built-in handles it |
| Download with resume | Custom HTTP client | Plugin's download() | Handles chunked downloads, progress, retry |
| Manifest generation | Manual JSON file | tauri-action | Generates latest.json automatically with signatures |
| App restart after install | Process spawn | tauri-plugin-process relaunch() | Handles Windows restart correctly |
| Version comparison | semver parsing | Plugin's check() | Compares against current_version automatically |

**Key insight:** Tauri's updater plugin handles the entire update lifecycle - verification, download, install, restart. Focus on UI/UX, not the plumbing.

</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Signing Key Confusion
**What goes wrong:** Confusing Tauri update signing with Windows code signing
**Why it happens:** Both involve "signing" but are completely different
**How to avoid:**
- Tauri signing (Ed25519): REQUIRED for updater, generates .sig files
- Windows code signing (Authenticode): OPTIONAL, prevents SmartScreen
**Warning signs:** "A public key has been found, but no private key" error

### Pitfall 2: Missing TAURI_SIGNING_PRIVATE_KEY in CI
**What goes wrong:** Build fails when createUpdaterArtifacts is true
**Why it happens:** Private key not set as environment variable
**How to avoid:**
1. Generate keys: `npx tauri signer generate -w ~/.tauri/sonicdeck.key`
2. Add public key to tauri.conf.json
3. Add private key as GitHub Secret: `TAURI_SIGNING_PRIVATE_KEY`
4. Set in workflow: `env: TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}`
**Warning signs:** CI build fails at signing step

### Pitfall 3: Windows App Exits During Update
**What goes wrong:** App closes before user sees "update complete" message
**Why it happens:** NSIS installer requires app to close for file replacement
**How to avoid:**
- Show "Installing update, app will restart..." message BEFORE install()
- Use `installMode: "passive"` for progress bar without interaction
**Warning signs:** User thinks app crashed during update

### Pitfall 4: latest.json URL Mismatch
**What goes wrong:** Updates never found or wrong version downloaded
**Why it happens:** releaseId set without tagName, URLs point to /latest/download/
**How to avoid:** Either use tauri-action's defaults OR set both releaseId and tagName
**Warning signs:** Update check returns null despite new release existing

### Pitfall 5: Network Failure Silent Crash
**What goes wrong:** App freezes or crashes if no internet during update check
**Why it happens:** Unhandled Promise rejection
**How to avoid:** Always wrap check() in try-catch, set reasonable timeout
**Warning signs:** Uncaught promise rejection in console

</common_pitfalls>

<code_examples>
## Code Examples

### tauri.conf.json Configuration
```json
// Source: https://v2.tauri.app/plugin/updater/
{
  "bundle": {
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk...",
      "endpoints": [
        "https://github.com/DraneLixX/SonicDeck/releases/latest/download/latest.json"
      ],
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

### Plugin Registration (lib.rs)
```rust
// Source: tauri-apps/plugins-workspace README
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        // ... other plugins
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Capability Permission
```json
// src-tauri/capabilities/main-capability.json
{
  "permissions": [
    "updater:default",
    "process:allow-restart",
    "process:allow-exit"
  ]
}
```

### Complete Update Hook
```typescript
// Source: Verified pattern from tauri-apps/plugins-workspace
import { check, Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { useState, useEffect, useCallback } from 'react';

interface UpdateState {
  available: Update | null;
  checking: boolean;
  downloading: boolean;
  progress: number;
  error: string | null;
}

export function useUpdateCheck() {
  const [state, setState] = useState<UpdateState>({
    available: null,
    checking: false,
    downloading: false,
    progress: 0,
    error: null,
  });

  const checkForUpdates = useCallback(async () => {
    setState(prev => ({ ...prev, checking: true, error: null }));
    try {
      const update = await check({ timeout: 15000 });
      setState(prev => ({ ...prev, available: update, checking: false }));
    } catch (error) {
      setState(prev => ({
        ...prev,
        checking: false,
        error: error instanceof Error ? error.message : 'Update check failed'
      }));
    }
  }, []);

  const installUpdate = useCallback(async () => {
    if (!state.available) return;

    setState(prev => ({ ...prev, downloading: true, progress: 0 }));

    try {
      let totalSize = 0;
      let downloaded = 0;

      await state.available.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          totalSize = event.data.contentLength ?? 0;
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          const progress = totalSize > 0 ? (downloaded / totalSize) * 100 : 0;
          setState(prev => ({ ...prev, progress }));
        }
      });

      // Relaunch after install
      await relaunch();
    } catch (error) {
      setState(prev => ({
        ...prev,
        downloading: false,
        error: error instanceof Error ? error.message : 'Installation failed'
      }));
    }
  }, [state.available]);

  // Auto-check on mount
  useEffect(() => {
    const timer = setTimeout(checkForUpdates, 3000);
    return () => clearTimeout(timer);
  }, [checkForUpdates]);

  return {
    ...state,
    checkForUpdates,
    installUpdate,
  };
}
```

### GitHub Actions Workflow Update
```yaml
# Addition to existing release.yml
- name: Build Tauri app with updater
  run: yarn tauri build
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
```

</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 updater events | Tauri v2 plugin system | v2.0 (2024) | Different API, plugin-based |
| Manual latest.json | tauri-action auto-generation | 2024 | Simpler CI setup |
| Single endpoint | Multiple fallback endpoints | v2.0 | Better reliability |
| WiX for Windows updates | NSIS preferred | 2024-2025 | Better update UX |

**New tools/patterns to consider:**
- **tauri-plugin-updater 2.10.0+:** Supports {os}-{arch}-{installer} keys for multiple installers
- **updaterJsonPreferNsis:** New tauri-action option for choosing NSIS over WiX

**Deprecated/outdated:**
- **@tauri-apps/api/updater (v1):** Use @tauri-apps/plugin-updater for v2
- **onUpdaterEvent:** Replaced by progress callbacks in download()/downloadAndInstall()
- **window.tauri.updater:** Use proper imports from plugin package

</sota_updates>

<open_questions>
## Open Questions

1. **NSIS vs MSI for update artifacts**
   - What we know: Both work, NSIS has better UX for updates (passive mode)
   - What's unclear: Current release.yml uploads both - do we need both for updater?
   - Recommendation: Use `updaterJsonPreferNsis: true` in tauri-action

2. **Changelog Kurzfassung Generation**
   - What we know: `update.body` contains full release notes from GitHub
   - What's unclear: How to extract/generate "3 neue Features, 2 Bugfixes" summary
   - Recommendation: Parse release notes in frontend or add summary to release body

3. **Update Check Frequency**
   - What we know: User wants check at startup, not aggressive polling
   - What's unclear: Should there be a "Check for Updates" button in Settings too?
   - Recommendation: Startup check + optional manual check in Settings

</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- /tauri-apps/plugins-workspace (Context7) - updater plugin setup, JS API, download/install patterns
- /tauri-apps/tauri (Context7) - tauri.conf.json configuration, architecture
- https://v2.tauri.app/plugin/updater/ - Official updater plugin documentation
- https://v2.tauri.app/distribute/pipelines/github/ - GitHub Actions workflow setup

### Secondary (MEDIUM confidence)
- https://github.com/tauri-apps/tauri-action - tauri-action parameters, latest.json generation
- WebSearch verified: Windows code signing vs Tauri signing distinction

### Tertiary (LOW confidence - needs validation)
- Error handling patterns - community patterns, should test during implementation
- Changelog summary extraction - needs implementation research

</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: tauri-plugin-updater 2.0, tauri-plugin-process
- Ecosystem: tauri-action, GitHub Releases, NSIS installer
- Patterns: User-controlled updates, background check, progress tracking
- Pitfalls: Signing confusion, CI secrets, Windows install behavior

**Confidence breakdown:**
- Standard stack: HIGH - verified with Context7 and official docs
- Architecture: HIGH - from official examples and docs
- Pitfalls: HIGH - documented in GitHub issues and discussions
- Code examples: HIGH - from Context7 and official plugin docs

**Research date:** 2026-01-01
**Valid until:** 2026-02-01 (30 days - Tauri plugin ecosystem stable)

</metadata>

---

*Phase: 04-auto-updater*
*Research completed: 2026-01-01*
*Ready for planning: yes*
