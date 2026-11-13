//! Health Dashboard API Module (P1-SIM-3)
//!
//! Provides REST endpoints for health monitoring and dashboard data.
//!
//! ## Routes
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/health` | Service health check |
//! | GET | `/api/health/dashboard` | Full dashboard data |
//! | GET | `/api/health/simulations` | All simulation health |
//! | GET | `/api/health/simulations/:id` | Specific simulation health |
//! | GET | `/api/health/simulations/:id/metrics` | Raw metrics |
//! | GET | `/api/health/simulations/:id/timeseries` | Time series data |
//! | POST | `/api/health/simulations/:id/metrics` | Submit metrics |
//! | GET | `/api/health/alerts` | List alerts |
//! | POST | `/api/health/alerts/:id/ack` | Acknowledge alert |
//! | GET | `/api/health/thresholds` | Get alert thresholds |
//! | PUT | `/api/health/thresholds` | Update thresholds |

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::sse::Event,
    response::{sse::Sse, IntoResponse},
    Json,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::health::collector::HealthCollector;
use crate::health::metrics::{AlertThreshold, DashboardData, SimulationMetrics, TimeSeries};
use crate::health::status::evaluate_service_health;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// State for health endpoints
#[derive(Clone, Debug)]
pub struct HealthState {
    /// collector
    pub collector: Arc<HealthCollector>,
    /// service start
    pub service_start: std::time::Instant,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthCheckResponse {
    /// status
    pub status: String,
    /// service
    pub service: String,
    /// version
    pub version: String,
    /// uptime secs
    pub uptime_secs: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Simulation health list response
#[derive(Debug, Serialize)]
pub struct SimulationHealthListResponse {
    /// simulations
    pub simulations: Vec<SimulationHealthSummary>,
}

/// Simulation health summary for API
#[derive(Debug, Serialize)]
pub struct SimulationHealthSummary {
    /// simulation id
    pub simulation_id: String,
    /// status
    pub status: String,
    /// message
    pub message: Option<String>,
    /// progress percent
    pub progress_percent: f64,
    /// agents count
    pub agents_count: u32,
    /// cpu percent
    pub cpu_percent: f64,
    /// memory mb
    pub memory_mb: f64,
    /// ticks per second
    pub ticks_per_second: f64,
    /// checks
    pub checks: Vec<HealthCheckView>,
}

/// Health check view
#[derive(Debug, Serialize)]
pub struct HealthCheckView {
    /// name
    pub name: String,
    /// status
    pub status: String,
    /// message
    pub message: String,
    /// value
    pub value: Option<f64>,
    /// threshold
    pub threshold: Option<f64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl From<crate::health::status::HealthCheck> for HealthCheckView {
    fn from(c: crate::health::status::HealthCheck) -> Self {
        Self {
            name: c.name,
            status: c.status.as_str().to_string(),
            message: c.message,
            value: c.value,
            threshold: c.threshold,
        }
    }
}

/// Alerts list response
#[derive(Debug, Serialize)]
pub struct AlertsResponse {
    /// alerts
    pub alerts: Vec<AlertView>,
}

/// Alert view
#[derive(Debug, Serialize)]
pub struct AlertView {
    /// id
    pub id: String,
    /// severity
    pub severity: String,
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
impl From<crate::health::metrics::Alert> for AlertView {
    fn from(a: crate::health::metrics::Alert) -> Self {
        Self {
            id: a.id,
            severity: format!("{:?}", a.severity).to_lowercase(),
            message: a.message,
            simulation_id: a.simulation_id,
            metric_name: a.metric_name,
            threshold: a.threshold,
            actual_value: a.actual_value,
            created_at: a.created_at,
            acknowledged: a.acknowledged,
        }
    }
}

/// Thresholds response
#[derive(Debug, Serialize)]
pub struct ThresholdsResponse {
    /// thresholds
    pub thresholds: Vec<AlertThreshold>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Update thresholds request
#[derive(Debug, Deserialize)]
pub struct UpdateThresholdsRequest {
    /// thresholds
    pub thresholds: Vec<AlertThreshold>,
}

/// Time series query params
#[derive(Debug, Deserialize)]
pub struct TimeSeriesQuery {
    /// metric
    pub metric: String,
    /// limit
    pub limit: Option<usize>,
}

/// Metrics query params
#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    /// include acknowledged
    pub include_acknowledged: Option<bool>,
}

/// GET /api/health - Service health check
pub async fn get_health(State(state): State<HealthState>) -> Json<HealthCheckResponse> {
    let uptime_secs = state.service_start.elapsed().as_secs();

    Json(HealthCheckResponse {
        status: "healthy".to_string(),
        service: "sim-service".to_string(),
        version: crate::VERSION.to_string(),
        uptime_secs,
    })
}

/// GET /api/health/dashboard - Full dashboard data
pub async fn get_dashboard(State(state): State<HealthState>) -> Json<DashboardData> {
    let dashboard = state.collector.get_dashboard_data().await;
    Json(dashboard)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/health/simulations - All simulation health
pub async fn get_simulations_health(
    State(state): State<HealthState>,
) -> Json<SimulationHealthListResponse> {
    let health_list = state.collector.get_all_health().await;

    let summaries: Vec<_> = health_list
        .into_iter()
        .map(|h| SimulationHealthSummary {
            simulation_id: h.simulation_id,
            status: h.status.as_str().to_string(),
            message: h.message,
            progress_percent: h.metrics.ticks.progress_percent,
            agents_count: h.metrics.agents.total_count,
            cpu_percent: h.metrics.resources.cpu_percent,
            memory_mb: h.metrics.resources.memory_mb,
            ticks_per_second: h.metrics.performance.ticks_per_second,
            checks: h.checks.into_iter().map(Into::into).collect(),
        })
        .collect();

    Json(SimulationHealthListResponse {
        simulations: summaries,
    })
}

/// GET /api/health/simulations/:id - Specific simulation health
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_simulation_health(
    State(state): State<HealthState>,
    Path(simulation_id): Path<String>,
) -> Result<Json<SimulationHealthSummary>, StatusCode> {
    match state.collector.get_health(&simulation_id).await {
        Some(health) => Ok(Json(SimulationHealthSummary {
            simulation_id: health.simulation_id,
            status: health.status.as_str().to_string(),
            message: health.message,
            progress_percent: health.metrics.ticks.progress_percent,
            agents_count: health.metrics.agents.total_count,
            cpu_percent: health.metrics.resources.cpu_percent,
            memory_mb: health.metrics.resources.memory_mb,
            ticks_per_second: health.metrics.performance.ticks_per_second,
            checks: health.checks.into_iter().map(Into::into).collect(),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/health/simulations/:id/metrics - Raw metrics
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_simulation_metrics(
    State(state): State<HealthState>,
    Path(simulation_id): Path<String>,
) -> Result<Json<SimulationMetrics>, StatusCode> {
    match state.collector.get_metrics(&simulation_id).await {
        Some(metrics) => Ok(Json(metrics)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// POST /api/health/simulations/:id/metrics - Submit metrics
pub async fn submit_simulation_metrics(
    State(state): State<HealthState>,
    Path(simulation_id): Path<String>,
    Json(mut metrics): Json<SimulationMetrics>,
) -> StatusCode {
    // Ensure simulation_id matches path
    metrics.simulation_id = simulation_id;
    metrics.collected_at = chrono::Utc::now().to_rfc3339();

    state.collector.submit_metrics(metrics).await;

    StatusCode::CREATED
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// DELETE /api/health/simulations/:id - Remove simulation from monitoring
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn remove_simulation(
    State(state): State<HealthState>,
    Path(simulation_id): Path<String>,
) -> StatusCode {
    state.collector.remove_simulation(&simulation_id).await;
    StatusCode::NO_CONTENT
}

/// GET /api/health/simulations/:id/timeseries - Time series data
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_time_series(
    State(state): State<HealthState>,
    Path(simulation_id): Path<String>,
    Query(query): Query<TimeSeriesQuery>,
) -> Result<Json<TimeSeries>, StatusCode> {
    match state
        .collector
        .get_time_series(&simulation_id, &query.metric, query.limit)
        .await
    {
        Some(ts) => Ok(Json(ts)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/health/alerts - List alerts
pub async fn get_alerts(
    State(state): State<HealthState>,
    Query(query): Query<AlertsQuery>,
) -> Json<AlertsResponse> {
    let include_acknowledged = query.include_acknowledged.unwrap_or(false);
    let alerts = state.collector.get_alerts(include_acknowledged).await;

    Json(AlertsResponse {
        alerts: alerts.into_iter().map(Into::into).collect(),
    })
}

/// POST /api/health/alerts/:id/ack - Acknowledge alert
pub async fn acknowledge_alert(
    State(state): State<HealthState>,
    Path(alert_id): Path<String>,
) -> StatusCode {
    if state.collector.acknowledge_alert(&alert_id).await {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// GET /api/health/thresholds - Get alert thresholds
pub async fn get_thresholds(State(state): State<HealthState>) -> Json<ThresholdsResponse> {
    let thresholds = state.collector.get_thresholds().await;
    Json(ThresholdsResponse { thresholds })
}

/// PUT /api/health/thresholds - Update thresholds
pub async fn update_thresholds(
    State(state): State<HealthState>,
    Json(request): Json<UpdateThresholdsRequest>,
) -> Json<ThresholdsResponse> {
    state.collector.set_thresholds(request.thresholds).await;

    let thresholds = state.collector.get_thresholds().await;
    Json(ThresholdsResponse { thresholds })
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/health/dashboard/stream - SSE dashboard stream
pub async fn stream_dashboard(
    State(state): State<HealthState>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    use futures::stream;

    info!("Starting dashboard SSE stream");

    let stream = stream::unfold(state, |state| async move {
        // Wait between updates
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Get fresh dashboard data
        let dashboard = state.collector.get_dashboard_data().await;

        let event = Event::default()
            .event("dashboard")
            .data(serde_json::to_string(&dashboard).unwrap_or_default());

        Some((Ok(event), state))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/health/service - Detailed service health
pub async fn get_service_health(State(state): State<HealthState>) -> impl IntoResponse {
    let health = evaluate_service_health("sim-service", crate::VERSION, state.service_start);

    Json(health)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_health_state() -> HealthState {
        HealthState {
            collector: Arc::new(HealthCollector::new()),
            service_start: std::time::Instant::now(),
        }
    }

    #[tokio::test]
    async fn test_get_health() {
        let state = test_health_state();
        let response = get_health(State(state.clone())).await;

        assert_eq!(response.status, "healthy");
        assert_eq!(response.service, "sim-service");
        assert_eq!(response.version, crate::VERSION);
        // Uptime should be very small since we just created the state
        assert!(response.uptime_secs < 5);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let state = test_health_state();
        let response = get_dashboard(State(state)).await;

        // Dashboard should have default system health
        assert_eq!(response.system_health, "healthy");
    }

    #[tokio::test]
    async fn test_get_thresholds() {
        let state = test_health_state();
        let response = get_thresholds(State(state)).await;

        // Should have default thresholds
        assert!(!response.thresholds.is_empty());
    }

    #[tokio::test]
    async fn test_get_service_health() {
        let state = test_health_state();
        let response = get_service_health(State(state)).await;
        let (parts, _) = response.into_response().into_parts();
        assert_eq!(parts.status, StatusCode::OK);
    }
}
