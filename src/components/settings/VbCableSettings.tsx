import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AudioDevice, SavedDefaults, VbCableStatus } from "../../types";
import { useSettings } from "../../contexts/SettingsContext";
import { useAudio } from "../../contexts/AudioContext";

interface VbCableSettingsProps {
  onDeviceChange?: () => void;
}

export default function VbCableSettings({
  onDeviceChange,
}: VbCableSettingsProps) {
  const [status, setStatus] = useState<VbCableStatus | null>(null);
  const [isInstalling, setIsInstalling] = useState(false);
  const [installStep, setInstallStep] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  const { settings, saveSettings } = useSettings();
  const { refreshDevices } = useAudio();

  useEffect(() => {
    checkStatus();
  }, []);

  const checkStatus = async () => {
    try {
      const result = await invoke<VbCableStatus>("check_vb_cable_status");
      setStatus(result);
      setError(null);
    } catch (e) {
      setError(`Status-Check fehlgeschlagen: ${e}`);
    }
  };

  const handleInstall = async () => {
    setIsInstalling(true);
    setError(null);

    try {
      // Step 1: Save ALL current Windows default devices before install
      // VB-Cable changes 4 defaults: render/capture × console/communications
      setInstallStep("Speichere alle Standard-Geräte...");
      const savedDefaults = await invoke<SavedDefaults>(
        "save_all_default_devices"
      );
      console.log("Saved all default devices:", savedDefaults);

      // Step 2: Run installation
      setInstallStep("Installiere VB-Cable...");
      await invoke("start_vb_cable_install");

      // Step 3: Wait for Windows to register the new device
      setInstallStep("Warte auf Geräte-Registrierung...");
      await new Promise((resolve) => setTimeout(resolve, 3000));

      // Step 4: Restore ALL Windows default devices
      setInstallStep("Stelle alle Standard-Geräte wieder her...");
      try {
        await invoke("restore_all_default_devices", { saved: savedDefaults });
        console.log("Restored all default devices");
      } catch (e) {
        console.warn("Could not restore all default devices:", e);
      }

      // Step 5: Refresh device list
      setInstallStep("Aktualisiere Geräteliste...");
      await refreshDevices();
      onDeviceChange?.();

      // Step 6: Check if VB-Cable is now installed
      const newStatus = await invoke<VbCableStatus>("check_vb_cable_status");
      setStatus(newStatus);

      // Step 7: Auto-select VB-Cable as broadcast device
      if (newStatus.status === "installed") {
        setInstallStep("Konfiguriere VB-Cable als Broadcast-Gerät...");

        // Get fresh device list
        const devices = await invoke<AudioDevice[]>("list_audio_devices");

        // Find VB-Cable device (CABLE Input)
        const vbCableDevice = devices.find((d) =>
          d.name.toLowerCase().includes("cable input")
        );

        if (vbCableDevice && settings) {
          // Set VB-Cable as broadcast device
          const updatedSettings = {
            ...settings,
            broadcast_device_id: vbCableDevice.id,
          };
          await saveSettings(updatedSettings);
          console.log("Set VB-Cable as broadcast device:", vbCableDevice.name);
        }
      }

      // Step 8: Cleanup
      setInstallStep("Räume auf...");
      await invoke("cleanup_vb_cable_install");

      setInstallStep("");
    } catch (e) {
      setError(`Installation fehlgeschlagen: ${e}`);
      setInstallStep("");
    } finally {
      setIsInstalling(false);
    }
  };

  const handleOpenWebsite = async () => {
    try {
      await invoke("open_vb_audio_website");
    } catch (e) {
      setError(`Konnte Website nicht öffnen: ${e}`);
    }
  };

  return (
    <div className="bg-discord-dark rounded-lg p-6">
      <h3 className="text-lg font-semibold text-discord-text mb-3">
        VB-Cable Integration
      </h3>

      {status?.status === "installed" ? (
        <div className="space-y-3">
          <div className="flex items-center gap-2 text-discord-success">
            <span className="text-lg">✓</span>
            <span>VB-Cable ist installiert</span>
          </div>
          <p className="text-sm text-discord-text-muted">
            Gerät: {status.info.output_device}
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          <p className="text-sm text-discord-text-muted">
            VB-Cable wird für Dual-Output Routing zu Discord benötigt.
          </p>

          <div className="flex gap-3">
            <button
              onClick={handleInstall}
              disabled={isInstalling}
              className="px-4 py-2 bg-discord-primary hover:bg-discord-primary-hover
                       disabled:bg-gray-600 disabled:cursor-not-allowed rounded
                       text-white font-medium transition-colors"
            >
              {isInstalling ? "Installiere..." : "VB-Cable installieren"}
            </button>

            <button
              onClick={handleOpenWebsite}
              className="px-4 py-2 bg-discord-darker hover:bg-discord-dark
                       rounded text-discord-text-muted transition-colors"
            >
              Manueller Download
            </button>
          </div>

          {isInstalling && (
            <p className="text-sm text-discord-text-muted">
              {installStep ||
                "Download und Installation... Windows fragt nach Treiber-Genehmigung."}
            </p>
          )}
        </div>
      )}

      {error && (
        <div className="mt-4 p-3 bg-discord-danger/20 border border-discord-danger rounded">
          <p className="text-sm text-discord-danger">{error}</p>
        </div>
      )}
    </div>
  );
}
