import { useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Sound, PlaybackResult } from "../types";
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
      }

      // Start playback - backend handles policy (restart/ignore)
      try {
        const playbackVolume = sound.volume ?? volume;

        const result = await invoke<PlaybackResult>("play_dual_output", {
          filePath: sound.file_path,
          deviceId1: device1,
          deviceId2: device2,
          volume: playbackVolume,
          trimStartMs: sound.trim_start_ms,
          trimEndMs: sound.trim_end_ms,
          soundId: sound.id,
        });

        if (DEBUG) {
          console.log(`[PLAY] Result: ${result.action}`, result);
        }

        // Handle based on action taken by backend
        if (result.action === "ignored") {
          if (DEBUG) console.log(`[IGNORED] Sound already playing`);
          return;
        }

        // If restarted, clean up old tracking first
        if (result.action === "restarted" && result.stopped_playback_id) {
          if (DEBUG) {
            console.log(
              `[RESTART] Stopped ${result.stopped_playback_id}, starting new`
            );
          }
        }

        // Track new playback
        if (result.playback_id) {
          setPlayingSoundIds((prev) => new Set(prev).add(sound.id));
          playingSoundsRef.current.set(sound.id, result.playback_id);

          if (DEBUG) {
            console.log(
              `[PLAY] Started playback: ${result.playback_id} for ${sound.name}`
            );
          }

          // Set active waveform for header display
          setActiveWaveform({
            soundId: sound.id,
            soundName: sound.name,
            filePath: sound.file_path,
            currentTimeMs: 0,
            durationMs: 0, // Will be updated by progress events
            trimStartMs: sound.trim_start_ms ?? null,
            trimEndMs: sound.trim_end_ms ?? null,
          });
        }
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
