import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
}

interface AppSettings {
  monitor_device_id: string | null;
  broadcast_device_id: string | null;
  default_volume: number;
  last_file_path: string | null;
}

interface DashboardProps {
  devices: AudioDevice[];
  settings: AppSettings | null;
  refreshDevices: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;
  device1: string;
  device2: string;
  setDevice1: (id: string) => void;
  setDevice2: (id: string) => void;
}

export default function Dashboard({
  devices,
  settings,
  refreshDevices,
  saveSettings,
  device1,
  device2,
  setDevice1,
  setDevice2
}: DashboardProps) {
  const [volume, setVolume] = useState<number>(0.5);
  const [testFile, setTestFile] = useState<string>("");
  const [status, setStatus] = useState<string>("Ready");
  const [isRefreshing, setIsRefreshing] = useState<boolean>(false);
  const [isPlaying, setIsPlaying] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [hasLoadedSettings, setHasLoadedSettings] = useState<boolean>(false);

  // Apply settings from props only once on initial load
  useEffect(() => {
    if (settings && !hasLoadedSettings) {
      if (settings.monitor_device_id) {
        setDevice1(settings.monitor_device_id);
      }
      if (settings.broadcast_device_id) {
        setDevice2(settings.broadcast_device_id);
      }
      if (settings.default_volume !== undefined) {
        setVolume(settings.default_volume);
      }
      if (settings.last_file_path) {
        setTestFile(settings.last_file_path);
      }
      setHasLoadedSettings(true);
    }
  }, [settings, hasLoadedSettings, setDevice1, setDevice2]);

  // Listen for audio decode complete events
  useEffect(() => {
    const unlisten = listen<string>("audio-decode-complete", (event) => {
      setIsLoading(false);
      setIsPlaying(true);
      setStatus(`Playing on both devices (ID: ${event.payload})`);
    });

    const unlistenError = listen<string>("audio-decode-error", (event) => {
      setIsLoading(false);
      setIsPlaying(false);
      setStatus(`Decode Error: ${event.payload}`);
    });

    return () => {
      unlisten.then(fn => fn());
      unlistenError.then(fn => fn());
    };
  }, []);


  const handleRefreshDevices = async () => {
    try {
      setIsRefreshing(true);
      await refreshDevices();
      setStatus("Devices refreshed successfully");
      setTimeout(() => setStatus("Ready"), 2000);
    } catch (error) {
      setStatus(`Error: ${error}`);
      console.error("Failed to refresh devices:", error);
    } finally {
      setIsRefreshing(false);
    }
  };

  // Check if selected devices are still available after devices list changes
  // Only run this check when user explicitly refreshes devices, not on every mount
  useEffect(() => {
    if (devices.length > 0 && device1 && device2) {
      const device1Available = devices.some(d => d.id === device1);
      const device2Available = devices.some(d => d.id === device2);
      
      // Only clear devices that are actually invalid
      if (!device1Available) {
        setDevice1("");
        setStatus("Warning: Monitor output device disconnected");
      }
      if (!device2Available) {
        setDevice2("");
        setStatus("Warning: Broadcast output device disconnected");
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [devices.length]); // Only trigger when device count changes, not on every devices array change

  const testDualOutput = async () => {
    if (!device1 || !device2) {
      setStatus("Please select both output devices (or configure them in Settings)");
      return;
    }

    if (!testFile) {
      setStatus("Please provide a test audio file path");
      return;
    }

    try {
      setIsLoading(true);
      setStatus("Loading audio file...");

      console.log("Playing with devices:", { device1, device2, volume, testFile });

      await invoke<string>("play_dual_output", {
        filePath: testFile,
        deviceId1: device1,
        deviceId2: device2,
        volume: volume,
      });
      
      // Status will be updated by the audio-decode-complete event
      // This happens when actual decoding is done and playback starts
      
      // Save last used file path
      if (settings) {
        await saveSettings({ ...settings, last_file_path: testFile });
      }
    } catch (error) {
      setStatus(`Playback Error: ${error}`);
      console.error("Playback failed:", error);
      setIsLoading(false);
      setIsPlaying(false);
    }
  };

  const stopAll = async () => {
    try {
      await invoke("stop_all_audio");
      setIsPlaying(false);
      setIsLoading(false);
      setStatus("All audio stopped");
    } catch (error) {
      setStatus(`Stop Error: ${error}`);
      console.error("Stop failed:", error);
    }
  };

  // Check if devices are configured
  const devicesConfigured = device1 && device2;

  return (
    <div className="w-full h-full bg-discord-darkest flex flex-col">
      {/* Header */}
      <div className="bg-discord-darker px-6 py-4 border-b border-discord-dark">
        <h1 className="text-2xl font-bold text-discord-primary">
          🎵 SonicDeck
        </h1>
        <p className="text-sm text-discord-text-muted mt-1">
          Dual-output audio routing for streamers and content creators
        </p>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-4xl mx-auto space-y-6">
          
          {/* Status Banner */}
          <div className="bg-discord-dark rounded-lg p-4 border border-discord-darker">
            <div className="flex items-center justify-between">
              <span className="text-discord-text-muted">Status:</span>
              <div className="flex items-center gap-2">
                {isLoading && (
                  <div className="animate-spin h-4 w-4 border-2 border-discord-primary border-t-transparent rounded-full"></div>
                )}
                <span className={`font-medium ${isLoading ? 'text-discord-primary' : 'text-discord-text'}`}>
                  {status}
                </span>
              </div>
            </div>
          </div>

          {/* Configuration Warning - Only show if devices are not configured */}
          {!devicesConfigured && (
            <div className="bg-discord-warning/20 border border-discord-warning rounded-lg p-4">
              <h3 className="text-discord-warning font-semibold mb-2">
                ⚠️ Devices Not Configured
              </h3>
              <p className="text-sm text-discord-text-muted">
                Please select your audio devices below or go to Settings to configure default devices.
              </p>
            </div>
          )}

          {/* Quick Device Selection */}
          <div className="bg-discord-dark rounded-lg p-6 space-y-4">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-semibold text-discord-text">
                Audio Output Devices
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

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {/* Device 1 (Monitor) */}
              <div>
                <label className="block text-sm font-medium text-discord-text-muted mb-2">
                  Monitor Output (Your Headphones)
                </label>
                <select
                  value={device1}
                  onChange={(e) => setDevice1(e.target.value)}
                  className="w-full bg-discord-darker border border-discord-dark rounded px-3 py-2 
                           text-discord-text focus:outline-none focus:ring-2 focus:ring-discord-primary"
                >
                  <option value="">Select a device...</option>
                  {devices.map((device) => (
                    <option key={device.id} value={device.id}>
                      {device.name} {device.is_default ? "(Default)" : ""}
                    </option>
                  ))}
                </select>
              </div>

              {/* Device 2 (Broadcast) */}
              <div>
                <label className="block text-sm font-medium text-discord-text-muted mb-2">
                  Broadcast Output (Virtual Cable)
                </label>
                <select
                  value={device2}
                  onChange={(e) => setDevice2(e.target.value)}
                  className="w-full bg-discord-darker border border-discord-dark rounded px-3 py-2 
                           text-discord-text focus:outline-none focus:ring-2 focus:ring-discord-primary"
                >
                  <option value="">Select a device...</option>
                  {devices.map((device) => (
                    <option key={device.id} value={device.id}>
                      {device.name} {device.is_default ? "(Default)" : ""}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            {/* Warning if both devices are the same */}
            {device1 && device2 && device1 === device2 && (
              <div className="bg-discord-warning/20 border border-discord-warning rounded p-3 mt-4">
                <p className="text-sm text-discord-warning">
                  ⚠️ Warning: Both outputs are set to the same device. For dual-output routing, 
                  select different devices.
                </p>
              </div>
            )}
          </div>

          {/* Playback Test Controls */}
          <div className="bg-discord-dark rounded-lg p-6 space-y-4">
            <h2 className="text-xl font-semibold text-discord-text mb-4">
              Test Playback
            </h2>

            {/* File Path Input */}
            <div>
              <label className="block text-sm font-medium text-discord-text-muted mb-2">
                Audio File Path
              </label>
              <input
                type="text"
                value={testFile}
                onChange={(e) => setTestFile(e.target.value)}
                placeholder="e.g., C:/Users/YourName/Music/test.mp3"
                className="w-full bg-discord-darker border border-discord-dark rounded px-3 py-2 
                         text-discord-text focus:outline-none focus:ring-2 focus:ring-discord-primary"
              />
              <p className="text-xs text-discord-text-muted mt-1">
                Enter the full path to an MP3, WAV, or OGG file
              </p>
            </div>

            {/* Volume Control */}
            <div>
              <label className="block text-sm font-medium text-discord-text-muted mb-2">
                Volume: <span className={volume >= 0.75 ? "text-red-500 font-bold" : ""}>{Math.round(volume * 100)}%</span>
                {volume >= 0.75 && (
                  <span className="ml-2 text-xs text-red-400">⚠️ High volume - protect your hearing!</span>
                )}
              </label>
              <div className="relative">
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.01"
                  value={volume}
                  onChange={(e) => setVolume(parseFloat(e.target.value))}
                  className="w-full"
                  style={{
                    accentColor: volume >= 0.75 ? '#ef4444' : '#5865f2'
                  }}
                />
                {volume >= 0.75 && (
                  <div className="absolute -top-1 left-0 w-full h-2 bg-red-500/20 rounded -z-10"></div>
                )}
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex gap-3 pt-4">
              <button
                onClick={testDualOutput}
                disabled={isLoading || isPlaying || !device1 || !device2 || !testFile}
                className="flex-1 px-6 py-3 bg-discord-success hover:bg-green-600 
                         disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg 
                         text-white font-semibold transition-colors flex items-center justify-center gap-2"
              >
                {isLoading && (
                  <div className="animate-spin h-5 w-5 border-2 border-white border-t-transparent rounded-full"></div>
                )}
                {isLoading ? "Loading..." : isPlaying ? "Playing..." : "▶ Play on Both Devices"}
              </button>
              <button
                onClick={stopAll}
                className="px-6 py-3 bg-discord-danger hover:bg-red-600 rounded-lg 
                         text-white font-semibold transition-colors"
              >
                ⏹ Stop All
              </button>
            </div>
          </div>

          {/* Quick Guide */}
          <div className="bg-discord-dark rounded-lg p-6">
            <h3 className="text-lg font-semibold text-discord-text mb-3">
              🚀 Quick Start Guide
            </h3>
            <ul className="space-y-2 text-sm text-discord-text-muted">
              <li>1. Select your Monitor Output (where you hear sounds)</li>
              <li>2. Select your Broadcast Output (where your stream/recording hears sounds)</li>
              <li>3. Enter the path to a test audio file</li>
              <li>4. Click "Play on Both Devices" to test dual-output routing</li>
              <li>5. Go to Settings to save default device configuration</li>
            </ul>
          </div>

        </div>
      </div>
    </div>
  );
}
