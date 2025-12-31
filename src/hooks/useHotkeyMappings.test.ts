import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useHotkeyMappings } from "./useHotkeyMappings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("useHotkeyMappings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should initialize with empty mappings", () => {
    vi.mocked(invoke).mockResolvedValue({ mappings: {} });
    const { result } = renderHook(() => useHotkeyMappings());
    expect(result.current.hotkeyMappings).toEqual({ mappings: {} });
  });

  it("should load hotkeys on mount", async () => {
    const mockMappings = { mappings: { "Ctrl+1": "sound-1" } };
    vi.mocked(invoke).mockResolvedValue(mockMappings);

    const { result } = renderHook(() => useHotkeyMappings());

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("load_hotkeys");
    });

    await waitFor(() => {
      expect(result.current.hotkeyMappings).toEqual(mockMappings);
    });
  });

  it("should refresh hotkeys when called", async () => {
    const initialMappings = { mappings: {} };
    const updatedMappings = { mappings: { "Ctrl+2": "sound-2" } };

    vi.mocked(invoke)
      .mockResolvedValueOnce(initialMappings)
      .mockResolvedValueOnce(updatedMappings);

    const { result } = renderHook(() => useHotkeyMappings());

    // Wait for initial load
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("load_hotkeys");
    });

    // Call refreshHotkeys
    await act(async () => {
      await result.current.refreshHotkeys();
    });

    expect(invoke).toHaveBeenCalledTimes(2);
    expect(result.current.hotkeyMappings).toEqual(updatedMappings);
  });

  it("should handle invoke errors gracefully on mount", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(invoke).mockRejectedValue(new Error("Load failed"));

    renderHook(() => useHotkeyMappings());

    await waitFor(() => {
      expect(consoleSpy).toHaveBeenCalledWith(
        "Failed to load hotkey mappings:",
        expect.any(Error)
      );
    });

    consoleSpy.mockRestore();
  });

  it("should handle invoke errors gracefully on refresh", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    vi.mocked(invoke)
      .mockResolvedValueOnce({ mappings: {} })
      .mockRejectedValueOnce(new Error("Refresh failed"));

    const { result } = renderHook(() => useHotkeyMappings());

    // Wait for initial load
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("load_hotkeys");
    });

    // Call refreshHotkeys which should fail
    await act(async () => {
      await result.current.refreshHotkeys();
    });

    expect(consoleSpy).toHaveBeenCalledWith(
      "Failed to refresh hotkey mappings:",
      expect.any(Error)
    );

    consoleSpy.mockRestore();
  });

  it("should return refreshHotkeys function", () => {
    vi.mocked(invoke).mockResolvedValue({ mappings: {} });
    const { result } = renderHook(() => useHotkeyMappings());
    expect(typeof result.current.refreshHotkeys).toBe("function");
  });
});
