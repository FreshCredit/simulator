//! DOCAS: Neuromodulator-inspired state vector
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION neuromodulators, not medical advice
//!
//! Per §6.1 lines 617-660: DOCAS = [Dopamine, Oxytocin, Cortisol, Adrenaline, Serotonin]
//! Per §8.2.1 lines 4449-4471: `DOCASToggle` contract
//! Per §6.6.4: Multi-Agent Coupling Field (Φij)

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::genome::Genome;
use crate::types::{AgentId, Tick, DOCAS_SIZE};

/// Default window size for trust curvature calculation  // SIM-TRUST-001: Uses `compute_tau()`
pub const CURVATURE_WINDOW_SIZE: usize = 10;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// DOCAS state vector: [D, O, C, A, S]
/// Per §6.1 lines 617-660
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DocasState {
    /// [D, O, C, A, S] neuromodulator levels, each in [0, 1]
    /// D=Dopamine, O=Oxytocin, C=Cortisol, A=Adrenaline, S=Serotonin
    pub levels: [f32; DOCAS_SIZE],
}

impl Default for DocasState {
    /// Neutral state per §8.2.1 line 4469: [0.5, 0.5, 0.5, 0.5, 0.5]
    fn default() -> Self {
        Self {
            levels: [0.5; DOCAS_SIZE],
        }
    }
}

impl DocasState {
    /// Create new DOCAS state with specified levels
    #[must_use]
    pub const fn new(levels: [f32; DOCAS_SIZE]) -> Self {
        Self { levels }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create neutral state (all 0.5)
    #[must_use]
    pub fn neutral() -> Self {
        Self::default()
    }

    /// Get individual component by index
    /// P1-FIX: Bounds checking with `get()` instead of direct indexing
    #[must_use]
    pub fn get(&self, index: usize) -> f32 {
        debug_assert!(
            index < DOCAS_SIZE,
            "DOCAS index out of bounds: {index} >= {DOCAS_SIZE}"
        );
        *self.levels.get(index).unwrap_or(&0.5)
    }

    /// Dopamine level (reward/expectation)
    #[must_use]
    pub const fn dopamine(&self) -> f32 {
        self.levels[crate::types::D]
    }

    /// Oxytocin level (social salience)
    #[must_use]
    pub const fn oxytocin(&self) -> f32 {
        self.levels[crate::types::O]
    }

    /// Cortisol level (stress/threat)
    #[must_use]
    pub const fn cortisol(&self) -> f32 {
        self.levels[crate::types::C]
    }

    /// Adrenaline level (action urgency)
    #[must_use]
    pub const fn adrenaline(&self) -> f32 {
        self.levels[crate::types::A]
    }

    /// Serotonin level (stabilization)
    #[must_use]
    pub const fn serotonin(&self) -> f32 {
        self.levels[crate::types::S]
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Convert to array for serialization
    #[must_use]
    pub const fn to_array(&self) -> [f32; DOCAS_SIZE] {
        self.levels
    }

    /// Update DOCAS state with genome-modulated dynamics
    /// Per §6.1 lines 630-631: `DOCAS_t+1` = alpha * `DOCAS_t` + beta * `stimulus_response(event)`
    /// Per §8.2.1 lines 4452-4464: `docas_on` implementation
    pub fn update(
        &mut self,
        genome: &Genome,
        reward_error: f32,
        social_validation: f32,
        threat_signal: f32,
        action_urgency: f32,
        stability_signal: f32,
    ) {
        let alpha = genome.docas_alpha();
        let beta = genome.docas_beta();
        self.step_docas_dynamics(
            genome,
            alpha,
            beta,
            reward_error,
            social_validation,
            threat_signal,
            action_urgency,
            stability_signal,
        );
    }

    /// Update DOCAS state using continuous DialConfig parameters
    pub fn update_with_dials(
        &mut self,
        genome: &Genome,
        dials: &crate::toggles::DialConfig,
        reward_error: f32,
        social_validation: f32,
        threat_signal: f32,
        action_urgency: f32,
        stability_signal: f32,
    ) {
        let (a_min, a_max) = dials.docas_alpha_range;
        let a_spread = (a_max - a_min).max(0.001);
        let (b_min, b_max) = dials.docas_beta_range;
        let b_spread = (b_max - b_min).max(0.001);

        let alpha = [
            (a_min + genome.get_trait(0) * a_spread).clamp(0.01, 0.999),
            (a_min + genome.get_trait(1) * a_spread).clamp(0.01, 0.999),
            (a_min + genome.get_trait(2) * a_spread).clamp(0.01, 0.999),
            (a_min + genome.get_trait(3) * a_spread).clamp(0.01, 0.999),
            (a_min + genome.get_trait(4) * a_spread).clamp(0.01, 0.999),
        ];
        let beta = [
            (b_min + genome.get_trait(5) * b_spread).clamp(0.01, 1.0),
            (b_min + genome.get_trait(6) * b_spread).clamp(0.01, 1.0),
            (b_min + genome.get_trait(7) * b_spread).clamp(0.01, 1.0),
            (b_min + genome.get_trait(8) * b_spread).clamp(0.01, 1.0),
            (b_min + genome.get_trait(9) * b_spread).clamp(0.01, 1.0),
        ];

        self.step_docas_dynamics(
            genome,
            alpha,
            beta,
            reward_error,
            social_validation,
            threat_signal,
            action_urgency,
            stability_signal,
        );
    }

    fn step_docas_dynamics(
        &mut self,
        genome: &Genome,
        alpha: [f32; 5],
        beta: [f32; 5],
        reward_error: f32,
        social_validation: f32,
        threat_signal: f32,
        action_urgency: f32,
        stability_signal: f32,
    ) {
        let base = [
            0.35 + genome.get_trait(5) * 0.30, // D baseline in [0.35, 0.65]
            0.35 + genome.get_trait(6) * 0.30, // O baseline in [0.35, 0.65]
            0.35 + genome.get_trait(7) * 0.30, // C baseline in [0.35, 0.65]
            0.35 + genome.get_trait(8) * 0.30, // A baseline in [0.35, 0.65]
            0.35 + genome.get_trait(9) * 0.30, // S baseline in [0.35, 0.65]
        ];

        let stimuli = [
            reward_error,
            social_validation,
            threat_signal,
            action_urgency,
            stability_signal,
        ];

        for i in 0..DOCAS_SIZE {
            let rate = (1.0 - alpha[i]).clamp(0.05, 0.25);
            let gain = beta[i] * 2.5;
            let target = (base[i] + gain * stimuli[i].clamp(-1.0, 1.0)).clamp(0.05, 0.95);
            self.levels[i] = (self.levels[i] + rate * (target - self.levels[i])).clamp(0.05, 0.98);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Reset to neutral state (toggle OFF behavior)
    /// Per §8.2.1 lines 4467-4470
    pub fn reset_neutral(&mut self) {
        self.levels = [0.5; DOCAS_SIZE];
    }

    /// Compute τ (trust) from DOCAS per GTT framework
    /// τ = (D + O + S) / (1 + C + A) per §6.1
    ///
    /// Returns value in [0.0, 2.0] range:
    /// - τ = 0.0: Minimum trust (all positive factors at 0)
    /// - τ = 1.0: Neutral trust (balanced DOCAS)
    /// - τ = 2.0: Maximum trust (D=O=S=1, C=A=0)
    ///
    /// **NOTE:** This is a simple computation without agent context.
    /// For the full canonical trust computation with viscosity adjustment,
    /// use `agent.compute_trust()` instead.
    ///
    /// P0-FIX: Explicitly clamped to prevent unbounded values
    #[must_use]
    pub fn compute_tau(&self) -> f32 {
        let numerator = self.levels[crate::types::D]
            + self.levels[crate::types::O]
            + self.levels[crate::types::S];
        let denominator = 1.0 + self.levels[crate::types::C] + self.levels[crate::types::A];
        let tau = numerator / denominator;
        // Clamp to theoretical bounds [0.0, 3.0], practical [0.0, 2.0]
        // Max occurs when D=O=S=1, C=A=0 → τ = 3/1 = 3.0
        // But with clamped DOCAS in [0,1], realistic max is ~2.0
        tau.clamp(0.0, 2.0)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Trust Vector Field Curvature (κ) computation
///
/// Per §6.6.2: The trust vector field T(t) = ⟨TD, TO, TC, TA, TS⟩ has local curvature:
/// κ(t) = ∇·T(t) = Σ(∂Ti/∂xi) (divergence)
///
/// Interpretation:
/// - κ < 0: entropy falling → coherence (trust emerging)
/// - κ ≈ 0: equilibrium (stable system at critical point)
/// - κ > 0: entropy rising → instability (trust degrading)
///
/// Also computes systemic entropy: H = -Σ Ti × log(Ti) (Shannon entropy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustCurvature {
    /// Current curvature value (κ)
    pub kappa: f32,
    /// Current entropy value (H)
    pub entropy: f32,
    /// Rolling window of DOCAS states for divergence computation
    pub history: VecDeque<DocasState>,
    /// Maximum window size
    pub window_size: usize,
}

impl Default for TrustCurvature {
    fn default() -> Self {
        Self {
            kappa: 0.0,
            entropy: 0.0,
            history: VecDeque::with_capacity(CURVATURE_WINDOW_SIZE),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            window_size: CURVATURE_WINDOW_SIZE,
        }
    }
}

impl TrustCurvature {
    /// Create a new `TrustCurvature` with default window size
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new `TrustCurvature` with custom window size
    #[must_use]
    pub fn with_window_size(window_size: usize) -> Self {
        Self {
            kappa: 0.0,
            entropy: 0.0,
            history: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    /// Add a DOCAS state to the history window
    pub fn push_state(&mut self, state: DocasState) {
        if self.history.len() >= self.window_size {
            self.history.pop_front();
        }
        self.history.push_back(state);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute divergence (κ) from DOCAS history
    ///
    /// For each component, compute the rate of change over the window:
    /// κ ≈ Σ (`component_current` - `component_prev`) / `component_prev`
    ///
    /// Returns κ (divergence), where:
    /// - κ < 0: entropy falling → coherence (trust emerging)
    /// - κ ≈ 0: equilibrium (stable system)
    /// - κ > 0: entropy rising → instability (trust degrading)
    #[must_use]
    pub fn compute_divergence(docas_history: &[DocasState]) -> f32 {
        if docas_history.len() < 2 {
            return 0.0; // Not enough history
        }

        let current = &docas_history[docas_history.len() - 1];
        let previous = &docas_history[0]; // Oldest state in window

        let mut divergence = 0.0;

        for i in 0..DOCAS_SIZE {
            let curr_val = current.levels[i];
            let prev_val = previous.levels[i];

            // Avoid division by zero - use small epsilon
            let denominator = if prev_val > 0.0 { prev_val } else { 1e-6 };

            // Relative rate of change
            let rate_of_change = (curr_val - prev_val) / denominator;
            divergence += rate_of_change;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }

        divergence
    }

    /// Compute Shannon entropy H of DOCAS components
    ///
    /// H = -Σ Ti × log(Ti) where Ti are the normalized DOCAS components
    ///
    /// High entropy = more disorder = less predictable trust state
    /// Low entropy = more order = more coherent trust state
    #[must_use]
    pub fn compute_entropy(docas: &DocasState) -> f32 {
        let sum: f32 = docas.levels.iter().sum();

        // Avoid division by zero
        if sum <= 0.0 {
            return 0.0;
        }

        let mut entropy = 0.0;

        for &level in &docas.levels {
            // Normalize to probability distribution
            let p = level / sum;

            // Shannon entropy: -p * log(p), using natural log
            if p > 0.0 {
                entropy -= p * p.ln();
            }
        }

        entropy
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Predict instability based on curvature and entropy
    ///
    /// Returns an early warning score in [0, 1] where:
    /// - 0.0 = stable (negative curvature, low entropy)
    /// - 1.0 = high instability risk (positive curvature, high entropy)
    ///
    /// Formula: instability = sigmoid(κ) × `normalized_entropy`
    #[must_use]
    pub fn predict_instability(curvature: f32, entropy: f32) -> f32 {
        // Sigmoid of curvature to map to [0, 1]
        // Positive curvature → instability, negative → stability
        let kappa_factor = 1.0 / (1.0 + (-curvature).exp());

        // Normalize entropy: max entropy for 5 components is ln(5) ≈ 1.609
        let max_entropy = (DOCAS_SIZE as f32).ln();
        let normalized_entropy = (entropy / max_entropy).clamp(0.0, 1.0);

        // Combined instability score
        let instability = kappa_factor * normalized_entropy;

        instability.clamp(0.0, 1.0)
    }

    /// Update curvature and entropy from current history
    pub fn update(&mut self) {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        if let Some(current) = self.history.back() {
            self.entropy = Self::compute_entropy(current);
        }
        let history_vec: Vec<DocasState> = self.history.iter().copied().collect();
        self.kappa = Self::compute_divergence(&history_vec);
    }

    /// Get current curvature (κ)
    #[must_use]
    pub const fn curvature(&self) -> f32 {
        self.kappa
    }

    /// Get current entropy (H)
    #[must_use]
    pub const fn entropy(&self) -> f32 {
        self.entropy
    }

    /// Get instability prediction score
    #[must_use]
    pub fn instability_score(&self) -> f32 {
        Self::predict_instability(self.kappa, self.entropy)
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.history.clear();
        self.kappa = 0.0;
        self.entropy = 0.0;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the number of states in history
    #[must_use]
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Check if history has enough data for divergence computation
    #[must_use]
    pub fn has_sufficient_history(&self) -> bool {
        self.history.len() >= 2
    }
}

/// Non-Newtonian Viscosity dynamics for trust adaptation
/// Per §6.6.3: Trust resistance varies with perceived environmental stress
///
/// Viscosity equation:
/// ψ(t) = α × σ(β × |E(t)|) + (1-α) × τ
///
/// where:
/// - |E(t)|: perceived environmental volatility (std dev of recent trust changes)
/// - `σ()`: logistic activation function = 1 / (1 + exp(-x))
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// - τ: baseline resilience (mean trust level)
/// - α: stress sensitivity (default 0.3)
/// - β: volatility scaling (default 2.0)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TrustViscosity {
    /// Stress sensitivity α (0.0 to 1.0)
    /// Controls how much volatility affects viscosity vs baseline
    pub alpha: f32,

    /// Volatility scaling β
    /// Scales the perceived environmental volatility
    pub beta: f32,

    /// Baseline resilience τ
    /// Default trust level that anchors viscosity
    pub baseline_resilience: f32,
}

impl Default for TrustViscosity {
    /// Default values per §6.6.3
    /// α = 0.3, β = 2.0, τ = 0.75 (neutral trust)
    fn default() -> Self {
        Self {
            alpha: 0.3,
            beta: 2.0,
            baseline_resilience: 0.75,
        }
    }
}

impl TrustViscosity {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new `TrustViscosity` with specified parameters
    ///
    /// # Arguments
    /// * `alpha` - Stress sensitivity [0.0, 1.0]
    /// * `beta` - Volatility scaling (> 0.0)
    /// * `baseline_resilience` - Baseline trust level τ
    #[must_use]
    pub fn new(alpha: f32, beta: f32, baseline_resilience: f32) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
            beta: beta.max(0.01),
            baseline_resilience: baseline_resilience.clamp(0.0, 2.0),
        }
    }

    /// Compute environmental volatility |E(t)| from recent trust changes
    /// Returns standard deviation of the input values
    /// BANNED-001: "standard deviation" is a technical statistical term
    ///
    /// # Arguments
    /// * `recent_events` - Slice of recent trust values or changes
    ///
    /// # Returns
    /// Standard deviation of the values, or 0.0 if fewer than 2 values
    #[must_use]
    pub fn compute_volatility(recent_events: &[f32]) -> f32 {
        if recent_events.len() < 2 {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            return 0.0;
        }

        let n = recent_events.len() as f32;
        let mean = recent_events.iter().sum::<f32>() / n;

        let variance = recent_events
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>()
            / n;

        variance.sqrt()
    }

    /// Logistic activation function σ(x) = 1 / (1 + exp(-x))
    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Compute viscosity ψ(t) given current volatility and trust level
    ///
    /// ψ(t) = α × σ(β × |E(t)|) + (1-α) × τ
    ///
    /// # Arguments
    /// * `volatility` - Perceived environmental volatility |E(t)|
    /// * `current_tau` - Current trust level τ (may differ from baseline)
    ///
    /// # Returns
    /// Viscosity value ψ(t) representing resistance to trust change
    #[must_use]
    pub fn compute_viscosity(&self, volatility: f32, current_tau: f32) -> f32 {
        let stress_component = Self::sigmoid(self.beta * volatility);
        let resilience_component = current_tau.clamp(0.0, 2.0);

        self.alpha * stress_component + (1.0 - self.alpha) * resilience_component
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute adaptation rate as inverse of viscosity
    /// Returns how fast trust can change (1/ψ)
    ///
    /// # Arguments
    /// * `viscosity` - Current viscosity ψ(t)
    ///
    /// # Returns
    /// Adaptation rate in range (0, 1], higher = faster adaptation
    #[must_use]
    pub fn adaptation_rate(&self, viscosity: f32) -> f32 {
        // Avoid division by zero, clamp viscosity to minimum
        let clamped_viscosity = viscosity.max(0.01);
        (1.0 / clamped_viscosity).min(1.0)
    }

    /// Compute volatility from first-order differences (changes)
    /// Alternative method that looks at rate of change rather than absolute values
    ///
    /// # Arguments
    /// * `recent_values` - Slice of recent trust values
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ///
    /// # Returns
    /// Standard deviation of the differences between consecutive values
    #[must_use]
    pub fn compute_volatility_from_changes(recent_values: &[f32]) -> f32 {
        if recent_values.len() < 2 {
            return 0.0;
        }

        // Compute first-order differences (changes)
        let changes: Vec<f32> = recent_values
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .collect();

        Self::compute_volatility(&changes)
    }
}

/// Viscosity regime classification per §6.6.3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViscosityRegime {
    /// Thixotropy: Rapid viscosity drop under shock → fast adaptation (risk of collapse)
    /// Occurs when volatility spikes suddenly
    Thixotropic,
    /// Rheopexy: Viscosity increase under validation → strengthening
    /// Occurs when trust stabilizes with positive validation
    Rheopectic,
    /// Neutral: Normal viscosity dynamics
    Neutral,
}

impl ViscosityRegime {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Classify regime based on volatility trend and current trust level
    ///
    /// # Arguments
    /// * `current_volatility` - Current environmental volatility
    /// * `previous_volatility` - Previous tick's volatility
    /// * `current_tau` - Current trust level
    ///
    /// # Returns
    /// Classified viscosity regime
    #[must_use]
    pub fn classify(current_volatility: f32, previous_volatility: f32, current_tau: f32) -> Self {
        const VOLATILITY_SPIKE_THRESHOLD: f32 = 0.5;
        const HIGH_TRUST_THRESHOLD: f32 = 1.2;

        let volatility_change = current_volatility - previous_volatility;

        if volatility_change > VOLATILITY_SPIKE_THRESHOLD {
            // Rapid increase in volatility → thixotropic (fast adaptation risk)
            Self::Thixotropic
        } else if volatility_change < -0.2 && current_tau > HIGH_TRUST_THRESHOLD {
            // Decreasing volatility with high trust → rheopectic (strengthening)
            Self::Rheopectic
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        } else {
            Self::Neutral
        }
    }
}

/// Compute cosine similarity between two vectors
/// Returns value in [-1.0, 1.0], where 1.0 means identical direction
/// Returns 0.0 if either vector is zero-length
/// Per §6.6.4: `docas_similarity` = `cosine_similarity(agent_i.docas`, `agent_j.docas`)
#[must_use]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for i in 0..a.len() {
        if !a[i].is_finite() || !b[i].is_finite() {
            continue;
        }
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    let norm_a_sqrt = norm_a.sqrt();
    let norm_b_sqrt = norm_b.sqrt();

    if norm_a_sqrt <= 1e-12
        || norm_b_sqrt <= 1e-12
        || !norm_a_sqrt.is_finite()
        || !norm_b_sqrt.is_finite()
    {
        return 0.0;
    }

    let denom = norm_a_sqrt * norm_b_sqrt;
    if denom <= 1e-12 || !denom.is_finite() {
        return 0.0;
    }

    (dot_product / denom).clamp(-1.0, 1.0)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Multi-Agent Coupling Field (Φij)
/// Per §6.6.4: Computes influence between agents based on:
/// - Spatial distance decay
/// - Credibility (reputation)
/// - DOCAS state similarity
///
/// Formula: Φij = `base_coupling` × `distance_decay` × credibility × `docas_similarity`
#[derive(Debug, Clone)]
pub struct CouplingField {
    /// Coupling coefficients for agent pairs (i, j) where i < j
    /// Stored as symmetric pairs to avoid duplication
    coefficients: HashMap<(AgentId, AgentId), f32>,
    /// Update interval in simulation ticks (default: 10)
    pub update_interval: Tick,
    /// Maximum coupling radius - agents beyond this distance have Φij = 0 (default: 10.0)
    pub coupling_radius: f32,
    /// Base coupling strength (default: 0.1)
    pub base_coupling: f32,
    /// Decay constant for distance (default: 10.0)
    distance_decay_constant: f32,
}

impl Default for CouplingField {
    fn default() -> Self {
        Self {
            coefficients: HashMap::new(),
            update_interval: 10,
            coupling_radius: 10.0,
            base_coupling: 0.1,
            distance_decay_constant: 10.0,
        }
    }
}

impl CouplingField {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Create a new `CouplingField` with default parameters
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new `CouplingField` with custom parameters
    #[must_use]
    pub fn with_params(
        update_interval: Tick,
        coupling_radius: f32,
        base_coupling: f32,
        distance_decay_constant: f32,
    ) -> Self {
        Self {
            coefficients: HashMap::new(),
            update_interval,
            coupling_radius,
            base_coupling,
            distance_decay_constant,
        }
    }

    /// Compute coupling coefficient Φij for a pair of agents
    /// Per §6.6.4:
    /// - `base_coupling` = 0.1 (configurable)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// - `distance_decay` = exp(-distance / 10.0)
    /// - credibility = `agent_j.reputation_score.clamp(0.0`, 1.0)
    /// - `docas_similarity` = `cosine_similarity(agent_i.docas`, `agent_j.docas`)
    #[must_use]
    pub fn compute_phi_ij(&self, agent_i: &Agent, agent_j: &Agent) -> f32 {
        // Calculate distance
        let distance = agent_i.position.euclidean_distance(&agent_j.position);

        // Beyond coupling radius: no coupling
        if distance > self.coupling_radius {
            return 0.0;
        }

        // Distance decay: exp(-distance / decay_constant)
        let distance_decay = (-distance / self.distance_decay_constant).exp();

        // Credibility: clamp reputation to [0.0, 1.0]
        let credibility = agent_j.reputation_score.clamp(0.0, 1.0);

        // DOCAS similarity: cosine similarity of DOCAS state vectors
        let docas_similarity = cosine_similarity(&agent_i.docas.levels, &agent_j.docas.levels);

        // Φij = base_coupling × distance_decay × credibility × docas_similarity
        self.base_coupling * distance_decay * credibility * docas_similarity
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update all coupling coefficients for the given set of agents
    /// Only computes Φij for pairs within `coupling_radius`
    pub fn update_field(&mut self, agents: &[Agent]) {
        self.coefficients.clear();

        // Compute coupling for all pairs (i, j) where i < j
        for i in 0..agents.len() {
            for j in (i + 1)..agents.len() {
                let agent_i = &agents[i];
                let agent_j = &agents[j];

                let phi = self.compute_phi_ij(agent_i, agent_j);

                // Only store non-zero couplings
                if phi > 0.0 {
                    let key = if agent_i.id < agent_j.id {
                        (agent_i.id, agent_j.id)
                    } else {
                        (agent_j.id, agent_i.id)
                    };
                    self.coefficients.insert(key, phi);
                }
            }
        }
    }

    /// Compute mean field for an agent: sum of Φij over all j
    /// Returns the total coupling strength experienced by the agent
    #[must_use]
    pub fn mean_field(&self, agent_id: AgentId) -> f32 {
        let mut total = 0.0;

        for ((id_i, id_j), phi) in &self.coefficients {
            if *id_i == agent_id || *id_j == agent_id {
                total += phi;
            }
        }

        total
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the stored coupling coefficient for a pair of agents
    /// Returns 0.0 if no coupling exists (e.g., beyond `coupling_radius`)
    #[must_use]
    pub fn get_coupling(&self, i: AgentId, j: AgentId) -> f32 {
        if i == j {
            return 0.0;
        }

        let key = if i < j { (i, j) } else { (j, i) };
        self.coefficients.get(&key).copied().unwrap_or(0.0)
    }

    /// Get the number of stored coupling coefficients
    #[must_use]
    pub fn coefficient_count(&self) -> usize {
        self.coefficients.len()
    }

    /// Get all coupling coefficients (for debugging/analysis)
    #[must_use]
    pub const fn coefficients(&self) -> &HashMap<(AgentId, AgentId), f32> {
        &self.coefficients
    }

    /// Clear all coefficients
    pub fn clear(&mut self) {
        self.coefficients.clear();
    }

    /// Check if a coupling exists between two agents
    #[must_use]
    pub fn has_coupling(&self, i: AgentId, j: AgentId) -> bool {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        if i == j {
            return false;
        }
        let key = if i < j { (i, j) } else { (j, i) };
        self.coefficients.contains_key(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::Genome;
    use crate::types::{Position, Sex};

    #[test]
    fn test_docas_default() {
        let d = DocasState::default();
        assert_eq!(d.levels, [0.5, 0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_docas_tau() {
        let d = DocasState::new([0.8, 0.6, 0.2, 0.1, 0.7]);
        let tau = d.compute_tau();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // τ = (0.8 + 0.6 + 0.7) / (1 + 0.2 + 0.1) = 2.1 / 1.3 ≈ 1.615
        assert!((tau - 1.615).abs() < 0.01);
    }

    #[test]
    fn test_docas_update_clamps() {
        let mut d = DocasState::new([0.9, 0.9, 0.9, 0.9, 0.9]);
        let g = Genome::default();
        // Large positive stimulus should clamp to 1.0
        // P3-1: Include geometric parameters (kappa, psi) - neutral values for test
        d.update(&g, 1.0, 1.0, 1.0, 1.0, 1.0);
        for level in d.levels {
            assert!(level <= 1.0);
        }
    }

    // === TrustCurvature Tests ===

    #[test]
    fn test_trust_curvature_default() {
        let tc = TrustCurvature::default();
        assert_eq!(tc.kappa, 0.0);
        assert_eq!(tc.entropy, 0.0);
        assert_eq!(tc.history.len(), 0);
        assert_eq!(tc.window_size, CURVATURE_WINDOW_SIZE);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_trust_curvature_push_state() {
        let mut tc = TrustCurvature::new();
        let state = DocasState::new([0.6, 0.5, 0.4, 0.3, 0.7]);
        tc.push_state(state);
        assert_eq!(tc.history.len(), 1);
        assert_eq!(tc.history[0].levels, [0.6, 0.5, 0.4, 0.3, 0.7]);
    }

    #[test]
    fn test_trust_curvature_compute_entropy() {
        // Equal distribution: H = -5 × (0.2 × ln(0.2)) / ln(2) = ln(5) ≈ 1.609 / ln(2) in bits
        let state = DocasState::new([0.2, 0.2, 0.2, 0.2, 0.2]);
        let entropy = TrustCurvature::compute_entropy(&state);
        // All levels equal, sum = 1.0, p = 0.2 for each
        // H = -5 × 0.2 × ln(0.2) = -ln(0.2) = ln(5) ≈ 1.609
        assert!(entropy > 1.5 && entropy < 1.7);
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_trust_curvature_compute_divergence() {
        // Create history with increasing values
        let history = vec![
            DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]),
            DocasState::new([0.6, 0.6, 0.6, 0.6, 0.6]),
        ];

        let divergence = TrustCurvature::compute_divergence(&history);
        // κ = Σ (0.6 - 0.5) / 0.5 = 5 × 0.2 = 1.0
        assert!((divergence - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_trust_curvature_update() {
        let mut tc = TrustCurvature::new();

        // Push initial state
        tc.push_state(DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]));
        tc.update();

        // With only one state, divergence should be 0
        assert_eq!(tc.kappa, 0.0);

        // Push second state with higher values
        tc.push_state(DocasState::new([0.6, 0.6, 0.6, 0.6, 0.6]));
        tc.update();

        // Now divergence should be computed
        assert!(tc.kappa > 0.0);
    }

    #[test]
    fn test_trust_curvature_predict_instability() {
        // High curvature and high entropy = high instability
        let instability = TrustCurvature::predict_instability(1.0, 1.5);
        assert!(instability > 0.5 && instability <= 1.0);

        // Negative curvature (coherence) should give low instability
        let stable = TrustCurvature::predict_instability(-1.0, 0.5);
        assert!(stable < 0.5);
    }

    #[test]
    fn test_trust_curvature_window_size() {
        let mut tc = TrustCurvature::with_window_size(5);
        assert_eq!(tc.window_size, 5);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Fill beyond capacity
        for i in 0..10 {
            tc.push_state(DocasState::new([0.1 * i as f32; 5]));
        }

        // Should only keep window_size elements
        assert_eq!(tc.history.len(), 5);
    }

    // === Cosine Similarity Tests ===

    #[test]
    fn test_cosine_similarity_identical() {
        let a = [1.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = [1.0, 0.0, 0.0];
        let b = [-1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_cosine_similarity_docas_states() {
        let docas_a = DocasState::new([0.8, 0.6, 0.2, 0.1, 0.7]);
        let docas_b = DocasState::new([0.7, 0.5, 0.3, 0.2, 0.6]);
        let sim = cosine_similarity(&docas_a.levels, &docas_b.levels);
        // Similar vectors should have high positive similarity
        assert!(sim > 0.9);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(sim <= 1.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    // === CouplingField Tests ===

    #[test]
    fn test_coupling_field_new() {
        let field = CouplingField::new();
        assert_eq!(field.update_interval, 10);
        assert_eq!(field.coupling_radius, 10.0);
        assert_eq!(field.base_coupling, 0.1);
        assert_eq!(field.coefficient_count(), 0);
    }

    #[test]
    fn test_coupling_field_custom_params() {
        let field = CouplingField::with_params(20, 15.0, 0.2, 5.0);
        assert_eq!(field.update_interval, 20);
        assert_eq!(field.coupling_radius, 15.0);
        assert_eq!(field.base_coupling, 0.2);
    }

    #[test]
    fn test_compute_phi_ij_basic() {
        let genome = Genome::default();
        let agent_i = Agent::new(1, Position::new_2d(0.0, 0.0), genome.clone(), 0, Sex::Male);
        let mut agent_j = Agent::new(2, Position::new_2d(1.0, 0.0), genome, 0, Sex::Female);

        // Set reputation to 1.0 for maximum credibility
        agent_j.reputation_score = 1.0;

        // Both have default DOCAS [0.5, 0.5, 0.5, 0.5, 0.5], so similarity = 1.0
        let field = CouplingField::new();
        let phi = field.compute_phi_ij(&agent_i, &agent_j);

        // Expected: 0.1 * exp(-1.0/10.0) * 1.0 * 1.0 ≈ 0.1 * 0.9048 ≈ 0.0905
        let expected = 0.1 * (-1.0f32 / 10.0).exp();
        assert!((phi - expected).abs() < 0.001);
    }

    #[test]
    fn test_compute_phi_ij_beyond_radius() {
        let genome = Genome::default();
        let agent_i = Agent::new(1, Position::new_2d(0.0, 0.0), genome.clone(), 0, Sex::Male);
        let agent_j = Agent::new(2, Position::new_2d(15.0, 0.0), genome, 0, Sex::Female);

        let field = CouplingField::new(); // coupling_radius = 10.0
        let phi = field.compute_phi_ij(&agent_i, &agent_j);

        assert_eq!(phi, 0.0);
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    #[test]
    fn test_compute_phi_ij_zero_credibility() {
        let genome = Genome::default();
        let agent_i = Agent::new(1, Position::new_2d(0.0, 0.0), genome.clone(), 0, Sex::Male);
        let mut agent_j = Agent::new(2, Position::new_2d(1.0, 0.0), genome, 0, Sex::Female);

        // Zero reputation = zero credibility
        agent_j.reputation_score = 0.0;

        let field = CouplingField::new();
        let phi = field.compute_phi_ij(&agent_i, &agent_j);

        assert_eq!(phi, 0.0);
    }

    #[test]
    fn test_compute_phi_ij_reputation_clamping() {
        let genome = Genome::default();
        let agent_i = Agent::new(1, Position::new_2d(0.0, 0.0), genome.clone(), 0, Sex::Male);
        let mut agent_j = Agent::new(2, Position::new_2d(1.0, 0.0), genome, 0, Sex::Female);

        // Negative reputation should be clamped to 0.0
        agent_j.reputation_score = -0.5;

        let field = CouplingField::new();
        let phi = field.compute_phi_ij(&agent_i, &agent_j);

        assert_eq!(phi, 0.0);
    }

    #[test]
    fn test_update_field() {
        let genome = Genome::default();
        let mut agent1 = Agent::new(1, Position::new_2d(0.0, 0.0), genome.clone(), 0, Sex::Male);
        let mut agent2 = Agent::new(
            2,
            Position::new_2d(2.0, 0.0),
            genome.clone(),
            0,
            Sex::Female,
        );
        let mut agent3 = Agent::new(3, Position::new_2d(20.0, 0.0), genome, 0, Sex::Male);

        // Set reputations
        agent1.reputation_score = 1.0;
        agent2.reputation_score = 1.0;
        agent3.reputation_score = 1.0;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let agents = vec![agent1, agent2, agent3];
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut field = CouplingField::new();

        field.update_field(&agents);

        // Should have coupling between 1-2 (within radius), but not with 3 (beyond radius)
        assert!(field.has_coupling(1, 2));
        assert!(!field.has_coupling(1, 3));
        assert!(!field.has_coupling(2, 3));
        assert_eq!(field.coefficient_count(), 1);
    }

    #[test]
    fn test_get_coupling() {
        let genome = Genome::default();
        let mut agent1 = Agent::new(1, Position::new_2d(0.0, 0.0), genome.clone(), 0, Sex::Male);
        let mut agent2 = Agent::new(2, Position::new_2d(2.0, 0.0), genome, 0, Sex::Female);

        agent1.reputation_score = 1.0;
        agent2.reputation_score = 1.0;

        let agents = vec![agent1, agent2];
        let mut field = CouplingField::new();
        field.update_field(&agents);

        let phi = field.get_coupling(1, 2);
        assert!(phi > 0.0);

        // Should be symmetric
        assert_eq!(phi, field.get_coupling(2, 1));

        // Self-coupling should be 0
        assert_eq!(field.get_coupling(1, 1), 0.0);

        // Non-existent coupling should be 0
        assert_eq!(field.get_coupling(1, 99), 0.0);
    }

    #[test]
    fn test_mean_field() {
        let genome = Genome::default();
        let mut agent1 = Agent::new(1, Position::new_2d(0.0, 0.0), genome.clone(), 0, Sex::Male);
        let mut agent2 = Agent::new(
            2,
            Position::new_2d(2.0, 0.0),
            genome.clone(),
            0,
            Sex::Female,
        );
        let mut agent3 = Agent::new(3, Position::new_2d(3.0, 0.0), genome, 0, Sex::Male);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        agent1.reputation_score = 1.0;
        agent2.reputation_score = 1.0;
        agent3.reputation_score = 1.0;

        let agents = vec![agent1, agent2, agent3];
        let mut field = CouplingField::new();
        field.update_field(&agents);

        // Agent 1 should have coupling to agents 2 and 3
        let mean_field_1 = field.mean_field(1);
        let phi_12 = field.get_coupling(1, 2);
        let phi_13 = field.get_coupling(1, 3);

        assert!((mean_field_1 - (phi_12 + phi_13)).abs() < 0.0001);
    }

    #[test]
    fn test_clear() {
        let genome = Genome::default();
        let mut agent1 = Agent::new(1, Position::new_2d(0.0, 0.0), genome.clone(), 0, Sex::Male);
        let mut agent2 = Agent::new(2, Position::new_2d(2.0, 0.0), genome, 0, Sex::Female);

        agent1.reputation_score = 1.0;
        agent2.reputation_score = 1.0;

        let agents = vec![agent1, agent2];
        let mut field = CouplingField::new();
        field.update_field(&agents);

        assert_eq!(field.coefficient_count(), 1);

        field.clear();
        assert_eq!(field.coefficient_count(), 0);
        assert_eq!(field.get_coupling(1, 2), 0.0);
    }

    // === TrustViscosity Tests (§6.6.3) ===

    #[test]
    fn test_trust_viscosity_default() {
        let tv = TrustViscosity::default();
        assert!((tv.alpha - 0.3).abs() < 0.001);
        assert!((tv.beta - 2.0).abs() < 0.001);
        assert!((tv.baseline_resilience - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_trust_viscosity_new() {
        let tv = TrustViscosity::new(0.5, 3.0, 1.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!((tv.alpha - 0.5).abs() < 0.001);
        assert!((tv.beta - 3.0).abs() < 0.001);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!((tv.baseline_resilience - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_trust_viscosity_clamping() {
        // Alpha should clamp to [0, 1]
        let tv = TrustViscosity::new(-0.5, 3.0, 1.0);
        assert_eq!(tv.alpha, 0.0);

        let tv = TrustViscosity::new(1.5, 3.0, 1.0);
        assert_eq!(tv.alpha, 1.0);

        // Beta should be at least 0.01
        let tv = TrustViscosity::new(0.3, 0.0, 1.0);
        assert_eq!(tv.beta, 0.01);
    }

    #[test]
    fn test_compute_volatility() {
        // Empty or single element returns 0
        assert_eq!(TrustViscosity::compute_volatility(&[]), 0.0);
        assert_eq!(TrustViscosity::compute_volatility(&[1.0]), 0.0);

        // Two identical values have 0 std dev
        let values = [1.0, 1.0];
        assert_eq!(TrustViscosity::compute_volatility(&values), 0.0);

        // Known std dev calculation
        // Mean = 2.0, variance = ((1-2)² + (2-2)² + (3-2)²) / 3 = (1 + 0 + 1) / 3 = 0.667
        // std dev = sqrt(0.667) ≈ 0.816
        let values = [1.0, 2.0, 3.0];
        let vol = TrustViscosity::compute_volatility(&values);
        assert!((vol - 0.816).abs() < 0.01);
    }

    #[test]
    fn test_compute_volatility_from_changes() {
        // Changes: [0.1, 0.2, 0.1] → mean = 0.133, var = 0.00222, std = 0.047
        let values = [1.0, 1.1, 1.3, 1.4];
        let vol = TrustViscosity::compute_volatility_from_changes(&values);
        assert!(vol > 0.0);
    }

    #[test]
    fn test_sigmoid() {
        // sigmoid(0) = 0.5
        assert!((TrustViscosity::sigmoid(0.0) - 0.5).abs() < 0.001);

        // sigmoid(large positive) → 1
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(TrustViscosity::sigmoid(5.0) > 0.99);

        // sigmoid(large negative) → 0
        assert!(TrustViscosity::sigmoid(-5.0) < 0.01);
    }

    #[test]
    fn test_compute_viscosity() {
        let tv = TrustViscosity::new(0.5, 2.0, 0.75);

        // Zero volatility, tau = 1.0
        // ψ = 0.5 * sigmoid(0) + 0.5 * 1.0 = 0.5 * 0.5 + 0.5 = 0.75
        let psi = tv.compute_viscosity(0.0, 1.0);
        assert!((psi - 0.75).abs() < 0.01);

        // High volatility, tau = 1.0
        // sigmoid(2.0 * 2.0) = sigmoid(4.0) ≈ 0.982
        // ψ = 0.5 * 0.982 + 0.5 * 1.0 ≈ 0.991
        let psi = tv.compute_viscosity(2.0, 1.0);
        assert!(psi > 0.9);
    }

    #[test]
    fn test_adaptation_rate() {
        let tv = TrustViscosity::new(0.3, 2.0, 0.75);

        // Low viscosity → high adaptation rate (clamped to 1.0)
        let rate = tv.adaptation_rate(0.5);
        assert_eq!(rate, 1.0);

        // High viscosity → low adaptation rate
        let rate = tv.adaptation_rate(2.0);
        assert!((rate - 0.5).abs() < 0.01);

        // Very high viscosity
        let rate = tv.adaptation_rate(5.0);
        assert!((rate - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_viscosity_regime_classify() {
        // Large volatility spike → Thixotropic
        let regime = ViscosityRegime::classify(1.0, 0.2, 0.8);
        assert_eq!(regime, ViscosityRegime::Thixotropic);

        // Decreasing volatility with high trust → Rheopectic
        let regime = ViscosityRegime::classify(0.3, 0.6, 1.5);
        assert_eq!(regime, ViscosityRegime::Rheopectic);

        // Neutral case
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let regime = ViscosityRegime::classify(0.5, 0.4, 0.8);
        assert_eq!(regime, ViscosityRegime::Neutral);
    }
}
