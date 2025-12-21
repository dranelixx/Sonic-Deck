import { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sound } from "../../types";

interface TrimEditorProps {
  sound: Sound;
  onClose: () => void;
  onSave: (trimStartMs: number | null, trimEndMs: number | null) => void;
}

interface WaveformData {
  peaks: number[];
  duration_ms: number;
}

export default function TrimEditor({
  sound,
  onClose,
  onSave,
}: TrimEditorProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const [waveformData, setWaveformData] = useState<WaveformData | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Trim region state (in milliseconds)
  const [trimStart, setTrimStart] = useState<number>(sound.trim_start_ms || 0);
  const [trimEnd, setTrimEnd] = useState<number | null>(sound.trim_end_ms);

  // Drag state
  const [isDragging, setIsDragging] = useState<"start" | "end" | null>(null);

  // Load waveform data
  useEffect(() => {
    setIsLoading(true);
    invoke<WaveformData>("get_waveform", {
      filePath: sound.file_path,
      numPeaks: 400, // Good resolution for editor
    })
      .then((data) => {
        setWaveformData(data);
        // Initialize trim end if not set
        if (!trimEnd) {
          setTrimEnd(data.duration_ms);
        }
        setIsLoading(false);
      })
      .catch((err) => {
        console.error("Failed to load waveform:", err);
        setIsLoading(false);
      });
  }, [sound.file_path]);

  // Draw waveform with trim region
  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container || !waveformData || !waveformData.peaks.length)
      return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Get actual container dimensions
    const rect = container.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;

    // Set canvas size with device pixel ratio for sharp rendering
    canvas.width = rect.width * dpr;
    canvas.height = 120 * dpr;

    // Scale context to match device pixel ratio
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const height = 120;
    const barWidth = width / waveformData.peaks.length;
    const midY = height / 2;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Calculate trim positions
    const duration = waveformData.duration_ms;
    const startX = (trimStart / duration) * width;
    const endX = ((trimEnd || duration) / duration) * width;

    // Draw waveform bars
    for (let i = 0; i < waveformData.peaks.length; i++) {
      const barHeight = Math.max(2, waveformData.peaks[i] * midY * 0.85);
      const x = i * barWidth;
      const barMidX = x + barWidth / 2;

      // Determine color based on trim region
      let color = "#4e5058"; // Outside trim (muted)
      if (barMidX >= startX && barMidX <= endX) {
        color = "#5865f2"; // Inside trim (highlighted)
      }

      ctx.fillStyle = color;
      const actualBarWidth = Math.max(1, barWidth - 0.5);
      ctx.fillRect(x, midY - barHeight, actualBarWidth, barHeight * 2);
    }

    // Draw trim region borders
    ctx.strokeStyle = "#ffffff";
    ctx.lineWidth = 3;

    // Start line
    ctx.beginPath();
    ctx.moveTo(startX, 0);
    ctx.lineTo(startX, height);
    ctx.stroke();

    // End line
    ctx.beginPath();
    ctx.moveTo(endX, 0);
    ctx.lineTo(endX, height);
    ctx.stroke();
  }, [waveformData, trimStart, trimEnd]);

  // Handle mouse down on trim handles
  const handleMouseDown = useCallback(
    (e: React.MouseEvent, handle: "start" | "end") => {
      e.preventDefault();
      setIsDragging(handle);
    },
    []
  );

  // Handle mouse move for dragging
  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!isDragging || !containerRef.current || !waveformData) return;

      const rect = containerRef.current.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const percentage = Math.max(0, Math.min(1, x / rect.width));
      const ms = Math.round(percentage * waveformData.duration_ms);

      if (isDragging === "start") {
        const maxStart = (trimEnd || waveformData.duration_ms) - 100; // Min 100ms region
        setTrimStart(Math.min(ms, maxStart));
      } else if (isDragging === "end") {
        const minEnd = trimStart + 100; // Min 100ms region
        setTrimEnd(Math.max(ms, minEnd));
      }
    },
    [isDragging, waveformData, trimStart, trimEnd]
  );

  // Handle mouse up
  const handleMouseUp = useCallback(() => {
    setIsDragging(null);
  }, []);

  // Attach/detach global mouse listeners
  useEffect(() => {
    if (isDragging) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);
      return () => {
        window.removeEventListener("mousemove", handleMouseMove);
        window.removeEventListener("mouseup", handleMouseUp);
      };
    }
  }, [isDragging, handleMouseMove, handleMouseUp]);

  // Format time helper
  const formatTime = (ms: number): string => {
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    const millis = Math.floor((ms % 1000) / 10);
    return `${minutes}:${seconds.toString().padStart(2, "0")}.${millis.toString().padStart(2, "0")}`;
  };

  // Handle save
  const handleSave = async () => {
    try {
      // Convert 0 to null for trim_start (0 means no trim at start)
      const finalTrimStart = trimStart === 0 ? null : trimStart;

      await invoke("update_sound", {
        soundId: sound.id,
        name: sound.name,
        filePath: sound.file_path,
        categoryId: sound.category_id,
        icon: sound.icon,
        volume: sound.volume,
        trimStartMs: finalTrimStart,
        trimEndMs: trimEnd,
      });

      onSave(finalTrimStart, trimEnd);
      onClose();
    } catch (error) {
      console.error("Failed to save trim:", error);
      alert(`Failed to save trim: ${error}`);
    }
  };

  // Handle reset
  const handleReset = () => {
    setTrimStart(0);
    setTrimEnd(waveformData?.duration_ms || null);
  };

  if (isLoading || !waveformData) {
    return (
      <div className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center">
        <div className="bg-discord-darker rounded-lg p-8">
          <p className="text-discord-text">Loading waveform...</p>
        </div>
      </div>
    );
  }

  const duration = waveformData.duration_ms;
  const trimmedDuration = (trimEnd || duration) - trimStart;

  return (
    <div className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-6">
      <div className="bg-discord-darker rounded-lg border border-discord-dark max-w-4xl w-full p-6">
        {/* Header */}
        <div className="mb-6">
          <h2 className="text-2xl font-bold text-discord-text mb-2">
            {sound.name}
          </h2>
          <p className="text-sm text-discord-text-muted">
            Drag the white lines to trim the audio
          </p>
        </div>

        {/* Waveform Editor */}
        <div
          ref={containerRef}
          className="relative bg-discord-darkest rounded-lg p-4 mb-6"
          style={{ cursor: isDragging ? "col-resize" : "default" }}
        >
          <canvas
            ref={canvasRef}
            className="w-full rounded"
            style={{ height: "120px" }}
          />

          {/* Trim handles */}
          <div
            className="absolute top-0 bottom-0 w-2 bg-white/20 hover:bg-white/40 cursor-col-resize"
            style={{ left: `${(trimStart / duration) * 100}%` }}
            onMouseDown={(e) => handleMouseDown(e, "start")}
          />
          <div
            className="absolute top-0 bottom-0 w-2 bg-white/20 hover:bg-white/40 cursor-col-resize"
            style={{ left: `${((trimEnd || duration) / duration) * 100}%` }}
            onMouseDown={(e) => handleMouseDown(e, "end")}
          />
        </div>

        {/* Time Info */}
        <div className="grid grid-cols-3 gap-4 mb-6 text-sm">
          <div className="bg-discord-dark rounded-lg p-3">
            <div className="text-discord-text-muted mb-1">Start Time</div>
            <div className="text-discord-text font-mono text-lg">
              {formatTime(trimStart)}
            </div>
          </div>
          <div className="bg-discord-dark rounded-lg p-3">
            <div className="text-discord-text-muted mb-1">End Time</div>
            <div className="text-discord-text font-mono text-lg">
              {formatTime(trimEnd || duration)}
            </div>
          </div>
          <div className="bg-discord-dark rounded-lg p-3">
            <div className="text-discord-text-muted mb-1">Trimmed Duration</div>
            <div className="text-discord-text font-mono text-lg">
              {formatTime(trimmedDuration)}
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex justify-between">
          <button
            onClick={handleReset}
            className="px-4 py-2 bg-discord-warning hover:bg-yellow-600 rounded-lg
                     text-white font-medium transition-colors"
          >
            Reset to Full
          </button>
          <div className="flex gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 bg-discord-danger hover:bg-red-600 rounded-lg
                       text-white transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              className="px-6 py-2 bg-discord-primary hover:bg-discord-primary-hover rounded-lg
                       text-white font-medium transition-colors"
            >
              Save Trim
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
