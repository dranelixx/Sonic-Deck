import { AppSettings } from "../../types";

interface SystemTraySettingsProps {
  settings: AppSettings;
  onUpdateSetting: (
    key: keyof AppSettings,
    value: string | number | boolean | null
  ) => void;
  onToggleAutostart: () => Promise<void>;
}

export default function SystemTraySettings({
  settings,
  onUpdateSetting,
  onToggleAutostart,
}: SystemTraySettingsProps) {
  return (
    <>
      {/* System Tray Preferences */}
      <div className="bg-discord-dark rounded-lg p-6 space-y-4">
        <h2 className="text-xl font-semibold text-discord-text mb-4">
          System Tray
        </h2>

        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.minimize_to_tray}
            onChange={(e) =>
              onUpdateSetting("minimize_to_tray", e.target.checked)
            }
            className="rounded border-discord-dark bg-discord-darker
                     text-discord-primary focus:ring-discord-primary cursor-pointer"
          />
          <span className="text-sm text-discord-text">
            Minimize to tray instead of closing
          </span>
        </label>
        <p className="text-xs text-discord-text-muted ml-6">
          When enabled, clicking the close button will minimize the app to the
          system tray instead of quitting.
        </p>

        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.start_minimized}
            onChange={(e) =>
              onUpdateSetting("start_minimized", e.target.checked)
            }
            className="rounded border-discord-dark bg-discord-darker
                     text-discord-primary focus:ring-discord-primary cursor-pointer"
          />
          <span className="text-sm text-discord-text">
            Start application minimized to tray
          </span>
        </label>
        <p className="text-xs text-discord-text-muted ml-6">
          Launch SonicDeck directly in the system tray on startup.
        </p>
      </div>

      {/* Startup Behavior */}
      <div className="bg-discord-dark rounded-lg p-6 space-y-4">
        <h2 className="text-xl font-semibold text-discord-text mb-4">
          Startup Behavior
        </h2>

        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={settings.autostart_enabled}
            onChange={onToggleAutostart}
            className="rounded border-discord-dark bg-discord-darker
                     text-discord-primary focus:ring-discord-primary cursor-pointer"
          />
          <span className="text-sm text-discord-text">
            Launch SonicDeck on system startup
          </span>
        </label>
        <p className="text-xs text-discord-text-muted ml-6">
          Automatically start SonicDeck when you log into Windows.
        </p>
      </div>
    </>
  );
}
