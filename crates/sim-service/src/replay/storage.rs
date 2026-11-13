//! Replay Storage - Manages storage and retrieval of replay data

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::diff::DiffEngine;
use super::diff::RunDiff;
use super::types::{ReplayFrame, ReplaySession};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Storage for replay data
#[derive(Debug)]
pub struct ReplayStorage {
    /// Stored sessions
    sessions: Arc<RwLock<HashMap<String, ReplaySession>>>,
    /// Stored frames by session ID
    frames: Arc<RwLock<HashMap<String, Vec<ReplayFrame>>>>,
    /// Stored diffs
    diffs: Arc<RwLock<HashMap<String, RunDiff>>>,
    /// Maximum frames per session
    max_frames_per_session: usize,
    /// Maximum storage size (bytes)
    max_storage_bytes: usize,
    /// Current storage size estimate
    current_size: Arc<RwLock<usize>>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ReplayStorage {
    /// Create new replay storage
    #[must_use]
    pub fn new() -> Self {
        Self::with_limits(100000, 1024 * 1024 * 1024) // 100K frames, 1GB
    }

    /// Create with custom limits
    #[must_use]
    pub fn with_limits(max_frames: usize, max_bytes: usize) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            frames: Arc::new(RwLock::new(HashMap::new())),
            diffs: Arc::new(RwLock::new(HashMap::new())),
            max_frames_per_session: max_frames,
            max_storage_bytes: max_bytes,
            current_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Store a replay session
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn store_session(&self, session: ReplaySession) -> Result<String, String> {
        let session_id = session.session_id.clone();

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        info!(session_id = %session_id, "Replay session stored");
        Ok(session_id)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a session
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_session(&self, session_id: &str) -> Option<ReplaySession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Store frames for a session
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn store_frames(
        &self,
        session_id: &str,
        frames: Vec<ReplayFrame>,
    ) -> Result<(), String> {
        // Check frame limit
        if frames.len() > self.max_frames_per_session {
            return Err(format!(
                "Too many frames: {} (max: {})",
                frames.len(),
                self.max_frames_per_session
            ));
        }

        // Update storage size estimate
        let size_estimate = frames.len() * std::mem::size_of::<ReplayFrame>();
        {
            let mut current = self.current_size.write().await;
            if *current + size_estimate > self.max_storage_bytes {
                return Err("Storage quota exceeded".to_string());
            }
            *current += size_estimate;
        }

        let mut frames_storage = self.frames.write().await;
        frames_storage.insert(session_id.to_string(), frames);

        debug!(session_id = %session_id, "Frames stored");
        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get frames for a session
    pub async fn get_frames(&self, session_id: &str) -> Option<Vec<ReplayFrame>> {
        let frames = self.frames.read().await;
        frames.get(session_id).cloned()
    }

    /// Get frames in a range
    pub async fn get_frame_range(
        &self,
        session_id: &str,
        start_tick: u64,
        end_tick: u64,
    ) -> Option<Vec<ReplayFrame>> {
        let frames = self.frames.read().await;
        frames.get(session_id).map(|session_frames| {
            session_frames
                .iter()
                .filter(|f| f.tick >= start_tick && f.tick <= end_tick)
                .cloned()
                .collect()
        })
    }

    /// Get a single frame
    pub async fn get_frame(&self, session_id: &str, tick: u64) -> Option<ReplayFrame> {
        let frames = self.frames.read().await;
        frames
            .get(session_id)
            .and_then(|session_frames| session_frames.iter().find(|f| f.tick == tick).cloned())
    }

    /// Delete a session and its frames
    pub async fn delete_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        let mut frames = self.frames.write().await;

        let removed = sessions.remove(session_id).is_some();
        if let Some(session_frames) = frames.remove(session_id) {
            let size_estimate = session_frames.len() * std::mem::size_of::<ReplayFrame>();
            let mut current = self.current_size.write().await;
            *current = current.saturating_sub(size_estimate);
        }

        if removed {
            info!(session_id = %session_id, "Replay session deleted");
        }

        removed
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<ReplaySession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// List sessions for a run
    pub async fn list_sessions_for_run(&self, run_id: &str) -> Vec<ReplaySession> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.run_id == run_id)
            .cloned()
            .collect()
    }

    /// Store a diff
    pub async fn store_diff(&self, diff: RunDiff) -> String {
        let key = format!("{}_{}", diff.base_run_id, diff.compare_run_id);

        let mut diffs = self.diffs.write().await;
        diffs.insert(key.clone(), diff);

        debug!(diff_key = %key, "Diff stored");
        key
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a diff
    pub async fn get_diff(&self, base_run_id: &str, compare_run_id: &str) -> Option<RunDiff> {
        let key = format!("{base_run_id}_{compare_run_id}");
        let diffs = self.diffs.read().await;
        diffs.get(&key).cloned()
    }

    /// Delete a diff
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn delete_diff(&self, base_run_id: &str, compare_run_id: &str) -> bool {
        let key = format!("{base_run_id}_{compare_run_id}");
        let mut diffs = self.diffs.write().await;
        diffs.remove(&key).is_some()
    }

    /// Compare two runs and store the diff
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn compare_and_store(
        &self,
        base_run_id: &str,
        compare_run_id: &str,
    ) -> Result<RunDiff, String> {
        // Get frames for both runs
        let base_sessions = self.list_sessions_for_run(base_run_id).await;
        let compare_sessions = self.list_sessions_for_run(compare_run_id).await;

        if base_sessions.is_empty() {
            return Err(format!("No replay found for run: {base_run_id}"));
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
        if compare_sessions.is_empty() {
            return Err(format!("No replay found for run: {compare_run_id}"));
        }

        // Use first session for each run
        let base_frames = self
            .get_frames(&base_sessions[0].session_id)
            .await
            .ok_or("No frames for base run")?;
        let compare_frames = self
            .get_frames(&compare_sessions[0].session_id)
            .await
            .ok_or("No frames for compare run")?;

        // Compare
        let engine = DiffEngine::new();
        let result =
            engine.compare_runs(base_run_id, compare_run_id, &base_frames, &compare_frames);

        let diff = RunDiff {
            base_run_id: base_run_id.to_string(),
            compare_run_id: compare_run_id.to_string(),
            compared_at: chrono::Utc::now().to_rfc3339(),
            result,
        };

        let key = self.store_diff(diff.clone()).await;
        info!(diff_key = %key, similarity = %diff.result.similarity_score, "Runs compared");

        Ok(diff)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get storage statistics
    pub async fn stats(&self) -> StorageStats {
        let sessions = self.sessions.read().await;
        let frames = self.frames.read().await;
        let diffs = self.diffs.read().await;

        let total_frames: usize = frames.values().map(std::vec::Vec::len).sum();
        let total_sessions = sessions.len();
        let total_diffs = diffs.len();

        StorageStats {
            total_sessions,
            total_frames,
            total_diffs,
            estimated_size_bytes: *self.current_size.read().await,
            max_frames_per_session: self.max_frames_per_session,
            max_storage_bytes: self.max_storage_bytes,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Clean up old sessions
    pub async fn cleanup_old_sessions(&self, max_age_hours: u64) -> usize {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours as i64);

        let sessions = self.sessions.read().await;
        let to_delete: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| {
                if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&s.created_at) {
                    created.with_timezone(&chrono::Utc) < cutoff
                } else {
                    false
                }
            })
            .map(|(id, _)| id.clone())
            .collect();
        drop(sessions);

        let del_futs = to_delete.iter().map(|id| self.delete_session(id));
        let results = futures::future::join_all(del_futs).await;
        let deleted = results.into_iter().filter(|&ok| ok).count();

        warn!(deleted = %deleted, "Old replay sessions cleaned up");
        deleted
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ReplayStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// total sessions
    pub total_sessions: usize,
    /// total frames
    pub total_frames: usize,
    /// total diffs
    pub total_diffs: usize,
    /// estimated size bytes
    pub estimated_size_bytes: usize,
    /// max frames per session
    pub max_frames_per_session: usize,
    /// max storage bytes
    pub max_storage_bytes: usize,
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
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
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

    fn create_test_session(id: &str, run_id: &str) -> ReplaySession {
        ReplaySession {
            session_id: id.to_string(),
            run_id: run_id.to_string(),
            experiment_id: "exp-1".to_string(),
            name: "Test Session".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            total_ticks: 100,
            current_tick: 0,
            config: ReplayConfig::default(),
            stats: ReplayStats::default(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_store_and_retrieve() {
        let storage = ReplayStorage::new();

        let session = create_test_session("session-1", "run-1");
        storage.store_session(session.clone()).await.unwrap();

        let frames: Vec<_> = (0..10).map(create_test_frame).collect();
        storage
            .store_frames("session-1", frames.clone())
            .await
            .unwrap();

        let retrieved = storage.get_session("session-1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, "session-1");

        let retrieved_frames = storage.get_frames("session-1").await;
        assert_eq!(retrieved_frames.unwrap().len(), 10);
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let storage = ReplayStorage::new();

        storage
            .store_session(create_test_session("s1", "run-1"))
            .await
            .unwrap();
        storage
            .store_session(create_test_session("s2", "run-1"))
            .await
            .unwrap();
        storage
            .store_session(create_test_session("s3", "run-2"))
            .await
            .unwrap();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let all = storage.list_sessions().await;
        assert_eq!(all.len(), 3);

        let for_run1 = storage.list_sessions_for_run("run-1").await;
        assert_eq!(for_run1.len(), 2);
    }

    #[tokio::test]
    async fn test_frame_range() {
        let storage = ReplayStorage::new();

        storage
            .store_session(create_test_session("s1", "run-1"))
            .await
            .unwrap();

        let frames: Vec<_> = (0..100).map(create_test_frame).collect();
        storage.store_frames("s1", frames).await.unwrap();

        let range = storage.get_frame_range("s1", 10, 20).await.unwrap();
        assert_eq!(range.len(), 11);
        assert_eq!(range[0].tick, 10);
        assert_eq!(range[10].tick, 20);
    }
}
