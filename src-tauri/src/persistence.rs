//! Atomic file persistence utilities
//!
//! Provides crash-safe file writing using the write-to-temp-and-rename pattern.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;
use tracing::debug;

/// Writes data atomically to a file.
///
/// Uses the pattern: tempfile → write → flush → fsync → rename
/// This ensures that either the old file or the new file exists,
/// but never a corrupted partial write.
pub fn atomic_write(path: &Path, data: &str) -> Result<(), String> {
    let start = Instant::now();
    let bytes_written = data.len();
    let path_str = path.display().to_string();

    debug!(path = %path_str, bytes = bytes_written, "Starting atomic write");

    let temp_path = path.with_extension("json.tmp");

    // Create temp file
    let file =
        File::create(&temp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut writer = BufWriter::new(file);

    // Write data to buffer
    writer
        .write_all(data.as_bytes())
        .map_err(|e| format!("Failed to write data: {}", e))?;

    // Flush buffer to OS
    writer
        .flush()
        .map_err(|e| format!("Failed to flush buffer: {}", e))?;

    // Force sync to disk (fsync)
    writer
        .get_ref()
        .sync_all()
        .map_err(|e| format!("Failed to sync to disk: {}", e))?;

    // Atomic rename (overwrites target on Windows)
    fs::rename(&temp_path, path).map_err(|e| format!("Failed to rename temp file: {}", e))?;

    let duration_ms = start.elapsed().as_millis();
    debug!(
        path = %path_str,
        bytes_written = bytes_written,
        duration_ms = duration_ms,
        "Atomic write complete"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_atomic_write_basic() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        let result = atomic_write(&file_path, r#"{"key": "value"}"#);

        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn test_atomic_write_content_integrity() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        let content = r#"{"name": "test", "value": 42}"#;

        atomic_write(&file_path, content).unwrap();

        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_atomic_write_overwrites() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        // Write first version
        atomic_write(&file_path, "version1").unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "version1");

        // Overwrite with second version
        atomic_write(&file_path, "version2").unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "version2");
    }

    #[test]
    fn test_atomic_write_no_temp_residue() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        let temp_path = temp_dir.path().join("test.json.tmp");

        atomic_write(&file_path, "content").unwrap();

        // Temp file should not exist after successful write
        assert!(!temp_path.exists());
        assert!(file_path.exists());
    }

    #[test]
    fn test_atomic_write_large_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.json");

        // Create a large JSON-like string (~100KB)
        let large_content: String = (0..10000)
            .map(|i| format!(r#"{{"item": {}}}"#, i))
            .collect::<Vec<_>>()
            .join(",");
        let content = format!("[{}]", large_content);

        let result = atomic_write(&file_path, &content);

        assert!(result.is_ok());
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content.len(), content.len());
    }

    #[test]
    fn test_atomic_write_unicode() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unicode.json");
        let content = r#"{"emoji": "🎵🔊🎧", "german": "Größe", "japanese": "音楽"}"#;

        atomic_write(&file_path, content).unwrap();

        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_atomic_write_empty_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.json");

        atomic_write(&file_path, "").unwrap();

        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, "");
    }
}
