//! Integration tests for audio decoding
//!
//! Tests that the test fixture files exist and are valid.
//! The actual decoding tests run via unit tests in the audio module.

use std::path::PathBuf;

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn get_test_file_path(filename: &str) -> PathBuf {
    fixtures_path().join(filename)
}

// ============================================================================
// Fixture Existence Tests
// ============================================================================

#[test]
fn test_mp3_fixture_exists() {
    let path = get_test_file_path("test_mono.mp3");
    assert!(
        path.exists(),
        "Test fixture test_mono.mp3 not found at {:?}",
        path
    );
}

#[test]
fn test_ogg_fixture_exists() {
    let path = get_test_file_path("test_stereo.ogg");
    assert!(
        path.exists(),
        "Test fixture test_stereo.ogg not found at {:?}",
        path
    );
}

#[test]
fn test_m4a_fixture_exists() {
    let path = get_test_file_path("test_stereo.m4a");
    assert!(
        path.exists(),
        "Test fixture test_stereo.m4a not found at {:?}",
        path
    );
}

// ============================================================================
// File Size Sanity Tests
// ============================================================================

#[test]
fn test_mp3_file_size() {
    let path = get_test_file_path("test_mono.mp3");
    let metadata = std::fs::metadata(&path).unwrap();

    // Should be a small file (< 20KB for 1 second audio)
    assert!(metadata.len() < 20 * 1024, "MP3 file unexpectedly large");
    assert!(metadata.len() > 0, "MP3 file is empty");
}

#[test]
fn test_ogg_file_size() {
    let path = get_test_file_path("test_stereo.ogg");
    let metadata = std::fs::metadata(&path).unwrap();

    // Should be a small file (< 20KB for 1 second audio)
    assert!(metadata.len() < 20 * 1024, "OGG file unexpectedly large");
    assert!(metadata.len() > 0, "OGG file is empty");
}

#[test]
fn test_m4a_file_size() {
    let path = get_test_file_path("test_stereo.m4a");
    let metadata = std::fs::metadata(&path).unwrap();

    // Should be a small file (< 30KB for 1 second audio)
    assert!(metadata.len() < 30 * 1024, "M4A file unexpectedly large");
    assert!(metadata.len() > 0, "M4A file is empty");
}

// ============================================================================
// File Format Header Tests
// ============================================================================

#[test]
fn test_mp3_has_valid_header() {
    let path = get_test_file_path("test_mono.mp3");
    let bytes = std::fs::read(&path).unwrap();

    // MP3 files should start with ID3 tag or frame sync
    // ID3v2: starts with "ID3"
    // Frame sync: 0xFF 0xFB, 0xFF 0xFA, 0xFF 0xF3, 0xFF 0xF2
    let has_id3 = bytes.len() >= 3 && &bytes[0..3] == b"ID3";
    let has_frame_sync = bytes.len() >= 2 && bytes[0] == 0xFF && (bytes[1] & 0xE0) == 0xE0;

    assert!(has_id3 || has_frame_sync, "MP3 file has invalid header");
}

#[test]
fn test_ogg_has_valid_header() {
    let path = get_test_file_path("test_stereo.ogg");
    let bytes = std::fs::read(&path).unwrap();

    // OGG files start with "OggS"
    assert!(
        bytes.len() >= 4 && &bytes[0..4] == b"OggS",
        "OGG file has invalid header"
    );
}

#[test]
fn test_m4a_has_valid_header() {
    let path = get_test_file_path("test_stereo.m4a");
    let bytes = std::fs::read(&path).unwrap();

    // M4A/MP4 files have "ftyp" at bytes 4-7
    assert!(
        bytes.len() >= 8 && &bytes[4..8] == b"ftyp",
        "M4A file has invalid header"
    );
}
