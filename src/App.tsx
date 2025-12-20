import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Dashboard from "./components/Dashboard";
import Settings from "./components/Settings";

type View = "dashboard" | "settings";

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

function App() {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  
  // Shared state - loaded once and persisted across view changes
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [isInitialized, setIsInitialized] = useState<boolean>(false);
  
  // Dashboard-specific state that persists across view changes
  const [dashboardDevice1, setDashboardDevice1] = useState<string>("");
  const [dashboardDevice2, setDashboardDevice2] = useState<string>("");

  // Load devices and settings once on app mount
  useEffect(() => {
    const initialize = async () => {
      try {
        const [deviceList, loadedSettings] = await Promise.all([
          invoke<AudioDevice[]>("list_audio_devices"),
          invoke<AppSettings>("load_settings")
        ]);
        setDevices(deviceList);
        setSettings(loadedSettings);
        setIsInitialized(true);
      } catch (error) {
        console.error("Failed to initialize app:", error);
        setIsInitialized(true); // Still mark as initialized to show UI
      }
    };
    initialize();
  }, []);

  const refreshDevices = async () => {
    try {
      const deviceList = await invoke<AudioDevice[]>("list_audio_devices");
      setDevices(deviceList);
    } catch (error) {
      console.error("Failed to refresh devices:", error);
      throw error;
    }
  };

  const reloadSettings = async () => {
    try {
      const loadedSettings = await invoke<AppSettings>("load_settings");
      setSettings(loadedSettings);
    } catch (error) {
      console.error("Failed to reload settings:", error);
      throw error;
    }
  };

  const saveSettings = async (newSettings: AppSettings) => {
    try {
      await invoke("save_settings", { settings: newSettings });
      setSettings(newSettings);
    } catch (error) {
      console.error("Failed to save settings:", error);
      throw error;
    }
  };

  return (
    <div className="w-full h-full bg-discord-darkest flex">
      {/* Sidebar Navigation */}
      <div className="w-20 bg-discord-darker border-r border-discord-dark flex flex-col items-center py-6 space-y-4">
        {/* Logo/Home */}
        <div className="w-12 h-12 bg-discord-primary rounded-full flex items-center justify-center text-2xl cursor-pointer hover:bg-discord-primary-hover transition-colors"
             onClick={() => setCurrentView("dashboard")}
             title="Dashboard">
          🎵
        </div>

        <div className="w-full h-px bg-discord-dark"></div>

        {/* Navigation Icons */}
        <button
          onClick={() => setCurrentView("dashboard")}
          className={`w-12 h-12 rounded-full flex items-center justify-center text-2xl transition-colors ${
            currentView === "dashboard"
              ? "bg-discord-primary text-white"
              : "bg-discord-dark text-discord-text-muted hover:bg-discord-dark hover:text-discord-text"
          }`}
          title="Dashboard"
        >
          🏠
        </button>

        <button
          onClick={() => setCurrentView("settings")}
          className={`w-12 h-12 rounded-full flex items-center justify-center text-2xl transition-colors ${
            currentView === "settings"
              ? "bg-discord-primary text-white"
              : "bg-discord-dark text-discord-text-muted hover:bg-discord-dark hover:text-discord-text"
          }`}
          title="Settings"
        >
          ⚙️
        </button>
      </div>

      {/* Main Content Area */}
      <div className="flex-1">
        {isInitialized && currentView === "dashboard" && (
          <Dashboard
            devices={devices}
            settings={settings}
            refreshDevices={refreshDevices}
            saveSettings={saveSettings}
            device1={dashboardDevice1}
            device2={dashboardDevice2}
            setDevice1={setDashboardDevice1}
            setDevice2={setDashboardDevice2}
          />
        )}
        {isInitialized && currentView === "settings" && (
          <Settings 
            devices={devices}
            settings={settings}
            refreshDevices={refreshDevices}
            reloadSettings={reloadSettings}
            saveSettings={saveSettings}
          />
        )}
      </div>
    </div>
  );
}

export default App;
