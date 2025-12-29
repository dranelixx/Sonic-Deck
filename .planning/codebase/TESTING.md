# Testing Patterns

**Analysis Date:** 2025-12-29

## Test Framework

**Frontend Runner:**
- Vitest 4.0.16 (`vitest.config.ts`)
- Environment: jsdom
- Globals: enabled (describe, it, expect without imports)

**Frontend Assertion Library:**
- Vitest built-in expect
- @testing-library/jest-dom for DOM matchers
- Matchers: toBe, toEqual, toThrow, toMatchObject

**Backend Runner:**
- Cargo test (built-in)
- cargo-llvm-cov for coverage

**Run Commands:**
```bash
# Frontend
yarn test                              # Watch mode
yarn test:run                          # Single run
yarn test:coverage                     # With coverage report

# Backend
cd src-tauri && cargo test             # Run all Rust tests
cd src-tauri && cargo llvm-cov --html  # Coverage report
cd src-tauri && cargo llvm-cov --fail-under-lines 45  # CI threshold
```

## Test File Organization

**Frontend Location:**
- Co-located with source: `src/utils/hotkeyDisplay.ts` → `src/utils/hotkeyDisplay.test.ts`
- Test setup: `src/test/setup.ts`

**Frontend Naming:**
- Pattern: `{module-name}.test.ts`
- No distinction between unit/integration in filename

**Backend Location:**
- Unit tests: Inline `#[cfg(test)]` modules in source files
- Integration tests: `src-tauri/tests/` directory
- Fixtures: `src-tauri/tests/fixtures/`

**Structure:**
```
src/
  utils/
    hotkeyDisplay.ts
    hotkeyDisplay.test.ts
    waveformQueue.ts
    waveformQueue.test.ts
  test/
    setup.ts

src-tauri/
  src/
    audio/
      cache.rs          # Contains #[cfg(test)] mod tests
      manager.rs        # Contains #[cfg(test)] mod tests
      ...
  tests/
    audio_decode.rs     # Integration test
    fixtures/
      test_mono.mp3
      test_stereo.ogg
      test_stereo.m4a
```

## Test Structure

**Frontend Suite Organization:**
```typescript
import { describe, it, expect } from "vitest";
import { formatHotkeyForDisplay } from "./hotkeyDisplay";

describe("formatHotkeyForDisplay", () => {
  describe("modifier keys", () => {
    it("should format Ctrl key", () => {
      expect(formatHotkeyForDisplay("Ctrl")).toBe("Ctrl");
    });

    it("should format Super as Win", () => {
      expect(formatHotkeyForDisplay("Super")).toBe("Win");
    });
  });

  describe("combined hotkeys", () => {
    it("should format Ctrl+A", () => {
      expect(formatHotkeyForDisplay("Ctrl+A")).toBe("Ctrl + A");
    });
  });
});
```

**Rust Test Structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_eviction() {
        // arrange
        let mut cache = AudioCache::new(1); // 1MB

        // act
        cache.insert("test.mp3", sample_audio_data());

        // assert
        assert!(cache.contains("test.mp3"));
    }
}
```

**Patterns:**
- Use `describe()` for feature grouping
- Nested `describe()` for subcategories
- `it()` for individual test cases
- Test names: "should [expected behavior]"
- Arrange/Act/Assert structure

## Mocking

**Frontend Framework:**
- Vitest built-in (`vi`)
- Module mocking via `vi.mock()` at test file top

**Frontend Mock Setup** (`src/test/setup.ts`):
```typescript
import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// Mock Tauri APIs
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

// Mock browser APIs
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  })),
});

window.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));
```

**Frontend Mock Usage:**
```typescript
import { vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";

it("should call invoke with correct parameters", async () => {
  const mockData = { peaks: [0.5, 0.7], duration_ms: 1000 };
  vi.mocked(invoke).mockResolvedValueOnce(mockData);

  const result = await waveformQueue.add("/path/to/audio.mp3", 100);

  expect(invoke).toHaveBeenCalledWith("get_waveform", {
    filePath: "/path/to/audio.mp3",
    numPeaks: 100,
  });
  expect(result).toEqual(mockData);
});
```

**What to Mock:**
- Tauri APIs (`invoke`, `listen`, `emit`)
- Browser APIs (`matchMedia`, `ResizeObserver`)
- File system operations
- External services

**What NOT to Mock:**
- Pure functions (test directly)
- Internal utilities
- Type definitions

## Fixtures and Factories

**Test Data (Frontend):**
```typescript
// Inline test data
const mockSound: Sound = {
  id: "test-id",
  name: "Test Sound",
  file_path: "/path/to/test.mp3",
  category_id: "cat-1",
  icon: null,
  volume: 1.0,
  is_favorite: false,
  trim_start_ms: null,
  trim_end_ms: null,
};
```

**Test Fixtures (Rust):**
```
src-tauri/tests/fixtures/
├── test_mono.mp3      # 1s, 44.1kHz, Mono
├── test_stereo.ogg    # 1s, 48kHz, Stereo
└── test_stereo.m4a    # 1s, 48kHz, Stereo
```

**Fixture Access (Rust):**
```rust
fn get_test_file_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename)
}

#[test]
fn test_mp3_fixture_exists() {
    let path = get_test_file_path("test_mono.mp3");
    assert!(path.exists());
}
```

## Coverage

**Frontend Requirements:**
- Threshold: 5% (will be raised as more tests added)
- Tool: @vitest/coverage-v8
- Reporter: text, lcov, html

**Frontend Configuration** (`vitest.config.ts`):
```typescript
coverage: {
  provider: 'v8',
  reporter: ['text', 'lcov', 'html'],
  reportsDirectory: './coverage',
  thresholds: {
    lines: 5,
    functions: 3,
    branches: 5,
    statements: 5,
  }
}
```

**Backend Requirements:**
- Threshold: 45% line coverage
- Tool: cargo-llvm-cov
- CI fails if below threshold

**View Coverage:**
```bash
# Frontend
yarn test:coverage
open coverage/index.html

# Backend
cd src-tauri && cargo llvm-cov --html --open
```

## Test Types

**Frontend Unit Tests:**
- Scope: Single function/hook in isolation
- Mocking: Mock all external dependencies (Tauri, browser APIs)
- Speed: Each test <100ms
- Examples: `hotkeyDisplay.test.ts`, `waveformQueue.test.ts`

**Rust Unit Tests:**
- Scope: Single function/struct in isolation
- Location: Inline `#[cfg(test)]` modules
- Files with tests:
  - `audio/cache.rs` - LRU cache logic, eviction
  - `audio/waveform.rs` - Peak generation, normalization
  - `audio/manager.rs` - State machine, playback IDs
  - `audio/playback.rs` - Volume curve
  - `audio/mod.rs` - DeviceId parsing
  - `persistence.rs` - Atomic file writes
  - `sounds.rs` - Sound/Category CRUD
  - `settings.rs` - AppSettings defaults
  - `hotkeys.rs` - HotkeyMappings CRUD

**Rust Integration Tests:**
- Scope: Multiple modules together
- Location: `src-tauri/tests/`
- Examples: `audio_decode.rs` - Test fixture validation

**E2E Tests:**
- Not currently implemented
- Manual testing via `yarn tauri dev`

## Common Patterns

**Async Testing:**
```typescript
it("should handle async operation", async () => {
  const result = await asyncFunction();
  expect(result).toBe("expected");
});
```

**Error Testing:**
```typescript
it("should throw on invalid input", () => {
  expect(() => functionCall()).toThrow("error message");
});

// Async error
it("should reject on failure", async () => {
  await expect(asyncCall()).rejects.toThrow("error message");
});
```

**Tauri Mock Testing:**
```typescript
it("should invoke Tauri command", async () => {
  vi.mocked(invoke).mockResolvedValueOnce({ success: true });

  const result = await myFunction();

  expect(invoke).toHaveBeenCalledWith("command_name", { param: "value" });
  expect(result).toEqual({ success: true });
});
```

**Snapshot Testing:**
- Not used in this codebase
- Prefer explicit assertions for clarity

---

*Testing analysis: 2025-12-29*
*Update when test patterns change*
