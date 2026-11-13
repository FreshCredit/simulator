//! Health Metrics Types and Collection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of metrics that can be collected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    /// Counter - monotonically increasing value
    Counter,
    /// Gauge - can go up or down
    Gauge,
    /// Histogram - distribution of values
    Histogram,
    /// Timing - duration in milliseconds
    Timing,
}

/// A metric value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// name
    pub name: String,
    /// metric type
    pub metric_type: MetricType,
    /// value
    pub value: f64,
    /// unit
    pub unit: String,
    /// timestamp
    pub timestamp: String,
    /// labels
    pub labels: HashMap<String, String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MetricValue {
    /// Create a new gauge metric
    pub fn gauge(name: impl Into<String>, value: f64, unit: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Gauge,
            value,
            unit: unit.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            labels: HashMap::new(),
        }
    }

    /// Create a new counter metric
    pub fn counter(name: impl Into<String>, value: f64, unit: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Counter,
            value,
            unit: unit.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            labels: HashMap::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new timing metric
    pub fn timing(name: impl Into<String>, value_ms: f64) -> Self {
        Self {
            name: name.into(),
            metric_type: MetricType::Timing,
            value: value_ms,
            unit: "ms".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            labels: HashMap::new(),
        }
    }

    /// Add a label to this metric
    #[must_use]
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
}

/// Collection of metrics for a simulation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimulationMetrics {
    /// Simulation ID
    pub simulation_id: String,
    /// Timestamp when metrics were collected
    pub collected_at: String,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Resource utilization
    pub resources: ResourceMetrics,
    /// Agent statistics
    pub agents: AgentMetrics,
    /// Tick processing metrics
    pub ticks: TickMetrics,
    /// Error and warning counts
    pub errors: ErrorMetrics,
    /// Custom metrics
    pub custom: HashMap<String, f64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Performance-related metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Ticks per second
    pub ticks_per_second: f64,
    /// Average tick duration in ms
    pub avg_tick_duration_ms: f64,
    /// P95 tick duration in ms
    pub p95_tick_duration_ms: f64,
    /// P99 tick duration in ms
    pub p99_tick_duration_ms: f64,
    /// Total ticks processed
    pub total_ticks: u64,
    /// Simulation time vs real time ratio
    pub time_dilation: f64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_percent: f64,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// Memory usage percentage (0-100)
    pub memory_percent: f64,
    /// Disk I/O read bytes per second
    pub disk_read_bps: f64,
    /// Disk I/O write bytes per second
    pub disk_write_bps: f64,
    /// Network received bytes per second
    pub network_rx_bps: f64,
    /// Network transmitted bytes per second
    pub network_tx_bps: f64,
    /// Open file descriptors
    pub open_fds: u32,
    /// Thread count
    pub thread_count: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Agent-related metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Total number of agents
    pub total_count: u32,
    /// Active agents
    pub active_count: u32,
    /// Agents in each state
    pub state_distribution: HashMap<String, u32>,
    /// Average decision time in ms
    pub avg_decision_time_ms: f64,
    /// Agents with errors
    pub error_count: u32,
}

/// Tick processing metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TickMetrics {
    /// Current tick number
    pub current_tick: u64,
    /// Target tick number (for bounded simulations)
    pub target_tick: Option<u64>,
    /// Progress percentage (0-100)
    pub progress_percent: f64,
    /// Estimated time remaining in seconds
    pub estimated_remaining_secs: Option<u64>,
    /// Ticks processed in last minute
    pub ticks_last_minute: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Error and warning metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total errors since start
    pub total_errors: u32,
    /// Total warnings since start
    pub total_warnings: u32,
    /// Errors in last minute
    pub errors_last_minute: u32,
    /// Warnings in last minute
    pub warnings_last_minute: u32,
    /// Last error message
    pub last_error: Option<String>,
    /// Last warning message
    pub last_warning: Option<String>,
}

/// Time-series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// timestamp
    pub timestamp: String,
    /// value
    pub value: f64,
}

/// Time-series metric for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    /// name
    pub name: String,
    /// unit
    pub unit: String,
    /// data
    pub data: Vec<TimeSeriesPoint>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Dashboard data aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// Overall system health
    pub system_health: String,
    /// Active simulations count
    pub active_simulations: u32,
    /// Total agents across all simulations
    pub total_agents: u32,
    /// System-wide metrics
    pub system_metrics: SystemMetrics,
    /// Per-simulation health data
    pub simulations: Vec<SimulationHealthSummary>,
    /// Recent alerts
    pub recent_alerts: Vec<Alert>,
}

/// System-wide aggregated metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Average CPU usage across all simulations
    pub avg_cpu_percent: f64,
    /// Total memory usage in MB
    pub total_memory_mb: f64,
    /// Total ticks processed per second
    pub total_ticks_per_second: f64,
    /// Total error rate (errors per minute)
    pub total_error_rate: f64,
}

/// Summary of a simulation's health for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationHealthSummary {
    /// simulation id
    pub simulation_id: String,
    /// status
    pub status: String,
    /// progress percent
    pub progress_percent: f64,
    /// agents count
    pub agents_count: u32,
    /// cpu percent
    pub cpu_percent: f64,
    /// memory mb
    pub memory_mb: f64,
    /// errors last minute
    pub errors_last_minute: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    /// Info
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

/// Health alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// id
    pub id: String,
    /// severity
    pub severity: AlertSeverity,
    /// message
    pub message: String,
    /// simulation id
    pub simulation_id: Option<String>,
    /// metric name
    pub metric_name: String,
    /// threshold
    pub threshold: f64,
    /// actual value
    pub actual_value: f64,
    /// created at
    pub created_at: String,
    /// acknowledged
    pub acknowledged: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Threshold configuration for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThreshold {
    /// metric name
    pub metric_name: String,
    /// warning threshold
    pub warning_threshold: f64,
    /// error threshold
    pub error_threshold: f64,
    /// critical threshold
    pub critical_threshold: f64,
    /// Comparison: "above" or "below"
    pub comparison: String,
}

impl AlertThreshold {
    /// Check if a value triggers an alert
    #[must_use]
    pub fn check(&self, value: f64) -> Option<AlertSeverity> {
        let triggered = match self.comparison.as_str() {
            "below" => value < self.critical_threshold,
            _ => value > self.critical_threshold,
        };

        if triggered {
            return Some(AlertSeverity::Critical);
        }

        let triggered = match self.comparison.as_str() {
            "below" => value < self.error_threshold,
            _ => value > self.error_threshold,
        };

        if triggered {
            return Some(AlertSeverity::Error);
        }

        let triggered = match self.comparison.as_str() {
            "below" => value < self.warning_threshold,
            _ => value > self.warning_threshold,
        };

        if triggered {
            return Some(AlertSeverity::Warning);
        }

        None
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Default thresholds for common metrics
#[must_use]
pub fn default_thresholds() -> Vec<AlertThreshold> {
    vec![
        AlertThreshold {
            metric_name: "cpu_percent".to_string(),
            warning_threshold: 70.0,
            error_threshold: 85.0,
            critical_threshold: 95.0,
            comparison: "above".to_string(),
        },
        AlertThreshold {
            metric_name: "memory_percent".to_string(),
            warning_threshold: 75.0,
            error_threshold: 85.0,
            critical_threshold: 95.0,
            comparison: "above".to_string(),
        },
        AlertThreshold {
            metric_name: "ticks_per_second".to_string(),
            warning_threshold: 10.0,
            error_threshold: 5.0,
            critical_threshold: 1.0,
            comparison: "below".to_string(),
        },
        AlertThreshold {
            metric_name: "errors_last_minute".to_string(),
            warning_threshold: 5.0,
            error_threshold: 20.0,
            critical_threshold: 50.0,
            comparison: "above".to_string(),
        },
    ]
}
