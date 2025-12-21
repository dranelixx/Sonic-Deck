import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { DashboardProps, Sound } from "../../types";
import { ANIMATION_DURATIONS } from "../../constants";
import CategoryTabs from "../categories/CategoryTabs";
import DashboardHeader from "./DashboardHeader";
import DashboardSoundGrid from "./DashboardSoundGrid";
import SoundModal from "../modals/SoundModal";
import Toast from "../common/Toast";
import TrimEditor from "../modals/TrimEditor";
import { useAudioPlayback } from "../../hooks/useAudioPlayback";
import { useFileDrop } from "../../hooks/useFileDrop";
import { useHotkeyMappings } from "../../hooks/useHotkeyMappings";

export default function Dashboard({
  devices,
  settings,
  soundLibrary,
  refreshSounds,
  device1,
  device2,
  setDevice1,
  setDevice2,
}: DashboardProps) {
  const [volume, setVolume] = useState<number>(0.5);
  const [hasLoadedSettings, setHasLoadedSettings] = useState<boolean>(false);

  // Toast notification state
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  // Category selection
  const [selectedCategoryId, setSelectedCategoryId] = useState<string>("");
  const [showFavoritesOnly, setShowFavoritesOnly] = useState<boolean>(false);

  // Modal state
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingSound, setEditingSound] = useState<Sound | null>(null);
  const [droppedFilePath, setDroppedFilePath] = useState<string | null>(null);
  const [fileQueue, setFileQueue] = useState<string[]>([]);

  // Trim editor state
  const [trimEditorSound, setTrimEditorSound] = useState<Sound | null>(null);

  // Global context menu state - only one menu open at a time
  const [openContextMenu, setOpenContextMenu] = useState<{
    type: "sound" | "category";
    id: string;
  } | null>(null);

  // Helper to show toast notifications
  const showToast = useCallback((message: string) => {
    setToastMessage(message);
  }, []);

  // Custom Hooks
  const { hotkeyMappings, refreshHotkeys } = useHotkeyMappings();

  const {
    playingSoundIds,
    activeWaveform,
    isWaveformExiting,
    playSound,
    stopAllAudio,
    setupAudioListeners,
  } = useAudioPlayback({
    device1,
    device2,
    volume,
    showToast,
    soundLibrary,
  });

  const handleFilesDropped = useCallback(
    (audioFiles: string[]) => {
      if (audioFiles.length === 1) {
        // Single file - open modal directly
        setDroppedFilePath(audioFiles[0]);
        setEditingSound(null);
        setIsModalOpen(true);
      } else {
        // Multiple files - start queue
        setFileQueue(audioFiles);
        setDroppedFilePath(audioFiles[0]);
        setEditingSound(null);
        setIsModalOpen(true);
        showToast(
          `Adding ${audioFiles.length} sounds (1/${audioFiles.length})`
        );
      }
    },
    [showToast]
  );

  const {
    isDragging,
    setupFileDropListeners,
    handleDragOver,
    handleDragLeave,
    handleDrop,
  } = useFileDrop({
    showToast,
    onFilesDropped: handleFilesDropped,
  });

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
      setHasLoadedSettings(true);
    }
  }, [settings, hasLoadedSettings, setDevice1, setDevice2]);

  // Set initial category when categories load
  useEffect(() => {
    if (soundLibrary.categories.length > 0 && !selectedCategoryId) {
      const sorted = [...soundLibrary.categories].sort(
        (a, b) => a.sort_order - b.sort_order
      );
      setSelectedCategoryId(sorted[0]?.id || "");
    }
  }, [soundLibrary.categories, selectedCategoryId]);

  // Setup audio and file drop listeners
  useEffect(() => {
    const cleanupAudio = setupAudioListeners();
    const cleanupFileDrop = setupFileDropListeners();

    return () => {
      cleanupAudio();
      cleanupFileDrop();
    };
  }, [setupAudioListeners, setupFileDropListeners]);

  // Check if devices are configured
  const devicesConfigured = device1 && device2;

  // Get sounds for selected category
  const categorySounds = soundLibrary.sounds.filter(
    (s) => s.category_id === selectedCategoryId
  );

  // Split into favorites and non-favorites
  const favoriteSounds = categorySounds
    .filter((s) => s.is_favorite)
    .sort((a, b) => a.name.localeCompare(b.name));

  const regularSounds = categorySounds
    .filter((s) => !s.is_favorite)
    .sort((a, b) => a.name.localeCompare(b.name));

  // Check if selected devices are still available
  useEffect(() => {
    if (devices.length > 0 && device1 && device2) {
      const device1Available = devices.some((d) => d.id === device1);
      const device2Available = devices.some((d) => d.id === device2);

      if (!device1Available) {
        setDevice1("");
        showToast("Warning: Monitor output device disconnected");
      }
      if (!device2Available) {
        setDevice2("");
        showToast("Warning: Broadcast output device disconnected");
      }
    }
  }, [devices, device1, device2, setDevice1, setDevice2, showToast]);

  const handleAddSound = async () => {
    // Open file dialog for multiple files
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: "Audio Files",
            extensions: ["mp3", "wav", "ogg", "m4a", "flac"],
          },
        ],
      });

      if (!selected) return; // User cancelled

      // Handle both single file (string) and multiple files (array)
      const files = Array.isArray(selected) ? selected : [selected];
      const audioFiles = files.filter((path: string) =>
        /\.(mp3|wav|ogg|m4a|flac)$/i.test(path)
      );

      if (audioFiles.length === 0) {
        showToast("No audio files selected");
        return;
      }

      handleFilesDropped(audioFiles);
    } catch (error) {
      console.error("File dialog error:", error);
      showToast(`Error opening file dialog: ${error}`);
    }
  };

  const handleEditSound = (sound: Sound) => {
    setEditingSound(sound);
    setDroppedFilePath(null);
    setIsModalOpen(true);
  };

  const handleDeleteSound = async (sound: Sound) => {
    if (!confirm(`Delete "${sound.name}"?`)) return;

    try {
      await invoke("delete_sound", { soundId: sound.id });
      await refreshSounds();
      showToast(`Deleted: ${sound.name}`);
    } catch (error) {
      showToast(`Delete Error: ${error}`);
    }
  };

  const handleToggleFavorite = async (sound: Sound) => {
    try {
      await invoke("toggle_favorite", {
        soundId: sound.id,
      });
      await refreshSounds();
      showToast(
        sound.is_favorite
          ? `Removed from favorites: ${sound.name}`
          : `Added to favorites: ${sound.name}`
      );
    } catch (error) {
      showToast(`Favorite Error: ${error}`);
    }
  };

  const handleTrimSound = (sound: Sound) => {
    setTrimEditorSound(sound);
  };

  const handleTrimSave = useCallback(
    async (_trimStartMs: number | null, _trimEndMs: number | null) => {
      await refreshSounds();
      showToast("Trim saved successfully");
    },
    [refreshSounds, showToast]
  );

  return (
    <div
      className="w-full h-full bg-discord-darkest flex flex-col"
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Header */}
      <DashboardHeader
        volume={volume}
        onVolumeChange={setVolume}
        onStopAll={stopAllAudio}
        activeWaveform={activeWaveform}
        isWaveformExiting={isWaveformExiting}
        playingSoundIds={playingSoundIds}
      />

      {/* Device Warning */}
      {!devicesConfigured && (
        <div className="mx-6 mt-4 bg-discord-warning/20 border border-discord-warning rounded-lg p-4">
          <h3 className="text-discord-warning font-semibold mb-1">
            Audio Devices Not Configured
          </h3>
          <p className="text-sm text-discord-text-muted">
            Please configure your Monitor and Broadcast devices in Settings
            before playing sounds.
          </p>
        </div>
      )}

      {/* Main Content */}
      <div className="flex-1 overflow-hidden flex flex-col py-6">
        {/* Category Tabs + Favorites Button */}
        <div className="mb-4 flex items-center justify-between gap-4 pb-2 border-b border-discord-dark px-6">
          <div className="flex-1 min-w-0">
            <CategoryTabs
              categories={soundLibrary.categories}
              selectedCategoryId={selectedCategoryId}
              onSelectCategory={setSelectedCategoryId}
              onCategoriesChange={refreshSounds}
              openContextMenuId={
                openContextMenu?.type === "category" ? openContextMenu.id : null
              }
              onContextMenuChange={(categoryId) =>
                setOpenContextMenu(
                  categoryId ? { type: "category", id: categoryId } : null
                )
              }
            />
          </div>

          {/* Favorites Filter Toggle */}
          <button
            onClick={() => setShowFavoritesOnly(!showFavoritesOnly)}
            className={`px-3 py-2 rounded-lg font-medium transition-all flex-shrink-0
                     ${
                       showFavoritesOnly
                         ? "bg-yellow-500 text-white hover:bg-yellow-600"
                         : "bg-discord-dark text-discord-text-muted hover:bg-discord-darker hover:text-discord-text"
                     }`}
            title={
              showFavoritesOnly ? "Show all sounds" : "Show favorites only"
            }
          >
            <span className="text-xl">⭐</span>
          </button>
        </div>

        {/* Sound Grid */}
        <div className="flex-1 overflow-auto px-6">
          <DashboardSoundGrid
            favoriteSounds={favoriteSounds}
            regularSounds={regularSounds}
            showFavoritesOnly={showFavoritesOnly}
            playingSoundIds={playingSoundIds}
            onPlay={playSound}
            onEdit={handleEditSound}
            onDelete={handleDeleteSound}
            onToggleFavorite={handleToggleFavorite}
            onTrim={handleTrimSound}
            onAddSound={handleAddSound}
            openContextMenu={openContextMenu}
            onContextMenuChange={(menu) => setOpenContextMenu(menu)}
            hotkeyMappings={hotkeyMappings}
            onHotkeyChanged={refreshHotkeys}
          />
        </div>
      </div>

      {/* Drag & Drop Overlay */}
      {isDragging && (
        <div
          className="absolute inset-0 z-40 bg-discord-primary/20 border-4 border-dashed
                      border-discord-primary flex items-center justify-center pointer-events-none"
        >
          <div className="bg-discord-dark rounded-lg p-8 text-center">
            <div className="text-6xl mb-4">🎵</div>
            <p className="text-xl text-discord-text font-medium">
              Drop audio file to add sound
            </p>
            <p className="text-sm text-discord-text-muted mt-2">
              Supports MP3, WAV, OGG, M4A
            </p>
          </div>
        </div>
      )}

      {/* Sound Modal */}
      <SoundModal
        isOpen={isModalOpen}
        onClose={() => {
          setIsModalOpen(false);
          setEditingSound(null);
          setDroppedFilePath(null);
          setFileQueue([]);
        }}
        onSave={async () => {
          await refreshSounds();

          // Check if there are more files in queue
          if (fileQueue.length > 1) {
            // Remove first file and continue with next
            const remainingFiles = fileQueue.slice(1);
            const totalFiles = fileQueue.length;
            const currentFileNum = totalFiles - remainingFiles.length + 1;

            // Close modal briefly to trigger useEffect reset
            setIsModalOpen(false);

            // Then reopen with next file after a brief delay
            setTimeout(() => {
              setFileQueue(remainingFiles);
              setDroppedFilePath(remainingFiles[0]);
              setEditingSound(null);
              setIsModalOpen(true);
              showToast(`Adding sounds (${currentFileNum}/${totalFiles})`);
            }, ANIMATION_DURATIONS.MODAL_TRANSITION);
          } else {
            // Queue finished
            const totalAdded = fileQueue.length;
            setFileQueue([]);
            setDroppedFilePath(null);
            if (totalAdded > 0) {
              showToast(`Successfully added ${totalAdded} sounds!`);
            }
          }
        }}
        categories={soundLibrary.categories}
        sound={editingSound}
        defaultCategoryId={selectedCategoryId}
        defaultFilePath={droppedFilePath || undefined}
      />

      {/* Toast Notification */}
      {toastMessage && (
        <Toast message={toastMessage} onClose={() => setToastMessage(null)} />
      )}

      {/* Trim Editor Modal */}
      {trimEditorSound && (
        <TrimEditor
          sound={trimEditorSound}
          onClose={() => setTrimEditorSound(null)}
          onSave={handleTrimSave}
        />
      )}
    </div>
  );
}
