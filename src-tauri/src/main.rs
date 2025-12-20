// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::PathBuf;

fn setup_logging() {
    // Get app data directory for logs
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.sonicdeck.app")
        .join("logs");
    
    // Create logs directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir).ok();
    
    // Clone for later use
    let logs_path = app_data_dir.clone();
    
    // Create file appender with daily rotation (keeps last 7 days)
    // We use empty prefix and "log" suffix to get format: YYYY-MM-DD.log
    // Then we'll manually rename or use a custom wrapper
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("sonicdeck")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&app_data_dir)
        .expect("Failed to create log appender");
    
    // Create file layer
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false) // No color codes in log files
        .with_target(true)
        .with_thread_ids(true);
    
    // Create console layer (only in debug builds)
    let console_layer = if cfg!(debug_assertions) {
        Some(fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(true))
    } else {
        None
    };
    
    // Setup log filter: INFO level by default, DEBUG for our app in dev mode
    let filter = if cfg!(debug_assertions) {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("sonicdeck=debug,warn")) // Only debug for our crate, warn for others
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("sonicdeck=info,warn")) // Info for our crate, warn for others
    };
    
    // Initialize subscriber with both layers
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();
    
    tracing::info!("SonicDeck v{} starting up", env!("CARGO_PKG_VERSION"));
    tracing::info!("Logs directory: {}", logs_path.display());
}

fn main() {
    setup_logging();
    sonicdeck::run();
}
