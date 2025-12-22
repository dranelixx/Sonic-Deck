import FullWaveform from "../audio/FullWaveform";
import { ActiveWaveform } from "../../hooks/useAudioPlayback";

interface DashboardHeaderProps {
  volume: number;
  onVolumeChange: (volume: number) => void;
  onStopAll: () => void;
  activeWaveform: ActiveWaveform | null;
  isWaveformExiting: boolean;
  playingSoundIds: Set<string>;
}

export default function DashboardHeader({
  volume,
  onVolumeChange,
  onStopAll,
  activeWaveform,
  isWaveformExiting,
  playingSoundIds,
}: DashboardHeaderProps) {
  return (
    <div className="bg-discord-darker px-6 py-4 border-b border-discord-dark">
      <div className="flex items-center justify-between gap-4 h-10">
        {/* Left: Logo + Waveform */}
        <div className="flex items-center gap-4 flex-1 min-w-0 h-full">
          <div className="flex-shrink-0 flex items-center h-full">
            <h1
              className="text-xl font-bold text-discord-primary leading-none"
              style={{ transform: "translateY(-3px)" }}
            >
              Sonic Deck
            </h1>
          </div>

          {/* Waveform (fades in when playing) */}
          <div className="flex-1 min-w-0 h-full flex items-center">
            {activeWaveform && (
              <div
                className={`w-full transition-opacity duration-300 ${
                  isWaveformExiting ? "opacity-0" : "opacity-100 animate-fadeIn"
                }`}
              >
                <FullWaveform
                  filePath={activeWaveform.filePath}
                  soundName={activeWaveform.soundName}
                  isPlaying={playingSoundIds.has(activeWaveform.soundId)}
                  currentTimeMs={activeWaveform.currentTimeMs}
                  durationMs={activeWaveform.durationMs}
                  trimStartMs={activeWaveform.trimStartMs}
                  trimEndMs={activeWaveform.trimEndMs}
                />
              </div>
            )}
          </div>
        </div>

        {/* Right: Controls */}
        <div className="flex items-center gap-3 flex-shrink-0 h-full">
          {/* Volume Control */}
          <div className="flex items-center gap-2">
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={volume}
              onChange={(e) => onVolumeChange(parseFloat(e.target.value))}
              className="w-20"
              style={{
                accentColor: volume >= 0.75 ? "#ef4444" : "#5865f2",
              }}
            />
            <span
              className={`text-xs w-8 ${volume >= 0.75 ? "text-red-400" : "text-discord-text"}`}
            >
              {Math.round(volume * 100)}%
            </span>
          </div>

          {/* Stop Button */}
          <button
            onClick={onStopAll}
            className="px-4 py-2 bg-discord-danger hover:bg-red-600 rounded-lg
                       text-white font-medium transition-colors"
          >
            Stop
          </button>
        </div>
      </div>
    </div>
  );
}
