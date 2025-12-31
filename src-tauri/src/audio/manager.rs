//! Audio playback lifecycle management
//!
//! Manages active playbacks with thread-safe stop signaling and audio caching.

use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use super::cache::{AudioCache, CacheStats};

/// State of an active sound playback
#[derive(Clone, Debug)]
pub enum SoundState {
    /// Sound is being decoded (not audible yet)
    Decoding { playback_id: String },
    /// Sound is actively playing (audible)
    Playing {
        playback_id: String,
        started_at: std::time::Instant,
    },
}

impl SoundState {
    pub fn playback_id(&self) -> &str {
        match self {
            SoundState::Decoding { playback_id } => playback_id,
            SoundState::Playing { playback_id, .. } => playback_id,
        }
    }
}

/// Manages audio playback state, active streams, and audio cache
pub struct AudioManager {
    /// Stop signals for active playbacks (send () to stop)
    stop_senders: Arc<Mutex<HashMap<String, Sender<()>>>>,
    /// Counter for generating unique playback IDs
    playback_counter: Arc<Mutex<u64>>,
    /// LRU cache for decoded audio data
    cache: Arc<Mutex<AudioCache>>,
    /// Active sound_id -> SoundState mapping for policy enforcement
    active_sounds: Arc<Mutex<HashMap<String, SoundState>>>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            stop_senders: Arc::new(Mutex::new(HashMap::new())),
            playback_counter: Arc::new(Mutex::new(0)),
            cache: Arc::new(Mutex::new(AudioCache::default())),
            active_sounds: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create with custom cache size (in MB)
    pub fn with_cache_size(max_memory_mb: usize) -> Self {
        Self {
            stop_senders: Arc::new(Mutex::new(HashMap::new())),
            playback_counter: Arc::new(Mutex::new(0)),
            cache: Arc::new(Mutex::new(AudioCache::new(max_memory_mb))),
            active_sounds: Arc::new(Mutex::new(HashMap::new())),
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

    /// Returns the current state of a sound for playback policy enforcement.
    ///
    /// Used to determine if a sound is currently decoding or playing,
    /// which affects how new play requests for the same sound are handled.
    pub fn get_sound_state(&self, sound_id: &str) -> Option<SoundState> {
        self.active_sounds.lock().unwrap().get(sound_id).cloned()
    }

    /// Registers a sound as currently decoding (not yet audible).
    ///
    /// Called at the start of playback before audio streams are created.
    /// The state transitions to `Playing` once streams are ready.
    pub fn register_sound_decoding(&self, sound_id: String, playback_id: String) {
        self.active_sounds
            .lock()
            .unwrap()
            .insert(sound_id, SoundState::Decoding { playback_id });
    }

    /// Returns a thread-safe reference to the active sounds map.
    ///
    /// Used by playback threads to update sound state (Decoding -> Playing)
    /// and clean up when playback completes.
    pub fn get_active_sounds(&self) -> Arc<Mutex<HashMap<String, SoundState>>> {
        self.active_sounds.clone()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_audio_manager_new() {
        let manager = AudioManager::new();
        assert_eq!(manager.stop_senders.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_audio_manager_with_cache_size() {
        let manager = AudioManager::with_cache_size(100);
        let stats = manager.cache_stats();
        assert_eq!(stats.max_memory_mb, 100);
    }

    #[test]
    fn test_playback_id_generation() {
        let manager = AudioManager::new();

        let id1 = manager.next_playback_id();
        let id2 = manager.next_playback_id();
        let id3 = manager.next_playback_id();

        assert_eq!(id1, "playback_1");
        assert_eq!(id2, "playback_2");
        assert_eq!(id3, "playback_3");
    }

    #[test]
    fn test_playback_id_sequential() {
        let manager = AudioManager::new();

        for i in 1..=100 {
            let id = manager.next_playback_id();
            assert_eq!(id, format!("playback_{}", i));
        }
    }

    #[test]
    fn test_register_and_signal_stop() {
        let manager = AudioManager::new();
        let (tx, rx) = mpsc::channel::<()>();

        manager.register_playback("playback_1".to_string(), tx);

        // Signal stop should return true and send to channel
        assert!(manager.signal_stop("playback_1"));
        assert!(rx.try_recv().is_ok());

        // Signal again should return false (already removed)
        assert!(!manager.signal_stop("playback_1"));
    }

    #[test]
    fn test_signal_stop_nonexistent() {
        let manager = AudioManager::new();
        assert!(!manager.signal_stop("nonexistent"));
    }

    #[test]
    fn test_stop_all() {
        let manager = AudioManager::new();
        let (tx1, rx1) = mpsc::channel::<()>();
        let (tx2, rx2) = mpsc::channel::<()>();

        manager.register_playback("playback_1".to_string(), tx1);
        manager.register_playback("playback_2".to_string(), tx2);

        manager.stop_all();

        // Both channels should receive stop signal
        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());

        // Senders should be cleared
        assert_eq!(manager.stop_senders.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_sound_state_decoding() {
        let manager = AudioManager::new();

        manager.register_sound_decoding("sound_1".to_string(), "playback_1".to_string());

        let state = manager.get_sound_state("sound_1");
        assert!(state.is_some());

        if let Some(SoundState::Decoding { playback_id }) = state {
            assert_eq!(playback_id, "playback_1");
        } else {
            panic!("Expected Decoding state");
        }
    }

    #[test]
    fn test_sound_state_playing() {
        let manager = AudioManager::new();

        // Simulate transition to Playing state
        let active_sounds = manager.get_active_sounds();
        active_sounds.lock().unwrap().insert(
            "sound_1".to_string(),
            SoundState::Playing {
                playback_id: "playback_1".to_string(),
                started_at: std::time::Instant::now(),
            },
        );

        let state = manager.get_sound_state("sound_1");
        assert!(state.is_some());

        if let Some(SoundState::Playing { playback_id, .. }) = state {
            assert_eq!(playback_id, "playback_1");
        } else {
            panic!("Expected Playing state");
        }
    }

    #[test]
    fn test_sound_state_nonexistent() {
        let manager = AudioManager::new();
        assert!(manager.get_sound_state("nonexistent").is_none());
    }

    #[test]
    fn test_sound_state_playback_id() {
        let decoding = SoundState::Decoding {
            playback_id: "pb_1".to_string(),
        };
        assert_eq!(decoding.playback_id(), "pb_1");

        let playing = SoundState::Playing {
            playback_id: "pb_2".to_string(),
            started_at: std::time::Instant::now(),
        };
        assert_eq!(playing.playback_id(), "pb_2");
    }

    #[test]
    fn test_cache_clear() {
        let manager = AudioManager::new();
        manager.clear_cache();
        let stats = manager.cache_stats();
        assert_eq!(stats.entries, 0);
    }
}
