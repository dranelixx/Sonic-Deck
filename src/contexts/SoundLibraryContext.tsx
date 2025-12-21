import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { SoundLibrary } from "../types";

interface SoundLibraryContextType {
  soundLibrary: SoundLibrary;
  refreshSounds: () => Promise<void>;
  isLoading: boolean;
}

const SoundLibraryContext = createContext<SoundLibraryContextType | undefined>(
  undefined
);

export function SoundLibraryProvider({ children }: { children: ReactNode }) {
  const [soundLibrary, setSoundLibrary] = useState<SoundLibrary>({
    categories: [],
    sounds: [],
  });
  const [isLoading, setIsLoading] = useState(true);

  const refreshSounds = useCallback(async () => {
    try {
      const loadedSounds = await invoke<SoundLibrary>("load_sounds");
      setSoundLibrary(loadedSounds);
    } catch (error) {
      console.error("Failed to refresh sounds:", error);
      throw error;
    }
  }, []);

  useEffect(() => {
    const loadSounds = async () => {
      try {
        await refreshSounds();
      } catch (error) {
        console.error("Failed to load sounds:", error);
      } finally {
        setIsLoading(false);
      }
    };
    loadSounds();
  }, []);

  return (
    <SoundLibraryContext.Provider
      value={{ soundLibrary, refreshSounds, isLoading }}
    >
      {children}
    </SoundLibraryContext.Provider>
  );
}

export function useSoundLibrary() {
  const context = useContext(SoundLibraryContext);
  if (context === undefined) {
    throw new Error("useSoundLibrary must be used within SoundLibraryProvider");
  }
  return context;
}
