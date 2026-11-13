use super::{
    DmnParameters, DocasParameters, EmpiricalParameters, EnergyParameters, GenomeRangeParameters,
    LanguageParameters, MortalityParameters, ProtocolParameters, ReproductionParameters,
    ResourceParameters, SocietyParameters, SpatialParameters, ValidationError,
};
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Unified simulation parameters containing all categories
/// This struct centralizes all tunable parameters for easy configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimulationParameters {
    /// energy.
    pub energy: EnergyParameters,
    /// mortality.
    pub mortality: MortalityParameters,
    /// reproduction.
    pub reproduction: ReproductionParameters,
    /// resources.
    pub resources: ResourceParameters,
    /// spatial.
    pub spatial: SpatialParameters,
    /// genome ranges.
    pub genome_ranges: GenomeRangeParameters,
    /// docas.
    pub docas: DocasParameters,
    /// dmn.
    pub dmn: DmnParameters,
    /// society.
    pub society: SocietyParameters,
    /// language.
    pub language: LanguageParameters,
    /// protocol.
    pub protocol: ProtocolParameters,
    /// Empirical parameters from gap analysis (55 parameters)
    pub empirical: EmpiricalParameters,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl SimulationParameters {
    /// Validate all parameters
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.energy.validate()?;
        self.mortality.validate()?;
        self.reproduction.validate()?;
        self.genome_ranges.validate()?;
        self.empirical.validate()?;
        Ok(())
    }

    // === Parameter Presets ===
    // Per Task 6: Evolutionary dynamics scenarios

    /// Near-extinction scenario: High mortality, low resources
    /// Expected behavior: Population decline, strong selection pressure
    /// Scientific rationale: Models resource scarcity and environmental stress
    #[must_use]
    pub fn near_extinction() -> Self {
        let mut params = Self::default();
        params.energy.consumption_rate = 2.0; // Maximum consumption
        params.mortality.baseline = 0.001; // 100x higher baseline mortality
        params.mortality.aging_beta = 0.12; // Maximum aging effect
        params.reproduction.threshold = 100.0; // Maximum threshold (harder to reproduce)
        params.resources.density = 0.1; // Sparse resources
        params.resources.regrowth_rate = 0.1; // Slow regrowth
        params
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Rapid expansion scenario: Low mortality, abundant resources
    /// Expected behavior: Exponential population growth
    /// Scientific rationale: Models colonization of rich environments
    #[must_use]
    pub fn rapid_expansion() -> Self {
        let mut params = Self::default();
        params.energy.consumption_rate = 1.0; // Minimum consumption
        params.mortality.baseline = 0.000001; // Very low mortality
        params.mortality.aging_beta = 0.08; // Minimum aging effect
        params.reproduction.threshold = 50.0; // Easy to reproduce
        params.reproduction.cost = 20.0; // Lower reproduction cost
        params.resources.density = 0.6; // Abundant resources
        params.resources.regrowth_rate = 1.0; // Fast regrowth
        params
    }

    /// Boom-bust cycles scenario: Oscillating population dynamics
    /// Expected behavior: Population oscillations with periodic crashes
    /// Scientific rationale: Models prey-predator or resource-consumer dynamics
    #[must_use]
    pub fn boom_bust_cycles() -> Self {
        let mut params = Self::default();
        params.energy.consumption_rate = 1.5; // Moderate-high consumption
        params.reproduction.threshold = 60.0; // Lower threshold for faster reproduction
        params.reproduction.cost = 25.0; // Lower cost
        params.resources.density = 0.4; // Moderate resources
        params.resources.regrowth_rate = 0.3; // Slow regrowth (causes crashes)
        params.resources.capacity = 30.0; // Higher capacity (enables booms)
        params
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Stable equilibrium scenario: Balanced parameters for long-term stability
    /// Expected behavior: Population reaches carrying capacity and stabilizes
    /// Scientific rationale: Models mature ecosystems at equilibrium
    #[must_use]
    pub fn stable_equilibrium() -> Self {
        let mut params = Self::default();
        params.energy.consumption_rate = 1.2; // Slight consumption pressure
        params.mortality.aging_beta = 0.10; // Balanced aging
        params.reproduction.threshold = 70.0; // Moderate threshold
        params.resources.density = 0.35; // Balanced density
        params.resources.regrowth_rate = 0.4; // Moderate regrowth
        params
    }

    /// High genetic diversity scenario: Increased mutation and variation
    /// Expected behavior: Rapid trait evolution, diverse genome distribution
    /// Scientific rationale: Models adaptive radiation or high environmental change
    #[must_use]
    pub fn high_diversity() -> Self {
        let mut params = Self::default();
        params.reproduction.mutation_rate = 0.01; // 10x mutation rate
        params.genome_ranges.docas_alpha_range = (0.85, 0.99); // Wider alpha range
        params.genome_ranges.docas_beta_range = (0.02, 0.25); // Wider beta range
        params
    }

    /// Strong social dynamics scenario: Enhanced society stage parameters
    /// Expected behavior: Rapid norm emergence, strong institutional formation
    /// Scientific rationale: Models high-cooperation environments
    #[must_use]
    pub fn strong_social() -> Self {
        let mut params = Self::default();
        params.society.frequency_threshold = 0.40; // Lower threshold for norm adoption
        params.society.cascade_acceleration = 2.5; // Faster cascade
        params.society.early_adopter_threshold = 0.15; // Fewer early adopters needed
        params.society.min_group_size = 3; // Slightly larger groups
        params
    }

    // === Plausibility Checks ===

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if parameter combination is scientifically plausible
    /// Returns a list of warnings for questionable (but allowed) combinations
    #[must_use]
    pub fn check_plausibility(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Energy balance: consumption should not exceed typical resource gain
        if self.energy.consumption_rate > 1.5 && self.resources.density < 0.2 {
            warnings.push(
                "High consumption with sparse resources may cause rapid extinction".to_string(),
            );
        }

        // Reproduction viability: cost should leave parent with some energy
        if self.reproduction.cost > self.reproduction.threshold * 0.6 {
            warnings.push(
                "High reproduction cost relative to threshold may prevent reproduction".to_string(),
            );
        }

        // Mutation rate bounds
        if self.reproduction.mutation_rate > 0.05 {
            warnings.push("Very high mutation rate (>5%) may destabilize genome".to_string());
        }

        // Resource sustainability
        if self.resources.regrowth_rate < 0.1 && self.resources.density > 0.5 {
            warnings.push(
                "Slow regrowth with high initial density may cause boom-bust dynamics".to_string(),
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            );
        }

        // Social parameter coherence
        if self.society.frequency_threshold < 0.3 && self.society.early_adopter_threshold > 0.4 {
            warnings.push(
                "Low frequency threshold with high early adopter threshold is unusual".to_string(),
            );
        }

        warnings
    }

    /// Check if scenario is viable (no immediate extinction)
    /// Returns true if population has a chance to survive
    #[must_use]
    pub fn is_viable(&self) -> bool {
        // Must be able to collect resources faster than consumption
        // Rough heuristic: density * regrowth should exceed consumption
        let resource_rate = self.resources.density * self.resources.regrowth_rate * 10.0;
        let viable_resources = resource_rate >= self.energy.consumption_rate * 0.5;

        // Must be able to reproduce before dying
        // Rough heuristic: reproduction threshold reachable
        let viable_reproduction = self.reproduction.threshold < self.energy.max_energy;

        viable_resources && viable_reproduction
    }
}
