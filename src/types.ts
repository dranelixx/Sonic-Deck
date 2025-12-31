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
  microphone_routing_device_id: string | null; // Microphone device ID for VB-Cable routing
  microphone_routing_enabled: boolean; // Whether microphone routing is enabled
}

// ============================================================================
// Playback Types
// ============================================================================

/** Result of play_dual_output indicating what action was taken */
export interface PlaybackResult {
  playback_id: string | null;
  action: "started" | "restarted" | "ignored";
  stopped_playback_id: string | null;
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
// (Removed DashboardProps and SettingsProps - now using Context API)

// ============================================================================
// Hotkey Types
// ============================================================================

export interface HotkeyMapping {
  mappings: Record<string, string>; // hotkey -> sound_id
}

// ============================================================================
// VB-Cable Types
// ============================================================================

export interface VbCableInfo {
  output_device: string;
  input_device: string | null;
}

export type VbCableStatus =
  | { status: "installed"; info: VbCableInfo }
  | { status: "notInstalled" };

/** All 4 Windows default audio device settings */
export interface SavedDefaults {
  render_console: string | null;
  render_communications: string | null;
  capture_console: string | null;
  capture_communications: string | null;
}

/** Details about a failed device restore */
export interface RestoreFailure {
  device_role: string;
  error: string;
}

/** Result of restoring default devices */
export interface RestoreResult {
  restored_count: number;
  failed_count: number;
  failures: RestoreFailure[];
}
