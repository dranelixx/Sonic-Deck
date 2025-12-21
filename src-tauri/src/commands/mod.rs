//! Tauri command handlers organized by functionality
//!
//! This module contains all Tauri commands, grouped into logical submodules:
//! - `audio`: Audio playback, device management, caching, waveforms
//! - `settings`: App settings and autostart configuration
//! - `hotkeys`: Global hotkey registration and management
//! - `sounds`: Sound library and category management
//! - `logs`: Log file access and management

pub mod audio;
pub mod hotkeys;
pub mod logs;
pub mod settings;
pub mod sounds;

// Re-export all commands for easy access in lib.rs
pub use audio::*;
pub use hotkeys::*;
pub use logs::*;
pub use settings::*;
pub use sounds::*;
