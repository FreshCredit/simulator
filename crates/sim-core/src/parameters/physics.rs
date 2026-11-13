use super::ValidationError;
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Energy-related parameters
/// Per §7.4.0 lines 3755-3760
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyParameters {
    /// Energy consumed per tick (range: 1.0-2.0 per §7.4.0 line 3755)
    pub consumption_rate: f32,
    /// Energy cost for movement (range: 0.5-1.0)
    pub movement_cost: f32,
    /// Initial energy for new agents
    pub initial_energy: f32,
    /// Maximum energy cap
    pub max_energy: f32,
}

impl Default for EnergyParameters {
    fn default() -> Self {
        Self {
            consumption_rate: 1.0,
            movement_cost: 0.5,
            initial_energy: 50.0,
            max_energy: 100.0,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Mortality-related parameters
/// Per §6.21 mortality dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityParameters {
    /// Baseline death probability per tick
    pub baseline: f32,
    /// Aging effect coefficient (range: 0.08-0.12)
    pub aging_beta: f32,
}

impl Default for MortalityParameters {
    fn default() -> Self {
        Self {
            baseline: 0.00001,
            aging_beta: 0.10,
        }
    }
}

/// Reproduction-related parameters
/// Per §6.21 reproduction dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproductionParameters {
    /// Minimum energy to reproduce (range: 50-100)
    pub threshold: f32,
    /// Energy cost to reproduce
    pub cost: f32,
    /// Mutation rate per locus (Drake et al. 1998)
    pub mutation_rate: f32,
    /// DMN action value for reproduction decision
    pub action_value: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ReproductionParameters {
    fn default() -> Self {
        Self {
            threshold: 75.0,
            cost: 30.0,
            mutation_rate: 0.001,
            action_value: 20.0,
        }
    }
}

/// Resource cell parameters
/// Per §7.4.0 environment specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceParameters {
    /// Initial resource amount in cells
    pub initial_amount: f32,
    /// Maximum capacity per cell
    pub capacity: f32,
    /// Regrowth rate per tick
    pub regrowth_rate: f32,
    /// Fraction of cells with resources (0.0-1.0)
    pub density: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ResourceParameters {
    fn default() -> Self {
        Self {
            initial_amount: 10.0,
            capacity: 20.0,
            regrowth_rate: 0.5,
            density: 0.3,
        }
    }
}

/// Spatial sensing radii
/// Per §7.4.0 movement dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialParameters {
    /// Radius for detecting social clusters
    pub social_radius: f32,
    /// Radius for density calculations
    pub density_radius: f32,
    /// Radius for threat detection
    pub threat_radius: f32,
    /// Cell-level threat radius
    pub threat_cell_radius: f32,
}

impl Default for SpatialParameters {
    fn default() -> Self {
        Self {
            social_radius: 5.0,
            density_radius: 3.0,
            threat_radius: 8.0,
            threat_cell_radius: 5.0,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EnergyParameters {
    /// Validate energy parameters are within scientific bounds
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !(1.0..=2.0).contains(&self.consumption_rate) {
            return Err(ValidationError {
                field: "energy.consumption_rate".into(),
                message: format!("must be in [1.0, 2.0], got {}", self.consumption_rate),
            });
        }
        if !(0.5..=1.0).contains(&self.movement_cost) {
            return Err(ValidationError {
                field: "energy.movement_cost".into(),
                message: format!("must be in [0.5, 1.0], got {}", self.movement_cost),
            });
        }
        if self.initial_energy <= 0.0 || self.initial_energy > self.max_energy {
            return Err(ValidationError {
                field: "energy.initial_energy".into(),
                message: format!(
                    "must be in (0, {}], got {}",
                    self.max_energy, self.initial_energy
                ),
            });
        }
        Ok(())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MortalityParameters {
    /// Validate mortality parameters
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !self.baseline.is_finite() || !(0.0..=0.01).contains(&self.baseline) {
            return Err(ValidationError {
                field: "mortality.baseline".into(),
                message: format!("must be in [0, 0.01], got {}", self.baseline),
            });
        }
        if !(0.08..=0.12).contains(&self.aging_beta) {
            return Err(ValidationError {
                field: "mortality.aging_beta".into(),
                message: format!("must be in [0.08, 0.12], got {}", self.aging_beta),
            });
        }
        Ok(())
    }
}

impl ReproductionParameters {
    /// Validate reproduction parameters
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !(50.0..=100.0).contains(&self.threshold) {
            return Err(ValidationError {
                field: "reproduction.threshold".into(),
                message: format!("must be in [50, 100], got {}", self.threshold),
            });
        }
        if self.cost <= 0.0 || self.cost >= self.threshold {
            return Err(ValidationError {
                field: "reproduction.cost".into(),
                message: format!("must be in (0, {}), got {}", self.threshold, self.cost),
            });
        }
        if !(0.0001..=0.01).contains(&self.mutation_rate) {
            return Err(ValidationError {
                field: "reproduction.mutation_rate".into(),
                message: format!("must be in [0.0001, 0.01], got {}", self.mutation_rate),
            });
        }
        Ok(())
    }
}
