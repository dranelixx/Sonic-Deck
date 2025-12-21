import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../../types";
import { useAudio } from "../../contexts/AudioContext";
import { useSettings as useSettingsContext } from "../../contexts/SettingsContext";

export default function Settings() {
  // Contexts
  const { devices, refreshDevices } = useAudio();
  const {
    settings: contextSettings,
    saveSettings: saveSettingsToContext,
    reloadSettings,
  } = useSettingsContext();

  const [settings, setSettings] = useState<AppSettings>({
    monitor_device_id: null,
    broadcast_device_id: null,
    default_volume: 0.5,
    volume_multiplier: 1.0,
    last_file_path: null,
    minimize_to_tray: false,
    start_minimized: false,
    autostart_enabled: false,
  });
  const [isRefreshing, setIsRefreshing] = useState<boolean>(false);
  const [isSaving, setIsSaving] = useState<boolean>(false);
  const [status, setStatus] = useState<string>("");
  const [settingsPath, setSettingsPath] = useState<string>("");

  // Load settings path once
  useEffect(() => {
    const getSettingsPath = async () => {
      try {
        const path = await invoke<string>("get_settings_file_path");
        setSettingsPath(path);
      } catch (error) {
        console.error("Failed to get settings path:", error);
      }
    };
    getSettingsPath();
  }, []);

  // Update local state when context settings change
  useEffect(() => {
    if (contextSettings) {
      setSettings(contextSettings);
    }
  }, [contextSettings]);

  const handleRefreshDevices = async () => {
    try {
      setIsRefreshing(true);
      await refreshDevices();
      setStatus("Devices refreshed successfully");
      setTimeout(() => setStatus(""), 2000);
    } catch (error) {
      console.error("Failed to refresh devices:", error);
      setStatus(`Error loading devices: ${error}`);
    } finally {
      setIsRefreshing(false);
    }
  };

  const handleSaveSettings = async () => {
    try {
      setIsSaving(true);
      setStatus("");
      await saveSettingsToContext(settings);
      setIsSaving(false);
      setStatus("Settings saved successfully! ✓");
      setTimeout(() => setStatus(""), 3000);
    } catch (error) {
      console.error("Failed to save settings:", error);
      setIsSaving(false);
      setStatus(`Error saving settings: ${error}`);
    }
  };

  const handleResetSettings = async () => {
    try {
      await reloadSettings();
      setStatus("Settings reset to saved values");
      setTimeout(() => setStatus(""), 2000);
    } catch (error) {
      console.error("Failed to reset settings:", error);
      setStatus(`Error resetting settings: ${error}`);
    }
  };

  const updateSetting = <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
  };

  // Check if a device is currently available
  const isDeviceAvailable = (deviceId: string | null) => {
    if (!deviceId) return false;
    return devices.some((d) => d.id === deviceId);
  };

  return (
    <div className="w-full h-full bg-discord-darkest flex flex-col">
      {/* Header */}
      <div className="bg-discord-darker px-6 py-4 border-b border-discord-dark">
        <h1 className="text-2xl font-bold text-discord-primary">⚙️ Settings</h1>
        <p className="text-sm text-discord-text-muted mt-1">
          Configure your default audio routing and preferences
        </p>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-4xl mx-auto space-y-6">
          {/* Status Banner */}
          {status && (
            <div
              className={`rounded-lg p-4 border ${
                status.includes("Error")
                  ? "bg-red-900/20 border-discord-danger"
                  : "bg-green-900/20 border-discord-success"
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="text-discord-text font-medium">{status}</span>
              </div>
            </div>
          )}

          {/* Audio Device Configuration */}
          <div className="bg-discord-dark rounded-lg p-6 space-y-4">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-semibold text-discord-text">
                Audio Devices
              </h2>
              <button
                onClick={handleRefreshDevices}
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
                    updateSetting("monitor_device_id", e.target.value || null)
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
                    updateSetting("broadcast_device_id", e.target.value || null)
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

          {/* Playback Preferences */}
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
                    settings.default_volume >= 0.75
                      ? "text-red-500 font-bold"
                      : ""
                  }
                >
                  {Math.round(settings.default_volume * 100)}%
                </span>
                {settings.default_volume >= 0.75 && (
                  <span className="ml-2 text-xs text-red-400">
                    ⚠️ High volume
                  </span>
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
                    updateSetting("default_volume", parseFloat(e.target.value))
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
                    updateSetting(
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
                        updateSetting(
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
                    Amplifies all sounds beyond their normal volume. Range: +10%
                    to +200%. Use if sounds are too quiet.
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
                  updateSetting("minimize_to_tray", e.target.checked)
                }
                className="rounded border-discord-dark bg-discord-darker
                         text-discord-primary focus:ring-discord-primary cursor-pointer"
              />
              <span className="text-sm text-discord-text">
                Minimize to tray instead of closing
              </span>
            </label>
            <p className="text-xs text-discord-text-muted ml-6">
              When enabled, clicking the close button will minimize the app to
              the system tray instead of quitting.
            </p>

            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={settings.start_minimized}
                onChange={(e) =>
                  updateSetting("start_minimized", e.target.checked)
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
                onChange={async (e) => {
                  const enabled = e.target.checked;
                  updateSetting("autostart_enabled", enabled);
                  try {
                    if (enabled) {
                      await invoke("enable_autostart");
                    } else {
                      await invoke("disable_autostart");
                    }
                  } catch (err) {
                    setStatus(`Error: ${err}`);
                  }
                }}
                className="rounded border-discord-dark bg-discord-darker
                         text-discord-primary focus:ring-discord-primary cursor-pointer"
              />
              <span className="text-sm text-discord-text">
                Launch SonicDeck on system startup
              </span>
            </label>
            <p className="text-xs text-discord-text-muted ml-6">
              {settings.start_minimized
                ? "Will start minimized to tray"
                : "Will start normally in a window"}
            </p>
          </div>

          {/* Available Devices List */}
          <div className="bg-discord-dark rounded-lg p-6">
            <h3 className="text-lg font-semibold text-discord-text mb-3">
              Available Devices ({devices.length})
            </h3>
            <div className="space-y-2">
              {devices.length === 0 ? (
                <p className="text-sm text-discord-text-muted">
                  No audio devices found. Click "Refresh" to scan again.
                </p>
              ) : (
                devices.map((device) => (
                  <div
                    key={device.id}
                    className="bg-discord-darker rounded px-4 py-3 flex items-center justify-between"
                  >
                    <div className="flex items-center gap-3">
                      <span className="text-discord-text">{device.name}</span>
                      {device.is_default && (
                        <span className="px-2 py-0.5 bg-discord-success rounded text-xs text-white">
                          DEFAULT
                        </span>
                      )}
                    </div>
                    <div className="flex gap-2">
                      {settings.monitor_device_id === device.id && (
                        <span className="px-2 py-0.5 bg-blue-600 rounded text-xs text-white">
                          MONITOR
                        </span>
                      )}
                      {settings.broadcast_device_id === device.id && (
                        <span className="px-2 py-0.5 bg-purple-600 rounded text-xs text-white">
                          BROADCAST
                        </span>
                      )}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>

          {/* Info Section */}
          <div className="bg-discord-dark rounded-lg p-6">
            <h3 className="text-lg font-semibold text-discord-text mb-3">
              ℹ️ About Settings
            </h3>
            <div className="space-y-2 text-sm text-discord-text-muted">
              <p>
                • <strong>Monitor Output:</strong> Where you hear the sounds
                (your headphones/speakers)
              </p>
              <p>
                • <strong>Broadcast Output:</strong> Where your audience hears
                the sounds (virtual audio cable, OBS, etc.)
              </p>
              <p>
                • Settings are automatically saved to:{" "}
                <code className="text-xs bg-discord-darker px-2 py-0.5 rounded">
                  {settingsPath}
                </code>
              </p>
            </div>
          </div>

          {/* About SonicDeck Section */}
          <div className="bg-discord-dark rounded-lg p-6 border-l-4 border-discord-primary">
            <div className="flex items-center gap-3 mb-4">
              <span className="text-3xl">🎵</span>
              <div>
                <h3 className="text-xl font-bold text-discord-text">
                  SonicDeck
                </h3>
                <p className="text-sm text-discord-text-muted">
                  Version 0.1.0 Beta
                </p>
              </div>
            </div>

            <div className="space-y-3 text-sm">
              <p className="text-discord-text-muted">
                High-performance desktop soundboard built with Tauri v2, Rust,
                React, and TypeScript.
              </p>

              {/* Copyright */}
              <div className="pt-3 border-t border-discord-darker">
                <p className="text-xs text-discord-text-muted">
                  © 2025 Adrian Konopczynski (DraneLixX)
                </p>
                <p className="text-xs text-discord-text-muted mt-1">
                  Licensed under the{" "}
                  <a
                    href="https://github.com/DraneLixX/SonicDeck/blob/main/LICENSE"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-discord-primary hover:underline"
                  >
                    MIT License
                  </a>{" "}
                  • Open-Source Software
                </p>
              </div>

              {/* Contact & Support */}
              <div className="pt-3 border-t border-discord-darker">
                <h4 className="text-sm font-semibold text-discord-text mb-2">
                  📞 Contact & Support
                </h4>
                <div className="space-y-1.5 text-xs text-discord-text-muted">
                  <p>
                    📧 Email:{" "}
                    <a
                      href="mailto:adrikonop@gmail.com"
                      className="text-discord-primary hover:underline"
                    >
                      adrikonop@gmail.com
                    </a>
                  </p>
                  <p>
                    💬 Discord:{" "}
                    <span className="text-discord-text">dranelixx</span>
                    <span className="text-discord-text-muted ml-1">
                      (ID: 624679678573150219)
                    </span>
                  </p>
                  <p>
                    🐛 Report Bugs:{" "}
                    <a
                      href="https://github.com/DraneLixX/SonicDeck/issues"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-discord-primary hover:underline"
                    >
                      GitHub Issues
                    </a>
                  </p>
                  <p>
                    🌐 Source Code:{" "}
                    <a
                      href="https://github.com/DraneLixX/SonicDeck"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-discord-primary hover:underline"
                    >
                      github.com/DraneLixX/SonicDeck
                    </a>
                  </p>
                </div>
              </div>

              {/* Artist Wanted */}
              <div className="pt-3 border-t border-discord-darker bg-discord-primary/10 -mx-6 -mb-6 px-6 py-4 rounded-b-lg">
                <h4 className="text-sm font-semibold text-discord-primary mb-2">
                  🎨 Artist Wanted!
                </h4>
                <p className="text-xs text-discord-text-muted">
                  We're looking for an artist to create visual assets (icons, UI
                  design, branding, etc.) for SonicDeck. This is an open-source
                  community project - unpaid, but with credit!
                </p>
                <p className="text-xs text-discord-text-muted mt-2">
                  Interested? Contact via email or Discord above.
                </p>
              </div>
            </div>
          </div>

          {/* Save Button */}
          <div className="flex gap-3">
            <button
              onClick={handleSaveSettings}
              disabled={isSaving}
              className="flex-1 px-6 py-3 bg-discord-success hover:bg-green-600 
                       disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg 
                       text-white font-semibold transition-colors"
            >
              {isSaving ? "Saving..." : "Save Settings"}
            </button>
            <button
              onClick={handleResetSettings}
              disabled={isSaving}
              className="px-6 py-3 bg-discord-primary hover:bg-discord-primary-hover 
                       disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg 
                       text-white font-semibold transition-colors"
            >
              Reset
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
