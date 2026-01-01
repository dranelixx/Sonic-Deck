import { useState, useEffect, useCallback } from "react";

// Dynamically import to avoid errors when Tauri is not available
type Update = {
  version: string;
  body?: string;
  downloadAndInstall: (
    callback: (event: {
      event: string;
      data: { contentLength?: number; chunkLength: number };
    }) => void
  ) => Promise<void>;
};

export interface UpdateState {
  available: Update | null;
  checking: boolean;
  downloading: boolean;
  progress: number;
  error: string | null;
}

export interface UseUpdateCheckReturn extends UpdateState {
  checkForUpdates: () => Promise<void>;
  installUpdate: () => Promise<void>;
  dismissError: () => void;
}

/**
 * Check if running in Tauri context
 */
function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Hook for checking and installing app updates via Tauri updater plugin.
 *
 * Features:
 * - Auto-check on mount (3s delay, silent fail)
 * - Manual check via checkForUpdates()
 * - Download with progress tracking
 * - App relaunch after install
 * - Gracefully handles non-Tauri environments
 */
export function useUpdateCheck(): UseUpdateCheckReturn {
  const [state, setState] = useState<UpdateState>({
    available: null,
    checking: false,
    downloading: false,
    progress: 0,
    error: null,
  });

  const checkForUpdates = useCallback(async () => {
    // Skip if not running in Tauri
    if (!isTauri()) {
      return;
    }

    setState((prev) => ({ ...prev, checking: true, error: null }));
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check({ timeout: 15000 });
      setState((prev) => ({
        ...prev,
        available: update as Update | null,
        checking: false,
      }));
    } catch (error) {
      // Silent fail for auto-check, but store error for manual check feedback
      console.warn("Update check failed:", error);
      setState((prev) => ({
        ...prev,
        checking: false,
        error: null, // Don't show error for failed checks during development
      }));
    }
  }, []);

  const installUpdate = useCallback(async () => {
    if (!state.available || !isTauri()) return;

    setState((prev) => ({ ...prev, downloading: true, progress: 0 }));

    try {
      let totalSize = 0;
      let downloaded = 0;

      await state.available.downloadAndInstall((event) => {
        if (event.event === "Started") {
          totalSize = event.data.contentLength ?? 0;
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          const progress = totalSize > 0 ? (downloaded / totalSize) * 100 : 0;
          setState((prev) => ({ ...prev, progress }));
        }
      });

      // Relaunch after install
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (error) {
      setState((prev) => ({
        ...prev,
        downloading: false,
        error: error instanceof Error ? error.message : "Installation failed",
      }));
    }
  }, [state.available]);

  const dismissError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }));
  }, []);

  // Auto-check on mount with 3s delay (silent fail)
  useEffect(() => {
    const timer = setTimeout(() => {
      checkForUpdates().catch(() => {
        // Silent fail - don't show toast for auto-check
      });
    }, 3000);
    return () => clearTimeout(timer);
  }, [checkForUpdates]);

  return {
    ...state,
    checkForUpdates,
    installUpdate,
    dismissError,
  };
}
