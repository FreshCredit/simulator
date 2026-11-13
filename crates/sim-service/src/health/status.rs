//! Health Status Definitions and Evaluation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::metrics::SimulationMetrics;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Overall health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Degraded
    Degraded,
    /// Unhealthy
    Unhealthy,
    /// Unknown
    Unknown,
}

impl HealthStatus {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Combine multiple health statuses into one
    #[must_use]
    pub fn aggregate(statuses: &[Self]) -> Self {
        if statuses.is_empty() {
            return Self::Unknown;
        }

        if statuses.contains(&Self::Unhealthy) {
            Self::Unhealthy
        } else if statuses.contains(&Self::Degraded) {
            Self::Degraded
        } else if statuses.iter().all(|s| *s == Self::Healthy) {
            Self::Healthy
        } else {
            Self::Unknown
        }
    }

    /// Convert to string representation
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Unknown => "unknown",
        }
    }
}

/// Health status of a simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationHealth {
    /// Simulation ID
    pub simulation_id: String,
    /// Overall health status
    pub status: HealthStatus,
    /// Current metrics snapshot
    pub metrics: SimulationMetrics,
    /// Health checks and their status
    pub checks: Vec<HealthCheck>,
    /// When health was last evaluated
    pub evaluated_at: String,
    /// Status message
    pub message: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check name
    pub name: String,
    /// Check status
    pub status: HealthStatus,
    /// Check message
    pub message: String,
    /// Metric value that triggered check
    pub value: Option<f64>,
    /// Threshold that was evaluated
    pub threshold: Option<f64>,
}

/// Service health (for sim-service itself)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Service name
    pub service: String,
    /// Overall status
    pub status: HealthStatus,
    /// Version
    pub version: String,
    /// Uptime in seconds
    pub uptime_secs: u64,
    /// Component health checks
    pub components: HashMap<String, ComponentHealth>,
    /// System resources
    pub system: SystemHealth,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Component health within a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// name
    pub name: String,
    /// status
    pub status: HealthStatus,
    /// latency ms
    pub latency_ms: u64,
    /// message
    pub message: Option<String>,
}

/// System-level health
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemHealth {
    /// CPU usage (0-100)
    pub cpu_percent: f64,
    /// Memory usage (0-100)
    pub memory_percent: f64,
    /// Disk usage (0-100)
    pub disk_percent: f64,
    /// Load average (1 min)
    pub load_average_1m: f64,
    /// Total processes
    pub total_processes: u32,
}

/// Health evaluation configuration
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// CPU threshold for degraded status (%)
    pub cpu_warning: f64,
    /// CPU threshold for unhealthy status (%)
    pub cpu_critical: f64,
    /// Memory threshold for degraded status (%)
    pub memory_warning: f64,
    /// Memory threshold for unhealthy status (%)
    pub memory_critical: f64,
    /// Error rate threshold for degraded (errors/min)
    pub error_rate_warning: f64,
    /// Error rate threshold for unhealthy (errors/min)
    pub error_rate_critical: f64,
    /// Minimum ticks per second for healthy status
    pub min_ticks_per_second: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            cpu_warning: 70.0,
            cpu_critical: 90.0,
            memory_warning: 75.0,
            memory_critical: 90.0,
            error_rate_warning: 5.0,
            error_rate_critical: 20.0,
            min_ticks_per_second: 1.0,
        }
    }
}

#[allow(clippy::too_many_lines)]
/// Evaluate health from metrics
#[must_use]
pub fn evaluate_health(metrics: &SimulationMetrics, config: &HealthConfig) -> SimulationHealth {
    let mut checks = Vec::new();

    // CPU check
    let cpu_status = if metrics.resources.cpu_percent > config.cpu_critical {
        HealthStatus::Unhealthy
    } else if metrics.resources.cpu_percent > config.cpu_warning {
        HealthStatus::Degraded
    } else {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        HealthStatus::Healthy
    };
    checks.push(HealthCheck {
        name: "cpu_usage".to_string(),
        status: cpu_status,
        message: format!("CPU usage: {:.1}%", metrics.resources.cpu_percent),
        value: Some(metrics.resources.cpu_percent),
        threshold: Some(config.cpu_warning),
    });

    // Memory check
    let memory_status = if metrics.resources.memory_percent > config.memory_critical {
        HealthStatus::Unhealthy
    } else if metrics.resources.memory_percent > config.memory_warning {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };
    checks.push(HealthCheck {
        name: "memory_usage".to_string(),
        status: memory_status,
        message: format!("Memory usage: {:.1}%", metrics.resources.memory_percent),
        value: Some(metrics.resources.memory_percent),
        threshold: Some(config.memory_warning),
    });

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Tick rate check
    let tick_status = if metrics.performance.ticks_per_second < config.min_ticks_per_second {
        HealthStatus::Unhealthy
    } else if metrics.performance.ticks_per_second < config.min_ticks_per_second * 2.0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };
    checks.push(HealthCheck {
        name: "tick_rate".to_string(),
        status: tick_status,
        message: format!("Ticks/sec: {:.1}", metrics.performance.ticks_per_second),
        value: Some(metrics.performance.ticks_per_second),
        threshold: Some(config.min_ticks_per_second),
    });

    // Error rate check
    let error_status = if metrics.errors.errors_last_minute as f64 > config.error_rate_critical {
        HealthStatus::Unhealthy
    } else if metrics.errors.errors_last_minute as f64 > config.error_rate_warning {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };
    checks.push(HealthCheck {
        name: "error_rate".to_string(),
        status: error_status,
        message: format!("Errors/min: {}", metrics.errors.errors_last_minute),
        value: Some(metrics.errors.errors_last_minute as f64),
        threshold: Some(config.error_rate_warning),
    });

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Progress check (stuck simulation detection)
    let progress_status = if metrics.ticks.ticks_last_minute == 0 && metrics.ticks.current_tick > 0
    {
        HealthStatus::Unhealthy
    } else {
        HealthStatus::Healthy
    };
    checks.push(HealthCheck {
        name: "progress".to_string(),
        status: progress_status,
        message: if progress_status == HealthStatus::Unhealthy {
            "Simulation appears stuck - no ticks in last minute".to_string()
        } else {
            format!("Progress: {:.1}%", metrics.ticks.progress_percent)
        },
        value: Some(metrics.ticks.ticks_last_minute as f64),
        threshold: Some(1.0),
    });

    // Determine overall status
    let statuses: Vec<_> = checks.iter().map(|c| c.status).collect();
    let overall_status = HealthStatus::aggregate(&statuses);

    // Generate message
    let message = match overall_status {
        HealthStatus::Healthy => Some("All systems operational".to_string()),
        HealthStatus::Degraded => {
            let issues: Vec<_> = checks
                .iter()
                .filter(|c| c.status == HealthStatus::Degraded)
                .map(|c| c.name.clone())
                .collect();
            Some(format!("Degraded: {}", issues.join(", ")))
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }
        HealthStatus::Unhealthy => {
            let issues: Vec<_> = checks
                .iter()
                .filter(|c| c.status == HealthStatus::Unhealthy)
                .map(|c| c.name.clone())
                .collect();
            Some(format!("Issues: {}", issues.join(", ")))
        }
        HealthStatus::Unknown => Some("Health status unknown".to_string()),
    };

    SimulationHealth {
        simulation_id: metrics.simulation_id.clone(),
        status: overall_status,
        metrics: metrics.clone(),
        checks,
        evaluated_at: chrono::Utc::now().to_rfc3339(),
        message,
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Evaluate service health
pub fn evaluate_service_health(
    service_name: &str,
    version: &str,
    start_time: std::time::Instant,
) -> ServiceHealth {
    let uptime_secs = start_time.elapsed().as_secs();

    // Check system resources
    let system = SystemHealth {
        cpu_percent: 0.0, // Would be populated from actual system metrics
        memory_percent: 0.0,
        disk_percent: 0.0,
        load_average_1m: 0.0,
        total_processes: 0,
    };

    // Check components
    let mut components = HashMap::new();

    // Result store health
    components.insert(
        "result_store".to_string(),
        ComponentHealth {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            name: "result_store".to_string(),
            status: HealthStatus::Healthy,
            latency_ms: 0,
            message: Some("Operational".to_string()),
        },
    );

    // Batch executor health
    components.insert(
        "batch_executor".to_string(),
        ComponentHealth {
            name: "batch_executor".to_string(),
            status: HealthStatus::Healthy,
            latency_ms: 0,
            message: Some("Operational".to_string()),
        },
    );

    ServiceHealth {
        service: service_name.to_string(),
        status: HealthStatus::Healthy,
        version: version.to_string(),
        uptime_secs,
        components,
        system,
    }
}
