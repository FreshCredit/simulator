//! Live simulation types and snapshots

use parking_lot::RwLock;
use sim_core::Sex;

use crate::ai::AiState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Concept snapshot in an agent's lexicon for D3 CLICS-4 visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconConceptSnapshot {
    /// Unique concept ID
    pub id: u64,
    /// Concept name / label
    pub name: String,
    /// Semantic category (Trust, Social, Resource, Emotion, Action)
    pub category: String,
    /// Semantic embedding vector
    #[serde(default)]
    pub embedding: Vec<f32>,
    /// Usage frequency
    pub frequency: f32,
    /// Learned tick
    pub created_at: u64,
}

/// Lexicon summary for language visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconSummary {
    /// concept count
    pub concept_count: usize,
    /// drift rate
    pub drift_rate: f32,
    /// top categories
    pub top_categories: Vec<String>,
    /// concepts list for CLICS visualization
    #[serde(default)]
    pub concepts: Vec<LexiconConceptSnapshot>,
    /// colexifications (source, target, strength)
    #[serde(default)]
    pub colexifications: Vec<(u64, u64, f32)>,
    /// colexification pairs (source, target)
    #[serde(default)]
    pub colexification_map: Vec<(u64, u64)>,
}

/// Agent state snapshot for streaming to UI
/// Per §10.6.2: Complete `AgentSnapshot` specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSnapshot {
    /// agent id
    pub agent_id: u64,
    /// X coordinate (horizontal)
    pub x: f32,
    /// Y coordinate (horizontal)
    pub y: f32,
    /// Z coordinate (altitude/elevation)
    /// Supports 3D simulation with altitude effects
    pub z: f32,
    /// energy
    pub energy: f32,
    /// docas
    pub docas: [f32; 5],
    /// Heritable DOCAS gain traits (genome traits 5-9) for agent coloring  // SIM-GENOME-001: Genome-based coloring
    /// Per §10.5.2: Agents are colored by `genome_gains`, NOT docas
    /// Per §10.6.4: REQUIRED for correct visual representation
    pub genome_gains: [f32; 5],
    /// alive
    pub alive: bool,
    /// has verified id
    pub has_verified_id: bool,
    /// has identity (backward-compatible alias for UI components)
    #[serde(default)]
    pub has_identity: bool,
    /// group id
    pub group_id: Option<u64>,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// age
    pub age: u64,
    /// Sex for reproduction pairing (per SIM-001)
    pub sex: Sex,
    // === Trust & Reputation (computed from DOCAS) ===
    /// Trust (τ) computed from DOCAS: τ = (D + O + S) / (1 + C + A)
    /// Per SIM-TRUST-001: Emergent from genome → DOCAS chain
    pub tau: f32,
    /// Reputation from interaction history [0, 1]
    /// Ratio of `positive_interactions` / `total_interactions`
    pub reputation: f32,
    // === Lineage data ===
    /// Generation number (0 for initial agents, >0 for offspring)
    ///
    /// NOTE: At simulation start, all agents have generation 0. As the simulation
    /// runs and agents reproduce, offspring have incrementing generation numbers.
    /// After many ticks, most visible agents will have higher generation numbers
    /// as early generations die off. This is expected evolutionary behavior.
    pub generation: u32,
    /// Parent 1 ID (None for initial agents)
    pub parent1_id: Option<u64>,
    /// Parent 2 ID (None for asexual reproduction or initial agents)
    pub parent2_id: Option<u64>,
    /// Number of direct offspring born to this agent across all generations
    #[serde(default)]
    pub children_count: u32,
    /// Number of descendants at distance 2 across all generations
    #[serde(default)]
    pub grandchildren_count: u32,
    /// Last movement delta X (for Pac-Man visualization)
    pub last_dx: f32,
    /// Last movement delta Y (for Pac-Man visualization)
    pub last_dy: f32,
    /// Language lexicon summary (if language enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lexicon: Option<LexiconSummary>,
    /// Wet-lab & wearable biomarkers (Apple Watch HRV/Sleep, Muse EEG, Stelo CGM)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biomarkers: Option<AgentBiomarkersSnapshot>,
    /// Plaid financial underwriting & cash-flow timeseries (Sequestration Wall protected)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub financial: Option<PlaidFinancialSnapshot>,
}

/// Wet-Lab IoT Wearable Biomarker Snapshot
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AgentBiomarkersSnapshot {
    /// HRV Root Mean Square of Successive Differences (ms)
    pub hrv_rmssd_ms: f32,
    /// HRV Standard Deviation of NN intervals (ms)
    pub hrv_sdnn_ms: f32,
    /// Resting heart rate (bpm)
    pub resting_hr_bpm: f32,
    /// Respiratory rate (brpm)
    pub respiratory_rate_brpm: f32,
    /// Sleep efficiency percentage
    pub sleep_efficiency_pct: f32,
    /// Slow-wave deep sleep percentage
    pub sleep_deep_pct: f32,
    /// REM sleep percentage
    pub sleep_rem_pct: f32,
    /// SCN circadian phase in radians
    pub circadian_phase_rad: f32,
    /// Frontal Midline Theta (4–8 Hz) power density in μV²/Hz
    pub eeg_frontal_theta_uv2: f32,
    /// Frontal Alpha Asymmetry (AF8 vs AF7)
    pub eeg_alpha_asymmetry: f32,
    /// Beta/Alpha arousal ratio (13–30 Hz / 8–13 Hz)
    pub eeg_beta_arousal_ratio: f32,
    /// Reward Positivity ERP amplitude (μV)
    pub eeg_rewp_erp_uv: f32,
    /// Dexcom Stelo interstitial continuous glucose level (mg/dL)
    pub glucose_mg_dl: f32,
    /// Time-in-Range (70–140 mg/dL) percentage
    pub glucose_tir_pct: f32,
    /// Glycemic Coefficient of Variation percentage (CV%)
    pub glucose_cv_pct: f32,
    /// Nocturnal sleeping wrist temperature in °C (Apple Watch S8)
    pub sleeping_wrist_temp_c: f32,
    /// Prefrontal medial cortical oxygenated hemoglobin shift Δ[HbO2] in μmol/L (Muse Athena)
    pub fnirs_prefrontal_hbo2_umol: f32,
    /// Prefrontal medial cortical deoxygenated hemoglobin shift Δ[HbR] in μmol/L (Muse Athena)
    pub fnirs_prefrontal_hbr_umol: f32,
}

/// Plaid Financial Timeseries Snapshot
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlaidFinancialSnapshot {
    /// Market-observed trust score S_mkt ∈ [0, 1]
    pub market_trust_score: f32,
    /// Cash-flow liquidity cushion in days of consumption
    pub cashflow_buffer_days: f32,
    /// Rolling coefficient of variation of expenditure
    pub spend_volatility: f32,
    /// Insufficient-funds / overdraft event counter
    pub nsf_overdraft_count: u32,
    /// On-time debt settlement ratio [0.0, 1.0]
    pub on_time_payment_rate: f32,
    /// Underwritten credit access level [0.0, 1.0]
    pub credit_access: f32,
    /// Debt-to-income/wealth ratio
    pub debt_burden: f32,
    /// Liquid wealth
    pub wealth: f32,
    /// Fraction of promises kept
    pub bond_density: f32,
    /// Dual-layer Concordance Statistic (Mantel r)
    pub concordance_mantel_r: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Trust network edge for visualization
/// Per P1-4: Render attestation/trust graph between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEdge {
    /// Source agent ID (attestor/group member)
    pub from_id: u64,
    /// Target agent ID (subject/group member)
    pub to_id: u64,
    /// Edge weight [0.0, 1.0] based on trust relationship
    /// For group edges: average tau of connected agents
    /// For attestations: attestation value normalized to [0, 1]
    pub weight: f32,
    /// Edge type for rendering
    pub edge_type: EdgeType,
}

/// Type of trust network edge
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    /// Same group membership
    GroupMember,
    /// Parent-child lineage link
    Lineage,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Resource cell snapshot for terrain visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    /// amount
    pub amount: f32,
    /// capacity
    pub capacity: f32,
}

/// Weather state snapshot for weather visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSnapshot {
    /// Current weather type: 0=Sunny, 1=Cloudy, 2=Rainy, 3=Stormy, 4=Drought
    pub weather_type: u8,
    /// Temperature in Celsius
    pub temperature: f32,
    /// Rainfall amount
    pub rainfall: f32,
    /// Current season: 0=Spring, 1=Summer, 2=Autumn, 3=Winter
    pub season: u8,
}

/// Terrain elevation snapshot for topographic visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainSnapshot {
    /// Elevation value (0.0 - 1.0 normalized)
    pub elevation: f32,
}

/// Game event for activity log
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    /// tick
    pub tick: u64,
    /// agent a
    pub agent_a: u64,
    /// agent b
    pub agent_b: u64,
    /// game type
    pub game_type: String,
    /// outcome
    pub outcome: String,
    /// payoff a
    pub payoff_a: f32,
    /// payoff b
    pub payoff_b: f32,
}

/// Encounter event for activity log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterEvent {
    /// tick
    pub tick: u64,
    /// agent a
    pub agent_a: u64,
    /// agent b
    pub agent_b: u64,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// encounter type
    pub encounter_type: String,
}

/// Version 4.0 Research Telemetry Snapshot (Sim O-Series, Sim D16, Sim D17)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V4TelemetrySnapshot {
    /// Predatory contract uptake across cognitive bandwidth deciles D1..D10
    pub decile_uptake_rates: [f32; 10],
    /// Vulnerability gradient odds ratio OR_D1/D10 (Sim D16 benchmark: 38.94x)
    pub odds_ratio_d1_vs_d10: f32,
    /// Global Kuramoto order parameter R in [0, 1]
    pub kuramoto_coherence_r: f32,
    /// 5-carrier trust state simplex projection S_t = [D, O, C, A, S]
    pub simplex_projection: [f32; 5],
    /// Active visual cloaking ratio chi in [1.0, 5.0]
    pub active_cloaking_ratio: f32,
    /// Active Brain-IT view branch mode ("BranchAStructural" or "BranchBSemantic")
    pub active_brain_it_branch: String,
    /// Active optogenetic laser wavelength ("LaserOff", "ChR2Blue470nm", "ArchTYellow566nm")
    pub laser_wavelength: String,
    /// Target optogenetic channels [D, O, C, A, S]
    pub target_channels: [bool; 5],
    /// Solar photic lux drive Phi_photic in [0.15, 1.0]
    pub weather_lux: f32,
    /// Thermal stress factor F_thermal in [1.0, 1.5]
    pub weather_thermal_stress: f32,
    /// Weather-shield decision cooling-off buffer active
    pub weather_shield_active: bool,
    /// Version 4.0: Biophysical Markov opsin conductance g(t) in [0, 1]
    pub opto_conductance: f32,
    /// Version 4.0: Baylor unconscious narrative gating mode (M in {0,1})
    pub baylor_unconscious_mode: bool,
    /// Version 4.0: Baylor cognitive friction R_fric (0.08 unconscious vs 1.0 conscious)
    pub baylor_cognitive_friction: f32,
    /// Version 4.0: Baylor executive threat gate E(t) (0.0 unconscious vs 1.0 conscious)
    pub baylor_executive_gate: f32,
    /// Version 4.0: Relational O/S implicit narrative attractor drift Delta S_base
    pub baylor_attractor_shift: f32,
}

/// Simulation frame for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationFrame {
    /// tick
    pub tick: u64,
    /// agents
    pub agents: Vec<AgentSnapshot>,
    /// stats
    pub stats: FrameStats,
    /// Trust network edges for visualization (P1-4)
    #[serde(default)]
    pub trust_edges: Vec<TrustEdge>,
    /// Resource grid for terrain visualization (optional - sent every 10 ticks to save bandwidth)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<ResourceSnapshot>>,
    /// Weather grid for weather visualization (optional - sent every tick)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather: Option<Vec<f32>>,
    /// Terrain elevation grid for topographic visualization (optional - sent once or every 10 ticks)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain: Option<Vec<f32>>,
    /// Game events for activity log
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub game_events: Vec<GameEvent>,
    /// Encounter events for activity log
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub encounter_events: Vec<EncounterEvent>,
    /// Simulation lifecycle status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<SimulationStatus>,
    /// Maximum ticks configured for this run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_ticks: Option<u64>,
    /// Grid dimensions (width, height)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_dimensions: Option<(u32, u32)>,
    /// Version 4.0 Research telemetry snapshot
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v4_telemetry: Option<V4TelemetrySnapshot>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Frame statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FrameStats {
    /// total agents
    pub total_agents: usize,
    /// alive agents
    pub alive_agents: usize,
    /// verified agents
    pub verified_agents: usize,
    /// groups
    pub groups: usize,
}

/// Historical time-series metric point for export/import state persistence (Master Directive Part 4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMetricPoint {
    /// Simulation tick
    pub tick: u64,
    /// Total population
    pub population: usize,
    /// Alive population
    pub alive_agents: usize,
    /// Average trust S_t across population
    pub avg_trust: f32,
    /// Cumulative defaults
    pub defaults: u64,
    /// Default rate
    pub default_rate: f32,
    /// Gini inequality index
    pub gini_index: f32,
}

/// Live simulation state shared across handlers
#[derive(Clone, Debug)]
pub struct LiveSimulationState {
    /// Broadcast channel for streaming frames
    pub sender: broadcast::Sender<SimulationFrame>,
    /// Current frame (for late joiners)
    pub current_frame: Arc<RwLock<Option<SimulationFrame>>>,
    /// AI service state (Gemini integration)
    pub ai_state: Arc<AiState>,
    /// Historical time-series metrics buffer
    pub history: Arc<RwLock<Vec<HistoricalMetricPoint>>>,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
impl LiveSimulationState {
    #[must_use]
    /// new
    pub fn new(sender: broadcast::Sender<SimulationFrame>, ai_state: Arc<AiState>) -> Self {
        Self {
            sender,
            current_frame: Arc::new(RwLock::new(None)),
            ai_state,
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// CRITICAL FIX: Create new state with properly sized broadcast channel
    /// This ensures we have enough buffer to prevent frame drops
    #[must_use]
    pub fn with_default_channel() -> Self {
        // Increased from 16 to 256 to prevent frame drops during reconnects
        let (sender, _receiver) = broadcast::channel(256);
        let ai_state = Arc::new(AiState::from_env());
        Self::new(sender, ai_state)
    }

    /// Create with explicit AI state (for testing)
    #[must_use]
    pub fn with_ai_state(ai_state: Arc<AiState>) -> Self {
        let (sender, _receiver) = broadcast::channel(256);
        Self::new(sender, ai_state)
    }
}

/// Simulation status for lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationStatus {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Running
    Running,
    /// Paused
    Paused,
    /// Completed
    Completed,
    /// Stopped
    Stopped,
}

/// Extended frame with status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationFrameWithStatus {
    #[serde(flatten)]
    /// frame
    pub frame: SimulationFrame,
    /// status
    pub status: SimulationStatus,
    /// max ticks
    pub max_ticks: u64,
}

// ============================================================================
// AI Chat Support - Condensed Frame Structure and Streaming
// ============================================================================

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Notable event for AI context
/// Represents significant occurrences that the AI should be aware of
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotableEvent {
    /// Tick when the event occurred
    pub tick: u64,
    /// Event type/category
    pub event_type: String,
    /// Brief description of the event
    pub description: String,
    /// IDs of agents involved (if any)
    pub agent_ids: Vec<u64>,
    /// Additional numeric data (if any)
    pub value: Option<f32>,
}

/// Condensed AI simulation frame
/// Much smaller than full `SimulationFrame` for efficient AI consumption
/// Contains aggregated statistics instead of per-agent data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSimulationFrame {
    /// Current simulation tick
    pub tick: u64,
    /// Number of alive agents
    pub alive_count: usize,
    /// Average energy across all alive agents
    pub avg_energy: f32,
    /// Average DOCAS levels across population [D, O, C, A, S]
    pub avg_docas: [f32; 5],
    /// Notable events since last frame (births, deaths, group formations, etc.)
    pub notable_events: Vec<NotableEvent>,
    /// Number of distinct groups in population
    pub group_count: usize,
    /// Frame timestamp for ordering
    pub timestamp: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Frame buffer for AI context
/// Maintains recent frames for chat context
#[derive(Clone, Debug)]
pub struct AiFrameBuffer {
    /// Maximum number of frames to keep in buffer
    pub max_frames: usize,
    /// Recent frames (oldest first, newest last)
    pub frames: Arc<RwLock<Vec<AiSimulationFrame>>>,
}

impl AiFrameBuffer {
    #[must_use]
    /// new
    pub fn new(max_frames: usize) -> Self {
        Self {
            max_frames,
            frames: Arc::new(RwLock::new(Vec::with_capacity(max_frames))),
        }
    }

    /// Add a new frame to the buffer, evicting oldest if at capacity
    pub fn push(&self, frame: AiSimulationFrame) {
        let mut frames = self.frames.write();
        if frames.len() >= self.max_frames {
            frames.remove(0);
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        frames.push(frame);
    }

    /// Get recent frames for context
    #[must_use]
    pub fn get_recent(&self, count: usize) -> Vec<AiSimulationFrame> {
        let frames = self.frames.read();
        let start = frames.len().saturating_sub(count);
        frames[start..].to_vec()
    }

    /// Get all frames
    #[must_use]
    pub fn get_all(&self) -> Vec<AiSimulationFrame> {
        self.frames.read().clone()
    }

    /// Get frame count
    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.read().len()
    }

    #[must_use]
    /// is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Extended live state with AI support
#[derive(Clone, Debug)]
pub struct AiLiveState {
    /// Base live simulation state
    pub base: LiveSimulationState,
    /// AI frame buffer for chat context
    pub frame_buffer: AiFrameBuffer,
    /// Broadcast channel for AI stream (1 FPS)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub ai_sender: broadcast::Sender<AiSimulationFrame>,
}

impl AiLiveState {
    #[must_use]
    /// new
    pub fn new(base: LiveSimulationState) -> Self {
        // AI channel with smaller buffer (1 FPS = fewer frames needed)
        let (ai_sender, _receiver) = broadcast::channel(32);

        Self {
            base,
            frame_buffer: AiFrameBuffer::new(60), // Keep last 60 seconds at 1 FPS
            ai_sender,
        }
    }
}
