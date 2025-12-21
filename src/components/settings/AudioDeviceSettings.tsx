import { AudioDevice, AppSettings } from "../../types";

interface AudioDeviceSettingsProps {
  settings: AppSettings;
  devices: AudioDevice[];
  isRefreshing: boolean;
  onRefresh: () => void;
  onUpdateSetting: (key: keyof AppSettings, value: string | null) => void;
  isDeviceAvailable: (deviceId: string) => boolean;
}

export default function AudioDeviceSettings({
  settings,
  devices,
  isRefreshing,
  onRefresh,
  onUpdateSetting,
  isDeviceAvailable,
}: AudioDeviceSettingsProps) {
  return (
    <div className="bg-discord-dark rounded-lg p-6 space-y-4">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-discord-text">
          Audio Devices
        </h2>
        <button
          onClick={onRefresh}
          disabled={isRefreshing}
          className="px-3 py-1.5 text-sm bg-discord-primary hover:bg-discord-primary-hover 
                   disabled:bg-gray-600 disabled:cursor-not-allowed rounded text-white 
                   font-medium transition-colors"
        >
          {isRefreshing ? "Refreshing..." : "Refresh"}
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Monitor Output Device */}
        <div>
          <label className="block text-sm font-medium text-discord-text mb-2">
            Monitor Output
            <span className="text-discord-text-muted text-xs ml-2">
              (Your headphones/speakers)
            </span>
          </label>
          <select
            value={settings.monitor_device_id || ""}
            onChange={(e) =>
              onUpdateSetting("monitor_device_id", e.target.value || null)
            }
            className="w-full bg-discord-darker border border-discord-dark rounded px-3 py-2 
                     text-discord-text focus:outline-none focus:ring-2 focus:ring-discord-primary"
          >
            <option value="">Not configured</option>
            {devices.map((device) => (
              <option key={device.id} value={device.id}>
                {device.name} {device.is_default ? "(Default)" : ""}
              </option>
            ))}
          </select>
          {settings.monitor_device_id &&
            !isDeviceAvailable(settings.monitor_device_id) && (
              <p className="text-xs text-discord-danger mt-1">
                ⚠️ Device not available
              </p>
            )}
          {settings.monitor_device_id &&
            isDeviceAvailable(settings.monitor_device_id) && (
              <p className="text-xs text-discord-success mt-1">
                ✓ Device online
              </p>
            )}
        </div>

        {/* Broadcast Output Device */}
        <div>
          <label className="block text-sm font-medium text-discord-text mb-2">
            Broadcast Output
            <span className="text-discord-text-muted text-xs ml-2">
              (Virtual cable/stream)
            </span>
          </label>
          <select
            value={settings.broadcast_device_id || ""}
            onChange={(e) =>
              onUpdateSetting("broadcast_device_id", e.target.value || null)
            }
            className="w-full bg-discord-darker border border-discord-dark rounded px-3 py-2 
                     text-discord-text focus:outline-none focus:ring-2 focus:ring-discord-primary"
          >
            <option value="">Not configured</option>
            {devices.map((device) => (
              <option key={device.id} value={device.id}>
                {device.name} {device.is_default ? "(Default)" : ""}
              </option>
            ))}
          </select>
          {settings.broadcast_device_id &&
            !isDeviceAvailable(settings.broadcast_device_id) && (
              <p className="text-xs text-discord-danger mt-1">
                ⚠️ Device not available
              </p>
            )}
          {settings.broadcast_device_id &&
            isDeviceAvailable(settings.broadcast_device_id) && (
              <p className="text-xs text-discord-success mt-1">
                ✓ Device online
              </p>
            )}
        </div>
      </div>

      {/* Warning if both devices are the same */}
      {settings.monitor_device_id &&
        settings.broadcast_device_id &&
        settings.monitor_device_id === settings.broadcast_device_id && (
          <div className="bg-discord-warning/20 border border-discord-warning rounded p-3">
            <p className="text-sm text-discord-warning">
              ⚠️ Warning: Both outputs are set to the same device. For
              dual-output routing, select different devices.
            </p>
          </div>
        )}
    </div>
  );
}
