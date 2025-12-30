import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  AudioDevice,
  RestoreResult,
  SavedDefaults,
  VbCableStatus,
} from "../../types";
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
  const [isUninstalling, setIsUninstalling] = useState(false);
  const [installStep, setInstallStep] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  // Microphone routing state
  const [microphones, setMicrophones] = useState<[string, string][]>([]);
  const [selectedMicrophone, setSelectedMicrophone] = useState<string>("");
  const [isRoutingActive, setIsRoutingActive] = useState(false);
  const [isRoutingLoading, setIsRoutingLoading] = useState(false);

  const { settings, saveSettings } = useSettings();
  const { refreshDevices } = useAudio();

  // Load microphones and routing status when VB-Cable is installed
  const loadMicrophoneData = useCallback(async () => {
    try {
      // Load available microphones
      const mics = await invoke<[string, string][]>("list_microphones");
      setMicrophones(mics);

      // Check current routing status
      const routingStatus = await invoke<string | null>(
        "get_microphone_routing_status"
      );
      if (routingStatus) {
        setIsRoutingActive(true);
        setSelectedMicrophone(routingStatus);
      } else {
        setIsRoutingActive(false);
        // Restore from settings if available AND microphone still exists
        if (settings?.microphone_routing_device_id) {
          const micExists = mics.some(
            ([id]) => id === settings.microphone_routing_device_id
          );
          if (micExists) {
            setSelectedMicrophone(settings.microphone_routing_device_id);
          }
        }
      }
    } catch (e) {
      console.error("Failed to load microphone data:", e);
    }
  }, [settings?.microphone_routing_device_id]);

  useEffect(() => {
    checkStatus();
  }, []);

  // Load microphone data when VB-Cable is installed
  useEffect(() => {
    if (status?.status === "installed") {
      loadMicrophoneData();
    }
  }, [status, loadMicrophoneData]);

  const checkStatus = async () => {
    try {
      const result = await invoke<VbCableStatus>("check_vb_cable_status");
      setStatus(result);
      setError(null);
    } catch (e) {
      setError(`Status check failed: ${e}`);
    }
  };

  // Microphone routing handlers
  const handleEnableRouting = async () => {
    if (!selectedMicrophone) return;

    setIsRoutingLoading(true);
    setError(null);

    try {
      await invoke("enable_microphone_routing", {
        microphoneId: selectedMicrophone,
      });
      setIsRoutingActive(true);

      // Save to settings
      if (settings) {
        await saveSettings({
          ...settings,
          microphone_routing_device_id: selectedMicrophone,
          microphone_routing_enabled: true,
        });
      }
    } catch (e) {
      setError(`Microphone routing failed: ${e}`);
    } finally {
      setIsRoutingLoading(false);
    }
  };

  const handleDisableRouting = async () => {
    setIsRoutingLoading(true);
    setError(null);

    try {
      await invoke("disable_microphone_routing");
      setIsRoutingActive(false);

      // Save to settings
      if (settings) {
        await saveSettings({
          ...settings,
          microphone_routing_enabled: false,
        });
      }
    } catch (e) {
      setError(`Failed to disable microphone routing: ${e}`);
    } finally {
      setIsRoutingLoading(false);
    }
  };

  const handleInstall = async () => {
    setIsInstalling(true);
    setError(null);

    try {
      // Step 1: Save ALL current Windows default devices before install
      // VB-Cable changes 4 defaults: render/capture × console/communications
      setInstallStep("Saving default devices...");
      const savedDefaults = await invoke<SavedDefaults>(
        "save_all_default_devices"
      );
      console.log("Saved all default devices:", savedDefaults);

      // Step 2: Run installation
      setInstallStep("Installing VB-Cable...");
      await invoke("start_vb_cable_install");

      // Step 3: Wait for VB-Cable device with smart retry
      setInstallStep("Waiting for VB-Cable device...");
      const detectedDevice = await invoke<string | null>(
        "wait_for_vb_cable_device"
      );

      if (!detectedDevice) {
        console.warn(
          "VB-Cable not detected after retries, continuing anyway..."
        );
      }

      // Step 4: Restore ALL Windows default devices
      setInstallStep("Restoring default devices...");
      const restoreResult = await invoke<RestoreResult>(
        "restore_all_default_devices",
        { saved: savedDefaults }
      );
      console.log(
        `Restored ${restoreResult.restored_count} devices, ${restoreResult.failed_count} failed`
      );

      // Show warning if some devices failed to restore
      if (restoreResult.failed_count > 0) {
        const failedDevices = restoreResult.failures
          .map((f) => f.device_role)
          .join(", ");
        setError(
          `Some audio defaults could not be restored: ${failedDevices}. ` +
            `You may need to set them manually in Windows Sound Settings.`
        );
      }

      // Step 5: Refresh device list
      setInstallStep("Refreshing device list...");
      await refreshDevices();
      onDeviceChange?.();

      // Step 6: Check if VB-Cable is now installed
      const newStatus = await invoke<VbCableStatus>("check_vb_cable_status");
      setStatus(newStatus);

      // Step 7: Auto-select VB-Cable as broadcast device
      if (newStatus.status === "installed") {
        setInstallStep("Configuring VB-Cable as broadcast device...");

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
      setInstallStep("Cleaning up...");
      await invoke("cleanup_vb_cable_install");

      setInstallStep("");
    } catch (e) {
      setError(`Installation failed: ${e}`);
      setInstallStep("");
    } finally {
      setIsInstalling(false);
    }
  };

  const handleOpenWebsite = async () => {
    try {
      await invoke("open_vb_audio_website");
    } catch (e) {
      setError(`Could not open website: ${e}`);
    }
  };

  const handleUninstall = async () => {
    setIsUninstalling(true);
    setError(null);

    try {
      // Step 1: Disable microphone routing if active
      if (isRoutingActive) {
        setInstallStep("Stopping microphone routing...");
        await invoke("disable_microphone_routing");
        setIsRoutingActive(false);
      }

      // Step 2: Clear broadcast device setting
      if (settings?.broadcast_device_id) {
        setInstallStep("Clearing VB-Cable settings...");
        await saveSettings({
          ...settings,
          broadcast_device_id: null,
          microphone_routing_enabled: false,
          microphone_routing_device_id: null,
        });
      }

      // Step 3: Run uninstaller
      setInstallStep("Uninstalling VB-Cable...");
      await invoke("start_vb_cable_uninstall");

      // Step 4: Refresh device list
      setInstallStep("Refreshing devices...");
      await refreshDevices();
      onDeviceChange?.();

      // Step 5: Check status
      const newStatus = await invoke<VbCableStatus>("check_vb_cable_status");
      setStatus(newStatus);

      setInstallStep("");
    } catch (e) {
      setError(`Uninstallation failed: ${e}`);
      setInstallStep("");
    } finally {
      setIsUninstalling(false);
    }
  };

  const handleOpenSoundSettings = async () => {
    // Open Windows classic sound control panel (mmsys.cpl)
    // ms-settings:sound doesn't work with window.open, need shell command
    try {
      await invoke("open_sound_settings");
    } catch (e) {
      console.error("Failed to open sound settings:", e);
    }
  };

  return (
    <div className="bg-discord-dark rounded-lg p-6">
      <h3 className="text-lg font-semibold text-discord-text mb-3">
        VB-Cable Integration
      </h3>

      {status?.status === "installed" ? (
        <div className="space-y-4">
          <div className="space-y-3">
            <div className="flex items-center gap-2 text-discord-success">
              <span className="text-lg">✓</span>
              <span>VB-Cable is installed</span>
            </div>
            <p className="text-sm text-discord-text-muted">
              Device: {status.info.output_device}
            </p>
          </div>

          {/* Microphone Routing Section */}
          <div className="pt-4 border-t border-discord-darker">
            <h4 className="text-sm font-medium text-discord-text mb-2">
              Microphone Routing (for Discord)
            </h4>
            <p className="text-xs text-discord-text-muted mb-3">
              Enable this so your friends can hear you AND the sounds.
            </p>

            <div className="flex items-center gap-3">
              <select
                value={selectedMicrophone}
                onChange={(e) => setSelectedMicrophone(e.target.value)}
                disabled={isRoutingActive || isRoutingLoading}
                className="flex-1 bg-discord-darker text-discord-text rounded px-3 py-2
                         border border-discord-darker hover:border-discord-text-muted
                         focus:border-discord-primary focus:outline-none
                         disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <option value="">Select microphone...</option>
                {microphones.map(([id, name]) => (
                  <option key={id} value={id}>
                    {name}
                  </option>
                ))}
              </select>

              <button
                onClick={
                  isRoutingActive ? handleDisableRouting : handleEnableRouting
                }
                disabled={
                  (!selectedMicrophone && !isRoutingActive) || isRoutingLoading
                }
                className={`px-4 py-2 rounded font-medium transition-colors
                          disabled:opacity-50 disabled:cursor-not-allowed ${
                            isRoutingActive
                              ? "bg-discord-danger hover:bg-discord-danger/80 text-white"
                              : "bg-discord-primary hover:bg-discord-primary-hover text-white"
                          }`}
              >
                {isRoutingLoading
                  ? "..."
                  : isRoutingActive
                    ? "Disable"
                    : "Enable"}
              </button>
            </div>

            {isRoutingActive && (
              <p className="mt-2 text-xs text-discord-success">
                Microphone is being routed to CABLE Input
              </p>
            )}
          </div>

          {/* Tip: CABLE In 16 Ch Device */}
          <div className="pt-4 border-t border-discord-darker">
            <h4 className="text-sm font-medium text-discord-text mb-2">
              Tip: Hide "CABLE In 16 Ch"
            </h4>
            <p className="text-xs text-discord-text-muted mb-2">
              VB-Cable installs an additional device that is not needed. You can
              disable it in Windows Sound settings:
            </p>
            <ol className="text-xs text-discord-text-muted mb-3 list-decimal list-inside space-y-1">
              <li>
                Click{" "}
                <button
                  onClick={handleOpenSoundSettings}
                  className="text-discord-primary hover:underline"
                >
                  Open Sound Settings
                </button>
              </li>
              <li>Find "CABLE In 16 Ch" in the Playback tab</li>
              <li>Right-click → Disable</li>
              <li>Click OK to save changes</li>
            </ol>
          </div>

          {/* Uninstall Section */}
          <div className="pt-4 border-t border-discord-darker">
            <h4 className="text-sm font-medium text-discord-text mb-2">
              Uninstall VB-Cable
            </h4>
            <p className="text-xs text-discord-text-muted mb-3">
              Removes VB-Cable and all related settings.
            </p>
            <button
              onClick={handleUninstall}
              disabled={isUninstalling || isRoutingActive}
              className="px-4 py-2 bg-discord-danger hover:bg-discord-danger/80
                       disabled:bg-gray-600 disabled:cursor-not-allowed rounded
                       text-white font-medium text-sm transition-colors"
            >
              {isUninstalling ? "Uninstalling..." : "Uninstall VB-Cable"}
            </button>
            {isRoutingActive && (
              <p className="mt-2 text-xs text-discord-warning">
                Please disable Microphone Routing first.
              </p>
            )}
            {isUninstalling && installStep && (
              <p className="mt-2 text-xs text-discord-text-muted">
                {installStep}
              </p>
            )}
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          <p className="text-sm text-discord-text-muted">
            VB-Cable is required for dual-output routing to Discord.
          </p>

          <div className="flex gap-3">
            <button
              onClick={handleInstall}
              disabled={isInstalling}
              className="px-4 py-2 bg-discord-primary hover:bg-discord-primary-hover
                       disabled:bg-gray-600 disabled:cursor-not-allowed rounded
                       text-white font-medium transition-colors"
            >
              {isInstalling ? "Installing..." : "Install VB-Cable"}
            </button>

            <button
              onClick={handleOpenWebsite}
              className="px-4 py-2 bg-discord-darker hover:bg-discord-dark
                       rounded text-discord-text-muted transition-colors"
            >
              Manual Download
            </button>
          </div>

          {isInstalling && (
            <p className="text-sm text-discord-text-muted">
              {installStep ||
                "Downloading and installing... Windows will ask for driver approval."}
            </p>
          )}
        </div>
      )}

      {error && (
        <div className="mt-4 p-3 bg-discord-danger/20 border border-discord-danger rounded">
          <p className="text-sm text-discord-danger">{error}</p>
        </div>
      )}

      {/* Donationware notice - always visible per VB-Audio licensing requirements */}
      <div className="mt-4 p-3 bg-discord-darker rounded text-sm text-discord-text-muted">
        <p>
          VB-Cable is donationware by{" "}
          <button
            onClick={handleOpenWebsite}
            className="text-discord-primary hover:underline"
          >
            vb-audio.com
          </button>
        </p>
        <p className="mt-1">
          If you find it useful, please consider supporting the developers!
        </p>
      </div>
    </div>
  );
}
