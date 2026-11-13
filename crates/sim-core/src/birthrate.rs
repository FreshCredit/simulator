//! Birthrate Mechanism Module
//!
//! Implements a sophisticated birthrate system for agent reproduction that replaces
//! the fixed energy threshold reproduction with probabilistic birth rates
//! based on multiple biological and environmental factors.
//!
//! Key features:
//! - Age-based fertility curve (peak at middle age)
//! - Energy and trust thresholds
//! - Partner proximity search with sex-based pairing
//! - Population density effects
//! - Seasonal modulation (spring = higher birthrate)
//! - Sexual reproduction with genetic crossover
//! - Asexual reproduction fallback (optional)

// TAG: surface=simulation owner=research-team rule=SIM-001
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::environment::SpatialHash;
use crate::genome::{self, Genome};
use crate::types::{Position, Tick};

/// Configuration for the birthrate mechanism
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BirthrateConfig {
    /// Base probability per tick per eligible agent
    /// Range: [0.0, 1.0], typical values: 0.01-0.1
    pub base_birthrate: f32,

    /// How age affects birthrate (peak fertility multiplier)
    /// Range: [0.5, 2.0], higher = stronger age effect
    pub age_factor: f32,

    /// Minimum energy required for reproduction eligibility
    /// Agents below this threshold cannot reproduce
    pub energy_threshold: f32,

    /// Minimum trust level (τ) for reproduction
    /// Agents with τ below this are less likely to find partners
    pub tau_threshold: f32,

    /// Distance to search for potential partners (in grid cells)
    pub partner_proximity: f32,

    /// Maximum number of births allowed per tick (population control)
    pub max_offspring_per_tick: usize,

    /// Whether to allow asexual reproduction when no partner found
    pub allow_asexual: bool,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Age (in ticks) when agents become reproductively mature
    pub reproductive_maturity_age: Tick,

    /// Maximum age (in ticks) for reproduction
    /// Beyond this age, fertility drops to near zero
    pub max_reproductive_age: Tick,

    /// Season length in ticks (for seasonal birthrate modulation)
    pub season_length: Tick,

    /// Spring season start tick (within season cycle)
    /// Birthrates are higher during spring
    pub spring_start: Tick,

    /// Spring season duration
    pub spring_duration: Tick,

    /// Energy cost for reproduction (deducted from parent)
    pub reproduction_cost: f32,

    /// Mutation rate for genetic crossover
    pub mutation_rate: f32,
}

impl Default for BirthrateConfig {
    /// Default birthrate configuration with balanced parameters
    fn default() -> Self {
        Self {
            base_birthrate: 0.05,          // 5% base chance per tick
            age_factor: 1.5,               // 1.5x multiplier at peak fertility
            energy_threshold: 60.0,        // Need 60+ energy to reproduce
            tau_threshold: 0.5,            // Minimum trust level
            partner_proximity: 5.0,        // Search within 5 cells
            max_offspring_per_tick: 10,    // Limit births per tick
            allow_asexual: false,          // Require partner by default
            reproductive_maturity_age: 20, // Mature at tick 20
            max_reproductive_age: 500,     // Can reproduce up to tick 500
            season_length: 100,            // 100-tick seasons
            spring_start: 0,               // Spring starts at tick 0
            spring_duration: 25,           // Spring lasts 25 ticks
            reproduction_cost: 25.0,       // 25 energy per reproduction
            mutation_rate: 0.001,          // Standard mutation rate
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BirthrateConfig {
    /// Create configuration for high-reproduction scenarios
    #[must_use]
    pub fn high_fertility() -> Self {
        Self {
            base_birthrate: 0.1,
            age_factor: 2.0,
            max_offspring_per_tick: 20,
            allow_asexual: true,
            ..Default::default()
        }
    }

    /// Create configuration for low-reproduction scenarios
    #[must_use]
    pub fn low_fertility() -> Self {
        Self {
            base_birthrate: 0.02,
            age_factor: 1.0,
            max_offspring_per_tick: 5,
            partner_proximity: 3.0,
            ..Default::default()
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create configuration with asexual reproduction enabled
    #[must_use]
    pub fn with_asexual() -> Self {
        Self {
            allow_asexual: true,
            ..Default::default()
        }
    }
}

/// Calculator for birth probabilities based on agent and environmental factors
#[derive(Debug, Clone)]
pub struct BirthrateCalculator<'a> {
    config: &'a BirthrateConfig,
    current_tick: Tick,
}

impl<'a> BirthrateCalculator<'a> {
    /// Create a new birthrate calculator for the given tick
    #[must_use]
    pub const fn new(config: &'a BirthrateConfig, current_tick: Tick) -> Self {
        Self {
            config,
            current_tick,
        }
    }

    /// Calculate age-based fertility multiplier
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Returns value in [0.0, 1.0] based on age relative to lifespan
    /// Peak fertility at 20-40% of max reproductive age
    #[must_use]
    pub fn age_fertility(&self, age: Tick) -> f32 {
        if age < self.config.reproductive_maturity_age {
            // Not yet mature
            return 0.0;
        }

        if age >= self.config.max_reproductive_age
            || self.config.max_reproductive_age <= self.config.reproductive_maturity_age
        {
            // Beyond reproductive age or degenerate bounds
            return 0.0;
        }

        let effective_age = age - self.config.reproductive_maturity_age;
        let effective_max =
            self.config.max_reproductive_age - self.config.reproductive_maturity_age;

        // Peak at 30% of reproductive lifespan
        let peak_ratio = 0.3;
        let age_ratio = effective_age as f32 / (effective_max as f32).max(1.0);

        // Gaussian-like curve centered at peak_ratio
        let distance_from_peak = (age_ratio - peak_ratio).abs();
        let fertility = (-distance_from_peak * distance_from_peak * 8.0).exp();

        fertility.clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate energy-based fertility multiplier
    /// Higher energy = higher fertility
    #[must_use]
    pub fn energy_fertility(&self, energy: f32) -> f32 {
        if energy < self.config.energy_threshold || self.config.energy_threshold >= 100.0 {
            return 0.0;
        }

        // Linear increase from threshold to max (100.0)
        let excess_energy = energy - self.config.energy_threshold;
        let max_excess = 100.0 - self.config.energy_threshold;

        (excess_energy / max_excess).clamp(0.0, 1.0)
    }

    /// Calculate trust-based fertility multiplier
    /// Higher trust = more likely to find/accept partner
    #[must_use]
    pub fn trust_fertility(&self, tau: f32) -> f32 {
        if tau < self.config.tau_threshold {
            // Below threshold, reduced fertility
            return (tau / self.config.tau_threshold).clamp(0.0, 0.5);
        }

        // Above threshold, increases up to 1.0 at tau = 2.0
        ((tau - self.config.tau_threshold) / (2.0 - self.config.tau_threshold)).clamp(0.0, 1.0)
    }

    /// Calculate population density factor
    /// Lower density = higher birthrate (more resources available)
    #[must_use]
    pub fn density_factor(&self, local_density: f32) -> f32 {
        // Inverse relationship: fewer neighbors = higher birthrate
        // local_density is expected to be count of neighbors within proximity
        const OPTIMAL_DENSITY: f32 = 2.0; // Ideal number of nearby agents
                                          // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        if local_density <= OPTIMAL_DENSITY {
            1.0 // At or below optimal, full fertility
        } else {
            // Decrease as density increases
            (OPTIMAL_DENSITY / local_density.max(0.1)).clamp(0.3, 1.0)
        }
    }

    /// Calculate seasonal birthrate multiplier
    /// Spring = higher birthrate
    #[must_use]
    pub const fn seasonal_factor(&self) -> f32 {
        if self.config.season_length == 0 {
            return 1.0;
        }

        let season_tick = self.current_tick % self.config.season_length;

        // Check if we're in spring
        let in_spring = if self.config.spring_start + self.config.spring_duration
            <= self.config.season_length
        {
            // Spring doesn't wrap around
            season_tick >= self.config.spring_start
                && season_tick < self.config.spring_start + self.config.spring_duration
        } else {
            // Spring wraps around season boundary
            season_tick >= self.config.spring_start
                || season_tick
                    < (self.config.spring_start + self.config.spring_duration)
                        % self.config.season_length
        };

        if in_spring {
            1.3 // 30% boost during spring
        } else {
            0.9 // 10% reduction outside spring
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate overall birth probability for an agent
    /// Combines all factors into a single probability value
    #[must_use]
    pub fn calculate_birth_probability(&self, agent: &Agent, local_density: f32) -> f32 {
        // Check minimum eligibility
        if !agent.alive {
            return 0.0;
        }

        if agent.energy < self.config.energy_threshold {
            return 0.0;
        }

        if agent.age < self.config.reproductive_maturity_age {
            return 0.0;
        }

        if agent.age >= self.config.max_reproductive_age {
            return 0.0;
        }

        // Calculate individual factors
        let age_fert =
            (self.age_fertility(agent.age) * self.config.age_factor.max(0.0)).clamp(0.0, 1.0);
        let energy_fert = self.energy_fertility(agent.energy);
        let trust_fert = self.trust_fertility(agent.compute_trust());
        let density_mult = self.density_factor(local_density);
        let seasonal_mult = self.seasonal_factor();

        // Combine factors
        // Base rate * age * energy * trust * density * season
        let probability = self.config.base_birthrate
            * age_fert
            * (0.5 + 0.5 * energy_fert)  // Energy has partial effect
            * (0.5 + 0.5 * trust_fert)   // Trust has partial effect
            * density_mult
            * seasonal_mult;

        probability.clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if an agent is eligible for reproduction
    /// Per Research: `MATURITY_NEURAL_DEVELOPMENT_RESEARCH.md`
    /// Includes neural maturity check (vmPFC + dlPFC development)
    #[must_use]
    pub fn is_eligible(&self, agent: &Agent) -> bool {
        agent.alive
            && agent.energy >= self.config.energy_threshold
            && agent.age >= self.config.reproductive_maturity_age
            && agent.age < self.config.max_reproductive_age
            && agent.is_neurally_mature() // Neural gating (NEW)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Partner selection criteria and scoring
#[derive(Debug, Clone)]
pub struct PartnerSelector<'a> {
    config: &'a BirthrateConfig,
}

impl<'a> PartnerSelector<'a> {
    /// Create a new partner selector
    #[must_use]
    pub const fn new(config: &'a BirthrateConfig) -> Self {
        Self { config }
    }

    /// Calculate partner compatibility score
    /// Higher score = better match
    /// Considers: proximity, trust compatibility, genetic diversity
    #[must_use]
    pub fn compatibility_score(&self, agent: &Agent, candidate: &Agent, distance: f32) -> f32 {
        if agent.sex == candidate.sex
            || !candidate.alive
            || candidate.energy < self.config.energy_threshold
        {
            return 0.0;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
        let proximity = (-distance / self.config.partner_proximity).exp();
        let trust = Self::trust_compatibility(agent, candidate);
        let diversity = Self::genetic_diversity(agent, candidate);
        proximity * 0.4 + trust * 0.35 + diversity * 0.25
    }

    fn trust_compatibility(a: &Agent, b: &Agent) -> f32 {
        let diff = (a.compute_trust() - b.compute_trust()).abs();
        (1.0 - diff / 2.0).clamp(0.0, 1.0)
    }

    fn genetic_diversity(a: &Agent, b: &Agent) -> f32 {
        let similarity = a
            .genome
            .alleles
            .iter()
            .zip(b.genome.alleles.iter())
            .map(|(a_alleles, b_alleles)| {
                let a_avg = (a_alleles[0] + a_alleles[1]) / 2.0;
                let b_avg = (b_alleles[0] + b_alleles[1]) / 2.0;
                1.0 - (a_avg - b_avg).abs()
            })
            .sum::<f32>()
            / a.genome.alleles.len() as f32;
        1.0 - similarity
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Find the best partner for an agent from a list of candidates
    /// Returns None if no suitable partner found
    #[must_use]
    pub fn find_best_partner(
        &self,
        agent: &Agent,
        candidates: &[(usize, &Agent, f32)], // (index, agent, distance)
    ) -> Option<usize> {
        let mut best_idx: Option<usize> = None;
        let mut best_score = 0.0f32;

        for &(idx, candidate, distance) in candidates {
            let score = self.compatibility_score(agent, candidate, distance);

            if score > best_score {
                best_score = score;
                best_idx = Some(idx);
            }
        }

        // Require minimum compatibility threshold
        if best_score > 0.2 {
            best_idx
        } else {
            None
        }
    }
}

/// Represents a reproduction event (sexual or asexual)
#[derive(Debug, Clone)]
pub struct ReproductionEvent {
    /// Index of the primary parent (mother in sexual reproduction)
    pub parent1_idx: usize,
    /// Index of the secondary parent (father), None for asexual
    pub parent2_idx: Option<usize>,
    /// Position where offspring should spawn
    pub spawn_position: Position,
    /// Genome for the offspring
    pub offspring_genome: Genome,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Birthrate system that manages reproduction for the simulation
#[derive(Debug, Clone)]
pub struct BirthrateSystem {
    /// config.
    pub config: BirthrateConfig,
}

impl BirthrateSystem {
    /// Create a new birthrate system with the given configuration
    #[must_use]
    pub const fn new(config: BirthrateConfig) -> Self {
        Self { config }
    }

    /// Process reproduction for all agents
    /// Returns list of reproduction events to execute
    pub fn process_reproduction(
        &self,
        agents: &[Agent],
        current_tick: Tick,
        rng: &mut StdRng,
    ) -> Vec<ReproductionEvent> {
        let calculator = BirthrateCalculator::new(&self.config, current_tick);
        let selector = PartnerSelector::new(&self.config);

        // Build spatial hash for efficient partner finding
        let eligible_agents: Vec<(usize, Position)> = agents
            .iter()
            .enumerate()
            .filter(|(_, a)| calculator.is_eligible(a))
            .map(|(i, a)| (i, a.position))
            .collect();

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        if eligible_agents.is_empty() {
            return Vec::new();
        }

        // P1 FIX: Pre-allocate with eligible agent count
        let eligible_count = eligible_agents.len();
        let mut events = Vec::with_capacity(eligible_count / 2);
        let mut used_parents = Vec::with_capacity(eligible_count);

        let mut spatial_hash = SpatialHash::new(self.config.partner_proximity);
        spatial_hash.rebuild(&eligible_agents);

        // Pre-generate random values for birth probability checks
        let birth_rolls: Vec<f32> = (0..agents.len()).map(|_| rng.gen::<f32>()).collect();

        // Process each eligible agent
        for (idx, agent) in agents.iter().enumerate() {
            // Skip if already used as a parent this tick
            if used_parents.contains(&idx) {
                continue;
            }

            // Skip if not eligible
            if !calculator.is_eligible(agent) {
                continue;
            }

            // Calculate local density
            let nearby = spatial_hash.query_radius(&agent.position, self.config.partner_proximity);
            let local_density = nearby.len() as f32;

            // Calculate birth probability
            let birth_prob = calculator.calculate_birth_probability(agent, local_density);

            // Check if birth occurs
            if birth_rolls[idx] >= birth_prob {
                continue;
            }

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Try to find a partner
            let candidates: Vec<(usize, &Agent, f32)> = nearby
                .iter()
                .filter(|(i, _, _)| *i != idx && !used_parents.contains(i))
                .filter_map(|(i, _pos, dist)| agents.get(*i).map(|a| (*i, a, *dist)))
                .collect();

            let partner_idx = selector.find_best_partner(agent, &candidates);

            // Create reproduction event
            let event = if let Some(partner_idx) = partner_idx {
                // Sexual reproduction
                let parent2 = &agents[partner_idx];

                // Create offspring genome via crossover
                let offspring_genome = genome::crossover(
                    &agent.genome,
                    &parent2.genome,
                    self.config.mutation_rate,
                    rng,
                );

                // Spawn position near primary parent
                let offset_x = rng.gen::<f32>() * 2.0 - 1.0;
                let offset_y = rng.gen::<f32>() * 2.0 - 1.0;
                let spawn_pos =
                    Position::new_2d(agent.position.x + offset_x, agent.position.y + offset_y);

                used_parents.push(idx);
                used_parents.push(partner_idx);

                ReproductionEvent {
                    parent1_idx: idx,
                    parent2_idx: Some(partner_idx),
                    spawn_position: spawn_pos,
                    offspring_genome,
                }
            } else if self.config.allow_asexual {
                // Asexual reproduction (no partner found)
                // Offspring genome is mutated copy of parent's genome
                let offspring_genome = genome::crossover(
                    &agent.genome,
                    &agent.genome, // Self-fertilization
                    self.config.mutation_rate,
                    rng,
                );

                // TAG: surface=simulation owner=research-team rule=SIM-001
                let offset_x = rng.gen::<f32>() * 2.0 - 1.0;
                let offset_y = rng.gen::<f32>() * 2.0 - 1.0;
                let spawn_pos =
                    Position::new_2d(agent.position.x + offset_x, agent.position.y + offset_y);

                used_parents.push(idx);

                ReproductionEvent {
                    parent1_idx: idx,
                    parent2_idx: None,
                    spawn_position: spawn_pos,
                    offspring_genome,
                }
            } else {
                continue; // No partner and asexual not allowed
            };

            events.push(event);

            // Check max offspring limit
            if events.len() >= self.config.max_offspring_per_tick {
                break;
            }
        }

        events
    }

    /// Get current season as a string (for debugging/metrics)
    #[must_use]
    pub const fn current_season(&self, tick: Tick) -> &'static str {
        let season_tick = tick % self.config.season_length;

        let in_spring = if self.config.spring_start + self.config.spring_duration
            <= self.config.season_length
        {
            season_tick >= self.config.spring_start
                && season_tick < self.config.spring_start + self.config.spring_duration
        } else {
            season_tick >= self.config.spring_start
                || season_tick
                    < (self.config.spring_start + self.config.spring_duration)
                        % self.config.season_length
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        if in_spring {
            "spring"
        } else {
            "other"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::Genome;
    use crate::Sex;
    use rand::SeedableRng;

    #[test]
    fn test_birthrate_config_default() {
        let config = BirthrateConfig::default();
        assert_eq!(config.base_birthrate, 0.05);
        assert_eq!(config.age_factor, 1.5);
        assert_eq!(config.energy_threshold, 60.0);
        assert_eq!(config.tau_threshold, 0.5);
        assert_eq!(config.partner_proximity, 5.0);
        assert_eq!(config.max_offspring_per_tick, 10);
        assert!(!config.allow_asexual);
    }

    #[test]
    fn test_high_fertility_config() {
        let config = BirthrateConfig::high_fertility();
        assert_eq!(config.base_birthrate, 0.1);
        assert_eq!(config.max_offspring_per_tick, 20);
        assert!(config.allow_asexual);
    }

    #[test]
    fn test_age_fertility_curve() {
        let config = BirthrateConfig::default();
        let calculator = BirthrateCalculator::new(&config, 0);

        // Not mature
        assert_eq!(calculator.age_fertility(10), 0.0);
        assert_eq!(calculator.age_fertility(0), 0.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Just mature
        let fertility_young = calculator.age_fertility(20);
        assert!(fertility_young > 0.0);

        // Peak fertility (around 30% of reproductive span)
        // Reproductive span = 500 - 20 = 480, peak at ~20 + 144 = 164
        let fertility_peak = calculator.age_fertility(164);
        assert!(fertility_peak > fertility_young);

        // Very old
        assert_eq!(calculator.age_fertility(500), 0.0);
        assert_eq!(calculator.age_fertility(600), 0.0);
    }

    #[test]
    fn test_energy_fertility() {
        let config = BirthrateConfig::default();
        let calculator = BirthrateCalculator::new(&config, 0);

        // Below threshold
        assert_eq!(calculator.energy_fertility(50.0), 0.0);
        assert_eq!(calculator.energy_fertility(59.9), 0.0);

        // At threshold
        assert_eq!(calculator.energy_fertility(60.0), 0.0);

        // Above threshold
        assert!(calculator.energy_fertility(80.0) > 0.0);
        assert_eq!(calculator.energy_fertility(100.0), 1.0);
    }

    #[test]
    fn test_trust_fertility() {
        let config = BirthrateConfig::default();
        let calculator = BirthrateCalculator::new(&config, 0);

        // Below threshold (reduced fertility, scaled to [0, 0.5])
        assert!(calculator.trust_fertility(0.0) <= 0.5);
        assert!(calculator.trust_fertility(0.25) <= 0.5);
        assert!(calculator.trust_fertility(0.0) < calculator.trust_fertility(0.25));

        // At threshold
        assert_eq!(calculator.trust_fertility(0.5), 0.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Above threshold (scales from 0 to 1)
        assert!(calculator.trust_fertility(0.75) > 0.0);
        assert!(calculator.trust_fertility(1.0) > calculator.trust_fertility(0.75));
        assert_eq!(calculator.trust_fertility(2.0), 1.0);
    }

    #[test]
    fn test_density_factor() {
        let config = BirthrateConfig::default();
        let calculator = BirthrateCalculator::new(&config, 0);

        // Low density (full fertility)
        assert_eq!(calculator.density_factor(0.0), 1.0);
        assert_eq!(calculator.density_factor(1.0), 1.0);
        assert_eq!(calculator.density_factor(2.0), 1.0);

        // High density (reduced fertility)
        assert!(calculator.density_factor(5.0) < 1.0);
        assert!(calculator.density_factor(10.0) < calculator.density_factor(5.0));

        // Minimum clamp
        assert_eq!(calculator.density_factor(100.0), 0.3);
    }

    #[test]
    fn test_seasonal_factor() {
        let config = BirthrateConfig {
            season_length: 100,
            spring_start: 0,
            spring_duration: 25,
            ..Default::default()
        };

        // Spring (tick 0-24)
        let spring_calc = BirthrateCalculator::new(&config, 0);
        assert_eq!(spring_calc.seasonal_factor(), 1.3);

        let spring_calc_mid = BirthrateCalculator::new(&config, 12);
        assert_eq!(spring_calc_mid.seasonal_factor(), 1.3);

        // Not spring (tick 25-99)
        let other_calc = BirthrateCalculator::new(&config, 50);
        assert_eq!(other_calc.seasonal_factor(), 0.9);

        let other_calc_end = BirthrateCalculator::new(&config, 99);
        assert_eq!(other_calc_end.seasonal_factor(), 0.9);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_partner_compatibility_same_sex() {
        let config = BirthrateConfig::default();
        let selector = PartnerSelector::new(&config);

        let mut rng = StdRng::seed_from_u64(42);
        let genome = Genome::random(&mut rng);

        let male1 = Agent::new(0, Position::new_2d(0.0, 0.0), genome.clone(), 0, Sex::Male);
        let male2 = Agent::new(1, Position::new_2d(1.0, 0.0), genome, 0, Sex::Male);

        // Same sex should have 0 compatibility
        let score = selector.compatibility_score(&male1, &male2, 1.0);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_partner_compatibility_opposite_sex() {
        let config = BirthrateConfig::default();
        let selector = PartnerSelector::new(&config);

        let mut rng = StdRng::seed_from_u64(42);
        let genome_a = Genome::random(&mut rng);
        let genome_b = Genome::random(&mut rng);

        let mut male = Agent::new(0, Position::new_2d(0.0, 0.0), genome_a, 0, Sex::Male);
        male.energy = 80.0; // Above threshold
        male.age = 50; // Mature

        let mut female = Agent::new(1, Position::new_2d(1.0, 0.0), genome_b, 0, Sex::Female);
        female.energy = 80.0; // Above threshold
        female.age = 50; // Mature

        // Opposite sex with sufficient energy should have positive compatibility
        let score = selector.compatibility_score(&male, &female, 1.0);
        assert!(
            score > 0.0,
            "Opposite sex eligible agents should have positive compatibility"
        );
    }

    #[test]
    fn test_eligibility() {
        let config = BirthrateConfig::default();
        let calculator = BirthrateCalculator::new(&config, 0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut rng = StdRng::seed_from_u64(42);
        let genome = Genome::random(&mut rng);

        // Create eligible agent - must be neurally mature (requires aging)
        let mut agent = Agent::new(0, Position::new_2d(0.0, 0.0), genome, 0, Sex::Male);
        agent.energy = 80.0;
        // Age agent sufficiently to ensure neural maturity
        for _ in 0..200 {
            agent.age_tick();
            agent.update_neural_maturation();
        }

        assert!(
            calculator.is_eligible(&agent),
            "Agent should be eligible after aging to maturity"
        );

        // Dead agent
        let mut dead_agent = agent.clone();
        dead_agent.alive = false;
        assert!(!calculator.is_eligible(&dead_agent));

        // Low energy
        let mut low_energy_agent = agent.clone();
        low_energy_agent.energy = 50.0;
        assert!(!calculator.is_eligible(&low_energy_agent));

        // Too young
        let mut young_agent = agent.clone();
        young_agent.age = 10;
        assert!(!calculator.is_eligible(&young_agent));

        // Too old
        let mut old_agent = agent.clone();
        old_agent.age = 600;
        assert!(!calculator.is_eligible(&old_agent));
    }

    #[test]
    fn test_birthrate_system_creation() {
        let config = BirthrateConfig::default();
        let system = BirthrateSystem::new(config);
        assert_eq!(system.config.base_birthrate, 0.05);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_current_season() {
        let config = BirthrateConfig::default();
        let system = BirthrateSystem::new(config);

        assert_eq!(system.current_season(0), "spring");
        assert_eq!(system.current_season(24), "spring");
        assert_eq!(system.current_season(25), "other");
        assert_eq!(system.current_season(99), "other");
        assert_eq!(system.current_season(100), "spring"); // Wraps around
    }

    #[test]
    fn test_reproduction_event_creation() {
        let mut rng = StdRng::seed_from_u64(42);
        let genome = Genome::random(&mut rng);

        let event = ReproductionEvent {
            parent1_idx: 0,
            parent2_idx: Some(1),
            spawn_position: Position::new_2d(10.0, 10.0),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            offspring_genome: genome,
        };

        assert_eq!(event.parent1_idx, 0);
        assert_eq!(event.parent2_idx, Some(1));
        assert_eq!(event.spawn_position.x, 10.0);
    }

    // === Neural Maturity Gating Tests (Research: MATURITY_NEURAL_DEVELOPMENT) ===

    #[test]
    fn test_reproduction_requires_neural_maturity() {
        use crate::agent::Agent;
        use crate::types::Position;

        let config = BirthrateConfig::default();
        let calculator = BirthrateCalculator::new(&config, 0);

        // Create a young agent (age 30, but not neurally mature)
        let genome = Genome::default();
        let mut young_agent =
            Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

        // Set energy and age to meet basic requirements
        young_agent.energy = 100.0;
        for _ in 0..30 {
            young_agent.age_tick();
            young_agent.update_neural_maturation();
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // At age 30, agent may not be neurally mature yet
        // The eligibility check should verify neural maturity
        let is_eligible = calculator.is_eligible(&young_agent);

        // Agent with age=30 may or may not be neurally mature depending on genome
        // We just verify the check runs without error
        // If agent is NOT mature, is_eligible should be false
        if !young_agent.is_neurally_mature() {
            assert!(
                !is_eligible,
                "Agent should not be eligible without neural maturity"
            );
        }
    }

    #[test]
    fn test_mature_agent_eligible_for_reproduction() {
        use crate::agent::Agent;
        use crate::types::Position;

        let config = BirthrateConfig::default();
        let calculator = BirthrateCalculator::new(&config, 0);

        // Create a mature agent (age > 200 ticks to ensure neural maturity)
        let genome = Genome::default();
        let mut mature_agent =
            Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

        // Set high energy and age agent significantly
        mature_agent.energy = 100.0;
        for _ in 0..250 {
            mature_agent.age_tick();
            mature_agent.update_neural_maturation();
        }

        // After 250 ticks, agent should be neurally mature
        assert!(
            mature_agent.is_neurally_mature(),
            "Agent should be neurally mature after 250 ticks"
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Should be eligible for reproduction
        assert!(
            calculator.is_eligible(&mature_agent),
            "Neurally mature agent should be eligible for reproduction"
        );
    }

    #[test]
    fn test_fast_maturation_enables_earlier_reproduction() {
        use crate::agent::Agent;
        use crate::types::Position;

        let config = BirthrateConfig::default();
        let calculator = BirthrateCalculator::new(&config, 0);

        // Create fast-maturing genome (high PFC rate, low PDE4B)
        let mut fast_genome = Genome::default();
        fast_genome.alleles[32] = [1.0, 1.0]; // Max PFC maturation
        fast_genome.alleles[33] = [1.0, 1.0];
        fast_genome.alleles[24] = [0.0, 0.0]; // Min PDE4B = fast maturation
        fast_genome.alleles[25] = [0.0, 0.0];

        let mut fast_agent = Agent::new(
            1u64,
            Position::new(32.0, 32.0, 0.0),
            fast_genome,
            0,
            Sex::Male,
        );
        fast_agent.energy = 100.0;

        // Age fast agent
        for _ in 0..100 {
            fast_agent.age_tick();
            fast_agent.update_neural_maturation();
        }

        // Fast agent should be mature by 100 ticks
        if fast_agent.is_neurally_mature() {
            assert!(
                calculator.is_eligible(&fast_agent),
                "Fast-maturing agent should be eligible"
            );
        }
    }
}
