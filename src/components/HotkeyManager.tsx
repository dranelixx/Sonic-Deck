import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sound } from "../types";
import { formatHotkeyForDisplay } from "../utils/hotkeyDisplay";

interface HotkeyMapping {
  mappings: Record<string, string>; // hotkey -> sound_id
}

interface HotkeyManagerProps {
  sound: Sound;
  hotkeyMappings: HotkeyMapping;
  onClose: () => void;
  onHotkeyAssigned: () => void;
}

export default function HotkeyManager({
  sound,
  hotkeyMappings,
  onClose,
  onHotkeyAssigned,
}: HotkeyManagerProps) {
  const [isCapturing, setIsCapturing] = useState(false);
  const [capturedKeys, setCapturedKeys] = useState<string[]>([]);
  const [error, setError] = useState<string>("");
  const [status, setStatus] = useState<string>("");

  // Get existing hotkey for this sound
  const existingHotkey = Object.entries(hotkeyMappings.mappings).find(
    ([_, soundId]) => soundId === sound.id
  )?.[0];

  // Normalize modifier key names
  const normalizeKey = (key: string): string => {
    const keyMap: Record<string, string> = {
      Control: "Ctrl",
      Meta: "Super",
      " ": "Space",
    };
    return keyMap[key] || key;
  };

  // Handle key down during capture
  // eslint-disable-next-line no-undef
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!isCapturing) return;

    e.preventDefault();
    e.stopPropagation();

    const keys: string[] = [];

    // Add modifiers
    if (e.ctrlKey) keys.push("Ctrl");
    if (e.shiftKey) keys.push("Shift");
    if (e.altKey) keys.push("Alt");
    if (e.metaKey) keys.push("Super");

    // Add actual key if it's not a modifier
    if (!["Control", "Shift", "Alt", "Meta"].includes(e.key)) {
      let keyName = e.key;

      // Handle NumPad keys correctly using e.code instead of e.key
      if (e.code.startsWith("Numpad")) {
        if (e.code === "Numpad0") keyName = "NumPad0";
        else if (e.code === "Numpad1") keyName = "NumPad1";
        else if (e.code === "Numpad2") keyName = "NumPad2";
        else if (e.code === "Numpad3") keyName = "NumPad3";
        else if (e.code === "Numpad4") keyName = "NumPad4";
        else if (e.code === "Numpad5") keyName = "NumPad5";
        else if (e.code === "Numpad6") keyName = "NumPad6";
        else if (e.code === "Numpad7") keyName = "NumPad7";
        else if (e.code === "Numpad8") keyName = "NumPad8";
        else if (e.code === "Numpad9") keyName = "NumPad9";
        else if (e.code === "NumpadDecimal") keyName = "NumPadDecimal";
        else if (e.code === "NumpadEnter") keyName = "NumPadEnter";
        else if (e.code === "NumpadAdd") keyName = "NumPadAdd";
        else if (e.code === "NumpadSubtract") keyName = "NumPadSubtract";
        else if (e.code === "NumpadMultiply") keyName = "NumPadMultiply";
        else if (e.code === "NumpadDivide") keyName = "NumPadDivide";
      } else {
        keyName = normalizeKey(e.key);
      }

      keys.push(keyName);
      console.log(
        "Captured key:",
        keyName,
        "Code:",
        e.code,
        "Key:",
        e.key,
        "Modifiers:",
        keys.slice(0, -1)
      );
    }

    setCapturedKeys(keys);
  };

  // Setup keyboard listener
  useEffect(() => {
    if (isCapturing) {
      window.addEventListener("keydown", handleKeyDown);
      return () => {
        window.removeEventListener("keydown", handleKeyDown);
      };
    }
  }, [isCapturing]);

  // Start capturing
  const startCapture = () => {
    setIsCapturing(true);
    setCapturedKeys([]);
    setError("");
    setStatus("Press your desired key combination...");
  };

  // Cancel capture
  const cancelCapture = () => {
    setIsCapturing(false);
    setCapturedKeys([]);
    setStatus("");
  };

  // Assign the captured hotkey
  const assignHotkey = async () => {
    if (capturedKeys.length === 0) {
      setError("No keys captured");
      return;
    }

    // Format hotkey string (e.g., "Ctrl+Shift+F1")
    const hotkeyString = capturedKeys.join("+");

    // Validate hotkey - prevent single keys that block normal input
    if (capturedKeys.length === 1) {
      const singleKey = capturedKeys[0];
      // Block problematic single keys
      if (
        singleKey.match(/^[a-zA-Z0-9]$/) ||
        singleKey.startsWith("NumPad") ||
        singleKey.match(/^[0-9]$/)
      ) {
        setError(
          `'${hotkeyString}' would block normal typing. Please use combinations like 'Ctrl + ${singleKey}' or 'Alt + ${singleKey}'.`
        );
        return;
      }
    }

    try {
      // First, remove any existing hotkey for this sound (one hotkey per sound policy)
      if (existingHotkey && existingHotkey !== hotkeyString) {
        try {
          await invoke("unregister_hotkey", {
            hotkey: existingHotkey,
          });
          console.log(`Removed old hotkey: ${existingHotkey}`);
        } catch (err) {
          console.warn(`Failed to remove old hotkey ${existingHotkey}:`, err);
          // Continue anyway - maybe it wasn't properly registered
        }
      }

      // Check if hotkey is already registered
      const isRegistered = await invoke<boolean>("is_hotkey_registered", {
        hotkey: hotkeyString,
      });

      if (isRegistered) {
        // Check if it's registered to this sound or another
        const assignedTo = hotkeyMappings.mappings[hotkeyString];
        if (assignedTo && assignedTo !== sound.id) {
          setError(
            `Hotkey '${hotkeyString}' is already assigned to another sound`
          );
          return;
        }
      }

      // Register the new hotkey
      await invoke("register_hotkey", {
        hotkey: hotkeyString,
        soundId: sound.id,
      });

      setStatus(
        `✓ Hotkey '${formatHotkeyForDisplay(hotkeyString)}' assigned successfully`
      );
      setIsCapturing(false);
      setCapturedKeys([]);
      setError("");

      // Notify parent
      onHotkeyAssigned();

      // Don't auto-close modal - let user close it manually
    } catch (err) {
      setError(`Failed to assign hotkey: ${err}`);
      setIsCapturing(false);
    }
  };

  // Remove existing hotkey
  const removeHotkey = async () => {
    if (!existingHotkey) return;

    try {
      await invoke("unregister_hotkey", {
        hotkey: existingHotkey,
      });

      setStatus(`✓ Hotkey '${formatHotkeyForDisplay(existingHotkey)}' removed`);
      setError("");

      // Notify parent
      onHotkeyAssigned();

      // Don't auto-close modal - let user close it manually
    } catch (err) {
      setError(`Failed to remove hotkey: ${err}`);
    }
  };

  // Auto-save hotkey after user stops pressing keys (500ms debounce)
  useEffect(() => {
    if (!isCapturing || capturedKeys.length === 0) return;

    const timer = setTimeout(() => {
      // Automatically save the captured hotkey
      assignHotkey();
    }, 500);

    return () => clearTimeout(timer);
  }, [capturedKeys, isCapturing]);

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      onClick={onClose}
    >
      <div
        className="bg-discord-dark rounded-lg p-6 w-[500px] max-h-[80vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-discord-text">
            Assign Hotkey
          </h2>
          <button
            onClick={onClose}
            className="text-discord-text-muted hover:text-discord-text transition-colors"
          >
            ✕
          </button>
        </div>

        {/* Sound info */}
        <div className="bg-discord-darker rounded p-3 mb-4">
          <div className="flex items-center gap-2">
            {sound.icon && <span className="text-2xl">{sound.icon}</span>}
            <div>
              <p className="text-discord-text font-medium">{sound.name}</p>
              <p className="text-xs text-discord-text-muted">
                Press a keyboard shortcut to trigger this sound
              </p>
            </div>
          </div>
        </div>

        {/* Existing hotkey */}
        {existingHotkey && !isCapturing && (
          <div className="bg-discord-success/10 border border-discord-success rounded p-3 mb-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-discord-text-muted mb-1">
                  Current hotkey:
                </p>
                <p className="text-lg font-mono text-discord-success">
                  {formatHotkeyForDisplay(existingHotkey)}
                </p>
              </div>
              <button
                onClick={removeHotkey}
                className="px-3 py-1.5 bg-discord-danger hover:bg-discord-danger-hover rounded text-white text-sm transition-colors"
              >
                Remove
              </button>
            </div>
          </div>
        )}

        {/* Capture area */}
        {!isCapturing ? (
          <button
            onClick={startCapture}
            className="w-full py-3 bg-discord-primary hover:bg-discord-primary-hover rounded text-white font-medium transition-colors"
          >
            {existingHotkey ? "Change Hotkey" : "Capture Hotkey"}
          </button>
        ) : (
          <div className="space-y-3">
            {/* Captured keys display */}
            <div className="bg-discord-darker border-2 border-discord-primary rounded p-4 min-h-[80px] flex items-center justify-center">
              {capturedKeys.length > 0 ? (
                <span className="text-2xl font-mono text-discord-primary">
                  {formatHotkeyForDisplay(capturedKeys.join("+"))}
                </span>
              ) : (
                <span className="text-discord-text-muted">
                  Waiting for key combination...
                </span>
              )}
            </div>

            {/* Action buttons */}
            <div className="flex gap-2">
              <button
                onClick={assignHotkey}
                disabled={capturedKeys.length === 0}
                className="flex-1 py-2 bg-discord-success hover:bg-discord-success-hover disabled:bg-discord-darker disabled:text-discord-text-muted disabled:cursor-not-allowed rounded text-white font-medium transition-colors"
              >
                Save Now
              </button>
              <button
                onClick={cancelCapture}
                className="flex-1 py-2 bg-discord-darker hover:bg-discord-darkest rounded text-discord-text font-medium transition-colors"
              >
                Cancel
              </button>
            </div>

            <p className="text-xs text-discord-text-muted text-center">
              💡 Auto-saves 0.5s after releasing keys, or click "Save Now"
              <br />
              Use combinations like Ctrl+NumPad8, Alt+F5. Single keys block
              typing.
            </p>
          </div>
        )}

        {/* Status messages */}
        {status && (
          <div className="mt-3 p-2 bg-discord-success/10 border border-discord-success rounded text-sm text-discord-success">
            {status}
          </div>
        )}

        {/* Error messages */}
        {error && (
          <div className="mt-3 p-2 bg-discord-danger/10 border border-discord-danger rounded text-sm text-discord-danger">
            {error}
          </div>
        )}
      </div>
    </div>
  );
}
