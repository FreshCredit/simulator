// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Dot Collector: 360° Feedback System (Ray Dalio's Principles)
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION feedback, NOT product ratings
//!
//! Per Dalio's "Principles": Continuous feedback creates "believability" scores
//! that weight opinions by track record. The "baseball card" system provides
//! transparent competency profiles for decision-making.
//!
//! ## Key Concepts
//!
//! - **Dot**: A single rating on a specific dimension (0.0-1.0)
//! - **Believability**: Confidence-weighted reputation per dimension
//! - **Baseball Card**: Comprehensive competency profile
//! - **360° Feedback**: Ratings from all interaction partners

/// types module.
pub mod types;
pub use types::{
    BelievabilityVoteResult, DimensionScore, Dot, FeedbackDimension, InteractionType,
    PopulationStats, RatingBias,
};

/// profile module.
pub mod profile;
pub use profile::{BaseballCard, DotCollector};

/// registry module.
pub mod registry;
pub use registry::DotCollectorRegistry;

/// utils module.
pub mod utils;

#[cfg(test)]
mod tests;
