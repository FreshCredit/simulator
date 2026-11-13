//! Replay Player - Controls playback of simulation replays

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::Instant;
use tracing::{debug, info, trace, warn};

use super::types::{PlaybackState, ReplayBookmark, ReplayFrame, ReplaySession};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Controls replay playback
#[derive(Debug)]
pub struct ReplayPlayer {
    /// Session being played
    session: Arc<RwLock<ReplaySession>>,
    /// Current playback state
    state: Arc<RwLock<PlaybackState>>,
    /// Frame storage
    frames: Arc<RwLock<HashMap<u64, ReplayFrame>>>,
    /// Current frame sender
    frame_tx: mpsc::Sender<ReplayFrame>,
    /// Playback speed
    speed: Arc<RwLock<f64>>,
    /// Bookmarks
    bookmarks: Arc<RwLock<Vec<ReplayBookmark>>>,
    /// Playback task handle
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ReplayPlayer {
    /// Create a new replay player
    #[must_use]
    pub fn new(session: ReplaySession, frame_tx: mpsc::Sender<ReplayFrame>) -> Self {
        Self {
            session: Arc::new(RwLock::new(session)),
            state: Arc::new(RwLock::new(PlaybackState::Stopped)),
            frames: Arc::new(RwLock::new(HashMap::new())),
            frame_tx,
            speed: Arc::new(RwLock::new(1.0)),
            bookmarks: Arc::new(RwLock::new(Vec::new())),
            task_handle: None,
        }
    }

    /// Load frames into the player
    pub async fn load_frames(&self, frames: Vec<ReplayFrame>) {
        let mut storage = self.frames.write().await;
        for frame in frames {
            storage.insert(frame.tick, frame);
        }

        let mut session = self.session.write().await;
        session.stats.total_frames = storage.len() as u64;

        info!(frame_count = %storage.len(), "Frames loaded into replay player");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Start playback
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn play(&mut self) -> Result<(), String> {
        let mut state = self.state.write().await;
        if *state == PlaybackState::Playing {
            return Ok(());
        }

        *state = PlaybackState::Playing;
        drop(state);

        // Start playback task
        let session = self.session.clone();
        let playback_state = self.state.clone();
        let frames = self.frames.clone();
        let frame_tx = self.frame_tx.clone();
        let speed = self.speed.clone();

        let handle = tokio::spawn(async move {
            let mut last_frame_time = Instant::now();
            let base_interval = Duration::from_millis(100); // 10 FPS base

            loop {
                // Check if still playing
                let state = playback_state.read().await;
                if *state != PlaybackState::Playing {
                    drop(state);
                    break;
                }
                drop(state);

                // Get current speed
                let speed_val = *speed.read().await;
                let frame_delay = base_interval.div_f64(speed_val);
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001

                // Wait for next frame time
                tokio::time::sleep_until(last_frame_time + frame_delay).await;
                last_frame_time = Instant::now();

                // Get and send next frame
                let mut session_guard = session.write().await;
                let current_tick = session_guard.current_tick;

                // Check if reached end
                if current_tick >= session_guard.total_ticks {
                    let mut state_guard = playback_state.write().await;
                    *state_guard = PlaybackState::Completed;
                    break;
                }

                // Send frame if available
                let frames_guard = frames.read().await;
                if let Some(frame) = frames_guard.get(&current_tick) {
                    if frame_tx.send(frame.clone()).await.is_err() {
                        warn!("Frame receiver dropped, stopping playback");
                        break;
                    }
                }
                drop(frames_guard);

                // Advance tick
                session_guard.current_tick += 1;
                trace!(tick = %session_guard.current_tick, "Advanced to next tick");
            }
        });

        self.task_handle = Some(handle);
        info!("Replay playback started");
        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Pause playback
    pub async fn pause(&self) {
        let mut state = self.state.write().await;
        *state = PlaybackState::Paused;
        info!("Replay playback paused");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Stop playback
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn stop(&self) {
        let mut state = self.state.write().await;
        *state = PlaybackState::Stopped;

        let mut session = self.session.write().await;
        session.current_tick = session.config.start_tick;

        info!("Replay playback stopped");
    }

    /// Seek to specific tick
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn seek(&self, tick: u64) -> Result<(), String> {
        let session = self.session.read().await;
        if tick >= session.total_ticks {
            return Err(format!(
                "Tick {} is beyond replay length {}",
                tick, session.total_ticks
            ));
        }
        drop(session);

        let mut session = self.session.write().await;
        session.current_tick = tick;

        // If paused, send the frame immediately
        let state = self.state.read().await;
        if *state == PlaybackState::Paused {
            drop(state);
            let frames = self.frames.read().await;
            if let Some(frame) = frames.get(&tick) {
                let _ = self.frame_tx.send(frame.clone()).await;
            }
        }

        debug!(tick = %tick, "Seeked to tick");
        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Step forward one tick
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn step_forward(&self) -> Result<(), String> {
        let session = self.session.read().await;
        let next_tick = session.current_tick + 1;
        drop(session);

        self.seek(next_tick).await
    }

    /// Step backward one tick
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn step_backward(&self) -> Result<(), String> {
        let session = self.session.read().await;
        if session.current_tick == 0 {
            return Err("Already at tick 0".to_string());
        }
        let prev_tick = session.current_tick - 1;
        drop(session);

        self.seek(prev_tick).await
    }

    /// Set playback speed
    pub async fn set_speed(&self, speed: f64) {
        let mut s = self.speed.write().await;
        *s = speed.clamp(0.1, 10.0);
        debug!(speed = %*s, "Playback speed updated");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get current playback speed
    pub async fn get_speed(&self) -> f64 {
        *self.speed.read().await
    }

    /// Get current state
    pub async fn get_state(&self) -> PlaybackState {
        *self.state.read().await
    }

    /// Get current tick
    pub async fn get_current_tick(&self) -> u64 {
        self.session.read().await.current_tick
    }

    /// Get total ticks
    pub async fn get_total_ticks(&self) -> u64 {
        self.session.read().await.total_ticks
    }

    /// Add bookmark
    pub async fn add_bookmark(&self, tick: u64, label: impl Into<String>) -> String {
        let bookmark = ReplayBookmark {
            id: uuid::Uuid::new_v4().to_string(),
            tick,
            label: label.into(),
            color: "#FFD700".to_string(),
        };

        let mut bookmarks = self.bookmarks.write().await;
        bookmarks.push(bookmark.clone());
        bookmarks.sort_by_key(|b| b.tick);

        debug!(bookmark_id = %bookmark.id, tick = %tick, "Bookmark added");
        bookmark.id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get all bookmarks
    pub async fn get_bookmarks(&self) -> Vec<ReplayBookmark> {
        self.bookmarks.read().await.clone()
    }

    /// Remove bookmark
    pub async fn remove_bookmark(&self, bookmark_id: &str) -> bool {
        let mut bookmarks = self.bookmarks.write().await;
        let initial_len = bookmarks.len();
        bookmarks.retain(|b| b.id != bookmark_id);
        bookmarks.len() < initial_len
    }

    /// Jump to next bookmark
    pub async fn jump_to_next_bookmark(&self) -> Option<u64> {
        let current_tick = self.get_current_tick().await;
        let bookmarks = self.bookmarks.read().await;

        bookmarks
            .iter()
            .find(|b| b.tick > current_tick)
            .map(|b| b.tick)
    }

    /// Jump to previous bookmark
    pub async fn jump_to_prev_bookmark(&self) -> Option<u64> {
        let current_tick = self.get_current_tick().await;
        let bookmarks = self.bookmarks.read().await;

        bookmarks
            .iter()
            .rev()
            .find(|b| b.tick < current_tick)
            .map(|b| b.tick)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get session info
    pub async fn get_session(&self) -> ReplaySession {
        self.session.read().await.clone()
    }

    /// Get frame at specific tick
    pub async fn get_frame(&self, tick: u64) -> Option<ReplayFrame> {
        self.frames.read().await.get(&tick).cloned()
    }

    /// Get frames in range
    pub async fn get_frame_range(&self, start_tick: u64, end_tick: u64) -> Vec<ReplayFrame> {
        let frames = self.frames.read().await;
        (start_tick..=end_tick)
            .filter_map(|tick| frames.get(&tick).cloned())
            .collect()
    }
}

/// Player manager for multiple replay sessions
#[derive(Debug)]
pub struct PlayerManager {
    players: Arc<RwLock<HashMap<String, ReplayPlayer>>>,
}

impl PlayerManager {
    /// Create new player manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            players: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new player
    pub async fn create_player(
        &self,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        session_id: impl Into<String>,
        session: ReplaySession,
    ) -> mpsc::Receiver<ReplayFrame> {
        let (tx, rx) = mpsc::channel(100);
        let player = ReplayPlayer::new(session, tx);

        let mut players = self.players.write().await;
        players.insert(session_id.into(), player);

        rx
    }

    /// Check if player exists
    pub async fn has_player(&self, session_id: &str) -> bool {
        self.players.read().await.contains_key(session_id)
    }

    /// Remove player
    pub async fn remove_player(&self, session_id: &str) {
        self.players.write().await.remove(session_id);
    }

    /// List active players
    pub async fn list_players(&self) -> Vec<String> {
        self.players.read().await.keys().cloned().collect()
    }
}

impl Default for PlayerManager {
    fn default() -> Self {
        Self::new()
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::types::{AgentState, EnvironmentState, ReplayConfig, ReplayStats};

    fn create_test_frame(tick: u64) -> ReplayFrame {
        ReplayFrame {
            tick,
            timestamp: chrono::Utc::now().to_rfc3339(),
            agents: vec![AgentState {
                agent_id: "agent-1".to_string(),
                position: (tick as f64, tick as f64),
                velocity: (1.0, 1.0),
                orientation: 0.0,
                state: "moving".to_string(),
                energy: 100.0,
                health: 100.0,
                properties: HashMap::new(),
            }],
            environment: EnvironmentState {
                time_of_day: 12.0,
                weather: "clear".to_string(),
                temperature: 20.0,
                resources: vec![],
                regions: HashMap::new(),
            },
            events: vec![],
            metrics: HashMap::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_player_seek() {
        let (tx, _rx) = mpsc::channel(10);
        let session = ReplaySession {
            session_id: "test-1".to_string(),
            run_id: "run-1".to_string(),
            experiment_id: "exp-1".to_string(),
            name: "Test".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            total_ticks: 100,
            current_tick: 0,
            config: ReplayConfig::default(),
            stats: ReplayStats::default(),
        };

        let player = ReplayPlayer::new(session, tx);

        // Load frames
        let frames: Vec<_> = (0..10).map(create_test_frame).collect();
        player.load_frames(frames).await;

        // Seek to tick 5
        player.seek(5).await.unwrap();
        assert_eq!(player.get_current_tick().await, 5);
    }

    #[tokio::test]
    async fn test_bookmarks() {
        let (tx, _rx) = mpsc::channel(10);
        let session = ReplaySession {
            session_id: "test-2".to_string(),
            run_id: "run-1".to_string(),
            experiment_id: "exp-1".to_string(),
            name: "Test".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            total_ticks: 100,
            current_tick: 0,
            config: ReplayConfig::default(),
            stats: ReplayStats::default(),
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let player = ReplayPlayer::new(session, tx);

        // Add bookmarks
        let _id1 = player.add_bookmark(10, "Event A").await;
        let id2 = player.add_bookmark(20, "Event B").await;
        let _id3 = player.add_bookmark(30, "Event C").await;

        // Test next bookmark
        assert_eq!(player.jump_to_next_bookmark().await, Some(10));

        // Seek and test again
        player.seek(15).await.unwrap();
        assert_eq!(player.jump_to_next_bookmark().await, Some(20));

        // Test prev bookmark
        assert_eq!(player.jump_to_prev_bookmark().await, Some(10));

        // Remove bookmark
        assert!(player.remove_bookmark(&id2).await);

        let bookmarks = player.get_bookmarks().await;
        assert_eq!(bookmarks.len(), 2);
    }
}
