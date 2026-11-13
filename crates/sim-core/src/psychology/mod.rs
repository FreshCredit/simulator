// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Psychology module: Personality, cognition, and behavioral disposition
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION psychology, not clinical diagnosis
//!
//! ## Submodules
//! - `personality`: Big Five (OCEAN) personality traits per Costa & `McCrae` (1992)
//!
//! ## Design Principles
//! - Personality emerges from genetics + environment (h² ≈ 0.50)
//! - Traits influence behavior but don't override DOCAS dynamics
//! - Cross-trait correlations should be minimal (orthogonal factors)

pub mod personality;

// Re-export personality types for convenience
pub use personality::{
    compute_big5_heritability, loci, EarlyLifeEnvironment,
    EarlyLifeEnvironment as EarlyEnvironment, PersonalityHeritability, PersonalityTraits,
    TRAIT_AGREEABLENESS, TRAIT_CONSCIENTIOUSNESS, TRAIT_EXTRAVERSION, TRAIT_NEUROTICISM,
    TRAIT_OPENNESS,
};
