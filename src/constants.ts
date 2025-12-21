// ============================================================================
// Animation & Timing Constants
// ============================================================================

export const ANIMATION_DURATIONS = {
  WAVEFORM_EXIT: 300, // ms - Waveform fade out animation
  TOAST_DURATION: 3000, // ms - Default toast notification duration
  TOAST_EXIT_START: 600, // ms - Time before toast starts exit animation
  CLEANUP_DELAY: 100, // ms - Delay for audio cleanup after stop
  MODAL_TRANSITION: 100, // ms - Delay for modal state transitions
} as const;

// ============================================================================
// Debug & Development
// ============================================================================

export const DEBUG = import.meta.env.DEV;
