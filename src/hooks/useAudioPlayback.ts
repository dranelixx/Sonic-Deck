import { useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Sound } from "../types";
import { DEBUG, ANIMATION_DURATIONS } from "../constants";

// Playback progress event payload (matches Rust struct)
interface PlaybackProgress {
  playback_id: string;
  elapsed_ms: number;
  total_ms: number;
  progress_pct: number;
}

// Active waveform state for header display
export interface ActiveWaveform {
  soundId: string;
  soundName: string;
  filePath: string;
  currentTimeMs: number;
  durationMs: number;
  trimStartMs: number | null;
  trimEndMs: number | null;
}

interface UseAudioPlaybackProps {
  device1: string;
  device2: string;
  volume: number;
  showToast: (message: string) => void;
  soundLibrary: { sounds: Sound[] };
}

export function useAudioPlayback({
  device1,
  device2,
  volume,
  showToast,
  soundLibrary,
}: UseAudioPlaybackProps) {
  const [playingSoundIds, setPlayingSoundIds] = useState<Set<string>>(
    new Set()
  );
  const [activeWaveform, setActiveWaveform] = useState<ActiveWaveform | null>(
    null
  );
  const [isWaveformExiting, setIsWaveformExiting] = useState(false);

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
          await new Promise((resolve) =>
            setTimeout(resolve, ANIMATION_DURATIONS.CLEANUP_DELAY)
          );
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

  const stopAllAudio = useCallback(async () => {
    try {
      await invoke("stop_all_audio");
      playingSoundsRef.current.clear();
      setPlayingSoundIds(new Set());

      // Trigger exit animation before removing waveform
      setIsWaveformExiting(true);
      setTimeout(() => {
        setActiveWaveform(null);
        setIsWaveformExiting(false);
      }, ANIMATION_DURATIONS.WAVEFORM_EXIT);

      showToast("All audio stopped");
    } catch (error) {
      showToast(`Stop Error: ${error}`);
    }
  }, [showToast]);

  // Setup audio event listeners
  const setupAudioListeners = useCallback(() => {
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
          }, ANIMATION_DURATIONS.WAVEFORM_EXIT);
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

    return () => {
      unlisten.then((fn: () => void) => fn());
      unlistenError.then((fn: () => void) => fn());
      unlistenComplete.then((fn: () => void) => fn());
      unlistenProgress.then((fn: () => void) => fn());
    };
  }, [showToast, soundLibrary.sounds]);

  return {
    playingSoundIds,
    activeWaveform,
    isWaveformExiting,
    playSound,
    stopAllAudio,
    setupAudioListeners,
  };
}
