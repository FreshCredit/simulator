// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Lineage Tracking System
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-004: Deterministic - all lineage operations are deterministic
//!
//! This module provides lineage tracking for agent genealogy and trait evolution.
//! It tracks parent-child relationships, genome evolution, and "brain evolution"
//! (DOCAS sensitivity changes across generations).
//!
//! ## Key Data Structures
//! - `LineageNode`: Single agent's lineage information
//! - `LineageTree`: Collection of all lineage nodes for ancestry queries
//! - `TraitSnapshot`: Genome trait values at a point in time
//!
//! ## Storage Considerations
//! Lineage data grows with population over time. For long simulations:
//! - Consider pruning dead lineages with no living descendants
//! - Consider periodic snapshots instead of full history
//! - Consider compact binary encoding for storage

/// core module.
pub mod core;
pub use core::{LineageNode, LineageSummary, TraitSnapshot};

/// tree module.
pub mod tree;
pub use tree::LineageTree;

/// genetics module.
pub mod genetics;
pub use crate::types::Tick;
pub use genetics::{
    linkage_disequilibrium, normalized_ld, sample_binomial, tajimas_d, Allele, AlleleFrequencies,
    AlleleFrequencyTracker, FixationStatus, Genotype, PopulationGeneticsStats,
};
pub use tree::BrainEvolutionStats;

/// haplogroups module.
pub mod haplogroups;
pub use haplogroups::{
    HaplogroupAssignment, HaplogroupDatabase, HaplogroupMarker, LineageType, MaternalLineage,
    PaternalLineage, PopulationHaplogroups,
};

#[cfg(test)]
mod tests;
