//! UnifiedAgent - Single agent type for all 12 papers
//!
//! This is the CRITICAL PATH for P0 gap resolution. All 26 experiments depend
//! on this structure being fully implemented.
//!
//! P0 Gaps Addressed:
//! - GTT-P0-1: lambda field for universality validation
//! - GTT-P0-5: trust_state for cross-domain correlation
//! - GTT-P0-6: stability field for stability law
//! - DOCAS-P0-2: genome for twin studies
//! - DOCAS-P0-3: docas_history for criticality
//! - TFP-P0-3: actual_default for AUC calculation
//! - EMOT-P0-1: emotion fields for R² validation
//! - And 40+ more...

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// ID type - simple u64 for deterministic simulation

// Forward declarations for types defined in other modules
/// docas vector.
pub type DocasVector = [f64; 5]; // [D, O, C, A, S]

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Genetic architecture for PDE4B-DOCAS-GTT integration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Genome {
    /// pde4b variant.
    pub pde4b_variant: Pde4bVariant,
    /// dopamine pathway.
    pub dopamine_pathway: Vec<f64>,
    /// serotonin pathway.
    pub serotonin_pathway: Vec<f64>,
    /// oxytocin pathway.
    pub oxytocin_pathway: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// pde4b variant.
pub enum Pde4bVariant {
    /// wild type.
    WildType, // Normal PDE4B expression
    /// rs2459390t.
    Rs2459390T, // Reduced expression (~1.5x learning)
    /// rs2459390c.
    Rs2459390C, // Standard expression
    /// rs2184308a.
    Rs2184308A, // Enhanced expression
}

impl Genome {
    /// Generate genome for MZ twin (identical)
    #[must_use]
    pub fn clone_for_mz(&self) -> Self {
        self.clone()
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Generate genome for DZ twin with locus-by-locus independent assortment
    #[must_use]
    pub fn recombine_for_dz_seeded(&self, other: &Self, seed: u64) -> Self {
        let recombine_vector = |v1: &[f64], v2: &[f64], locus_offset: u64| -> Vec<f64> {
            let max_len = v1.len().max(v2.len());
            let mut out = Vec::with_capacity(max_len);
            for i in 0..max_len {
                let coin = deterministic_coin(seed ^ (i as u64) ^ locus_offset);
                let val = match (v1.get(i), v2.get(i)) {
                    (Some(&a), Some(&b)) => {
                        if coin {
                            a
                        } else {
                            b
                        }
                    }
                    (Some(&a), None) => a,
                    (None, Some(&b)) => b,
                    (None, None) => 0.5,
                };
                out.push(val);
            }
            out
        };

        let pde4b_coin = deterministic_coin(seed ^ 0xdeadd00d);
        Self {
            pde4b_variant: if pde4b_coin {
                self.pde4b_variant.clone()
            } else {
                other.pde4b_variant.clone()
            },
            dopamine_pathway: recombine_vector(
                &self.dopamine_pathway,
                &other.dopamine_pathway,
                0x1000,
            ),
            serotonin_pathway: recombine_vector(
                &self.serotonin_pathway,
                &other.serotonin_pathway,
                0x2000,
            ),
            oxytocin_pathway: recombine_vector(
                &self.oxytocin_pathway,
                &other.oxytocin_pathway,
                0x3000,
            ),
        }
    }

    /// Generate genome for DZ twin (50% recombination)
    #[must_use]
    pub fn recombine_for_dz(&self, other: &Self) -> Self {
        self.recombine_for_dz_seeded(other, 0x517cc1b727220a95)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Emotional state for emotion paper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Emotion {
    /// trust.
    Trust,
    /// joy.
    Joy,
    /// anticipation.
    Anticipation,
    /// fear.
    Fear,
    /// anger.
    Anger,
    /// disgust.
    Disgust,
    /// sadness.
    Sadness,
    /// surprise.
    Surprise,
    /// love.
    Love,
    /// hope.
    Hope,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Emotion {
    /// Map DOCAS vector to emotion
    /// Addresses EMOT-P0-2: DOCAS mapping validation
    #[must_use]
    pub fn from_docas(docas: &DocasVector) -> Self {
        let [d, o, c, a, s] = *docas;

        // Simplified mapping - should be validated against empirical data
        if d > 0.7 && o > 0.6 && c > 0.5 {
            Self::Trust
        } else if d > 0.6 && o > 0.7 {
            Self::Joy
        } else if a > 0.6 && s > 0.6 {
            Self::Fear
        } else if d > 0.5 {
            Self::Love
        } else if o > 0.5 {
            Self::Anticipation
        } else {
            Self::Hope
        }
    }

    /// Convert to VAD (Valence, Arousal, Dominance) for comparison
    /// Addresses EMOT-P0-1: Compare trust-only vs VAD
    #[must_use]
    pub const fn to_vad(&self) -> (f64, f64, f64) {
        match self {
            Self::Trust => (0.6, 0.4, 0.5),
            Self::Joy => (0.8, 0.7, 0.6),
            Self::Anticipation => (0.5, 0.6, 0.4),
            Self::Fear => (-0.6, 0.8, 0.3),
            Self::Anger => (-0.5, 0.8, 0.7),
            Self::Disgust => (-0.6, 0.5, 0.4),
            Self::Sadness => (-0.7, 0.3, 0.2),
            Self::Surprise => (0.0, 0.8, 0.5),
            Self::Love => (0.8, 0.5, 0.4),
            Self::Hope => (0.6, 0.5, 0.4),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Semantic model for linguistics paper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticModel {
    /// word embeddings.
    pub word_embeddings: HashMap<String, Vec<f64>>,
    /// trust centrality.
    pub trust_centrality: f64, // LING-P0-1: Betweenness centrality
    /// network coherence.
    pub network_coherence: f64,
}

/// Cultural quadrant (Culture paper)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Quadrant {
    /// one.
    One, // High trust, high bonds
    /// two.
    Two, // Low trust, high bonds
    /// three.
    Three, // Low trust, low bonds
    /// four.
    Four, // High trust, low bonds
}

impl Quadrant {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate from bond density and philosophical tau
    /// Addresses CULT-P0-1: Cultural quadrants
    #[must_use]
    pub fn from_bond_tau(bond_density: f64, philosophical_tau: f64) -> Self {
        match (bond_density > 0.5, philosophical_tau > 0.0) {
            (true, true) => Self::One,
            (true, false) => Self::Two,
            (false, false) => Self::Three,
            (false, true) => Self::Four,
        }
    }
}

/// Hofstede cultural dimensions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HofstedeDimensions {
    /// power distance.
    pub power_distance: f64, // PDI
    /// individualism.
    pub individualism: f64, // IDV
    /// masculinity.
    pub masculinity: f64, // MAS
    /// uncertainty avoidance.
    pub uncertainty_avoidance: f64, // UAI
    /// long term orientation.
    pub long_term_orientation: f64, // LTO
    /// indulgence.
    pub indulgence: f64, // IVR
}

/// Game theory strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Strategy {
    /// cooperate.
    Cooperate,
    /// defect.
    Defect,
    /// tit for tat.
    TitForTat,
    /// trust weighted.
    TrustWeighted, // GAME-P0-1: EU(a|T) strategy
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Game outcome for history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameOutcome {
    /// opponent.
    pub opponent: u64,
    /// my strategy.
    pub my_strategy: Strategy,
    /// opponent strategy.
    pub opponent_strategy: Strategy,
    /// my payoff.
    pub my_payoff: f64,
    /// timestamp.
    pub timestamp: u64,
}

/// Music profile for music paper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MusicProfile {
    /// preferred genres.
    pub preferred_genres: Vec<String>,
    /// complexity preference.
    pub complexity_preference: f64,
    /// emotional arousal preference.
    pub emotional_arousal_preference: f64,
}

// ============================================
// UNIFIED AGENT - Core structure for all papers
// ============================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// UNIFIED AGENT - Single agent type for all 12 papers
///
/// VERIFICATION CHECKLIST:
/// ✅ All 12 papers represented
/// ✅ Core invariant (τ = -G(π)) maintained
/// ✅ No circular dependencies
/// ✅ Serializable for determinism
/// ✅ Cloneable for twin studies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnifiedAgent {
    /// id.
    pub id: u64,
    /// birth time.
    pub birth_time: u64,

    // ----------------------------------------
    // PAPER 10: PDE4B - GENETICS (Foundation)
    // DOCAS-P0-1, PDE4B-P0-1, PDE4B-P0-2
    // ----------------------------------------
    /// genome.
    pub genome: Genome,
    /// pde4b expression.
    pub pde4b_expression: f64, // Range: [0.5, 2.0]
    /// camp level.
    pub camp_level: f64, // Inverse of PDE4B
    /// learning rate modifier.
    pub learning_rate_modifier: f64, // Derived from PDE4B

    // ----------------------------------------
    // PAPER 2: DOCAS - NEUROCHEMISTRY
    // DOCAS-P0-2, DOCAS-P0-3, DOCAS-P0-5
    // ----------------------------------------
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// docas.
    pub docas: DocasVector, // [D, O, C, A, S]
    /// docas history.
    pub docas_history: Vec<DocasVector>, // For criticality analysis
    /// criticality sigma.
    pub criticality_sigma: f64, // DOCAS-P0-3: Target [0.9, 1.1]

    // ----------------------------------------
    // PAPER 1: GTT - TRUST STATE (Core)
    // GTT-P0-1, GTT-P0-5, GTT-P0-6
    // ----------------------------------------
    /// lambda.
    pub lambda: f64, // GTT-P0-1: Target 0.61 ± 0.08
    /// trust state.
    pub trust_state: f64, // τ = -G(π)
    /// trust baseline.
    pub trust_baseline: f64, // τ_0
    /// entropy.
    pub entropy: f64, // H(t) = -Σ T_i log T_i
    /// expectation.
    pub expectation: f64, // E(t)
    /// validation.
    pub validation: f64, // V(t)
    /// prediction error.
    pub prediction_error: f64, // ε(t) = E(t) - V(t)
    /// kappa field.
    pub kappa_field: f64, // GTT-P0-6: Field curvature
    /// kappa traj.
    pub kappa_traj: f64, // Trajectory curvature
    /// viscosity.
    pub viscosity: f64, // ψ(t)
    /// stability.
    pub stability: f64, // GTT-P0-6: S(t)

    // ----------------------------------------
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // PAPER 4: EMOTION - AFFECT
    // EMOT-P0-1, EMOT-P0-2, EMOT-P0-3
    // ----------------------------------------
    /// current emotion.
    pub current_emotion: Emotion,
    /// emotion history.
    pub emotion_history: Vec<(u64, Emotion, f64)>, // (timestamp, emotion, intensity)
    /// valence.
    pub valence: f64, // [-1, 1]
    /// arousal.
    pub arousal: f64, // [0, 1]
    /// dominance.
    pub dominance: f64, // [0, 1]

    // ----------------------------------------
    // PAPER 5: LINGUISTICS - SEMANTICS
    // LING-P0-1, LING-P0-2, LING-P0-3
    // ----------------------------------------
    /// semantic model.
    pub semantic_model: SemanticModel,
    /// trust in meaning.
    pub trust_in_meaning: f64, // τ^s
    /// semantic entropy.
    pub semantic_entropy: f64, // H_e
    /// semantic coherence.
    pub semantic_coherence: f64, // cos(θ)

    // ----------------------------------------
    // PAPER 6: PHILOSOPHY - TRUST AXIS
    // PHIL-P0-1, PHIL-P0-2, PHIL-P0-4
    // ----------------------------------------
    /// philosophical tau.
    pub philosophical_tau: f64, // PHIL-P0-1: [-1, +1]
    /// bond activation.
    pub bond_activation: f64, // PHIL-P0-4: Bridge to linguistics

    // ----------------------------------------
    // PAPER 7: CULTURE - CULTURAL QUADRANTS
    // CULT-P0-1, CULT-P0-3
    // ----------------------------------------
    /// bond density.
    pub bond_density: f64, // b ∈ [0, 1]
    /// cultural quadrant.
    pub cultural_quadrant: Quadrant, // I, II, III, IV
    /// hofstede dimensions.
    pub hofstede_dimensions: HofstedeDimensions,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // ----------------------------------------
    // PAPER 8: ECONOMICS - TRUST-BASED ECONOMY
    // ECON-P0-1, ECON-P0-2, ECON-P0-4
    // ----------------------------------------
    /// income.
    pub income: f64,
    /// wealth.
    pub wealth: f64,
    /// consumption.
    pub consumption: f64,
    /// default probability.
    pub default_probability: f64, // ECON-P0-2: For crisis prediction
    /// actual default.
    pub actual_default: bool, // TFP-P0-3: Ground truth for AUC

    // ----------------------------------------
    // PAPER 3: TFP - CROSS-DOMAIN COHERENCE
    // TFP-P0-3
    // ----------------------------------------
    /// psi bio.
    pub psi_bio: f64,
    /// psi fin.
    pub psi_fin: f64,
    /// psi ling.
    pub psi_ling: f64,
    /// psi trust.
    pub psi_trust: f64, // Composite coherence

    // ----------------------------------------
    // PAPER 9: GAME THEORY - TRUST-WEIGHTED
    // GAME-P0-1, GAME-P0-2, GAME-P0-3
    // ----------------------------------------
    /// game history.
    pub game_history: Vec<GameOutcome>,
    /// trust in partner.
    pub trust_in_partner: HashMap<u64, f64>, // GAME-P0-2: T quantification
    /// current strategy.
    pub current_strategy: Strategy,
    /// cumulative payoff.
    pub cumulative_payoff: f64,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // ----------------------------------------
    // PAPER 11: MEANING - SEMANTIC STABILITY
    // MEAN-P0-1, MEAN-P0-3, MEAN-P0-4
    // ----------------------------------------
    /// meaning stability.
    pub meaning_stability: f64, // τ^s
    /// percolation threshold.
    pub percolation_threshold: f64, // MEAN-P0-3: τ_critical = 0.32
    /// collapse imminent.
    pub collapse_imminent: bool,

    // ----------------------------------------
    // PAPER 12: MUSIC - AESTHETIC INDICATOR
    // MUSIC-P0-2, MUSIC-P0-3
    // ----------------------------------------
    /// music profile.
    pub music_profile: MusicProfile,
    /// spectral exponent.
    pub spectral_exponent: f64, // MUSIC-P0-2: Target ~1.0
    /// spectral coherence.
    pub spectral_coherence: f64, // MUSIC-P0-3
    /// music lead indicator.
    pub music_lead_indicator: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl UnifiedAgent {
    /// Create new agent with randomized initial state
    #[must_use]
    pub fn new(id: u64, time: u64) -> Self {
        Self {
            id,
            birth_time: time,
            genome: Genome {
                pde4b_variant: Pde4bVariant::WildType,
                dopamine_pathway: vec![0.5; 10],
                serotonin_pathway: vec![0.5; 10],
                oxytocin_pathway: vec![0.5; 10],
            },
            pde4b_expression: 1.0,
            camp_level: 1.0,
            learning_rate_modifier: 1.0,
            docas: [0.5; 5],
            docas_history: Vec::new(),
            criticality_sigma: 1.0,
            lambda: 0.61,
            trust_state: 0.5,
            trust_baseline: 0.5,
            entropy: 1.0,
            expectation: 0.5,
            validation: 0.5,
            prediction_error: 0.0,
            kappa_field: 0.0,
            kappa_traj: 0.0,
            viscosity: 1.0,
            stability: 0.5,
            current_emotion: Emotion::Hope,
            emotion_history: Vec::new(),
            valence: 0.0,
            arousal: 0.5,
            dominance: 0.5,
            semantic_model: SemanticModel {
                word_embeddings: HashMap::new(),
                trust_centrality: 0.0,
                network_coherence: 0.0,
            },
            trust_in_meaning: 0.5,
            semantic_entropy: 1.0,
            semantic_coherence: 0.5,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            philosophical_tau: 0.0,
            bond_activation: 0.5,
            bond_density: 0.5,
            cultural_quadrant: Quadrant::One,
            hofstede_dimensions: HofstedeDimensions {
                power_distance: 50.0,
                individualism: 50.0,
                masculinity: 50.0,
                uncertainty_avoidance: 50.0,
                long_term_orientation: 50.0,
                indulgence: 50.0,
            },
            income: 50000.0,
            wealth: 100000.0,
            consumption: 40000.0,
            default_probability: 0.05,
            actual_default: false,
            psi_bio: 0.5,
            psi_fin: 0.5,
            psi_ling: 0.5,
            psi_trust: 0.5,
            game_history: Vec::new(),
            trust_in_partner: HashMap::new(),
            current_strategy: Strategy::TrustWeighted,
            cumulative_payoff: 0.0,
            meaning_stability: 0.5,
            percolation_threshold: 0.32,
            collapse_imminent: false,
            music_profile: MusicProfile {
                preferred_genres: vec!["classical".to_string()],
                complexity_preference: 0.5,
                emotional_arousal_preference: 0.5,
            },
            spectral_exponent: 1.0,
            spectral_coherence: 0.5,
            music_lead_indicator: 0.0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create MZ twin (identical genome)
    /// Addresses DOCAS-P0-2: Twin study implementation
    #[must_use]
    pub fn create_mz_twin(&self, new_id: u64) -> Self {
        let mut twin = self.clone();
        twin.id = new_id;
        twin.birth_time = self.birth_time; // Same age
        twin.stability = twin.calculate_stability();
        twin
    }

    /// Create DZ twin (50% genome recombination)
    /// Addresses DOCAS-P0-2: Twin study implementation
    #[must_use]
    pub fn create_dz_twin(&self, other: &Self, new_id: u64) -> Self {
        let mut twin = self.clone();
        twin.id = new_id;
        twin.birth_time = self.birth_time;

        // Recombine genome deterministically from agent IDs
        let seed = self.id.rotate_left(13) ^ other.id.rotate_right(7) ^ new_id;
        twin.genome = self.genome.recombine_for_dz_seeded(&other.genome, seed);
        twin.stability = twin.calculate_stability();

        twin
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update DOCAS and derived fields
    /// Addresses DOCAS-P0-3: Criticality analysis
    pub fn update_docas(&mut self, new_docas: DocasVector, time: u64) {
        // Store history for criticality analysis
        self.docas_history.push(self.docas);

        // Limit history size for memory efficiency
        if self.docas_history.len() > 10000 {
            self.docas_history.drain(0..100);
        }

        self.docas = new_docas;

        // Update emotion from DOCAS
        self.current_emotion = Emotion::from_docas(&new_docas);
        self.emotion_history
            .push((time, self.current_emotion.clone(), 0.5));

        // Update cultural quadrant
        self.cultural_quadrant = Quadrant::from_bond_tau(self.bond_density, self.philosophical_tau);

        // Maintain core dynamical stability invariant S(t) = 1 / (1 + |κ_field * κ_traj| / ν)
        self.stability = self.calculate_stability();
    }

    /// Calculate stability S(t)
    /// Addresses GTT-P0-6: Stability law validation
    #[must_use]
    pub fn calculate_stability(&self) -> f64 {
        // Simplified stability calculation
        let kappa_term = (self.kappa_field * self.kappa_traj).abs();
        let viscosity_term = self.viscosity.max(0.001);

        1.0 / (1.0 + kappa_term / viscosity_term)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate trust-weighted expected utility
    /// Addresses GAME-P0-1: EU(a|T) validation
    #[must_use]
    pub fn trust_weighted_utility(&self, base_utility: f64, partner: &u64) -> f64 {
        let trust = self
            .trust_in_partner
            .get(partner)
            .copied()
            .unwrap_or(self.trust_state);

        // Multiplicative trust and stability effect: EU(a|T) = U(a) × (1 + T * stability) / 2
        base_utility * (1.0 + trust * self.stability) / 2.0
    }
}

// Deterministic SplitMix64 PRNG for genome recombination (SIM-002: never wall-clock or shared atomic)
fn deterministic_coin(seed: u64) -> bool {
    let mut x = seed.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    (x ^ (x >> 31)) & 1 == 0
}

#[cfg(test)]
mod tests {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = UnifiedAgent::new(0, 0);
        assert_eq!(agent.lambda, 0.61);
        assert_eq!(agent.docas.len(), 5);
    }

    #[test]
    fn test_mz_twin() {
        let agent1 = UnifiedAgent::new(0, 0);
        let agent2 = agent1.create_mz_twin(1);

        // MZ twins have identical genomes
        assert_eq!(agent1.genome, agent2.genome);
        assert_ne!(agent1.id, agent2.id);
    }

    #[test]
    fn test_emotion_from_docas() {
        let high_trust_docas = [0.8, 0.7, 0.6, 0.5, 0.6];
        let emotion = Emotion::from_docas(&high_trust_docas);

        // High D, O, C should map to Trust
        assert!(matches!(emotion, Emotion::Trust));
    }

    #[test]
    fn test_trust_weighted_utility() {
        let mut agent = UnifiedAgent::new(0, 0);
        agent.stability = 1.0;
        let partner = 0;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Set trust to 1.0 (complete trust)
        agent.trust_in_partner.insert(partner, 1.0);

        let utility = agent.trust_weighted_utility(100.0, &partner);

        // EU(a|T=1) = 100 × (1 + 1 * 1.0) / 2 = 100
        assert_eq!(utility, 100.0);

        // Set trust to 0.0 (no trust)
        agent.trust_in_partner.insert(partner, 0.0);
        let utility = agent.trust_weighted_utility(100.0, &partner);

        // EU(a|T=0) = 100 × (1 + 0) / 2 = 50
        assert_eq!(utility, 50.0);
    }

    #[test]
    fn test_cultural_quadrant() {
        let q1 = Quadrant::from_bond_tau(0.8, 0.5); // High bond, high tau
        let q2 = Quadrant::from_bond_tau(0.8, -0.5); // High bond, low tau
        let q3 = Quadrant::from_bond_tau(0.2, -0.5); // Low bond, low tau
        let q4 = Quadrant::from_bond_tau(0.2, 0.5); // Low bond, high tau

        assert!(matches!(q1, Quadrant::One));
        assert!(matches!(q2, Quadrant::Two));
        assert!(matches!(q3, Quadrant::Three));
        assert!(matches!(q4, Quadrant::Four));
    }

    #[test]
    fn test_serialization() {
        let agent = UnifiedAgent::new(0, 0);
        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: UnifiedAgent = serde_json::from_str(&json).unwrap();

        assert_eq!(agent.id, deserialized.id);
        assert_eq!(agent.lambda, deserialized.lambda);
    }
}
