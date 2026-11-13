//! Explicit Fitness Testing Protocol for Agent Society Simulator
//!
//! This module provides structured fitness testing capabilities for research purposes,
//! complementing the implicit fitness mechanisms already present in the simulation.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! ## Research Background
//!
//! Current fitness in the simulator is **implicit** - it emerges from:
//! - Energy dynamics (survival)
//! - Reproduction gating (sexual selection)
//! - Trust-based partner matching (social selection)
//! - Inbreeding depression (genetic fitness)
//! - Allee effects (population-level fitness)
//!
//! This module adds **explicit fitness testing** for experimental validation.
//!
//! ## References
//! - Charlesworth & Willis (2009): "The genetics of inbreeding depression"
//! - Courchamp et al. (2008): "Allee effects in ecology and conservation"
//! - Wright, S. (1931): "Evolution in Mendelian populations"

use crate::agent::Agent;
use crate::types::{AgentId, Tick};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Fitness test configuration for experimental protocols
#[derive(Debug, Clone)]
pub struct FitnessTestConfig {
    /// Enable survival selection testing
    pub survival_selection: bool,
    /// Enable reproductive selection testing
    pub reproductive_selection: bool,
    /// Enable sexual selection testing
    pub sexual_selection: bool,
    /// Selection strength coefficient (0.0 to 2.0)
    /// 0.0 = neutral, 1.0 = moderate, 2.0 = strong selection
    pub selection_strength: f32,
    /// Directional selection target trait (optional)
    pub directional_trait: Option<DirectionalTrait>,
    /// Stabilizing selection optimal value (optional)
    pub stabilizing_optimum: Option<f32>,
    /// Disruptive selection threshold (optional)
    pub disruptive_threshold: Option<f32>,
}

impl Default for FitnessTestConfig {
    fn default() -> Self {
        Self {
            survival_selection: true,
            reproductive_selection: true,
            sexual_selection: false,
            selection_strength: 1.0,
            directional_trait: None,
            stabilizing_optimum: None,
            disruptive_threshold: None,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Traits that can be under directional selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirectionalTrait {
    /// Dopamine sensitivity (exploration/risk-taking)
    DopamineSensitivity,
    /// Oxytocin sensitivity (social bonding)
    OxytocinSensitivity,
    /// Cortisol sensitivity (stress response)
    CortisolSensitivity,
    /// Adrenaline sensitivity (arousal)
    AdrenalineSensitivity,
    /// Serotonin sensitivity (mood stability)
    SerotoninSensitivity,
    /// Metabolic efficiency
    MetabolicEfficiency,
    /// Vision range
    VisionRange,
    /// Learning rate
    LearningRate,
}

/// Individual fitness score with components
#[derive(Debug, Clone, Copy, Default)]
pub struct FitnessScore {
    /// Overall fitness (0.0 to 1.0+)
    pub total: f32,
    /// Survival fitness component
    pub survival: f32,
    /// Reproductive fitness component
    pub reproductive: f32,
    /// Social fitness component
    pub social: f32,
    /// Genetic fitness component
    pub genetic: f32,
    /// Energy efficiency component
    pub energy_efficiency: f32,
    /// Age factor (fitness decay at old age)
    pub age_factor: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl FitnessScore {
    /// Calculate weighted total from components
    pub fn calculate_total(&mut self) {
        self.total = (self.survival * 0.25
            + self.reproductive * 0.25
            + self.social * 0.20
            + self.genetic * 0.15
            + self.energy_efficiency * 0.10
            + self.age_factor * 0.05)
            .clamp(0.0, 2.0); // Allow up to 2.0 for highly fit agents
    }
}

/// Fitness landscape tracker for population analysis
#[derive(Debug, Clone, Default)]
pub struct FitnessLandscape {
    /// Fitness scores by agent ID
    pub scores: HashMap<AgentId, FitnessScore>,
    /// Mean fitness across population
    pub mean_fitness: f32,
    /// Fitness variance (selection intensity indicator)
    pub fitness_variance: f32,
    /// Fitness skew (direction of selection)
    pub fitness_skew: f32,
    /// Trait combinations and their fitness values
    pub trait_fitness_map: HashMap<TraitProfile, f32>,
    /// Adaptive peaks detected in landscape
    pub adaptive_peaks: Vec<AdaptivePeak>,
    /// Selection gradient vector
    pub selection_gradient: Vec<f32>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Profile of traits for fitness mapping
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitProfile {
    /// docas profile.
    pub docas_profile: [u8; 5], // Discretized DOCAS levels (0-10)
    /// genome signature.
    pub genome_signature: u64, // Hash of key genome loci
}

/// Detected adaptive peak in fitness landscape
#[derive(Debug, Clone, Copy)]
pub struct AdaptivePeak {
    /// position.
    pub position: [f32; 5], // DOCAS coordinates
    /// fitness.
    pub fitness: f32,
    /// basin size.
    pub basin_size: usize, // Number of agents near this peak
}

/// Fitness tracker for evolutionary dynamics analysis
#[derive(Debug)]
pub struct FitnessTracker {
    /// Configuration for fitness testing
    pub config: FitnessTestConfig,
    /// Current fitness landscape
    pub landscape: FitnessLandscape,
    /// Fitness history over time
    pub fitness_history: Vec<(Tick, PopulationFitnessStats)>,
    /// Generation-to-generation fitness change
    pub fitness_trajectory: Vec<f32>,
    /// Heritability estimate (h²)
    pub heritability_estimate: f32,
    /// Selection differential (S)
    pub selection_differential: f32,
    /// Response to selection (R)
    pub response_to_selection: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Population-level fitness statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct PopulationFitnessStats {
    /// tick.
    pub tick: Tick,
    /// mean fitness.
    pub mean_fitness: f32,
    /// max fitness.
    pub max_fitness: f32,
    /// min fitness.
    pub min_fitness: f32,
    /// variance.
    pub variance: f32,
    /// alive count.
    pub alive_count: usize,
    /// reproducing count.
    pub reproducing_count: usize,
}

impl FitnessTracker {
    /// Create new fitness tracker with configuration
    #[must_use]
    pub fn new(config: FitnessTestConfig) -> Self {
        Self {
            config,
            landscape: FitnessLandscape::default(),
            fitness_history: Vec::new(),
            fitness_trajectory: Vec::new(),
            heritability_estimate: 0.5, // Start with neutral expectation
            selection_differential: 0.0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            response_to_selection: 0.0,
        }
    }

    /// Calculate fitness for an individual agent
    #[must_use]
    pub fn calculate_agent_fitness(&self, agent: &Agent) -> FitnessScore {
        // Calculate all components first
        let survival = self.calculate_survival_fitness(agent);
        let reproductive = self.calculate_reproductive_fitness(agent);
        let social = self.calculate_social_fitness(agent);
        let genetic = self.calculate_genetic_fitness(agent);
        let energy_efficiency = self.calculate_energy_efficiency(agent);
        let age_factor = self.calculate_age_factor(agent);

        // Create score with all fields initialized
        let mut score = FitnessScore {
            total: 0.0,
            survival,
            reproductive,
            social,
            genetic,
            energy_efficiency,
            age_factor,
        };

        // Apply directional selection if configured
        if let Some(trait_target) = self.config.directional_trait {
            score = self.apply_directional_selection(score, agent, trait_target);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        score.calculate_total();
        score
    }

    /// Survival fitness: probability of surviving to next tick
    fn calculate_survival_fitness(&self, agent: &Agent) -> f32 {
        if !agent.alive {
            return 0.0;
        }

        // Energy-based survival probability
        let energy_factor = (agent.energy / 100.0).min(1.0);

        // Age-based mortality (increases with age)
        let age_factor = if agent.age > 100 {
            1.0 - ((agent.age - 100) as f32 / 100.0).min(1.0)
        } else {
            1.0
        };

        energy_factor * age_factor
    }

    /// Reproductive fitness: expected number of offspring
    fn calculate_reproductive_fitness(&self, agent: &Agent) -> f32 {
        if !agent.alive || agent.energy < 50.0 {
            return 0.0;
        }

        // Energy above reproduction threshold
        let energy_component = ((agent.energy - 50.0) / 50.0).min(1.0);

        // Age in reproductive window (peak at 30-50 ticks)
        let age_component = if agent.age < 20 {
            agent.age as f32 / 20.0
        } else if agent.age < 80 {
            1.0
        } else {
            1.0 - ((agent.age - 80) as f32 / 40.0).min(1.0)
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        energy_component * age_component
    }

    /// Social fitness: based on trust and network position
    fn calculate_social_fitness(&self, agent: &Agent) -> f32 {
        // Use computed trust (τ)
        let trust_score = agent.compute_trust();

        // Normalize to 0-1 range
        let normalized_trust = (trust_score / 2.0).clamp(0.0, 1.0);

        // Factor in reputation
        let reputation_bonus = agent.reputation_score;

        (normalized_trust * 0.7 + reputation_bonus * 0.3).min(1.0)
    }

    /// Genetic fitness: based on inbreeding and mutation load
    fn calculate_genetic_fitness(&self, agent: &Agent) -> f32 {
        // Use the inbreeding tracker's fitness multiplier if available
        // Otherwise estimate from genome diversity
        let genome_diversity = self.estimate_genome_diversity(&agent.genome);

        // Higher diversity = better genetic fitness
        genome_diversity.min(1.0)
    }

    /// Energy efficiency: energy gained per tick
    fn calculate_energy_efficiency(&self, agent: &Agent) -> f32 {
        // Metabolic efficiency (inverse of metabolic rate)
        let metabolic_efficiency = 2.0 - agent.metabolic_rate;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Normalize to 0-1
        (metabolic_efficiency / 1.0).clamp(0.0, 1.0)
    }

    /// Age factor: fitness modification based on age
    fn calculate_age_factor(&self, agent: &Agent) -> f32 {
        // Prime reproductive age gets bonus
        if agent.age >= 20 && agent.age <= 60 {
            1.0
        } else if agent.age < 20 {
            0.8 + (agent.age as f32 / 100.0)
        } else {
            0.8 - ((agent.age - 60) as f32 / 200.0).max(0.0)
        }
    }

    /// Apply directional selection to a specific trait
    fn apply_directional_selection(
        &self,
        mut score: FitnessScore,
        agent: &Agent,
        trait_target: DirectionalTrait,
    ) -> FitnessScore {
        let trait_value = match trait_target {
            DirectionalTrait::DopamineSensitivity => agent.genome.docas_alpha()[0],
            DirectionalTrait::OxytocinSensitivity => agent.genome.docas_alpha()[1],
            DirectionalTrait::CortisolSensitivity => agent.genome.docas_alpha()[2],
            DirectionalTrait::AdrenalineSensitivity => agent.genome.docas_alpha()[3],
            DirectionalTrait::SerotoninSensitivity => agent.genome.docas_alpha()[4],
            DirectionalTrait::MetabolicEfficiency => agent.metabolic_rate,
            DirectionalTrait::VisionRange => agent.vision_range as f32 / 10.0,
            DirectionalTrait::LearningRate => agent.genome.docas_beta()[0],
        };

        // Directional selection favors higher values
        let selection_multiplier = 1.0 + (trait_value * self.config.selection_strength);
        score.total *= selection_multiplier;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        score
    }

    /// Estimate genome diversity from heterozygosity
    fn estimate_genome_diversity(&self, genome: &crate::genome::Genome) -> f32 {
        let mut heterozygous_count = 0;
        let mut total_loci = 0;

        for locus in &genome.alleles {
            total_loci += 1;
            // Consider heterozygous if allele difference > 0.1
            if (locus[0] - locus[1]).abs() > 0.1 {
                heterozygous_count += 1;
            }
        }

        if total_loci == 0 {
            return 0.5;
        }

        heterozygous_count as f32 / total_loci as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update fitness landscape for entire population
    pub fn update_landscape(&mut self, agents: &[Agent], tick: Tick) {
        self.landscape.scores.clear();

        let mut total_fitness: f32 = 0.0;
        let mut max_fitness: f32 = 0.0;
        let mut min_fitness: f32 = f32::MAX;
        let mut reproducing_count = 0;

        for agent in agents {
            if !agent.alive {
                continue;
            }

            let fitness = self.calculate_agent_fitness(agent);

            total_fitness += fitness.total;
            max_fitness = max_fitness.max(fitness.total);
            min_fitness = min_fitness.min(fitness.total);

            if fitness.reproductive > 0.5 {
                reproducing_count += 1;
            }

            self.landscape.scores.insert(agent.id, fitness);
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let alive_count = self.landscape.scores.len();

        if alive_count > 0 {
            self.landscape.mean_fitness = total_fitness / alive_count as f32;

            // Calculate variance
            let mut variance_sum = 0.0;
            for score in self.landscape.scores.values() {
                let diff = score.total - self.landscape.mean_fitness;
                variance_sum += diff * diff;
            }
            self.landscape.fitness_variance = variance_sum / alive_count as f32;
        }

        // Record population stats
        let stats = PopulationFitnessStats {
            tick,
            mean_fitness: self.landscape.mean_fitness,
            max_fitness,
            min_fitness: if min_fitness == f32::MAX {
                0.0
            } else {
                min_fitness
            },
            variance: self.landscape.fitness_variance,
            alive_count,
            reproducing_count,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        self.fitness_history.push((tick, stats));

        // Trim history to prevent memory bloat
        if self.fitness_history.len() > 10000 {
            self.fitness_history.remove(0);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate breeder's equation parameters
    /// R = h² × S
    /// Where:
    ///   R = response to selection (change in mean fitness)
    ///   h² = narrow-sense heritability
    ///   S = selection differential
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn calculate_breeders_equation(&mut self) {
        if self.fitness_history.len() < 2 {
            return;
        }

        let Some((_, current)) = self.fitness_history.last().copied() else {
            return;
        };
        let previous = self.fitness_history[self.fitness_history.len() - 2].1;

        // Selection differential: difference between reproducers and population mean
        self.selection_differential = current.max_fitness - current.mean_fitness;

        // Response to selection: actual change in mean fitness
        self.response_to_selection = current.mean_fitness - previous.mean_fitness;

        // Estimate heritability: h² = R / S (if S > 0)
        if self.selection_differential > 0.001 {
            self.heritability_estimate =
                (self.response_to_selection / self.selection_differential).clamp(0.0, 1.0);
        }
    }

    /// Get fitness report for research output
    #[must_use]
    pub fn generate_report(&self) -> FitnessReport {
        FitnessReport {
            config: self.config.clone(),
            current_landscape: self.landscape.clone(),
            latest_stats: self.fitness_history.last().map(|(_, s)| *s),
            heritability: self.heritability_estimate,
            selection_differential: self.selection_differential,
            response_to_selection: self.response_to_selection,
            trajectory_length: self.fitness_history.len(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Serializable fitness report for output
#[derive(Debug, Clone)]
pub struct FitnessReport {
    /// config.
    pub config: FitnessTestConfig,
    /// current landscape.
    pub current_landscape: FitnessLandscape,
    /// latest stats.
    pub latest_stats: Option<PopulationFitnessStats>,
    /// heritability.
    pub heritability: f32,
    /// selection differential.
    pub selection_differential: f32,
    /// response to selection.
    pub response_to_selection: f32,
    /// trajectory length.
    pub trajectory_length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn test_fitness_score_calculation() {
        let mut score = FitnessScore {
            survival: 0.8,
            reproductive: 0.7,
            social: 0.6,
            genetic: 0.9,
            energy_efficiency: 0.75,
            age_factor: 0.85,
            total: 0.0,
        };

        score.calculate_total();

        // Verify weighted calculation
        let expected =
            0.8 * 0.25 + 0.7 * 0.25 + 0.6 * 0.20 + 0.9 * 0.15 + 0.75 * 0.10 + 0.85 * 0.05;
        assert!((score.total - expected).abs() < 0.01);
    }

    #[test]
    fn test_fitness_config_default() {
        let config = FitnessTestConfig::default();
        assert!(config.survival_selection);
        assert!(config.reproductive_selection);
        assert!(!config.sexual_selection);
        assert_eq!(config.selection_strength, 1.0);
    }
}
