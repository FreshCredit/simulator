//! Replay & Diff API (P1-SIM-6)
//!
//! Provides REST endpoints for simulation replay and run comparison.
//!
//! ## Routes
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/replay` | Create replay session |
//! | GET | `/api/replay/:session_id` | Get session info |
//! | GET | `/api/replay/:session_id/frames` | Get frames |
//! | POST | `/api/replay/:session_id/play` | Start playback |
//! | POST | `/api/replay/:session_id/pause` | Pause playback |
//! | POST | `/api/replay/:session_id/seek` | Seek to tick |
//! | POST | `/api/replay/:session_id/bookmark` | Add bookmark |
//! | GET | `/api/replay/:session_id/stream` | SSE frame stream |
//! | POST | `/api/diff` | Compare two runs |
//! | GET | `/api/diff/:base_id/:compare_id` | Get comparison |
//! | GET | `/api/replay/runs/:run_id/sessions` | List run sessions |

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::sse::Event,
    response::sse::Sse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::replay::player::PlayerManager;
use crate::replay::storage::ReplayStorage;
use crate::replay::types::{
    CompareRunsRequest, ComparisonResult, ReplayBookmark, ReplayConfig, ReplayFrame, ReplaySession,
    ReplayStats,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// State for replay endpoints
#[derive(Clone, Debug)]
pub struct ReplayState {
    /// storage
    pub storage: Arc<ReplayStorage>,
    /// player manager
    pub player_manager: Arc<PlayerManager>,
}

/// Create replay request
#[derive(Debug, Deserialize)]
pub struct CreateReplayRequest {
    /// run id
    pub run_id: String,
    /// experiment id
    pub experiment_id: String,
    /// name
    pub name: Option<String>,
    /// config
    pub config: Option<ReplayConfig>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create replay response
#[derive(Debug, Serialize)]
pub struct CreateReplayResponse {
    /// session id
    pub session_id: String,
    /// run id
    pub run_id: String,
    /// total ticks
    pub total_ticks: u64,
}

/// Playback control request
#[derive(Debug, Deserialize)]
pub struct PlaybackControlRequest {
    /// speed
    pub speed: Option<f64>,
}

/// Seek request
#[derive(Debug, Deserialize)]
pub struct SeekRequest {
    /// tick
    pub tick: u64,
}

/// Frame query params
#[derive(Debug, Deserialize)]
pub struct FrameQuery {
    /// start tick
    pub start_tick: Option<u64>,
    /// end tick
    pub end_tick: Option<u64>,
}

/// Playback status response
#[derive(Debug, Serialize)]
pub struct PlaybackStatusResponse {
    /// session id
    pub session_id: String,
    /// state
    pub state: String,
    /// current tick
    pub current_tick: u64,
    /// total ticks
    pub total_ticks: u64,
    /// speed
    pub speed: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Bookmark request
#[derive(Debug, Deserialize)]
pub struct BookmarkRequest {
    /// tick
    pub tick: u64,
    /// label
    pub label: String,
}

/// Bookmark response
#[derive(Debug, Serialize)]
pub struct BookmarkResponse {
    /// bookmark id
    pub bookmark_id: String,
    /// tick
    pub tick: u64,
    /// label
    pub label: String,
}

/// Diff response
#[derive(Debug, Serialize)]
pub struct DiffResponse {
    /// base run id
    pub base_run_id: String,
    /// compare run id
    pub compare_run_id: String,
    /// similarity score
    pub similarity_score: f64,
    /// comparison
    pub comparison: ComparisonResult,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /api/replay - Create replay session
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn create_replay(
    State(state): State<ReplayState>,
    Json(request): Json<CreateReplayRequest>,
) -> Result<(StatusCode, Json<CreateReplayResponse>), StatusCode> {
    info!(run_id = %request.run_id, "Creating replay session");

    // Initialize active replay session tracking from run state
    let session = ReplaySession {
        session_id: uuid::Uuid::new_v4().to_string(),
        run_id: request.run_id.clone(),
        experiment_id: request.experiment_id,
        name: request.name.unwrap_or_else(|| "Untitled".to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
        total_ticks: 0,
        current_tick: 0,
        config: request.config.unwrap_or_default(),
        stats: ReplayStats::default(),
    };

    let total_ticks = session.total_ticks;
    let session_id = session.session_id.clone();

    state
        .storage
        .store_session(session)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(CreateReplayResponse {
            session_id,
            run_id: request.run_id,
            total_ticks,
        }),
    ))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`api/replay/:session_id` - Get session info
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_session(
    State(state): State<ReplayState>,
    Path(session_id): Path<String>,
) -> Result<Json<ReplaySession>, StatusCode> {
    match state.storage.get_session(&session_id).await {
        Some(session) => Ok(Json(session)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// GET /`api/replay/:session_id/frames` - Get frames
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_frames(
    State(state): State<ReplayState>,
    Path(session_id): Path<String>,
    Query(query): Query<FrameQuery>,
) -> Result<Json<Vec<ReplayFrame>>, StatusCode> {
    let frames = if let (Some(start), Some(end)) = (query.start_tick, query.end_tick) {
        state.storage.get_frame_range(&session_id, start, end).await
    } else {
        state.storage.get_frames(&session_id).await
    };

    match frames {
        Some(f) => Ok(Json(f)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /`api/replay/:session_id/play` - Start playback
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn play_replay(
    State(state): State<ReplayState>,
    Path(session_id): Path<String>,
    Json(request): Json<PlaybackControlRequest>,
) -> Result<Json<PlaybackStatusResponse>, StatusCode> {
    // Get or create player
    let session = state
        .storage
        .get_session(&session_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    // Note: Playback would be managed by the player manager
    // For now, simplified implementation

    Ok(Json(PlaybackStatusResponse {
        session_id,
        state: "playing".to_string(),
        current_tick: session.current_tick,
        total_ticks: session.total_ticks,
        speed: request.speed.unwrap_or(1.0),
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }))
}

/// POST /`api/replay/:session_id/pause` - Pause playback
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn pause_replay(
    State(state): State<ReplayState>,
    Path(session_id): Path<String>,
) -> Result<Json<PlaybackStatusResponse>, StatusCode> {
    let session = state
        .storage
        .get_session(&session_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(PlaybackStatusResponse {
        session_id,
        state: "paused".to_string(),
        current_tick: session.current_tick,
        total_ticks: session.total_ticks,
        speed: 1.0,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /`api/replay/:session_id/seek` - Seek to tick
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn seek_replay(
    State(state): State<ReplayState>,
    Path(session_id): Path<String>,
    Json(request): Json<SeekRequest>,
) -> Result<Json<PlaybackStatusResponse>, StatusCode> {
    let mut session = state
        .storage
        .get_session(&session_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    if request.tick >= session.total_ticks {
        return Err(StatusCode::BAD_REQUEST);
    }

    session.current_tick = request.tick;

    Ok(Json(PlaybackStatusResponse {
        session_id,
        state: "paused".to_string(),
        current_tick: request.tick,
        total_ticks: session.total_ticks,
        speed: 1.0,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /`api/replay/:session_id/bookmark` - Add bookmark
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn add_bookmark(
    State(_state): State<ReplayState>,
    Path(_session_id): Path<String>,
    Json(request): Json<BookmarkRequest>,
) -> Result<Json<BookmarkResponse>, StatusCode> {
    // Simplified - would store bookmark in storage
    Ok(Json(BookmarkResponse {
        bookmark_id: uuid::Uuid::new_v4().to_string(),
        tick: request.tick,
        label: request.label,
    }))
}

/// GET /`api/replay/:session_id/bookmarks` - Get bookmarks
pub async fn get_bookmarks(
    State(_state): State<ReplayState>,
    Path(_session_id): Path<String>,
) -> Json<Vec<ReplayBookmark>> {
    // Simplified - would load from storage
    Json(vec![])
}

/// DELETE /`api/replay/:session_id/bookmarks/:bookmark_id` - Remove bookmark
pub async fn remove_bookmark(
    State(_state): State<ReplayState>,
    Path((_session_id, _bookmark_id)): Path<(String, String)>,
) -> StatusCode {
    // Simplified - would remove from storage
    StatusCode::NO_CONTENT
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`api/replay/:session_id/stream` - SSE frame stream
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn stream_replay(
    State(state): State<ReplayState>,
    Path(session_id): Path<String>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    use futures::stream;

    let stream = stream::unfold(
        (state, session_id, 0u64),
        |(state, session_id, last_tick)| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Get frames directly from storage
            if let Some(frames) = state
                .storage
                .get_frame_range(&session_id, last_tick, last_tick + 1)
                .await
            {
                if let Some(frame) = frames.first() {
                    if frame.tick != last_tick {
                        let event = Event::default()
                            .event("frame")
                            .data(serde_json::to_string(&frame).unwrap_or_default());
                        return Some((Ok(event), (state, session_id, frame.tick)));
                    }
                }
            }

            // Send heartbeat
            let event = Event::default().event("heartbeat").data("{}");
            Some((Ok(event), (state, session_id, last_tick)))
        },
    );

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /api/diff - Compare two runs
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn compare_runs(
    State(state): State<ReplayState>,
    Json(request): Json<CompareRunsRequest>,
) -> Result<Json<DiffResponse>, StatusCode> {
    info!(
        base = %request.base_run_id,
        compare = %request.compare_run_id,
        "Comparing runs"
    );

    match state
        .storage
        .compare_and_store(&request.base_run_id, &request.compare_run_id)
        .await
    {
        Ok(diff) => Ok(Json(DiffResponse {
            base_run_id: diff.base_run_id,
            compare_run_id: diff.compare_run_id,
            similarity_score: diff.result.similarity_score,
            comparison: diff.result,
        })),
        Err(e) => {
            tracing::error!("Comparison failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`api/diff/:base_id/:compare_id` - Get comparison
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_diff(
    State(state): State<ReplayState>,
    Path((base_id, compare_id)): Path<(String, String)>,
) -> Result<Json<DiffResponse>, StatusCode> {
    match state.storage.get_diff(&base_id, &compare_id).await {
        Some(diff) => Ok(Json(DiffResponse {
            base_run_id: diff.base_run_id,
            compare_run_id: diff.compare_run_id,
            similarity_score: diff.result.similarity_score,
            comparison: diff.result,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// GET /`api/replay/runs/:run_id/sessions` - List sessions for run
pub async fn list_run_sessions(
    State(state): State<ReplayState>,
    Path(run_id): Path<String>,
) -> Json<Vec<ReplaySession>> {
    let sessions = state.storage.list_sessions_for_run(&run_id).await;
    Json(sessions)
}

/// DELETE /`api/replay/:session_id` - Delete replay session
pub async fn delete_session(
    State(state): State<ReplayState>,
    Path(session_id): Path<String>,
) -> StatusCode {
    if state.storage.delete_session(&session_id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/replay/stats - Storage statistics
pub async fn get_storage_stats(State(state): State<ReplayState>) -> Json<serde_json::Value> {
    let stats = state.storage.stats().await;

    Json(serde_json::json!({
        "total_sessions": stats.total_sessions,
        "total_frames": stats.total_frames,
        "total_diffs": stats.total_diffs,
        "estimated_size_bytes": stats.estimated_size_bytes,
        "max_frames_per_session": stats.max_frames_per_session,
        "max_storage_bytes": stats.max_storage_bytes,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_replay_state() -> ReplayState {
        ReplayState {
            storage: Arc::new(crate::replay::storage::ReplayStorage::new()),
            player_manager: Arc::new(crate::replay::player::PlayerManager::new()),
        }
    }

    #[tokio::test]
    async fn test_get_session_not_found() {
        let state = test_replay_state();
        let response = get_session(State(state), Path("sess1".to_string())).await;
        assert_eq!(response.unwrap_err(), StatusCode::NOT_FOUND);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    #[tokio::test]
    async fn test_get_frames_not_found() {
        let state = test_replay_state();
        let response = get_frames(
            State(state),
            Path("sess1".to_string()),
            Query(FrameQuery {
                start_tick: None,
                end_tick: None,
            }),
        )
        .await;
        assert_eq!(response.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_bookmarks_empty() {
        let state = test_replay_state();
        let response = get_bookmarks(State(state), Path("sess1".to_string())).await;
        assert!(response.0.is_empty());
    }

    #[tokio::test]
    async fn test_remove_bookmark() {
        let state = test_replay_state();
        let response =
            remove_bookmark(State(state), Path(("sess1".to_string(), "bm1".to_string()))).await;
        assert_eq!(response, StatusCode::NO_CONTENT);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_list_run_sessions_empty() {
        let state = test_replay_state();
        let response = list_run_sessions(State(state), Path("run1".to_string())).await;
        assert!(response.0.is_empty());
    }

    #[tokio::test]
    async fn test_delete_session_not_found() {
        let state = test_replay_state();
        let response = delete_session(State(state), Path("sess1".to_string())).await;
        assert_eq!(response, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_storage_stats() {
        let state = test_replay_state();
        let response = get_storage_stats(State(state)).await;
        assert_eq!(response.0["total_sessions"], 0);
        assert_eq!(response.0["total_frames"], 0);
        assert_eq!(response.0["total_diffs"], 0);
    }

    #[tokio::test]
    async fn test_add_bookmark() {
        let state = test_replay_state();
        let request = BookmarkRequest {
            tick: 42,
            label: "test".to_string(),
        };
        let response = add_bookmark(State(state), Path("sess1".to_string()), Json(request)).await;
        let bookmark = response.unwrap().0;
        assert_eq!(bookmark.tick, 42);
        assert_eq!(bookmark.label, "test");
        assert!(!bookmark.bookmark_id.is_empty());
    }
}
