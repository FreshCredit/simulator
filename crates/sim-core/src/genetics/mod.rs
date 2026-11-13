// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Genetics Module
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION genetics, not product genetics
//!
//! This module contains genetic mechanisms for the agent simulation including:
//! - Inbreeding depression and pedigree tracking
//! - Genetic drift and allele frequency dynamics
//! - Heritability calculations
//! - Population genetics utilities

pub mod inbreeding;

// Re-export main types
pub use inbreeding::{
    calculate_offspring_f, can_reproduce_inbreeding, compute_population_inbreeding_stats,
    describe_relationship, is_inbred, is_severe_inbreeding, predict_offspring_fitness,
    InbreedingStats, InbreedingTracker, INBREEDING_AVOIDANCE_THRESHOLD, MIN_REPRODUCTION_FITNESS,
};
