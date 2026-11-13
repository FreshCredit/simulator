// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Genealogy Module
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-004: Deterministic - all genealogy operations use seeded RNG
//!
//! This module provides genealogical analysis tools including:
//! - Kingman coalescent for MRCA detection
//! - Coalescent tree construction and analysis
//! - Pairwise coalescence time calculations
//!
//! ## Mathematical Foundation
//!
//! The Kingman coalescent is a stochastic model for the genealogy of
//! a sample of individuals. It operates backward in time, starting
//! from n sampled lineages and tracing their ancestry until all
//! lineages converge to a single common ancestor.

pub mod coalescent;

// Re-export main types for convenience
pub use coalescent::{
    build_coalescent_from_lineage, find_all_pairwise_mrcas, find_mrca, validate_coalescent_tree,
    CoalescentNode, CoalescentStats, KingmanCoalescent, NodeId,
};
