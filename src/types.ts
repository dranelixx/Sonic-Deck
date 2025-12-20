// ============================================================================
// Audio Device Types
// ============================================================================

export interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
}

// ============================================================================
// App Settings Types
// ============================================================================

export interface AppSettings {
  monitor_device_id: string | null;
  broadcast_device_id: string | null;
  default_volume: number;
  volume_multiplier: number; // Global volume scaling (0.1 - 1.0), default 0.2
  last_file_path: string | null;
  minimize_to_tray: boolean; // Close button behavior: true = minimize to tray, false = quit app
  start_minimized: boolean; // Start application minimized to tray
  autostart_enabled: boolean; // Enable autostart on system boot
}

// ============================================================================
// Sound Library Types
// ============================================================================

export interface Sound {
  id: string;
  name: string;
  file_path: string;
  category_id: string;
  icon: string | null;
  volume: number | null;
  is_favorite: boolean;
  trim_start_ms: number | null;
  trim_end_ms: number | null;
}

export interface Category {
  id: string;
  name: string;
  icon: string | null;
  sort_order: number;
}

export interface SoundLibrary {
  categories: Category[];
  sounds: Sound[];
}

// ============================================================================
// Component Props Types
// ============================================================================

export interface DashboardProps {
  devices: AudioDevice[];
  settings: AppSettings | null;
  soundLibrary: SoundLibrary;
  refreshDevices: () => Promise<void>;
  refreshSounds: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;
  device1: string;
  device2: string;
  setDevice1: (id: string) => void;
  setDevice2: (id: string) => void;
}

export interface SettingsProps {
  devices: AudioDevice[];
  settings: AppSettings | null;
  refreshDevices: () => Promise<void>;
  reloadSettings: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;
}
