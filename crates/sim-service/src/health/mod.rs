// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Health Monitoring Dashboard (P1-SIM-3)
//!
//! Provides real-time health monitoring for simulations with metrics collection,
//! alerting, and dashboard data aggregation.

pub mod collector;
pub mod metrics;
pub mod status;

pub use collector::HealthCollector;
pub use metrics::{MetricType, MetricValue, SimulationMetrics};
pub use status::{HealthStatus, ServiceHealth, SimulationHealth};
