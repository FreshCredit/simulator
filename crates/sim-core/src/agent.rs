//! Agent module: Core agent state and behavior
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION agents, NOT `FreshCredit` AI agents (AGENT-* rules)
//!
//! Per §7.5.1 lines 3878-3960: Formal Agent State Definition

// TAG: surface=simulation owner=research-team rule=SIM-001
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::cognition::WorkingMemory;
use crate::docas::{DocasState, TrustCurvature, TrustViscosity, CURVATURE_WINDOW_SIZE};
use crate::economics::{EconomicState, TimePreferenceProfile};
use crate::epigenome::Epigenome;
use crate::genome::Genome;
use crate::identity::{CreditReport, Identity, RiskProfile};
use crate::language::Lexicon;
use crate::neural::pde4b_pathway::{compute_pathway, PDE4BPathwayParams};
use crate::protocol::IdentityTheftEvent;
use crate::psychology::PersonalityTraits;
use crate::types::{
    AgentId, AttestationId, Energy, GroupId, NormId, Position, Sex, StrategyType, Tick,
};

pub mod types;

pub use types::{
    CulturalQuadrant, DLPFCState, GameOutcome, HofstedeDimensions, PrefrontalCortex, VMPFCState,
};

/// Core Agent State Vector
/// Per §7.5.1 lines 3884-3924
#[derive(Debug, Clone, Serialize, Deserialize)]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
pub struct Agent {
    /// Unique agent identifier
    pub id: AgentId,

    // === Spatial (Stage 0: Geography) ===
    /// Grid coordinates (2D for Phase 0, 3D deferred to Phase 2+)
    /// Per §7.5.1 line 3886
    pub position: Position,

    // === Biological (Stage 1: Biology) ===
    /// Current energy level (mortality check in §6.21)
    /// Per §7.5.1 line 3889
    pub energy: Energy,

    /// Ticks since birth
    /// Per §7.5.1 line 3890
    pub age: Tick,

    /// Sex for reproduction pairing (per SIM-001)
    /// Required for Stage 1: Biology reproduction rules
    pub sex: Sex,

    /// Heritable traits (40 loci, see §7.5)
    /// Per §7.5.1 line 3891
    pub genome: Genome,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Sugarscape vision range per Epstein & Axtell 1996 (P2-1)
    /// Determines how many grid cells the agent can see in each direction
    /// Range: 1-11 cells (inherited from genome loci 26-27)
    pub vision_range: u32,

    /// Sugarscape metabolic rate per Epstein & Axtell 1996 (P2-2)
    /// Energy consumed per tick
    /// Range: 1.0-2.0 energy/tick (inherited from genome loci 28-29)
    pub metabolic_rate: f32,

    // === Cognitive (Stage 2: Cognition) ===
    /// [D, O, C, A, S] neuromodulator state
    /// Per §7.5.1 line 3894
    pub docas: DocasState,

    /// K-rollout operations available per tick
    /// Per §7.5.1 line 3895
    pub dmn_budget: u32,

    /// H-horizon for DMN planning (steps lookahead)
    /// Per SIM-008: Configurable planning horizon
    pub dmn_horizon: u32,

    /// Branching factor (target: 1.0 ± 0.1)
    /// Per §7.5.1 line 3896
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub criticality_sigma: f32,

    /// Cognitive system for dual-process cognition (SIM-008)
    /// System1 = fast heuristic, System2 = slow analytical
    pub cognitive_system: crate::types::CognitiveSystem,

    /// τ₀ homeostatic trust baseline
    /// Per §7.5.1 line 3897
    pub emotion_baseline: f32,

    // === Social (Stage 3: Society) ===
    /// Computed from attestation network (§6.22)
    /// Per §7.5.1 line 3900
    pub reputation_score: f32,

    /// Pairwise local reputation tracking (SIM-004)
    /// Maps `agent_id` -> `reputation_score` for direct interactions
    pub local_reputation: std::collections::HashMap<AgentId, f32>,

    /// Current group membership
    /// Per §7.5.1 line 3901
    pub group_id: Option<GroupId>,

    /// Inherited/learned norm compliance
    /// Per §7.5.1 line 3902
    pub cultural_traits: Vec<NormId>,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Computed from interaction patterns (§6.21)
    /// Per §7.5.1 line 3903
    pub isolation_score: f32,

    // === Attestation (Stage 3: Society, §6.22) ===
    /// Attestations this agent has made
    /// Per §7.5.1 line 3907
    pub attestations_given: Vec<AttestationId>,

    /// Attestations about this agent
    /// Per §7.5.1 line 3908
    pub attestations_received: Vec<AttestationId>,

    // === Linguistic (Stage 4: Language) ===
    /// Agent's vocabulary mapping concepts to embeddings
    /// Per §7.5.1 line 4014
    pub lexicon: Lexicon,

    // === Protocol (Stage 5: Protocol) ===
    /// Whether agent has blockchain-verified identity
    /// Per §7.5.1 line 3915
    pub has_verified_id: bool,

    /// Number of verifications performed
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Per §7.5.1 line 3916
    pub verification_count: u32,

    // === Memory ===
    /// Current game-theoretic strategy
    /// Per §7.5.1 line 3920
    pub strategy: StrategyType,

    // === Identity Theft Status (§6.23) ===
    /// Whether this agent is currently a victim of identity theft
    /// Per §7.5.1 line 3923
    pub is_identity_theft_victim: bool,

    /// Active identity theft event (if victim)
    /// Per §6.23.2: Contains severity, loss, discovery delay
    pub identity_theft_event: Option<IdentityTheftEvent>,

    /// Identity system with claims, verification, and theft status (SIM-010)
    /// Per §6.23: Full identity anchoring and theft mechanics
    pub identity: Option<Identity>,

    /// Tick when recovery from identity theft will complete
    /// Per §6.23.5: Recovery time depends on protocol protection
    pub theft_recovery_tick: Option<Tick>,

    /// Credit report with error status (GAP-002)
    /// Per `empirical_parameters`: 44% error rate, 14.5% serious errors
    pub credit_report: CreditReport,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Tick when recovery from credit report errors will complete
    pub credit_error_recovery_tick: Option<Tick>,

    /// Risk profile for identity theft victimization
    /// Per BJS 2016: 10% annual, risk-weighted by demographics
    pub risk_profile: RiskProfile,

    /// Engagement network connections
    /// Agents this agent has interacted with (for breach propagation)
    pub engagement_network: Vec<AgentId>,

    /// Tick when agent was created/born
    pub birth_tick: Tick,

    /// Whether agent is alive
    pub alive: bool,

    // === Trust Vector Field Curvature (§6.6.2) ===
    /// History of DOCAS states for curvature computation
    /// Per §6.6.2: Tracks trajectory through DOCAS space
    pub docas_history: VecDeque<DocasState>,

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Trust curvature analyzer
    /// Per §6.6.2: Computes κ(t) = ||d²τ/dt²||
    pub trust_curvature: TrustCurvature,

    // === Non-Newtonian Viscosity (§6.6.3) ===
    /// Trust viscosity parameters for stress-dependent adaptation
    /// Per §6.6.3: ψ(t) = α × σ(β × |E(t)|) + (1-α) × τ
    pub trust_viscosity: TrustViscosity,

    /// Recent tau values for volatility computation (window of 20 ticks)
    /// Per §6.6.3: tracks |E(t)| = environmental volatility
    pub tau_history: Vec<f32>,

    /// Current adaptation rate (1/ψ)
    /// Updated each tick based on recent volatility
    pub adaptation_rate: f32,

    // === Interaction History (for reputation-based DOCAS modulation) ===
    // NOTE: parent_trust_propensity was REMOVED per trust architecture review.
    // Trust inheritance is Mendelian (genome → DOCAS sensitivity), not Lamarckian (direct trust transfer).
    // The genome already provides heritable trust dynamics via docas_alpha() and docas_beta().
    /// Count of positive interactions (cooperation, successful trades)
    /// Used to modulate social validation signal in DOCAS updates
    pub positive_interactions: u32,

    /// Count of negative interactions (defection, failed trades)
    /// Used to modulate threat/stress signal in DOCAS updates
    pub negative_interactions: u32,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Interaction history for game strategy (per partner agent)
    /// Maps `partner_id` -> (`my_last_action`, tick)
    /// true = cooperated, false = defected
    /// Used by `TitForTat` and other history-dependent strategies
    pub interaction_history: std::collections::HashMap<AgentId, (bool, Tick)>,

    // === Economic State (SIM-ECON-001) ===
    /// Agent's economic state (credit, debt, innovation, wealth)
    /// Per docs/simulation/economics.md: Unified Theory of Economics
    pub economics: EconomicState,

    // === Debt Obligations (SIM-002) ===
    /// Outstanding debts owed to other agents
    /// Per SIM-002: Debt mechanics for economy stage
    pub debt_obligations: Vec<crate::types::DebtRecord>,

    // === Music State (Stage 4+) ===
    /// Agent's music preferences and listening history
    /// Per music.md: Music as geometric trust indicator
    /// Only populated when `music_enabled` toggle is true
    pub music_state: Option<crate::music::MusicState>,

    // === Emotion State (Interpretability) ===
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Agent's emotional state for DOCAS→Emotion reverse projection
    /// Per §6.13: All emotion derives from trust deviation Δτ = τ - τ₀
    pub emotion_state: crate::emotion::EmotionState,

    // === Language Drift Tracking (LING-C1) ===
    /// Tracks semantic drift rate ∝ 1/τ
    /// Per §6.11.2: `drift_rate` = `cosine_distance(current`, initial) / ticks
    pub language_drift: crate::language_drift::LanguageDriftTracker,

    // === Psychology Layer (Stage 2) ===
    /// Semantic state vector for conceptual alignment
    /// Per Layer 5: Psychology → affects worldview and social orientation
    pub semantic_state: [f32; 8],

    /// Affect valence [-1.0, 1.0]: current emotional tone
    /// Derived from DOCAS, used for worldview updates
    pub affect_valence: f32,

    /// Affect arousal [0.0, 1.0]: activation level
    /// Per psychology layer requirements
    pub affect_arousal: f32,

    /// Social orientation [-1.0, 1.0]: pro-social vs antagonistic
    /// High O → positive `social_orientation`
    pub social_orientation: f32,

    // === Philosophy Layer (Stage 2) ===
    /// Philosophical state with `τ_φ` axis (Paper 6)
    /// Replaces: `worldview_coherence`, `norm_adherence`, `trust_posture`, machiavellianism
    pub philosophical_state: Option<crate::philosophy::PhilosophicalState>,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // === Meaning Layer (Paper 11) ===
    /// Meaning stability and percolation state
    pub meaning_state: Option<crate::meaning::MeaningState>,

    // === GTT Fields (Paper 1) ===
    /// Lambda (λ) learning rate from genome
    pub lambda: f32,
    /// Trust expectation E\[τ\]
    pub expectation: f32,
    /// Trust validation score from interactions
    pub validation: f32,
    /// Prediction error |`τ_predicted` - `τ_actual`|
    pub prediction_error: f32,
    /// Trust stability (rolling variance)
    pub stability: f32,

    // === TFP Fields (Paper 3) ===
    /// Cross-domain alignment: biological
    pub psi_bio: f32,
    /// Cross-domain alignment: financial
    pub psi_fin: f32,
    /// Cross-domain alignment: linguistic
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub psi_ling: f32,
    /// Composite `Ψ_trust`
    pub psi_trust: f32,

    // === Emotion Fields (Paper 4) ===
    /// VAD dominance dimension
    pub dominance: f32,

    // === Linguistics Fields (Paper 5) ===
    /// Trust in meaning (τ^s)
    pub trust_in_meaning: f32,
    /// Semantic entropy
    pub semantic_entropy: f32,
    /// Semantic coherence
    pub semantic_coherence: f32,

    // === Culture Fields (Paper 7) ===
    /// Cultural quadrant (from b × `τ_φ`)
    pub cultural_quadrant: Option<CulturalQuadrant>,
    /// Hofstede dimension scores
    pub hofstede_dimensions: Option<HofstedeDimensions>,
    /// CVSCALE-based individual cultural orientation
    /// Replaces ecological fallacy of direct nation-to-individual assignment
    pub cultural_orientation: Option<crate::culture::IndividualCulturalOrientation>,

    // === Game Theory Fields (Paper 9) ===
    /// Full game interaction history
    pub game_history: Vec<GameOutcome>,
    /// Partner-specific trust scores
    pub trust_in_partner: std::collections::HashMap<AgentId, f32>,
    /// Cumulative game payoff
    pub cumulative_payoff: f32,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // === PDE4B Fields (Paper 10) ===
    /// PDE4B expression level from genome loci 24-25
    /// Per §6.6.5: Modulates cAMP → learning rate λ
    /// Range: [0.0, 1.0] where 0.0 = low PDE4B (fast learning), 1.0 = high PDE4B (slow learning)
    pub pde4b_expression: f32,
    /// cAMP level (inverse of PDE4B)
    /// ↑cAMP → fast prediction updating (high λ)
    /// ↓cAMP → sticky priors, slow updating (low λ)
    pub camp_level: f32,
    /// Learning rate modifier λ computed from PDE4B
    /// Formula: λ = 0.61 × (1.5 - 0.5 × `pde4b_expression`)
    /// Range: [0.305, 0.915] per Montague et al. 1996
    pub learning_rate_modifier: f32,

    // === Neural Maturation Fields (Research: MATURITY_NEURAL_DEVELOPMENT) ===
    /// Prefrontal cortex development state (dlPFC and vmPFC)
    /// Tracks neural maturation with genome-controlled rates
    pub prefrontal_cortex: PrefrontalCortex,

    // === Epigenetics Fields (DNA Methylation) ===
    /// Epigenetic state (DNA methylation patterns)
    /// Based on Meaney & Szyf (2005) - maternal care and stress reactivity
    pub epigenome: Epigenome,

    // === Big Five Personality (OCEAN) ===
    /// Big Five personality traits per Costa & `McCrae` (1992)
    /// Emerges from genome (h² ≈ 0.50) + early life environment
    /// Loci 20-29: Openness, Conscientiousness, Extraversion, Agreeableness, Neuroticism
    pub personality: PersonalityTraits,

    // === Working Memory (Miller's Law) ===
    /// Working memory system with capacity limits
    /// Miller (1956): 7 ± 2 chunks capacity
    pub working_memory: WorkingMemory,

    // === Inbreeding Depression Tracking ===
    /// Inbreeding tracker for pedigree-based F coefficient and fitness effects
    /// Per Charlesworth & Willis (2009): Tracks inbreeding depression from related matings
    pub inbreeding: crate::genetics::InbreedingTracker,

    // === Movement Direction (for Pac-Man visualization) ===
    /// Last movement delta X (for visual orientation)
    /// Updated each tick based on position change
    pub last_dx: f32,
    /// Last movement delta Y (for visual orientation)
    /// Updated each tick based on position change
    pub last_dy: f32,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // === Social Forces (Barnes-Hut) ===
    /// Accumulated social force from other agents
    /// Computed using Barnes-Hut approximation
    pub social_force: crate::types::Vector,

    // === Stigmergy Layer (SIM-STIG-001) ===
    /// Pheromone sensitivity for each pheromone type [Food, Danger, Repro, Territory, Recruit]
    pub pheromone_sensitivity: [f32; 5],

    /// Current chemotaxis direction (where agent is following pheromone gradient)
    pub chemotaxis_direction: Option<crate::types::Direction>,

    /// Last pheromone deposited (for trail continuity)
    pub last_pheromone_deposit_tick: u32,

    // === Time Preference Profile (Hyperbolic Discounting) ===
    /// Temporal preferences for intertemporal decisions
    /// Per economics.rs: β-δ model for present bias and long-term discounting
    pub time_preference: TimePreferenceProfile,
    // === Version 4.0: Bounded Attention & Visual Cloaking (Sim D16 / D17) ===
    /// Baseline cognitive bandwidth kappa_0 in [0.01, 1.0] (Beta(2, 5) distributed in population)
    pub cognitive_bandwidth: f32,
    // === Wet-Lab IoT Biomarkers (Apple Watch HRV/Sleep, Muse EEG, Stelo CGM) ===
    /// Multi-modal wet-lab and IoT wearable biomarkers (Apple Watch HRV/Sleep, Muse EEG, Stelo CGM)
    pub biomarkers: crate::biomarkers::AgentBiomarkers,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Genome-derived initialization state computed once at agent birth.
struct GenomeDerivedState {
    /// PDE4B expression level [0.0, 1.0] from genome loci 24-25.
    pde4b_level: f32,
    /// Normalized cAMP level [0.0, 1.0] (inverse: ↓PDE4B → ↑cAMP).
    camp: f32,
    /// Learning rate λ [0.305, 0.915].
    lambda: f32,
    /// Sugarscape vision range (1-11 cells).
    vision_range: u32,
    /// Sugarscape metabolic rate (1.0-2.0 energy/tick).
    metabolic_rate: f32,
    /// Prefrontal cortex with genome-controlled maturation parameters.
    prefrontal_cortex: PrefrontalCortex,
    /// Time preference profile (heritable, initialized from genome).
    time_preference: TimePreferenceProfile,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
impl Agent {
    /// Create a new agent with default state
    /// Per §7.5.1 lines 3884-3924
    /// Trust emerges from: genome → DOCAS sensitivity → DOCAS state → `compute_tau()`
    #[must_use]
    pub fn new(
        id: AgentId,
        position: Position,
        genome: Genome,
        birth_tick: Tick,
        sex: Sex,
    ) -> Self {
        // Compute genome-derived state (PDE4B pathway, λ, Sugarscape traits,
        // neural maturation, prefrontal cortex, time preference).
        let derived = Self::compute_genome_derived_state(&genome);

        let initial_docas = DocasState::new([
            (0.35 + genome.get_trait(5) * 0.30).clamp(0.1, 0.9),
            (0.35 + genome.get_trait(6) * 0.30).clamp(0.1, 0.9),
            (0.35 + genome.get_trait(7) * 0.30).clamp(0.1, 0.9),
            (0.35 + genome.get_trait(8) * 0.30).clamp(0.1, 0.9),
            (0.35 + genome.get_trait(9) * 0.30).clamp(0.1, 0.9),
        ]);
        let dmn_budget = 100 + ((genome.get_trait(10) * 150.0) as u32);
        let dmn_horizon = (2 + ((genome.get_trait(11) * 5.0) as u32)).clamp(2, 7);
        let criticality_sigma = (0.85 + genome.get_trait(12) * 0.30).clamp(0.85, 1.15);
        let emotion_baseline = (0.35 + genome.get_trait(13) * 0.30).clamp(0.35, 0.65);

        let mut genome_traits = [0.5f32; 14];
        for (t, item) in genome_traits.iter_mut().enumerate() {
            *item = genome.get_trait(t);
        }
        let initial_biomarkers =
            crate::biomarkers::AgentBiomarkers::new_individualized(&genome_traits, sex);

        Self {
            id,
            position,
            energy: 35.0, // TUNABLE: Initial energy per §7.4.0 line 3757 (FIX: Reduced from 75.0 for scarcity challenge)
            // TAG: surface=simulation owner=research-team rule=SIM-001
            age: 0,
            sex,
            genome,
            vision_range: derived.vision_range,
            metabolic_rate: derived.metabolic_rate,
            docas: initial_docas,
            dmn_budget,
            dmn_horizon,
            criticality_sigma,
            cognitive_system: crate::types::CognitiveSystem::default(),
            emotion_baseline,
            reputation_score: 0.0,
            local_reputation: std::collections::HashMap::new(), // SIM-004
            group_id: None,
            cultural_traits: Vec::new(),
            isolation_score: 0.0,
            attestations_given: Vec::new(),
            attestations_received: Vec::new(),
            lexicon: Lexicon::default(), // Stage 4: Empty until initialized
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            has_verified_id: false,
            verification_count: 0,
            strategy: StrategyType::default(),
            is_identity_theft_victim: false,
            identity_theft_event: None,
            identity: None, // Initialized in simulation step
            theft_recovery_tick: None,
            credit_report: CreditReport::clean(),
            credit_error_recovery_tick: None,
            risk_profile: RiskProfile::default(),
            engagement_network: Vec::new(),
            birth_tick,
            alive: true,
            // Interaction history for reputation-based DOCAS modulation
            positive_interactions: 0,
            negative_interactions: 0,
            // Game strategy interaction history
            interaction_history: std::collections::HashMap::new(),
            // Trust Vector Field Curvature (§6.6.2)
            docas_history: VecDeque::with_capacity(CURVATURE_WINDOW_SIZE),
            trust_curvature: TrustCurvature::new(),
            // Economic state (SIM-ECON-001)
            economics: EconomicState::default(),
            // Debt obligations (SIM-002)
            debt_obligations: Vec::new(),
            // Music state (Stage 4+, only when enabled)
            music_state: None,
            // Emotion state (DOCAS→Emotion reverse projection)
            emotion_state: crate::emotion::EmotionState::default(),
            // Language drift tracking (LING-C1: Drift ∝ 1/τ)
            language_drift: crate::language_drift::LanguageDriftTracker::new(id),
            // Non-Newtonian Viscosity (§6.6.3)
            trust_viscosity: TrustViscosity::default(),
            // TAG: surface=simulation owner=research-team rule=SIM-001
            tau_history: Vec::with_capacity(20),
            adaptation_rate: 1.0, // Start with maximum adaptation
            // Psychology Layer (Stage 2)
            semantic_state: [0.5; 8], // Neutral initial semantic state
            affect_valence: 0.0,      // Neutral valence
            affect_arousal: 0.5,      // Moderate arousal
            social_orientation: 0.0,  // Neutral social orientation
            // Philosophy Layer (Stage 2) - initialized on first update
            philosophical_state: None,
            // Meaning Layer (Paper 11) - initialized on first update
            meaning_state: None,
            // GTT Fields (Paper 1)
            lambda: derived.lambda, // From genome.compute_lambda() [0.305, 0.915]
            expectation: 0.5,       // Neutral expectation
            validation: 0.5,        // Neutral validation
            prediction_error: 0.0,  // No error initially
            stability: 1.0,         // Full stability initially
            // TFP Fields (Paper 3)
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            psi_bio: 0.5,
            psi_fin: 0.5,
            psi_ling: 0.5,
            psi_trust: 0.5,
            // Emotion Fields (Paper 4)
            dominance: 0.5, // Neutral dominance
            // Linguistics Fields (Paper 5)
            trust_in_meaning: 0.5,
            semantic_entropy: 0.5,
            semantic_coherence: 0.5,
            // Culture Fields (Paper 7) - initialized on first update
            cultural_quadrant: None,
            hofstede_dimensions: None,
            cultural_orientation: None,
            // Game Theory Fields (Paper 9)
            game_history: Vec::new(),
            trust_in_partner: std::collections::HashMap::new(),
            cumulative_payoff: 0.0,
            // PDE4B Fields (Paper 10) - computed from genome at initialization
            // Per §6.6.5: PDE4B at loci 24-25 modulates cAMP → learning rate λ
            pde4b_expression: derived.pde4b_level, // [0.0, 1.0] from genome
            camp_level: derived.camp,              // Inverse: ↓PDE4B → ↑cAMP
            learning_rate_modifier: derived.lambda, // [0.305, 0.915] per Montague 1996
            // Neural Maturation Fields - initialized with genome-controlled parameters
            prefrontal_cortex: derived.prefrontal_cortex, // dlPFC/vmPFC with genome-controlled maturation
            // Epigenetics Fields
            epigenome: Epigenome::new(), // Empty epigenome, will be programmed or inherited

            // Big Five Personality - initialized to default, set during birth
            personality: PersonalityTraits::default(),

            // Working Memory - initialized with dlPFC development
            // Per Miller (1956): Capacity increases with neural maturation
            working_memory: WorkingMemory::new(0.0, 0), // Will be updated by update_neural_maturation

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Inbreeding - initialized to outbred (F = 0)
            // Will be updated when parents are known during reproduction
            inbreeding: crate::genetics::InbreedingTracker::new(),
            // Default facing direction (right, 0 degrees)
            last_dx: 1.0,
            last_dy: 0.0,
            // Social forces (Barnes-Hut)
            social_force: crate::types::Vector::zero(),
            // Stigmergy layer - initialized with varied sensitivity
            // Per SIM-STIG-001: Individual sensitivity profiles
            pheromone_sensitivity: [1.0, 1.0, 1.0, 0.5, 0.8], // Different sensitivities
            chemotaxis_direction: None,
            last_pheromone_deposit_tick: 0,
            // Time preference - initialized from genome (heritable)
            // Per economics.rs: β-δ model for intertemporal choice
            time_preference: derived.time_preference,
            // Bounded cognitive bandwidth kappa_0 in [0.01, 1.0] (Beta(2, 5) distributed in population)
            cognitive_bandwidth: {
                let u = (((id.wrapping_mul(2654435761) ^ birth_tick.wrapping_mul(104729)) % 10000)
                    as f32
                    + 0.5)
                    / 10000.0;
                (u.powf(0.5) * (1.0 - (1.0 - u).powf(0.2))).clamp(0.02, 0.98)
            },
            biomarkers: initial_biomarkers,
        }
    }

    /// Compute the genome-derived initialization state (PDE4B pathway, λ,
    /// Sugarscape traits, neural maturation, prefrontal cortex, time
    /// preference) before the genome is moved into the agent.
    fn compute_genome_derived_state(genome: &Genome) -> GenomeDerivedState {
        // Compute PDE4B-derived values from genome before moving genome into struct
        // Per §6.6.5: PDE4B at loci 24-25 modulates cAMP → learning rate λ
        let pde4b_level = genome.pde4b_level(); // [0.0, 1.0]

        // Compute cAMP using research-based PDE4B pathway
        let pathway_params = PDE4BPathwayParams::default();
        let pathway_output = compute_pathway(pde4b_level, &pathway_params);
        let camp = pathway_output.camp_normalized(); // Normalized [0.0, 1.0]

        let lambda = genome.compute_lambda(); // [0.305, 0.915]

        // Compute Sugarscape traits (P2-1, P2-2)
        let vision_range = genome.vision_range(); // 1-11 cells
        let metabolic_rate = genome.metabolic_rate(); // 1.0-2.0 energy/tick

        // Compute neural maturation parameters from genome (Research: MATURITY_NEURAL_DEVELOPMENT)
        // Genome traits control maturation speed - faster/slower than typical
        let maturation_rate = genome.pfc_maturation_rate(); // [0.5, 1.5]
        let _pruning_efficiency = genome.pruning_efficiency(); // [0.3, 1.0] - reserved for future use
        let myelination_rate = genome.myelination_rate(); // [0.5, 1.5]
        let _stress_sensitivity = genome.stress_sensitivity(); // [0.2, 1.0] - used in compute_maturation_rate

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Base maturation ages (in ticks, assuming ~10 ticks/year)
        // These are adjusted by genome-controlled maturation rates
        let base_vmpfc_mature = 150; // ~15 years (human)
        let base_dlpfc_mature = 300; // ~30 years (human)
        let base_pruning_complete = 70; // ~7 years (human)
        let base_myelination_complete = 400; // ~40 years (human)

        // Adjust maturation ages by genome rate (faster rate = earlier completion)
        let vmpfc_mature = ((base_vmpfc_mature as f32 / maturation_rate) as u64).max(50);
        let dlpfc_mature = ((base_dlpfc_mature as f32 / maturation_rate) as u64).max(100);
        let pruning_complete = ((base_pruning_complete as f32 / maturation_rate) as u64).max(30);
        let myelination_complete =
            ((base_myelination_complete as f32 / myelination_rate) as u64).max(200);

        // Initialize prefrontal cortex with genome-controlled parameters
        let prefrontal_cortex = PrefrontalCortex {
            dlpfc: DLPFCState {
                development: 0.0,
                working_memory_capacity: 1,
                cognitive_control_strength: 0.1,
                age_mature: dlpfc_mature,
                executive_efficiency: 0.1,
            },
            vmpfc: VMPFCState {
                development: 0.0,
                social_evaluation_accuracy: 0.2,
                risk_assessment_calibration: 0.1,
                age_mature: vmpfc_mature,
                value_integration_strength: 0.1,
            },
            pruning_progress: 0.0,
            myelination_progress: 0.0,
            pruning_completion_age: pruning_complete,
            myelination_completion_age: myelination_complete,
            current_maturation_rate: maturation_rate,
            chronic_cortisol: 0.0,
        };

        // Time preference - initialized from genome (heritable)
        // Per economics.rs: β-δ model for intertemporal choice
        let time_preference = TimePreferenceProfile::from_genome(genome);

        GenomeDerivedState {
            pde4b_level,
            camp,
            lambda,
            vision_range,
            metabolic_rate,
            prefrontal_cortex,
            time_preference,
        }
    }

    /// Create a new agent with initialized lexicon (for Stage 4+)
    /// Per §7.5.1 line 4014: lexicon: `HashMap`<`ConceptId`, Embedding>
    #[must_use]
    pub fn new_with_lexicon(
        id: AgentId,
        position: Position,
        genome: Genome,
        birth_tick: Tick,
        sex: Sex,
        lexicon: Lexicon,
    ) -> Self {
        let mut agent = Self::new(id, position, genome, birth_tick, sex);
        agent.lexicon = lexicon;
        agent
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if agent should die (energy depleted or max age)
    /// Compute effective cognitive bandwidth under thermal/photic weather stress and optogenetic current
    #[must_use]
    pub fn effective_bandwidth(&self, weather_mod: f32, opto_stress_mod: f32) -> f32 {
        let cortisol_suppression = 1.0 - (self.docas.cortisol() * 0.2).min(0.5);
        (self.cognitive_bandwidth * weather_mod * opto_stress_mod * cortisol_suppression)
            .clamp(0.005, 1.0)
    }

    /// Per §6.21 mortality dynamics
    #[must_use]
    pub fn should_die(&self) -> bool {
        self.energy <= 0.0
    }

    /// Check if agent is alive and not in a group
    /// Used for group formation decisions
    #[must_use]
    pub const fn is_alive_and_ungrouped(&self) -> bool {
        self.alive && self.group_id.is_none() && self.energy > 0.0
    }

    /// Consume energy (called each tick)
    /// Per §7.4.0 line 3755: `consumption_rate` 1.0-2.0 energy/tick
    pub fn consume_energy(&mut self, amount: Energy) {
        self.energy = (self.energy - amount).max(0.0);
    }

    /// Consume energy based on metabolic rate (Sugarscape P2-2)
    /// Called automatically each tick with agent's `metabolic_rate`
    pub fn metabolize(&mut self) {
        self.energy = (self.energy - self.metabolic_rate).max(0.0);
    }

    /// Gain energy from resource collection
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub fn gain_energy(&mut self, amount: Energy) {
        self.energy += amount;
        // Cap at maximum energy (TUNABLE)
        self.energy = self.energy.min(100.0);
    }

    /// CANONICAL trust computation
    ///
    /// Uses DOCAS state with viscosity adjustment.
    /// Formula: τ = (D + O + S) / (1 + C + A) with viscosity smoothing
    ///
    /// Per §6.1: τ = f(D, O, C, A, S) emerges from neuromodulator balance
    #[must_use]
    pub fn compute_trust(&self) -> f32 {
        let base_tau = self.compute_base_tau();
        self.apply_viscosity(base_tau)
    }

    /// Base tau without viscosity
    ///
    /// Formula: τ = (D + O + S) / (1 + C + A)
    /// Returns value in [0.0, 2.0] range
    fn compute_base_tau(&self) -> f32 {
        use crate::types::{A, C, D, O, S};

        let d = self.docas.levels[D];
        let o = self.docas.levels[O];
        let s = self.docas.levels[S];
        let c = self.docas.levels[C];
        let a = self.docas.levels[A];

        let numerator = d + o + s;
        let denominator = 1.0 + c + a;

        (numerator / denominator).clamp(0.0, 2.0)
    }

    /// Apply viscosity smoothing to base tau
    ///
    /// Uses the trust viscosity parameters to smooth trust changes based on
    /// environmental volatility and baseline resilience.
    fn apply_viscosity(&self, base_tau: f32) -> f32 {
        // Compute current volatility from tau history
        let volatility = TrustViscosity::compute_volatility_from_changes(&self.tau_history);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Compute viscosity-adjusted trust
        let viscous_factor = self.trust_viscosity.compute_viscosity(volatility, base_tau);

        // Blend base tau with viscous adaptation (30% viscosity influence)
        let viscosity_weight = 0.3;
        let blended = base_tau * (1.0 - viscosity_weight) + viscous_factor * viscosity_weight;

        // Apply stability factor from tau history variance
        let stability_factor = if self.tau_history.len() >= 2 {
            let mean = self.tau_history.iter().sum::<f32>() / self.tau_history.len() as f32;
            let variance = self
                .tau_history
                .iter()
                .map(|t| (t - mean).powi(2))
                .sum::<f32>()
                / self.tau_history.len() as f32;
            1.0 - (variance * 2.0).min(1.0) // High variance = low stability
        } else {
            1.0
        };

        blended * (0.5 + 0.5 * stability_factor)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update trust history with new tau value
    ///
    /// NOTE: This method now uses Rescorla-Wagner learning for expectation updates.
    /// The simple EMA is replaced by prediction-error based learning.
    pub fn update_tau_history(&mut self, tau: f32) {
        self.tau_history.push(tau);
        if self.tau_history.len() > 20 {
            self.tau_history.remove(0);
        }

        // Update prediction error if we had an expectation
        if self.expectation > 0.0 {
            self.prediction_error = (tau - self.expectation).abs();
        }

        // Update expectation using Rescorla-Wagner learning
        // E[t+1] = E[t] + α(τ[t] - E[t])
        let learning_rate = self.get_learning_rate();
        let prediction_error = tau - self.expectation;
        self.expectation += learning_rate * prediction_error;
        self.expectation = self.expectation.clamp(0.0, 2.0);
    }

    /// Update trust using Rescorla-Wagner learning rule
    /// dτ/dt = α(R - τ)
    ///
    /// # Arguments
    /// * `reward_signal` - R: observed trustworthiness (0-2 scale)
    ///
    /// Per §6.6.5: PDE4B modulates cAMP → learning rate λ
    pub fn update_trust_rescorla_wagner(&mut self, reward_signal: f32) {
        let current_tau = self.compute_trust();
        let learning_rate = self.get_learning_rate();

        // Rescorla-Wagner: τ_new = τ_old + α(R - τ_old)
        let prediction_error = reward_signal - current_tau;
        let delta_tau = learning_rate * prediction_error;
        let new_tau = (current_tau + delta_tau).clamp(0.0, 2.0);

        // Map new_tau back to DOCAS components
        // Adjust Oxytocin and Cortisol to achieve target tau
        self.adjust_docas_for_target_tau(new_tau);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Store history for analysis
        self.tau_history.push(new_tau);
        if self.tau_history.len() > 20 {
            self.tau_history.remove(0);
        }

        // Update expectation to match actual tau after adjustment
        self.expectation = new_tau;

        // Store prediction error for curiosity/reward
        self.prediction_error = prediction_error.abs();
    }

    /// Update trust using the canonical [0, 1] Master Law (MasterLaw.lean):
    ///   S_t = clamp(tau_t / 2, 0, 1),  eps_tilde = (V_t - E_t) / s_d
    ///   S_{t+1} = clamp(S_t + [lambda^+ max(eps,0) + lambda^- min(eps,0)] r_t - delta (S_t - S_base), 0, 1)
    ///   tau_{t+1} = 2 * S_{t+1} in [0, 2]
    pub fn update_trust_master_law(
        &mut self,
        validation: f32,
        config: &crate::master_law::MasterLawConfig,
    ) {
        let current_tau = self.compute_trust() as f64;
        let current_s = (current_tau * 0.5).clamp(0.0, 1.0);
        let expectation_s = (current_tau * 0.5).clamp(0.0, 1.0);
        let validation_s = (validation as f64).clamp(0.0, 1.0);
        let eps = config.compute_normalized_mismatch(validation_s, expectation_s);
        let new_s = config.step(current_s, eps).clamp(0.0, 1.0);
        let new_tau = (new_s * 2.0) as f32;

        self.adjust_docas_for_target_tau(new_tau);
        self.tau_history.push(new_tau);
        if self.tau_history.len() > 20 {
            self.tau_history.remove(0);
        }
        self.expectation = new_tau;
        self.prediction_error = (validation_s as f32 - expectation_s as f32).abs();
    }

    /// Get learning rate from PDE4B genome trait
    ///
    /// Per §6.6.5: PDE4B affects cAMP → learning rate
    /// Empirical: h²(λ) ≈ 0.45
    fn get_learning_rate(&self) -> f32 {
        // Base rate of 0.1, modified by PDE4B expression
        // Low PDE4B (high cAMP) → faster learning
        // High PDE4B (low cAMP) → slower learning
        let base_rate = 0.1;
        let pde4b_modifier = (0.5 - self.pde4b_expression) * 0.2; // Inverted: low PDE4B = faster
        (base_rate + pde4b_modifier).clamp(0.01, 0.5)
    }

    /// Adjust DOCAS components to achieve target tau
    ///
    /// Uses a stronger adjustment factor to ensure the update has
    /// meaningful impact on trust computation.
    ///
    /// # Arguments
    /// * `target_tau` - Desired trust level
    fn adjust_docas_for_target_tau(&mut self, target_tau: f32) {
        use crate::types::{A, C, D, O, S};

        let current_tau = self.compute_trust();
        let tau_error = target_tau - current_tau;
        let adjustment_strength = 0.25;

        // Distribute Master Law trust update across all 5 carriers proportional to genome sensitivity:
        // Increasing trust = (+D, +O, +S), decreasing stress = (-C, -A)
        let beta = self.genome.docas_beta();
        let d_adj = tau_error * adjustment_strength * beta[D] * 2.0;
        let o_adj = tau_error * adjustment_strength * beta[O] * 2.0;
        let s_adj = tau_error * adjustment_strength * beta[S] * 2.0;
        let c_adj = -tau_error * adjustment_strength * beta[C] * 1.5;
        let a_adj = -tau_error * adjustment_strength * beta[A] * 1.5;

        self.docas.levels[D] = (self.docas.levels[D] + d_adj).clamp(0.05, 0.98);
        self.docas.levels[O] = (self.docas.levels[O] + o_adj).clamp(0.05, 0.98);
        self.docas.levels[S] = (self.docas.levels[S] + s_adj).clamp(0.05, 0.98);
        self.docas.levels[C] = (self.docas.levels[C] + c_adj).clamp(0.05, 0.98);
        self.docas.levels[A] = (self.docas.levels[A] + a_adj).clamp(0.05, 0.98);
    }

    /// Initialize philosophy state from genome and DOCAS
    /// Called once when agent is first created/initialized
    pub fn initialize_philosophy(&mut self, _tick: Tick) {
        if self.philosophical_state.is_none() {
            let seed_value = (self.id as f32 % 100.0) / 100.0;
            self.philosophical_state = Some(
                crate::philosophy::PhilosophicalState::from_genome_and_docas(
                    &self.docas,
                    seed_value,
                ),
            );
        }
    }

    /// Update philosophy layer state based on psychology and DOCAS
    /// Should be called each tick
    pub fn update_philosophy(&mut self, tick: Tick) {
        // Initialize if needed
        self.initialize_philosophy(tick);

        // Update the philosophical state
        // TAG: surface=simulation owner=research-team rule=SIM-001
        if let Some(ref mut phil) = self.philosophical_state {
            phil.update(&self.docas, tick);
        }
    }

    /// Compute semantic similarity with another agent
    /// Used for social bonding and mate selection
    #[must_use]
    pub fn semantic_similarity(&self, other: &Self) -> f32 {
        let mut dot_product = 0.0;
        let mut norm_self = 0.0;
        let mut norm_other = 0.0;

        for i in 0..8 {
            dot_product += self.semantic_state[i] * other.semantic_state[i];
            norm_self += self.semantic_state[i] * self.semantic_state[i];
            norm_other += other.semantic_state[i] * other.semantic_state[i];
        }

        let denominator = (norm_self.sqrt() * norm_other.sqrt()).max(1e-6);
        let cos_sim = (dot_product / denominator).clamp(-1.0, 1.0);
        let align = 1.0 - (self.social_orientation - other.social_orientation).abs() * 0.1;
        (cos_sim * align).clamp(-1.0, 1.0)
    }

    /// Update cultural quadrant based on bond density and `τ_φ`
    /// Called when bond density or philosophy changes
    pub fn update_cultural_quadrant(&mut self) {
        let bond_density = self.economics.bond_density;
        let tau_phi = self.philosophical_state.as_ref().map_or(0.0, |p| p.tau_phi);

        self.cultural_quadrant = Some(match (bond_density > 0.5, tau_phi > 0.0) {
            (true, true) => CulturalQuadrant::HighBondHighTrust,
            (true, false) => CulturalQuadrant::HighBondLowTrust,
            (false, true) => CulturalQuadrant::LowBondHighTrust,
            (false, false) => CulturalQuadrant::LowBondLowTrust,
        });
    }

    /// Add a game outcome to history
    pub fn record_game_outcome(&mut self, outcome: GameOutcome) {
        self.game_history.push(outcome);
        self.cumulative_payoff += outcome.my_payoff;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Update partner trust
        let trust_delta = if outcome.partner_action {
            0.1 // Partner cooperated → increase trust
        } else {
            -0.1 // Partner defected → decrease trust
        };

        let current_trust = self
            .trust_in_partner
            .get(&outcome.partner_id)
            .copied()
            .unwrap_or(0.5);
        let new_trust = (current_trust + trust_delta).clamp(0.0, 1.0);
        self.trust_in_partner.insert(outcome.partner_id, new_trust);
        self.update_local_reputation(outcome.partner_id, trust_delta);
    }

    /// Initialize meaning state
    pub fn initialize_meaning(&mut self) {
        if self.meaning_state.is_none() {
            self.meaning_state = Some(crate::meaning::MeaningState::new());
        }
    }

    /// Update meaning state based on trust
    pub fn update_meaning(&mut self, neighbor_stabilities: &[f32], tick: Tick) {
        self.initialize_meaning();

        let trust = self.compute_trust();
        let validation_weight = self.validation;
        if let Some(ref mut meaning) = self.meaning_state {
            meaning.update(
                trust * (0.8 + 0.2 * validation_weight),
                neighbor_stabilities,
                tick,
            );
        }
    }

    /// Get local reputation for a specific agent
    #[must_use]
    pub fn get_local_reputation(&self, other_id: AgentId) -> f32 {
        self.local_reputation.get(&other_id).copied().unwrap_or(0.5)
    }

    /// Check if agent knows reputation of another agent
    #[must_use]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub fn knows_reputation_of(&self, other_id: AgentId) -> bool {
        self.local_reputation.contains_key(&other_id)
    }

    /// Update local reputation for a specific agent
    pub fn update_local_reputation(&mut self, other_id: AgentId, delta: f32) {
        let current = self.local_reputation.get(&other_id).copied().unwrap_or(0.5);
        let new_value = (current + delta).clamp(0.0, 1.0);
        self.local_reputation.insert(other_id, new_value);
        if delta > 0.0 {
            self.validation = (self.validation * 0.98 + 0.02).clamp(0.0, 1.0);
        }
    }

    /// Record a positive interaction
    pub fn record_positive_interaction(&mut self) {
        self.positive_interactions += 1;
        self.validation = (self.validation * 0.95 + 0.05).clamp(0.0, 1.0);
    }

    /// Record a negative interaction
    pub fn record_negative_interaction(&mut self) {
        self.negative_interactions += 1;
        self.prediction_error = (self.prediction_error * 0.95 + 0.05).clamp(0.0, 1.0);
    }

    /// Compute decision confidence based on cognitive state
    #[must_use]
    pub fn decision_confidence(&self) -> f32 {
        // Confidence based on:
        // 1. DOCAS stability (serotonin)
        // 2. Recent prediction error (lower error = higher confidence)
        // 3. DMN budget availability
        // 4. Cognitive bandwidth

        let serotonin_factor = self.docas.serotonin();
        let error_factor = 1.0 - self.prediction_error.min(1.0);
        let budget_factor = (self.dmn_budget as f32 / 100.0).min(1.0);
        let bandwidth_factor = self.cognitive_bandwidth.clamp(0.0, 1.0);

        (serotonin_factor * 0.35
            + error_factor * 0.35
            + budget_factor * 0.15
            + bandwidth_factor * 0.15)
            .clamp(0.0, 1.0)
    }

    /// Add a debt obligation
    pub fn add_debt(
        &mut self,
        creditor_id: AgentId,
        amount: f32,
        tick_created: Tick,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        tick_due: Option<Tick>,
    ) {
        let debt = crate::types::DebtRecord::new(creditor_id, amount, tick_created, tick_due);
        self.debt_obligations.push(debt);
        self.economics.debt_burden += amount;
    }

    /// Age the agent by one tick
    pub fn age_tick(&mut self) {
        self.age += 1;
    }

    /// Update neural maturation progress
    /// Per Research: `MATURITY_NEURAL_DEVELOPMENT_RESEARCH.md`
    /// Updates dlPFC, vmPFC, synaptic pruning, and myelination based on age and genome
    pub fn update_neural_maturation(&mut self) {
        let pfc = &mut self.prefrontal_cortex;
        let current_age = self.age as f32;

        let current_cortisol = self.docas.cortisol();
        pfc.chronic_cortisol = 0.95 * pfc.chronic_cortisol + 0.05 * current_cortisol;

        let energy_ratio = self.energy / 100.0;
        pfc.current_maturation_rate = self
            .genome
            .compute_maturation_rate(pfc.chronic_cortisol, energy_ratio);

        pfc.vmpfc.development = Self::sigmoid_development(current_age, pfc.vmpfc.age_mature as f32);
        pfc.vmpfc.social_evaluation_accuracy = 0.2 + 0.8 * pfc.vmpfc.development;
        pfc.vmpfc.risk_assessment_calibration = 0.1 + 0.9 * pfc.vmpfc.development;
        pfc.vmpfc.value_integration_strength = pfc.vmpfc.development;

        pfc.dlpfc.development = Self::sigmoid_development(current_age, pfc.dlpfc.age_mature as f32);
        pfc.dlpfc.working_memory_capacity = (1.0 + 6.0 * pfc.dlpfc.development) as u32;
        pfc.dlpfc.cognitive_control_strength = pfc.dlpfc.development;
        pfc.dlpfc.executive_efficiency = pfc.dlpfc.development.powi(2);

        pfc.pruning_progress =
            (current_age / (pfc.pruning_completion_age as f32).max(1.0)).min(1.0);
        pfc.myelination_progress =
            (current_age / (pfc.myelination_completion_age as f32).max(1.0)).min(1.0);

        self.update_working_memory_capacity();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn sigmoid_development(age: f32, mature_age: f32) -> f32 {
        if age >= mature_age {
            1.0
        } else {
            let progress = age / mature_age.max(1.0);
            1.0 / (1.0 + (-10.0 * (progress - 0.5)).exp())
        }
    }

    /// Check if agent has reached neural maturity for reproduction
    /// Per Research: `MATURITY_NEURAL_DEVELOPMENT_RESEARCH.md`
    /// Requires: vmPFC sufficiently developed + dlPFC has basic function
    #[must_use]
    pub fn is_neurally_mature(&self) -> bool {
        let pfc = &self.prefrontal_cortex;
        // vmPFC threshold: 70% developed (can make value-based decisions)
        // dlPFC threshold: 50% developed (has basic executive function)
        pfc.vmpfc.development >= 0.7 && pfc.dlpfc.development >= 0.5
    }

    /// Update psychology layer (placeholder for future expansion)
    pub fn update_psychology(&mut self) {
        // Update valence based on DOCAS
        let d = self.docas.dopamine();
        let c = self.docas.cortisol();
        self.affect_valence = (d - c).clamp(-1.0, 1.0);

        // Update arousal based on adrenaline
        self.affect_arousal = self.docas.adrenaline();

        // Update social orientation based on oxytocin and personality
        let oxytocin_factor = self.docas.oxytocin() * 2.0 - 1.0;
        let personality_factor = self.personality.agreeableness * 2.0 - 1.0;
        self.social_orientation =
            (oxytocin_factor * 0.6 + personality_factor * 0.4).clamp(-1.0, 1.0);
    }
}

pub mod behavior;
pub mod cognition;

#[cfg(test)]
mod tests;
