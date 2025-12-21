//! Log file management commands
//!
//! Provides access to application logs for debugging and support.

use tracing::info;

/// Get logs directory path
#[tauri::command]
pub fn get_logs_path() -> Result<String, String> {
    let logs_dir = dirs::data_local_dir()
        .ok_or("Could not find app data directory")?
        .join("com.sonicdeck.app")
        .join("logs");

    Ok(logs_dir.to_string_lossy().to_string())
}

/// Read the current log file
#[tauri::command]
pub fn read_logs() -> Result<String, String> {
    let logs_dir = dirs::data_local_dir()
        .ok_or("Could not find app data directory")?
        .join("com.sonicdeck.app")
        .join("logs");

    // Find the most recent log file
    let log_files = std::fs::read_dir(&logs_dir)
        .map_err(|e| format!("Failed to read logs directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "log")
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    if log_files.is_empty() {
        return Ok("No log files found.".to_string());
    }

    // Get the most recent log file (by modified time)
    let most_recent = log_files
        .iter()
        .max_by_key(|entry| entry.metadata().and_then(|m| m.modified()).ok())
        .ok_or("Failed to find recent log file")?;

    std::fs::read_to_string(most_recent.path())
        .map_err(|e| format!("Failed to read log file: {}", e))
}

/// Clear all log files
#[tauri::command]
pub fn clear_logs() -> Result<(), String> {
    let logs_dir = dirs::data_local_dir()
        .ok_or("Could not find app data directory")?
        .join("com.sonicdeck.app")
        .join("logs");

    if !logs_dir.exists() {
        return Ok(());
    }

    let log_files = std::fs::read_dir(&logs_dir)
        .map_err(|e| format!("Failed to read logs directory: {}", e))?;

    for entry in log_files.filter_map(|e| e.ok()) {
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("log") {
            std::fs::remove_file(entry.path())
                .map_err(|e| format!("Failed to delete log file: {}", e))?;
        }
    }

    info!("Log files cleared by user");
    Ok(())
}
