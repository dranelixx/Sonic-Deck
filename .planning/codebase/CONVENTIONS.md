# Coding Conventions

**Analysis Date:** 2025-12-29

## Naming Patterns

**Files:**
- PascalCase.tsx - React components (`SoundButton.tsx`, `ErrorBoundary.tsx`)
- camelCase.ts - Hooks with `use` prefix (`useAudioPlayback.ts`, `useFileDrop.ts`)
- kebab-case.ts - Utilities (`hotkeyDisplay.ts`, `waveformQueue.ts`)
- snake_case.rs - Rust modules (`audio/manager.rs`, `commands/audio.rs`)
- *.test.ts - Test files alongside source (`hotkeyDisplay.test.ts`)

**Functions:**
- TypeScript: camelCase for all functions
- Rust: snake_case for all functions (`normalize_hotkey_string()`)
- Event handlers: `handle{Event}` prefix (`handleClick`, `handleContextMenu`)
- Async functions: No special prefix

**Variables:**
- TypeScript: camelCase (`playingSoundIds`, `activeWaveform`)
- TypeScript constants: UPPER_SNAKE_CASE in `constants.ts` (`ANIMATION_DURATIONS`, `DEBUG`)
- Rust: snake_case (`active_sounds`, `file_path`)
- Rust constants: UPPER_SNAKE_CASE (`DEFAULT_MAX_CACHE_BYTES`)

**Types:**
- TypeScript interfaces: PascalCase, no I prefix (`Sound`, `Category`, `AppSettings`)
- Rust structs/enums: PascalCase (`AudioManager`, `SoundState`, `AudioError`)
- Type properties: snake_case for JSON interop (`file_path`, `category_id`)

## Code Style

**Formatting:**
- Prettier with `.prettierrc` configuration
- Print width: 80 characters
- Tab width: 2 spaces (TypeScript), 4 spaces (Rust)
- Single quotes: No (double quotes only)
- Semicolons: Required
- Trailing commas: ES5 style (objects/arrays)
- End of line: Auto (platform-specific)

**Linting:**
- ESLint 9.39.2 with flat config (`eslint.config.js`)
- TypeScript ESLint parser with React support
- Rules:
  - `@typescript-eslint/no-unused-vars`: Error (prefix with `_` to ignore)
  - `@typescript-eslint/no-explicit-any`: Off
  - `react-hooks/rules-of-hooks`: Error
  - `react-hooks/exhaustive-deps`: Off
- Maximum warnings: 10 (`yarn lint` fails if exceeded)
- Rust: `cargo fmt` standard formatting, `clippy` for lints

## Import Organization

**Order:**
1. React and external packages (`react`, `@tauri-apps/api`)
2. Internal modules (`../contexts/`, `../hooks/`)
3. Relative imports (`./utils`, `../types`)
4. Type imports (`import type { ... }`)

**Grouping:**
- No strict grouping enforced
- Alphabetical order within groups preferred
- Destructured imports for specific exports

**Path Aliases:**
- Not used (relative imports only)
- TypeScript paths configured but not utilized

## Error Handling

**TypeScript Patterns:**
- try/catch around `invoke()` calls
- Toast notifications for user-facing errors
- Error boundaries for component crashes (`ErrorBoundary.tsx`)
- Async errors: catch and show Toast

**Rust Patterns:**
- `Result<T, E>` for fallible operations
- `AudioError` enum for audio-specific errors (`src-tauri/src/audio/error.rs`)
- `?` operator for error propagation
- `tracing::error!()` before returning errors
- Never use `.unwrap()` in production code (`.expect()` with message if needed)

**Error Types:**
- Throw on: Invalid input, missing dependencies, decode failures
- Return error: Expected failures like file not found
- Log context: Always log before returning error to frontend

## Logging

**Framework:**
- Rust: `tracing` crate (NOT `log` crate)
- Frontend: `console.log/warn/error` for debugging

**Patterns:**
```rust
use tracing::{info, warn, error, debug, trace};

info!("User initiated playback for sound: {}", sound_id);
warn!("Device {} not found, using default", device_name);
error!("Failed to decode audio: {:?}", err);
debug!("Detailed flow info");
trace!("Very verbose debugging");
```

**When to Log:**
- `info`: User actions, successful operations
- `warn`: Recoverable issues, fallbacks
- `error`: Failures, critical issues
- `debug`: Development-only detailed flow
- `trace`: Very verbose debugging

**Log Location:** `%LOCALAPPDATA%\com.sonicdeck.app\logs\sonicdeck.YYYY-MM-DD.log`

## Comments

**When to Comment:**
- Explain "why", not "what"
- Document business logic and edge cases
- Explain non-obvious algorithms or workarounds
- Avoid obvious comments (`// increment counter`)

**Section Headers:**
```typescript
// ============================================================================
// Animation & Timing Constants
// ============================================================================
```

**Rust Doc Comments:**
```rust
/// Normalize hotkey string to match our storage format
///
/// Converts "control+A" to "Ctrl+A", handles modifiers, NumPad keys
fn normalize_hotkey_string(hotkey: &str) -> String { ... }
```

**TODO Comments:**
- Format: `// TODO: description`
- Not currently used (codebase is clean)

## Function Design

**Size:**
- Keep under 50 lines (extract helpers for complex logic)
- One level of abstraction per function
- Single responsibility principle

**Parameters:**
- Max 3 parameters (use options object for more)
- Destructure objects in parameter list
- TypeScript: Explicit type annotations

**Return Values:**
- Explicit return statements
- Return early for guard clauses
- Rust: Use `Result<T, E>` for fallible operations

## Module Design

**React Components:**
- Functional components only (no class components)
- `const` arrow functions for definitions
- `memo()` wrapper for expensive components
- Export default at bottom

**Exports:**
- Named exports for utilities and hooks
- Default exports for React components
- Rust: `pub use` in `mod.rs` for public API

**Context Pattern:**
```typescript
// Create context with undefined default
const MyContext = createContext<MyContextType | undefined>(undefined);

// Provider component
export function MyProvider({ children }: { children: ReactNode }) {
  // state management
  return <MyContext.Provider value={...}>{children}</MyContext.Provider>;
}

// Custom hook with error handling
export function useMyContext() {
  const context = useContext(MyContext);
  if (!context) throw new Error("useMyContext must be used within MyProvider");
  return context;
}
```

**Rust Modules:**
- `mod.rs` for re-exports
- Inline `#[cfg(test)]` for unit tests
- `pub` only for items that need external access

---

*Convention analysis: 2025-12-29*
*Update when patterns change*
