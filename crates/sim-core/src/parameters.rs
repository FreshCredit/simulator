//! Tunable Simulation Parameters
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-004: Deterministic - all parameters are deterministic values
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! This module centralizes all tunable parameters across the simulation,
//! enabling runtime configuration and scientific experimentation.
//!
//! ## Parameter Categories
//! - **Energy**: Consumption, movement costs, initial/max values
//! - **Mortality**: Baseline death rate, aging effects
//! - **Reproduction**: Thresholds, costs, mutation rates
//! - **Environment**: Grid size, resource distribution
//! - **Spatial**: Sensing radii for social/threat detection
//! - **Genome Ranges**: Trait expression bounds
//! - **Society**: Norm emergence, group formation, attestations
//! - **Language**: Semantic embedding parameters
//! - **Protocol**: Blockchain verification costs
//!
//! ## Evolutionary Dynamics Scenarios (Presets)
//!
//! The following presets model extreme evolutionary scenarios:
//!
//! | Scenario | Expected Behavior | Key Parameters |
//! |----------|-------------------|----------------|
//! | `near_extinction` | Population decline, strong selection | High mortality, sparse resources |
//! | `rapid_expansion` | Exponential growth, colonization | Low mortality, abundant resources |
//! | `boom_bust_cycles` | Oscillating population dynamics | Slow resource regrowth |
//! | `stable_equilibrium` | Carrying capacity convergence | Balanced energy/mortality |
//! | `high_diversity` | Rapid trait evolution | 10× mutation rate |
//! | `strong_social` | Fast norm/institution emergence | Low adoption thresholds |
//!
//! ### Scientific Rationale
//!
//! - **Near-Extinction**: Models resource scarcity, climate change effects, or invasive species
//!   scenarios. High selection pressure accelerates evolution but risks population collapse.
//!   Reference: Gompertz mortality model (Finch 1990), Resource depletion (Lotka-Volterra).
//!
//! - **Rapid Expansion**: Models colonization of new environments or post-extinction recovery.
//!   Low mortality allows exponential growth until resource limits are hit.
//!   Reference: R-selection (`MacArthur` & Wilson 1967).
//!
//! - **Boom-Bust Cycles**: Models predator-prey dynamics or seasonal environments.
//!   Slow regrowth creates temporal resource scarcity, causing population oscillations.
// TAG: surface=simulation owner=research-team rule=SIM-001
//!   Reference: Lotka-Volterra equations, Nicholson's blowflies (1954).
//!
//! - **Stable Equilibrium**: Models mature ecosystems at carrying capacity.
//!   Balanced parameters produce logistic growth with stable equilibrium.
//!   Reference: Logistic growth model (Verhulst 1838), K-selection.
//!
//! - **High Diversity**: Models adaptive radiation or rapidly changing environments.
//!   Higher mutation rate increases phenotypic variation, enabling faster adaptation.
//!   Reference: Drake et al. 1998 mutation rates, Fisher's fundamental theorem.
//!
//! - **Strong Social**: Models cooperative species or high-trust societies.
//!   Lower thresholds accelerate norm emergence and institutional formation.
//!   Reference: Boyd & Richerson cultural evolution, Henrich (2016).

/// physics module.
pub mod physics;
pub use physics::{
    EnergyParameters, MortalityParameters, ReproductionParameters, ResourceParameters,
    SpatialParameters,
};

/// bio module.
pub mod bio;
pub use bio::{
    DmnParameters, DocasParameters, GenomeRangeParameters, LanguageParameters, ProtocolParameters,
    SocietyParameters,
};

/// simulation module.
pub mod simulation;
pub use simulation::SimulationParameters;

/// empirical module.
pub mod empirical;
pub use empirical::EmpiricalParameters;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validation error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// field.
    pub field: String,
    /// message.
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validate a (min, max) range tuple where min < max and both are within bounds
fn validate_min_max_range(
    range: (f32, f32),
    field: &str,
    bounds: (f32, f32),
) -> Result<(), ValidationError> {
    let (min_val, max_val) = range;
    let (bound_min, bound_max) = bounds;

    if min_val < bound_min || max_val > bound_max || min_val >= max_val {
        return Err(ValidationError {
            field: field.into(),
            message: format!(
                "must satisfy {bound_min} ≤ min < max ≤ {bound_max}, got ({min_val}, {max_val})"
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
