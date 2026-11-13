//! Server-Sent Events Implementation (§23.10.3)
//!
//! Provides real-time streaming of simulation metrics via SSE.

use axum::{
    extract::Path,
    response::sse::{Event, Sse},
};
use futures::stream::{Stream, StreamExt};
use serde::Serialize;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
#[allow(unused_imports)]
use tracing::{error, info, warn};

/// Default buffer size for broadcast channels
pub const DEFAULT_BUFFER_SIZE: usize = 1024;

/// Default keep-alive interval
pub const DEFAULT_KEEP_ALIVE: Duration = Duration::from_secs(15);

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create a new broadcast channel for SSE streaming
#[must_use]
pub fn create_broadcast_channel<T>(
    buffer_size: usize,
) -> (broadcast::Sender<T>, broadcast::Receiver<T>)
where
    T: Clone + Send + 'static,
{
    broadcast::channel(buffer_size)
}

/// SSE stream wrapper type
pub type SseStream = Sse<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>;

/// Wrapper for broadcast streams
#[allow(dead_code)]
#[derive(Debug)]
pub struct BroadcastStreamWrapper<T> {
    receiver: broadcast::Receiver<T>,
}

impl<T> BroadcastStreamWrapper<T> {
    #[must_use]
    /// fn
    pub const fn new(receiver: broadcast::Receiver<T>) -> Self {
        Self { receiver }
    }
}

/// Simulation metrics event for SSE streaming
#[derive(Debug, Clone, Serialize)]
pub struct SimulationMetricsEvent {
    /// tick
    pub tick: u64,
    /// agent count
    pub agent_count: usize,
    /// cooperation rate
    pub cooperation_rate: f64,
    /// mean docas
    pub mean_docas: [f32; 5],
    /// trade volume
    pub trade_volume: u64,
    /// dispute rate
    pub dispute_rate: f64,
    /// timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl SimulationMetricsEvent {
    /// Create a new metrics event
    #[must_use]
    pub fn new(
        tick: u64,
        agent_count: usize,
        cooperation_rate: f64,
        mean_docas: [f32; 5],
        trade_volume: u64,
        dispute_rate: f64,
    ) -> Self {
        Self {
            tick,
            agent_count,
            cooperation_rate,
            mean_docas,
            trade_volume,
            dispute_rate,
            timestamp: chrono::Utc::now(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Convert to SSE Event
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn to_sse_event(&self) -> Result<Event, serde_json::Error> {
        let data = serde_json::to_string(self)?;
        Ok(Event::default().event("metric").data(data))
    }
}

/// SSE broadcaster for simulation jobs
#[derive(Debug, Clone)]
pub struct SseBroadcaster {
    sender: broadcast::Sender<SimulationMetricsEvent>,
}

impl SseBroadcaster {
    /// Create a new broadcaster with specified buffer size
    #[must_use]
    pub fn new(buffer_size: usize) -> Self {
        let (sender, _receiver) = broadcast::channel(buffer_size);
        Self { sender }
    }

    /// Subscribe to the broadcast channel
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<SimulationMetricsEvent> {
        self.sender.subscribe()
    }

    /// Broadcast a metrics event to all subscribers
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn broadcast(
        &self,
        event: SimulationMetricsEvent,
    ) -> Result<usize, broadcast::error::SendError<SimulationMetricsEvent>> {
        self.sender.send(event)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the number of active subscribers
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for SseBroadcaster {
    fn default() -> Self {
        Self::new(DEFAULT_BUFFER_SIZE)
    }
}

/// State for managing job-specific SSE streams
#[derive(Debug, Default, Clone)]
pub struct StreamingState {
    broadcasters: Arc<parking_lot::RwLock<std::collections::HashMap<String, SseBroadcaster>>>,
}

impl StreamingState {
    /// Create new streaming state
    #[must_use]
    pub fn new() -> Self {
        Self {
            broadcasters: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Register a new job broadcaster
    pub fn register_job(&self, job_id: &str) -> SseBroadcaster {
        let broadcaster = SseBroadcaster::new(DEFAULT_BUFFER_SIZE);
        self.broadcasters
            .write()
            .insert(job_id.to_string(), broadcaster.clone());
        info!("Registered SSE broadcaster for job {}", job_id);
        broadcaster
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get broadcaster for a job
    #[must_use]
    pub fn get_broadcaster(&self, job_id: &str) -> Option<SseBroadcaster> {
        self.broadcasters.read().get(job_id).cloned()
    }

    /// Remove a job broadcaster
    pub fn remove_job(&self, job_id: &str) {
        self.broadcasters.write().remove(job_id);
        info!("Removed SSE broadcaster for job {}", job_id);
    }

    /// Subscribe to a job's SSE stream
    #[must_use]
    pub fn subscribe(&self, job_id: &str) -> Option<broadcast::Receiver<SimulationMetricsEvent>> {
        self.get_broadcaster(job_id).map(|b| b.subscribe())
    }
}

/// Create an SSE stream from a broadcast receiver
#[must_use]
pub fn create_sse_stream(
    receiver: broadcast::Receiver<SimulationMetricsEvent>,
) -> std::pin::Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> {
    Box::pin(
        BroadcastStream::new(receiver).filter_map(|result| async move {
            match result {
                Ok(event) => match event.to_sse_event() {
                    Ok(evt) => Some(Ok(evt)),
                    Err(e) => {
                        error!("Failed to serialize metrics event: {}", e);
                        None
                    }
                },
                Err(e) => {
                    warn!("Broadcast stream lagged: {}", e);
                    None
                }
            }
        }),
    )
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Axum handler for SSE streaming
///
/// Returns SSE stream for the requested job. Uses boxed stream for type erasure
/// to handle both success and missing job cases uniformly.
pub async fn sse_handler(
    Path(job_id): Path<String>,
    streaming_state: axum::extract::State<StreamingState>,
) -> Sse<Box<dyn Stream<Item = Result<Event, Infallible>> + Send + Unpin>> {
    info!("SSE connection requested for job {}", job_id);

    let stream: Box<dyn Stream<Item = Result<Event, Infallible>> + Send + Unpin> =
        if let Some(receiver) = streaming_state.subscribe(&job_id) {
            Box::new(create_sse_stream(receiver))
        } else {
            warn!("No broadcaster found for job {}", job_id);
            // Return empty stream that immediately ends
            Box::new(futures::stream::iter(
                Vec::<Result<Event, Infallible>>::new(),
            ))
        };

    // Note: KeepAliveStream changes the type, so we use the current implementation
    // In production, you may want to add keep-alive pings
    Sse::new(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_broadcaster() {
        let broadcaster = SseBroadcaster::new(10);
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();

        let event =
            SimulationMetricsEvent::new(100, 1000, 0.75, [0.5, 0.3, 0.2, 0.1, 0.4], 500, 0.05);

        // Broadcast to all subscribers
        let count = broadcaster.broadcast(event.clone()).unwrap();
        assert_eq!(count, 2);

        // Both receivers should get the event
        let recv1 = rx1.try_recv().unwrap();
        let recv2 = rx2.try_recv().unwrap();
        assert_eq!(recv1.tick, event.tick);
        assert_eq!(recv2.tick, event.tick);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_streaming_state() {
        let state = StreamingState::new();

        // Register a job
        let broadcaster = state.register_job("test-job-1");
        assert!(state.get_broadcaster("test-job-1").is_some());

        // Subscribe through state
        let mut rx = state.subscribe("test-job-1").unwrap();

        // Broadcast through the returned broadcaster
        let event = SimulationMetricsEvent::new(1, 100, 0.5, [0.1; 5], 10, 0.01);
        broadcaster.broadcast(event).unwrap();

        // Should receive
        let recv = rx.try_recv().unwrap();
        assert_eq!(recv.tick, 1);

        // Remove job
        state.remove_job("test-job-1");
        assert!(state.get_broadcaster("test-job-1").is_none());
    }

    #[test]
    fn test_metrics_event_serialization() {
        let event =
            SimulationMetricsEvent::new(100, 1000, 0.75, [0.5, 0.3, 0.2, 0.1, 0.4], 500, 0.05);

        // Verify the event can be created successfully
        let _sse_event = event.to_sse_event().unwrap();
    }
}
