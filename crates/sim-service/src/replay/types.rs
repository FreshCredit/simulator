//! Replay Types and Data Structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// A replay session containing all frames
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySession {
    /// Unique session ID
    pub session_id: String,
    /// Original run ID this replay is based on
    pub run_id: String,
    /// Experiment ID
    pub experiment_id: String,
    /// Session name/description
    pub name: String,
    /// When the replay was created
    pub created_at: String,
    /// Total number of ticks in replay
    pub total_ticks: u64,
    /// Current playback position
    pub current_tick: u64,
    /// Replay configuration
    pub config: ReplayConfig,
    /// Summary statistics
    pub stats: ReplayStats,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for replay playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayConfig {
    /// Playback speed multiplier (1.0 = normal)
    pub playback_speed: f64,
    /// Whether to loop playback
    pub loop_playback: bool,
    /// Start tick
    pub start_tick: u64,
    /// End tick (None = end of simulation)
    pub end_tick: Option<u64>,
    /// Entities to highlight
    pub highlight_entities: Vec<String>,
    /// Visualization options
    pub visualization: VisualizationOptions,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            playback_speed: 1.0,
            loop_playback: false,
            start_tick: 0,
            end_tick: None,
            highlight_entities: Vec::new(),
            visualization: VisualizationOptions::default(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[allow(clippy::struct_excessive_bools)]
/// Visualization options for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationOptions {
    /// Show agent trails
    pub show_trails: bool,
    /// Show agent vision/cones
    pub show_vision: bool,
    /// Show communication lines
    pub show_communication: bool,
    /// Show resource distribution
    pub show_resources: bool,
    /// Trail length (ticks)
    pub trail_length: u32,
    /// Color scheme
    pub color_scheme: String,
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            show_trails: true,
            show_vision: false,
            show_communication: false,
            show_resources: true,
            trail_length: 10,
            color_scheme: "default".to_string(),
        }
    }
}

/// Replay statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayStats {
    /// Total frames
    pub total_frames: u64,
    /// Total agents
    pub total_agents: u32,
    /// Total events
    pub total_events: u64,
    /// Average tick duration (ms)
    pub avg_tick_duration_ms: f64,
    /// Peak agent count
    pub peak_agent_count: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// A single frame in the replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFrame {
    /// Tick number
    pub tick: u64,
    /// Timestamp
    pub timestamp: String,
    /// Agent states
    pub agents: Vec<AgentState>,
    /// Environment state
    pub environment: EnvironmentState,
    /// Events that occurred this tick
    pub events: Vec<FrameEvent>,
    /// Metrics snapshot
    pub metrics: HashMap<String, f64>,
}

/// Agent state in a frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Agent ID
    pub agent_id: String,
    /// Position (x, y)
    pub position: (f64, f64),
    /// Velocity (vx, vy)
    pub velocity: (f64, f64),
    /// Orientation (radians)
    pub orientation: f64,
    /// Current state/action
    pub state: String,
    /// Energy level
    pub energy: f64,
    /// Health level
    pub health: f64,
    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Environment state in a frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    /// Time of day (0-24)
    pub time_of_day: f64,
    /// Weather condition
    pub weather: String,
    /// Temperature
    pub temperature: f64,
    /// Resources in environment
    pub resources: Vec<ResourceState>,
    /// Regions/zones
    pub regions: HashMap<String, RegionState>,
}

/// Resource state in environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceState {
    /// Resource ID
    pub resource_id: String,
    /// Resource type
    pub resource_type: String,
    /// Position
    pub position: (f64, f64),
    /// Amount available
    pub amount: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Region/zone state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionState {
    /// Region name
    pub name: String,
    /// Bounds (`min_x`, `min_y`, `max_x`, `max_y`)
    pub bounds: (f64, f64, f64, f64),
    /// Properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Event in a frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameEvent {
    /// Event type
    pub event_type: String,
    /// Involved agents
    pub agents: Vec<String>,
    /// Position where event occurred
    pub position: Option<(f64, f64)>,
    /// Event data
    pub data: serde_json::Value,
}

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackState {
    /// Playback is stopped
    Stopped,
    /// Playback is playing
    Playing,
    /// Playback is paused
    Paused,
    /// Playback completed
    Completed,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Bookmark in replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayBookmark {
    /// Bookmark ID
    pub id: String,
    /// Tick number
    pub tick: u64,
    /// Label/description
    pub label: String,
    /// Color for visualization
    pub color: String,
}

/// Replay comparison request
#[derive(Debug, Deserialize)]
pub struct CompareRunsRequest {
    /// Base run ID
    pub base_run_id: String,
    /// Comparison run ID
    pub compare_run_id: String,
    /// Start tick for comparison
    pub start_tick: Option<u64>,
    /// End tick for comparison
    pub end_tick: Option<u64>,
    /// Metrics to compare
    pub metrics: Option<Vec<String>>,
    /// Comparison granularity (ticks)
    pub granularity: Option<u64>,
}

/// Comparison result
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonResult {
    /// Base run ID
    pub base_run_id: String,
    /// Compare run ID
    pub compare_run_id: String,
    /// Overall similarity score (0.0 - 1.0)
    pub similarity_score: f64,
    /// Metric comparisons
    pub metric_comparisons: Vec<MetricComparison>,
    /// Significant differences
    pub significant_differences: Vec<Difference>,
    /// Divergence points (where runs significantly diverged)
    pub divergence_points: Vec<DivergencePoint>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Metric comparison between runs
#[derive(Debug, Clone, Serialize)]
pub struct MetricComparison {
    /// Metric name
    pub metric: String,
    /// Base run average
    pub base_avg: f64,
    /// Compare run average
    pub compare_avg: f64,
    /// Percentage difference
    pub percent_diff: f64,
    /// Correlation coefficient
    pub correlation: f64,
}

/// A detected difference
#[derive(Debug, Clone, Serialize)]
pub struct Difference {
    /// Tick where difference occurred
    pub tick: u64,
    /// Type of difference
    pub diff_type: String,
    /// Description
    pub description: String,
    /// Severity (0.0 - 1.0)
    pub severity: f64,
    /// Affected entities
    pub affected: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Point where runs diverged
#[derive(Debug, Clone, Serialize)]
pub struct DivergencePoint {
    /// Tick of divergence
    pub tick: u64,
    /// Description of what diverged
    pub description: String,
    /// Divergence magnitude
    pub magnitude: f64,
}

/// Replay export format
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// Json
    Json,
    /// Mp4
    Mp4,
    /// Gif
    Gif,
}

/// Export request
#[derive(Debug, Deserialize)]
pub struct ExportReplayRequest {
    /// Session ID
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub session_id: String,
    /// Export format
    pub format: ExportFormat,
    /// Start tick
    pub start_tick: Option<u64>,
    /// End tick
    pub end_tick: Option<u64>,
    /// Resolution (for video)
    pub resolution: Option<(u32, u32)>,
    /// Frame rate (for video)
    pub fps: Option<u32>,
}

/// Export response
#[derive(Debug, Serialize)]
pub struct ExportReplayResponse {
    /// Export ID
    pub export_id: String,
    /// Format
    pub format: String,
    /// Download URL
    pub download_url: String,
    /// Status
    pub status: String,
    /// Estimated completion time
    pub estimated_completion_secs: u64,
}
