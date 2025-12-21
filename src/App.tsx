import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Dashboard from "./components/dashboard/Dashboard";
import Settings from "./components/settings/Settings";
import ErrorBoundary from "./components/common/ErrorBoundary";
import { AudioProvider } from "./contexts/AudioContext";
import { SettingsProvider, useSettings } from "./contexts/SettingsContext";
import { SoundLibraryProvider } from "./contexts/SoundLibraryContext";

type View = "dashboard" | "settings";

function AppContent() {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const { settings } = useSettings();

  // Dashboard-specific state that persists across view changes
  const [dashboardDevice1, setDashboardDevice1] = useState<string>("");
  const [dashboardDevice2, setDashboardDevice2] = useState<string>("");

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
          {currentView === "dashboard" && (
            <Dashboard
              device1={dashboardDevice1}
              device2={dashboardDevice2}
              setDevice1={setDashboardDevice1}
              setDevice2={setDashboardDevice2}
            />
          )}
          {currentView === "settings" && <Settings />}
        </ErrorBoundary>
      </div>
    </div>
  );
}

function App() {
  return (
    <AudioProvider>
      <SettingsProvider>
        <SoundLibraryProvider>
          <AppContent />
        </SoundLibraryProvider>
      </SettingsProvider>
    </AudioProvider>
  );
}

export default App;
