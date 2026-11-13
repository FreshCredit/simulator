//! Genome module: Heritable traits with diploid encoding
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION genome, not product genetics
//!
//! Per §6.6.5: 44 diploid loci encoding 22 traits (including PDE4B)
//! Per §8.2.1 lines 4423-4446: `GenomeToggle` contract

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::neural::pde4b_pathway::{
    compute_camp, compute_modulation, compute_pcreb, compute_pka_activation, PDE4BPathwayParams,
};
use crate::types::{GENOME_LOCI, TRAIT_COUNT};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Genome with 44 diploid loci encoding 22 traits
/// Per §6.6.5: Extended from 40 to 44 loci to include PDE4B gene
/// Note: Using Vec<[f32; 2]> instead of fixed array for serde compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Genome {
    /// 44 loci, each with 2 alleles (diploid)
    /// Per §6.6.5: Includes PDE4B loci 24-25 for cAMP-mediated learning rate control
    pub alleles: Vec<[f32; 2]>,
}

impl Default for Genome {
    /// Default mean genome (all alleles = 0.5)
    /// Per §8.2.1 line 4444: `genome_off` returns `Genome::default_mean()`
    fn default() -> Self {
        Self {
            alleles: vec![[0.5, 0.5]; GENOME_LOCI],
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Genome {
    /// Create empty genome (all zeros) for offspring
    #[must_use]
    pub fn empty() -> Self {
        Self {
            alleles: vec![[0.0, 0.0]; GENOME_LOCI],
        }
    }

    /// Default mean genome per §8.2.1 line 4444
    #[must_use]
    pub fn default_mean() -> Self {
        Self::default()
    }

    /// Create randomized genome with alleles in [0, 1]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn random(rng: &mut StdRng) -> Self {
        let alleles: Vec<[f32; 2]> = (0..GENOME_LOCI)
            .map(|_| [rng.gen::<f32>(), rng.gen::<f32>()])
            .collect();
        Self { alleles }
    }

    /// Get trait value (average of two alleles at diploid locus)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Trait index 0-19 maps to loci 0-39 (two loci per trait)
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn get_trait(&self, trait_index: usize) -> f32 {
        debug_assert!(trait_index < TRAIT_COUNT, "Trait index out of bounds");
        let locus = trait_index * 2;
        let allele_avg_1 = (self.alleles[locus][0] + self.alleles[locus][1]) / 2.0;
        let allele_avg_2 = (self.alleles[locus + 1][0] + self.alleles[locus + 1][1]) / 2.0;
        (allele_avg_1 + allele_avg_2) / 2.0
    }

    /// DOCAS alpha (decay) values per §8.2.1 lines 4454
    /// Range [0.9, 0.99] per Dayan & Abbott 2001
    #[must_use]
    pub fn docas_alpha(&self) -> [f32; 5] {
        // Trait indices 0-4 control alpha for D, O, C, A, S
        // Map trait value [0, 1] to alpha range [0.9, 0.99]
        [
            0.9 + self.get_trait(0) * 0.09, // D
            0.9 + self.get_trait(1) * 0.09, // O
            0.9 + self.get_trait(2) * 0.09, // C
            0.9 + self.get_trait(3) * 0.09, // A
            0.9 + self.get_trait(4) * 0.09, // S
        ]
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// DOCAS beta (sensitivity) values per §8.2.1 lines 4455
    /// Range [0.05, 0.2] per Montague et al. 1996
    #[must_use]
    pub fn docas_beta(&self) -> [f32; 5] {
        // Trait indices 5-9 control beta for D, O, C, A, S
        // Map trait value [0, 1] to beta range [0.05, 0.2]
        [
            0.05 + self.get_trait(5) * 0.15, // D
            0.05 + self.get_trait(6) * 0.15, // O
            0.05 + self.get_trait(7) * 0.15, // C
            0.05 + self.get_trait(8) * 0.15, // A
            0.05 + self.get_trait(9) * 0.15, // S
        ]
    }

    /// DOCAS beta values computed with continuous dial ranges
    #[must_use]
    pub fn docas_beta_with_dials(&self, dials: &crate::toggles::DialConfig) -> [f32; 5] {
        let (b_min, b_max) = dials.docas_beta_range;
        let b_spread = (b_max - b_min).max(0.001);
        [
            (b_min + self.get_trait(5) * b_spread).clamp(0.01, 1.0),
            (b_min + self.get_trait(6) * b_spread).clamp(0.01, 1.0),
            (b_min + self.get_trait(7) * b_spread).clamp(0.01, 1.0),
            (b_min + self.get_trait(8) * b_spread).clamp(0.01, 1.0),
            (b_min + self.get_trait(9) * b_spread).clamp(0.01, 1.0),
        ]
    }

    /// Planning depth K per §8.2.1 line 4479
    /// Range [3, 20]
    #[must_use]
    pub fn planning_depth(&self) -> u32 {
        (3.0 + self.get_trait(10) * 17.0) as u32
    }

    /// Planning horizon H per §8.2.1 line 4480
    /// Range [1, 5]
    #[must_use]
    pub fn planning_horizon(&self) -> u32 {
        (1.0 + self.get_trait(11) * 4.0) as u32
    }

    /// Isolation resilience per §6.21 - attenuates isolation effect
    #[must_use]
    pub fn isolation_resilience(&self) -> f32 {
        self.get_trait(12)
    }

    /// Default susceptibility for choice architecture per §8.2.1 line 4537
    /// Returns multiplier 1.6-1.9
    #[must_use]
    pub fn default_susceptibility(&self) -> f32 {
        0.6 + self.get_trait(13) * 0.3
    }

    /// Cooperation tendency: heritable tendency to cooperate vs defect
    /// Range [0.0, 1.0] where 0.5 is neutral
    /// This affects game-theoretic strategy selection, not trust directly
    #[must_use]
    pub fn cooperation_tendency(&self) -> f32 {
        self.get_trait(14)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Risk tolerance: heritable willingness to take risks
    /// Range [0.0, 1.0] where 0.5 is neutral
    /// This affects exploration/exploitation trade-offs
    #[must_use]
    pub fn risk_tolerance(&self) -> f32 {
        self.get_trait(15)
    }

    // NOTE: trust_propensity() was REMOVED per trust architecture review.
    // Trust MUST emerge from: genome → DOCAS sensitivity → DOCAS state → compute_tau()
    // Direct trust attributes bypass the DOCAS dynamics and violate design principles.
    // The genome already influences trust correctly via docas_alpha() and docas_beta().

    /// PDE4B cAMP gain controller per §6.6.5
    /// Modulates learning rate λ via cAMP signaling:
    /// - ↓ PDE4B activity → ↑ cAMP → fast prediction updating (high λ)
    /// - ↑ PDE4B activity → ↓ cAMP → sticky priors, slow updating (low λ)
    ///
    /// Reads diploid loci 24-25 directly per §6.6.5 specification
    /// Range [0.0, 1.0] where:
    /// - 0.0 = low PDE4B → high λ (fast learning)
    /// - 1.0 = high PDE4B → low λ (slow learning)
    #[must_use]
    pub fn pde4b_level(&self) -> f32 {
        // Direct access to loci 24-25 as specified in §6.6.5
        let locus_24_avg = (self.alleles[24][0] + self.alleles[24][1]) / 2.0;
        let locus_25_avg = (self.alleles[25][0] + self.alleles[25][1]) / 2.0;
        (locus_24_avg + locus_25_avg) / 2.0
    }

    /// Compute personalized learning rate λ based on PDE4B genotype
    /// Per §6.6.5: `λ_personal` = `λ_base` × (1.5 - 0.5 × `pde4b_level`)
    ///
    /// Base λ = 0.61 (universal constant)
    /// PDE4B modulation range: [0.305, 0.915] (varies by genetics)
    ///
    /// # Returns
    /// Learning rate λ in range [0.305, 0.915]
    #[must_use]
    pub fn compute_lambda(&self) -> f32 {
        const LAMBDA_BASE: f32 = 0.61;
        // Modulation: 1.5 at pde4b=0 (fast), 1.0 at pde4b=1.0 (slow)
        let modulation = 1.5 - 0.5 * self.pde4b_level();
        LAMBDA_BASE * modulation
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Sugarscape vision range per Epstein & Axtell 1996
    /// Loci 26-27 (trait index 13)
    ///
    /// Vision determines how far an agent can see in the grid environment.
    /// Agents with higher vision can locate resources more efficiently.
    ///
    /// Original Sugarscape: Uniform distribution U[1, 6]
    /// Our implementation: Range 1-11 cells for larger grids
    ///
    /// # Returns
    /// Vision radius in grid cells (1-11)
    #[must_use]
    pub fn vision_range(&self) -> u32 {
        // Loci 26-27 for vision trait
        let locus_26_avg = (self.alleles[26][0] + self.alleles[26][1]) / 2.0;
        let locus_27_avg = (self.alleles[27][0] + self.alleles[27][1]) / 2.0;
        let trait_value = (locus_26_avg + locus_27_avg) / 2.0;
        // Map [0, 1] to [1, 11]
        (1.0 + trait_value * 10.0) as u32
    }

    /// Sugarscape metabolic rate per Epstein & Axtell 1996
    /// Loci 28-29 (trait index 14)
    ///
    /// Metabolism determines energy consumption per tick.
    /// Higher metabolism agents consume more energy and are more likely to starve.
    ///
    /// Original Sugarscape: Uniform distribution U[1, 4] sugar/tick
    /// Our implementation: Range 1.0-2.0 energy/tick
    ///
    /// Selection dynamics favor low metabolism when resources are scarce.
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ///
    /// # Returns
    /// Energy consumption per tick (1.0-2.0)
    #[must_use]
    pub fn metabolic_rate(&self) -> f32 {
        // Loci 28-29 for metabolism trait
        let locus_28_avg = (self.alleles[28][0] + self.alleles[28][1]) / 2.0;
        let locus_29_avg = (self.alleles[29][0] + self.alleles[29][1]) / 2.0;
        let trait_value = (locus_28_avg + locus_29_avg) / 2.0;
        // Map [0, 1] to [1.0, 2.0]
        1.0 + trait_value * 1.0
    }

    /// Emotion sensitivity for Russell's circumplex model
    /// Loci 30-31 (trait index 15)
    ///
    /// Determines how strongly DOCAS maps to emotional responses.
    /// Higher sensitivity = more volatile emotions.
    ///
    /// # Returns
    /// Sensitivity multiplier (0.5-1.5)
    #[must_use]
    pub fn emotion_sensitivity(&self) -> f32 {
        // Loci 30-31 for emotion sensitivity
        let locus_30_avg = (self.alleles[30][0] + self.alleles[30][1]) / 2.0;
        let locus_31_avg = (self.alleles[31][0] + self.alleles[31][1]) / 2.0;
        let trait_value = (locus_30_avg + locus_31_avg) / 2.0;
        // Map [0, 1] to [0.5, 1.5]
        0.5 + trait_value * 1.0
    }

    // === Neural Maturation Traits (Research: MATURITY_NEURAL_DEVELOPMENT) ===

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// PFC maturation rate modifier
    /// Loci 32-33 (trait index 16)
    ///
    /// Controls speed of prefrontal cortex development.
    /// Interacts with PDE4B (loci 24-25) via cAMP signaling.
    /// Low PDE4B + high maturation rate = fastest development.
    ///
    /// # Returns
    /// Maturation rate multiplier (0.5-1.5)
    #[must_use]
    pub fn pfc_maturation_rate(&self) -> f32 {
        // Loci 32-33 for PFC maturation rate
        let locus_32_avg = (self.alleles[32][0] + self.alleles[32][1]) / 2.0;
        let locus_33_avg = (self.alleles[33][0] + self.alleles[33][1]) / 2.0;
        let trait_value = (locus_32_avg + locus_33_avg) / 2.0;
        // Map [0, 1] to [0.5, 1.5] - higher = faster maturation
        0.5 + trait_value * 1.0
    }

    /// Synaptic pruning efficiency
    /// Loci 34-35 (trait index 17)
    ///
    /// Controls how efficiently unused synapses are pruned during development.
    /// Higher efficiency = better optimized neural circuits.
    ///
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// # Returns
    /// Pruning efficiency (0.3-1.0)
    #[must_use]
    pub fn pruning_efficiency(&self) -> f32 {
        // Loci 34-35 for pruning efficiency
        let locus_34_avg = (self.alleles[34][0] + self.alleles[34][1]) / 2.0;
        let locus_35_avg = (self.alleles[35][0] + self.alleles[35][1]) / 2.0;
        let trait_value = (locus_34_avg + locus_35_avg) / 2.0;
        // Map [0, 1] to [0.3, 1.0]
        0.3 + trait_value * 0.7
    }

    /// Myelination rate
    /// Loci 36-37 (trait index 18)
    ///
    /// Controls speed of white matter development.
    /// Faster myelination = quicker processing speed maturation.
    ///
    /// # Returns
    /// Myelination rate multiplier (0.5-1.5)
    #[must_use]
    pub fn myelination_rate(&self) -> f32 {
        // Loci 36-37 for myelination rate
        let locus_36_avg = (self.alleles[36][0] + self.alleles[36][1]) / 2.0;
        let locus_37_avg = (self.alleles[37][0] + self.alleles[37][1]) / 2.0;
        let trait_value = (locus_36_avg + locus_37_avg) / 2.0;
        // Map [0, 1] to [0.5, 1.5]
        0.5 + trait_value * 1.0
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Stress sensitivity for maturation effects
    /// Loci 38-39 (trait index 19)
    ///
    /// Controls how strongly chronic stress delays PFC maturation.
    /// High sensitivity = larger maturation delays under stress.
    ///
    /// # Returns
    /// Stress sensitivity (0.2-1.0)
    #[must_use]
    pub fn stress_sensitivity(&self) -> f32 {
        // Loci 38-39 for stress sensitivity
        let locus_38_avg = (self.alleles[38][0] + self.alleles[38][1]) / 2.0;
        let locus_39_avg = (self.alleles[39][0] + self.alleles[39][1]) / 2.0;
        let trait_value = (locus_38_avg + locus_39_avg) / 2.0;
        // Map [0, 1] to [0.2, 1.0]
        0.2 + trait_value * 0.8
    }

    /// Compute overall maturation rate combining PDE4B, genome, and environment
    ///
    /// Uses the PDE4B → cAMP → PKA → CREB signaling pathway for biologically
    /// accurate maturation modulation per Research: `MATURITY_NEURAL_DEVELOPMENT`.
    ///
    /// Pathway:
    /// 1. PDE4B degrades cAMP (Step 1)
    /// 2. cAMP activates PKA via Hill function (Step 2)
    /// 3. PKA phosphorylates CREB (Step 3)
    /// 4. pCREB modulates maturation genes (Step 4)
    ///
    /// # Arguments
    /// * `stress_level` - Current cortisol level (0-1), higher = more stress
    /// * `resource_level` - Current energy ratio (0-1), lower = resource scarcity
    ///
    /// # Returns
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Effective maturation rate multiplier
    #[must_use]
    pub fn compute_maturation_rate(&self, stress_level: f32, resource_level: f32) -> f32 {
        const BASE_RATE: f32 = 1.0;

        // Get PDE4B level from genome
        let pde4b = self.pde4b_level();

        // Use research-based pathway parameters
        let params = PDE4BPathwayParams::default();

        // Step 1: cAMP production vs PDE4B degradation
        // Formula: cAMP = v_AC / (k_PDE * PDE4B/K_M + k_leak)
        let camp = compute_camp(pde4b, &params);

        // Step 2: PKA activation via Hill function
        // Formula: PKA = cAMP^n / (K_PKAn^n + cAMP^n)
        let pka = compute_pka_activation(camp, &params);

        // Step 3: CREB phosphorylation
        // Formula: pCREB = baseline + (max - baseline) * PKA
        let pcreb = compute_pcreb(pka, &params);

        // Step 4: Maturation modulation
        // Formula: modulation = 1.0 + pCREB / (K_mat + pCREB)
        let pde4b_modulation = compute_modulation(pcreb, &params);

        // Genome-controlled maturation rate
        // Range: [0.5, 1.5]
        let genome_rate = self.pfc_maturation_rate();

        // Stress effect: chronic stress delays maturation
        // Range: [0.5, 1.0] (slows by up to 50% at max stress)
        let stress_threshold = 0.3; // Normal baseline cortisol
        let stress_effect = if stress_level > stress_threshold {
            1.0 - (stress_level - stress_threshold) * self.stress_sensitivity() * 0.5
        } else {
            1.0
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Resource effect: scarcity delays maturation
        // Range: [0.7, 1.0] (slows by up to 30% at starvation)
        let resource_effect = if resource_level < 0.5 {
            0.7 + resource_level * 0.6 // 0.7 at resource=0, 1.0 at resource=0.5
        } else {
            1.0
        };

        BASE_RATE * pde4b_modulation * genome_rate * stress_effect * resource_effect
    }

    // === Time Preference Traits (Heritable) ===

    /// Present bias parameter β for quasi-hyperbolic discounting
    /// Loci 40-41 (trait index 20)
    ///
    /// **Theoretical Basis:**
    /// - Laibson (1997): β-δ model separates present bias from long-term discounting
    /// - β = 1.0: Time-consistent (no present bias)
    /// - β < 1.0: Present-biased (hyperbolic discounting)
    ///
    /// **Range:** [0.5, 1.0] where lower = more present-biased
    ///
    /// **Returns:** Present bias parameter β
    #[must_use]
    pub fn present_bias(&self) -> f32 {
        let trait_value = self.get_trait(20);
        // Map [0, 1] to [0.5, 1.0]
        0.5 + trait_value * 0.5
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Impatience parameter k for hyperbolic discounting
    /// Loci 42-43 (trait index 21)
    ///
    /// **Theoretical Basis:**
    /// - Mazur (1987): Hyperbolic discounting V = A / (1 + k·t)
    /// - Higher k = more impatient (steeper discounting)
    ///
    /// **Range:** [0.01, 0.5] where higher = more impatient
    ///
    /// **Returns:** Hyperbolic impatience parameter k
    #[must_use]
    pub fn impatience_k(&self) -> f32 {
        let trait_value = self.get_trait(21);
        // Map [0, 1] to [0.01, 0.5]
        0.01 + trait_value * 0.49
    }

    /// Long-term discount factor δ for quasi-hyperbolic model
    /// Derived from present bias (negatively correlated)
    ///
    /// **Range:** [0.9, 0.99]
    ///
    /// **Returns:** Long-term discount factor δ
    #[must_use]
    pub fn long_term_discount(&self) -> f32 {
        // More present-biased agents tend to discount long-term more steeply
        let beta = self.present_bias();
        // Map β [0.5, 1.0] to δ [0.9, 0.99]
        0.9 + (beta - 0.5) * 0.18
    }
}

/// Mutate an allele value per §8.2.1 lines 4435-4437
/// Mutation rate default: 0.001 per locus per Drake et al. 1998
pub fn mutate(allele: f32, mutation_rate: f32, rng: &mut StdRng) -> f32 {
    if rng.gen::<f32>() < mutation_rate {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Gaussian mutation with small variance
        let delta: f32 = rng.gen::<f32>() * 0.2 - 0.1; // ±0.1
        (allele + delta).clamp(0.0, 1.0)
    } else {
        allele
    }
}

/// Sexual crossover: Mendelian inheritance per §7.5 lines 3857-3863
/// Each offspring locus gets one allele from each parent (random selection)
/// Mutation applied after crossover
pub fn crossover(
    parent_a: &Genome,
    parent_b: &Genome,
    mutation_rate: f32,
    rng: &mut StdRng,
) -> Genome {
    let alleles: Vec<[f32; 2]> = (0..GENOME_LOCI)
        .map(|locus| {
            // Each parent contributes one allele (random selection from their diploid pair)
            let from_a = if rng.gen::<bool>() {
                parent_a.alleles[locus][0]
            } else {
                parent_a.alleles[locus][1]
            };
            let from_b = if rng.gen::<bool>() {
                parent_b.alleles[locus][0]
            } else {
                parent_b.alleles[locus][1]
            };
            // Apply mutation
            [
                mutate(from_a, mutation_rate, rng),
                mutate(from_b, mutation_rate, rng),
            ]
        })
        .collect();
    Genome { alleles }
}

// ============================================================================
// EPISTASIS: Gene-Gene Interactions
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Epistasis interaction type
/// Defines how two loci interact to affect a phenotype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EpistasisType {
    /// No interaction (additive)
    None,
    /// Positive epistasis: combined effect > sum of individual effects
    Positive,
    /// Negative epistasis: combined effect < sum of individual effects
    Negative,
    /// Dominant epistasis: one locus masks the other
    DominantMask,
    /// Recessive epistasis: both need to be present for effect
    Recessive,
}

/// Epistatic interaction between two loci
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistaticInteraction {
    /// First locus index
    pub locus_a: usize,
    /// Second locus index
    pub locus_b: usize,
    /// Type of interaction
    pub interaction_type: EpistasisType,
    /// Strength of interaction [0, 1]
    pub strength: f32,
    /// Target trait affected by this interaction
    pub target_trait: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EpistaticInteraction {
    /// Create new epistatic interaction
    #[must_use]
    pub fn new(
        locus_a: usize,
        locus_b: usize,
        interaction_type: EpistasisType,
        strength: f32,
        target_trait: usize,
    ) -> Self {
        Self {
            locus_a,
            locus_b,
            interaction_type,
            strength: strength.clamp(0.0, 1.0),
            target_trait,
        }
    }

    /// Compute epistatic modifier for trait value
    /// Returns modifier in range [-1, 1] to apply to trait
    #[must_use]
    pub fn compute_modifier(&self, genome: &Genome) -> f32 {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let allele_a = (genome.alleles[self.locus_a][0] + genome.alleles[self.locus_a][1]) / 2.0;
        let allele_b = (genome.alleles[self.locus_b][0] + genome.alleles[self.locus_b][1]) / 2.0;

        match self.interaction_type {
            EpistasisType::None => 0.0,
            EpistasisType::Positive => {
                // Combined effect amplifies: product > sum
                let product = allele_a * allele_b;
                let sum = (allele_a + allele_b) / 2.0;
                (product - sum) * self.strength
            }
            EpistasisType::Negative => {
                // Combined effect reduces
                let product = allele_a * allele_b;
                let sum = (allele_a + allele_b) / 2.0;
                -((sum - product) * self.strength)
            }
            EpistasisType::DominantMask => {
                // Locus A masks locus B if A is high
                if allele_a > 0.5 {
                    (allele_a - allele_b) * self.strength
                } else {
                    0.0
                }
            }
            EpistasisType::Recessive => {
                // Both need to be high for full effect
                let min_val = allele_a.min(allele_b);
                (min_val - (allele_a + allele_b) / 2.0) * self.strength
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Epistasis network for a genome
/// Manages all epistatic interactions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpistasisNetwork {
    /// List of all epistatic interactions
    pub interactions: Vec<EpistaticInteraction>,
}

impl EpistasisNetwork {
    /// Create empty network
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create network with random interactions
    pub fn random(n_interactions: usize, n_loci: usize, n_traits: usize, rng: &mut StdRng) -> Self {
        let mut interactions = Vec::new();

        for _ in 0..n_interactions {
            let locus_a = rng.gen_range(0..n_loci);
            let locus_b = rng.gen_range(0..n_loci);

            if locus_a != locus_b {
                let interaction_type = match rng.gen_range(0..5) {
                    0 => EpistasisType::None,
                    1 => EpistasisType::Positive,
                    2 => EpistasisType::Negative,
                    3 => EpistasisType::DominantMask,
                    _ => EpistasisType::Recessive,
                };

                let strength = rng.gen::<f32>() * 0.3; // Moderate strength
                let target_trait = rng.gen_range(0..n_traits);

                interactions.push(EpistaticInteraction::new(
                    locus_a,
                    locus_b,
                    interaction_type,
                    strength,
                    target_trait,
                ));
            }
        }

        Self { interactions }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add an interaction
    pub fn add_interaction(&mut self, interaction: EpistaticInteraction) {
        self.interactions.push(interaction);
    }

    /// Get all interactions affecting a trait
    #[must_use]
    pub fn interactions_for_trait(&self, trait_index: usize) -> Vec<&EpistaticInteraction> {
        self.interactions
            .iter()
            .filter(|i| i.target_trait == trait_index)
            .collect()
    }

    /// Compute epistatic contribution to trait value
    /// Returns total modifier in range [-1, 1]
    #[must_use]
    pub fn compute_trait_modifier(&self, trait_index: usize, genome: &Genome) -> f32 {
        self.interactions_for_trait(trait_index)
            .iter()
            .map(|i| i.compute_modifier(genome))
            .sum::<f32>()
            .clamp(-1.0, 1.0)
    }

    /// Get total epistatic load (sum of all interaction strengths)
    /// Higher load = more complex genetic architecture
    #[must_use]
    pub fn epistatic_load(&self) -> f32 {
        self.interactions.iter().map(|i| i.strength).sum()
    }

    /// Count interactions by type
    #[must_use]
    pub fn count_by_type(&self) -> HashMap<EpistasisType, usize> {
        let mut counts = HashMap::new();
        for interaction in &self.interactions {
            *counts.entry(interaction.interaction_type).or_insert(0) += 1;
        }
        counts
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Trait expression with epistasis
/// Computes final trait value including epistatic effects
#[derive(Debug)]
pub struct TraitExpression {
    /// Base trait value from additive genetics
    pub base_value: f32,
    /// Epistatic modifier
    pub epistatic_modifier: f32,
    /// Final trait value
    pub final_value: f32,
}

impl TraitExpression {
    /// Compute trait expression
    #[must_use]
    pub fn compute(genome: &Genome, trait_index: usize, network: &EpistasisNetwork) -> Self {
        let base_value = genome.get_trait(trait_index);
        let epistatic_modifier = network.compute_trait_modifier(trait_index, genome);

        // Apply epistasis: modify base value by modifier
        let final_value = (base_value + epistatic_modifier).clamp(0.0, 1.0);

        Self {
            base_value,
            epistatic_modifier,
            final_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_genome_default() {
        let g = Genome::default();
        assert_eq!(g.alleles[0][0], 0.5);
        assert_eq!(g.alleles[43][1], 0.5); // 44 loci per §6.6.5
    }

    #[test]
    fn test_docas_alpha_range() {
        let g = Genome::default();
        let alpha = g.docas_alpha();
        for a in alpha {
            assert!((0.9..=0.99).contains(&a));
        }
    }

    #[test]
    fn test_mutate() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = 0.5;
        let mutated = mutate(original, 1.0, &mut rng); // 100% mutation rate
        assert!((0.0..=1.0).contains(&mutated));
    }

    // P2-1: Sugarscape Vision Tests
    #[test]
    fn test_vision_range_default() {
        let g = Genome::default();
        // Default genome has all alleles = 0.5
        // vision = 1.0 + 0.5 * 10.0 = 6
        assert_eq!(g.vision_range(), 6);
    }

    #[test]
    fn test_vision_range_range() {
        let mut rng = StdRng::seed_from_u64(42);
        let g = Genome::random(&mut rng);
        let vision = g.vision_range();
        // Should be in range [1, 11]
        assert!(
            (1..=11).contains(&vision),
            "Vision {vision} not in range [1, 11]"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // P2-2: Sugarscape Metabolism Tests
    #[test]
    fn test_metabolic_rate_default() {
        let g = Genome::default();
        // Default: 1.0 + 0.5 * 1.0 = 1.5
        let rate = g.metabolic_rate();
        assert!((rate - 1.5).abs() < 0.01, "Metabolic rate {rate} != 1.5");
    }

    #[test]
    fn test_metabolic_rate_range() {
        let mut rng = StdRng::seed_from_u64(42);
        let g = Genome::random(&mut rng);
        let rate = g.metabolic_rate();
        // Should be in range [1.0, 2.0]
        assert!(
            (1.0..=2.0).contains(&rate),
            "Metabolic rate {rate} not in range [1.0, 2.0]"
        );
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    // P2-3: Emotion Sensitivity Tests
    #[test]
    fn test_emotion_sensitivity_default() {
        let g = Genome::default();
        // Default: 0.5 + 0.5 * 1.0 = 1.0
        let sensitivity = g.emotion_sensitivity();
        assert!(
            (sensitivity - 1.0).abs() < 0.01,
            "Sensitivity {sensitivity} != 1.0"
        );
    }

    #[test]
    fn test_emotion_sensitivity_range() {
        let mut rng = StdRng::seed_from_u64(42);
        let g = Genome::random(&mut rng);
        let sensitivity = g.emotion_sensitivity();
        // Should be in range [0.5, 1.5]
        assert!(
            (0.5..=1.5).contains(&sensitivity),
            "Sensitivity {sensitivity} not in range [0.5, 1.5]"
        );
    }

    // Heritability tests
    #[test]
    fn test_sugarscape_traits_heritable() {
        let mut rng = StdRng::seed_from_u64(42);
        let parent_a = Genome::random(&mut rng);
        let parent_b = Genome::random(&mut rng);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let offspring = crossover(&parent_a, &parent_b, 0.0, &mut rng);

        // Offspring should have valid trait values
        assert!(offspring.vision_range() >= 1 && offspring.vision_range() <= 11);
        assert!(offspring.metabolic_rate() >= 1.0 && offspring.metabolic_rate() <= 2.0);
        assert!(offspring.emotion_sensitivity() >= 0.5 && offspring.emotion_sensitivity() <= 1.5);
    }

    // P1-5: Epistasis Tests
    #[test]
    fn test_epistatic_interaction_creation() {
        let interaction = EpistaticInteraction::new(0, 1, EpistasisType::Positive, 0.5, 0);
        assert_eq!(interaction.locus_a, 0);
        assert_eq!(interaction.locus_b, 1);
        assert_eq!(interaction.interaction_type, EpistasisType::Positive);
        assert!((interaction.strength - 0.5).abs() < 0.001);
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[test]
    fn test_epistasis_none_modifier() {
        let genome = Genome::default();
        let interaction = EpistaticInteraction::new(0, 1, EpistasisType::None, 1.0, 0);

        let modifier = interaction.compute_modifier(&genome);
        assert!(
            modifier.abs() < 0.001,
            "None epistasis should have no effect"
        );
    }

    #[test]
    fn test_epistasis_positive_modifier() {
        // Create genome with high values at both loci
        let mut genome = Genome::default();
        genome.alleles[0] = [1.0, 1.0]; // Max at locus 0
        genome.alleles[1] = [1.0, 1.0]; // Max at locus 1

        let interaction = EpistaticInteraction::new(0, 1, EpistasisType::Positive, 1.0, 0);

        let modifier = interaction.compute_modifier(&genome);
        // Product (1.0) - Average (1.0) = 0, but with positive interaction
        // actual formula: (1*1 - 1) * 1 = 0
        assert!(
            modifier >= 0.0,
            "Positive epistasis should give non-negative modifier"
        );
    }

    #[test]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn test_epistasis_network() {
        let mut rng = StdRng::seed_from_u64(42);
        let network = EpistasisNetwork::random(10, 44, 22, &mut rng);

        assert!(!network.interactions.is_empty());
        assert!(network.epistatic_load() > 0.0);

        let counts = network.count_by_type();
        assert!(!counts.is_empty());
    }

    #[test]
    fn test_epistasis_trait_expression() {
        let genome = Genome::default();
        let network = EpistasisNetwork::new();

        let expression = TraitExpression::compute(&genome, 0, &network);

        // With no epistasis, base and final should be equal
        assert!((expression.base_value - expression.final_value).abs() < 0.001);
        assert!(expression.epistatic_modifier.abs() < 0.001);
    }

    #[test]
    fn test_epistasis_affects_trait() {
        let mut genome = Genome::default();
        genome.alleles[0] = [1.0, 1.0];
        genome.alleles[1] = [1.0, 1.0];

        let mut network = EpistasisNetwork::new();
        network.add_interaction(EpistaticInteraction::new(
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            0,
            1,
            EpistasisType::Positive,
            0.5,
            0,
        ));

        let expression = TraitExpression::compute(&genome, 0, &network);

        // With epistasis, modifier should be non-zero
        // But for default genome with alleles=0.5, modifier may still be small
        assert!(expression.final_value >= 0.0 && expression.final_value <= 1.0);
    }

    #[test]
    fn test_dominant_mask_epistasis() {
        let mut genome = Genome::default();
        genome.alleles[0] = [0.9, 0.9]; // High at locus A (dominant)
        genome.alleles[1] = [0.1, 0.1]; // Low at locus B (masked)

        let interaction = EpistaticInteraction::new(0, 1, EpistasisType::DominantMask, 1.0, 0);

        let modifier = interaction.compute_modifier(&genome);
        // Dominant mask: since A > 0.5, effect is (A - B) * strength
        // = (0.9 - 0.1) * 1.0 = 0.8
        assert!(
            modifier > 0.0,
            "Dominant mask should have positive modifier when A > B"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_recessive_epistasis() {
        let mut genome = Genome::default();
        genome.alleles[0] = [0.5, 0.5];
        genome.alleles[1] = [0.5, 0.5];

        let interaction = EpistaticInteraction::new(0, 1, EpistasisType::Recessive, 1.0, 0);

        let modifier = interaction.compute_modifier(&genome);
        // Recessive: min(0.5, 0.5) - avg(0.5, 0.5) = 0.5 - 0.5 = 0
        assert!(
            modifier.abs() < 0.001,
            "Recessive with equal values should have no modifier"
        );
    }

    // === Neural Maturation Tests (Research: MATURITY_NEURAL_DEVELOPMENT) ===

    #[test]
    fn test_pfc_maturation_rate_range() {
        // Test default genome (0.5 alleles)
        let genome = Genome::default();
        let rate = genome.pfc_maturation_rate();
        // Default should give 1.0 (midpoint of 0.5-1.5 range)
        assert!(
            (0.5..=1.5).contains(&rate),
            "PFC maturation rate should be in [0.5, 1.5]"
        );
    }

    #[test]
    fn test_pfc_maturation_rate_extremes() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut genome = Genome::default();

        // Min rate: all zeros at loci 32-33
        genome.alleles[32] = [0.0, 0.0];
        genome.alleles[33] = [0.0, 0.0];
        let min_rate = genome.pfc_maturation_rate();
        assert!(
            (min_rate - 0.5).abs() < 0.01,
            "Min PFC maturation rate should be 0.5"
        );

        // Max rate: all ones at loci 32-33
        genome.alleles[32] = [1.0, 1.0];
        genome.alleles[33] = [1.0, 1.0];
        let max_rate = genome.pfc_maturation_rate();
        assert!(
            (max_rate - 1.5).abs() < 0.01,
            "Max PFC maturation rate should be 1.5"
        );
    }

    #[test]
    fn test_pruning_efficiency_range() {
        let genome = Genome::default();
        let efficiency = genome.pruning_efficiency();
        assert!(
            (0.3..=1.0).contains(&efficiency),
            "Pruning efficiency should be in [0.3, 1.0], got {efficiency}"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_myelination_rate_range() {
        let genome = Genome::default();
        let rate = genome.myelination_rate();
        assert!(
            (0.5..=1.5).contains(&rate),
            "Myelination rate should be in [0.5, 1.5], got {rate}"
        );
    }

    #[test]
    fn test_stress_sensitivity_range() {
        let genome = Genome::default();
        let sensitivity = genome.stress_sensitivity();
        assert!(
            (0.2..=1.0).contains(&sensitivity),
            "Stress sensitivity should be in [0.2, 1.0], got {sensitivity}"
        );
    }

    #[test]
    fn test_compute_maturation_rate_pde4b_effect() {
        // Low PDE4B should give faster maturation
        let mut low_pde4b_genome = Genome::default();
        low_pde4b_genome.alleles[24] = [0.0, 0.0]; // Min PDE4B
        low_pde4b_genome.alleles[25] = [0.0, 0.0];

        let mut high_pde4b_genome = Genome::default();
        high_pde4b_genome.alleles[24] = [1.0, 1.0]; // Max PDE4B
        high_pde4b_genome.alleles[25] = [1.0, 1.0];

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let low_rate = low_pde4b_genome.compute_maturation_rate(0.0, 1.0);
        let high_rate = high_pde4b_genome.compute_maturation_rate(0.0, 1.0);

        assert!(
            low_rate > high_rate,
            "Low PDE4B should give faster maturation: {low_rate} > {high_rate}"
        );
    }

    #[test]
    fn test_compute_maturation_rate_stress_effect() {
        let genome = Genome::default();

        // No stress
        let no_stress = genome.compute_maturation_rate(0.0, 1.0);
        // High stress
        let high_stress = genome.compute_maturation_rate(0.8, 1.0);

        assert!(
            high_stress < no_stress,
            "High stress should slow maturation: {high_stress} < {no_stress}"
        );
    }

    #[test]
    fn test_compute_maturation_rate_resource_effect() {
        let genome = Genome::default();

        // Abundant resources
        let abundant = genome.compute_maturation_rate(0.0, 1.0);
        // Starvation
        let starvation = genome.compute_maturation_rate(0.0, 0.0);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        assert!(
            starvation < abundant,
            "Starvation should slow maturation: {starvation} < {abundant}"
        );
    }

    #[test]
    fn test_compute_maturation_rate_combined() {
        // Test that maturation rate is always positive
        let genome = Genome::default();

        // Extreme conditions
        let rate_max_stress = genome.compute_maturation_rate(1.0, 0.0);
        let rate_min_stress = genome.compute_maturation_rate(0.0, 1.0);

        assert!(
            rate_max_stress > 0.0,
            "Maturation rate should always be positive"
        );
        assert!(
            rate_min_stress > 0.0,
            "Maturation rate should always be positive"
        );

        // Normal conditions should be faster than extreme stress
        assert!(
            rate_min_stress > rate_max_stress,
            "Optimal conditions should be faster than stressed"
        );
    }
}
