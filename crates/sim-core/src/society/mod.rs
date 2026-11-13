//! Society module: Groups, reputation, attestations, norms, encounters
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION society, not product social features
//!
//! Per §6.21-6.25: Stage 3 (Society) mechanisms
//! Per §7.6 Steps 11-12: Social isolation and attestation processing

pub mod attestations;
pub mod encounters;
pub mod groups;
pub mod institutions;
pub mod norms;
pub mod reputation;

pub use attestations::*;
pub use encounters::*;
pub use groups::*;
pub use institutions::*;
pub use norms::*;
pub use reputation::*;

#[cfg(test)]
mod tests;
