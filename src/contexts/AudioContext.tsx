import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { AudioDevice } from "../types";

interface AudioContextType {
  devices: AudioDevice[];
  refreshDevices: () => Promise<void>;
  isLoading: boolean;
}

const AudioContext = createContext<AudioContextType | undefined>(undefined);

export function AudioProvider({ children }: { children: ReactNode }) {
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const refreshDevices = useCallback(async () => {
    try {
      const deviceList = await invoke<AudioDevice[]>("list_audio_devices");
      setDevices(deviceList);
    } catch (error) {
      console.error("Failed to refresh devices:", error);
      throw error;
    }
  }, []);

  useEffect(() => {
    const loadDevices = async () => {
      try {
        await refreshDevices();
      } catch (error) {
        console.error("Failed to load devices:", error);
      } finally {
        setIsLoading(false);
      }
    };
    loadDevices();
  }, []);

  return (
    <AudioContext.Provider value={{ devices, refreshDevices, isLoading }}>
      {children}
    </AudioContext.Provider>
  );
}

export function useAudio() {
  const context = useContext(AudioContext);
  if (context === undefined) {
    throw new Error("useAudio must be used within AudioProvider");
  }
  return context;
}
