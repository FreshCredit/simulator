//! Optimized Event Queue Management
//!
//! Single-lock design eliminates lock ordering issues and reduces contention.
//! Uses `tokio::sync::Mutex` for exclusive access to all queue state.

use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{trace, warn};

use super::types::{
    EventFilter, EventMetadata, EventOutcome, EventStats, EventStatus, ExternalEvent, QueuedEvent,
};

/// Priority queue entry wrapper
#[derive(Clone, Debug)]
struct QueueEntry {
    priority: i32,
    target_tick: u64,
    sequence: u64,
    event_id: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
            && self.target_tick == other.target_tick
            && self.sequence == other.sequence
    }
}

impl Eq for QueueEntry {}

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.target_tick.cmp(&other.target_tick))
            .then_with(|| self.sequence.cmp(&other.sequence))
    }
}

/// Consolidated queue state protected by single lock
#[derive(Debug)]
struct QueueState {
    pending: BinaryHeap<QueueEntry>,
    events: HashMap<String, QueuedEvent>,
    ready: VecDeque<String>,
    stats: EventStats,
    sequence: u64,
}

/// Optimized event queue with single-lock design
#[derive(Debug)]
pub struct EventQueue {
    state: Arc<Mutex<QueueState>>,
    max_size: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EventQueue {
    /// Create a new event queue
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(10000)
    }

    /// Create with custom capacity
    #[must_use]
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(QueueState {
                pending: BinaryHeap::new(),
                events: HashMap::new(),
                ready: VecDeque::new(),
                stats: EventStats::new(),
                sequence: 0,
            })),
            max_size,
        }
    }

    /// Enqueue an event
    pub async fn enqueue(&self, event: ExternalEvent, metadata: EventMetadata) -> String {
        let event_id = metadata.event_id.clone();
        let mut state = self.state.lock().await;

        // Check if queue is full
        if state.events.len() >= self.max_size {
            warn!("Event queue is full, dropping event");
            return event_id;
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let queued = QueuedEvent {
            metadata: metadata.clone(),
            event: event.clone(),
            status: EventStatus::Pending,
            attempts: 0,
            last_error: None,
        };

        // Store the event and update stats
        state.events.insert(event_id.clone(), queued);
        state.stats.record_queued(&event, metadata.priority);

        // Increment sequence counter
        state.sequence += 1;
        let sequence = state.sequence;

        let entry = QueueEntry {
            priority: metadata.priority as i32,
            target_tick: metadata.target_tick.unwrap_or(0),
            sequence,
            event_id: event_id.clone(),
        };

        // Route to ready or pending queue
        if entry.target_tick == 0 {
            state.ready.push_back(event_id.clone());
        } else {
            state.pending.push(entry);
        }

        trace!(event_id = %event_id, "Event enqueued");
        event_id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get next event for processing
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub async fn dequeue(&self, current_tick: u64) -> Option<QueuedEvent> {
        let mut state = self.state.lock().await;

        // First check ready queue
        while let Some(event_id) = state.ready.pop_front() {
            if let Some(mut event) = state.events.remove(&event_id) {
                if event.metadata.is_expired() {
                    trace!(event_id = %event_id, "Skipping expired event");
                    continue;
                }
                event.status = EventStatus::Processing;
                state.events.insert(event_id.clone(), event.clone());
                return Some(event);
            }
        }

        // Check priority queue
        if let Some(entry) = state.pending.peek() {
            if entry.target_tick <= current_tick {
                let entry = state.pending.pop()?;
                let event_id = entry.event_id;

                if let Some(mut event) = state.events.remove(&event_id) {
                    if event.metadata.is_expired() {
                        trace!(event_id = %event_id, "Skipping expired event");
                        return None;
                    }
                    event.status = EventStatus::Processing;
                    state.events.insert(event_id.clone(), event.clone());
                    return Some(event);
                }
            }
        }

        None
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get event by ID
    pub async fn get(&self, event_id: &str) -> Option<QueuedEvent> {
        let state = self.state.lock().await;
        state.events.get(event_id).cloned()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update event status
    pub async fn update_status(&self, event_id: &str, status: EventStatus) {
        let mut state = self.state.lock().await;
        if let Some(event) = state.events.get_mut(event_id) {
            event.status = status;
        }
    }

    /// Complete an event with outcome
    pub async fn complete(&self, event_id: &str, outcome: EventOutcome) {
        let mut state = self.state.lock().await;
        if let Some(event) = state.events.get_mut(event_id) {
            event.status = EventStatus::Completed;
            state.stats.record_completed(outcome.processing_time_ms);
        }
    }

    /// Fail an event with error
    pub async fn fail(&self, event_id: &str, error: String) {
        let mut state = self.state.lock().await;
        if let Some(event) = state.events.get_mut(event_id) {
            event.attempts += 1;
            event.last_error = Some(error.clone());
            if event.attempts >= 3 {
                event.status = EventStatus::Failed;
                state.stats.record_failed();
            } else {
                // Retry: add back to ready queue
                event.status = EventStatus::Pending;
                state.ready.push_back(event_id.to_string());
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get queue statistics
    pub async fn stats(&self) -> EventStats {
        let state = self.state.lock().await;
        state.stats.clone()
    }

    /// Get current queue size
    pub async fn len(&self) -> usize {
        let state = self.state.lock().await;
        state.events.len()
    }

    /// Check if queue is empty
    pub async fn is_empty(&self) -> bool {
        let state = self.state.lock().await;
        state.events.is_empty()
    }

    /// List events with optional filter
    pub async fn list(&self, filter: Option<EventFilter>) -> Vec<QueuedEvent> {
        let state = self.state.lock().await;
        state
            .events
            .values()
            .filter(|e| filter.as_ref().is_none_or(|f| f.matches(e)))
            .cloned()
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .collect()
    }

    /// Cancel an event
    pub async fn cancel(&self, event_id: &str) -> bool {
        let mut state = self.state.lock().await;
        if let Some(event) = state.events.get_mut(event_id) {
            event.status = EventStatus::Cancelled;
            true
        } else {
            false
        }
    }

    /// Clear all events
    pub async fn clear(&self) {
        let mut state = self.state.lock().await;
        state.pending.clear();
        state.events.clear();
        state.ready.clear();
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}
