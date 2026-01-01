import { AppSettings } from "../../types";

interface PlaybackSettingsProps {
  settings: AppSettings;
  onUpdateSetting: (
    key: keyof AppSettings,
    value: string | number | boolean | null
  ) => void;
}

export default function PlaybackSettings({
  settings,
  onUpdateSetting,
}: PlaybackSettingsProps) {
  return (
    <div className="bg-discord-dark rounded-lg p-6 space-y-4">
      <h2 className="text-xl font-semibold text-discord-text mb-4">
        Playback Preferences
      </h2>

      {/* Default Volume */}
      <div>
        <label className="block text-sm font-medium text-discord-text mb-2">
          Default Volume:{" "}
          <span
            className={
              settings.default_volume >= 0.75 ? "text-red-500 font-bold" : ""
            }
          >
            {Math.round(settings.default_volume * 100)}%
          </span>
          {settings.default_volume >= 0.75 && (
            <span className="ml-2 text-xs text-red-400">⚠️ High volume</span>
          )}
        </label>
        <div className="relative">
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={settings.default_volume}
            onChange={(e) =>
              onUpdateSetting("default_volume", parseFloat(e.target.value))
            }
            className="w-full"
            style={{
              accentColor:
                settings.default_volume >= 0.75 ? "#ef4444" : "#5865f2",
            }}
          />
          {settings.default_volume >= 0.75 && (
            <div className="absolute -top-1 left-0 w-full h-2 bg-red-500/20 rounded -z-10"></div>
          )}
        </div>
        <p className="text-xs text-discord-text-muted mt-1">
          This volume will be used by default for new sound playbacks.
          Recommended: 50% or lower for hearing protection.
        </p>
      </div>

      {/* Global Volume Boost */}
      <div>
        <label className="flex items-center gap-2 text-sm font-medium text-discord-text mb-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.volume_multiplier > 1.0}
            onChange={(e) => {
              // Toggle: 1.0 = disabled, 2.0 = enabled with 2x boost
              onUpdateSetting(
                "volume_multiplier",
                e.target.checked ? 2.0 : 1.0
              );
            }}
            className="rounded border-discord-dark bg-discord-darker
                     text-discord-primary focus:ring-discord-primary cursor-pointer"
          />
          <span>Global Volume Boost</span>
          {settings.volume_multiplier > 1.0 && (
            <span className="ml-1 font-bold">
              (+
              {Math.round(
                (Math.min(3.0, settings.volume_multiplier) - 1.0) * 100
              )}
              %)
            </span>
          )}
        </label>

        {settings.volume_multiplier > 1.0 && (
          <>
            <div className="relative mt-2">
              <input
                type="range"
                min="1.1"
                max="3.0"
                step="0.1"
                value={Math.min(3.0, settings.volume_multiplier)}
                onChange={(e) =>
                  onUpdateSetting(
                    "volume_multiplier",
                    parseFloat(e.target.value)
                  )
                }
                className="w-full"
                style={{
                  accentColor: "#5865f2",
                }}
              />
            </div>
            <p className="text-xs text-discord-text-muted mt-1">
              Amplifies all sounds beyond their normal volume. Range: +10% to
              +200%. Use if sounds are too quiet.
            </p>
          </>
        )}

        {settings.volume_multiplier <= 1.0 && (
          <p className="text-xs text-discord-text-muted mt-1">
            Sounds play at normal Windows Media Player volume (no boost
            applied).
          </p>
        )}
      </div>

      {/* Loudness Normalization */}
      <div className="space-y-4">
        <h3 className="text-sm font-medium text-zinc-400">
          Loudness Normalization
        </h3>

        {/* Enable toggle */}
        <div className="flex items-center justify-between">
          <div>
            <label className="text-sm text-discord-text">
              Enable loudness normalization
            </label>
            <p className="text-xs text-discord-text-muted">
              Automatically adjust volume so all sounds play at similar loudness
            </p>
          </div>
          <button
            onClick={() =>
              onUpdateSetting(
                "enable_lufs_normalization",
                !settings.enable_lufs_normalization
              )
            }
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
              settings.enable_lufs_normalization
                ? "bg-discord-primary"
                : "bg-zinc-600"
            }`}
            aria-label="Toggle loudness normalization"
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                settings.enable_lufs_normalization
                  ? "translate-x-6"
                  : "translate-x-1"
              }`}
            />
          </button>
        </div>

        {/* Target loudness slider - only visible when enabled */}
        {settings.enable_lufs_normalization && (
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm text-discord-text">
                Target loudness
              </label>
              <span className="text-sm text-discord-text-muted">
                {settings.target_lufs} LUFS
                {settings.target_lufs === -14 && (
                  <span className="ml-1 text-xs text-zinc-500">
                    (Streaming standard)
                  </span>
                )}
              </span>
            </div>
            <input
              type="range"
              min="-23"
              max="-7"
              step="1"
              value={settings.target_lufs}
              onChange={(e) =>
                onUpdateSetting("target_lufs", parseFloat(e.target.value))
              }
              className="w-full h-2 bg-zinc-700 rounded-lg appearance-none cursor-pointer"
              style={{ accentColor: "#5865f2" }}
              aria-label="Target loudness level"
            />
            <div className="flex justify-between text-xs text-discord-text-muted">
              <span>Quieter (-23)</span>
              <span>Louder (-7)</span>
            </div>
            <p className="text-xs text-discord-text-muted">
              -14 LUFS is the standard for YouTube, Spotify, and most streaming
              platforms.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
