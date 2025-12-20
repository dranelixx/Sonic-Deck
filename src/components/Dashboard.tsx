import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { DashboardProps, Sound } from "../types";
import CategoryTabs from "./CategoryTabs";
import SoundButton from "./SoundButton";
import SoundModal from "./SoundModal";
import FullWaveform from "./FullWaveform";
import Toast from "./Toast";
import TrimEditor from "./TrimEditor";

// Playback progress event payload (matches Rust struct)
interface PlaybackProgress {
  playback_id: string;
  elapsed_ms: number;
  total_ms: number;
  progress_pct: number;
}

// Active waveform state for header display
interface ActiveWaveform {
  soundId: string;
  soundName: string;
  filePath: string;
  currentTimeMs: number;
  durationMs: number;
  trimStartMs: number | null;
  trimEndMs: number | null;
}

// Debug logging flag - only active in development
const DEBUG = import.meta.env.DEV;

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
  const [playingSoundIds, setPlayingSoundIds] = useState<Set<string>>(
    new Set()
  );
  const [hasLoadedSettings, setHasLoadedSettings] = useState<boolean>(false);

  // Toast notification state
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  // Active waveform state for header display
  const [activeWaveform, setActiveWaveform] = useState<ActiveWaveform | null>(
    null
  );
  const [isWaveformExiting, setIsWaveformExiting] = useState(false);

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

  // Drag & drop state
  const [isDragging, setIsDragging] = useState(false);

  // Global context menu state - only one menu open at a time
  const [openContextMenu, setOpenContextMenu] = useState<{
    type: "sound" | "category";
    id: string;
  } | null>(null);

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

  // Helper to show toast notifications
  const showToast = useCallback((message: string) => {
    setToastMessage(message);
  }, []);

  // Listen for audio events and file drops
  useEffect(() => {
    const unlisten = listen<string>("audio-decode-complete", (event) => {
      if (DEBUG) console.log(`Playing (ID: ${event.payload})`);
    });

    const unlistenError = listen<string>("audio-decode-error", (event) => {
      showToast(`Decode Error: ${event.payload}`);
    });

    const unlistenComplete = listen<string>("playback-complete", (event) => {
      const completedPlaybackId = event.payload;
      if (DEBUG)
        console.log(`[COMPLETE] Playback complete: ${completedPlaybackId}`);

      // Use REF to find sound ID (avoid stale closure from state)
      let soundId: string | null = null;
      for (const [sid, pid] of playingSoundsRef.current.entries()) {
        if (pid === completedPlaybackId) {
          soundId = sid;
          break;
        }
      }

      if (!soundId) {
        if (DEBUG)
          console.log(
            `[WARN] Playback ${completedPlaybackId} not found in ref`
          );
        return;
      }

      if (DEBUG)
        console.log(`[CLEANUP] Cleaning up playback for sound: ${soundId}`);

      // Clean up all tracking
      playingSoundsRef.current.delete(soundId);
      setPlayingSoundIds((prev) => {
        const next = new Set(prev);
        next.delete(soundId);
        return next;
      });

      // Clear active waveform when playback completes
      setActiveWaveform((prev) => {
        if (prev?.soundId === soundId) {
          // Check if there are other sounds still playing
          const otherPlayingSounds = Array.from(
            playingSoundsRef.current.keys()
          ).filter((id) => id !== soundId);
          if (otherPlayingSounds.length > 0) {
            // Switch to the last remaining sound (newest)
            const nextSoundId =
              otherPlayingSounds[otherPlayingSounds.length - 1];
            const nextSound = soundLibrary.sounds.find(
              (s) => s.id === nextSoundId
            );
            if (nextSound) {
              return {
                soundId: nextSound.id,
                soundName: nextSound.name,
                filePath: nextSound.file_path,
                currentTimeMs: 0,
                durationMs: 0,
                trimStartMs: nextSound.trim_start_ms ?? null,
                trimEndMs: nextSound.trim_end_ms ?? null,
              };
            }
          }
          // Trigger exit animation before removing
          setIsWaveformExiting(true);
          setTimeout(() => {
            setIsWaveformExiting(false);
          }, 300);
          return null;
        }
        return prev;
      });
    });

    // Listen for playback progress events
    const unlistenProgress = listen<PlaybackProgress>(
      "playback-progress",
      (event) => {
        const { playback_id, elapsed_ms, total_ms } = event.payload;

        // Find which sound is playing
        for (const [soundId, pid] of playingSoundsRef.current.entries()) {
          if (pid === playback_id) {
            setActiveWaveform((prev) => {
              if (prev?.soundId === soundId) {
                return {
                  ...prev,
                  currentTimeMs: elapsed_ms,
                  durationMs: total_ms,
                };
              }
              return prev;
            });
            break;
          }
        }
      }
    );

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
          // Multi-file import: queue all files
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
        } else {
          showToast("Please drop an audio file (MP3, WAV, OGG, M4A)");
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
      unlisten.then((fn: () => void) => fn());
      unlistenError.then((fn: () => void) => fn());
      unlistenComplete.then((fn: () => void) => fn());
      unlistenProgress.then((fn: () => void) => fn());
      unlistenFileDrop.then((fn: () => void) => fn());
      unlistenFileHover.then((fn: () => void) => fn());
      unlistenFileCancel.then((fn: () => void) => fn());
    };
  }, [showToast]);

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

  // Apply favorites filter if active
  const filteredSounds = showFavoritesOnly
    ? favoriteSounds
    : [...favoriteSounds, ...regularSounds];

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
  }, [devices.length]);

  // Track playing sounds
  const playingSoundsRef = useRef<Map<string, string>>(new Map()); // sound_id -> playback_id

  const playSound = useCallback(
    async (sound: Sound) => {
      if (!device1 || !device2) {
        showToast("Please configure audio devices in Settings first");
        return;
      }

      if (DEBUG) {
        console.log(`\n=== PlaySound: ${sound.name} ===`);
        console.log(
          `Ref state:`,
          Array.from(playingSoundsRef.current.entries())
        );
        console.log(`Playing IDs:`, Array.from(playingSoundIds));
      }

      // Check ref for immediate state
      const currentPlaybackId = playingSoundsRef.current.get(sound.id);
      const shouldRestart = !!currentPlaybackId;

      if (DEBUG)
        console.log(
          `Should restart: ${shouldRestart}, Playback ID: ${currentPlaybackId || "NONE"}`
        );

      if (shouldRestart && currentPlaybackId) {
        if (DEBUG)
          console.log(
            `[RESTART] Restarting: ${sound.name} (${currentPlaybackId})`
          );

        try {
          await invoke("stop_playback", { playbackId: currentPlaybackId });
          if (DEBUG)
            console.log(`[STOP] Stopped playback: ${currentPlaybackId}`);

          // Clean up tracking
          playingSoundsRef.current.delete(sound.id);
          setPlayingSoundIds((prev) => {
            const next = new Set(prev);
            next.delete(sound.id);
            return next;
          });

          // Wait for cleanup
          await new Promise((resolve) => setTimeout(resolve, 100));
          if (DEBUG) console.log(`[CLEANUP] Cleanup complete`);
        } catch (err) {
          console.error("Failed to stop playback:", err);
        }
      } else {
        if (DEBUG) console.log(`[PLAY] Starting fresh playback`);
      }

      // Start playback
      try {
        setPlayingSoundIds((prev) => new Set(prev).add(sound.id));

        const playbackVolume = sound.volume ?? volume;

        const playbackId = await invoke<string>("play_dual_output", {
          filePath: sound.file_path,
          deviceId1: device1,
          deviceId2: device2,
          volume: playbackVolume,
          trimStartMs: sound.trim_start_ms,
          trimEndMs: sound.trim_end_ms,
        });

        if (DEBUG)
          console.log(
            `[PLAY] Started playback: ${playbackId} for ${sound.name}`
          );

        // Track in ref
        playingSoundsRef.current.set(sound.id, playbackId);

        // Set active waveform for header display
        setActiveWaveform((prev) => {
          // If no waveform is active, show this sound
          if (!prev) {
            return {
              soundId: sound.id,
              soundName: sound.name,
              filePath: sound.file_path,
              currentTimeMs: 0,
              durationMs: 0, // Will be updated by progress events
              trimStartMs: sound.trim_start_ms ?? null,
              trimEndMs: sound.trim_end_ms ?? null,
            };
          }
          // If waveform is for a sound that's no longer playing, replace it
          if (!playingSoundsRef.current.has(prev.soundId)) {
            return {
              soundId: sound.id,
              soundName: sound.name,
              filePath: sound.file_path,
              currentTimeMs: 0,
              durationMs: 0,
              trimStartMs: sound.trim_start_ms ?? null,
              trimEndMs: sound.trim_end_ms ?? null,
            };
          }
          // Always switch to the newest sound
          return {
            soundId: sound.id,
            soundName: sound.name,
            filePath: sound.file_path,
            currentTimeMs: 0,
            durationMs: 0,
            trimStartMs: sound.trim_start_ms ?? null,
            trimEndMs: sound.trim_end_ms ?? null,
          };
        });
      } catch (error) {
        console.error(`Playback error:`, error);
        showToast(`Error: ${error}`);
        playingSoundsRef.current.delete(sound.id);
        setPlayingSoundIds((prev) => {
          const next = new Set(prev);
          next.delete(sound.id);
          return next;
        });
      }
    },
    [device1, device2, volume, showToast]
  );

  const stopAllAudio = async () => {
    try {
      await invoke("stop_all_audio");
      playingSoundsRef.current.clear();
      setPlayingSoundIds(new Set());

      // Trigger exit animation before removing waveform
      setIsWaveformExiting(true);
      setTimeout(() => {
        setActiveWaveform(null);
        setIsWaveformExiting(false);
      }, 300);

      showToast("All audio stopped");
    } catch (error) {
      showToast(`Stop Error: ${error}`);
    }
  };

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

  // Note: Drag & drop is handled by Tauri's onFileDropEvent listener above
  // These handlers are kept for visual feedback only
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

  return (
    <div
      className="w-full h-full bg-discord-darkest flex flex-col"
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Header */}
      <div className="bg-discord-darker px-6 py-4 border-b border-discord-dark">
        <div className="flex items-center justify-between gap-4 h-10">
          {/* Left: Logo + Waveform */}
          <div className="flex items-center gap-4 flex-1 min-w-0 h-full">
            <div className="flex-shrink-0 flex items-center h-full">
              <h1
                className="text-xl font-bold text-discord-primary leading-none"
                style={{ transform: "translateY(-3px)" }}
              >
                SonicDeck
              </h1>
            </div>

            {/* Waveform (fades in when playing) */}
            <div className="flex-1 min-w-0 h-full flex items-center">
              {activeWaveform && (
                <div
                  className={`w-full transition-opacity duration-300 ${
                    isWaveformExiting
                      ? "opacity-0"
                      : "opacity-100 animate-fadeIn"
                  }`}
                >
                  <FullWaveform
                    filePath={activeWaveform.filePath}
                    soundName={activeWaveform.soundName}
                    isPlaying={playingSoundIds.has(activeWaveform.soundId)}
                    currentTimeMs={activeWaveform.currentTimeMs}
                    durationMs={activeWaveform.durationMs}
                    trimStartMs={activeWaveform.trimStartMs}
                    trimEndMs={activeWaveform.trimEndMs}
                  />
                </div>
              )}
            </div>
          </div>

          {/* Right: Controls */}
          <div className="flex items-center gap-3 flex-shrink-0 h-full">
            {/* Volume Control */}
            <div className="flex items-center gap-2">
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={volume}
                onChange={(e) => setVolume(parseFloat(e.target.value))}
                className="w-20"
                style={{
                  accentColor: volume >= 0.75 ? "#ef4444" : "#5865f2",
                }}
              />
              <span
                className={`text-xs w-8 ${volume >= 0.75 ? "text-red-400" : "text-discord-text"}`}
              >
                {Math.round(volume * 100)}%
              </span>
            </div>

            {/* Stop Button */}
            <button
              onClick={stopAllAudio}
              className="px-4 py-2 bg-discord-danger hover:bg-red-600 rounded-lg
                       text-white font-medium transition-colors"
            >
              Stop
            </button>
          </div>
        </div>
      </div>

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
      <div className="flex-1 overflow-hidden flex flex-col p-6">
        {/* Category Tabs + Favorites Button */}
        <div className="mb-4 flex items-center justify-between gap-4 pb-2 border-b border-discord-dark">
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
        <div className="flex-1 overflow-auto">
          {filteredSounds.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-discord-text-muted">
              <div className="text-6xl mb-4">🔇</div>
              <p className="text-lg mb-2">No sounds in this category</p>
              <p className="text-sm mb-4">
                Click "Add Sound" or drag & drop an audio file to get started
              </p>
              <button
                onClick={handleAddSound}
                className="px-4 py-2 bg-discord-primary hover:bg-discord-primary-hover
                         rounded-lg text-white font-medium transition-colors"
              >
                + Add Sound
              </button>
            </div>
          ) : (
            <div className="space-y-6">
              {/* Favorites Section */}
              {!showFavoritesOnly && favoriteSounds.length > 0 && (
                <div>
                  <h3 className="text-sm font-semibold text-discord-text mb-3 flex items-center gap-2">
                    <span className="text-yellow-400">⭐</span>
                    Favorites
                  </h3>
                  <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 gap-3">
                    {favoriteSounds.map((sound) => (
                      <SoundButton
                        key={sound.id}
                        sound={sound}
                        isPlaying={playingSoundIds.has(sound.id)}
                        onPlay={playSound}
                        onEdit={handleEditSound}
                        onDelete={handleDeleteSound}
                        onToggleFavorite={handleToggleFavorite}
                        onTrim={handleTrimSound}
                        showMenu={
                          openContextMenu?.type === "sound" &&
                          openContextMenu.id === sound.id
                        }
                        onMenuChange={(show: boolean) =>
                          setOpenContextMenu(
                            show ? { type: "sound", id: sound.id } : null
                          )
                        }
                      />
                    ))}
                  </div>
                </div>
              )}

              {/* Divider */}
              {!showFavoritesOnly &&
                favoriteSounds.length > 0 &&
                regularSounds.length > 0 && (
                  <div className="border-t border-discord-dark"></div>
                )}

              {/* Regular Sounds Section */}
              {!showFavoritesOnly && regularSounds.length > 0 && (
                <div>
                  {favoriteSounds.length > 0 && (
                    <h3 className="text-sm font-semibold text-discord-text-muted mb-3">
                      All Sounds
                    </h3>
                  )}
                  <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 gap-3">
                    {regularSounds.map((sound) => (
                      <SoundButton
                        key={sound.id}
                        sound={sound}
                        isPlaying={playingSoundIds.has(sound.id)}
                        onPlay={playSound}
                        onEdit={handleEditSound}
                        onDelete={handleDeleteSound}
                        onToggleFavorite={handleToggleFavorite}
                        onTrim={handleTrimSound}
                        showMenu={
                          openContextMenu?.type === "sound" &&
                          openContextMenu.id === sound.id
                        }
                        onMenuChange={(show: boolean) =>
                          setOpenContextMenu(
                            show ? { type: "sound", id: sound.id } : null
                          )
                        }
                      />
                    ))}

                    {/* Add Sound Button */}
                    <button
                      onClick={handleAddSound}
                      className="h-24 rounded-lg border-2 border-dashed border-discord-dark
                               text-discord-text-muted hover:border-discord-primary
                               hover:text-discord-primary transition-colors
                               flex flex-col items-center justify-center gap-1"
                    >
                      <span className="text-2xl">+</span>
                      <span className="text-xs">Add Sound</span>
                    </button>
                  </div>
                </div>
              )}

              {/* Favorites Only View */}
              {showFavoritesOnly && (
                <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 gap-3">
                  {favoriteSounds.map((sound) => (
                    <SoundButton
                      key={sound.id}
                      sound={sound}
                      isPlaying={playingSoundIds.has(sound.id)}
                      onPlay={playSound}
                      onEdit={handleEditSound}
                      onDelete={handleDeleteSound}
                      onToggleFavorite={handleToggleFavorite}
                      onTrim={handleTrimSound}
                      showMenu={
                        openContextMenu?.type === "sound" &&
                        openContextMenu.id === sound.id
                      }
                      onMenuChange={(show: boolean) =>
                        setOpenContextMenu(
                          show ? { type: "sound", id: sound.id } : null
                        )
                      }
                    />
                  ))}
                </div>
              )}
            </div>
          )}
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

            // Then reopen with next file after a tiny delay
            setTimeout(() => {
              setFileQueue(remainingFiles);
              setDroppedFilePath(remainingFiles[0]);
              setEditingSound(null);
              setIsModalOpen(true);
              showToast(`Adding sounds (${currentFileNum}/${totalFiles})`);
            }, 100);
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
