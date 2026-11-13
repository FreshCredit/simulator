//! DOCAS Visualization API (B-009)
//!
//! Real-time DOCAS state visualization endpoints for research validation.
//! Provides streaming and snapshot access to agent DOCAS states.

use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use sim_core::docas::DocasState;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// DOCAS snapshot for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocasSnapshot {
    /// agent id
    pub agent_id: u64,
    /// dopamine
    pub dopamine: f32,
    /// oxytocin
    pub oxytocin: f32,
    /// cortisol
    pub cortisol: f32,
    /// adrenaline
    pub adrenaline: f32,
    /// serotonin
    pub serotonin: f32,
    /// timestamp
    pub timestamp: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DocasSnapshot {
    /// Create from DOCAS state with agent ID
    #[must_use]
    pub fn from_state(agent_id: u64, state: &DocasState) -> Self {
        Self {
            agent_id,
            dopamine: state.levels[0],
            oxytocin: state.levels[1],
            cortisol: state.levels[2],
            adrenaline: state.levels[3],
            serotonin: state.levels[4],
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// Population-level DOCAS statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocasPopulationStats {
    /// tick
    pub tick: u64,
    /// agent count
    pub agent_count: usize,
    /// dopamine mean
    pub dopamine_mean: f32,
    /// oxytocin mean
    pub oxytocin_mean: f32,
    /// cortisol mean
    pub cortisol_mean: f32,
    /// adrenaline mean
    pub adrenaline_mean: f32,
    /// serotonin mean
    pub serotonin_mean: f32,
    /// trust index
    pub trust_index: f32, // Computed from DOCAS components
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// State for DOCAS streaming
#[derive(Clone, Debug)]
pub struct DocasStreamState {
    /// tx
    pub tx: broadcast::Sender<DocasPopulationStats>,
}

impl DocasStreamState {
    /// Create new DOCAS stream state
    #[must_use]
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel::<DocasPopulationStats>(100);
        Self { tx }
    }
}

impl Default for DocasStreamState {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current DOCAS snapshot for all agents
pub async fn get_docas_snapshot() -> Json<Vec<DocasSnapshot>> {
    // In production, this would fetch from live simulation state
    // For now, return sample data structure
    let snapshots = vec![DocasSnapshot {
        agent_id: 1,
        dopamine: 0.5,
        oxytocin: 0.6,
        cortisol: 0.3,
        adrenaline: 0.4,
        serotonin: 0.7,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    }];

    Json(snapshots)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get population-level DOCAS statistics
pub async fn get_docas_stats() -> Json<DocasPopulationStats> {
    // In production, compute from live simulation
    let stats = DocasPopulationStats {
        tick: 0,
        agent_count: 100,
        dopamine_mean: 0.5,
        oxytocin_mean: 0.55,
        cortisol_mean: 0.35,
        adrenaline_mean: 0.45,
        serotonin_mean: 0.6,
        trust_index: 0.52,
    };

    Json(stats)
}

/// Stream DOCAS statistics via SSE
pub async fn stream_docas_stats(
    State(state): State<Arc<DocasStreamState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx);

    let sse_stream = stream.map(|result| match result {
        Ok(stats) => {
            let json = serde_json::to_string(&stats).unwrap_or_default();
            Ok(Event::default().data(json))
        }
        Err(_) => Ok(Event::default().data("{}")),
    });

    Sse::new(sse_stream).keep_alive(axum::response::sse::KeepAlive::default())
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get DOCAS history for an agent
pub async fn get_agent_docas_history(Path(agent_id): Path<u64>) -> Json<Vec<DocasSnapshot>> {
    // In production, fetch from time-series database
    // For now, return empty history structure
    let _ = agent_id;
    Json(vec![])
}

/// DOCAS visualization HTML endpoint
pub async fn docas_visualizer() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../../static/docas_visualizer.html"))
}

/// DOCAS visualization client script endpoint
pub async fn docas_js() -> impl axum::response::IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../../static/docas.js"),
    )
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn test_get_docas_snapshot() {
        let response = get_docas_snapshot().await;
        assert_eq!(response.0.len(), 1);
        assert_eq!(response.0[0].agent_id, 1);
        assert_eq!(response.0[0].dopamine, 0.5);
        assert_eq!(response.0[0].oxytocin, 0.6);
        assert_eq!(response.0[0].cortisol, 0.3);
        assert_eq!(response.0[0].adrenaline, 0.4);
        assert_eq!(response.0[0].serotonin, 0.7);
    }

    #[tokio::test]
    async fn test_docas_visualizer_html() {
        let html = docas_visualizer().await;
        assert!(html.0.contains("DOCAS Real-Time Visualization"));
        assert!(html.0.contains("/api/docas/docas.js"));
        assert!(!html.0.contains("<script>"));
    }

    #[tokio::test]
    async fn test_docas_js_endpoint() {
        let res = docas_js().await.into_response();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/javascript; charset=utf-8"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_get_docas_stats() {
        let response = get_docas_stats().await;
        assert_eq!(response.0.tick, 0);
        assert_eq!(response.0.agent_count, 100);
        assert_eq!(response.0.dopamine_mean, 0.5);
        assert_eq!(response.0.oxytocin_mean, 0.55);
        assert_eq!(response.0.cortisol_mean, 0.35);
        assert_eq!(response.0.adrenaline_mean, 0.45);
        assert_eq!(response.0.serotonin_mean, 0.6);
        assert_eq!(response.0.trust_index, 0.52);
    }

    #[tokio::test]
    async fn test_get_agent_docas_history_empty() {
        let response = get_agent_docas_history(Path(1u64)).await;
        assert!(response.0.is_empty());
    }

    #[tokio::test]
    async fn test_docas_visualizer() {
        let response = docas_visualizer().await;
        assert!(response.0.contains("DOCAS"));
        assert!(response.0.contains("dopamine"));
        assert!(response.0.contains("oxytocin"));
        assert!(response.0.contains("cortisol"));
        assert!(response.0.contains("adrenaline"));
        assert!(response.0.contains("serotonin"));
    }
}
