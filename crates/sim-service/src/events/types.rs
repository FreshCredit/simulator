//! Event Types for External Event Injection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of external events that can be injected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExternalEvent {
    /// Agent-specific event (targeted at one or more agents)
    AgentEvent(AgentEvent),
    /// Environment-wide event (affects the whole simulation)
    EnvironmentEvent(EnvironmentEvent),
    /// Narrative/story event (drives plot/scenario)
    NarrativeEvent(NarrativeEvent),
    /// Communication event (message between agents)
    CommunicationEvent(CommunicationEvent),
    /// Resource event (add/remove resources)
    ResourceEvent(ResourceEvent),
    /// System event (control simulation)
    SystemEvent(SystemEvent),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Event targeted at agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentEvent {
    /// Agent IDs to target (empty = all agents)
    pub target_agents: Vec<String>,
    /// Type of agent event
    pub action: AgentAction,
    /// Event parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Actions that can be performed on agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AgentAction {
    /// Modify agent state
    SetState,
    /// Trigger a behavior
    TriggerBehavior,
    /// Modify agent property
    SetProperty,
    /// Teleport/move agent
    Teleport,
    /// Add/remove trait
    ModifyTrait,
    /// Influence emotion
    InfluenceEmotion,
}

/// Environment-wide events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvironmentEvent {
    /// Type of environmental change
    pub change_type: EnvironmentChangeType,
    /// Affected regions (empty = global)
    pub regions: Vec<String>,
    /// Event parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of environmental changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentChangeType {
    /// Weather change
    WeatherChange,
    /// Time of day change
    TimeChange,
    /// Disaster/emergency
    Disaster,
    /// Economic shift
    EconomicShift,
    /// Social event
    SocialEvent,
    /// Policy change
    PolicyChange,
}

/// Narrative/scenario events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NarrativeEvent {
    /// Narrative beat identifier
    pub beat_id: String,
    /// Story context
    pub story_context: String,
    /// Required conditions
    pub conditions: Vec<String>,
    /// Outcomes if conditions met
    pub success_outcomes: Vec<String>,
    /// Outcomes if conditions not met
    pub failure_outcomes: Vec<String>,
}

/// Communication between agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommunicationEvent {
    /// Sender agent ID
    pub sender: String,
    /// Recipient agent IDs
    pub recipients: Vec<String>,
    /// Message type
    pub message_type: MessageType,
    /// Message content
    pub content: String,
    /// Urgency level
    pub urgency: UrgencyLevel,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Direct conversation
    Conversation,
    /// Broadcast announcement
    Announcement,
    /// Warning/alert
    Warning,
    /// Request for help
    Request,
    /// Information sharing
    Information,
    /// Command/order
    Command,
}

/// Urgency levels for communications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum UrgencyLevel {
    /// Low
    Low,
    /// Normal
    Normal,
    /// High
    High,
    /// Critical
    Critical,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Resource-related events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceEvent {
    /// Resource type
    pub resource_type: String,
    /// Amount to add (positive) or remove (negative)
    pub amount: f64,
    /// Target location or agents
    pub targets: Vec<String>,
    /// Reason for change
    pub reason: String,
}

/// System/control events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemEvent {
    /// System command
    pub command: SystemCommand,
    /// Command parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// System commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SystemCommand {
    /// Pause simulation
    Pause,
    /// Resume simulation
    Resume,
    /// Speed up simulation
    SpeedUp,
    /// Slow down simulation
    SlowDown,
    /// Save checkpoint
    Checkpoint,
    /// Load scenario
    LoadScenario,
    /// Inject scenario event
    InjectScenario,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Event priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum EventPriority {
    /// Background events
    Background = 1,
    /// Normal events
    #[default]
    Normal = 2,
    /// Important events
    High = 3,
    /// Critical events (immediate processing)
    Critical = 4,
}

/// Event target specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventTarget {
    /// Target specific simulation
    Simulation(String),
    /// Target specific agents
    Agents(Vec<String>),
    /// Target specific region
    Region(String),
    /// Global event
    Global,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Metadata for an event injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event ID
    pub event_id: String,
    /// Source of the event (user, system, external API)
    pub source: String,
    /// Timestamp when event was created
    pub created_at: String,
    /// Target tick for injection (None = immediate)
    pub target_tick: Option<u64>,
    /// Event priority
    pub priority: EventPriority,
    /// Whether event can be batched with others
    pub batchable: bool,
    /// Expiration time (events older than this are dropped)
    pub expires_at: Option<String>,
    /// Custom tags
    pub tags: Vec<String>,
}

impl EventMetadata {
    /// Create new event metadata
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            source: source.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            target_tick: None,
            priority: EventPriority::Normal,
            batchable: true,
            expires_at: None,
            tags: Vec::new(),
        }
    }

    /// Set target tick
    #[must_use]
    pub const fn at_tick(mut self, tick: u64) -> Self {
        self.target_tick = Some(tick);
        self
    }

    /// Set priority
    #[must_use]
    pub const fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Set expiration
    #[must_use]
    pub fn expires_in(mut self, seconds: u64) -> Self {
        let expiry = chrono::Utc::now() + chrono::Duration::seconds(seconds as i64);
        self.expires_at = Some(expiry.to_rfc3339());
        self
    }

    /// Add tag
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Check if event is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        if let Some(ref expiry) = self.expires_at {
            if let Ok(expiry_time) = chrono::DateTime::parse_from_rfc3339(expiry) {
                return chrono::Utc::now() > expiry_time.with_timezone(&chrono::Utc);
            }
        }
        false
    }
}

/// A queued event ready for injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// The actual event
    pub event: ExternalEvent,
    /// Current status
    pub status: EventStatus,
    /// Processing attempts
    pub attempts: u32,
    /// Last error (if any)
    pub last_error: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Event processing status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    /// Waiting to be processed
    Pending,
    /// Currently being processed
    Processing,
    /// Successfully injected
    Completed,
    /// Failed to inject
    Failed,
    /// Cancelled
    Cancelled,
}

/// Result of event injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOutcome {
    /// Event ID
    pub event_id: String,
    /// Whether injection succeeded
    pub success: bool,
    /// Affected entities
    pub affected: Vec<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Timestamp
    pub processed_at: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EventOutcome {
    /// Create a successful outcome
    #[must_use]
    pub fn success(event_id: String, affected: Vec<String>) -> Self {
        Self {
            event_id,
            success: true,
            affected,
            error: None,
            processing_time_ms: 0,
            processed_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a failed outcome
    pub fn failure(event_id: String, error: impl Into<String>) -> Self {
        Self {
            event_id,
            success: false,
            affected: Vec::new(),
            error: Some(error.into()),
            processing_time_ms: 0,
            processed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Event statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStats {
    /// Total events queued
    pub total_queued: u64,
    /// Events completed
    pub completed: u64,
    /// Events failed
    pub failed: u64,
    /// Events pending
    pub pending: u64,
    /// Average processing time (ms)
    pub avg_processing_time_ms: f64,
    /// Events by type
    pub by_type: HashMap<String, u64>,
    /// Events by priority
    pub by_priority: HashMap<String, u64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EventStats {
    /// Create empty stats
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_queued: 0,
            completed: 0,
            failed: 0,
            pending: 0,
            avg_processing_time_ms: 0.0,
            by_type: HashMap::new(),
            by_priority: HashMap::new(),
        }
    }

    /// Update with a new event
    pub fn record_queued(&mut self, event: &ExternalEvent, priority: EventPriority) {
        self.total_queued += 1;
        self.pending += 1;

        let type_key = match event {
            ExternalEvent::AgentEvent(_) => "agent",
            ExternalEvent::EnvironmentEvent(_) => "environment",
            ExternalEvent::NarrativeEvent(_) => "narrative",
            ExternalEvent::CommunicationEvent(_) => "communication",
            ExternalEvent::ResourceEvent(_) => "resource",
            ExternalEvent::SystemEvent(_) => "system",
        };

        *self.by_type.entry(type_key.to_string()).or_insert(0) += 1;

        let priority_key = format!("{priority:?}").to_lowercase();
        *self.by_priority.entry(priority_key).or_insert(0) += 1;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record completion
    pub fn record_completed(&mut self, processing_time_ms: u64) {
        self.completed += 1;
        self.pending = self.pending.saturating_sub(1);

        // Update running average
        let n = self.completed as f64;
        self.avg_processing_time_ms =
            (self.avg_processing_time_ms * (n - 1.0) + processing_time_ms as f64) / n;
    }

    /// Record failure
    pub fn record_failed(&mut self) {
        self.failed += 1;
        self.pending = self.pending.saturating_sub(1);
    }
}

impl Default for EventStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Scenario definition for batch event injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario ID
    pub id: String,
    /// Scenario name
    pub name: String,
    /// Description
    pub description: String,
    /// Events in the scenario
    pub events: Vec<ScenarioEvent>,
    /// Preconditions
    pub preconditions: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// An event within a scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioEvent {
    /// Event to inject
    pub event: ExternalEvent,
    /// Delay after previous event (ticks)
    pub delay_ticks: u64,
    /// Priority
    pub priority: EventPriority,
    /// Condition to check before injection
    pub condition: Option<String>,
}

/// Event filter for querying events
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by status
    pub status: Option<EventStatus>,
    /// Filter by event type
    pub event_type: Option<String>,
    /// Filter by priority
    pub priority: Option<EventPriority>,
    /// Filter by source
    pub source: Option<String>,
    /// Time range start
    pub from_time: Option<String>,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Time range end
    pub to_time: Option<String>,
}

impl EventFilter {
    /// Check if a queued event matches this filter
    #[must_use]
    pub fn matches(&self, event: &QueuedEvent) -> bool {
        // Check status filter
        if let Some(ref status) = self.status {
            if event.status != *status {
                return false;
            }
        }

        // Check source filter
        if let Some(ref source) = self.source {
            if event.metadata.source != *source {
                return false;
            }
        }

        // Other filters would require additional fields on QueuedEvent
        // For now, return true if the basic filters pass
        true
    }
}
