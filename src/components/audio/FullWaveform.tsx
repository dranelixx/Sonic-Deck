import { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface WaveformData {
  peaks: number[];
  duration_ms: number;
}

interface FullWaveformProps {
  filePath: string;
  soundName: string;
  isPlaying: boolean;
  currentTimeMs: number;
  durationMs: number;
  trimStartMs: number | null;
  trimEndMs: number | null;
}

export default function FullWaveform({
  filePath,
  soundName,
  isPlaying,
  currentTimeMs,
  durationMs,
  trimStartMs,
  trimEndMs,
}: FullWaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [peaks, setPeaks] = useState<number[] | null>(null);
  const [actualDuration, setActualDuration] = useState(durationMs);
  const loadedPathRef = useRef<string>("");

  // Load waveform data from backend
  useEffect(() => {
    if (loadedPathRef.current === filePath && peaks) {
      return;
    }

    loadedPathRef.current = filePath;

    invoke<WaveformData>("get_waveform", {
      filePath,
      numPeaks: 250, // Higher resolution - not in viewport initially
    })
      .then((data) => {
        setPeaks(data.peaks);
        setActualDuration(data.duration_ms);
      })
      .catch((err) => {
        console.error("Failed to load waveform:", err);
      });
  }, [filePath]);

  // Draw waveform on canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container || !peaks || peaks.length === 0) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Get actual container dimensions
    const rect = container.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;

    // Set canvas size with device pixel ratio for sharp rendering
    canvas.width = rect.width * dpr;
    canvas.height = 32 * dpr;

    // Scale context to match device pixel ratio
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const height = 32;
    const midY = height / 2;

    // Calculate progress
    // If trimmed, the duration from backend is already the trimmed duration
    const effectiveDuration = durationMs > 0 ? durationMs : actualDuration;
    const progress =
      effectiveDuration > 0 ? currentTimeMs / effectiveDuration : 0;
    const progressX = progress * width;

    // Calculate trim range for waveform display
    // The waveform peaks represent the full audio, so we need to show only the trimmed portion
    const totalDuration = actualDuration;
    const startMs = trimStartMs ?? 0;
    const endMs = trimEndMs ?? totalDuration;

    // Calculate which portion of peaks to show
    const startRatio = totalDuration > 0 ? startMs / totalDuration : 0;
    const endRatio = totalDuration > 0 ? endMs / totalDuration : 1;
    const startPeakIdx = Math.floor(startRatio * peaks.length);
    const endPeakIdx = Math.ceil(endRatio * peaks.length);
    const visiblePeaks = peaks.slice(startPeakIdx, endPeakIdx);

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Draw bars (only visible peaks from trimmed range)
    const visibleBarWidth = width / visiblePeaks.length;
    for (let i = 0; i < visiblePeaks.length; i++) {
      const barHeight = Math.max(2, visiblePeaks[i] * midY * 0.85);
      const x = i * visibleBarWidth;

      // Color based on progress
      const barMidX = x + visibleBarWidth / 2;
      ctx.fillStyle = barMidX < progressX ? "#5865f2" : "#4e5058";

      // Draw bar centered vertically with spacing
      const actualBarWidth = Math.max(1, visibleBarWidth - 0.5);
      ctx.fillRect(x, midY - barHeight, actualBarWidth, barHeight * 2);
    }

    // Draw progress cursor
    if (isPlaying && progress > 0 && progress < 1) {
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(progressX, 0);
      ctx.lineTo(progressX, height);
      ctx.stroke();
    }
  }, [
    peaks,
    currentTimeMs,
    durationMs,
    actualDuration,
    isPlaying,
    trimStartMs,
    trimEndMs,
  ]);

  const formatTime = useCallback((ms: number): string => {
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }, []);

  const effectiveDuration = durationMs > 0 ? durationMs : actualDuration;

  return (
    <div className="w-full flex items-center gap-3">
      {/* Waveform Canvas */}
      <div ref={containerRef} className="flex-1">
        <canvas
          ref={canvasRef}
          className="w-full h-8 rounded bg-discord-darker"
        />
      </div>

      {/* Compact Info */}
      <div className="flex items-center gap-3 flex-shrink-0">
        <div className="text-xs text-discord-text truncate max-w-[150px]">
          {soundName}
        </div>
        <div className="flex gap-1 text-xs text-discord-text-muted whitespace-nowrap">
          <span>{formatTime(currentTimeMs)}</span>
          <span>/</span>
          <span>{formatTime(effectiveDuration)}</span>
        </div>
      </div>
    </div>
  );
}
