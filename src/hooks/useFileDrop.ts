import { useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";

// Debug logging flag - only active in development
const DEBUG = import.meta.env.DEV;

interface UseFileDropProps {
  showToast: (message: string) => void;
  onFilesDropped: (files: string[]) => void;
}

export function useFileDrop({ showToast, onFilesDropped }: UseFileDropProps) {
  const [isDragging, setIsDragging] = useState(false);

  // Setup file drop listeners
  const setupFileDropListeners = useCallback(() => {
    // Listen for file drops (Tauri v2 event system)
    const unlistenFileDrop = listen<{ paths: string[] }>(
      "tauri://drag-drop",
      (event) => {
        if (DEBUG) console.log("FILE DROP EVENT:", event);
        // Payload structure: { paths: string[], position: { x, y } }
        const paths = event.payload.paths || event.payload;
        if (DEBUG) console.log("Extracted paths:", paths);

        if (!Array.isArray(paths)) {
          console.error("Unexpected payload format:", event.payload);
          return;
        }

        const audioFiles = paths.filter((path: string) =>
          /\.(mp3|wav|ogg|m4a|flac)$/i.test(path)
        );

        if (audioFiles.length > 0) {
          if (DEBUG)
            console.log(
              `[DROP] Dropped ${audioFiles.length} audio file(s):`,
              audioFiles
            );
          onFilesDropped(audioFiles);
          setIsDragging(false);
        } else {
          showToast("Please drop an audio file (MP3, WAV, OGG, M4A)");
          setIsDragging(false);
        }
      }
    );

    const unlistenFileHover = listen("tauri://drag", () => {
      if (DEBUG) console.log("FILE HOVER EVENT");
      setIsDragging(true);
    });

    const unlistenFileCancel = listen("tauri://drag-cancelled", () => {
      if (DEBUG) console.log("FILE CANCEL EVENT");
      setIsDragging(false);
    });

    if (DEBUG) console.log("[INIT] File drop listeners registered");

    return () => {
      unlistenFileDrop.then((fn: () => void) => fn());
      unlistenFileHover.then((fn: () => void) => fn());
      unlistenFileCancel.then((fn: () => void) => fn());
    };
  }, [showToast, onFilesDropped]);

  // Drag & drop handlers for visual feedback
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  return {
    isDragging,
    setupFileDropListeners,
    handleDragOver,
    handleDragLeave,
    handleDrop,
  };
}
