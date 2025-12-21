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
    </div>
  );
}
