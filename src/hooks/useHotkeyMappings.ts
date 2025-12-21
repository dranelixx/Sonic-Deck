import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { HotkeyMapping } from "../types";

export function useHotkeyMappings() {
  const [hotkeyMappings, setHotkeyMappings] = useState<HotkeyMapping>({
    mappings: {},
  });

  // Load hotkey mappings on mount
  useEffect(() => {
    const loadHotkeys = async () => {
      try {
        const mappings = await invoke<HotkeyMapping>("load_hotkeys");
        setHotkeyMappings(mappings);
      } catch (error) {
        console.error("Failed to load hotkey mappings:", error);
      }
    };
    loadHotkeys();
  }, []);

  // Refresh hotkey mappings (called when hotkeys change)
  const refreshHotkeys = useCallback(async () => {
    try {
      const mappings = await invoke<HotkeyMapping>("load_hotkeys");
      setHotkeyMappings(mappings);
    } catch (error) {
      console.error("Failed to refresh hotkey mappings:", error);
    }
  }, []);

  return {
    hotkeyMappings,
    refreshHotkeys,
  };
}
