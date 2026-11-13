//! External Event Injection API (P1-SIM-5)
//!
//! Provides REST endpoints for injecting external events into simulations.
//!
//! ## Routes
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/events/:simulation_id` | Submit an event |
//! | POST | `/api/events/:simulation_id/immediate` | Inject immediately |
//! | POST | `/api/events/:simulation_id/scenario` | Inject scenario |
//! | GET | `/api/events/:simulation_id/pending` | List pending events |
//! | GET | `/api/events/:simulation_id/stats` | Event statistics |
//! | GET | `/api/events/:event_id` | Get event status |
//! | DELETE | `/api/events/:event_id` | Cancel event |
//! | POST | `/api/events/:event_id/cancel` | Cancel event (alt) |

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::Event,
    response::sse::Sse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use crate::events::injector::{EventInjector, InjectionResult};
use crate::events::types::{EventMetadata, EventPriority, ExternalEvent, Scenario};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// State for event endpoints
#[derive(Clone, Debug)]
pub struct EventState {
    /// injector
    pub injector: Arc<EventInjector>,
}

/// Request to submit an event
#[derive(Debug, Deserialize)]
pub struct SubmitEventRequest {
    /// event
    pub event: ExternalEvent,
    /// priority
    pub priority: Option<EventPriority>,
    /// target tick
    pub target_tick: Option<u64>,
    /// source
    pub source: Option<String>,
    /// tags
    pub tags: Option<Vec<String>>,
}

/// Response for event submission
#[derive(Debug, Serialize)]
pub struct SubmitEventResponse {
    /// event id
    pub event_id: String,
    /// queued
    pub queued: bool,
    /// estimated processing tick
    pub estimated_processing_tick: Option<u64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Response for immediate injection
#[derive(Debug, Serialize)]
pub struct ImmediateInjectResponse {
    /// event id
    pub event_id: String,
    /// success
    pub success: bool,
    /// affected
    pub affected: Vec<String>,
    /// error
    pub error: Option<String>,
    /// processing time ms
    pub processing_time_ms: u64,
}

/// Scenario injection request
#[derive(Debug, Deserialize)]
pub struct InjectScenarioRequest {
    /// scenario
    pub scenario: Scenario,
}

/// Scenario injection response
#[derive(Debug, Serialize)]
pub struct InjectScenarioResponse {
    /// scenario id
    pub scenario_id: String,
    /// events submitted
    pub events_submitted: usize,
    /// results
    pub results: Vec<InjectionResultView>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Injection result view
#[derive(Debug, Serialize)]
pub struct InjectionResultView {
    /// event id
    pub event_id: String,
    /// success
    pub success: bool,
    /// error
    pub error: Option<String>,
}

impl From<InjectionResult> for InjectionResultView {
    fn from(r: InjectionResult) -> Self {
        Self {
            event_id: r.event_id,
            success: r.success,
            error: r.error,
        }
    }
}

/// Event status response
#[derive(Debug, Serialize)]
pub struct EventStatusResponse {
    /// event id
    pub event_id: String,
    /// status
    pub status: String,
    /// event type
    pub event_type: String,
    /// priority
    pub priority: String,
    /// created at
    pub created_at: String,
    /// target tick
    pub target_tick: Option<u64>,
    /// attempts
    pub attempts: u32,
    /// last error
    pub last_error: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Pending events response
#[derive(Debug, Serialize)]
pub struct PendingEventsResponse {
    /// simulation id
    pub simulation_id: String,
    /// pending count
    pub pending_count: usize,
    /// events
    pub events: Vec<EventStatusResponse>,
}

/// Event statistics response
#[derive(Debug, Serialize)]
pub struct EventStatsResponse {
    /// total queued
    pub total_queued: u64,
    /// completed
    pub completed: u64,
    /// failed
    pub failed: u64,
    /// pending
    pub pending: u64,
    /// avg processing time ms
    pub avg_processing_time_ms: f64,
    /// by type
    pub by_type: std::collections::HashMap<String, u64>,
    /// by priority
    pub by_priority: std::collections::HashMap<String, u64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /`api/events/:simulation_id` - Submit an event
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn submit_event(
    State(state): State<EventState>,
    Path(simulation_id): Path<String>,
    Json(request): Json<SubmitEventRequest>,
) -> Result<(StatusCode, Json<SubmitEventResponse>), StatusCode> {
    info!(simulation_id = %simulation_id, "Submitting external event");

    let mut metadata = EventMetadata::new(request.source.unwrap_or_else(|| "api".to_string()))
        .with_priority(request.priority.unwrap_or_default());

    if let Some(tick) = request.target_tick {
        metadata = metadata.at_tick(tick);
    }

    if let Some(tags) = request.tags {
        for tag in tags {
            metadata = metadata.with_tag(tag);
        }
    }

    let result = state
        .injector
        .submit(&simulation_id, request.event, metadata)
        .await;

    if result.success {
        Ok((
            StatusCode::ACCEPTED,
            Json(SubmitEventResponse {
                event_id: result.event_id,
                queued: true,
                estimated_processing_tick: request.target_tick,
            }),
        ))
    } else {
        error!(error = ?result.error, "Event validation failed");
        Err(StatusCode::BAD_REQUEST)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /`api/events/:simulation_id/immediate` - Inject immediately
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn inject_immediate(
    State(state): State<EventState>,
    Path(simulation_id): Path<String>,
    Json(request): Json<SubmitEventRequest>,
) -> Result<Json<ImmediateInjectResponse>, StatusCode> {
    info!(simulation_id = %simulation_id, "Injecting event immediately");

    let metadata = EventMetadata::new(request.source.unwrap_or_else(|| "api".to_string()))
        .with_priority(request.priority.unwrap_or(EventPriority::High));

    let result = state
        .injector
        .inject_immediate(&simulation_id, request.event, metadata)
        .await;

    // Outcome is the same regardless of whether the event is still pending
    let _ = state.injector.get_event(&result.event_id).await;
    Ok(Json(ImmediateInjectResponse {
        event_id: result.event_id,
        success: result.success,
        affected: Vec::new(),
        error: result.error,
        processing_time_ms: result.processing_time_ms,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /`api/events/:simulation_id/scenario` - Inject scenario
pub async fn inject_scenario(
    State(state): State<EventState>,
    Path(simulation_id): Path<String>,
    Json(request): Json<InjectScenarioRequest>,
) -> Json<InjectScenarioResponse> {
    info!(
        simulation_id = %simulation_id,
        scenario_id = %request.scenario.id,
        "Injecting scenario"
    );

    let results = state
        .injector
        .inject_scenario(&simulation_id, request.scenario.clone())
        .await;

    Json(InjectScenarioResponse {
        scenario_id: request.scenario.id,
        events_submitted: results.len(),
        results: results.into_iter().map(Into::into).collect(),
    })
}

/// GET /`api/events/:simulation_id/pending` - List pending events
pub async fn get_pending_events(
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    State(state): State<EventState>,
    Path(simulation_id): Path<String>,
) -> Json<PendingEventsResponse> {
    let events = state.injector.get_pending(&simulation_id).await;

    let event_views: Vec<_> = events
        .iter()
        .map(|e| EventStatusResponse {
            event_id: e.metadata.event_id.clone(),
            status: format!("{:?}", e.status).to_lowercase(),
            event_type: event_type_string(&e.event),
            priority: format!("{:?}", e.metadata.priority).to_lowercase(),
            created_at: e.metadata.created_at.clone(),
            target_tick: e.metadata.target_tick,
            attempts: e.attempts,
            last_error: e.last_error.clone(),
        })
        .collect();

    Json(PendingEventsResponse {
        simulation_id,
        pending_count: event_views.len(),
        events: event_views,
    })
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`api/events/:simulation_id/stats` - Event statistics
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_event_stats(
    State(state): State<EventState>,
    Path(_simulation_id): Path<String>,
) -> Json<EventStatsResponse> {
    let stats = state.injector.stats().await;

    Json(EventStatsResponse {
        total_queued: stats.total_queued,
        completed: stats.completed,
        failed: stats.failed,
        pending: stats.pending,
        avg_processing_time_ms: stats.avg_processing_time_ms,
        by_type: stats.by_type,
        by_priority: stats.by_priority,
    })
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`api/events/:event_id` - Get event status
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_event_status(
    State(state): State<EventState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventStatusResponse>, StatusCode> {
    match state.injector.get_event(&event_id).await {
        Some(event) => Ok(Json(EventStatusResponse {
            event_id: event.metadata.event_id,
            status: format!("{:?}", event.status).to_lowercase(),
            event_type: event_type_string(&event.event),
            priority: format!("{:?}", event.metadata.priority).to_lowercase(),
            created_at: event.metadata.created_at,
            target_tick: event.metadata.target_tick,
            attempts: event.attempts,
            last_error: event.last_error,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// DELETE /`api/events/:event_id` - Cancel event
pub async fn cancel_event(
    State(state): State<EventState>,
    Path(event_id): Path<String>,
) -> StatusCode {
    if state.injector.cancel(&event_id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /`api/events/:event_id/cancel` - Cancel event (alt)
pub async fn cancel_event_post(
    State(state): State<EventState>,
    Path(event_id): Path<String>,
) -> StatusCode {
    cancel_event(State(state), Path(event_id)).await
}

/// GET /`api/events/:simulation_id/stream` - SSE event stream
pub async fn stream_events(
    State(state): State<EventState>,
    Path(simulation_id): Path<String>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    use futures::stream;

    info!(simulation_id = %simulation_id, "Starting event SSE stream");

    let state_stream = state;
    let sim_id = simulation_id;

    let stream = stream::unfold(
        (state_stream, sim_id, 0u64),
        |(state, sim_id, last_count)| async move {
            tokio::time::sleep(Duration::from_secs(2)).await;

            let pending = state.injector.get_pending(&sim_id).await;
            let current_count = pending.len() as u64;

            if current_count == last_count {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                // Send heartbeat
                let event = Event::default().event("heartbeat").data("{}");
                Some((Ok(event), (state, sim_id, last_count)))
            } else {
                let response = PendingEventsResponse {
                    simulation_id: sim_id.clone(),
                    pending_count: pending.len(),
                    events: pending
                        .iter()
                        .map(|e| EventStatusResponse {
                            event_id: e.metadata.event_id.clone(),
                            status: format!("{:?}", e.status).to_lowercase(),
                            event_type: event_type_string(&e.event),
                            priority: format!("{:?}", e.metadata.priority).to_lowercase(),
                            created_at: e.metadata.created_at.clone(),
                            target_tick: e.metadata.target_tick,
                            attempts: e.attempts,
                            last_error: e.last_error.clone(),
                        })
                        .collect(),
                };

                let event = Event::default()
                    .event("pending_events")
                    .data(serde_json::to_string(&response).unwrap_or_default());

                Some((Ok(event), (state, sim_id, current_count)))
            }
        },
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// Helper to get event type string
fn event_type_string(event: &ExternalEvent) -> String {
    match event {
        ExternalEvent::AgentEvent(_) => "agent",
        ExternalEvent::EnvironmentEvent(_) => "environment",
        ExternalEvent::NarrativeEvent(_) => "narrative",
        ExternalEvent::CommunicationEvent(_) => "communication",
        ExternalEvent::ResourceEvent(_) => "resource",
        ExternalEvent::SystemEvent(_) => "system",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::types::*;

    #[tokio::test]
    async fn test_event_type_string_agent() {
        let event = ExternalEvent::AgentEvent(AgentEvent {
            target_agents: vec![],
            action: AgentAction::SetState,
            parameters: std::collections::HashMap::default(),
        });
        assert_eq!(event_type_string(&event), "agent");
    }

    #[tokio::test]
    async fn test_event_type_string_environment() {
        let event = ExternalEvent::EnvironmentEvent(EnvironmentEvent {
            change_type: EnvironmentChangeType::WeatherChange,
            regions: vec![],
            parameters: std::collections::HashMap::default(),
        });
        assert_eq!(event_type_string(&event), "environment");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_event_type_string_narrative() {
        let event = ExternalEvent::NarrativeEvent(NarrativeEvent {
            beat_id: "beat1".to_string(),
            story_context: "test".to_string(),
            conditions: vec![],
            success_outcomes: vec![],
            failure_outcomes: vec![],
        });
        assert_eq!(event_type_string(&event), "narrative");
    }

    #[tokio::test]
    async fn test_event_type_string_communication() {
        let event = ExternalEvent::CommunicationEvent(CommunicationEvent {
            sender: "a".to_string(),
            recipients: vec![],
            message_type: MessageType::Conversation,
            content: "hi".to_string(),
            urgency: UrgencyLevel::Normal,
            metadata: std::collections::HashMap::default(),
        });
        assert_eq!(event_type_string(&event), "communication");
    }

    #[tokio::test]
    async fn test_event_type_string_resource() {
        let event = ExternalEvent::ResourceEvent(ResourceEvent {
            resource_type: "gold".to_string(),
            amount: 10.0,
            targets: vec![],
            reason: "test".to_string(),
        });
        assert_eq!(event_type_string(&event), "resource");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_event_type_string_system() {
        let event = ExternalEvent::SystemEvent(SystemEvent {
            command: SystemCommand::Pause,
            parameters: std::collections::HashMap::default(),
        });
        assert_eq!(event_type_string(&event), "system");
    }

    #[tokio::test]
    async fn test_cancel_event_not_found() {
        let state = EventState {
            injector: Arc::new(crate::events::injector::EventInjector::new()),
        };
        let response = cancel_event(State(state), Path("nonexistent".to_string())).await;
        assert_eq!(response, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_cancel_event_post_not_found() {
        let state = EventState {
            injector: Arc::new(crate::events::injector::EventInjector::new()),
        };
        let response = cancel_event_post(State(state), Path("nonexistent".to_string())).await;
        assert_eq!(response, StatusCode::NOT_FOUND);
    }
}
