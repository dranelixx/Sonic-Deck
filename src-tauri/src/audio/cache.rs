//! Audio caching system
//!
//! LRU memory cache for decoded audio data to avoid redundant decoding.

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::SystemTime;

use lru::LruCache;

use super::{AudioData, AudioError};
use super::decode::decode_audio_file;

/// Estimated bytes per sample (f32 = 4 bytes)
const BYTES_PER_SAMPLE: usize = 4;

/// Default max cache size in bytes (500 MB)
const DEFAULT_MAX_CACHE_BYTES: usize = 500 * 1024 * 1024;

/// Cache entry with metadata for validation
struct CacheEntry {
    /// The cached audio data
    audio_data: Arc<AudioData>,
    /// File modification time when cached
    file_modified: Option<SystemTime>,
    /// Size in bytes (for memory tracking)
    size_bytes: usize,
}

/// LRU cache for decoded audio data
pub struct AudioCache {
    /// LRU cache mapping file paths to decoded audio
    cache: LruCache<String, CacheEntry>,
    /// Current total memory usage in bytes
    current_bytes: usize,
    /// Maximum allowed memory in bytes
    max_bytes: usize,
    /// Track file modification times for invalidation
    file_times: HashMap<String, SystemTime>,
}

impl AudioCache {
    /// Create a new audio cache with specified max memory in MB
    pub fn new(max_memory_mb: usize) -> Self {
        // Use NonZeroUsize for LRU cache capacity (number of entries, not bytes)
        // We'll track bytes ourselves and evict when needed
        let capacity = NonZeroUsize::new(1000).unwrap(); // Max 1000 entries

        Self {
            cache: LruCache::new(capacity),
            current_bytes: 0,
            max_bytes: if max_memory_mb > 0 {
                max_memory_mb * 1024 * 1024
            } else {
                DEFAULT_MAX_CACHE_BYTES
            },
            file_times: HashMap::new(),
        }
    }

    /// Estimate memory size of audio data
    fn estimate_size(audio_data: &AudioData) -> usize {
        audio_data.samples.len() * BYTES_PER_SAMPLE
    }

    /// Get file modification time
    fn get_file_modified(file_path: &str) -> Option<SystemTime> {
        std::fs::metadata(file_path)
            .and_then(|m| m.modified())
            .ok()
    }

    /// Check if cached entry is still valid (file hasn't changed)
    fn is_cache_valid(entry: &CacheEntry, file_path: &str) -> bool {
        match (entry.file_modified, Self::get_file_modified(file_path)) {
            (Some(cached_time), Some(current_time)) => cached_time == current_time,
            // If we can't check, assume invalid (re-decode)
            _ => false,
        }
    }

    /// Evict entries until we have enough space for new entry
    fn make_space(&mut self, needed_bytes: usize) {
        while self.current_bytes + needed_bytes > self.max_bytes {
            // LRU pop removes least recently used entry
            if let Some((path, entry)) = self.cache.pop_lru() {
                self.current_bytes = self.current_bytes.saturating_sub(entry.size_bytes);
                self.file_times.remove(&path);
            } else {
                // Cache is empty, nothing more to evict
                break;
            }
        }
    }

    /// Get cached audio or decode and cache
    pub fn get_or_decode(&mut self, file_path: &str) -> Result<Arc<AudioData>, AudioError> {
        // Check if we have a valid cached version
        if let Some(entry) = self.cache.get(file_path) {
            if Self::is_cache_valid(entry, file_path) {
                // Cache hit - return the cached data
                return Ok(entry.audio_data.clone());
            } else {
                // Cache invalid - remove it (will be replaced below)
                if let Some(removed) = self.cache.pop(file_path) {
                    self.current_bytes = self.current_bytes.saturating_sub(removed.size_bytes);
                    self.file_times.remove(file_path);
                }
            }
        }

        // Cache miss - decode the file
        let audio_data = decode_audio_file(file_path)?;
        let audio_data = Arc::new(audio_data);

        // Calculate size and make space if needed
        let size_bytes = Self::estimate_size(&audio_data);
        self.make_space(size_bytes);

        // Get file modification time for future validation
        let file_modified = Self::get_file_modified(file_path);

        // Create cache entry
        let entry = CacheEntry {
            audio_data: audio_data.clone(),
            file_modified,
            size_bytes,
        };

        // Store in cache
        self.cache.put(file_path.to_string(), entry);
        self.current_bytes += size_bytes;

        if let Some(time) = file_modified {
            self.file_times.insert(file_path.to_string(), time);
        }

        Ok(audio_data)
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.file_times.clear();
        self.current_bytes = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            memory_bytes: self.current_bytes,
            max_memory_bytes: self.max_bytes,
            memory_mb: self.current_bytes / (1024 * 1024),
            max_memory_mb: self.max_bytes / (1024 * 1024),
        }
    }

    /// Invalidate a specific file from cache
    pub fn invalidate(&mut self, file_path: &str) {
        if let Some(entry) = self.cache.pop(file_path) {
            self.current_bytes = self.current_bytes.saturating_sub(entry.size_bytes);
            self.file_times.remove(file_path);
        }
    }
}

impl Default for AudioCache {
    fn default() -> Self {
        Self::new(500) // 500 MB default
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    /// Number of cached entries
    pub entries: usize,
    /// Current memory usage in bytes
    pub memory_bytes: usize,
    /// Maximum memory limit in bytes
    pub max_memory_bytes: usize,
    /// Current memory usage in MB
    pub memory_mb: usize,
    /// Maximum memory limit in MB
    pub max_memory_mb: usize,
}
