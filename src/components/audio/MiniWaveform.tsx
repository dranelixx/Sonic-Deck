import { useEffect, useRef, useState, memo } from "react";
import { waveformQueue } from "../../utils/waveformQueue";

interface MiniWaveformProps {
  filePath: string;
  isPlaying?: boolean;
  height?: number;
}

function MiniWaveformComponent({
  filePath,
  isPlaying = false,
  height = 24,
}: MiniWaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [peaks, setPeaks] = useState<number[] | null>(null);
  const [isVisible, setIsVisible] = useState(false);
  const loadedPathRef = useRef<string>("");

  // Intersection Observer for lazy loading
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setIsVisible(true);
          }
        });
      },
      { rootMargin: "200px" } // Start loading 200px before visible
    );

    observer.observe(container);

    return () => {
      observer.disconnect();
    };
  }, []);

  // Load waveform data from backend (only when visible, with delay)
  useEffect(() => {
    if (!isVisible) return;
    if (loadedPathRef.current === filePath && peaks) {
      return;
    }

    loadedPathRef.current = filePath;

    // Use global queue to throttle requests
    waveformQueue
      .add(filePath, 40)
      .then((data) => {
        setPeaks(data.peaks);
      })
      .catch((err) => {
        console.error("Failed to load waveform:", err);
      });
  }, [filePath, isVisible]);

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
    canvas.height = height * dpr;

    // Scale context to match device pixel ratio
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const barWidth = width / peaks.length;
    const midY = height / 2;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Set color based on playing state
    ctx.fillStyle = isPlaying ? "#ffffff" : "#5865f2";

    // Draw bars
    for (let i = 0; i < peaks.length; i++) {
      const barHeight = Math.max(2, peaks[i] * midY * 0.9);
      const x = i * barWidth;

      // Draw bar centered vertically with spacing
      const actualBarWidth = Math.max(1, barWidth - 1);
      ctx.fillRect(x, midY - barHeight, actualBarWidth, barHeight * 2);
    }
  }, [peaks, isPlaying, height]);

  return (
    <div ref={containerRef} className="w-full">
      <canvas
        ref={canvasRef}
        className="w-full rounded"
        style={{ height: `${height}px` }}
      />
    </div>
  );
}

// Memoize to prevent unnecessary re-renders
const MiniWaveform = memo(MiniWaveformComponent);
export default MiniWaveform;
