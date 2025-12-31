import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useAudioPlayback } from "./useAudioPlayback";
import { Sound, PlaybackResult } from "../types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("../constants", () => ({
  DEBUG: false,
  ANIMATION_DURATIONS: { WAVEFORM_EXIT: 300 },
}));

describe("useAudioPlayback", () => {
  const mockShowToast = vi.fn();
  const defaultProps = {
    device1: "device-1-id",
    device2: "device-2-id",
    volume: 0.8,
    showToast: mockShowToast,
    soundLibrary: { sounds: [] },
  };

  const createMockSound = (overrides: Partial<Sound> = {}): Sound => ({
    id: "sound-123",
    name: "Test Sound",
    file_path: "/path/to/test.mp3",
    category_id: "category-1",
    icon: null,
    volume: null,
    is_favorite: false,
    trim_start_ms: null,
    trim_end_ms: null,
    ...overrides,
  });

  const mockPlaybackResult: PlaybackResult = {
    playback_id: "playback-abc",
    action: "started",
    stopped_playback_id: null,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue(mockPlaybackResult);
  });

  // ===========================================================================
  // Task 1: Test playSound invoke parameters
  // ===========================================================================
  describe("playSound invoke parameters", () => {
    it("should call invoke with correct command name (play_dual_output)", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound();

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(invoke).toHaveBeenCalledWith(
        "play_dual_output",
        expect.any(Object)
      );
    });

    it("should pass file_path as filePath", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound({
        file_path: "/custom/path/audio.mp3",
      });

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(invoke).toHaveBeenCalledWith(
        "play_dual_output",
        expect.objectContaining({
          filePath: "/custom/path/audio.mp3",
        })
      );
    });

    it("should pass device IDs correctly", async () => {
      const props = {
        ...defaultProps,
        device1: "monitor-device-abc",
        device2: "broadcast-device-xyz",
      };
      const { result } = renderHook(() => useAudioPlayback(props));
      const mockSound = createMockSound();

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(invoke).toHaveBeenCalledWith(
        "play_dual_output",
        expect.objectContaining({
          deviceId1: "monitor-device-abc",
          deviceId2: "broadcast-device-xyz",
        })
      );
    });

    it("should use sound-specific volume when available", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound({ volume: 0.5 });

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(invoke).toHaveBeenCalledWith(
        "play_dual_output",
        expect.objectContaining({
          volume: 0.5,
        })
      );
    });

    it("should use global volume when sound has no specific volume", async () => {
      const props = { ...defaultProps, volume: 0.75 };
      const { result } = renderHook(() => useAudioPlayback(props));
      const mockSound = createMockSound({ volume: null });

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(invoke).toHaveBeenCalledWith(
        "play_dual_output",
        expect.objectContaining({
          volume: 0.75,
        })
      );
    });

    it("should pass trim parameters", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound({
        trim_start_ms: 1000,
        trim_end_ms: 5000,
      });

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(invoke).toHaveBeenCalledWith(
        "play_dual_output",
        expect.objectContaining({
          trimStartMs: 1000,
          trimEndMs: 5000,
        })
      );
    });

    it("should pass null trim parameters when not set", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound({
        trim_start_ms: null,
        trim_end_ms: null,
      });

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(invoke).toHaveBeenCalledWith(
        "play_dual_output",
        expect.objectContaining({
          trimStartMs: null,
          trimEndMs: null,
        })
      );
    });

    it("should pass soundId", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound({ id: "unique-sound-id-456" });

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(invoke).toHaveBeenCalledWith(
        "play_dual_output",
        expect.objectContaining({
          soundId: "unique-sound-id-456",
        })
      );
    });

    it("should pass all parameters correctly in single invoke call", async () => {
      const props = {
        ...defaultProps,
        device1: "dev1",
        device2: "dev2",
        volume: 0.6,
      };
      const { result } = renderHook(() => useAudioPlayback(props));
      const mockSound = createMockSound({
        id: "test-id",
        file_path: "/test/path.mp3",
        volume: 0.9,
        trim_start_ms: 100,
        trim_end_ms: 2000,
      });

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(invoke).toHaveBeenCalledWith("play_dual_output", {
        filePath: "/test/path.mp3",
        deviceId1: "dev1",
        deviceId2: "dev2",
        volume: 0.9,
        trimStartMs: 100,
        trimEndMs: 2000,
        soundId: "test-id",
      });
    });
  });

  // ===========================================================================
  // Task 2: Test device validation and error handling
  // ===========================================================================
  describe("device validation and error handling", () => {
    it("should show toast when device1 is empty", async () => {
      const props = { ...defaultProps, device1: "" };
      const { result } = renderHook(() => useAudioPlayback(props));
      const mockSound = createMockSound();

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(mockShowToast).toHaveBeenCalledWith(
        "Please configure audio devices in Settings first"
      );
      expect(invoke).not.toHaveBeenCalled();
    });

    it("should show toast when device2 is empty", async () => {
      const props = { ...defaultProps, device2: "" };
      const { result } = renderHook(() => useAudioPlayback(props));
      const mockSound = createMockSound();

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(mockShowToast).toHaveBeenCalledWith(
        "Please configure audio devices in Settings first"
      );
      expect(invoke).not.toHaveBeenCalled();
    });

    it("should show toast when both devices are empty", async () => {
      const props = { ...defaultProps, device1: "", device2: "" };
      const { result } = renderHook(() => useAudioPlayback(props));
      const mockSound = createMockSound();

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(mockShowToast).toHaveBeenCalledWith(
        "Please configure audio devices in Settings first"
      );
      expect(invoke).not.toHaveBeenCalled();
    });

    it("should handle invoke rejection gracefully and show error toast", async () => {
      const errorMessage = "Audio device not found";
      vi.mocked(invoke).mockRejectedValueOnce(new Error(errorMessage));

      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound();

      // Suppress console.error for this test
      const consoleErrorSpy = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(mockShowToast).toHaveBeenCalledWith(
        expect.stringContaining("Error:")
      );
      expect(consoleErrorSpy).toHaveBeenCalled();

      consoleErrorSpy.mockRestore();
    });

    it("should log error to console on invoke failure", async () => {
      const error = new Error("Playback failed");
      vi.mocked(invoke).mockRejectedValueOnce(error);

      const consoleErrorSpy = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});

      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound();

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(consoleErrorSpy).toHaveBeenCalledWith("Playback error:", error);

      consoleErrorSpy.mockRestore();
    });
  });

  // ===========================================================================
  // Task 3: Test state management and stopAllAudio
  // ===========================================================================
  describe("state management", () => {
    it("should initialize with no activeWaveform", () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      expect(result.current.activeWaveform).toBeNull();
    });

    it("should initialize with isWaveformExiting as false", () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      expect(result.current.isWaveformExiting).toBe(false);
    });

    it("should initialize with empty playingSoundIds", () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      expect(result.current.playingSoundIds.size).toBe(0);
    });

    it("should add sound to playingSoundIds on successful playback", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound({ id: "playing-sound-id" });

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(result.current.playingSoundIds.has("playing-sound-id")).toBe(true);
    });

    it("should set activeWaveform on successful playback", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound({
        id: "waveform-sound",
        name: "Waveform Test",
        file_path: "/waveform/test.mp3",
        trim_start_ms: 500,
        trim_end_ms: 3000,
      });

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(result.current.activeWaveform).toEqual({
        soundId: "waveform-sound",
        soundName: "Waveform Test",
        filePath: "/waveform/test.mp3",
        currentTimeMs: 0,
        durationMs: 0,
        trimStartMs: 500,
        trimEndMs: 3000,
      });
    });

    it("should not update state when playback action is ignored", async () => {
      vi.mocked(invoke).mockResolvedValueOnce({
        playback_id: null,
        action: "ignored",
        stopped_playback_id: null,
      });

      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound();

      await act(async () => {
        await result.current.playSound(mockSound);
      });

      expect(result.current.playingSoundIds.size).toBe(0);
      expect(result.current.activeWaveform).toBeNull();
    });
  });

  describe("stopAllAudio", () => {
    it("should call invoke with correct command (stop_all_audio)", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      await act(async () => {
        await result.current.stopAllAudio();
      });

      expect(invoke).toHaveBeenCalledWith("stop_all_audio");
    });

    it("should clear playingSoundIds after stopping", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));
      const mockSound = createMockSound();

      // First play a sound
      await act(async () => {
        await result.current.playSound(mockSound);
      });
      expect(result.current.playingSoundIds.size).toBe(1);

      // Then stop all audio
      await act(async () => {
        await result.current.stopAllAudio();
      });

      expect(result.current.playingSoundIds.size).toBe(0);
    });

    it("should set isWaveformExiting to true when stopping", async () => {
      vi.useFakeTimers();
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      // Play a sound first to have an active waveform
      const mockSound = createMockSound();
      await act(async () => {
        await result.current.playSound(mockSound);
      });

      await act(async () => {
        await result.current.stopAllAudio();
      });

      expect(result.current.isWaveformExiting).toBe(true);

      vi.useRealTimers();
    });

    it("should show toast message after stopping all audio", async () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      await act(async () => {
        await result.current.stopAllAudio();
      });

      expect(mockShowToast).toHaveBeenCalledWith("All audio stopped");
    });

    it("should show error toast when stop_all_audio fails", async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error("Stop failed"));

      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      await act(async () => {
        await result.current.stopAllAudio();
      });

      expect(mockShowToast).toHaveBeenCalledWith(
        expect.stringContaining("Stop Error:")
      );
    });
  });

  describe("setupAudioListeners", () => {
    it("should setup event listeners for audio events", () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      act(() => {
        result.current.setupAudioListeners();
      });

      expect(listen).toHaveBeenCalledWith(
        "audio-decode-complete",
        expect.any(Function)
      );
      expect(listen).toHaveBeenCalledWith(
        "audio-decode-error",
        expect.any(Function)
      );
      expect(listen).toHaveBeenCalledWith(
        "playback-complete",
        expect.any(Function)
      );
      expect(listen).toHaveBeenCalledWith(
        "playback-progress",
        expect.any(Function)
      );
    });

    it("should return cleanup function", () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      let cleanup: (() => void) | undefined;
      act(() => {
        cleanup = result.current.setupAudioListeners();
      });

      expect(cleanup).toBeDefined();
      expect(typeof cleanup).toBe("function");
    });
  });

  describe("return values", () => {
    it("should return all expected properties and functions", () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      expect(result.current).toHaveProperty("playingSoundIds");
      expect(result.current).toHaveProperty("activeWaveform");
      expect(result.current).toHaveProperty("isWaveformExiting");
      expect(result.current).toHaveProperty("playSound");
      expect(result.current).toHaveProperty("stopAllAudio");
      expect(result.current).toHaveProperty("setupAudioListeners");
    });

    it("should return playSound as a function", () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      expect(typeof result.current.playSound).toBe("function");
    });

    it("should return stopAllAudio as a function", () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      expect(typeof result.current.stopAllAudio).toBe("function");
    });

    it("should return setupAudioListeners as a function", () => {
      const { result } = renderHook(() => useAudioPlayback(defaultProps));

      expect(typeof result.current.setupAudioListeners).toBe("function");
    });
  });
});
