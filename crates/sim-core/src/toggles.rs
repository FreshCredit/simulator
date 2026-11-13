//! Toggles module: 10 mechanism toggle contracts and continuous parameter dials
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION toggles for factorial experiments
//!
//! Per §8.2.1 lines 4419-4679: Toggle Mechanism Contracts
//! Per §0.4 line 54: 10 toggleable mechanisms for factorial experiments
//! Per simulation-plan.md §6.16: Music layer added as geometric trust indicator

use serde::{Deserialize, Serialize};

const fn default_true() -> bool {
    true
}

const fn default_groups_enabled() -> bool {
    true
}

const fn default_kingman_coalescent() -> f32 {
    1.62
}

const fn default_lineage_depth() -> usize {
    10
}

const fn default_music_metric_hierarchy_tau() -> f32 {
    0.50
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for all 10 toggleable mechanisms
/// Per §8.2.1 lines 4419-4679
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ToggleConfig {
    /// Genome toggle: Mendelian inheritance vs fixed uniform traits
    /// Per §8.2.1 lines 4423-4446
    pub genome_enabled: bool,

    /// DOCAS toggle: Dynamic neuromodulator state vs fixed neutral [0.5; 5]
    /// Per §8.2.1 lines 4449-4471
    pub docas_enabled: bool,

    /// DMN toggle: K-rollout planning vs random action
    /// Per §8.2.1 lines 4474-4498
    pub dmn_enabled: bool,

    /// Criticality toggle: σ≈1 regulation vs unconstrained
    /// Per §8.2.1 lines 4500-4524
    pub criticality_enabled: bool,

    /// Choice Architecture toggle: Thaler nudges vs uniform distribution
    /// Per §8.2.1 lines 4526-4558
    pub choice_architecture_enabled: bool,

    /// Institutions toggle: Beinhocker selection-amplification vs no memory
    /// Per §8.2.1 lines 4561-4593
    pub institutions_enabled: bool,

    /// Norms toggle: Emerged rules + punishment vs no enforcement
    /// Per §8.2.1 lines 4596-4617
    pub norms_enabled: bool,

    /// Language toggle: Lexicons with colexification vs no semantic communication
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Per §8.2.1 lines 4620-4643
    pub language_enabled: bool,

    /// Protocol toggle: Blockchain verification vs no verification
    /// Per §8.2.1 lines 4646-4666
    /// This is the CORE research intervention toggle
    pub protocol_enabled: bool,

    /// Music toggle: Music as universal trust indicator vs no music layer
    /// Per simulation-plan.md §6.16: Music-specific POM targets
    pub music_enabled: bool,

    /// Economics toggle: Credit-innovation feedback loop vs no economic layer
    /// Per economics.md: Economic state and credit access mechanics
    pub economics_enabled: bool,

    /// Encounters toggle: Two-agent interaction pipeline vs no agent-agent interactions
    /// Per SIM-003: Enable/disable the encounter pipeline
    pub encounters_enabled: bool,

    /// Stigmergy toggle: Virtual pheromones for indirect coordination vs no chemical signaling
    /// Per SIM-STIG-001: Enable/disable stigmergy mechanism
    pub stigmergy_enabled: bool,

    /// Barnes-Hut toggle: O(n log n) approximation for long-range social forces vs O(n²) pairwise
    /// Barnes-Hut reduces complexity for large populations by approximating distant agent clusters
    pub barnes_hut_enabled: bool,

    /// Groups toggle: Emergence and tracking of multi-agent social groups
    #[serde(default = "default_groups_enabled")]
    pub groups_enabled: bool,

    /// Geography toggle: Spatial movement, elevation, and terrain constraints vs flat homogeneous plane
    #[serde(default = "default_true")]
    pub geography_enabled: bool,

    /// Biology toggle: Metabolic energy consumption, aging, and mortality vs immortal fixed-energy agents
    #[serde(default = "default_true")]
    pub biology_enabled: bool,

    /// Philosophy toggle: Philosophical trust axis (τ_φ ∈ [-1, +1]), ethical frameworks, and worldview drift
    #[serde(default = "default_true")]
    pub philosophy_enabled: bool,

    /// Biomarkers toggle: Multi-modal IoT wet-lab biomarkers (Apple Watch HRV/Sleep, Muse EEG, Stelo glucose)
    #[serde(default = "default_true")]
    pub biomarkers_enabled: bool,

    /// Dial configuration for continuous parameter ranges
    /// Only applied when parent toggles are enabled
    #[serde(default)]
    pub dials: DialConfig,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ToggleConfig {
    /// Default: All toggles OFF for baseline measurement
    /// Per research design: factorial experiments test toggle combinations
    fn default() -> Self {
        Self {
            genome_enabled: false,
            docas_enabled: false,
            dmn_enabled: false,
            criticality_enabled: false,
            choice_architecture_enabled: false,
            institutions_enabled: false,
            norms_enabled: false,
            language_enabled: false,
            protocol_enabled: false,
            music_enabled: false,
            economics_enabled: false,
            encounters_enabled: true,
            stigmergy_enabled: false,
            barnes_hut_enabled: false,
            groups_enabled: true,
            geography_enabled: true,
            biology_enabled: true,
            philosophy_enabled: true,
            biomarkers_enabled: true,
            dials: DialConfig::default(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ToggleConfig {
    /// All toggles ON - full complexity mode
    #[must_use]
    pub fn all_on() -> Self {
        Self {
            genome_enabled: true,
            docas_enabled: true,
            dmn_enabled: true,
            criticality_enabled: true,
            choice_architecture_enabled: true,
            institutions_enabled: true,
            norms_enabled: true,
            language_enabled: true,
            protocol_enabled: true,
            music_enabled: true,
            economics_enabled: true,
            encounters_enabled: true,
            stigmergy_enabled: true,
            barnes_hut_enabled: true,
            groups_enabled: true,
            geography_enabled: true,
            biology_enabled: true,
            philosophy_enabled: true,
            biomarkers_enabled: true,
            dials: DialConfig::default(),
        }
    }

    /// Stage 0 (Geography) - minimal toggles
    /// Per §7.4.0: Only spatial mechanics
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn stage_0() -> Self {
        Self::default()
    }

    /// Stage 1 (Biology) - genome toggle ON
    /// Per §7.4.0 lines 3757-3758
    #[must_use]
    pub fn stage_1() -> Self {
        Self {
            genome_enabled: true,
            ..Self::default()
        }
    }

    /// Stage 2 (Cognition) - genome + DOCAS + DMN + criticality
    /// Per §6.1, §6.2
    #[must_use]
    pub fn stage_2() -> Self {
        Self {
            genome_enabled: true,
            docas_enabled: true,
            dmn_enabled: true,
            criticality_enabled: true,
            ..Self::default()
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Stage 3 (Society) - Stage 2 + institutions + norms + stigmergy
    /// Per §4261-4366: Groups, norms, institutions, cooperation
    /// Per §7.6 Step 7: Cultural norm filter
    /// Per SIM-STIG-001: Stigmergy for indirect coordination
    #[must_use]
    pub fn stage_3() -> Self {
        Self {
            genome_enabled: true,
            docas_enabled: true,
            dmn_enabled: true,
            criticality_enabled: true,
            institutions_enabled: true,
            norms_enabled: true,
            stigmergy_enabled: true,
            ..Self::default()
        }
    }

    /// Stage 4 (Language) - Stage 3 + language + music + stigmergy + `barnes_hut`
    /// Per §4379-4385: Semantic drift, colexification, cross-cultural alignment
    /// Per §8.2.1 lines 4723-4746: Language toggle contract
    /// Per music.md: Music as geometric trust indicator (cultural emergence)
    /// Barnes-Hut enabled for large population efficiency
    #[must_use]
    pub fn stage_4() -> Self {
        Self {
            genome_enabled: true,
            docas_enabled: true,
            dmn_enabled: true,
            criticality_enabled: true,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            institutions_enabled: true,
            norms_enabled: true,
            language_enabled: true,
            music_enabled: true,
            stigmergy_enabled: true,
            barnes_hut_enabled: true, // For large populations
            ..Self::default()
        }
    }

    /// Stage 5 (Protocol) - blockchain verification simulation
    /// Per §D.3 line 7552-7568: Protocol stage
    /// Per §8.2.1 lines 4646-4666: Protocol toggle contract
    #[must_use]
    pub fn stage_5() -> Self {
        Self {
            genome_enabled: true,
            docas_enabled: true,
            dmn_enabled: true,
            criticality_enabled: true,
            choice_architecture_enabled: true,
            institutions_enabled: true,
            norms_enabled: true,
            language_enabled: true,
            protocol_enabled: true,
            music_enabled: true,
            economics_enabled: true,
            encounters_enabled: true,
            stigmergy_enabled: true,
            barnes_hut_enabled: true,
            groups_enabled: true,
            geography_enabled: true,
            biology_enabled: true,
            philosophy_enabled: true,
            biomarkers_enabled: true,
            dials: DialConfig::default(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Protocol ON baseline (core research condition)
    /// Per §8.2.1 lines 4646-4666
    /// Alias for `stage_5()` for research clarity
    #[must_use]
    pub fn protocol_on() -> Self {
        Self::stage_5()
    }

    /// Protocol OFF counterfactual (research control)
    #[must_use]
    pub fn protocol_off() -> Self {
        Self {
            genome_enabled: true,
            docas_enabled: true,
            dmn_enabled: true,
            criticality_enabled: true,
            choice_architecture_enabled: true,
            institutions_enabled: true,
            norms_enabled: true,
            language_enabled: true,
            protocol_enabled: false,
            music_enabled: true,
            economics_enabled: true,
            encounters_enabled: true,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            stigmergy_enabled: true,
            barnes_hut_enabled: true,
            groups_enabled: true,
            geography_enabled: true,
            biology_enabled: true,
            philosophy_enabled: true,
            biomarkers_enabled: true,
            dials: DialConfig::default(),
        }
    }

    /// Count number of enabled toggles across all 19 mechanism domains
    #[must_use]
    pub fn enabled_count(&self) -> usize {
        [
            self.genome_enabled,
            self.docas_enabled,
            self.dmn_enabled,
            self.criticality_enabled,
            self.choice_architecture_enabled,
            self.institutions_enabled,
            self.norms_enabled,
            self.language_enabled,
            self.protocol_enabled,
            self.music_enabled,
            self.economics_enabled,
            self.encounters_enabled,
            self.stigmergy_enabled,
            self.barnes_hut_enabled,
            self.groups_enabled,
            self.geography_enabled,
            self.biology_enabled,
            self.philosophy_enabled,
            self.biomarkers_enabled,
        ]
        .iter()
        .filter(|&&x| x)
        .count()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Validate toggle configuration
    /// Returns Ok(()) if valid, Err with description if invalid
    ///
    /// Constraints:
    /// - Protocol requires encounters (anchors/verifications need agent interactions)
    /// - Music requires DOCAS (for neuromodulator modulation)
    /// - Economics requires encounters (trade needs interactions)
    /// - Criticality requires DOCAS (homeostasis uses DOCAS levels)
    /// - Dial parameters must be valid when their parent toggles are enabled
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.protocol_enabled && !self.encounters_enabled {
            return Err("Protocol requires encounters_enabled (anchors/verifications need agent interactions)");
        }

        if self.music_enabled && !self.docas_enabled {
            return Err("Music requires docas_enabled (DOCAS modulation is core to music effects)");
        }

        if self.economics_enabled && !self.encounters_enabled {
            return Err(
                "Economics requires encounters_enabled (trade/debt needs agent interactions)",
            );
        }

        if self.criticality_enabled && !self.docas_enabled {
            return Err("Criticality requires docas_enabled (homeostatic correction uses DOCAS)");
        }

        if self.norms_enabled && !self.encounters_enabled {
            return Err(
                "Norms requires encounters_enabled (norm enforcement happens during encounters)",
            );
        }

        if self.institutions_enabled && !self.encounters_enabled {
            return Err("Institutions requires encounters_enabled (institutional memory updated during encounters)");
        }

        if self.institutions_enabled && !self.groups_enabled {
            return Err("Institutions requires groups_enabled (institutions operate across emergent groups)");
        }

        // Validate dial configuration
        if self.dials.validate().is_err() {
            return Err("Dial configuration validation failed");
        }

        Ok(())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Continuous parameter ranges (dials) for simulation mechanisms.
/// These provide gradient control when their parent toggle is enabled.
///
/// Dials are only applied when their corresponding toggle is enabled:
/// - DOCAS dials apply when `docas_enabled` = true
/// - Criticality dials apply when `criticality_enabled` = true
/// - DMN dials apply when `dmn_enabled` = true
/// - Protocol dials apply when `protocol_enabled` = true
/// - Language dials apply when `language_enabled` = true
/// - Music dials apply when `music_enabled` = true
/// - Choice Architecture dials apply when `choice_architecture_enabled` = true
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DialConfig {
    // DOCAS dials (when docas_enabled = true)
    /// Alpha range: (0.90, 0.99) - learning retention factor
    /// Per Dayan & Abbott 2001: TD learning decay parameter
    pub docas_alpha_range: (f32, f32),
    /// Beta range: (0.05, 0.20) - update rate for DOCAS components
    /// Per Montague et al. 1996: Neuromodulator sensitivity
    pub docas_beta_range: (f32, f32),

    // Criticality dial (when criticality_enabled = true)
    /// Target criticality σ: 0.8-1.2, default 1.0
    /// Per CRI-001: Self-organized criticality target
    pub criticality_sigma_target: f32,
    /// Tolerance for criticality deviation: ±0.1 default
    pub criticality_tolerance: f32,

    // DMN dials (when dmn_enabled = true)
    /// K-rollout range: (3, 20) - planning depth for counterfactuals
    /// Per §6.2: DMN planning horizon
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub dmn_k_rollout_range: (u32, u32),
    /// H-depth range: (1, 5) - heuristic evaluation depth
    pub dmn_h_depth_range: (u32, u32),
    /// B simulations: 100-1000 default 100
    /// Number of simulations per planning step
    pub dmn_b_simulations: u32,

    // Protocol dial (when protocol_enabled = true)
    /// Anchor cost range: (0.0001, 0.01) SIM tokens
    /// Per §D.3: Verification economics
    pub protocol_anchor_cost_range: (f32, f32),

    // Language dials (when language_enabled = true)
    /// Drift rate factor: k in drift ∝ k/τ
    /// Controls speed of semantic drift relative to trust
    pub language_drift_rate_factor: f32,
    /// Coherence threshold: cosine similarity for alignment
    /// Per §7.5.1: Lexical alignment threshold
    pub language_coherence_threshold: f32,

    // Music dials (when music_enabled = true)
    /// Engagement range: (0.1, 1.0) - baseline musical engagement
    pub music_engagement_range: (f32, f32),
    /// Sync skill range: (0.0, 1.0) - rhythmic synchronization ability
    pub music_sync_skill_range: (f32, f32),

    // Choice Architecture dials (when choice_architecture_enabled = true)
    /// Loss aversion lambda: 2.0-2.5 (Thaler range)
    /// Per Thaler & Sunstein: Prospect theory loss aversion
    pub choice_loss_aversion_lambda: f32,
    /// Friction units: 0-4 default 0
    /// Number of "clicks" added to dispreferred options
    pub choice_friction_units: u32,
    /// Default bias: 0.0-0.3 - probability boost for default option
    pub choice_default_bias: f32,
    /// Anchor adjustment: 0.0-0.5 - adjustment away from anchor
    pub choice_anchor_adjustment: f32,

    // Genealogy & Lineage dials (Sim K-Series)
    /// Kingman coalescence intensity kappa* (default: 1.62, calibrated in Sim K)
    #[serde(default = "default_kingman_coalescent")]
    pub kingman_coalescent: f32,
    /// Lineage depth for ancestral tracing (default: 10)
    #[serde(default = "default_lineage_depth")]
    pub lineage_depth: usize,
    /// Music metric hierarchy ratio tau (default: 0.50, Sim M)
    #[serde(default = "default_music_metric_hierarchy_tau")]
    pub music_metric_hierarchy_tau: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for DialConfig {
    /// Default dial values based on research literature
    fn default() -> Self {
        Self {
            // DOCAS: Conservative learning, moderate sensitivity
            docas_alpha_range: (0.90, 0.99),
            docas_beta_range: (0.05, 0.20),

            // Criticality: Target critical state (σ=1.0)
            criticality_sigma_target: 1.0,
            criticality_tolerance: 0.1,

            // DMN: Moderate planning depth
            dmn_k_rollout_range: (3, 20),
            dmn_h_depth_range: (1, 5),
            dmn_b_simulations: 100,

            // Protocol: Low anchor costs for high participation
            protocol_anchor_cost_range: (0.0001, 0.01),

            // Language: Moderate drift, typical coherence
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            language_drift_rate_factor: 1.0,
            language_coherence_threshold: 0.7,

            // Music: Moderate engagement, variable skill
            music_engagement_range: (0.1, 1.0),
            music_sync_skill_range: (0.0, 1.0),

            // Choice Architecture: Standard Thaler values
            choice_loss_aversion_lambda: 2.25,
            choice_friction_units: 0,
            choice_default_bias: 0.2,
            choice_anchor_adjustment: 0.3,

            // Genealogy & Lineage (Sim K) & Music Hierarchy (Sim M)
            kingman_coalescent: 1.62,
            lineage_depth: 10,
            music_metric_hierarchy_tau: 0.50,
        }
    }
}

/// Validation error for dial configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialValidationError {
    /// field.
    pub field: String,
    /// message.
    pub message: String,
}

impl std::fmt::Display for DialValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl std::error::Error for DialValidationError {}

/// Validate a (min, max) range tuple where min < max and both are within bounds
fn validate_dial_range<T: PartialOrd + std::fmt::Display>(
    range: (T, T),
    field: &str,
    bounds: (T, T),
) -> Result<(), DialValidationError> {
    let (min_val, max_val) = range;
    let (bound_min, bound_max) = bounds;

    if min_val < bound_min || max_val > bound_max || min_val >= max_val {
        return Err(DialValidationError {
            field: field.into(),
            message: format!(
                "must satisfy {bound_min} < min < max < {bound_max}, got ({min_val}, {max_val})"
            ),
        });
    }
    Ok(())
}

/// Validate a value is within exclusive bounds
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[allow(clippy::needless_pass_by_value)]
fn validate_dial_value<T: PartialOrd + std::fmt::Display>(
    value: T,
    field: &str,
    min: T,
    max: T,
) -> Result<(), DialValidationError> {
    if value <= min || value >= max {
        return Err(DialValidationError {
            field: field.into(),
            message: format!("must be in ({min}, {max}), got {value}"),
        });
    }
    Ok(())
}

/// Validate a value is within inclusive bounds
/// # Errors
///
/// Returns an error if the operation fails.
#[allow(clippy::needless_pass_by_value)]
fn validate_dial_inclusive<T: PartialOrd + std::fmt::Display>(
    value: T,
    field: &str,
    min: T,
    max: T,
) -> Result<(), DialValidationError> {
    if value < min || value > max {
        return Err(DialValidationError {
            field: field.into(),
            message: format!("must be in [{min}, {max}], got {value}"),
        });
    }
    Ok(())
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DialConfig {
    /// Validate all dial parameters are within bounds
    ///
    /// Validation Rules:
    /// - Alpha range: 0.8 < min < max < 1.0
    /// - Beta range: 0.01 < min < max < 0.5
    /// - Sigma target: 0.5 < value < 2.0
    /// - All ranges: min < max
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), DialValidationError> {
        self.validate_docas_ranges()?;
        self.validate_criticality()?;
        self.validate_dmn()?;
        self.validate_protocol()?;
        self.validate_language()?;
        self.validate_music()?;
        self.validate_choice_architecture()?;
        Ok(())
    }

    fn validate_docas_ranges(&self) -> Result<(), DialValidationError> {
        // Alpha: 0.8 < min < max < 1.0 (strict bounds)
        let (alpha_min, alpha_max) = self.docas_alpha_range;
        if alpha_min <= 0.8 || alpha_min >= alpha_max || alpha_max >= 1.0 {
            return Err(DialValidationError {
                field: "docas_alpha_range".into(),
                message: format!(
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    "must satisfy 0.8 < min < max < 1.0, got ({alpha_min}, {alpha_max})"
                ),
            });
        }
        // Beta: 0.01 ≤ min < max ≤ 0.5
        validate_dial_range(self.docas_beta_range, "docas_beta_range", (0.01, 0.5))
    }

    fn validate_criticality(&self) -> Result<(), DialValidationError> {
        validate_dial_value(
            self.criticality_sigma_target,
            "criticality_sigma_target",
            0.5,
            2.0,
        )?;
        validate_dial_inclusive(
            self.criticality_tolerance,
            "criticality_tolerance",
            0.0,
            0.5,
        )
    }

    fn validate_dmn(&self) -> Result<(), DialValidationError> {
        validate_dial_range(self.dmn_k_rollout_range, "dmn_k_rollout_range", (0, 100))?;
        validate_dial_range(self.dmn_h_depth_range, "dmn_h_depth_range", (0, 10))?;
        validate_dial_inclusive(self.dmn_b_simulations, "dmn_b_simulations", 1, 10000)
    }

    fn validate_protocol(&self) -> Result<(), DialValidationError> {
        // Protocol ranges: 0.0 < min < max ≤ 1.0 (min must be strictly > 0)
        let (cost_min, cost_max) = self.protocol_anchor_cost_range;
        if cost_min <= 0.0 || cost_min >= cost_max || cost_max > 1.0 {
            return Err(DialValidationError {
                field: "protocol_anchor_cost_range".into(),
                message: format!(
                    "must satisfy 0.0 < min < max ≤ 1.0, got ({cost_min}, {cost_max})"
                ),
            });
        }
        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn validate_language(&self) -> Result<(), DialValidationError> {
        validate_dial_inclusive(
            self.language_drift_rate_factor,
            "language_drift_rate_factor",
            0.0,
            10.0,
        )?;
        validate_dial_inclusive(
            self.language_coherence_threshold,
            "language_coherence_threshold",
            0.0,
            1.0,
        )
    }

    fn validate_music(&self) -> Result<(), DialValidationError> {
        validate_dial_range(
            self.music_engagement_range,
            "music_engagement_range",
            (0.0, 1.0),
        )?;
        validate_dial_range(
            self.music_sync_skill_range,
            "music_sync_skill_range",
            (0.0, 1.0),
        )
    }

    fn validate_choice_architecture(&self) -> Result<(), DialValidationError> {
        validate_dial_inclusive(
            self.choice_loss_aversion_lambda,
            "choice_loss_aversion_lambda",
            1.0,
            5.0,
        )?;
        validate_dial_inclusive(self.choice_friction_units, "choice_friction_units", 0, 10)?;
        validate_dial_inclusive(self.choice_default_bias, "choice_default_bias", 0.0, 1.0)?;
        validate_dial_inclusive(
            self.choice_anchor_adjustment,
            "choice_anchor_adjustment",
            0.0,
            1.0,
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a random alpha value within the configured range
    /// Uses the provided RNG for deterministic sampling
    pub fn sample_alpha<R: rand::Rng>(&self, rng: &mut R) -> f32 {
        rng.gen_range(self.docas_alpha_range.0..self.docas_alpha_range.1)
    }

    /// Get a random beta value within the configured range
    pub fn sample_beta<R: rand::Rng>(&self, rng: &mut R) -> f32 {
        rng.gen_range(self.docas_beta_range.0..self.docas_beta_range.1)
    }

    /// Get a random K-rollout value within the configured range
    pub fn sample_k_rollout<R: rand::Rng>(&self, rng: &mut R) -> u32 {
        rng.gen_range(self.dmn_k_rollout_range.0..self.dmn_k_rollout_range.1)
    }

    /// Get a random H-depth value within the configured range
    pub fn sample_h_depth<R: rand::Rng>(&self, rng: &mut R) -> u32 {
        rng.gen_range(self.dmn_h_depth_range.0..self.dmn_h_depth_range.1)
    }

    /// Get a random anchor cost within the configured range
    pub fn sample_anchor_cost<R: rand::Rng>(&self, rng: &mut R) -> f32 {
        rng.gen_range(self.protocol_anchor_cost_range.0..self.protocol_anchor_cost_range.1)
    }

    /// Get a random music engagement value within the configured range
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub fn sample_music_engagement<R: rand::Rng>(&self, rng: &mut R) -> f32 {
        rng.gen_range(self.music_engagement_range.0..self.music_engagement_range.1)
    }

    /// Get a random music sync skill value within the configured range
    pub fn sample_music_sync_skill<R: rand::Rng>(&self, rng: &mut R) -> f32 {
        rng.gen_range(self.music_sync_skill_range.0..self.music_sync_skill_range.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_social_enabled() {
        let config = ToggleConfig::default();
        // Default now enables encounters for social interactions (POM-SOC-001)
        assert!(config.encounters_enabled);
        // Other toggles remain off for minimal configuration
        assert!(!config.genome_enabled);
        assert!(!config.docas_enabled);
    }

    #[test]
    fn test_all_on() {
        let config = ToggleConfig::all_on();
        assert_eq!(config.enabled_count(), 19); // All 19 toggles across all mechanism domains
    }

    #[test]
    fn test_stage_progression() {
        // Stage 0 includes the 6 baseline environment/body toggles
        assert_eq!(ToggleConfig::stage_0().enabled_count(), 6);
        assert_eq!(ToggleConfig::stage_1().enabled_count(), 7);
        assert_eq!(ToggleConfig::stage_2().enabled_count(), 10);
        // Stage 3 adds institutions + norms + stigmergy
        assert_eq!(ToggleConfig::stage_3().enabled_count(), 13);
        // Stage 4 adds language + music + barnes_hut
        assert_eq!(ToggleConfig::stage_4().enabled_count(), 16);
        // Stage 5 has all 19 toggles enabled
        assert_eq!(ToggleConfig::stage_5().enabled_count(), 19);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_stage_4_language_enabled() {
        let config = ToggleConfig::stage_4();
        assert!(config.language_enabled);
        assert!(config.norms_enabled);
        assert!(config.institutions_enabled);
        assert!(config.music_enabled); // Music enabled at Stage 4
        assert!(!config.protocol_enabled); // Protocol is Stage 5
    }

    #[test]
    fn test_stage_5_protocol_enabled() {
        let config = ToggleConfig::stage_5();
        assert!(config.protocol_enabled);
        assert!(config.language_enabled);
        assert!(config.norms_enabled);
        assert!(config.institutions_enabled);
        assert!(config.music_enabled);
        assert!(config.genome_enabled);
        assert!(config.docas_enabled);
        assert!(config.barnes_hut_enabled);
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[test]
    fn test_protocol_on_equals_stage_5() {
        let stage_5 = ToggleConfig::stage_5();
        let protocol_on = ToggleConfig::protocol_on();
        assert_eq!(stage_5.enabled_count(), protocol_on.enabled_count());
        assert_eq!(stage_5.protocol_enabled, protocol_on.protocol_enabled);
        assert_eq!(stage_5.barnes_hut_enabled, protocol_on.barnes_hut_enabled);
    }

    #[test]
    fn test_validate_valid_configs() {
        // Standard stage configs should all be valid
        assert!(ToggleConfig::stage_0().validate().is_ok());
        assert!(ToggleConfig::stage_1().validate().is_ok());
        assert!(ToggleConfig::stage_2().validate().is_ok());
        assert!(ToggleConfig::stage_3().validate().is_ok());
        assert!(ToggleConfig::stage_4().validate().is_ok());
        assert!(ToggleConfig::stage_5().validate().is_ok());
        assert!(ToggleConfig::all_on().validate().is_ok());
    }

    #[test]
    fn test_validate_protocol_requires_encounters() {
        let config = ToggleConfig {
            protocol_enabled: true,
            encounters_enabled: false,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_music_requires_docas() {
        let config = ToggleConfig {
            music_enabled: true,
            docas_enabled: false,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_validate_economics_requires_encounters() {
        let config = ToggleConfig {
            economics_enabled: true,
            encounters_enabled: false,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    // === DialConfig Tests ===

    #[test]
    fn test_dial_config_default() {
        let dials = DialConfig::default();

        // DOCAS defaults
        assert_eq!(dials.docas_alpha_range, (0.90, 0.99));
        assert_eq!(dials.docas_beta_range, (0.05, 0.20));

        // Criticality defaults
        assert_eq!(dials.criticality_sigma_target, 1.0);
        assert_eq!(dials.criticality_tolerance, 0.1);

        // DMN defaults
        assert_eq!(dials.dmn_k_rollout_range, (3, 20));
        assert_eq!(dials.dmn_h_depth_range, (1, 5));
        assert_eq!(dials.dmn_b_simulations, 100);

        // Protocol defaults
        assert_eq!(dials.protocol_anchor_cost_range, (0.0001, 0.01));

        // Language defaults
        assert_eq!(dials.language_drift_rate_factor, 1.0);
        assert_eq!(dials.language_coherence_threshold, 0.7);

        // Music defaults
        assert_eq!(dials.music_engagement_range, (0.1, 1.0));
        assert_eq!(dials.music_sync_skill_range, (0.0, 1.0));

        // Choice Architecture defaults
        assert_eq!(dials.choice_loss_aversion_lambda, 2.25);
        assert_eq!(dials.choice_friction_units, 0);
        assert_eq!(dials.choice_default_bias, 0.2);
        assert_eq!(dials.choice_anchor_adjustment, 0.3);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_dial_config_custom() {
        let dials = DialConfig {
            docas_alpha_range: (0.92, 0.95),
            docas_beta_range: (0.08, 0.15),
            criticality_sigma_target: 1.1,
            criticality_tolerance: 0.15,
            dmn_k_rollout_range: (5, 15),
            dmn_h_depth_range: (2, 4),
            dmn_b_simulations: 200,
            protocol_anchor_cost_range: (0.001, 0.005),
            language_drift_rate_factor: 1.5,
            language_coherence_threshold: 0.8,
            music_engagement_range: (0.2, 0.9),
            music_sync_skill_range: (0.1, 0.8),
            choice_loss_aversion_lambda: 2.5,
            choice_friction_units: 2,
            choice_default_bias: 0.25,
            choice_anchor_adjustment: 0.4,
            kingman_coalescent: 1.62,
            lineage_depth: 10,
            music_metric_hierarchy_tau: 0.50,
        };

        assert!(dials.validate().is_ok());
    }

    #[test]
    fn test_dial_config_validation_pass() {
        let dials = DialConfig::default();
        assert!(dials.validate().is_ok());
    }

    #[test]
    fn test_dial_config_alpha_range_invalid() {
        // Alpha min too low
        let dials = DialConfig {
            docas_alpha_range: (0.7, 0.95),
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // Alpha min >= max
        let dials = DialConfig {
            docas_alpha_range: (0.95, 0.90),
            ..Default::default()
        };
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(dials.validate().is_err());

        // Alpha max too high
        let dials = DialConfig {
            docas_alpha_range: (0.90, 1.0),
            ..Default::default()
        };
        assert!(dials.validate().is_err());
    }

    #[test]
    fn test_dial_config_beta_range_invalid() {
        // Beta min too low
        let dials = DialConfig {
            docas_beta_range: (0.001, 0.20),
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // Beta min >= max
        let dials = DialConfig {
            docas_beta_range: (0.30, 0.20),
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // Beta max too high
        let dials = DialConfig {
            docas_beta_range: (0.05, 0.6),
            ..Default::default()
        };
        assert!(dials.validate().is_err());
    }

    #[test]
    fn test_dial_config_criticality_invalid() {
        // Sigma too low
        let dials = DialConfig {
            criticality_sigma_target: 0.4,
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // Sigma too high
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let dials = DialConfig {
            criticality_sigma_target: 2.5,
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // Tolerance too high
        let dials = DialConfig {
            criticality_tolerance: 0.6,
            ..Default::default()
        };
        assert!(dials.validate().is_err());
    }

    #[test]
    fn test_dial_config_dmn_invalid() {
        // K-rollout min >= max
        let dials = DialConfig {
            dmn_k_rollout_range: (20, 10),
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // H-depth max too high
        let dials = DialConfig {
            dmn_h_depth_range: (1, 20),
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // B simulations zero
        let dials = DialConfig {
            dmn_b_simulations: 0,
            ..Default::default()
        };
        assert!(dials.validate().is_err());
    }

    #[test]
    fn test_dial_config_protocol_invalid() {
        // Anchor cost min <= 0
        let dials = DialConfig {
            protocol_anchor_cost_range: (0.0, 0.01),
            ..Default::default() // TAG: surface=simulation owner=research-team rule=SIM-001
        };
        assert!(dials.validate().is_err());

        // Anchor cost min >= max
        let dials = DialConfig {
            protocol_anchor_cost_range: (0.02, 0.01),
            ..Default::default()
        };
        assert!(dials.validate().is_err());
    }

    #[test]
    fn test_dial_config_language_invalid() {
        // Drift factor too high
        let dials = DialConfig {
            language_drift_rate_factor: 15.0,
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // Coherence threshold too high
        let dials = DialConfig {
            language_coherence_threshold: 1.5,
            ..Default::default()
        };
        assert!(dials.validate().is_err());
    }

    #[test]
    fn test_dial_config_music_invalid() {
        // Engagement min >= max
        let dials = DialConfig {
            music_engagement_range: (0.8, 0.2),
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // Sync skill max too high
        let dials = DialConfig {
            music_sync_skill_range: (0.0, 1.5),
            ..Default::default()
        };
        assert!(dials.validate().is_err());
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[test]
    fn test_dial_config_choice_architecture_invalid() {
        // Loss aversion too high
        let dials = DialConfig {
            choice_loss_aversion_lambda: 6.0,
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // Friction too high
        let dials = DialConfig {
            choice_friction_units: 15,
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // Default bias too high
        let dials = DialConfig {
            choice_default_bias: 1.5,
            ..Default::default()
        };
        assert!(dials.validate().is_err());

        // Anchor adjustment too high
        let dials = DialConfig {
            choice_anchor_adjustment: 1.5,
            ..Default::default()
        };
        assert!(dials.validate().is_err());
    }

    #[test]
    fn test_dial_config_serialization_roundtrip() {
        let original = DialConfig::default();
        // ERR-001: Test code - unwrap() exempt for test assertions
        let json = serde_json::to_string(&original).unwrap();
        let parsed: DialConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original.docas_alpha_range, parsed.docas_alpha_range);
        assert_eq!(original.docas_beta_range, parsed.docas_beta_range);
        assert_eq!(
            original.criticality_sigma_target,
            parsed.criticality_sigma_target
        );
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(original.criticality_tolerance, parsed.criticality_tolerance);
        assert_eq!(original.dmn_k_rollout_range, parsed.dmn_k_rollout_range);
        assert_eq!(original.dmn_h_depth_range, parsed.dmn_h_depth_range);
        assert_eq!(original.dmn_b_simulations, parsed.dmn_b_simulations);
        assert_eq!(
            original.protocol_anchor_cost_range,
            parsed.protocol_anchor_cost_range
        );
        assert_eq!(
            original.language_drift_rate_factor,
            parsed.language_drift_rate_factor
        );
        assert_eq!(
            original.language_coherence_threshold,
            parsed.language_coherence_threshold
        );
        assert_eq!(
            original.music_engagement_range,
            parsed.music_engagement_range
        );
        assert_eq!(
            original.music_sync_skill_range,
            parsed.music_sync_skill_range
        );
        assert_eq!(
            original.choice_loss_aversion_lambda,
            parsed.choice_loss_aversion_lambda
        );
        assert_eq!(original.choice_friction_units, parsed.choice_friction_units);
        assert_eq!(original.choice_default_bias, parsed.choice_default_bias);
        assert_eq!(
            original.choice_anchor_adjustment,
            parsed.choice_anchor_adjustment
        );
    }

    #[test]
    fn test_toggle_config_with_dials_serialization() {
        let original = ToggleConfig::stage_5();
        // ERR-001: Test code - unwrap() exempt for test assertions
        let json = serde_json::to_string(&original).unwrap();
        let parsed: ToggleConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original.enabled_count(), parsed.enabled_count());
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(
            original.dials.docas_alpha_range,
            parsed.dials.docas_alpha_range
        );
        assert_eq!(
            original.dials.criticality_sigma_target,
            parsed.dials.criticality_sigma_target
        );
    }

    #[test]
    fn test_backward_compatibility_no_dials_field() {
        // Simulate old config without dials field - should use default
        let json = r#"{
            "genome_enabled": true,
            "docas_enabled": false,
            "dmn_enabled": true,
            "criticality_enabled": false,
            "choice_architecture_enabled": true,
            "institutions_enabled": false,
            "norms_enabled": true,
            "language_enabled": false,
            "protocol_enabled": true,
            "music_enabled": false,
            "economics_enabled": true,
            "encounters_enabled": true,
            "stigmergy_enabled": true,
            "barnes_hut_enabled": false
        }"#;

        // ERR-001: Test code - unwrap() exempt for test assertions
        let parsed: ToggleConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.genome_enabled);
        assert!(!parsed.docas_enabled);
        assert!(parsed.dmn_enabled);

        // Dials should have default values
        assert_eq!(parsed.dials.docas_alpha_range, (0.90, 0.99));
        assert_eq!(parsed.dials.criticality_sigma_target, 1.0);
    }

    #[test]
    fn test_dial_sampling() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        // TAG: surface=simulation owner=research-team rule=SIM-001

        let dials = DialConfig::default();
        let mut rng = StdRng::seed_from_u64(42);

        // Test that sampling produces values within range
        let alpha = dials.sample_alpha(&mut rng);
        assert!(alpha >= dials.docas_alpha_range.0 && alpha < dials.docas_alpha_range.1);

        let beta = dials.sample_beta(&mut rng);
        assert!(beta >= dials.docas_beta_range.0 && beta < dials.docas_beta_range.1);

        let k = dials.sample_k_rollout(&mut rng);
        assert!(k >= dials.dmn_k_rollout_range.0 && k < dials.dmn_k_rollout_range.1);

        let h = dials.sample_h_depth(&mut rng);
        assert!(h >= dials.dmn_h_depth_range.0 && h < dials.dmn_h_depth_range.1);

        let anchor_cost = dials.sample_anchor_cost(&mut rng);
        assert!(
            anchor_cost >= dials.protocol_anchor_cost_range.0
                && anchor_cost < dials.protocol_anchor_cost_range.1
        );

        let engagement = dials.sample_music_engagement(&mut rng);
        assert!(
            engagement >= dials.music_engagement_range.0
                && engagement < dials.music_engagement_range.1
        );

        let sync_skill = dials.sample_music_sync_skill(&mut rng);
        assert!(
            sync_skill >= dials.music_sync_skill_range.0
                && sync_skill < dials.music_sync_skill_range.1
        );
    }

    #[test]
    fn test_dial_validation_error_display() {
        let err = DialValidationError {
            field: "test_field".into(),
            message: "test message".into(),
        };
        assert_eq!(format!("{err}"), "test_field: test message");
    }
}
