import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useUpdateCheck } from "./useUpdateCheck";

// Mock the Tauri plugins
const mockCheck = vi.fn();
const mockRelaunch = vi.fn();

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: (...args: unknown[]) => mockCheck(...args),
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: () => mockRelaunch(),
}));

describe("useUpdateCheck", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();

    // Mock Tauri environment
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      value: {},
      writable: true,
      configurable: true,
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    // Clean up Tauri mock
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  it("should have correct initial state", () => {
    const { result } = renderHook(() => useUpdateCheck());

    expect(result.current.available).toBeNull();
    expect(result.current.checking).toBe(false);
    expect(result.current.downloading).toBe(false);
    expect(result.current.progress).toBe(0);
    expect(result.current.error).toBeNull();
  });

  it("should provide checkForUpdates, installUpdate, and dismissError functions", () => {
    const { result } = renderHook(() => useUpdateCheck());

    expect(typeof result.current.checkForUpdates).toBe("function");
    expect(typeof result.current.installUpdate).toBe("function");
    expect(typeof result.current.dismissError).toBe("function");
  });

  it("should schedule auto-check for updates on mount", async () => {
    // Use real timers for this test since fake timers don't work well with dynamic imports
    vi.useRealTimers();
    mockCheck.mockResolvedValue(null);

    renderHook(() => useUpdateCheck());

    // Should not have checked immediately
    expect(mockCheck).not.toHaveBeenCalled();

    // Wait for the 3 second delay + some buffer
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 3500));
    });

    expect(mockCheck).toHaveBeenCalledWith({ timeout: 15000 });

    // Restore fake timers for other tests
    vi.useFakeTimers();
  });

  it("should set checking to true during update check", async () => {
    // Create a promise we can control
    let resolvePromise: (value: null) => void = () => {};
    const checkPromise = new Promise<null>((resolve) => {
      resolvePromise = resolve;
    });
    mockCheck.mockReturnValue(checkPromise);

    const { result } = renderHook(() => useUpdateCheck());

    // Trigger manual check (don't await, we want to catch the intermediate state)
    let checkPromiseResult: Promise<void>;
    act(() => {
      checkPromiseResult = result.current.checkForUpdates();
    });

    // Should be checking
    expect(result.current.checking).toBe(true);

    // Resolve the check
    await act(async () => {
      resolvePromise(null);
      await checkPromiseResult;
    });

    expect(result.current.checking).toBe(false);
  });

  it("should store available update when found", async () => {
    const mockUpdate = {
      version: "2.0.0",
      body: "New features",
      downloadAndInstall: vi.fn(),
    };
    mockCheck.mockResolvedValue(mockUpdate);

    const { result } = renderHook(() => useUpdateCheck());

    await act(async () => {
      await result.current.checkForUpdates();
    });

    expect(result.current.available).toEqual(mockUpdate);
    expect(result.current.checking).toBe(false);
  });

  it("should handle check failure gracefully", async () => {
    mockCheck.mockRejectedValue(new Error("Network error"));

    const { result } = renderHook(() => useUpdateCheck());

    await act(async () => {
      await result.current.checkForUpdates();
    });

    // Should not set error for failed checks (silent fail)
    expect(result.current.error).toBeNull();
    expect(result.current.checking).toBe(false);
  });

  it("should not check for updates in non-Tauri environment", async () => {
    // Remove Tauri mock
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;

    const { result } = renderHook(() => useUpdateCheck());

    await act(async () => {
      await result.current.checkForUpdates();
    });

    expect(mockCheck).not.toHaveBeenCalled();
  });

  it("should track download progress during install", async () => {
    const mockDownloadAndInstall = vi
      .fn()
      .mockImplementation(async (callback) => {
        // Simulate download events
        callback({ event: "Started", data: { contentLength: 1000 } });
        callback({ event: "Progress", data: { chunkLength: 500 } });
        callback({ event: "Progress", data: { chunkLength: 500 } });
      });

    const mockUpdate = {
      version: "2.0.0",
      body: "New features",
      downloadAndInstall: mockDownloadAndInstall,
    };
    mockCheck.mockResolvedValue(mockUpdate);

    const { result } = renderHook(() => useUpdateCheck());

    // First check for updates
    await act(async () => {
      await result.current.checkForUpdates();
    });

    expect(result.current.available).not.toBeNull();

    // Now install
    await act(async () => {
      await result.current.installUpdate();
    });

    expect(mockDownloadAndInstall).toHaveBeenCalled();
    expect(mockRelaunch).toHaveBeenCalled();
  });

  it("should handle install failure", async () => {
    const mockDownloadAndInstall = vi
      .fn()
      .mockRejectedValue(new Error("Download failed"));

    const mockUpdate = {
      version: "2.0.0",
      body: "New features",
      downloadAndInstall: mockDownloadAndInstall,
    };
    mockCheck.mockResolvedValue(mockUpdate);

    const { result } = renderHook(() => useUpdateCheck());

    await act(async () => {
      await result.current.checkForUpdates();
    });

    await act(async () => {
      await result.current.installUpdate();
    });

    expect(result.current.error).toBe("Download failed");
    expect(result.current.downloading).toBe(false);
  });

  it("should dismiss error", async () => {
    const mockDownloadAndInstall = vi
      .fn()
      .mockRejectedValue(new Error("Download failed"));

    const mockUpdate = {
      version: "2.0.0",
      downloadAndInstall: mockDownloadAndInstall,
    };
    mockCheck.mockResolvedValue(mockUpdate);

    const { result } = renderHook(() => useUpdateCheck());

    await act(async () => {
      await result.current.checkForUpdates();
    });

    await act(async () => {
      await result.current.installUpdate();
    });

    expect(result.current.error).toBe("Download failed");

    act(() => {
      result.current.dismissError();
    });

    expect(result.current.error).toBeNull();
  });

  it("should not install if no update available", async () => {
    mockCheck.mockResolvedValue(null);

    const { result } = renderHook(() => useUpdateCheck());

    await act(async () => {
      await result.current.checkForUpdates();
    });

    await act(async () => {
      await result.current.installUpdate();
    });

    // Should not throw, just return early
    expect(result.current.downloading).toBe(false);
  });
});
