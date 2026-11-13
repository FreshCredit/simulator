// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=e2e
//! Temporal dynamics module: Protocol introduction and era tracking
//!
//! SIM-TEMPORAL-001: Temporal staging for protocol adoption studies
//!
//! This module provides infrastructure for simulating the introduction
//! of credit reporting (the Protocol) into a reputation-only society.
//!
//! ## Key Concepts
//!
//! - **Era**: Temporal phase (`PreProtocol`, Transition, `PostProtocol`)
//! - **S-curve adoption**: Bass diffusion model for technology adoption
//! - **`TemporalConfig`**: Configuration for protocol introduction timing
//!
//! ## Academic Foundation
//!
//! - Rai & `McAndrews` (2015): ABM of technology adoption with S-curves
//! - Bass (1969): Diffusion of innovations model
//! - Schlüter et al. (2019): Multi-scale temporal dynamics in SES

pub mod adoption;
pub mod config;
pub mod metrics;
pub mod tracker;

pub use adoption::{AdoptionCurve, BassDiffusionModel};
pub use config::{SimulationEra, TemporalConfig};
pub use metrics::{
    DamageEventType, DamageSummary, DamageTracker, HypothesisSummary, TemporalMetrics,
    VolatilityTracker,
};
pub use tracker::{EraTracker, EraTransitionEvent};
