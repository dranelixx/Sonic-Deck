//! Audio playback lifecycle management
//!
//! Manages active playbacks with thread-safe stop signaling and audio caching.

use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use super::cache::{AudioCache, CacheStats};

/// Manages audio playback state, active streams, and audio cache
pub struct AudioManager {
    /// Stop signals for active playbacks (send () to stop)
    stop_senders: Arc<Mutex<HashMap<String, Sender<()>>>>,
    /// Counter for generating unique playback IDs
    playback_counter: Arc<Mutex<u64>>,
    /// LRU cache for decoded audio data
    cache: Arc<Mutex<AudioCache>>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            stop_senders: Arc::new(Mutex::new(HashMap::new())),
            playback_counter: Arc::new(Mutex::new(0)),
            cache: Arc::new(Mutex::new(AudioCache::default())),
        }
    }

    /// Create with custom cache size (in MB)
    pub fn with_cache_size(max_memory_mb: usize) -> Self {
        Self {
            stop_senders: Arc::new(Mutex::new(HashMap::new())),
            playback_counter: Arc::new(Mutex::new(0)),
            cache: Arc::new(Mutex::new(AudioCache::new(max_memory_mb))),
        }
    }

    /// Get a clone of the cache Arc for thread-safe access
    pub fn get_cache(&self) -> Arc<Mutex<AudioCache>> {
        self.cache.clone()
    }

    /// Clear the audio cache
    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.lock().unwrap().stats()
    }

    /// Generate a unique playback ID
    pub fn next_playback_id(&self) -> String {
        let mut counter = self.playback_counter.lock().unwrap();
        *counter += 1;
        format!("playback_{}", *counter)
    }

    /// Register a stop sender for a playback
    pub fn register_playback(&self, playback_id: String, sender: Sender<()>) {
        let mut senders = self.stop_senders.lock().unwrap();
        senders.insert(playback_id, sender);
    }

    /// Unregister a playback (called when playback completes)
    #[allow(dead_code)]
    pub fn unregister_playback(&self, playback_id: &str) {
        let mut senders = self.stop_senders.lock().unwrap();
        senders.remove(playback_id);
    }

    /// Stop all active playbacks
    pub fn stop_all(&self) {
        let mut senders = self.stop_senders.lock().unwrap();
        for (_, sender) in senders.drain() {
            let _ = sender.send(()); // Ignore errors if thread already stopped
        }
    }

    /// Signal a specific playback to stop
    pub fn signal_stop(&self, playback_id: &str) -> bool {
        let mut senders = self.stop_senders.lock().unwrap();
        if let Some(sender) = senders.remove(playback_id) {
            let _ = sender.send(());
            true
        } else {
            false
        }
    }

    /// Get a clone of the stop_senders Arc for use in spawned threads
    pub fn get_stop_senders(&self) -> Arc<Mutex<HashMap<String, Sender<()>>>> {
        self.stop_senders.clone()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}
