import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { waveformQueue } from "./waveformQueue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("WaveformQueue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should call invoke with correct parameters", async () => {
    const mockData = { peaks: [0.5, 0.7, 0.3], duration_ms: 1000 };
    vi.mocked(invoke).mockResolvedValueOnce(mockData);

    const result = await waveformQueue.add("/path/to/audio.mp3", 100);

    expect(invoke).toHaveBeenCalledWith("get_waveform", {
      filePath: "/path/to/audio.mp3",
      numPeaks: 100,
    });
    expect(result).toEqual(mockData);
  });

  it("should return waveform data on success", async () => {
    const mockData = {
      peaks: [0.1, 0.2, 0.3, 0.4, 0.5],
      duration_ms: 5000,
    };
    vi.mocked(invoke).mockResolvedValueOnce(mockData);

    const result = await waveformQueue.add("/test/file.ogg", 50);

    expect(result.peaks).toHaveLength(5);
    expect(result.duration_ms).toBe(5000);
  });

  it("should reject on error", async () => {
    const error = new Error("Failed to decode audio");
    vi.mocked(invoke).mockRejectedValueOnce(error);

    await expect(waveformQueue.add("/invalid/file.mp3", 100)).rejects.toThrow(
      "Failed to decode audio"
    );
  });

  it("should process multiple requests sequentially", async () => {
    const mockData1 = { peaks: [0.1], duration_ms: 100 };
    const mockData2 = { peaks: [0.2], duration_ms: 200 };

    vi.mocked(invoke)
      .mockResolvedValueOnce(mockData1)
      .mockResolvedValueOnce(mockData2);

    // Add both requests simultaneously
    const promise1 = waveformQueue.add("/file1.mp3", 10);
    const promise2 = waveformQueue.add("/file2.mp3", 20);

    const [result1, result2] = await Promise.all([promise1, promise2]);

    expect(result1).toEqual(mockData1);
    expect(result2).toEqual(mockData2);
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("should handle mixed success and failure", async () => {
    const mockData = { peaks: [0.5], duration_ms: 500 };
    const error = new Error("Decode failed");

    vi.mocked(invoke)
      .mockResolvedValueOnce(mockData)
      .mockRejectedValueOnce(error);

    const promise1 = waveformQueue.add("/success.mp3", 10);
    const promise2 = waveformQueue.add("/fail.mp3", 10);

    const result1 = await promise1;
    expect(result1).toEqual(mockData);

    await expect(promise2).rejects.toThrow("Decode failed");
  });
});
