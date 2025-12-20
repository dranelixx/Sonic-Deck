import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Dashboard from "./components/Dashboard";
import Settings from "./components/Settings";
import ErrorBoundary from "./components/ErrorBoundary";
import { AudioDevice, AppSettings, SoundLibrary } from "./types";

type View = "dashboard" | "settings";

function App() {
  const [currentView, setCurrentView] = useState<View>("dashboard");

  // Shared state - loaded once and persisted across view changes
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [soundLibrary, setSoundLibrary] = useState<SoundLibrary>({
    categories: [],
    sounds: [],
  });
  const [isInitialized, setIsInitialized] = useState<boolean>(false);

  // Dashboard-specific state that persists across view changes
  const [dashboardDevice1, setDashboardDevice1] = useState<string>("");
  const [dashboardDevice2, setDashboardDevice2] = useState<string>("");

  // Load devices, settings, and sounds once on app mount
  useEffect(() => {
    const initialize = async () => {
      try {
        const [deviceList, loadedSettings, loadedSounds] = await Promise.all([
          invoke<AudioDevice[]>("list_audio_devices"),
          invoke<AppSettings>("load_settings"),
          invoke<SoundLibrary>("load_sounds"),
        ]);
        setDevices(deviceList);
        setSettings(loadedSettings);
        setSoundLibrary(loadedSounds);
        setIsInitialized(true);
      } catch (error) {
        console.error("Failed to initialize app:", error);
        setIsInitialized(true); // Still mark as initialized to show UI
      }
    };
    initialize();
  }, []);

  // Handle window close event based on minimize_to_tray setting
  useEffect(() => {
    if (!settings) return;

    const window = getCurrentWindow();
    console.log(
      "Setting up close handler, minimize_to_tray:",
      settings.minimize_to_tray
    );

    const unlisten = window.onCloseRequested(async (event) => {
      console.log(
        "Close event received! minimize_to_tray:",
        settings.minimize_to_tray
      );

      if (settings.minimize_to_tray) {
        event.preventDefault();
        try {
          await window.hide();
          console.log("Window minimized to tray via X button");
        } catch (err) {
          console.error("Failed to hide window:", err);
        }
      } else {
        console.log("Window closing normally (minimize_to_tray is disabled)");
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [settings?.minimize_to_tray]);

  const refreshDevices = async () => {
    try {
      const deviceList = await invoke<AudioDevice[]>("list_audio_devices");
      setDevices(deviceList);
    } catch (error) {
      console.error("Failed to refresh devices:", error);
      throw error;
    }
  };

  const refreshSounds = async () => {
    try {
      const loadedSounds = await invoke<SoundLibrary>("load_sounds");
      setSoundLibrary(loadedSounds);
    } catch (error) {
      console.error("Failed to refresh sounds:", error);
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
        <div
          className="w-12 h-12 bg-discord-primary rounded-full flex items-center justify-center text-2xl cursor-pointer hover:bg-discord-primary-hover transition-colors"
          onClick={() => setCurrentView("dashboard")}
          title="Dashboard"
        >
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
      <div className="flex-1 relative">
        <ErrorBoundary>
          {isInitialized && currentView === "dashboard" && (
            <Dashboard
              devices={devices}
              settings={settings}
              soundLibrary={soundLibrary}
              refreshDevices={refreshDevices}
              refreshSounds={refreshSounds}
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
        </ErrorBoundary>
      </div>
    </div>
  );
}

export default App;
