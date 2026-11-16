//! Agent Memory API Module (P1-SIM-4)
//!
//! Provides REST endpoints for agent memory management.
//!
//! ## Routes
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/memory` | Store a memory |
//! | GET | `/api/memory/:agent_id` | Query agent memories |
//! | GET | `/api/memory/:agent_id/stats` | Memory statistics |
//! | GET | `/api/memory/:agent_id/retrieve` | Advanced retrieval |
//! | POST | `/api/memory/:agent_id/consolidate` | Run consolidation |
//! | DELETE | `/api/memory/:agent_id` | Clear all memories |
//! | DELETE | `/api/memory/:agent_id/:memory_id` | Delete specific memory |
//! | POST | `/api/memory/:agent_id/:memory_id/relate` | Add related memory |

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

use crate::memory::consolidation::MemoryMaintenance;
use crate::memory::retrieval::{MemoryRetriever, RetrievalContext};
use crate::memory::store::MemoryStore;
use crate::memory::types::{
    MemoryQuery, MemoryStats, QueryMemoryRequest, ScoredMemory, StoreMemoryRequest,
    StoreMemoryResponse,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// State for memory endpoints
#[derive(Clone, Debug)]
pub struct MemoryState {
    /// store
    pub store: Arc<MemoryStore>,
    /// maintenance
    pub maintenance: Arc<tokio::sync::Mutex<MemoryMaintenance>>,
}

/// Response for memory query
#[derive(Debug, Serialize)]
pub struct MemoryQueryResponse {
    /// agent id
    pub agent_id: String,
    /// memories
    pub memories: Vec<ScoredMemory>,
    /// total
    pub total: usize,
}

/// Response for memory retrieval
#[derive(Debug, Serialize)]
pub struct MemoryRetrieveResponse {
    /// agent id
    pub agent_id: String,
    /// retrieved
    pub retrieved: Vec<ScoredMemory>,
    /// method
    pub method: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Request for context-based retrieval
#[derive(Debug, Deserialize)]
pub struct ContextRetrieveRequest {
    /// location
    pub location: Option<String>,
    /// participants
    pub participants: Option<Vec<String>>,
    /// current tick
    pub current_tick: Option<u64>,
    /// emotional state
    pub emotional_state: Option<f64>,
    /// top k
    pub top_k: Option<usize>,
}

/// Request for similarity-based retrieval
#[derive(Debug, Deserialize)]
pub struct SimilarityRetrieveRequest {
    /// memory id
    pub memory_id: String,
    /// top k
    pub top_k: Option<usize>,
}

/// Request for associative retrieval
#[derive(Debug, Deserialize)]
pub struct AssociativeRetrieveRequest {
    /// seed memory id
    pub seed_memory_id: String,
    /// max depth
    pub max_depth: Option<u32>,
}

/// Consolidation response
#[derive(Debug, Serialize)]
pub struct ConsolidationResponse {
    /// agent id
    pub agent_id: String,
    /// memories processed
    pub memories_processed: usize,
    /// plans created
    pub plans_created: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Request to add related memory
#[derive(Debug, Deserialize)]
pub struct AddRelatedRequest {
    /// related memory id
    pub related_memory_id: String,
}

/// POST /api/memory - Store a memory
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn store_memory(
    State(state): State<MemoryState>,
    Json(request): Json<StoreMemoryRequest>,
) -> Result<(StatusCode, Json<StoreMemoryResponse>), StatusCode> {
    let agent_id = request.agent_id.clone();
    if agent_id.trim().is_empty() || agent_id.len() > 128 {
        return Err(StatusCode::BAD_REQUEST);
    }
    debug!(
        agent_id = %agent_id,
        memory_type = ?request.memory_type,
        "Storing memory"
    );

    let memory_id = state.store.store(request).await;

    Ok((
        StatusCode::CREATED,
        Json(StoreMemoryResponse {
            memory_id,
            agent_id,
            stored: true,
        }),
    ))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`api/memory/:agent_id` - Query agent memories
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn query_memories(
    State(state): State<MemoryState>,
    Path(agent_id): Path<String>,
    Query(query): Query<QueryMemoryRequest>,
) -> Json<MemoryQueryResponse> {
    let memory_query = MemoryQuery {
        memory_type: query.memory_type,
        min_importance: None,
        tags: query.tags,
        from_tick: query.from_tick,
        to_tick: query.to_tick,
        content_search: query.content_search,
        limit: query.limit.unwrap_or(50).clamp(1, 500),
        offset: 0,
        sort_by_recency: true,
    };

    let memories = state.store.query(&agent_id, memory_query).await;
    let total = memories.len();

    Json(MemoryQueryResponse {
        agent_id,
        memories,
        total,
    })
}

/// GET /`api/memory/:agent_id/stats` - Memory statistics
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_memory_stats(
    State(state): State<MemoryState>,
    Path(agent_id): Path<String>,
) -> Result<Json<MemoryStats>, StatusCode> {
    match state.store.get_stats(&agent_id).await {
        Some(stats) => Ok(Json(stats)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// DELETE /`api/memory/:agent_id` - Clear all agent memories
pub async fn clear_memories(
    State(state): State<MemoryState>,
    Path(agent_id): Path<String>,
) -> impl IntoResponse {
    let count = state.store.delete_agent_memories(&agent_id).await;

    info!(agent_id = %agent_id, count = %count, "Cleared agent memories");

    StatusCode::NO_CONTENT
}

/// DELETE /`api/memory/:agent_id/:memory_id` - Delete specific memory
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn delete_memory(
    State(state): State<MemoryState>,
    Path((agent_id, memory_id)): Path<(String, String)>,
) -> StatusCode {
    // Verify the memory belongs to this agent
    if let Some(memory) = state.store.get(&memory_id).await {
        if memory.agent_id != agent_id {
            return StatusCode::FORBIDDEN;
        }
    } else {
        return StatusCode::NOT_FOUND;
    }

    if state.store.delete(&memory_id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /`api/memory/:agent_id/consolidate` - Run consolidation
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn consolidate_memories(
    State(state): State<MemoryState>,
    Path(agent_id): Path<String>,
) -> Result<Json<ConsolidationResponse>, StatusCode> {
    let memories = state.store.get_agent_memories(&agent_id).await;

    if memories.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let current_tick = memories.iter().map(|m| m.tick).max().unwrap_or(0);

    let mut maintenance = state.maintenance.lock().await;
    let (processed, plans) = maintenance.run_consolidation(&memories, current_tick);

    Ok(Json(ConsolidationResponse {
        agent_id,
        memories_processed: processed,
        plans_created: plans.len(),
    }))
}

/// POST /`api/memory/:agent_id/:memory_id/relate` - Add related memory
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
pub async fn add_related_memory(
    State(state): State<MemoryState>,
    Path((agent_id, memory_id)): Path<(String, String)>,
    Json(request): Json<AddRelatedRequest>,
) -> StatusCode {
    // Verify ownership
    if let Some(memory) = state.store.get(&memory_id).await {
        if memory.agent_id != agent_id {
            return StatusCode::FORBIDDEN;
        }
    } else {
        return StatusCode::NOT_FOUND;
    }

    if let Some(related) = state.store.get(&request.related_memory_id).await {
        if related.agent_id != agent_id {
            return StatusCode::FORBIDDEN;
        }
    } else {
        return StatusCode::NOT_FOUND;
    }

    // Record relation linkage between the two memories
    state
        .store
        .link_memories(&memory_id, &request.related_memory_id)
        .await;
    StatusCode::OK
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`api/memory/:agent_id/retrieve/context` - Context-based retrieval
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn retrieve_by_context(
    State(state): State<MemoryState>,
    Path(agent_id): Path<String>,
    Query(params): Query<ContextRetrieveRequest>,
) -> Json<MemoryRetrieveResponse> {
    let memories = state.store.get_agent_memories(&agent_id).await;

    let context = RetrievalContext::new()
        .at_location(params.location.unwrap_or_default())
        .with_participants(params.participants.unwrap_or_default())
        .at_tick(params.current_tick.unwrap_or(0))
        .feeling(params.emotional_state.unwrap_or(0.0));

    let retriever = MemoryRetriever::new();
    let results = retriever.retrieve_by_context(&context, &memories, params.top_k.unwrap_or(10));

    Json(MemoryRetrieveResponse {
        agent_id,
        retrieved: results,
        method: "context".to_string(),
    })
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`api/memory/:agent_id/retrieve/similar/:memory_id` - Similarity-based retrieval
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn retrieve_similar(
    State(state): State<MemoryState>,
    Path((agent_id, cue_memory_id)): Path<(String, String)>,
    Query(params): Query<SimilarityRetrieveRequest>,
) -> Result<Json<MemoryRetrieveResponse>, StatusCode> {
    let Some(cue) = state.store.get(&cue_memory_id).await else {
        return Err(StatusCode::NOT_FOUND);
    };

    if cue.agent_id != agent_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let memories = state.store.get_agent_memories(&agent_id).await;

    let retriever = MemoryRetriever::new();
    let results = retriever.retrieve_similar(&cue, &memories, params.top_k.unwrap_or(10));

    Ok(Json(MemoryRetrieveResponse {
        agent_id,
        retrieved: results,
        method: "similarity".to_string(),
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`api/memory/:agent_id/retrieve/associative/:memory_id` - Associative retrieval
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn retrieve_associative(
    State(state): State<MemoryState>,
    Path((agent_id, seed_memory_id)): Path<(String, String)>,
    Query(params): Query<AssociativeRetrieveRequest>,
) -> Result<Json<MemoryRetrieveResponse>, StatusCode> {
    let Some(seed) = state.store.get(&seed_memory_id).await else {
        return Err(StatusCode::NOT_FOUND);
    };

    if seed.agent_id != agent_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let all_memories: std::collections::HashMap<String, _> = state
        .store
        .get_agent_memories(&agent_id)
        .await
        .into_iter()
        .map(|m| (m.id.clone(), m))
        .collect();

    let retriever = MemoryRetriever::new();
    let results =
        retriever.retrieve_associative(&seed, &all_memories, params.max_depth.unwrap_or(3));

    Ok(Json(MemoryRetrieveResponse {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        agent_id,
        retrieved: results,
        method: "associative".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_memory_state() -> MemoryState {
        MemoryState {
            store: Arc::new(crate::memory::store::MemoryStore::new()),
            maintenance: Arc::new(tokio::sync::Mutex::new(
                crate::memory::consolidation::MemoryMaintenance::new(),
            )),
        }
    }

    #[tokio::test]
    async fn test_get_memory_stats_not_found() {
        let state = test_memory_state();
        let response = get_memory_stats(State(state), Path("agent1".to_string())).await;
        assert_eq!(response.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_clear_memories() {
        let state = test_memory_state();
        let response = clear_memories(State(state), Path("agent1".to_string())).await;
        let (parts, _) = response.into_response().into_parts();
        assert_eq!(parts.status, StatusCode::NO_CONTENT);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_delete_memory_not_found() {
        let state = test_memory_state();
        let response = delete_memory(
            State(state),
            Path(("agent1".to_string(), "mem1".to_string())),
        )
        .await;
        assert_eq!(response, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_add_related_memory_not_found() {
        let state = test_memory_state();
        let request = AddRelatedRequest {
            related_memory_id: "mem2".to_string(),
        };
        let response = add_related_memory(
            State(state),
            Path(("agent1".to_string(), "mem1".to_string())),
            Json(request),
        )
        .await;
        assert_eq!(response, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_consolidate_memories_not_found() {
        let state = test_memory_state();
        let response = consolidate_memories(State(state), Path("agent1".to_string())).await;
        assert_eq!(response.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_store_memory_returns_real_agent_id_and_sanitizes_nan_valence() {
        use crate::memory::types::{EpisodicMemory, MemoryContent, MemoryImportance, MemoryType};

        let state = test_memory_state();
        let req = StoreMemoryRequest {
            agent_id: "agent-42".to_string(),
            memory_type: MemoryType::Episodic,
            content: MemoryContent::Episodic(EpisodicMemory {
                event: "Observed event".to_string(),
                location: None,
                participants: vec![],
                sensory_details: Default::default(),
                narrative: "Observed event".to_string(),
            }),
            importance: Some(MemoryImportance::Normal),
            emotional_valence: Some(f64::NAN),
            tags: Some(vec!["trust".to_string()]),
            tick: 5,
            source: None,
        };
        let (status, Json(resp)) = store_memory(State(state.clone()), Json(req)).await.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(resp.agent_id, "agent-42");
        assert_ne!(resp.agent_id, resp.memory_id);

        let stored = state.store.get(&resp.memory_id).await.unwrap();
        assert!(stored.emotional_valence.is_finite());
        assert_eq!(stored.emotional_valence, 0.0);
    }
}
