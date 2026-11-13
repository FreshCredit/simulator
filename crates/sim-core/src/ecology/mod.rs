// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Ecology module: Population dynamics and environmental interactions
//!
//! SIM-001: Isolation verified - no product dependencies
//!
//! This module implements ecological dynamics including:
//! - Allee effects (small population fitness reduction)
//! - Population density-dependent regulation
//! - Component mechanisms (mate limitation, cooperative defense, etc.)

pub mod allee;

// Re-export main types
pub use allee::{AlleeComponent, AlleeConfig, AlleeEffect, AlleeMetrics};
