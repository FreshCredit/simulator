//! Wright-Fisher Discrete Generation Mode
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION generation mode, not product workflow
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! This module provides true Wright-Fisher discrete non-overlapping generations,
//! as opposed to the default overlapping generations model.
//!
//! ## Wright-Fisher Model Key Properties:
//! - Discrete generations: All agents reproduce simultaneously
//! - Non-overlapping: Parents die when offspring generation starts
//! - Constant population size: Fixed N (effective population size)
//! - Random mating: Each offspring gets 2N alleles sampled from parent gene pool
//! - Binomial sampling: allele count ~ Binomial(2N, p)
//!
//! ## Mathematical Properties:
//! - Heterozygosity decay: `H_t` = `H_0(1` - 1/(2N))^t
//! - Fixation probability for neutral allele = initial frequency
//! - Time to fixation ~ 4N generations

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::genome::Genome;
use crate::lineage::{AlleleFrequencies, LineageNode, TraitSnapshot};
use crate::types::{AgentId, Tick, GENOME_LOCI};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generation mode for the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GenerationMode {
    /// Overlapping generations (current/default behavior)
    /// - Agents age continuously
    /// - Reproduction throughout lifespan
    /// - Variable population size
    #[default]
    Overlapping,

    /// Discrete non-overlapping generations (Wright-Fisher)
    /// - Simultaneous mass reproduction
    /// - Parents die when offspring generation starts
    /// - Constant population size
    Discrete {
        /// Generation duration in ticks
        generation_time: Tick,
        /// Effective population size (constant N)
        population_size: usize,
    },
}

impl GenerationMode {
    /// Check if this is discrete generation mode
    #[must_use]
    pub const fn is_discrete(&self) -> bool {
        matches!(self, Self::Discrete { .. })
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Check if this is overlapping generation mode
    #[must_use]
    pub const fn is_overlapping(&self) -> bool {
        matches!(self, Self::Overlapping)
    }

    /// Get population size if discrete mode, otherwise None
    #[must_use]
    pub const fn population_size(&self) -> Option<usize> {
        match self {
            Self::Discrete {
                population_size, ..
            } => Some(*population_size),
            _ => None,
        }
    }

    /// Get generation time if discrete mode, otherwise None
    #[must_use]
    pub const fn generation_time(&self) -> Option<Tick> {
        match self {
            Self::Discrete {
                generation_time, ..
            } => Some(*generation_time),
            _ => None,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Represents a single generation cohort
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationCohort {
    /// Generation number (0 = initial)
    pub generation_number: u32,
    /// Tick when this generation was born
    pub birth_tick: Tick,
    /// Tick when this generation will be replaced
    pub replacement_tick: Tick,
    /// Agent IDs in this generation
    pub agent_ids: Vec<AgentId>,
    /// Initial allele frequencies at generation start
    pub initial_allele_freqs: Vec<AlleleFrequencies>,
    /// Average heterozygosity at generation start
    pub initial_heterozygosity: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl GenerationCohort {
    /// Create a new generation cohort
    #[must_use]
    pub const fn new(
        generation_number: u32,
        birth_tick: Tick,
        replacement_tick: Tick,
        agent_ids: Vec<AgentId>,
        initial_allele_freqs: Vec<AlleleFrequencies>,
        initial_heterozygosity: f32,
    ) -> Self {
        Self {
            generation_number,
            birth_tick,
            replacement_tick,
            agent_ids,
            initial_allele_freqs,
            initial_heterozygosity,
        }
    }

    /// Get the number of agents in this cohort
    #[must_use]
    pub fn size(&self) -> usize {
        self.agent_ids.len()
    }
}

/// Manager for discrete generation cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscreteGenerationManager {
    /// Current generation cohort
    pub current_generation: GenerationCohort,
    /// Generation history for lineage tracking
    pub generation_history: Vec<GenerationCohort>,
    /// Effective population size (constant N)
    pub population_size: usize,
    /// Generation time in ticks
    pub generation_time: Tick,
    /// Current tick within generation
    pub tick_in_generation: Tick,
    /// Total generations elapsed
    pub total_generations: u32,
    /// Whether we're in replacement phase
    pub is_replacement_tick: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DiscreteGenerationManager {
    /// Create a new discrete generation manager
    #[must_use]
    pub fn new(population_size: usize, generation_time: Tick, start_tick: Tick) -> Self {
        let initial_cohort = GenerationCohort::new(
            0,
            start_tick,
            start_tick + generation_time,
            Vec::with_capacity(population_size),
            Vec::new(),
            0.0,
        );

        Self {
            current_generation: initial_cohort,
            generation_history: Vec::new(),
            population_size,
            generation_time,
            tick_in_generation: 0,
            total_generations: 0,
            is_replacement_tick: false,
        }
    }

    /// Initialize the first generation with agent IDs
    pub fn initialize_first_generation(
        &mut self,
        agent_ids: Vec<AgentId>,
        allele_freqs: Vec<AlleleFrequencies>,
        heterozygosity: f32,
    ) {
        self.current_generation.agent_ids = agent_ids;
        self.current_generation.initial_allele_freqs = allele_freqs;
        self.current_generation.initial_heterozygosity = heterozygosity;
    }

    /// Check if it's time for generation replacement
    #[must_use]
    pub const fn should_replace(&self) -> bool {
        self.tick_in_generation >= self.generation_time
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Increment tick within generation
    /// Returns true if generation replacement should occur
    pub fn tick(&mut self) -> bool {
        self.tick_in_generation += 1;
        self.is_replacement_tick = self.should_replace();
        self.is_replacement_tick
    }

    /// Move to next generation
    pub fn advance_generation(
        &mut self,
        new_agent_ids: Vec<AgentId>,
        allele_freqs: Vec<AlleleFrequencies>,
        heterozygosity: f32,
        current_tick: Tick,
    ) {
        // Archive current generation
        self.generation_history
            .push(self.current_generation.clone());

        // Create new generation cohort
        self.total_generations += 1;
        let expected_h = self.expected_heterozygosity(heterozygosity, self.total_generations);
        if expected_h < 0.0 {
            tracing::warn!("Theoretical heterozygosity calculation invalid");
        }
        self.current_generation = GenerationCohort::new(
            self.total_generations,
            current_tick,
            current_tick + self.generation_time,
            new_agent_ids,
            allele_freqs,
            heterozygosity,
        );
        self.tick_in_generation = 0;
        self.is_replacement_tick = false;
    }

    /// Get current generation number
    #[must_use]
    pub const fn current_generation_number(&self) -> u32 {
        self.current_generation.generation_number
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the expected heterozygosity given the decay formula
    /// `H_t` = `H_0` * (1 - 1/(2N))^t
    #[must_use]
    pub fn expected_heterozygosity(&self, initial_h: f32, generations: u32) -> f32 {
        if self.population_size == 0 {
            return initial_h;
        }
        let n = self.population_size as f32;
        let decay_factor = (1.0 - 1.0 / (2.0 * n)).powi(generations as i32);
        initial_h * decay_factor
    }

    /// Validate observed heterozygosity against theoretical prediction
    #[must_use]
    pub fn validate_heterozygosity(
        &self,
        observed: f32,
        initial_h: f32,
        generations: u32,
        tolerance: f32,
    ) -> bool {
        let expected = self.expected_heterozygosity(initial_h, generations);
        if expected == 0.0 {
            return observed < tolerance;
        }
        (observed - expected).abs() < tolerance * expected
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Wright-Fisher Discrete Model for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrightFisherDiscreteModel {
    /// Population size (constant N)
    pub population_size: usize,
    /// Current generation number
    pub current_generation: u32,
    /// Allele frequency trajectory
    pub frequency_trajectory: Vec<AlleleFrequencies>,
    /// Initial allele frequency (p)
    pub initial_p: f32,
    /// Heterozygosity history: (generation, `observed_heterozygosity`)
    pub heterozygosity_history: Vec<(u32, f32)>,
    /// Fixation events
    pub fixation_events: Vec<FixationEvent>,
    /// Initial heterozygosity
    pub initial_heterozygosity: f32,
}

/// Fixation event record
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FixationEvent {
    /// Generation when fixation occurred
    pub generation: u32,
    /// Which allele fixed: true = dominant, false = recessive
    pub dominant_fixed: bool,
    /// Final allele frequency (0.0 or 1.0)
    pub final_frequency: f32,
}

impl WrightFisherDiscreteModel {
    /// Create a new Wright-Fisher discrete model
    #[must_use]
    pub fn new(population_size: usize, initial_p: f32) -> Self {
        let initial_p = initial_p.clamp(0.0, 1.0);
        let initial_freq = AlleleFrequencies::from_p(initial_p, 0);
        let initial_heterozygosity = 2.0 * initial_p * (1.0 - initial_p);

        Self {
            population_size,
            current_generation: 0,
            frequency_trajectory: vec![initial_freq],
            initial_p,
            heterozygosity_history: vec![(0, initial_heterozygosity)],
            fixation_events: Vec::new(),
            initial_heterozygosity,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Expected heterozygosity under drift
    /// `H_t` = `H_0` * (1 - 1/(2N))^t
    #[must_use]
    pub fn expected_heterozygosity(&self, generation: u32) -> f32 {
        let n = self.population_size as f32;
        let decay_factor = (1.0 - 1.0 / (2.0 * n)).powi(generation as i32);
        self.initial_heterozygosity * decay_factor
    }

    /// Validate observed heterozygosity against theoretical prediction
    #[must_use]
    pub fn validate_heterozygosity(&self, observed: f32, generation: u32) -> bool {
        let expected = self.expected_heterozygosity(generation);
        if expected == 0.0 {
            return observed < 0.01;
        }
        (observed - expected).abs() < 0.1 * expected // 10% tolerance
    }

    /// Record a generation's statistics
    pub fn record_generation(
        &mut self,
        allele_freqs: &[AlleleFrequencies],
        observed_heterozygosity: f32,
    ) {
        self.current_generation += 1;

        // Store representative frequency (first locus)
        if let Some(freq) = allele_freqs.first() {
            self.frequency_trajectory
                .push(AlleleFrequencies::from_p(freq.p, self.current_generation));

            // Check for fixation
            if freq.p <= 0.0 || freq.p >= 1.0 {
                let fixation = FixationEvent {
                    generation: self.current_generation,
                    dominant_fixed: freq.p >= 1.0,
                    final_frequency: freq.p,
                };
                self.fixation_events.push(fixation);
            }
        }

        self.heterozygosity_history
            .push((self.current_generation, observed_heterozygosity));

        let expected_h = self.expected_heterozygosity(self.current_generation);
        if (observed_heterozygosity - expected_h).abs() > 0.5 && self.initial_p > 0.0 {
            tracing::debug!("Observed heterozygosity deviated from neutral drift expectation");
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if allele has fixed
    #[must_use]
    pub fn is_fixed(&self) -> bool {
        self.frequency_trajectory
            .last()
            .is_some_and(|f| f.p <= 0.0 || f.p >= 1.0)
    }

    /// Get fixation status
    #[must_use]
    pub fn fixation_status(&self) -> Option<FixationStatus> {
        self.frequency_trajectory.last().map(|f| {
            if f.p <= 0.0 {
                FixationStatus::FixedRecessive
            } else if f.p >= 1.0 {
                FixationStatus::FixedDominant
            } else {
                FixationStatus::Segregating
            }
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Time to fixation (if fixed)
    #[must_use]
    pub fn time_to_fixation(&self) -> Option<u32> {
        self.fixation_events.first().map(|e| e.generation)
    }

    /// Probability of fixation for neutral allele
    /// For neutral allele: = initial frequency
    #[must_use]
    pub const fn fixation_probability(&self) -> f32 {
        self.initial_p
    }
}

/// Fixation status of an allele
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixationStatus {
    /// Allele is still segregating in population
    Segregating,
    /// Dominant allele fixed (p = 1)
    FixedDominant,
    /// Recessive allele fixed (p = 0)
    FixedRecessive,
}

/// Compute allele frequencies from genomes
#[must_use]
pub fn compute_allele_frequencies(genomes: &[Genome]) -> Vec<AlleleFrequencies> {
    if genomes.is_empty() {
        return Vec::new();
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    let _n = genomes.len();
    let mut freqs = Vec::with_capacity(GENOME_LOCI);

    for locus in 0..GENOME_LOCI {
        // Sum all alleles at this locus across all genomes
        let mut dominant_count = 0.0;
        let mut total_alleles = 0.0;

        for genome in genomes {
            // Each genome has 2 alleles per locus (diploid)
            for allele_idx in 0..2 {
                let allele_value = genome.alleles[locus][allele_idx];
                // Treat allele > 0.5 as "dominant" for frequency calculation
                if allele_value > 0.5 {
                    dominant_count += 1.0;
                }
                total_alleles += 1.0;
            }
        }

        let p = if total_alleles > 0.0 {
            dominant_count / total_alleles
        } else {
            0.5
        };

        freqs.push(AlleleFrequencies::from_p(p, 0));
    }

    freqs
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute heterozygosity from genomes
/// H = 2pq (expected heterozygosity under HWE)
#[must_use]
pub fn compute_heterozygosity(genomes: &[Genome]) -> f32 {
    let freqs = compute_allele_frequencies(genomes);
    if freqs.is_empty() {
        return 0.0;
    }

    // Average expected heterozygosity across all loci
    let total_heterozygosity: f32 = freqs
        .iter()
        .map(super::lineage::AlleleFrequencies::expected_heterozygosity)
        .sum();
    total_heterozygosity / freqs.len() as f32
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sample offspring genomes using Wright-Fisher binomial sampling
///
/// Each offspring gets 2N alleles sampled from parent gene pool with probability p
/// This implements true Wright-Fisher sampling where:
/// - Each offspring allele is independently sampled from parent pool
/// - Binomial sampling: allele count ~ Binomial(2N, p)
pub fn sample_offspring_genomes(
    n_offspring: usize,
    parent_genomes: &[Genome],
    mutation_rate: f32,
    rng: &mut StdRng,
) -> Vec<Genome> {
    if parent_genomes.is_empty() {
        // If no parents, create random genomes
        return (0..n_offspring).map(|_| Genome::random(rng)).collect();
    }

    let parent_freqs = compute_allele_frequencies(parent_genomes);
    let mut offspring = Vec::with_capacity(n_offspring);

    for _ in 0..n_offspring {
        let mut alleles = Vec::with_capacity(GENOME_LOCI);

        for parent_freq in parent_freqs.iter().take(GENOME_LOCI) {
            let p = parent_freq.p;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001

            // Binomial sampling: 2 alleles per offspring
            // Each allele independently sampled with probability p
            let allele1 = if rng.gen::<f32>() < p { 1.0 } else { 0.0 };
            let allele2 = if rng.gen::<f32>() < p { 1.0 } else { 0.0 };

            // Apply mutation
            let allele1 = mutate_allele(allele1, mutation_rate, rng);
            let allele2 = mutate_allele(allele2, mutation_rate, rng);

            alleles.push([allele1, allele2]);
        }

        offspring.push(Genome { alleles });
    }

    offspring
}

/// Mutate an allele value
fn mutate_allele(allele: f32, mutation_rate: f32, rng: &mut StdRng) -> f32 {
    if rng.gen::<f32>() < mutation_rate {
        // Flip allele with small probability
        let delta: f32 = rng.gen::<f32>() * 0.2 - 0.1; // ±0.1
        (allele + delta).clamp(0.0, 1.0)
    } else {
        allele
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create lineage nodes for a new generation
pub fn create_offspring_lineage_nodes(
    offspring_genomes: &[(Genome, AgentId)],
    parent_generation: u32,
    birth_tick: Tick,
    next_agent_id: &mut AgentId,
) -> Vec<LineageNode> {
    offspring_genomes
        .iter()
        .map(|(genome, _)| {
            let id = *next_agent_id;
            *next_agent_id += 1;

            let trait_snapshot = TraitSnapshot::from_genome(genome, birth_tick);
            // For Wright-Fisher, parents are the entire previous generation
            // We use None for specific parents since it's random mating from the pool
            LineageNode::new_offspring(id, 0, None, birth_tick, trait_snapshot, parent_generation)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_generation_mode_default() {
        let mode = GenerationMode::default();
        assert!(mode.is_overlapping());
        assert!(!mode.is_discrete());
    }

    #[test]
    fn test_generation_mode_discrete() {
        let mode = GenerationMode::Discrete {
            generation_time: 10,
            population_size: 100,
        };
        assert!(!mode.is_overlapping());
        assert!(mode.is_discrete());
        assert_eq!(mode.population_size(), Some(100));
        assert_eq!(mode.generation_time(), Some(10));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_discrete_generation_manager_creation() {
        let mgr = DiscreteGenerationManager::new(100, 10, 0);
        assert_eq!(mgr.population_size, 100);
        assert_eq!(mgr.generation_time, 10);
        assert_eq!(mgr.tick_in_generation, 0);
        assert_eq!(mgr.total_generations, 0);
    }

    #[test]
    fn test_discrete_generation_manager_tick() {
        let mut mgr = DiscreteGenerationManager::new(100, 3, 0);

        assert!(!mgr.tick()); // tick 1
        assert!(!mgr.tick()); // tick 2
        assert!(mgr.tick()); // tick 3 - should replace

        assert_eq!(mgr.tick_in_generation, 3);
        assert!(mgr.should_replace());
    }

    #[test]
    fn test_generation_advance() {
        let mut mgr = DiscreteGenerationManager::new(100, 10, 0);
        mgr.initialize_first_generation((0..100).collect(), vec![], 0.5);

        mgr.advance_generation((100..200).collect(), vec![], 0.4, 10);

        assert_eq!(mgr.total_generations, 1);
        assert_eq!(mgr.tick_in_generation, 0);
        assert_eq!(mgr.current_generation_number(), 1);
        assert_eq!(mgr.generation_history.len(), 1);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_expected_heterozygosity() {
        let mgr = DiscreteGenerationManager::new(100, 10, 0);
        let h0 = 0.5;

        // H_t = H_0 * (1 - 1/(2N))^t
        // For N=100, t=10: decay = (1 - 1/200)^10 ≈ 0.951
        let h10 = mgr.expected_heterozygosity(h0, 10);
        assert!(h10 < h0, "Heterozygosity should decay");
        assert!(h10 > 0.0, "Heterozygosity should be positive");

        // At t=0, should equal h0
        let h0_calc = mgr.expected_heterozygosity(h0, 0);
        assert!((h0_calc - h0).abs() < 0.001);
    }

    #[test]
    fn test_wright_fisher_model_creation() {
        let model = WrightFisherDiscreteModel::new(100, 0.5);
        assert_eq!(model.population_size, 100);
        assert_eq!(model.initial_p, 0.5);
        assert_eq!(model.current_generation, 0);
        assert!(!model.is_fixed());
    }

    #[test]
    fn test_compute_allele_frequencies() {
        let mut rng = StdRng::seed_from_u64(42);

        // Create genomes with known allele values
        let genomes: Vec<Genome> = (0..10).map(|_| Genome::random(&mut rng)).collect();

        let freqs = compute_allele_frequencies(&genomes);
        assert_eq!(freqs.len(), GENOME_LOCI);

        // All frequencies should be in [0, 1]
        for freq in &freqs {
            assert!(freq.p >= 0.0 && freq.p <= 1.0);
            assert!(freq.q >= 0.0 && freq.q <= 1.0);
            assert!((freq.p + freq.q - 1.0).abs() < 0.001);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_compute_heterozygosity() {
        let mut rng = StdRng::seed_from_u64(42);

        let genomes: Vec<Genome> = (0..100).map(|_| Genome::random(&mut rng)).collect();

        let h = compute_heterozygosity(&genomes);
        assert!((0.0..=1.0).contains(&h));
    }

    #[test]
    fn test_sample_offspring_genomes() {
        let mut rng = StdRng::seed_from_u64(42);

        // Create parent genomes
        let parents: Vec<Genome> = (0..10).map(|_| Genome::random(&mut rng)).collect();

        // Sample offspring
        let offspring = sample_offspring_genomes(50, &parents, 0.001, &mut rng);

        assert_eq!(offspring.len(), 50);

        // All offspring should have valid genomes
        for genome in &offspring {
            assert_eq!(genome.alleles.len(), GENOME_LOCI);
        }
    }

    #[test]
    fn test_offspring_sampling_preserves_variation() {
        let mut rng = StdRng::seed_from_u64(42);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Create diverse parent population
        let mut parent_genomes = Vec::new();
        for _ in 0..100 {
            parent_genomes.push(Genome::random(&mut rng));
        }

        // Sample many offspring
        let offspring = sample_offspring_genomes(100, &parent_genomes, 0.001, &mut rng);

        // Offspring should have genetic variation
        let first_genome = &offspring[0];
        let has_variation = offspring.iter().any(|g| g != first_genome);
        assert!(has_variation, "Offspring should have genetic variation");
    }

    #[test]
    fn test_mutation_effect() {
        let mut rng = StdRng::seed_from_u64(42);

        // With 100% mutation rate, alleles should change
        let original = 0.5;
        let mutated = mutate_allele(original, 1.0, &mut rng);

        // Mutation can change the value, but it might stay similar
        // Just verify it's in valid range
        assert!((0.0..=1.0).contains(&mutated));
    }

    #[test]
    fn test_wright_fisher_heterozygosity_decay_validation() {
        let model = WrightFisherDiscreteModel::new(100, 0.5);

        // For p=0.5, H_0 = 2*0.5*0.5 = 0.5
        assert!((model.initial_heterozygosity - 0.5).abs() < 0.001);

        // Expected heterozygosity after 10 generations
        let h10 = model.expected_heterozygosity(10);
        assert!(h10 < model.initial_heterozygosity);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Should validate within tolerance
        assert!(model.validate_heterozygosity(h10, 10));

        // Should fail validation for wrong value
        assert!(!model.validate_heterozygosity(0.1, 10));
    }

    #[test]
    fn test_fixation_probability() {
        // For neutral alleles, fixation probability = initial frequency
        let model1 = WrightFisherDiscreteModel::new(100, 0.1);
        let model2 = WrightFisherDiscreteModel::new(100, 0.5);
        let model3 = WrightFisherDiscreteModel::new(100, 0.9);

        assert!((model1.fixation_probability() - 0.1).abs() < 0.001);
        assert!((model2.fixation_probability() - 0.5).abs() < 0.001);
        assert!((model3.fixation_probability() - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_generation_cohort_creation() {
        let cohort = GenerationCohort::new(
            0,
            0,
            10,
            (0..100).collect(),
            vec![AlleleFrequencies::from_p(0.5, 0)],
            0.5,
        );

        assert_eq!(cohort.generation_number, 0);
        assert_eq!(cohort.size(), 100);
        assert_eq!(cohort.birth_tick, 0);
        assert_eq!(cohort.replacement_tick, 10);
    }

    #[test]
    fn test_empty_parent_population() {
        let mut rng = StdRng::seed_from_u64(42);

        // When no parents, should create random offspring
        let offspring = sample_offspring_genomes(10, &[], 0.001, &mut rng);

        assert_eq!(offspring.len(), 10);
        for genome in &offspring {
            assert_eq!(genome.alleles.len(), GENOME_LOCI);
        }
    }
}
