//! Event Injector
//!
//! Handles the actual injection of external events into simulations,
//! with support for batch processing and outcome tracking.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::queue::EventQueue;
use super::types::{
    AgentEvent, CommunicationEvent, EnvironmentEvent, EventMetadata, EventOutcome, EventStats,
    EventStatus, ExternalEvent, NarrativeEvent, QueuedEvent, ResourceEvent, Scenario, SystemEvent,
};
use super::validation::EventValidator;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Result of event injection
#[derive(Debug, Clone)]
pub struct InjectionResult {
    /// Event ID
    pub event_id: String,
    /// Whether injection was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Event injector for managing external event injection
pub struct EventInjector {
    /// Event queue
    queue: Arc<EventQueue>,
    /// Event validator
    validator: Arc<RwLock<EventValidator>>,
    /// Active simulations and their event handlers
    handlers: Arc<RwLock<HashMap<String, Box<dyn EventHandler>>>>,
    /// Whether injector is running
    running: Arc<RwLock<bool>>,
}

impl std::fmt::Debug for EventInjector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventInjector").finish_non_exhaustive()
    }
}

/// Trait for handling injected events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an agent event
    async fn handle_agent_event(&self, event: &AgentEvent) -> Result<Vec<String>, String>;

    /// Handle an environment event
    async fn handle_environment_event(
        &self,
        event: &EnvironmentEvent,
    ) -> Result<Vec<String>, String>;

    /// Handle a narrative event
    async fn handle_narrative_event(&self, event: &NarrativeEvent) -> Result<Vec<String>, String>;

    /// Handle a communication event
    async fn handle_communication_event(
        &self,
        event: &CommunicationEvent,
    ) -> Result<Vec<String>, String>;

    /// Handle a resource event
    async fn handle_resource_event(&self, event: &ResourceEvent) -> Result<Vec<String>, String>;

    /// Handle a system event
    async fn handle_system_event(&self, event: &SystemEvent) -> Result<Vec<String>, String>;

    /// Get current simulation tick
    async fn current_tick(&self) -> u64;
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EventInjector {
    /// Create a new event injector
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: Arc::new(EventQueue::new()),
            validator: Arc::new(RwLock::new(EventValidator::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create with custom queue capacity
    #[must_use]
    pub fn with_capacity(max_queue_size: usize) -> Self {
        Self {
            queue: Arc::new(EventQueue::with_capacity(max_queue_size)),
            validator: Arc::new(RwLock::new(EventValidator::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Register a simulation event handler
    pub async fn register_handler(
        &self,
        simulation_id: impl Into<String>,
        handler: Box<dyn EventHandler>,
    ) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(simulation_id.into(), handler);
        info!("Registered event handler for simulation");
    }

    /// Unregister a simulation handler
    pub async fn unregister_handler(&self, simulation_id: &str) {
        let mut handlers = self.handlers.write().await;
        handlers.remove(simulation_id);
    }

    /// Submit an event for injection
    pub async fn submit(
        &self,
        _simulation_id: &str,
        event: ExternalEvent,
        metadata: EventMetadata,
    ) -> InjectionResult {
        let start = std::time::Instant::now();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Validate the event
        let validator = self.validator.read().await;
        let validation = validator.validate(&event);
        drop(validator);

        if !validation.valid {
            return InjectionResult {
                event_id: metadata.event_id.clone(),
                success: false,
                error: Some(validation.errors.join("; ")),
                processing_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Queue the event
        let event_id = self.queue.enqueue(event, metadata).await;

        InjectionResult {
            event_id,
            success: true,
            error: None,
            processing_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Submit and immediately process an event (if possible)
    pub async fn inject_immediate(
        &self,
        simulation_id: &str,
        event: ExternalEvent,
        metadata: EventMetadata,
    ) -> InjectionResult {
        let start = std::time::Instant::now();

        // Validate
        let validator = self.validator.read().await;
        let validation = validator.validate(&event);
        drop(validator);

        if !validation.valid {
            return InjectionResult {
                event_id: metadata.event_id.clone(),
                success: false,
                error: Some(validation.errors.join("; ")),
                processing_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Get handler
        let handlers = self.handlers.read().await;
        let Some(handler) = handlers.get(simulation_id) else {
            return InjectionResult {
                event_id: metadata.event_id.clone(),
                success: false,
                error: Some(format!("No handler for simulation: {simulation_id}")),
                processing_time_ms: start.elapsed().as_millis() as u64,
            };
        };

        // Process immediately
        let outcome = self.process_event(handler.as_ref(), &event).await;

        InjectionResult {
            event_id: metadata.event_id,
            success: outcome.success,
            error: outcome.error,
            processing_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Inject a scenario (batch of events)
    pub async fn inject_scenario(
        &self,
        simulation_id: &str,
        scenario: Scenario,
    ) -> Vec<InjectionResult> {
        let submit_futs = scenario.events.into_iter().map(|scenario_event| {
            let metadata = EventMetadata::new("scenario")
                .with_priority(scenario_event.priority)
                .at_tick(scenario_event.delay_ticks);

            self.submit(simulation_id, scenario_event.event, metadata)
        });
        let results = futures::future::join_all(submit_futs).await;

        info!(
            scenario_id = %scenario.id,
            event_count = %results.len(),
            "Scenario submitted"
        );

        results
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Start the injection processor loop
    pub fn start_processor(&self) {
        let queue = self.queue.clone();
        let handlers = self.handlers.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

            loop {
                interval.tick().await;

                // Check if still running
                {
                    let r = running.read().await;
                    if !*r {
                        break;
                    }
                }

                // Process pending events
                // Get current tick from first available handler
                let current_tick = {
                    let handlers_guard = handlers.read().await;
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    if let Some((_, handler)) = handlers_guard.iter().next() {
                        handler.current_tick().await
                    } else {
                        0
                    }
                };

                // Get events to process
                while let Some(queued) = queue.dequeue(current_tick).await {
                    // Find appropriate handler
                    // For now, use first available handler - in real impl, track per-event simulation
                    let handlers_guard = handlers.read().await;
                    if let Some((_, handler)) = handlers_guard.iter().next() {
                        let outcome =
                            process_event_with_handler(handler.as_ref(), &queued.event).await;
                        queue.complete(&queued.metadata.event_id, outcome).await;
                    } else {
                        warn!("No handler available for event processing");
                        break;
                    }
                }
            }
        });

        info!("Event injection processor started");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Stop the processor
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Get event by ID
    pub async fn get_event(&self, event_id: &str) -> Option<QueuedEvent> {
        self.queue.get(event_id).await
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get pending events for a simulation
    pub async fn get_pending(&self, _simulation_id: &str) -> Vec<QueuedEvent> {
        // In real implementation, filter by simulation
        use super::types::EventFilter;
        self.queue
            .list(Some(EventFilter {
                status: Some(EventStatus::Pending),
                ..Default::default()
            }))
            .await
    }

    /// Cancel a pending event
    pub async fn cancel(&self, event_id: &str) -> bool {
        self.queue.cancel(event_id).await
    }

    /// Get queue statistics
    pub async fn stats(&self) -> EventStats {
        self.queue.stats().await
    }

    /// Clear all events
    pub async fn clear(&self) {
        self.queue.clear().await;
    }

    /// Process a single event
    async fn process_event(
        &self,
        handler: &dyn EventHandler,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        event: &ExternalEvent,
    ) -> EventOutcome {
        let start = std::time::Instant::now();

        let result = match event {
            ExternalEvent::AgentEvent(e) => handler.handle_agent_event(e).await,
            ExternalEvent::EnvironmentEvent(e) => handler.handle_environment_event(e).await,
            ExternalEvent::NarrativeEvent(e) => handler.handle_narrative_event(e).await,
            ExternalEvent::CommunicationEvent(e) => handler.handle_communication_event(e).await,
            ExternalEvent::ResourceEvent(e) => handler.handle_resource_event(e).await,
            ExternalEvent::SystemEvent(e) => handler.handle_system_event(e).await,
        };

        let processing_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(affected) => EventOutcome {
                event_id: String::new(), // Will be set by caller
                success: true,
                affected,
                error: None,
                processing_time_ms,
                processed_at: chrono::Utc::now().to_rfc3339(),
            },
            Err(error) => EventOutcome {
                event_id: String::new(),
                success: false,
                affected: Vec::new(),
                error: Some(error),
                processing_time_ms,
                processed_at: chrono::Utc::now().to_rfc3339(),
            },
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for EventInjector {
    fn default() -> Self {
        Self::new()
    }
}

/// Process an event with a given handler
async fn process_event_with_handler(
    handler: &dyn EventHandler,
    event: &ExternalEvent,
) -> EventOutcome {
    let start = std::time::Instant::now();

    let result = match event {
        ExternalEvent::AgentEvent(e) => handler.handle_agent_event(e).await,
        ExternalEvent::EnvironmentEvent(e) => handler.handle_environment_event(e).await,
        ExternalEvent::NarrativeEvent(e) => handler.handle_narrative_event(e).await,
        ExternalEvent::CommunicationEvent(e) => handler.handle_communication_event(e).await,
        ExternalEvent::ResourceEvent(e) => handler.handle_resource_event(e).await,
        ExternalEvent::SystemEvent(e) => handler.handle_system_event(e).await,
    };

    let processing_time_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(affected) => EventOutcome {
            event_id: String::new(),
            // TAG: surface=simulation owner=research-team rule=SIM-001
            success: true,
            affected,
            error: None,
            processing_time_ms,
            processed_at: chrono::Utc::now().to_rfc3339(),
        },
        Err(error) => EventOutcome {
            event_id: String::new(),
            success: false,
            affected: Vec::new(),
            error: Some(error),
            processing_time_ms,
            processed_at: chrono::Utc::now().to_rfc3339(),
        },
    }
}

/// Mock event handler for testing
#[cfg(test)]
pub mod test_utils {
    use super::*;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Mock event handler for testing.
    #[derive(Debug)]
    pub struct MockEventHandler {
        /// Tick counter for tracking handler invocations.
        pub tick: Arc<RwLock<u64>>,
    }

    #[async_trait::async_trait]
    impl EventHandler for MockEventHandler {
        async fn handle_agent_event(&self, _event: &AgentEvent) -> Result<Vec<String>, String> {
            Ok(vec!["agent-1".to_string()])
        }

        async fn handle_environment_event(
            &self,
            _event: &EnvironmentEvent,
        ) -> Result<Vec<String>, String> {
            Ok(vec!["global".to_string()])
        }

        async fn handle_narrative_event(
            &self,
            _event: &NarrativeEvent,
        ) -> Result<Vec<String>, String> {
            Ok(vec!["story".to_string()])
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        async fn handle_communication_event(
            &self,
            _event: &CommunicationEvent,
        ) -> Result<Vec<String>, String> {
            Ok(vec!["sender".to_string(), "recipient".to_string()])
        }

        async fn handle_resource_event(
            &self,
            _event: &ResourceEvent,
        ) -> Result<Vec<String>, String> {
            Ok(vec!["resource".to_string()])
        }

        async fn handle_system_event(&self, _event: &SystemEvent) -> Result<Vec<String>, String> {
            Ok(vec!["system".to_string()])
        }

        async fn current_tick(&self) -> u64 {
            *self.tick.read().await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::MockEventHandler;
    use super::*;
    use crate::events::types::{AgentAction, AgentEvent};
    use std::collections::HashMap;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn create_test_event() -> ExternalEvent {
        ExternalEvent::AgentEvent(AgentEvent {
            target_agents: vec!["agent-1".to_string()],
            action: AgentAction::SetState,
            parameters: HashMap::new(),
        })
    }

    #[tokio::test]
    async fn test_submit_event() {
        let injector = EventInjector::new();
        let mock_handler = MockEventHandler {
            tick: Arc::new(RwLock::new(0)),
        };
        injector
            .register_handler("sim-1", Box::new(mock_handler))
            .await;

        let event = create_test_event();
        let metadata = EventMetadata::new("test");

        let result = injector.submit("sim-1", event, metadata).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_inject_immediate() {
        let injector = EventInjector::new();
        let mock_handler = MockEventHandler {
            tick: Arc::new(RwLock::new(0)),
        };
        injector
            .register_handler("sim-1", Box::new(mock_handler))
            .await;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let event = create_test_event();
        let metadata = EventMetadata::new("test");

        let result = injector.inject_immediate("sim-1", event, metadata).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_validation_failure() {
        let injector = EventInjector::new();

        // Invalid event - empty agent list
        let event = ExternalEvent::CommunicationEvent(CommunicationEvent {
            sender: "agent-1".to_string(),
            recipients: vec![], // Empty - should fail
            message_type: crate::events::types::MessageType::Conversation,
            content: "Hello".to_string(),
            urgency: crate::events::types::UrgencyLevel::Normal,
            metadata: HashMap::new(),
        });
        let metadata = EventMetadata::new("test");

        let result = injector.submit("sim-1", event, metadata).await;
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_cancel() {
        let injector = EventInjector::new();
        let mock_handler = MockEventHandler {
            tick: Arc::new(RwLock::new(0)),
        };
        injector
            .register_handler("sim-1", Box::new(mock_handler))
            .await;

        let event = create_test_event();
        let metadata = EventMetadata::new("test");
        let result = injector.submit("sim-1", event, metadata).await;

        assert!(injector.cancel(&result.event_id).await);
    }
}
