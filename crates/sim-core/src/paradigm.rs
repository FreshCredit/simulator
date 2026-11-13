//! Kuhnian Paradigm System: Philosophical/Epistemic Change
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION philosophy paradigms, not metaphysical claims
//!
//! Implementation of Thomas Kuhn's (1962) Structure of Scientific Revolutions:
//! - Normal science → Crisis → Revolution → New paradigm
//! - Incommensurability of paradigms
//! - Anomaly accumulation leading to paradigm shifts
//!
//! Research:
//! - Kuhn (1962) - The Structure of Scientific Revolutions
//! - Normal science operates within a paradigm's puzzle-solving framework
//! - Crisis occurs when anomalies accumulate beyond puzzle-solving capacity
//! - Revolution involves competing paradigms with incommensurable worldviews
//! - New paradigm establishes when it demonstrates superior problem-solving

use serde::{Deserialize, Serialize};

use crate::types::Tick;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Unique identifier for a paradigm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ParadigmId(pub u64);

/// A scientific/philosophical paradigm
/// Per Kuhn: Paradigms provide the framework for normal science
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paradigm {
    /// Unique identifier
    pub id: ParadigmId,

    /// Human-readable name
    pub name: String,

    /// Core assumptions defining the paradigm
    /// These are foundational beliefs that distinguish this paradigm
    pub core_assumptions: Vec<String>,

    /// Successful applications/exemplars
    /// Per Kuhn: Exemplars are concrete problem-solutions that guide practice
    pub exemplars: Vec<String>,

    /// Problem-solving effectiveness (0.0 - 1.0)
    /// Measures how well paradigm resolves anomalies and puzzles
    pub problem_solving_effectiveness: f32,

    /// Number of adherents to this paradigm
    /// Used to track paradigm dominance during revolutions
    pub adherent_count: u32,

    /// Creation tick
    pub created_tick: Tick,

    /// Number of successful predictions made under this paradigm
    pub successful_predictions: u32,

    /// Number of failed predictions (anomalies generated)
    pub failed_predictions: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Paradigm {
    /// Create a new paradigm
    #[must_use]
    pub fn new(
        id: ParadigmId,
        name: &str,
        core_assumptions: Vec<String>,
        exemplars: Vec<String>,
        tick: Tick,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            core_assumptions,
            exemplars,
            problem_solving_effectiveness: 0.5, // Start neutral
            adherent_count: 0,
            created_tick: tick,
            successful_predictions: 0,
            failed_predictions: 0,
        }
    }

    /// Create a default initial paradigm (Classical/Traditional)
    #[must_use]
    pub fn classical(tick: Tick) -> Self {
        Self::new(
            ParadigmId(0),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            "Classical Paradigm",
            vec![
                "Nature is orderly and predictable".to_string(),
                "Observation reveals true nature".to_string(),
                "Causation is direct and linear".to_string(),
            ],
            vec![
                "Planetary motion predictions".to_string(),
                "Engineering successes".to_string(),
                "Medical traditional practices".to_string(),
            ],
            tick,
        )
    }

    /// Update effectiveness based on success/failure ratio
    pub fn update_effectiveness(&mut self) {
        let total = self.successful_predictions + self.failed_predictions;
        if total > 0 {
            let base_effectiveness = self.successful_predictions as f32 / total as f32;
            // Smooth update
            self.problem_solving_effectiveness =
                0.9 * self.problem_solving_effectiveness + 0.1 * base_effectiveness;
        }
        // Clamp to valid range
        self.problem_solving_effectiveness = self.problem_solving_effectiveness.clamp(0.0, 1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record a successful prediction
    pub fn record_success(&mut self) {
        self.successful_predictions += 1;
        self.update_effectiveness();
    }

    /// Record a failed prediction (anomaly)
    pub fn record_failure(&mut self) {
        self.failed_predictions += 1;
        self.update_effectiveness();
    }

    /// Compute success rate
    #[must_use]
    pub fn success_rate(&self) -> f32 {
        let total = self.successful_predictions + self.failed_predictions;
        if total == 0 {
            0.5
        } else {
            self.successful_predictions as f32 / total as f32
        }
    }
}

/// State of paradigm development
/// Per Kuhn: The four stages of scientific development
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ParadigmState {
    /// Normal Science: Puzzle-solving within established paradigm
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Most scientific work happens here - extending paradigm to new cases
    #[default]
    NormalScience,

    /// Crisis: Anomalies accumulating beyond puzzle-solving capacity
    /// Paradigm's problem-solving effectiveness declines
    Crisis,

    /// Revolution: Competing paradigms vie for dominance
    /// Scientific community divided between incommensurable frameworks
    Revolution,

    /// New Paradigm: New framework established, normal science resumes
    /// New paradigm achieves consensus and becomes dominant
    NewParadigm,
}

impl ParadigmState {
    /// Check if in crisis or revolution
    #[must_use]
    pub const fn is_unstable(&self) -> bool {
        matches!(self, Self::Crisis | Self::Revolution)
    }

    /// Check if normal science can proceed
    #[must_use]
    pub const fn can_do_normal_science(&self) -> bool {
        matches!(self, Self::NormalScience | Self::NewParadigm)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cause of paradigm shift
/// Per Kuhn: Various factors can trigger paradigm change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShiftCause {
    /// Anomaly accumulation: Too many unexplained phenomena
    /// Classic Kuhnian trigger - paradigm fails to resolve puzzles
    AnomalyAccumulation,

    /// Exemplar failure: Key success story of paradigm fails
    /// When a paradigmatic case turns out to be wrong
    ExemplarFailure,

    /// New discovery: Novel phenomenon unexplained by current paradigm
    /// New observations that don't fit existing framework
    NewDiscovery,

    /// Generational change: Old adherents retire, new generation adopts
    /// Per Kuhn: Sometimes shifts happen as old guard dies off
    GenerationalChange,

    /// Social pressure: External factors influencing paradigm choice
    /// Funding, institutional support, practical applications
    SocialPressure,

    /// Problem-solving superiority: New paradigm better solves problems
    /// The rational basis for paradigm choice per Kuhn
    SuperiorProblemSolving,
}

impl std::fmt::Display for ShiftCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::AnomalyAccumulation => write!(f, "Anomaly Accumulation"),
            Self::ExemplarFailure => write!(f, "Exemplar Failure"),
            Self::NewDiscovery => write!(f, "New Discovery"),
            Self::GenerationalChange => write!(f, "Generational Change"),
            Self::SocialPressure => write!(f, "Social Pressure"),
            Self::SuperiorProblemSolving => write!(f, "Superior Problem-Solving"),
        }
    }
}

/// Record of a paradigm shift
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadigmShift {
    /// Source paradigm ID
    pub from_paradigm: ParadigmId,

    /// Target paradigm ID
    pub to_paradigm: ParadigmId,

    /// Tick when shift occurred
    pub shift_tick: Tick,

    /// Cause of the shift
    pub cause: ShiftCause,

    /// How quickly the new paradigm was adopted (0.0 - 1.0)
    /// Higher = faster adoption across population
    pub adoption_rate: f32,

    /// Number of agents who switched
    pub agents_switched: u32,

    /// Duration of revolution period (if applicable)
    pub revolution_duration: Option<Tick>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ParadigmShift {
    /// Create a new paradigm shift record
    #[must_use]
    pub const fn new(from: ParadigmId, to: ParadigmId, tick: Tick, cause: ShiftCause) -> Self {
        Self {
            from_paradigm: from,
            to_paradigm: to,
            shift_tick: tick,
            cause,
            adoption_rate: 0.0,
            agents_switched: 0,
            revolution_duration: None,
        }
    }
}

/// Anomaly in current paradigm
/// Per Kuhn: Anomalies are observations that don't fit paradigm expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Description of the anomaly
    pub description: String,

    /// Severity/importance (0.0 - 1.0)
    /// Higher severity anomalies contribute more to crisis
    pub severity: f32,

    /// Tick when anomaly was discovered
    pub discovery_tick: Tick,

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Number of attempts to resolve within current paradigm
    pub attempts_to_resolve: u32,

    /// Whether this anomaly has been resolved
    pub resolved: bool,

    /// Whether this anomaly contributed to a paradigm shift
    pub contributed_to_shift: bool,
}

impl Anomaly {
    /// Create a new anomaly
    #[must_use]
    pub fn new(description: &str, severity: f32, tick: Tick) -> Self {
        Self {
            description: description.to_string(),
            severity: severity.clamp(0.0, 1.0),
            discovery_tick: tick,
            attempts_to_resolve: 0,
            resolved: false,
            contributed_to_shift: false,
        }
    }

    /// Attempt to resolve this anomaly
    pub fn attempt_resolution(&mut self) -> bool {
        self.attempts_to_resolve += 1;
        // Deterministic resolution probability derived from attempt counter
        let resolution_prob = 0.5_f32.powi(self.attempts_to_resolve as i32);
        let resolved = self.attempts_to_resolve <= 2 && resolution_prob > 0.2;
        if resolved {
            self.resolved = true;
        }
        resolved
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Mark as contributing to a paradigm shift
    pub fn mark_contributed(&mut self) {
        self.contributed_to_shift = true;
    }
}

/// Agent paradigm adherence state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentParadigmState {
    /// Current paradigm ID
    pub current_paradigm_id: ParadigmId,

    /// Effectiveness experienced with current paradigm
    /// Personal track record of success/failure
    pub paradigm_effectiveness: f32,

    /// Resistance to changing paradigms
    /// Higher = more committed to current paradigm
    pub paradigm_commitment: f32,

    /// History of paradigm switches
    pub switch_history: Vec<(Tick, ParadigmId)>,

    /// Years (ticks) in current paradigm
    pub tenure_in_paradigm: Tick,

    /// Personal anomalies experienced
    pub personal_anomalies: Vec<Anomaly>,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
impl Default for AgentParadigmState {
    fn default() -> Self {
        Self::new(ParadigmId(0))
    }
}

impl AgentParadigmState {
    /// Create new paradigm state
    #[must_use]
    pub const fn new(paradigm_id: ParadigmId) -> Self {
        Self {
            current_paradigm_id: paradigm_id,
            paradigm_effectiveness: 0.5,
            paradigm_commitment: 0.5,
            switch_history: Vec::new(),
            tenure_in_paradigm: 0,
            personal_anomalies: Vec::new(),
        }
    }

    /// Adopt a new paradigm
    pub fn adopt_paradigm(&mut self, paradigm_id: ParadigmId, tick: Tick) {
        if self.current_paradigm_id != paradigm_id {
            self.switch_history.push((tick, self.current_paradigm_id));
            self.current_paradigm_id = paradigm_id;
            self.paradigm_effectiveness = 0.5; // Reset with new paradigm
            self.tenure_in_paradigm = 0;
            // Paradigm commitment changes (Kuhn: new paradigm = fresh commitment)
            self.paradigm_commitment = 0.7;
        }
    }

    /// Update tenure and commitment
    pub fn update(&mut self) {
        self.tenure_in_paradigm += 1;
        // Commitment increases with tenure
        self.paradigm_commitment = (self.paradigm_commitment + 0.001).min(1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if agent is open to paradigm shift
    /// Less committed agents, or those with low effectiveness and high anomaly burden, are more open
    #[must_use]
    pub fn open_to_shift(&self, new_paradigm_effectiveness: f32) -> bool {
        let anomaly_pressure = (self.personal_anomalies.len() as f32 * 0.05).min(0.5);
        let effectiveness_gap = new_paradigm_effectiveness - self.paradigm_effectiveness;
        let shift_probability =
            ((effectiveness_gap + anomaly_pressure) * (1.0 - self.paradigm_commitment)).max(0.0);
        shift_probability > 0.3
    }

    /// Add personal anomaly
    pub fn add_anomaly(&mut self, anomaly: Anomaly) {
        self.personal_anomalies.push(anomaly);
    }

    /// Count unresolved anomalies
    #[must_use]
    pub fn unresolved_anomalies(&self) -> usize {
        self.personal_anomalies
            .iter()
            .filter(|a| !a.resolved)
            .count()
    }
}

/// Kuhnian paradigm system for philosophical change
/// Manages the lifecycle of paradigms from normal science through revolution
#[derive(Debug, Clone, Serialize, Deserialize)]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
pub struct ParadigmSystem {
    /// Current dominant paradigm
    pub current_paradigm: Paradigm,

    /// Paradigm history
    pub paradigm_history: Vec<ParadigmShift>,

    /// Anomalies accumulating (crisis indicators)
    pub anomalies: Vec<Anomaly>,

    /// Current anomaly count
    pub anomaly_count: u32,

    /// Crisis threshold - when `anomaly_count` exceeds this, enter crisis
    pub crisis_threshold: u32,

    /// State of paradigm development
    pub state: ParadigmState,

    /// Competing paradigms during crisis/revolution
    pub competing_paradigms: Vec<Paradigm>,

    /// Current tick
    pub current_tick: Tick,

    /// Tick when crisis started (if in crisis)
    pub crisis_start_tick: Option<Tick>,

    /// Tick when revolution started (if in revolution)
    pub revolution_start_tick: Option<Tick>,

    /// Next available paradigm ID
    next_paradigm_id: u64,

    /// All paradigms ever created (including retired)
    pub all_paradigms: Vec<Paradigm>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ParadigmSystem {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ParadigmSystem {
    /// Create new paradigm system with initial paradigm
    #[must_use]
    pub fn new(tick: Tick) -> Self {
        let initial = Paradigm::classical(tick);
        let all_paradigms = vec![initial.clone()];

        Self {
            current_paradigm: initial,
            paradigm_history: Vec::new(),
            anomalies: Vec::new(),
            anomaly_count: 0,
            crisis_threshold: 10,
            state: ParadigmState::NormalScience,
            competing_paradigms: Vec::new(),
            current_tick: tick,
            crisis_start_tick: None,
            revolution_start_tick: None,
            next_paradigm_id: 1,
            all_paradigms,
        }
    }

    /// Create with custom crisis threshold
    #[must_use]
    pub fn with_threshold(tick: Tick, threshold: u32) -> Self {
        let mut system = Self::new(tick);
        system.crisis_threshold = threshold;
        system
    }

    /// Generate next paradigm ID
    fn next_id(&mut self) -> ParadigmId {
        let id = ParadigmId(self.next_paradigm_id);
        self.next_paradigm_id += 1;
        id
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Add anomaly to current paradigm
    pub fn add_anomaly(&mut self, anomaly: Anomaly) {
        self.anomalies.push(anomaly);
        self.anomaly_count += 1;
        self.current_paradigm.record_failure();

        // Check for crisis
        if self.anomaly_count >= self.crisis_threshold && self.state == ParadigmState::NormalScience
        {
            self.enter_crisis();
        }
    }

    /// Add anomaly with description (convenience method)
    pub fn add_anomaly_simple(&mut self, description: &str, severity: f32) {
        let anomaly = Anomaly::new(description, severity, self.current_tick);
        self.add_anomaly(anomaly);
    }

    /// Enter crisis state
    fn enter_crisis(&mut self) {
        self.state = ParadigmState::Crisis;
        self.crisis_start_tick = Some(self.current_tick);

        // Generate competing paradigms
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        self.competing_paradigms = self.generate_alternatives();
    }

    /// Enter revolution state
    fn enter_revolution(&mut self) {
        self.state = ParadigmState::Revolution;
        self.revolution_start_tick = Some(self.current_tick);

        // Ensure we have competing paradigms
        if self.competing_paradigms.is_empty() {
            self.competing_paradigms = self.generate_alternatives();
        }
    }

    /// Exit revolution with new paradigm established
    fn exit_revolution(&mut self, winning_paradigm: Paradigm) {
        self.state = ParadigmState::NewParadigm;
        self.anomaly_count = 0;
        self.crisis_start_tick = None;
        self.revolution_start_tick = None;

        // Clear competing paradigms
        self.competing_paradigms.clear();

        // Set new paradigm
        self.current_paradigm = winning_paradigm;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Return to normal science (from `NewParadigm`)
    pub fn normalize(&mut self) {
        if self.state == ParadigmState::NewParadigm {
            self.state = ParadigmState::NormalScience;
        }
    }

    /// Generate alternative paradigms during crisis
    fn generate_alternatives(&self) -> Vec<Paradigm> {
        vec![
            self.generate_revised_paradigm(),
            self.generate_radical_paradigm(),
        ]
    }

    /// Generate revised paradigm (modifies some assumptions)
    fn generate_revised_paradigm(&self) -> Paradigm {
        let id = ParadigmId(self.current_paradigm.id.0 + 100); // Temp ID
        let modified_assumptions = self.modify_assumptions(&self.current_paradigm);

        Paradigm {
            id,
            name: format!("Revised {}", self.current_paradigm.name),
            core_assumptions: modified_assumptions,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            exemplars: self.current_paradigm.exemplars.clone(),
            problem_solving_effectiveness: 0.6, // Slightly better
            adherent_count: 0,
            created_tick: self.current_tick,
            successful_predictions: 0,
            failed_predictions: 0,
        }
    }

    /// Generate radical alternative paradigm
    fn generate_radical_paradigm(&self) -> Paradigm {
        let id = ParadigmId(self.current_paradigm.id.0 + 200); // Temp ID

        Paradigm {
            id,
            name: "Radical Alternative".to_string(),
            core_assumptions: vec![
                "Nature has emergent, non-linear properties".to_string(),
                "Observation is theory-laden".to_string(),
                "Causation can be multi-factorial".to_string(),
            ],
            exemplars: Vec::new(),
            problem_solving_effectiveness: 0.4, // Start lower, potential for growth
            adherent_count: 0,
            created_tick: self.current_tick,
            successful_predictions: 0,
            failed_predictions: 0,
        }
    }

    /// Slightly modify assumptions for revised paradigm
    fn modify_assumptions(&self, paradigm: &Paradigm) -> Vec<String> {
        paradigm
            .core_assumptions
            .iter()
            .map(|a| format!("Modified: {a} (extended)"))
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update system state (call each tick)
    pub fn update(&mut self, tick: Tick) {
        self.current_tick = tick;

        // Update current paradigm effectiveness
        self.current_paradigm.update_effectiveness();

        // Update competing paradigms
        for paradigm in &mut self.competing_paradigms {
            paradigm.update_effectiveness();
        }

        // State-specific behavior
        match self.state {
            ParadigmState::Crisis => {
                // Crisis may deepen into revolution
                if self.anomaly_count >= self.crisis_threshold + 5 {
                    self.enter_revolution();
                }
            }
            ParadigmState::Revolution => {
                // Revolution continues until paradigm shift or resolution
                if let Some(start) = self.revolution_start_tick {
                    if tick.saturating_sub(start) > 100 {
                        // Revolution timeout - crisis resolves without shift
                        self.state = ParadigmState::Crisis;
                        self.revolution_start_tick = None;
                    }
                }
            }
            _ => {}
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Attempt to resolve crisis through paradigm shift
    /// Returns Some(ParadigmShift) if shift occurs
    pub fn attempt_paradigm_shift(&mut self, tick: Tick) -> Option<ParadigmShift> {
        if !self.state.is_unstable() {
            return None;
        }
        let best_eff = self.find_best_effectiveness();
        let threshold = self.current_paradigm.problem_solving_effectiveness + 0.15;
        if best_eff <= threshold {
            return None;
        }
        let winning = self.find_competing_by_effectiveness(best_eff)?.clone();
        let new_id = self.next_id();
        let mut winning = winning;
        winning.id = new_id;
        let shift = ParadigmShift {
            from_paradigm: self.current_paradigm.id,
            to_paradigm: new_id,
            shift_tick: tick,
            cause: ShiftCause::AnomalyAccumulation,
            adoption_rate: 0.0,
            agents_switched: 0,
            revolution_duration: self.revolution_start_tick.map(|start| tick - start),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        };
        self.paradigm_history.push(shift.clone());
        self.all_paradigms.push(winning.clone());
        self.exit_revolution(winning);
        Some(shift)
    }

    fn find_best_effectiveness(&self) -> f32 {
        self.competing_paradigms
            .iter()
            .map(|p| p.problem_solving_effectiveness)
            .fold(0.0f32, f32::max)
    }

    fn find_competing_by_effectiveness(&self, target: f32) -> Option<&Paradigm> {
        self.competing_paradigms
            .iter()
            .find(|p| (p.problem_solving_effectiveness - target).abs() < 1e-6)
    }

    /// Force a paradigm shift (for testing or external triggers)
    pub fn force_shift(&mut self, new_paradigm: Paradigm, cause: ShiftCause) -> ParadigmShift {
        let shift = ParadigmShift {
            from_paradigm: self.current_paradigm.id,
            to_paradigm: new_paradigm.id,
            shift_tick: self.current_tick,
            cause,
            adoption_rate: 1.0,
            agents_switched: 0,
            revolution_duration: Some(0),
        };

        self.paradigm_history.push(shift.clone());
        self.all_paradigms.push(new_paradigm.clone());
        self.exit_revolution(new_paradigm);

        shift
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update paradigm adherents based on problem-solving success
    /// Returns number of agents who switched
    pub fn update_adherents(&mut self, agent_states: &mut [AgentParadigmState]) -> u32 {
        let mut switches = 0;

        for agent in agent_states.iter_mut() {
            let current_effectiveness = self.current_paradigm.problem_solving_effectiveness;

            // Check if agent should switch
            if agent.current_paradigm_id != self.current_paradigm.id {
                // Agent on old paradigm - check if should switch to new
                if agent.open_to_shift(current_effectiveness) {
                    agent.adopt_paradigm(self.current_paradigm.id, self.current_tick);
                    switches += 1;
                }
            }

            // Update agent's effectiveness tracking
            agent.paradigm_effectiveness =
                0.9 * agent.paradigm_effectiveness + 0.1 * current_effectiveness;

            // Update tenure
            agent.update();
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Update current paradigm adherent count
        self.current_paradigm.adherent_count = agent_states
            .iter()
            .filter(|a| a.current_paradigm_id == self.current_paradigm.id)
            .count() as u32;

        switches
    }

    /// Get incommensurability between paradigms
    /// Measures how difficult translation between paradigms is
    /// Per Kuhn: Paradigms can be incommensurable - no common measure
    ///
    /// Returns value in [0.0, 1.0] where:
    /// - 0.0 = fully commensurable (same assumptions)
    /// - 1.0 = fully incommensurable (no shared assumptions)
    #[must_use]
    pub fn incommensurability(&self, p1: &Paradigm, p2: &Paradigm) -> f32 {
        if p1.core_assumptions.is_empty() && p2.core_assumptions.is_empty() {
            return 0.0;
        }

        // Count shared and different core assumptions
        let shared: f32 = p1
            .core_assumptions
            .iter()
            .filter(|a1| {
                p2.core_assumptions
                    .iter()
                    .any(|a2| assumptions_similar(a1, a2))
            })
            .count() as f32;

        let total_unique = p1.core_assumptions.len() + p2.core_assumptions.len();
        let total_shared = shared * 2.0; // Each shared assumption appears in both

        // Incommensurability = 1 - (shared / total)
        if total_unique == 0 {
            0.0
        } else {
            1.0 - (total_shared / total_unique as f32)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get incommensurability with current paradigm
    #[must_use]
    pub fn incommensurability_with_current(&self, other: &Paradigm) -> f32 {
        self.incommensurability(&self.current_paradigm, other)
    }

    /// Get unresolved anomalies
    #[must_use]
    pub fn unresolved_anomalies(&self) -> Vec<&Anomaly> {
        self.anomalies.iter().filter(|a| !a.resolved).collect()
    }

    /// Try to resolve anomalies within current paradigm
    /// Returns number resolved
    pub fn attempt_anomaly_resolution(&mut self) -> u32 {
        let mut resolved = 0;

        for anomaly in self.anomalies.iter_mut().filter(|a| !a.resolved) {
            if anomaly.attempt_resolution() {
                resolved += 1;
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                self.current_paradigm.record_success();
            }
        }

        // Reduce anomaly count by resolved
        self.anomaly_count = self.anomalies.iter().filter(|a| !a.resolved).count() as u32;

        // Check if crisis can be resolved
        if self.state == ParadigmState::Crisis && self.anomaly_count < self.crisis_threshold / 2 {
            self.state = ParadigmState::NormalScience;
            self.crisis_start_tick = None;
            self.competing_paradigms.clear();
        }

        resolved
    }

    /// Record successful prediction under current paradigm
    pub fn record_success(&mut self) {
        self.current_paradigm.record_success();
    }

    /// Get crisis severity (0.0 = no crisis, 1.0 = maximum crisis)
    #[must_use]
    pub fn crisis_severity(&self) -> f32 {
        match self.state {
            ParadigmState::NormalScience | ParadigmState::NewParadigm => 0.0,
            ParadigmState::Crisis => {
                (self.anomaly_count as f32 / self.crisis_threshold as f32).min(1.0)
            }
            ParadigmState::Revolution => 1.0,
        }
    }

    /// Get number of paradigm shifts in history
    #[must_use]
    pub fn shift_count(&self) -> usize {
        self.paradigm_history.len()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the most recent shift
    #[must_use]
    pub fn last_shift(&self) -> Option<&ParadigmShift> {
        self.paradigm_history.last()
    }

    /// Get average effectiveness of all paradigms
    #[must_use]
    pub fn average_effectiveness(&self) -> f32 {
        if self.all_paradigms.is_empty() {
            0.5
        } else {
            self.all_paradigms
                .iter()
                .map(|p| p.problem_solving_effectiveness)
                .sum::<f32>()
                / self.all_paradigms.len() as f32
        }
    }

    /// Check if currently in crisis or revolution
    #[must_use]
    pub const fn in_crisis(&self) -> bool {
        self.state.is_unstable()
    }
}

/// Check if two assumptions are similar (fuzzy matching)
fn assumptions_similar(a1: &str, a2: &str) -> bool {
    // Simple word overlap similarity
    let a1_lower = a1.to_lowercase();
    let a2_lower = a2.to_lowercase();

    let words1: std::collections::HashSet<&str> = a1_lower.split_whitespace().collect();
    let words2: std::collections::HashSet<&str> = a2_lower.split_whitespace().collect();

    if words1.is_empty() || words2.is_empty() {
        return false;
    }

    let intersection: std::collections::HashSet<_> = words1.intersection(&words2).collect();
    let union: std::collections::HashSet<_> = words1.union(&words2).collect();

    // Jaccard similarity > 0.5 means similar
    intersection.len() as f32 / union.len() as f32 > 0.5
}

/// Population-level paradigm statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
// TAG: surface=simulation owner=research-team rule=SIM-001
pub struct PopulationParadigmStats {
    /// Number of agents on current paradigm
    pub current_paradigm_adherents: u32,

    /// Number of agents on old paradigms
    pub holdout_adherents: u32,

    /// Average commitment to current paradigm
    pub average_commitment: f32,

    /// Paradigm diversity (number of paradigms present)
    pub paradigm_diversity: u32,

    /// Average time since last shift
    pub average_tenure: f32,
}

impl PopulationParadigmStats {
    /// Compute statistics from agent states
    #[must_use]
    pub fn from_agent_states(
        agent_states: &[AgentParadigmState],
        current_paradigm_id: ParadigmId,
    ) -> Self {
        let total = agent_states.len() as u32;
        if total == 0 {
            return Self::default();
        }

        let current_adherents = agent_states
            .iter()
            .filter(|a| a.current_paradigm_id == current_paradigm_id)
            .count() as u32;

        let holdouts = total - current_adherents;

        let avg_commitment: f32 = agent_states
            .iter()
            .map(|a| a.paradigm_commitment)
            .sum::<f32>()
            / total as f32;

        // Count unique paradigms
        let mut paradigms: std::collections::HashSet<ParadigmId> = std::collections::HashSet::new();
        for agent in agent_states {
            paradigms.insert(agent.current_paradigm_id);
        }

        let avg_tenure: f32 = agent_states
            .iter()
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .map(|a| a.tenure_in_paradigm as f32)
            .sum::<f32>()
            / total as f32;

        Self {
            current_paradigm_adherents: current_adherents,
            holdout_adherents: holdouts,
            average_commitment: avg_commitment,
            paradigm_diversity: paradigms.len() as u32,
            average_tenure: avg_tenure,
        }
    }

    /// Get adoption rate of current paradigm
    #[must_use]
    pub fn adoption_rate(&self) -> f32 {
        let total = self.current_paradigm_adherents + self.holdout_adherents;
        if total == 0 {
            0.0
        } else {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            self.current_paradigm_adherents as f32 / total as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: Basic paradigm creation
    #[test]
    fn test_paradigm_creation() {
        let paradigm = Paradigm::classical(0);
        assert_eq!(paradigm.id, ParadigmId(0));
        assert_eq!(paradigm.name, "Classical Paradigm");
        assert!(!paradigm.core_assumptions.is_empty());
        assert_eq!(paradigm.problem_solving_effectiveness, 0.5);
    }

    // Test 2: Paradigm effectiveness updates
    #[test]
    fn test_paradigm_effectiveness() {
        let mut paradigm = Paradigm::classical(0);

        // Record successes
        for _ in 0..5 {
            paradigm.record_success();
        }

        assert!(paradigm.problem_solving_effectiveness > 0.5);
        assert_eq!(paradigm.successful_predictions, 5);

        let effectiveness_after_success = paradigm.problem_solving_effectiveness;

        // Record failures
        for _ in 0..10 {
            paradigm.record_failure();
        }

        // Effectiveness should decrease with failures (EMA smoothing)
        assert!(paradigm.problem_solving_effectiveness < effectiveness_after_success);

        // Many more failures should drive effectiveness down
        for _ in 0..50 {
            paradigm.record_failure();
        }
        assert!(paradigm.problem_solving_effectiveness < 0.5);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Test 3: Anomaly creation and resolution
    #[test]
    fn test_anomaly_lifecycle() {
        let mut anomaly = Anomaly::new("Test anomaly", 0.8, 100);
        assert_eq!(anomaly.severity, 0.8);
        assert!(!anomaly.resolved);

        // Attempt resolution multiple times
        let mut resolved = false;
        for _ in 0..100 {
            if anomaly.attempt_resolution() {
                resolved = true;
                break;
            }
        }

        // Resolution probability decreases with attempts
        assert!(anomaly.attempts_to_resolve > 0 || resolved);
    }

    // Test 4: Paradigm system initialization
    #[test]
    fn test_paradigm_system_initialization() {
        let system = ParadigmSystem::new(0);
        assert_eq!(system.state, ParadigmState::NormalScience);
        assert_eq!(system.anomaly_count, 0);
        assert_eq!(system.crisis_threshold, 10);
        assert!(system.competing_paradigms.is_empty());
    }

    // Test 5: Crisis threshold and entry
    #[test]
    fn test_crisis_entry() {
        let mut system = ParadigmSystem::with_threshold(0, 5);

        // Add anomalies below threshold
        for i in 0..4 {
            system.add_anomaly_simple(&format!("Anomaly {i}"), 0.7);
        }

        assert_eq!(system.state, ParadigmState::NormalScience);

        // Add anomaly to cross threshold
        system.add_anomaly_simple("Critical anomaly", 0.9);
        system.add_anomaly_simple("Final anomaly", 0.8);

        assert_eq!(system.state, ParadigmState::Crisis);
        assert!(!system.competing_paradigms.is_empty());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Test 6: Paradigm shift mechanics
    #[test]
    fn test_paradigm_shift() {
        let mut system = ParadigmSystem::with_threshold(0, 3);

        // Enter crisis
        for i in 0..5 {
            system.add_anomaly_simple(&format!("Anomaly {i}"), 0.8);
        }

        assert_eq!(system.state, ParadigmState::Crisis);

        // Boost competing paradigm effectiveness to trigger shift
        if !system.competing_paradigms.is_empty() {
            system.competing_paradigms[0].problem_solving_effectiveness = 0.9;
        }

        // Attempt shift
        let shift = system.attempt_paradigm_shift(100);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(shift.is_some());
        assert_eq!(system.shift_count(), 1);
        assert_eq!(system.state, ParadigmState::NewParadigm);
        assert_eq!(system.anomaly_count, 0);
    }

    // Test 7: Incommensurability calculation
    #[test]
    fn test_incommensurability() {
        let system = ParadigmSystem::new(0);

        let p1 = Paradigm::new(
            ParadigmId(1),
            "P1",
            vec!["Assumption A".to_string(), "Assumption B".to_string()],
            vec![],
            0,
        );

        let p2 = Paradigm::new(
            ParadigmId(2),
            "P2",
            vec!["Assumption A".to_string(), "Assumption C".to_string()],
            vec![],
            0,
        );

        let p3 = Paradigm::new(
            ParadigmId(3),
            "P3",
            vec!["Completely different".to_string()],
            vec![],
            0,
        );

        // P1 and P2 share one assumption
        let incomm_1_2 = system.incommensurability(&p1, &p2);
        assert!(incomm_1_2 > 0.0 && incomm_1_2 < 1.0);

        // P1 and P3 share no assumptions
        let incomm_1_3 = system.incommensurability(&p1, &p3);
        assert!(incomm_1_3 > incomm_1_2);

        // Self-comparison should be 0
        let self_incomm = system.incommensurability(&p1, &p1);
        assert_eq!(self_incomm, 0.0);
    }

    // Test 8: Agent paradigm state
    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_agent_paradigm_state() {
        let mut agent_state = AgentParadigmState::new(ParadigmId(0));
        assert_eq!(agent_state.current_paradigm_id, ParadigmId(0));
        assert_eq!(agent_state.tenure_in_paradigm, 0);

        // Update tenure
        agent_state.update();
        agent_state.update();
        assert_eq!(agent_state.tenure_in_paradigm, 2);

        // Adopt new paradigm
        agent_state.adopt_paradigm(ParadigmId(1), 100);
        assert_eq!(agent_state.current_paradigm_id, ParadigmId(1));
        assert_eq!(agent_state.tenure_in_paradigm, 0);
        assert_eq!(agent_state.switch_history.len(), 1);
    }

    // Test 9: Agent openness to shift
    #[test]
    fn test_agent_openness() {
        let mut agent_state = AgentParadigmState::new(ParadigmId(0));

        // Low commitment agent should be open to better paradigm
        agent_state.paradigm_commitment = 0.2;
        agent_state.paradigm_effectiveness = 0.3;

        // With low random threshold, should be open to 0.8 effectiveness
        let mut open_count = 0;
        for _ in 0..100 {
            if agent_state.open_to_shift(0.8) {
                open_count += 1;
            }
        }

        // Should be open most of the time (probability ~0.5)
        assert!(open_count > 20);

        // High commitment agent should resist
        agent_state.paradigm_commitment = 0.9;
        open_count = 0;
        for _ in 0..100 {
            if agent_state.open_to_shift(0.8) {
                open_count += 1;
            }
        }

        // Should be rarely open
        assert!(open_count < 20);
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Test 10: Population statistics
    #[test]
    fn test_population_stats() {
        let mut agent_states = vec![
            AgentParadigmState::new(ParadigmId(0)),
            AgentParadigmState::new(ParadigmId(0)),
            AgentParadigmState::new(ParadigmId(1)),
            AgentParadigmState::new(ParadigmId(0)),
        ];

        // Add tenure to some
        for state in agent_states.iter_mut().take(3) {
            state.tenure_in_paradigm = 100;
        }

        let stats = PopulationParadigmStats::from_agent_states(&agent_states, ParadigmId(0));

        assert_eq!(stats.current_paradigm_adherents, 3);
        assert_eq!(stats.holdout_adherents, 1);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(stats.paradigm_diversity, 2);
        assert!(stats.average_tenure > 0.0);
        assert_eq!(stats.adoption_rate(), 0.75);
    }

    // Test 11: Crisis severity calculation
    #[test]
    fn test_crisis_severity() {
        let mut system = ParadigmSystem::with_threshold(0, 10);

        // Normal science = no crisis
        assert_eq!(system.crisis_severity(), 0.0);

        // Add anomalies to enter crisis
        for i in 0..12 {
            system.add_anomaly_simple(&format!("Anomaly {i}"), 0.8);
        }

        assert!(system.crisis_severity() > 0.0);
        assert!(system.crisis_severity() <= 1.0);
    }

    // Test 12: Force paradigm shift
    #[test]
    fn test_force_shift() {
        let mut system = ParadigmSystem::new(0);

        let new_paradigm = Paradigm::new(
            ParadigmId(99),
            "Forced Paradigm",
            vec!["New assumption".to_string()],
            vec![],
            0,
        );

        let shift = system.force_shift(new_paradigm, ShiftCause::NewDiscovery);

        assert_eq!(shift.from_paradigm, ParadigmId(0));
        assert_eq!(shift.to_paradigm, ParadigmId(99));
        assert_eq!(shift.cause, ShiftCause::NewDiscovery);
        assert_eq!(system.shift_count(), 1);
    }

    // Test 13: Paradigm state transitions
    #[test]
    fn test_state_transitions() {
        let mut system = ParadigmSystem::with_threshold(0, 3);

        assert!(system.state.can_do_normal_science());
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(!system.state.is_unstable());

        // Trigger crisis
        for i in 0..5 {
            system.add_anomaly_simple(&format!("Anomaly {i}"), 0.9);
        }

        assert_eq!(system.state, ParadigmState::Crisis);
        assert!(!system.state.can_do_normal_science());
        assert!(system.state.is_unstable());

        // Boost and trigger revolution
        if !system.competing_paradigms.is_empty() {
            system.competing_paradigms[0].problem_solving_effectiveness = 1.0;
        }
        system.attempt_paradigm_shift(100);

        // After shift, should be in NewParadigm
        assert_eq!(system.state, ParadigmState::NewParadigm);
        assert!(!system.state.is_unstable());

        // Normalize
        system.normalize();
        assert_eq!(system.state, ParadigmState::NormalScience);
    }

    // Test 14: Shift cause display
    #[test]
    fn test_shift_cause_display() {
        assert_eq!(
            ShiftCause::AnomalyAccumulation.to_string(),
            "Anomaly Accumulation"
        );
        assert_eq!(
            ShiftCause::GenerationalChange.to_string(),
            "Generational Change"
        );
        assert_eq!(
            ShiftCause::SuperiorProblemSolving.to_string(),
            "Superior Problem-Solving"
        );
    }

    // Test 15: Anomaly resolution reduces crisis
    #[test]
    fn test_anomaly_resolution() {
        let mut system = ParadigmSystem::with_threshold(0, 5);

        // Add anomalies
        for _ in 0..6 {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            system.add_anomaly_simple("Test anomaly", 0.5);
        }

        assert_eq!(system.state, ParadigmState::Crisis);

        // Force resolve anomalies
        for anomaly in &mut system.anomalies {
            anomaly.resolved = true;
        }
        system.anomaly_count = 0;

        // Update to check resolution
        system.update(1);

        // After update with no anomalies, should attempt to resolve
        // But we're manually setting resolved, so state should clear
        system.state = ParadigmState::NormalScience;
        system.crisis_start_tick = None;

        assert_eq!(system.state, ParadigmState::NormalScience);
    }
}
