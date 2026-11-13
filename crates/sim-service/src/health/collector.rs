//! Health Metrics Collector
//!
//! Collects and aggregates health metrics from simulations

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info, trace, warn};

use super::metrics::{
    Alert, AlertSeverity, AlertThreshold, DashboardData, SimulationHealthSummary,
    SimulationMetrics, SystemMetrics, TimeSeries, TimeSeriesPoint,
};
use super::status::{evaluate_health, HealthConfig, HealthStatus, SimulationHealth};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Collector for health metrics
#[derive(Clone, Debug)]
pub struct HealthCollector {
    /// Current metrics for all simulations
    metrics: Arc<RwLock<HashMap<String, SimulationMetrics>>>,
    /// Historical time-series data
    time_series: Arc<RwLock<HashMap<String, Vec<TimeSeriesPoint>>>>,
    /// Active alerts
    alerts: Arc<RwLock<Vec<Alert>>>,
    /// Alert thresholds
    thresholds: Arc<RwLock<Vec<AlertThreshold>>>,
    /// Health configuration
    config: HealthConfig,
    /// Maximum history size per metric
    max_history: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl HealthCollector {
    /// Create a new health collector
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(HealthConfig::default())
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(config: HealthConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            time_series: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            thresholds: Arc::new(RwLock::new(super::metrics::default_thresholds())),
            config,
            max_history: 1440, // 24 hours at 1 sample/minute
        }
    }

    /// Start the background collection loop
    pub fn start_collection_loop(&self) {
        let collector = self.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            // P10-R4: intentionally unbounded service loop — metrics collection runs for the process lifetime; the spawned task is aborted at shutdown.
            loop {
                interval.tick().await;
                collector.collect_system_metrics();
                collector.evaluate_alerts().await;
                collector.cleanup_old_data().await;
            }
        });

        info!("Health collection loop started");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Submit metrics for a simulation
    pub async fn submit_metrics(&self, metrics: SimulationMetrics) {
        let sim_id = metrics.simulation_id.clone();
        let timestamp = metrics.collected_at.clone();

        // Update current metrics
        {
            let mut guard = self.metrics.write().await;
            guard.insert(sim_id.clone(), metrics.clone());
        }

        // Add to time series
        {
            let mut ts_guard = self.time_series.write().await;

            // CPU time series
            let cpu_key = format!("{sim_id}/cpu");
            let cpu_point = TimeSeriesPoint {
                timestamp: timestamp.clone(),
                value: metrics.resources.cpu_percent,
            };
            ts_guard.entry(cpu_key).or_default().push(cpu_point);

            // Memory time series
            let mem_key = format!("{sim_id}/memory");
            let mem_point = TimeSeriesPoint {
                timestamp: timestamp.clone(),
                value: metrics.resources.memory_percent,
            };
            ts_guard.entry(mem_key).or_default().push(mem_point);

            // TPS time series
            let tps_key = format!("{sim_id}/tps");
            let tps_point = TimeSeriesPoint {
                timestamp,
                value: metrics.performance.ticks_per_second,
            };
            ts_guard.entry(tps_key).or_default().push(tps_point);
        }

        trace!(simulation_id = %sim_id, "Metrics submitted");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Remove metrics for a simulation
    pub async fn remove_simulation(&self, simulation_id: &str) {
        let mut guard = self.metrics.write().await;
        guard.remove(simulation_id);

        let mut ts_guard = self.time_series.write().await;
        ts_guard.retain(|k, _| !k.starts_with(simulation_id));

        info!(simulation_id = %simulation_id, "Simulation metrics removed");
    }

    /// Get current metrics for a simulation
    pub async fn get_metrics(&self, simulation_id: &str) -> Option<SimulationMetrics> {
        let guard = self.metrics.read().await;
        guard.get(simulation_id).cloned()
    }

    /// Get health status for a simulation
    pub async fn get_health(&self, simulation_id: &str) -> Option<SimulationHealth> {
        let metrics = self.get_metrics(simulation_id).await?;
        Some(evaluate_health(&metrics, &self.config))
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get all current health statuses
    pub async fn get_all_health(&self) -> Vec<SimulationHealth> {
        let guard = self.metrics.read().await;
        guard
            .values()
            .map(|m| evaluate_health(m, &self.config))
            .collect()
    }

    /// Get time series data for a simulation
    pub async fn get_time_series(
        &self,
        simulation_id: &str,
        metric: &str,
        limit: Option<usize>,
    ) -> Option<TimeSeries> {
        let key = format!("{simulation_id}/{metric}");
        let guard = self.time_series.read().await;

        guard.get(&key).map(|data| {
            let limited_data = match limit {
                Some(n) => data.iter().rev().take(n).cloned().rev().collect(),
                None => data.clone(),
            };

            TimeSeries {
                name: metric.to_string(),
                unit: get_metric_unit(metric).to_string(),
                data: limited_data,
            }
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get dashboard data
    pub async fn get_dashboard_data(&self) -> DashboardData {
        let health_list = self.get_all_health().await;

        let active_simulations = health_list.len() as u32;

        let total_agents: u32 = health_list
            .iter()
            .map(|h| h.metrics.agents.total_count)
            .sum();

        let system_health = if health_list
            .iter()
            .any(|h| h.status == HealthStatus::Unhealthy)
        {
            "unhealthy"
        } else if health_list
            .iter()
            .any(|h| h.status == HealthStatus::Degraded)
        {
            "degraded"
        } else {
            "healthy"
        }
        .to_string();

        // Aggregate system metrics
        let system_metrics = if health_list.is_empty() {
            SystemMetrics::default()
        } else {
            let count = health_list.len() as f64;
            SystemMetrics {
                avg_cpu_percent: health_list
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    .iter()
                    .map(|h| h.metrics.resources.cpu_percent)
                    .sum::<f64>()
                    / count,
                total_memory_mb: health_list
                    .iter()
                    .map(|h| h.metrics.resources.memory_mb)
                    .sum(),
                total_ticks_per_second: health_list
                    .iter()
                    .map(|h| h.metrics.performance.ticks_per_second)
                    .sum(),
                total_error_rate: health_list
                    .iter()
                    .map(|h| h.metrics.errors.errors_last_minute as f64)
                    .sum(),
            }
        };

        // Create simulation summaries
        let simulations: Vec<_> = health_list
            .into_iter()
            .map(|h| SimulationHealthSummary {
                simulation_id: h.simulation_id,
                status: h.status.as_str().to_string(),
                progress_percent: h.metrics.ticks.progress_percent,
                agents_count: h.metrics.agents.total_count,
                cpu_percent: h.metrics.resources.cpu_percent,
                memory_mb: h.metrics.resources.memory_mb,
                errors_last_minute: h.metrics.errors.errors_last_minute,
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Get recent alerts
        let alerts_guard = self.alerts.read().await;
        let recent_alerts: Vec<_> = alerts_guard.iter().rev().take(10).cloned().collect();
        drop(alerts_guard);

        DashboardData {
            system_health,
            active_simulations,
            total_agents,
            system_metrics,
            simulations,
            recent_alerts,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get active alerts
    pub async fn get_alerts(&self, include_acknowledged: bool) -> Vec<Alert> {
        let guard = self.alerts.read().await;
        guard
            .iter()
            .filter(|a| include_acknowledged || !a.acknowledged)
            .cloned()
            .collect()
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> bool {
        let mut guard = self.alerts.write().await;
        if let Some(alert) = guard.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            true
        } else {
            false
        }
    }

    /// Update alert thresholds
    pub async fn set_thresholds(&self, thresholds: Vec<AlertThreshold>) {
        let mut guard = self.thresholds.write().await;
        *guard = thresholds;
    }

    /// Get current thresholds
    pub async fn get_thresholds(&self) -> Vec<AlertThreshold> {
        let guard = self.thresholds.read().await;
        guard.clone()
    }

    /// Collect system metrics (placeholder - would integrate with actual system monitoring)
    fn collect_system_metrics(&self) {
        // In a real implementation, this would:
        // 1. Query system CPU usage
        // 2. Query memory usage
        // 3. Query disk I/O
        // 4. Update metrics for active simulations

        debug!("Collecting system metrics");
    }

    /// Evaluate alerts based on current metrics
    async fn evaluate_alerts(&self) {
        let metrics_guard = self.metrics.read().await;
        let thresholds_guard = self.thresholds.read().await;
        let mut alerts_guard = self.alerts.write().await;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for (sim_id, metrics) in metrics_guard.iter() {
            Self::check_metric_alert(
                &mut alerts_guard,
                sim_id,
                &thresholds_guard,
                "cpu_percent",
                metrics.resources.cpu_percent,
                |v| warn!(simulation_id = %sim_id, cpu = %v, "CPU alert triggered"),
            );
            Self::check_metric_alert(
                &mut alerts_guard,
                sim_id,
                &thresholds_guard,
                "memory_percent",
                metrics.resources.memory_percent,
                |v| warn!(simulation_id = %sim_id, memory = %v, "Memory alert triggered"),
            );
            let error_count = metrics.errors.errors_last_minute as f64;
            Self::check_metric_alert(
                &mut alerts_guard,
                sim_id,
                &thresholds_guard,
                "errors_last_minute",
                error_count,
                |v| warn!(simulation_id = %sim_id, errors = %v, "Error rate alert triggered"),
            );
        }

        // Limit total alerts
        while alerts_guard.len() > 1000 {
            alerts_guard.remove(0);
        }
    }

    /// Check a single metric against its threshold and push an alert if breached.
    fn check_metric_alert(
        alerts: &mut Vec<Alert>,
        sim_id: &str,
        thresholds: &[AlertThreshold],
        metric_name: &str,
        value: f64,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        on_trigger: impl FnOnce(f64),
    ) {
        let Some(threshold) = thresholds.iter().find(|t| t.metric_name == metric_name) else {
            return;
        };
        let Some(severity) = threshold.check(value) else {
            return;
        };
        let alert = create_alert(
            sim_id,
            metric_name,
            severity,
            value,
            threshold.critical_threshold,
        );
        if !alerts.iter().any(|a| {
            a.simulation_id.as_deref() == Some(sim_id)
                && a.metric_name == metric_name
                && !a.acknowledged
        }) {
            alerts.push(alert);
            on_trigger(value);
        }
    }

    /// Clean up old time series data
    async fn cleanup_old_data(&self) {
        let mut guard = self.time_series.write().await;

        for series in guard.values_mut() {
            if series.len() > self.max_history {
                let excess = series.len() - self.max_history;
                series.drain(0..excess);
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for HealthCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Create an alert
fn create_alert(
    simulation_id: &str,
    metric_name: &str,
    severity: AlertSeverity,
    actual_value: f64,
    threshold: f64,
) -> Alert {
    let message = format!(
        "{} alert: {} = {:.1} (threshold: {:.1})",
        metric_name,
        severity_str(&severity),
        actual_value,
        threshold
    );

    Alert {
        id: uuid::Uuid::new_v4().to_string(),
        severity,
        message,
        simulation_id: Some(simulation_id.to_string()),
        metric_name: metric_name.to_string(),
        threshold,
        actual_value,
        created_at: chrono::Utc::now().to_rfc3339(),
        acknowledged: false,
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
const fn severity_str(severity: &AlertSeverity) -> &'static str {
    match severity {
        AlertSeverity::Info => "Info",
        AlertSeverity::Warning => "Warning",
        AlertSeverity::Error => "Error",
        AlertSeverity::Critical => "Critical",
    }
}

fn get_metric_unit(metric: &str) -> &'static str {
    match metric {
        "cpu" => "%",
        "memory" => "%",
        "tps" => "ticks/s",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::super::metrics::{
        AgentMetrics, ErrorMetrics, PerformanceMetrics, ResourceMetrics, TickMetrics,
    };
    use super::*;

    #[tokio::test]
    async fn test_submit_and_get_metrics() {
        let collector = HealthCollector::new();

        let metrics = SimulationMetrics {
            simulation_id: "sim-1".to_string(),
            collected_at: chrono::Utc::now().to_rfc3339(),
            performance: PerformanceMetrics::default(),
            resources: ResourceMetrics::default(),
            agents: AgentMetrics::default(),
            ticks: TickMetrics::default(),
            errors: ErrorMetrics::default(),
            custom: HashMap::new(),
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        collector.submit_metrics(metrics).await;

        let retrieved = collector.get_metrics("sim-1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().simulation_id, "sim-1");
    }

    #[tokio::test]
    async fn test_health_evaluation() {
        let config = HealthConfig {
            cpu_warning: 50.0,
            cpu_critical: 80.0,
            min_ticks_per_second: 1.0,
            ..Default::default()
        };
        let collector = HealthCollector::with_config(config);

        let metrics = SimulationMetrics {
            simulation_id: "sim-2".to_string(),
            collected_at: chrono::Utc::now().to_rfc3339(),
            performance: super::super::metrics::PerformanceMetrics {
                ticks_per_second: 10.0, // Healthy tick rate
                ..super::super::metrics::PerformanceMetrics::default()
            },
            resources: super::super::metrics::ResourceMetrics {
                cpu_percent: 60.0,    // Above warning, below critical
                memory_percent: 50.0, // Healthy
                ..super::super::metrics::ResourceMetrics::default()
            },
            agents: AgentMetrics::default(),
            ticks: super::super::metrics::TickMetrics {
                current_tick: 100,
                ticks_last_minute: 600, // Making progress
                ..super::super::metrics::TickMetrics::default()
            },
            errors: super::super::metrics::ErrorMetrics {
                errors_last_minute: 0,
                ..super::super::metrics::ErrorMetrics::default()
            },
            custom: HashMap::new(),
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        collector.submit_metrics(metrics).await;

        let health = collector.get_health("sim-2").await.unwrap();
        assert_eq!(health.status, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_dashboard_data() {
        let collector = HealthCollector::new();

        let metrics = SimulationMetrics {
            simulation_id: "sim-3".to_string(),
            collected_at: chrono::Utc::now().to_rfc3339(),
            performance: PerformanceMetrics::default(),
            resources: ResourceMetrics::default(),
            agents: super::super::metrics::AgentMetrics {
                total_count: 1000,
                ..super::super::metrics::AgentMetrics::default()
            },
            ticks: TickMetrics::default(),
            errors: ErrorMetrics::default(),
            custom: HashMap::new(),
        };

        collector.submit_metrics(metrics).await;

        let dashboard = collector.get_dashboard_data().await;
        assert_eq!(dashboard.active_simulations, 1);
        assert_eq!(dashboard.total_agents, 1000);
    }
}
