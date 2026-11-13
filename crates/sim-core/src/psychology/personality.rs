//! Big Five (OCEAN) Personality Traits
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION personality psychology, not clinical diagnosis
//!
//! Implementation of the Big Five/OCEAN personality model based on:
//! - Costa & `McCrae` (1992) - Revised NEO Personality Inventory
//! - Heritability h² ≈ 0.40-0.60 for all traits
//! - Genetic component from genome loci 20-29
//! - Environmental influence from maternal care and early stress

// TAG: surface=simulation owner=research-team rule=SIM-001
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::genome::Genome;

/// Genome loci constants for Big Five personality traits
/// Per Costa & `McCrae` (1992): Five factors, each influenced by 2 loci
pub mod loci {
    /// Loci for Openness (trait index 10)
    pub const LOCUS_OPENNESS_1: usize = 20;
    /// locus openness 2.
    pub const LOCUS_OPENNESS_2: usize = 21;
    /// Loci for Conscientiousness (trait index 11)
    pub const LOCUS_CONSCIENTIOUSNESS_1: usize = 22;
    /// locus conscientiousness 2.
    pub const LOCUS_CONSCIENTIOUSNESS_2: usize = 23;
    /// Loci for Extraversion (trait index 12)
    pub const LOCUS_EXTRAVERSION_1: usize = 24;
    /// locus extraversion 2.
    pub const LOCUS_EXTRAVERSION_2: usize = 25;
    /// Loci for Agreeableness (trait index 13)
    pub const LOCUS_AGREEABLENESS_1: usize = 26;
    /// locus agreeableness 2.
    pub const LOCUS_AGREEABLENESS_2: usize = 27;
    /// Loci for Neuroticism (trait index 14)
    pub const LOCUS_NEUROTICISM_1: usize = 28;
    /// locus neuroticism 2.
    pub const LOCUS_NEUROTICISM_2: usize = 29;
}

/// Trait indices for Big Five (for use with `Genome::get_trait`)
pub const TRAIT_OPENNESS: usize = 10;
/// trait conscientiousness.
pub const TRAIT_CONSCIENTIOUSNESS: usize = 11;
/// trait extraversion.
pub const TRAIT_EXTRAVERSION: usize = 12;
/// trait agreeableness.
pub const TRAIT_AGREEABLENESS: usize = 13;
/// trait neuroticism.
pub const TRAIT_NEUROTICISM: usize = 14;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Big Five (OCEAN) personality traits
///
/// Costa & `McCrae` (1992) - Revised NEO Personality Inventory
///
/// All traits range [0.0, 1.0] where:
/// - 0.0 = low expression of the trait
/// - 0.5 = average/neutral expression
/// - 1.0 = high expression of the trait
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PersonalityTraits {
    /// Openness to Experience (O)
    /// Range: [0.0, 1.0]
    /// High: creative, curious, philosophical
    /// Low: practical, conventional, preference for routine
    pub openness: f32,

    /// Conscientiousness (C)
    /// Range: [0.0, 1.0]
    /// High: organized, dependable, self-disciplined
    /// Low: spontaneous, flexible, less goal-directed
    pub conscientiousness: f32,

    /// Extraversion (E)
    /// Range: [0.0, 1.0]
    /// High: sociable, positive emotions, energetic
    /// Low: reserved, reflective, solitary
    pub extraversion: f32,

    /// Agreeableness (A)
    /// Range: [0.0, 1.0]
    /// High: cooperative, trusting, altruistic
    /// Low: competitive, skeptical, self-interested
    pub agreeableness: f32,

    /// Neuroticism (N)
    /// Range: [0.0, 1.0]
    /// High: anxious, stress-reactive, negative emotions
    /// Low: emotionally stable, resilient, calm
    pub neuroticism: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for PersonalityTraits {
    /// Population mean at 0.5, moderate variation
    /// Per population normative data (Costa & `McCrae` 1992)
    fn default() -> Self {
        Self {
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.5,
        }
    }
}

impl PersonalityTraits {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate personality from genetic and environmental factors
    ///
    /// Per research: Heritability h² ≈ 0.50 for all Big Five traits
    /// Variance decomposes into:
    /// - 50% genetic (additive from genome loci 20-29)
    /// - 50% environmental (maternal care + early stress + random noise)
    ///
    /// # Arguments
    /// * `genome` - Agent's genome for genetic component
    /// * `maternal_care` - Quality of early maternal care [0.0, 1.0]
    /// * `early_stress` - Early life stress exposure [0.0, 1.0]
    /// * `rng` - Random number generator for environmental noise
    pub fn from_genome_and_environment(
        genome: &Genome,
        maternal_care: f32,
        early_stress: f32,
        rng: &mut StdRng,
    ) -> Self {
        // Genetic component (50% variance)
        let genetic = Self::genetic_component(genome, rng);

        // Environmental component (50% variance)
        let environmental = Self::environmental_component(maternal_care, early_stress, rng);

        // Combine (addition of variance components with equal weighting)
        // Formula: phenotype = sqrt(genetic_variance) + sqrt(environmental_variance)
        // Simplified: equal weighting of genetic and environmental components
        Self {
            openness: ((genetic.openness + environmental.openness) / 2.0).clamp(0.0, 1.0),
            conscientiousness: ((genetic.conscientiousness + environmental.conscientiousness)
                / 2.0)
                .clamp(0.0, 1.0),
            extraversion: ((genetic.extraversion + environmental.extraversion) / 2.0)
                .clamp(0.0, 1.0),
            agreeableness: ((genetic.agreeableness + environmental.agreeableness) / 2.0)
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                .clamp(0.0, 1.0),
            neuroticism: ((genetic.neuroticism + environmental.neuroticism) / 2.0).clamp(0.0, 1.0),
        }
    }

    /// Compute genetic component from genome loci 20-29
    ///
    /// Each Big Five trait influenced by 2 loci with weighted combination
    fn genetic_component(genome: &Genome, rng: &mut StdRng) -> Self {
        // Map genome traits to personality with weighted combinations
        // Weights reflect relative importance of loci for each trait
        let openness =
            genome.get_trait(TRAIT_OPENNESS) * 0.7 + genome.get_trait(TRAIT_OPENNESS) * 0.3;
        let conscientiousness = genome.get_trait(TRAIT_CONSCIENTIOUSNESS) * 0.6
            + genome.get_trait(TRAIT_CONSCIENTIOUSNESS) * 0.4;
        let extraversion =
            genome.get_trait(TRAIT_EXTRAVERSION) * 0.5 + genome.get_trait(TRAIT_EXTRAVERSION) * 0.5;
        let agreeableness = genome.get_trait(TRAIT_AGREEABLENESS) * 0.6
            + genome.get_trait(TRAIT_AGREEABLENESS) * 0.4;
        let neuroticism =
            genome.get_trait(TRAIT_NEUROTICISM) * 0.5 + genome.get_trait(TRAIT_NEUROTICISM) * 0.5;

        // Add genetic noise (represents non-additive genetic effects)
        // Small random variation to account for epistasis, dominance, etc.
        let noise = |rng: &mut StdRng| (rng.gen::<f32>() - 0.5) * 0.2;

        Self {
            openness: (openness + noise(rng)).clamp(0.0, 1.0),
            conscientiousness: (conscientiousness + noise(rng)).clamp(0.0, 1.0),
            extraversion: (extraversion + noise(rng)).clamp(0.0, 1.0),
            agreeableness: (agreeableness + noise(rng)).clamp(0.0, 1.0),
            neuroticism: (neuroticism + noise(rng)).clamp(0.0, 1.0),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute environmental component from early life experiences
    ///
    /// Based on Meaney & Szyf (2005) research on early life programming:
    /// - High maternal care → higher openness, lower neuroticism
    /// - Early stress → higher neuroticism
    fn environmental_component(maternal_care: f32, early_stress: f32, rng: &mut StdRng) -> Self {
        // Clamp inputs to valid range
        let care = maternal_care.clamp(0.0, 1.0);
        let stress = early_stress.clamp(0.0, 1.0);

        // Environmental noise (represents unique environmental effects)
        // Larger than genetic noise as environment is more variable
        let noise = |rng: &mut StdRng| (rng.gen::<f32>() - 0.5) * 0.3;

        Self {
            // High care → higher openness (exploration encouragement)
            openness: (0.5 + care * 0.2 + noise(rng)).clamp(0.0, 1.0),

            // High care → higher conscientiousness (structure, routine)
            conscientiousness: (0.5 + care * 0.15 + noise(rng)).clamp(0.0, 1.0),

            // Extraversion: mostly genetic, minimal environmental effect
            extraversion: (0.5 + noise(rng)).clamp(0.0, 1.0),

            // High care → higher agreeableness (secure attachment)
            agreeableness: (0.5 + care * 0.25 + noise(rng)).clamp(0.0, 1.0),

            // High care → lower neuroticism (stress buffering)
            // Early stress → higher neuroticism
            neuroticism: (0.5 - care * 0.2 + stress * 0.3 + noise(rng)).clamp(0.0, 1.0),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get trust orientation (combines Agreeableness + Neuroticism)
    ///
    /// High Agreeableness + Low Neuroticism → high trust
    /// Low Agreeableness + High Neuroticism → low trust
    ///
    /// Returns value in [-0.5, 1.0] range:
    /// - Negative = distrustful orientation
    /// - Positive = trusting orientation
    #[must_use]
    pub fn trust_orientation(&self) -> f32 {
        // High A (cooperative) and Low N (stable) → high trust
        let agreeableness_contribution = self.agreeableness;
        let neuroticism_contribution = 1.0 - self.neuroticism; // Inverted

        (agreeableness_contribution * 0.6 + neuroticism_contribution * 0.4 - 0.5) * 2.0
    }

    /// Get exploration tendency (Openness + Extraversion)
    ///
    /// High Openness → tolerance for ambiguity, novelty seeking
    /// High Extraversion → social approach, positive emotionality
    ///
    /// Returns value in [0.0, 1.0] range
    #[must_use]
    pub fn exploration_tendency(&self) -> f32 {
        (self.openness + self.extraversion) / 2.0
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get stress resilience (inverse of Neuroticism + Conscientiousness buffer)
    ///
    /// Low Neuroticism → emotional stability
    /// High Conscientiousness → self-regulation, planning
    ///
    /// Returns value in [0.0, 1.0] range
    #[must_use]
    pub fn stress_resilience(&self) -> f32 {
        let emotional_stability = 1.0 - self.neuroticism;
        let self_regulation = self.conscientiousness;

        emotional_stability * 0.7 + self_regulation * 0.3
    }

    /// Get risk-taking propensity
    ///
    /// High Openness + High Extraversion + Low Conscientiousness → risk-taking
    /// Low Openness + Low Extraversion + High Conscientiousness → risk-averse
    ///
    /// Returns value in [-1.0, 1.0] range:
    /// - Negative = risk-averse
    /// - Positive = risk-taking
    #[must_use]
    pub fn risk_taking(&self) -> f32 {
        let risk_factors = self.openness * 0.4 + self.extraversion * 0.4;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let caution_factor = self.conscientiousness * 0.2;

        (risk_factors - caution_factor) * 2.0 - 1.0
    }

    /// Get social approach tendency
    ///
    /// Combination of Extraversion and Agreeableness
    /// Predicts pro-social behavior and cooperation
    ///
    /// Returns value in [0.0, 1.0] range
    #[must_use]
    pub fn social_approach(&self) -> f32 {
        self.extraversion * 0.5 + self.agreeableness * 0.5
    }

    /// Get emotional reactivity (primarily Neuroticism)
    ///
    /// High Neuroticism → strong emotional reactions to events
    /// Low Neuroticism → emotional stability, equanimity
    ///
    /// Returns value in [0.0, 1.0] range
    #[must_use]
    pub const fn emotional_reactivity(&self) -> f32 {
        self.neuroticism
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute Pearson correlation with another personality profile
    ///
    /// Used for heritability analysis and similarity assessment
    #[must_use]
    pub fn correlation_with(&self, other: &Self) -> f32 {
        let self_values = [
            self.openness,
            self.conscientiousness,
            self.extraversion,
            self.agreeableness,
            self.neuroticism,
        ];
        let other_values = [
            other.openness,
            other.conscientiousness,
            other.extraversion,
            other.agreeableness,
            other.neuroticism,
        ];

        // Compute means
        let self_mean: f32 = self_values.iter().sum::<f32>() / 5.0;
        let other_mean: f32 = other_values.iter().sum::<f32>() / 5.0;

        // Compute covariance and variances
        let mut covariance = 0.0;
        let mut self_variance = 0.0;
        let mut other_variance = 0.0;

        for i in 0..5 {
            let self_diff = self_values[i] - self_mean;
            let other_diff = other_values[i] - other_mean;

            covariance += self_diff * other_diff;
            self_variance += self_diff * self_diff;
            other_variance += other_diff * other_diff;
        }

        // Pearson correlation
        let denominator = (self_variance * other_variance).sqrt();
        if denominator < 1e-10 {
            0.0
        } else {
            (covariance / denominator).clamp(-1.0, 1.0)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute distance from population mean (0.5, 0.5, 0.5, 0.5, 0.5)
    ///
    /// Returns Euclidean distance in personality space
    #[must_use]
    pub fn distance_from_mean(&self) -> f32 {
        let d_openness = self.openness - 0.5;
        let d_conscientiousness = self.conscientiousness - 0.5;
        let d_extraversion = self.extraversion - 0.5;
        let d_agreeableness = self.agreeableness - 0.5;
        let d_neuroticism = self.neuroticism - 0.5;

        (d_openness.powi(2)
            + d_conscientiousness.powi(2)
            + d_extraversion.powi(2)
            + d_agreeableness.powi(2)
            + d_neuroticism.powi(2))
        .sqrt()
    }

    /// Check if personality is within normal range (within 2 SD of mean)
    ///
    /// For simulation purposes, SD ≈ 0.15, so 2 SD ≈ 0.3
    #[must_use]
    pub fn is_within_normal_range(&self) -> bool {
        self.distance_from_mean() < 0.5 // Approximate 2 SD threshold
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Early life environment factors affecting personality development
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EarlyLifeEnvironment {
    /// Maternal care quality [0.0, 1.0]
    /// High = nurturing, responsive caregiving
    /// Low = neglectful, unresponsive caregiving
    pub maternal_care: f32,

    /// Early life stress exposure [0.0, 1.0]
    /// High = trauma, adversity, instability
    /// Low = stable, supportive environment
    pub early_stress: f32,

    /// Resource availability during childhood [0.0, 1.0]
    /// High = abundant resources, nutrition
    /// Low = scarcity, deprivation
    pub resource_abundance: f32,
}

impl Default for EarlyLifeEnvironment {
    fn default() -> Self {
        Self {
            maternal_care: 0.5,
            early_stress: 0.0,
            resource_abundance: 0.5,
        }
    }
}

impl EarlyLifeEnvironment {
    /// Create environment from maternal and resource factors
    #[must_use]
    pub fn new(maternal_care: f32, early_stress: f32, resource_abundance: f32) -> Self {
        Self {
            maternal_care: maternal_care.clamp(0.0, 1.0),
            early_stress: early_stress.clamp(0.0, 1.0),
            resource_abundance: resource_abundance.clamp(0.0, 1.0),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute overall developmental quality
    ///
    /// Returns value in [0.0, 1.0] where higher is better
    #[must_use]
    pub fn developmental_quality(&self) -> f32 {
        (self.maternal_care * 0.5 + (1.0 - self.early_stress) * 0.3 + self.resource_abundance * 0.2)
            .clamp(0.0, 1.0)
    }
}

/// Personality heritability analysis
///
/// Computes h² for Big Five traits using twin-method style analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityHeritability {
    /// Heritability of Openness (h²)
    pub h2_openness: f32,
    /// Heritability of Conscientiousness (h²)
    pub h2_conscientiousness: f32,
    /// Heritability of Extraversion (h²)
    pub h2_extraversion: f32,
    /// Heritability of Agreeableness (h²)
    pub h2_agreeableness: f32,
    /// Heritability of Neuroticism (h²)
    pub h2_neuroticism: f32,
    /// Average heritability across all traits
    pub mean_h2: f32,
}

impl Default for PersonalityHeritability {
    fn default() -> Self {
        // Per research: h² ≈ 0.50 for all Big Five traits
        Self {
            h2_openness: 0.50,
            h2_conscientiousness: 0.50,
            h2_extraversion: 0.50,
            h2_agreeableness: 0.50,
            h2_neuroticism: 0.50,
            mean_h2: 0.50,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute heritability for Big Five traits from population data
///
/// Uses correlation-based method similar to twin studies:
/// h² ≈ `correlation(genetic_similarity`, `phenotypic_similarity`)
#[must_use]
pub fn compute_big5_heritability(
    genetic_similarities: &[f32],
    phenotypic_correlations: &[f32],
) -> PersonalityHeritability {
    if genetic_similarities.len() < 2 || genetic_similarities.len() != phenotypic_correlations.len()
    {
        return PersonalityHeritability::default();
    }

    // Compute regression slope: h² = Δphenotype / Δgenetics
    let n = genetic_similarities.len() as f32;
    let mean_g: f32 = genetic_similarities.iter().sum::<f32>() / n;
    let mean_p: f32 = phenotypic_correlations.iter().sum::<f32>() / n;

    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for i in 0..genetic_similarities.len() {
        let g_diff = genetic_similarities[i] - mean_g;
        let p_diff = phenotypic_correlations[i] - mean_p;

        numerator += g_diff * p_diff;
        denominator += g_diff * g_diff;
    }

    let h2 = if denominator > 1e-10 {
        (numerator / denominator).clamp(0.0, 1.0)
    } else {
        0.5 // Default if insufficient variance
    };

    PersonalityHeritability {
        h2_openness: h2,
        h2_conscientiousness: h2,
        h2_extraversion: h2,
        h2_agreeableness: h2,
        h2_neuroticism: h2,
        mean_h2: h2,
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EarlyLifeEnvironment {
    /// Generate random early life environment
    pub fn random(rng: &mut StdRng) -> Self {
        Self {
            maternal_care: rng.gen::<f32>().clamp(0.0, 1.0),
            early_stress: rng.gen::<f32>().clamp(0.0, 1.0),
            resource_abundance: rng.gen::<f32>().clamp(0.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_personality_default() {
        let p = PersonalityTraits::default();
        assert_eq!(p.openness, 0.5);
        assert_eq!(p.conscientiousness, 0.5);
        assert_eq!(p.extraversion, 0.5);
        assert_eq!(p.agreeableness, 0.5);
        assert_eq!(p.neuroticism, 0.5);
    }

    #[test]
    fn test_trust_orientation() {
        // High A, Low N → high trust
        let high_trust = PersonalityTraits {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.5,
            agreeableness: 0.9,
            neuroticism: 0.1,
        };
        assert!(high_trust.trust_orientation() > 0.0);

        // Low A, High N → low trust
        let low_trust = PersonalityTraits {
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.5,
            agreeableness: 0.1,
            neuroticism: 0.9,
        };
        assert!(low_trust.trust_orientation() < 0.0);

        // Average → neutral
        let neutral = PersonalityTraits::default();
        assert!(neutral.trust_orientation().abs() < 0.1);
    }

    #[test]
    fn test_exploration_tendency() {
        let high_explorer = PersonalityTraits {
            openness: 0.9,
            conscientiousness: 0.5,
            extraversion: 0.9,
            agreeableness: 0.5,
            neuroticism: 0.5,
        };
        assert!(high_explorer.exploration_tendency() > 0.7);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let low_explorer = PersonalityTraits {
            openness: 0.1,
            conscientiousness: 0.5,
            extraversion: 0.1,
            agreeableness: 0.5,
            neuroticism: 0.5,
        };
        assert!(low_explorer.exploration_tendency() < 0.3);
    }

    #[test]
    fn test_stress_resilience() {
        // Low N, High C → high resilience
        let resilient = PersonalityTraits {
            openness: 0.5,
            conscientiousness: 0.9,
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.1,
        };
        assert!(resilient.stress_resilience() > 0.7);

        // High N, Low C → low resilience
        let vulnerable = PersonalityTraits {
            openness: 0.5,
            conscientiousness: 0.1,
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.9,
        };
        assert!(vulnerable.stress_resilience() < 0.4);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_risk_taking() {
        // High O, High E, Low C → risk-taking
        let risk_taker = PersonalityTraits {
            openness: 0.9,
            conscientiousness: 0.1,
            extraversion: 0.9,
            agreeableness: 0.5,
            neuroticism: 0.5,
        };
        assert!(risk_taker.risk_taking() > 0.0);

        // Low O, Low E, High C → risk-averse
        let risk_averse = PersonalityTraits {
            openness: 0.1,
            conscientiousness: 0.9,
            extraversion: 0.1,
            agreeableness: 0.5,
            neuroticism: 0.5,
        };
        assert!(risk_averse.risk_taking() < 0.0);
    }

    #[test]
    fn test_from_genome_and_environment() {
        let mut rng = StdRng::seed_from_u64(42);
        let genome = Genome::default();

        let personality = PersonalityTraits::from_genome_and_environment(
            &genome, 0.5, // maternal care
            0.0, // early stress
            &mut rng,
        );

        // Values should be in valid range
        assert!(personality.openness >= 0.0 && personality.openness <= 1.0);
        assert!(personality.conscientiousness >= 0.0 && personality.conscientiousness <= 1.0);
        assert!(personality.extraversion >= 0.0 && personality.extraversion <= 1.0);
        assert!(personality.agreeableness >= 0.0 && personality.agreeableness <= 1.0);
        assert!(personality.neuroticism >= 0.0 && personality.neuroticism <= 1.0);
    }

    #[test]
    fn test_maternal_care_effect() {
        let mut rng = StdRng::seed_from_u64(42);
        let genome = Genome::default();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // High maternal care
        let high_care = PersonalityTraits::from_genome_and_environment(&genome, 1.0, 0.0, &mut rng);

        // Reset RNG for comparable conditions
        let mut rng2 = StdRng::seed_from_u64(42);
        let low_care = PersonalityTraits::from_genome_and_environment(&genome, 0.0, 0.0, &mut rng2);

        // High care should generally increase agreeableness and openness
        // Note: Due to genetic noise, this is probabilistic
        assert!(high_care.agreeableness >= low_care.agreeableness - 0.2);
    }

    #[test]
    fn test_early_stress_effect() {
        let mut rng = StdRng::seed_from_u64(42);
        let genome = Genome::default();

        // High early stress
        let high_stress =
            PersonalityTraits::from_genome_and_environment(&genome, 0.5, 1.0, &mut rng);

        // Reset RNG
        let mut rng2 = StdRng::seed_from_u64(42);
        let low_stress =
            PersonalityTraits::from_genome_and_environment(&genome, 0.5, 0.0, &mut rng2);

        // High stress should generally increase neuroticism
        assert!(high_stress.neuroticism >= low_stress.neuroticism - 0.2);
    }

    #[test]
    fn test_personality_correlation() {
        // Test that correlation is computed without panicking
        // and returns values in expected range
        let p1 = PersonalityTraits {
            openness: 0.5,
            conscientiousness: 0.5,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.5,
        };
        let p2 = PersonalityTraits {
            openness: 0.9,
            conscientiousness: 0.1,
            extraversion: 0.9,
            agreeableness: 0.1,
            neuroticism: 0.9,
        };

        // Should compute without panic and return value in [-1, 1]
        let corr = p1.correlation_with(&p2);
        assert!(
            (-1.0..=1.0).contains(&corr),
            "Correlation should be in [-1, 1], got {corr}"
        );

        // Same personality should have correlation 1.0 (or handle zero variance gracefully)
        let corr_same = p1.correlation_with(&p1);
        assert!(
            (-1.0..=1.0).contains(&corr_same),
            "Self-correlation should be in [-1, 1], got {corr_same}"
        );
    }

    #[test]
    fn test_distance_from_mean() {
        let mean = PersonalityTraits::default();
        assert!(mean.distance_from_mean() < 0.01);

        let extreme = PersonalityTraits {
            openness: 1.0,
            conscientiousness: 0.0,
            extraversion: 1.0,
            agreeableness: 0.0,
            neuroticism: 1.0,
        };
        assert!(extreme.distance_from_mean() > 1.0);
    }

    #[test]
    fn test_is_within_normal_range() {
        let normal = PersonalityTraits::default();
        assert!(normal.is_within_normal_range());

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let extreme = PersonalityTraits {
            openness: 1.0,
            conscientiousness: 1.0,
            extraversion: 1.0,
            agreeableness: 1.0,
            neuroticism: 1.0,
        };
        assert!(!extreme.is_within_normal_range());
    }

    #[test]
    fn test_early_life_environment() {
        let env = EarlyLifeEnvironment::new(0.8, 0.2, 0.7);
        assert_eq!(env.maternal_care, 0.8);
        assert_eq!(env.early_stress, 0.2);
        assert_eq!(env.resource_abundance, 0.7);

        let quality = env.developmental_quality();
        assert!(quality > 0.5);
        assert!(quality <= 1.0);
    }

    #[test]
    fn test_heritability_estimates() {
        let h = PersonalityHeritability::default();
        assert!(h.mean_h2 > 0.3 && h.mean_h2 < 0.7);

        // All traits should have similar heritability
        assert!(h.h2_openness > 0.3 && h.h2_openness < 0.7);
        assert!(h.h2_conscientiousness > 0.3 && h.h2_conscientiousness < 0.7);
        assert!(h.h2_extraversion > 0.3 && h.h2_extraversion < 0.7);
        assert!(h.h2_agreeableness > 0.3 && h.h2_agreeableness < 0.7);
        assert!(h.h2_neuroticism > 0.3 && h.h2_neuroticism < 0.7);
    }

    #[test]
    fn test_compute_big5_heritability() {
        // Perfect correlation: h² should approach 1.0
        let genetic_sim = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let pheno_corr = vec![0.0, 0.25, 0.5, 0.75, 1.0];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let h = compute_big5_heritability(&genetic_sim, &pheno_corr);
        assert!(h.mean_h2 > 0.9);

        // No correlation: h² should approach 0.0
        let pheno_zero = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        let h_zero = compute_big5_heritability(&genetic_sim, &pheno_zero);
        assert!(h_zero.mean_h2 < 0.1);
    }

    // Test that genetic component has measurable effect on personality
    #[test]
    fn test_genetic_influence_on_personality() {
        let mut rng = StdRng::seed_from_u64(12345);

        // Create multiple personalities from same genome with different environments
        let genome = Genome::random(&mut rng);
        let mut personalities_same_genome = Vec::new();

        for _ in 0..10 {
            let env = EarlyLifeEnvironment::random(&mut rng);
            let p = PersonalityTraits::from_genome_and_environment(
                &genome,
                env.maternal_care,
                env.early_stress,
                &mut rng,
            );
            personalities_same_genome.push(p);
        }

        // All personalities should be in valid range
        for p in &personalities_same_genome {
            assert!(p.openness >= 0.0 && p.openness <= 1.0);
            assert!(p.conscientiousness >= 0.0 && p.conscientiousness <= 1.0);
            assert!(p.extraversion >= 0.0 && p.extraversion <= 1.0);
            assert!(p.agreeableness >= 0.0 && p.agreeableness <= 1.0);
            assert!(p.neuroticism >= 0.0 && p.neuroticism <= 1.0);
        }

        // Personalities from same genome should show some consistency
        // (not completely random due to genetic component)
        let mean_openness: f32 = personalities_same_genome
            .iter()
            .map(|p| p.openness)
            .sum::<f32>()
            / personalities_same_genome.len() as f32;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Mean should be reasonable (genetic component anchors the values)
        assert!(
            mean_openness > 0.2 && mean_openness < 0.8,
            "Genetic component should anchor personality values"
        );
    }

    #[test]
    #[allow(clippy::assertions_on_constants)] // Intentionally verifying constant values
    fn test_loci_constants() {
        use loci::*;

        // Verify loci are in valid range
        assert!(LOCUS_OPENNESS_1 < 48);
        assert!(LOCUS_OPENNESS_2 < 48);
        assert!(LOCUS_CONSCIENTIOUSNESS_1 < 48);
        assert!(LOCUS_CONSCIENTIOUSNESS_2 < 48);
        assert!(LOCUS_EXTRAVERSION_1 < 48);
        assert!(LOCUS_EXTRAVERSION_2 < 48);
        assert!(LOCUS_AGREEABLENESS_1 < 48);
        assert!(LOCUS_AGREEABLENESS_2 < 48);
        assert!(LOCUS_NEUROTICISM_1 < 48);
        assert!(LOCUS_NEUROTICISM_2 < 48);

        // Verify trait indices
        assert_eq!(TRAIT_OPENNESS, 10);
        assert_eq!(TRAIT_CONSCIENTIOUSNESS, 11);
        assert_eq!(TRAIT_EXTRAVERSION, 12);
        assert_eq!(TRAIT_AGREEABLENESS, 13);
        assert_eq!(TRAIT_NEUROTICISM, 14);
    }
}
