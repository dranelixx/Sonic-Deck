import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../../types";
import { useAudio } from "../../contexts/AudioContext";
import { useSettings as useSettingsContext } from "../../contexts/SettingsContext";
import AudioDeviceSettings from "./AudioDeviceSettings";
import PlaybackSettings from "./PlaybackSettings";
import SystemTraySettings from "./SystemTraySettings";
import VbCableSettings from "./VbCableSettings";
import SettingsAbout from "./SettingsAbout";

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
    microphone_routing_device_id: null,
    microphone_routing_enabled: false,
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

  // Sync from context
  useEffect(() => {
    if (contextSettings) {
      setSettings(contextSettings);
    }
  }, [contextSettings]);

  const handleRefreshDevices = async () => {
    setIsRefreshing(true);
    try {
      await refreshDevices();
      setStatus("Devices refreshed successfully!");
    } catch (error) {
      console.error("Failed to refresh devices:", error);
      setStatus(`Error: ${error}`);
    } finally {
      setIsRefreshing(false);
    }
  };

  const handleSaveSettings = async () => {
    setIsSaving(true);
    try {
      await saveSettingsToContext(settings);
      setStatus("Settings saved successfully!");
    } catch (error) {
      console.error("Failed to save settings:", error);
      setStatus(`Error: ${error}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleResetSettings = async () => {
    try {
      await invoke("reset_settings");
      await reloadSettings();
      setStatus("Settings reset to defaults!");
    } catch (error) {
      console.error("Failed to reset settings:", error);
      setStatus(`Error: ${error}`);
    }
  };

  const updateSetting = (
    key: keyof AppSettings,
    value: string | number | boolean | null
  ) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
  };

  const isDeviceAvailable = (deviceId: string): boolean => {
    return devices.some((d) => d.id === deviceId);
  };

  const handleToggleAutostart = async () => {
    const newValue = !settings.autostart_enabled;
    updateSetting("autostart_enabled", newValue);
    try {
      if (newValue) {
        await invoke("enable_autostart");
      } else {
        await invoke("disable_autostart");
      }
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
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

          {/* VB-Cable Integration */}
          <VbCableSettings onDeviceChange={handleRefreshDevices} />

          {/* Audio Device Configuration */}
          <AudioDeviceSettings
            settings={settings}
            devices={devices}
            isRefreshing={isRefreshing}
            onRefresh={handleRefreshDevices}
            onUpdateSetting={updateSetting}
            isDeviceAvailable={isDeviceAvailable}
          />

          {/* Playback Preferences */}
          <PlaybackSettings
            settings={settings}
            onUpdateSetting={updateSetting}
          />

          {/* System Tray & Startup */}
          <SystemTraySettings
            settings={settings}
            onUpdateSetting={updateSetting}
            onToggleAutostart={handleToggleAutostart}
          />

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

          {/* About SonicDeck */}
          <SettingsAbout />

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
