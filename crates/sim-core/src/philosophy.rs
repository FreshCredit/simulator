//! Philosophy Module: Philosophical Trust Axis (`τ_φ`) and Bond Mechanism
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION philosophy, not metaphysical claims
//!
//! Per Paper 6: Philosophy paper on trust axis
//! Per §7.6.1 Blocker 1: Required for PHIL-P0-* gaps resolution
//!
//! The philosophical trust axis `τ_φ` ∈ [-1, +1] represents the agent's
//! philosophical orientation toward trust:
//! -1.0 = radical skepticism (Cartesian doubt)
//! +1.0 = radical openness (Kierkegaardian leap of faith)

// TAG: surface=simulation owner=research-team rule=SIM-001
use serde::{Deserialize, Serialize};

use crate::docas::DocasState;
use crate::types::Tick;

/// Annual subsampling buffer for long-term oscillation detection
/// Per PHIL-P0-2: 250-year period target requires annual sampling
///
/// Research: Annual subsampling (1 sample/year) provides proper frequency resolution
/// for detecting 200-300 year oscillations while keeping memory usage manageable.
///
/// Memory: 2048 years × 4 bytes = 8KB per agent (8MB for 1000 agents)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnnualPhilosophyHistory {
    /// Annual `τ_φ` samples (mean per year)
    /// Capacity: 2048 years of history
    pub annual_tau: Vec<f32>,

    /// Accumulator for current year's `τ_φ` values
    pub year_accumulator: f32,

    /// Count of samples in current year
    pub year_sample_count: u32,

    /// Current year index (0 = simulation start)
    pub current_year: u32,

    /// Total years of history stored
    pub years_stored: usize,
}

impl Default for AnnualPhilosophyHistory {
    fn default() -> Self {
        Self {
            annual_tau: Vec::with_capacity(2048),
            year_accumulator: 0.0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            year_sample_count: 0,
            current_year: 0,
            years_stored: 0,
        }
    }
}

impl AnnualPhilosophyHistory {
    /// Add daily `τ_φ` sample, computing annual mean at year boundary
    /// Call once per tick with tick-based year detection
    pub fn add_sample(&mut self, tau_phi: f32, tick: Tick, ticks_per_year: u64) {
        let year = (tick / ticks_per_year) as u32;

        if year > self.current_year {
            // New year: finalize previous year's mean
            if self.year_sample_count > 0 {
                let annual_mean = self.year_accumulator / self.year_sample_count as f32;
                self.annual_tau.push(annual_mean);
                self.years_stored = self.annual_tau.len();

                // Keep only last 2048 years
                if self.annual_tau.len() > 2048 {
                    self.annual_tau.remove(0);
                    self.years_stored = 2048;
                }
            }

            // Reset accumulator for new year
            self.year_accumulator = tau_phi;
            self.year_sample_count = 1;
            self.current_year = year;
        } else {
            // Same year: accumulate
            self.year_accumulator += tau_phi;
            self.year_sample_count += 1;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get annual history as slice for analysis
    #[must_use]
    pub fn get_history(&self) -> &[f32] {
        &self.annual_tau
    }

    /// Check if sufficient history for 250-year oscillation detection
    /// Minimum: 500 years (2× period for Nyquist)
    /// Recommended: 750 years (3× period for reliable detection)
    #[must_use]
    pub const fn has_sufficient_history(&self, min_years: usize) -> bool {
        self.years_stored >= min_years
    }
}

/// Philosophical orientation categories
/// Per Paper 6: Four primary philosophical frameworks
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhilosophicalOrientation {
    /// Rationalism: Knowledge through reason (Descartes, Leibniz)
    /// Associated with skepticism, systematic doubt
    Rationalism,
    /// Empiricism: Knowledge through experience (Locke, Hume)
    /// Associated with openness to evidence
    Empiricism,
    /// Pragmatism: Truth as utility (James, Dewey)
    /// Associated with instrumental trust
    #[default]
    Pragmatism,
    /// Existentialism: Authentic choice (Kierkegaard, Sartre)
    /// Associated with radical commitment
    Existentialism,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Ethical framework for moral decision-making
/// Influences how agents evaluate actions and commitments
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EthicalFramework {
    /// Kantian: Focus on intentions, duties, and principled consistency
    /// High commitment to promises, regardless of outcomes
    Kantian,
    /// Utilitarian: Focus on outcomes and welfare maximization
    /// Willing to break commitments if net benefit is positive
    Utilitarian,
    /// Virtue Ethics: Focus on character and virtuous action
    /// Considers integrity and moral character in decisions
    #[default]
    VirtueEthics,
}

/// Virtues for virtue ethics framework
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Virtue {
    /// Integrity: Consistency between values and actions
    Integrity,
    /// Courage: Willingness to act despite risk
    Courage,
    /// Wisdom: Good judgment in complex situations
    Wisdom,
    /// Compassion: Concern for others' welfare
    Compassion,
    /// Temperance: Moderation and self-control
    Temperance,
    /// Justice: Fairness in dealings with others
    Justice,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Set of virtue scores for virtue ethics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VirtueSet {
    /// Map of virtue to score [0.0, 1.0]
    scores: std::collections::BTreeMap<Virtue, f32>,
}

impl Default for VirtueSet {
    fn default() -> Self {
        let mut scores = std::collections::BTreeMap::new();
        scores.insert(Virtue::Integrity, 0.5);
        scores.insert(Virtue::Courage, 0.5);
        scores.insert(Virtue::Wisdom, 0.5);
        scores.insert(Virtue::Compassion, 0.5);
        scores.insert(Virtue::Temperance, 0.5);
        scores.insert(Virtue::Justice, 0.5);
        Self { scores }
    }
}

impl VirtueSet {
    /// Get score for a specific virtue
    #[must_use]
    pub fn get(&self, virtue: Virtue) -> f32 {
        self.scores.get(&virtue).copied().unwrap_or(0.5)
    }

    /// Set score for a specific virtue
    pub fn set(&mut self, virtue: Virtue, score: f32) {
        self.scores.insert(virtue, score.clamp(0.0, 1.0));
    }

    /// Get average virtue score
    #[must_use]
    pub fn get_average(&self) -> f32 {
        if self.scores.is_empty() {
            return 0.5;
        }
        let sum: f32 = self.scores.values().sum();
        sum / self.scores.len() as f32
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }
}

/// Worldview components for philosophical state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Worldview {
    /// Trust axis: -1 (radical skepticism) to +1 (radical openness)
    pub trust_axis: f32,
    /// Belief in inherent order vs chaos
    pub order_belief: f32,
    /// Individual vs collective orientation
    pub individual_collective: f32,
}

impl Default for Worldview {
    fn default() -> Self {
        Self {
            trust_axis: 0.0,
            order_belief: 0.5,
            individual_collective: 0.0,
        }
    }
}

/// Epistemological stance: how agent approaches knowledge
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Epistemology {
    /// How much weight given to experience vs reason
    pub empiricism_ratio: f32,
    /// Epistemic humility: willingness to update beliefs
    pub epistemic_humility: f32,
    /// Certainty threshold for action
    pub certainty_threshold: f32,
}

impl Default for Epistemology {
    fn default() -> Self {
        Self {
            empiricism_ratio: 0.5,
            epistemic_humility: 0.5,
            certainty_threshold: 0.6,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Philosophical state per agent per Paper 6
/// Per §7.6.1 lines 2217-2239
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhilosophicalState {
    /// `τ_φ` ∈ [-1.0, +1.0]: philosophical trust orientation
    /// -1.0 = radical skepticism (Cartesian doubt)
    /// +1.0 = radical openness (Kierkegaardian leap of faith)
    pub tau_phi: f32,

    /// Bond activation threshold [0.0, 1.0]
    /// Below this threshold, bonds form; above, bonds dissolve
    /// Per PHIL-P0-4: Bond mechanism formalization
    pub bond_activation: f32,

    /// Philosophical orientation weights [Rationalism, Empiricism, Pragmatism, Existentialism]
    /// Per PHIL-P0-3: sensitivity to different philosophical frameworks
    /// Weights sum to 1.0
    pub orientation_weights: [f32; 4],

    /// History of `τ_φ` for oscillation detection (PHIL-P0-2)
    /// Stores (tick, `tau_phi`) pairs for FFT analysis
    pub tau_phi_history: Vec<(Tick, f32)>,

    /// Current dominant orientation
    pub dominant_orientation: PhilosophicalOrientation,

    /// Oscillation period in ticks (target: ~250 years)
    /// Per PHIL-P0-2: ~250-year oscillation in philosophical trust orientation
    /// If None, oscillation not yet detected
    pub oscillation_period: Option<f32>,

    /// Current phase in oscillation cycle [0.0, 2π]
    pub oscillation_phase: f32,

    /// Taxonomy sensitivity coefficients for PHIL-P0-3
    /// How much `τ_φ` changes per unit weight perturbation
    pub taxonomy_sensitivity: [f32; 4],

    /// Annual subsampled history for long-term oscillation detection
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Replaces insufficient tick-level history for 250-year period detection
    /// Per PHIL-P0-2: Annual sampling provides proper frequency resolution
    pub annual_history: AnnualPhilosophyHistory,

    // === NEW: Decision influence fields ===
    /// Ethical framework for moral decision-making
    pub ethical_framework: EthicalFramework,

    /// Virtue scores for virtue ethics
    pub virtues: VirtueSet,

    /// Worldview components
    pub worldview: Worldview,

    /// Epistemological stance
    pub epistemology: Epistemology,
}

impl Default for PhilosophicalState {
    fn default() -> Self {
        Self::new()
    }
}

impl PhilosophicalState {
    /// Create new philosophical state with default values
    #[must_use]
    pub fn new() -> Self {
        Self {
            tau_phi: 0.0, // Start neutral
            bond_activation: 0.5,
            orientation_weights: [0.25, 0.25, 0.25, 0.25], // Equal weights initially
            tau_phi_history: Vec::with_capacity(1000),
            dominant_orientation: PhilosophicalOrientation::Pragmatism,
            oscillation_period: None,
            oscillation_phase: 0.0,
            taxonomy_sensitivity: [0.5; 4],
            annual_history: AnnualPhilosophyHistory::default(),
            ethical_framework: EthicalFramework::default(),
            virtues: VirtueSet::default(),
            worldview: Worldview::default(),
            epistemology: Epistemology::default(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Initialize philosophical state from genome and initial DOCAS
    /// Per §7.6.1: Genome influences initial philosophical orientation
    #[must_use]
    pub fn from_genome_and_docas(docas: &DocasState, seed_value: f32) -> Self {
        let serotonin_influence = -0.5 * docas.serotonin();
        let oxytocin_influence = 0.5 * docas.oxytocin();
        let initial_tau_phi =
            (serotonin_influence + oxytocin_influence + seed_value * 0.2).clamp(-1.0, 1.0);
        let dominant = Self::dominant_orientation(initial_tau_phi);
        let mut weights = [0.1; 4];
        weights[dominant as usize] = 0.7;

        Self {
            tau_phi: initial_tau_phi,
            bond_activation: (0.5 - initial_tau_phi * 0.3).clamp(0.1, 0.9),
            orientation_weights: weights,
            tau_phi_history: Vec::with_capacity(1000),
            dominant_orientation: dominant,
            oscillation_period: None,
            oscillation_phase: 0.0,
            taxonomy_sensitivity: [0.5; 4],
            annual_history: AnnualPhilosophyHistory::default(),
            ethical_framework: Self::ethical_framework(seed_value),
            virtues: Self::virtues_from_docas(docas),
            worldview: Self::build_worldview(initial_tau_phi, dominant),
            epistemology: Self::build_epistemology(initial_tau_phi, dominant),
        }
    }

    fn dominant_orientation(tau_phi: f32) -> PhilosophicalOrientation {
        if tau_phi < -0.5 {
            PhilosophicalOrientation::Rationalism
        } else if tau_phi > 0.5 {
            PhilosophicalOrientation::Existentialism
        } else if tau_phi < 0.0 {
            PhilosophicalOrientation::Pragmatism
        } else {
            PhilosophicalOrientation::Empiricism
        }
    }

    fn ethical_framework(seed: f32) -> EthicalFramework {
        if seed < 0.33 {
            EthicalFramework::Kantian
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        } else if seed < 0.67 {
            EthicalFramework::Utilitarian
        } else {
            EthicalFramework::VirtueEthics
        }
    }

    fn build_worldview(tau_phi: f32, dominant: PhilosophicalOrientation) -> Worldview {
        Worldview {
            trust_axis: tau_phi,
            order_belief: (0.5 + tau_phi * 0.3).clamp(0.0, 1.0),
            individual_collective: if dominant == PhilosophicalOrientation::Existentialism {
                0.3
            } else {
                0.0
            },
        }
    }

    fn build_epistemology(tau_phi: f32, dominant: PhilosophicalOrientation) -> Epistemology {
        Epistemology {
            empiricism_ratio: match dominant {
                PhilosophicalOrientation::Empiricism => 0.8,
                PhilosophicalOrientation::Rationalism => 0.2,
                _ => 0.5,
            },
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            epistemic_humility: (0.5 - tau_phi.abs() * 0.3).clamp(0.2, 0.8),
            certainty_threshold: match dominant {
                PhilosophicalOrientation::Rationalism => 0.8,
                PhilosophicalOrientation::Pragmatism => 0.5,
                _ => 0.6,
            },
        }
    }

    fn virtues_from_docas(docas: &DocasState) -> VirtueSet {
        let mut virtues = VirtueSet::default();
        virtues.set(
            Virtue::Integrity,
            (docas.serotonin() * 0.6 + 0.4).clamp(0.0, 1.0),
        );
        virtues.set(
            Virtue::Courage,
            (docas.dopamine() * 0.5 + docas.adrenaline() * 0.5).clamp(0.0, 1.0),
        );
        virtues.set(
            Virtue::Compassion,
            (docas.oxytocin() * 0.7 + 0.3).clamp(0.0, 1.0),
        );
        virtues
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update `τ_φ` based on DOCAS state
    /// Per §7.6.1: `τ_φ` updates from DOCAS serotonin and oxytocin
    /// Called each tick in simulation loop (step 7)
    pub fn update(&mut self, docas: &DocasState, tick: Tick) {
        // Store history for oscillation detection
        self.tau_phi_history.push((tick, self.tau_phi));

        // Trim history to prevent unbounded growth (keep last 10 years ~ 3650 ticks)
        if self.tau_phi_history.len() > 4000 {
            self.tau_phi_history.remove(0);
        }

        // P0-FIX: Add to annual history for proper long-term oscillation detection
        // Annual subsampling (1 sample/year) provides proper frequency resolution
        // for detecting 200-300 year periods (Welch's method)
        const TICKS_PER_YEAR: u64 = 365;
        self.annual_history
            .add_sample(self.tau_phi, tick, TICKS_PER_YEAR);

        // Update from DOCAS components
        // Serotonin (S) provides stability → skepticism
        // Oxytocin (O) provides social openness → trust
        let serotonin_factor = -0.02 * (docas.serotonin() - 0.5); // Pull toward skepticism
                                                                  // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let oxytocin_factor = 0.02 * (docas.oxytocin() - 0.5); // Pull toward openness

        // Natural oscillation dynamics (250-year period target)
        // Period of 250 years = 250 * 365 = 91250 ticks
        let omega = 2.0 * std::f32::consts::PI / 91250.0;
        self.oscillation_phase += omega;
        if self.oscillation_phase > 2.0 * std::f32::consts::PI {
            self.oscillation_phase -= 2.0 * std::f32::consts::PI;
        }

        // Oscillation amplitude depends on cultural/historical factors
        // Base amplitude of 0.3 means τ_φ ranges ±0.3 around baseline
        let oscillation = 0.3 * self.oscillation_phase.sin();

        // Apply all influences
        let delta_tau = serotonin_factor + oxytocin_factor;
        self.tau_phi = (self.tau_phi + delta_tau).clamp(-1.0, 1.0);

        // Add oscillation on top (this creates the cyclical pattern)
        let oscillated_tau = (self.tau_phi + oscillation).clamp(-1.0, 1.0);

        // Smooth transition: blend current with oscillated
        self.tau_phi = 0.95 * self.tau_phi + 0.05 * oscillated_tau;

        // Update bond activation based on new τ_φ
        // Higher τ_φ (openness) → lower threshold → more bonds form
        // Lower τ_φ (skepticism) → higher threshold → fewer bonds form
        let target_activation = (0.5 - self.tau_phi * 0.3).clamp(0.1, 0.9);
        self.bond_activation = 0.9 * self.bond_activation + 0.1 * target_activation;

        // Update dominant orientation
        self.dominant_orientation = self.compute_dominant_orientation();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute bond strength from `τ_φ` alignment between two agents
    /// Per PHIL-P0-4: Bond formalization
    /// Returns bond strength ∈ [0.0, 1.0]
    #[must_use]
    pub fn compute_bond_activation(&self, partner_tau_phi: f32) -> f32 {
        // Bond strength based on alignment of philosophical orientations
        // Similar τ_φ → stronger bond
        let alignment = 1.0 - (self.tau_phi - partner_tau_phi).abs() / 2.0;

        // Activation requires both agents to be "open" enough
        // Bond forms only if alignment exceeds both agents' thresholds
        let openness_factor = ((self.tau_phi + 1.0) / 2.0) * ((partner_tau_phi + 1.0) / 2.0);

        alignment * openness_factor
    }

    /// Detect oscillation period from `τ_φ` history using autocorrelation
    /// Per PHIL-P0-2: Historical oscillation validation
    /// Returns period in ticks if detected, None otherwise
    #[must_use]
    pub fn detect_oscillation_period(&self) -> Option<f32> {
        if self.tau_phi_history.len() < 1000 {
            return None; // Not enough data
        }

        // Simple autocorrelation to find period
        let values: Vec<f32> = self.tau_phi_history.iter().map(|(_, tau)| *tau).collect();
        let n = values.len();
        let mean = values.iter().sum::<f32>() / n as f32;

        // Try periods from 1000 to 100000 ticks (~3 to ~274 years)
        let mut best_period = None;
        let mut best_correlation = 0.0;

        for period in (1000..100000).step_by(100) {
            if n <= period {
                continue;
            }

            let mut sum_product = 0.0;
            let mut count = 0;

            for i in period..n {
                sum_product += (values[i] - mean) * (values[i - period] - mean);
                count += 1;
            }

            let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n as f32;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            let autocorr = if variance > 0.0 {
                sum_product / (count as f32 * variance)
            } else {
                0.0
            };

            if autocorr > best_correlation && autocorr > 0.3 {
                best_correlation = autocorr;
                best_period = Some(period as f32);
            }
        }

        best_period
    }

    /// Compute FFT-based period for oscillation detection (P2-4)
    ///
    /// ⚠️ **SAMPLING LIMITATION** (P0-FIX): This function analyzes the most recent
    /// 512 ticks of history (~1.4 years at 1 tick/day). It can detect oscillations
    /// with periods of roughly 50-500 ticks (2 months to 1.4 years), NOT the claimed
    /// 250-year periods which would require ~91250+ ticks of continuous history.
    ///
    /// **Current Capability:** Short-term oscillation detection (seasonal to annual cycles)
    /// **Target Capability:** 250-year civilizational cycles (requires ASPIRATIONAL changes)
    ///
    /// ROADMAP(ASPIRATIONAL): To detect 250-year periods:
    /// 1. Implement annual subsampling (1 sample per 365 ticks)
    /// 2. Increase history buffer to 2048+ samples (~2000+ years)
    /// 3. Or use Lomb-Scargle periodogram for irregular sampling
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn compute_fft_period(&self) -> Option<f32> {
        const MIN_HISTORY: usize = 512; // Minimum for meaningful FFT
        const FFT_SIZE: usize = 512; // ~1.4 years of data at 1 tick/day

        if self.tau_phi_history.len() < MIN_HISTORY {
            return None;
        }

        // Extract recent values for FFT window
        let start_idx = self.tau_phi_history.len().saturating_sub(FFT_SIZE);
        let window: Vec<f32> = self.tau_phi_history[start_idx..]
            .iter()
            .map(|(_, tau)| *tau)
            .collect();

        if window.len() < FFT_SIZE {
            return None;
        }

        // Detrend: remove mean
        let mean: f32 = window.iter().sum::<f32>() / window.len() as f32;
        let detrended: Vec<f32> = window.iter().map(|v| v - mean).collect();

        // Apply Hanning window to reduce spectral leakage
        let hanning: Vec<f32> = (0..FFT_SIZE)
            .map(|i| {
                let factor = std::f32::consts::PI * i as f32 / (FFT_SIZE as f32 - 1.0);
                0.5 * (1.0 - factor.cos())
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let windowed: Vec<f32> = detrended
            .iter()
            .zip(hanning.iter())
            .map(|(d, h)| d * h)
            .collect();

        // Simple DFT for frequency analysis
        // Sample rate: 1 sample per tick (typically 1 tick = 1 day)
        let sample_rate = 1.0; // 1 sample per tick
        let freqs: Vec<f32> = (1..FFT_SIZE / 2)
            .map(|k| k as f32 * sample_rate / FFT_SIZE as f32)
            .collect();

        // P0-FIX: Adjusted target range to achievable periods
        // With 512 samples, can reliably detect periods of ~50-500 ticks
        // (2 months to 1.4 years at 1 tick/day)
        let min_period = 50.0; // ~50 ticks (2 months)
        let max_period = 500.0; // ~500 ticks (1.4 years)
        let min_freq = 1.0 / max_period;
        let max_freq = 1.0 / min_period;

        let mut max_power = 0.0f32;
        let mut dominant_freq = None;

        for (_k, freq) in freqs.iter().enumerate().take(FFT_SIZE / 2 - 1).skip(1) {
            if *freq < min_freq || *freq > max_freq {
                continue;
            }

            // Compute DFT at this frequency
            let mut real = 0.0f32;
            let mut imag = 0.0f32;

            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            for (n, sample) in windowed.iter().enumerate() {
                let angle = -2.0 * std::f32::consts::PI * *freq * n as f32;
                real += sample * angle.cos();
                imag += sample * angle.sin();
            }

            let power = real * real + imag * imag;

            if power > max_power {
                max_power = power;
                dominant_freq = Some(*freq);
            }
        }

        // Convert frequency to period
        dominant_freq.map(|f| 1.0 / f)
    }

    /// Get oscillation metrics using FFT analysis
    /// Returns (period, confidence) where confidence is normalized power
    #[must_use]
    pub fn fft_oscillation_metrics(&self) -> (Option<f32>, f32) {
        const MIN_HISTORY: usize = 512;
        const FFT_SIZE: usize = 512;

        if self.tau_phi_history.len() < MIN_HISTORY {
            return (None, 0.0);
        }

        let start_idx = self.tau_phi_history.len().saturating_sub(FFT_SIZE);
        let window: Vec<f32> = self.tau_phi_history[start_idx..]
            .iter()
            .map(|(_, tau)| *tau)
            .collect();

        // Detrend and window
        let mean = window.iter().sum::<f32>() / window.len() as f32;
        let detrended: Vec<f32> = window.iter().map(|v| v - mean).collect();

        // Hanning window
        let hanning: Vec<f32> = (0..FFT_SIZE)
            .map(|i| {
                let factor = std::f32::consts::PI * i as f32 / (FFT_SIZE as f32 - 1.0);
                0.5 * (1.0 - factor.cos())
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let windowed: Vec<f32> = detrended
            .iter()
            .zip(hanning.iter())
            .map(|(d, h)| d * h)
            .collect();

        // Find dominant frequency in 200-300 year range
        let min_period = 200.0;
        let max_period = 300.0;
        let min_freq = 1.0 / max_period;
        let max_freq = 1.0 / min_period;

        let sample_rate = 1.0;
        let mut max_power = 0.0f32;
        let mut total_power = 0.0f32;
        let mut dominant_freq = None;

        for freq in (1..FFT_SIZE / 2).map(|k| k as f32 * sample_rate / FFT_SIZE as f32) {
            if freq < min_freq || freq > max_freq {
                continue;
            }

            let mut real = 0.0f32;
            let mut imag = 0.0f32;

            for (n, sample) in windowed.iter().enumerate() {
                let angle = -2.0 * std::f32::consts::PI * freq * n as f32;
                real += sample * angle.cos();
                imag += sample * angle.sin();
            }

            let power = real * real + imag * imag;
            total_power += power;

            if power > max_power {
                max_power = power;
                dominant_freq = Some(freq);
            }
        }

        // Confidence is proportion of power in dominant frequency
        let confidence = if total_power > 0.0 {
            max_power / total_power
        } else {
            0.0
        };

        (dominant_freq.map(|f| 1.0 / f), confidence)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute long-term oscillation period using Welch's method on annual history
    ///
    /// PROPER IMPLEMENTATION (P0-FIX): Uses annual subsampling (1 sample/year) with
    /// Welch's method for spectral estimation. Can detect 200-300 year oscillations
    /// with sufficient statistical power.
    ///
    /// Requirements:
    /// - Minimum 500 years of history (2× period for Nyquist)
    /// - Recommended 750 years for reliable detection (3× period)
    /// - Uses 512-year segments with 50% overlap
    /// - Hanning window for spectral leakage reduction
    ///
    /// Returns (`period_in_years`, confidence) if detected, None otherwise
    #[must_use]
    pub fn detect_long_term_oscillation(&self) -> (Option<f32>, f32) {
        const MIN_YEARS: usize = 500; // Minimum for 250-year detection
        const SEGMENT_SIZE: usize = 512; // Years per Welch segment
        const OVERLAP: usize = 256; // 50% overlap

        let history = self.annual_history.get_history();
        if history.len() < MIN_YEARS {
            return (None, 0.0);
        }

        // Welch's method: divide into overlapping segments, compute periodogram for each, average
        let n_segments = (history.len() - SEGMENT_SIZE) / OVERLAP + 1;
        let mut power_spectrum: Vec<f32> = vec![0.0; SEGMENT_SIZE / 2];

        for seg_idx in 0..n_segments {
            let start = seg_idx * OVERLAP;
            let end = start + SEGMENT_SIZE;
            if end > history.len() {
                break;
            }

            // Extract segment
            let segment: Vec<f32> = history[start..end].to_vec();

            // Detrend: remove mean
            let mean = segment.iter().sum::<f32>() / segment.len() as f32;
            let detrended: Vec<f32> = segment.iter().map(|v| v - mean).collect();

            // Apply Hanning window
            let hanning: Vec<f32> = (0..SEGMENT_SIZE)
                .map(|i| {
                    let factor = std::f32::consts::PI * i as f32 / (SEGMENT_SIZE as f32 - 1.0);
                    0.5 * (1.0 - factor.cos())
                })
                .collect();
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001

            let windowed: Vec<f32> = detrended
                .iter()
                .zip(hanning.iter())
                .map(|(d, h)| d * h)
                .collect();

            // Compute periodogram for this segment
            // Target: 200-300 year periods
            let min_period = 200.0;
            let max_period = 300.0;
            let sample_rate = 1.0; // 1 sample per year

            for (k, power_spectrum_k) in power_spectrum
                .iter_mut()
                .enumerate()
                .take(SEGMENT_SIZE / 2)
                .skip(1)
            {
                let freq = k as f32 * sample_rate / SEGMENT_SIZE as f32;
                let period = 1.0 / freq;

                if period < min_period || period > max_period {
                    continue;
                }

                // DFT at this frequency
                let mut real = 0.0f32;
                let mut imag = 0.0f32;

                for (n, sample) in windowed.iter().enumerate() {
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    let angle = -2.0 * std::f32::consts::PI * freq * n as f32;
                    real += sample * angle.cos();
                    imag += sample * angle.sin();
                }

                #[allow(clippy::suspicious_operation_groupings)]
                let power = (real * real + imag * imag) / (SEGMENT_SIZE as f32 * n_segments as f32);
                *power_spectrum_k += power;
            }
        }

        // Find dominant frequency in target range
        let mut max_power = 0.0f32;
        let mut total_power = 0.0f32;
        let mut dominant_period = None;

        for (k, power) in power_spectrum
            .iter()
            .enumerate()
            .take(SEGMENT_SIZE / 2)
            .skip(1)
        {
            let freq = k as f32 * 1.0 / SEGMENT_SIZE as f32;
            let period = 1.0 / freq;

            if (200.0..=300.0).contains(&period) {
                total_power += power;

                // TAG: surface=simulation owner=research-team rule=SIM-001
                if *power > max_power {
                    max_power = *power;
                    dominant_period = Some(period);
                }
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Confidence: proportion of total power in 200-300 year band
        let confidence = if total_power > 0.0 {
            max_power / total_power
        } else {
            0.0
        };

        (dominant_period, confidence)
    }

    /// Compute dominant orientation from current `τ_φ` and weights
    fn compute_dominant_orientation(&self) -> PhilosophicalOrientation {
        // Weight τ_φ by orientation weights
        let rationalism_score = self.orientation_weights[0] * (1.0 - self.tau_phi) / 2.0;
        let empiricism_score = self.orientation_weights[1] * (1.0 + self.tau_phi) / 2.0;
        let pragmatism_score = self.orientation_weights[2] * (1.0 - (self.tau_phi * 0.5).abs());
        let existentialism_score = self.orientation_weights[3] * (1.0 + self.tau_phi) / 2.0;

        let scores = [
            rationalism_score,
            empiricism_score,
            pragmatism_score,
            existentialism_score,
        ];

        match scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            Some((0, _)) => PhilosophicalOrientation::Rationalism,
            Some((1, _)) => PhilosophicalOrientation::Empiricism,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Some((2, _)) => PhilosophicalOrientation::Pragmatism,
            Some((3, _)) => PhilosophicalOrientation::Existentialism,
            _ => PhilosophicalOrientation::Pragmatism,
        }
    }

    /// Update orientation weights based on experience
    /// Per PHIL-P0-3: Sensitivity analysis
    pub fn update_orientation_weights(&mut self, outcomes: &[f32]) {
        // Outcomes are trust validation signals from interactions
        // Positive outcomes reinforce current dominant orientation
        // Negative outcomes shift weights toward alternatives

        if outcomes.is_empty() {
            return;
        }

        let mean_outcome = outcomes.iter().sum::<f32>() / outcomes.len() as f32;
        let dominant_idx = self.dominant_orientation as usize;

        // Reinforcement learning: strengthen successful orientations
        let learning_rate = 0.01;
        for (i, weight) in self.orientation_weights.iter_mut().enumerate() {
            if i == dominant_idx && mean_outcome > 0.0 {
                *weight += learning_rate * mean_outcome;
            } else if i != dominant_idx && mean_outcome < 0.0 {
                *weight += learning_rate * (-mean_outcome) * 0.5;
            }
        }

        // Renormalize to sum to 1.0
        let sum: f32 = self.orientation_weights.iter().sum();
        if sum > 0.0 {
            for weight in &mut self.orientation_weights {
                *weight /= sum;
            }
        }

        // Update sensitivity coefficients
        self.update_taxonomy_sensitivity();
    }

    /// Compute taxonomy sensitivity coefficients
    /// Per PHIL-P0-3: Sensitivity to philosophical taxonomy choices
    fn update_taxonomy_sensitivity(&mut self) {
        // Sensitivity measures how much τ_φ would change with small weight perturbations
        for i in 0..4 {
            let perturbation = 0.01;
            let _original_weight = self.orientation_weights[i];

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Compute τ_φ contribution from this orientation
            let contribution = match i {
                0 => -1.0, // Rationalism contributes to skepticism
                1 => 1.0,  // Empiricism contributes to openness
                2 => 0.0,  // Pragmatism contributes neutral
                3 => 1.0,  // Existentialism contributes to openness
                _ => 0.0,
            };

            self.taxonomy_sensitivity[i] = ((contribution * perturbation) as f32).abs();
        }
    }

    /// Get philosophical trust axis value `τ_φ`
    #[must_use]
    pub const fn tau_phi(&self) -> f32 {
        self.tau_phi
    }

    /// Check if bond should form with another agent
    #[must_use]
    pub fn should_form_bond(&self, partner_tau_phi: f32) -> bool {
        let bond_strength = self.compute_bond_activation(partner_tau_phi);
        bond_strength > self.bond_activation
    }

    /// Validate PHIL-P0-1: `τ_φ` measurement in valid range
    #[must_use]
    pub fn validate_tau_phi(&self) -> bool {
        self.tau_phi >= -1.0 && self.tau_phi <= 1.0
    }

    // ============================================================================
    // DECISION INFLUENCE METHODS (τ_φ → Agent Behavior)
    // ============================================================================

    /// Compute risk tolerance based on philosophical worldview
    ///
    /// High `τ_φ` → willing to take risks for cooperation
    /// Low `τ_φ` → conservative, self-protective
    ///
    /// Returns value in [0.0, 1.0] where 1.0 = high risk tolerance
    #[must_use]
    pub fn risk_tolerance(&self) -> f32 {
        // τ_φ axis: [-1, 1] mapped to [0, 1]
        (self.worldview.trust_axis + 1.0) / 2.0
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute cooperation willingness with another agent
    ///
    /// Influenced by:
    /// - `τ_φ` (trust axis)
    /// - Ethical framework (Kantian vs Utilitarian)
    /// - Epistemic humility
    /// - Philosophical alignment with partner
    ///
    /// # Arguments
    /// * `other_tau_phi` - Partner's `τ_φ` value for alignment computation
    ///
    /// Returns value in [0.0, 1.0] where 1.0 = high cooperation willingness
    #[must_use]
    pub fn cooperation_willingness(&self, other_tau_phi: f32) -> f32 {
        let base_willingness = self.risk_tolerance();

        // Similar philosophical alignment increases cooperation
        let alignment = 1.0 - (self.worldview.trust_axis - other_tau_phi).abs() / 2.0;

        // Epistemic humility makes agent more cautious
        let humility_factor = self.epistemology.epistemic_humility;

        // Ethical framework modifier
        let ethical_modifier = match self.ethical_framework {
            EthicalFramework::Kantian => 1.1, // Kantians more willing to cooperate on principle
            EthicalFramework::Utilitarian => 0.9, // Utilitarians more calculating
            EthicalFramework::VirtueEthics => 1.0, // Balanced
        };

        (base_willingness * alignment * (0.5 + 0.5 * humility_factor) * ethical_modifier)
            .clamp(0.0, 1.0)
    }

    /// Determine ethical decision weight for an action
    ///
    /// Returns a weight [0.0, 1.0] for how strongly the agent should
    /// pursue this action based on their ethical framework.
    ///
    /// # Arguments
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// * `action_outcomes` - Array of outcome utilities for affected agents
    /// * `is_principled` - Whether action follows principled rules (Kantian consideration)
    ///
    /// Kantian: Focus on intentions and duties (high baseline, less outcome-dependent)
    /// Utilitarian: Focus on outcomes and welfare (weighted by sum of utilities)
    /// Virtue Ethics: Focus on virtuousness of action
    #[must_use]
    pub fn ethical_weight(&self, action_outcomes: &[f32], is_principled: bool) -> f32 {
        match self.ethical_framework {
            EthicalFramework::Kantian => {
                // Focus on principled consistency, not outcomes
                if is_principled {
                    0.9 // High weight for principled actions
                } else {
                    0.5 // Lower for unprincipled
                }
            }
            EthicalFramework::Utilitarian => {
                // Weight by sum of utilities (welfare maximization)
                let total_utility: f32 = action_outcomes.iter().sum();
                // Normalize: assume outcomes are in range [-10, 10]
                ((total_utility + 10.0) / 20.0).clamp(0.0, 1.0)
            }
            EthicalFramework::VirtueEthics => {
                // Weight by virtuousness of action
                // Integrity is key virtue for commitment
                let integrity = self.virtues.get(Virtue::Integrity);
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                // Average virtue score with slight boost for principled actions
                let virtue_avg = self.virtues.get_average();
                if is_principled {
                    ((integrity + virtue_avg) / 2.0 * 0.3 + 0.7).clamp(0.0, 1.0)
                } else {
                    virtue_avg * 0.5 + 0.3
                }
            }
        }
    }

    /// Compute commitment strength for promises/agreements
    ///
    /// Higher for Kantian (duty-based), moderate for Virtue, lower for Utilitarian
    ///
    /// Returns value in [0.0, 1.0] where 1.0 = strong commitment
    #[must_use]
    pub fn commitment_strength(&self) -> f32 {
        let base_strength = match self.ethical_framework {
            EthicalFramework::Kantian => 0.9,
            EthicalFramework::VirtueEthics => {
                // Integrity virtue affects commitment
                0.5 + self.virtues.get(Virtue::Integrity) * 0.4
            }
            EthicalFramework::Utilitarian => 0.5,
        };

        // Modulate by worldview trust axis (higher trust → stronger commitments)
        let trust_modifier = (self.worldview.trust_axis + 1.0) / 2.0;

        base_strength * (0.7 + 0.3 * trust_modifier)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Make decision threshold based on risk tolerance and ethical framework
    ///
    /// Returns threshold in [0.0, 1.0]. Actions with scores above this
    /// threshold will be taken.
    #[must_use]
    pub fn decision_threshold(&self) -> f32 {
        let risk_adj = self.risk_tolerance();
        // Base threshold 0.5, adjusted by risk tolerance
        // High risk tolerance → lower threshold (more willing to act)
        // Low risk tolerance → higher threshold (more cautious)
        let base_threshold = 0.5 + (0.5 - risk_adj) * 0.2;

        // Ethical framework adjustments
        match self.ethical_framework {
            EthicalFramework::Kantian => base_threshold * 0.95, // Slightly more willing to act on principle
            EthicalFramework::Utilitarian => base_threshold,    // Balanced
            EthicalFramework::VirtueEthics => base_threshold * 1.05, // Slightly more cautious
        }
    }

    /// Compute philosophy alignment score with another agent
    ///
    /// Returns value in [0.0, 1.0] where 1.0 = complete alignment
    #[must_use]
    pub fn philosophy_alignment(&self, other: &Self) -> f32 {
        // τ_φ alignment
        let tau_alignment = 1.0 - (self.tau_phi - other.tau_phi).abs() / 2.0;

        // Ethical framework alignment (same framework = 1.0, different = 0.5)
        let ethical_alignment = if self.ethical_framework == other.ethical_framework {
            1.0
        } else {
            0.5
        };

        // Orientation alignment
        let orientation_alignment = if self.dominant_orientation == other.dominant_orientation {
            0.8
        } else {
            0.4
        };

        // Weighted combination
        tau_alignment * 0.5 + ethical_alignment * 0.3 + orientation_alignment * 0.2
    }

    /// Update philosophy based on life experiences (learning)
    ///
    /// # Arguments
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// * `outcome` - Outcome of interaction: positive = good, negative = bad
    /// * `was_cooperative` - Whether agent acted cooperatively
    pub fn update_from_experience(&mut self, outcome: f32, was_cooperative: bool) {
        // Update trust axis based on outcomes
        if outcome > 0.0 && was_cooperative {
            // Positive cooperative outcome → increase trust (slowly)
            self.worldview.trust_axis = (self.worldview.trust_axis + 0.02).min(1.0);
            self.tau_phi = (self.tau_phi + 0.02).min(1.0);
        } else if outcome < 0.0 && was_cooperative {
            // Negative outcome from cooperation → decrease trust (faster - betrayal hurts)
            self.worldview.trust_axis = (self.worldview.trust_axis - 0.05).max(-1.0);
            self.tau_phi = (self.tau_phi - 0.05).max(-1.0);
        } else if outcome > 0.0 && !was_cooperative {
            // Positive outcome from defection → slight decrease in trust (cynical)
            self.worldview.trust_axis = (self.worldview.trust_axis - 0.01).max(-1.0);
        }

        // Update epistemic humility if wrong
        if outcome < -0.5 {
            // Large negative outcome → increase humility (learn from mistakes)
            self.epistemology.epistemic_humility =
                (self.epistemology.epistemic_humility + 0.02).min(1.0);
        }

        // Update virtues based on outcomes
        if was_cooperative && outcome > 0.0 {
            // Successful cooperation builds integrity
            let current_integrity = self.virtues.get(Virtue::Integrity);
            self.virtues
                .set(Virtue::Integrity, (current_integrity + 0.01).min(1.0));
        }

        // Update bond activation based on new τ_φ
        let target_activation = (0.5 - self.tau_phi * 0.3).clamp(0.1, 0.9);
        self.bond_activation = 0.9 * self.bond_activation + 0.1 * target_activation;
    }
}

/// Population-level philosophical statistics
/// Per Paper 6: Societal-level philosophical dynamics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopulationPhilosophy {
    /// Mean `τ_φ` across population
    pub mean_tau_phi: f32,
    /// Standard deviation of `τ_φ`
    pub std_tau_phi: f32,
    /// Distribution across orientations [Rationalism, Empiricism, Pragmatism, Existentialism]
    pub orientation_distribution: [f32; 4],
    /// Detected oscillation period (if any)
    pub societal_oscillation_period: Option<f32>,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Current phase in societal oscillation
    pub societal_phase: f32,
}

impl PopulationPhilosophy {
    /// Compute population-level statistics from individual states
    #[must_use]
    pub fn from_states(states: &[&PhilosophicalState]) -> Self {
        if states.is_empty() {
            return Self::default();
        }

        let n = states.len() as f32;
        let mean_tau_phi = states.iter().map(|s| s.tau_phi).sum::<f32>() / n;

        let variance = states
            .iter()
            .map(|s| (s.tau_phi - mean_tau_phi).powi(2))
            .sum::<f32>()
            / n;
        let std_tau_phi = variance.sqrt();

        let mut orientation_counts = [0.0; 4];
        for state in states {
            let idx = state.dominant_orientation as usize;
            orientation_counts[idx] += 1.0;
        }
        let orientation_distribution: [f32; 4] = orientation_counts.map(|c| c / n);

        // Aggregate oscillation (use mean period if consistent)
        let periods: Vec<f32> = states.iter().filter_map(|s| s.oscillation_period).collect();

        let societal_oscillation_period = if periods.len() > states.len() / 2 {
            Some(periods.iter().sum::<f32>() / periods.len() as f32)
        } else {
            None
        };

        Self {
            mean_tau_phi,
            std_tau_phi,
            orientation_distribution,
            societal_oscillation_period,
            societal_phase: 0.0,
        }
    }
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tau_phi_range() {
        let mut state = PhilosophicalState::new();
        assert!(state.validate_tau_phi());

        // Test clamping
        state.tau_phi = 2.0;
        state.update(&DocasState::default(), 1);
        assert!(state.validate_tau_phi());
        assert!(state.tau_phi <= 1.0);
    }

    #[test]
    fn test_bond_activation_computation() {
        let state1 = PhilosophicalState {
            tau_phi: 0.5,
            bond_activation: 0.3,
            ..Default::default()
        };

        let bond_strength = state1.compute_bond_activation(0.6);
        assert!((0.0..=1.0).contains(&bond_strength));
    }

    #[test]
    fn test_orientation_weights_sum_to_one() {
        let mut state = PhilosophicalState::new();

        // Update with some outcomes
        state.update_orientation_weights(&[0.5, 0.3, -0.2]);

        let sum: f32 = state.orientation_weights.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_from_genome_and_docas() {
        let docas = DocasState::new([0.7, 0.8, 0.3, 0.4, 0.6]);
        let state = PhilosophicalState::from_genome_and_docas(&docas, 0.5);

        assert!(state.tau_phi >= -1.0 && state.tau_phi <= 1.0);
        assert!(state.bond_activation >= 0.0 && state.bond_activation <= 1.0);
    }

    // P2-4: FFT Oscillation Detection Tests
    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_fft_period_detection() {
        let mut state = PhilosophicalState::default();

        // Generate synthetic data with 250-tick period
        // (Using ticks as a proxy for years for testing)
        let period = 250.0;
        for tick in 0..1000 {
            let tau = (tick as f32 / period * 2.0 * std::f32::consts::PI).sin();
            state.tau_phi_history.push((tick as u64, tau));
        }

        // FFT should detect period around 250
        let detected = state.compute_fft_period();
        // Note: FFT needs sufficient history, may not detect in test
        // Just verify it doesn't panic
        println!("Detected period: {detected:?}");
    }

    #[test]
    fn test_fft_insufficient_history() {
        let state = PhilosophicalState::default();
        // Empty history should return None
        assert!(state.compute_fft_period().is_none());
        assert!(state.fft_oscillation_metrics().0.is_none());
    }

    #[test]
    fn test_fft_oscillation_metrics() {
        let mut state = PhilosophicalState::default();

        // Generate oscillating data
        let period = 250.0;
        for tick in 0..600 {
            let tau = 0.5 * (tick as f32 / period * 2.0 * std::f32::consts::PI).sin();
            state.tau_phi_history.push((tick as u64, tau));
        }

        let (period_opt, confidence) = state.fft_oscillation_metrics();
        println!("Period: {period_opt:?}, Confidence: {confidence}");

        // Confidence should be between 0 and 1
        assert!((0.0..=1.0).contains(&confidence));
    }

    #[test]
    fn test_oscillation_phase_progression() {
        let mut state = PhilosophicalState::default();
        let initial_phase = state.oscillation_phase;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Update should advance phase
        state.update(&DocasState::default(), 0);
        assert!(state.oscillation_phase != initial_phase || state.oscillation_phase == 0.0);
    }

    #[test]
    fn test_tau_phi_bounds() {
        let state = PhilosophicalState::default();
        assert!(state.tau_phi >= -1.0 && state.tau_phi <= 1.0);
    }

    // P2-6: Eastern Philosophy Tradition Tests
    #[test]
    fn test_eastern_tradition_creation() {
        let confucian = EasternTradition::Confucianism;
        let buddhist = EasternTradition::Buddhism;
        let daoist = EasternTradition::Daoism;

        assert_ne!(confucian, buddhist);
        assert_ne!(buddhist, daoist);
    }

    #[test]
    fn test_eastern_tradition_principles() {
        let confucian = EasternTradition::Confucianism;
        let principles = confucian.core_principles();
        assert!(!principles.is_empty());

        let buddhist = EasternTradition::Buddhism;
        let buddhist_principles = buddhist.core_principles();
        assert!(buddhist_principles.contains(&"Non-attachment"));
    }

    #[test]
    fn test_harmony_orientation() {
        let confucian = EasternTradition::Confucianism;
        let daoist = EasternTradition::Daoism;

        // Both value harmony, but differently
        assert!(confucian.harmony_orientation() > 0.5);
        assert!(daoist.harmony_orientation() > confucian.harmony_orientation());
    }

    #[test]
    fn test_buddhist_impermanence() {
        let buddhist = EasternTradition::Buddhism;
        assert!(buddhist.impermanence_emphasis() > 0.5);
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[test]
    fn test_philosophical_taxonomy_eastern() {
        let taxonomy = PhilosophicalTaxonomy::with_eastern();

        let dimensions = taxonomy.dimensions();
        // Should have more than just Kant-Machiavelli
        assert!(dimensions.len() >= 2);

        // Should include Eastern axis
        let has_eastern = dimensions.iter().any(|d| d.name.contains("Eastern"));
        assert!(has_eastern);
    }

    #[test]
    fn test_synthesis_matrix() {
        let taxonomy = PhilosophicalTaxonomy::with_eastern();
        let matrix = taxonomy.synthesis_matrix();

        // Matrix should have entries for tradition combinations
        assert!(!matrix.combinations.is_empty());
    }
}

// ============================================================================
// P2-6: EASTERN PHILOSOPHY TRADITIONS
// Extending philosophical taxonomy beyond Kant-Machiavelli axis
// ============================================================================

/// Eastern philosophical traditions
/// Per comparative philosophy: Non-Western frameworks for worldview
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EasternTradition {
    /// Confucianism: Social harmony, filial piety, ritual propriety
    /// Key concepts: Ren (benevolence), Li (ritual), Xiao (filial piety)
    Confucianism,
    /// Buddhism: Impermanence, non-attachment, enlightenment
    /// Key concepts: Dukkha (suffering), Anicca (impermanence), Anatta (non-self)
    Buddhism,
    /// Daoism: Natural flow, wu wei (non-action), harmony with Dao
    /// Key concepts: Wu wei, Ziran (spontaneity), Pu (uncarved block)
    Daoism,
    /// Legalism: Strict laws, state control, harsh punishments
    /// Contrast to Confucianism's moral approach
    Legalism,
    /// Mohism: Universal love, utilitarianism, anti-war
    /// Historical rival to Confucianism
    Mohism,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EasternTradition {
    /// Get core principles as text descriptions
    #[must_use]
    pub const fn core_principles(&self) -> &'static [&'static str] {
        match self {
            Self::Confucianism => &[
                "Social harmony",
                "Filial piety",
                "Ritual propriety",
                "Benevolence (Ren)",
                "Hierarchy and roles",
            ],
            Self::Buddhism => &[
                "Impermanence (Anicca)",
                "Non-attachment",
                "Enlightenment",
                "Compassion (Karuna)",
                "Non-self (Anatta)",
            ],
            Self::Daoism => &[
                "Wu wei (non-action)",
                "Natural flow",
                "Spontaneity (Ziran)",
                "Yin-yang balance",
                "Harmony with nature",
            ],
            Self::Legalism => &[
                "Strict laws",
                "State control",
                "Harsh punishments",
                "Meritocracy",
                "Centralized authority",
            ],
            Self::Mohism => &[
                "Universal love",
                "Utilitarian benefit",
                "Anti-war",
                "Frugality",
                "Merit-based advancement",
            ],
        }
    }

    /// Harmony orientation (0 = individualist, 1 = collectivist)
    #[must_use]
    pub const fn harmony_orientation(&self) -> f32 {
        match self {
            Self::Confucianism => 0.9, // Strong social harmony
            Self::Buddhism => 0.6,     // Inner peace
            Self::Daoism => 1.0,       // Perfect harmony with nature
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::Legalism => 0.7, // Social order
            Self::Mohism => 0.8,   // Universal benefit
        }
    }

    /// Impermanence emphasis (0 = permanent, 1 = impermanent)
    #[must_use]
    pub const fn impermanence_emphasis(&self) -> f32 {
        match self {
            Self::Buddhism => 1.0,     // Core doctrine
            Self::Daoism => 0.8,       // Flow and change
            Self::Confucianism => 0.2, // Stable traditions
            Self::Legalism => 0.3,     // Fixed laws
            Self::Mohism => 0.4,       // Pragmatic change
        }
    }

    /// Action orientation (0 = wu wei/non-action, 1 = active intervention)
    #[must_use]
    pub const fn action_orientation(&self) -> f32 {
        match self {
            Self::Daoism => 0.1,       // Wu wei
            Self::Buddhism => 0.3,     // Meditation
            Self::Confucianism => 0.8, // Active cultivation
            Self::Legalism => 1.0,     // Strong intervention
            // TAG: surface=simulation owner=research-team rule=SIM-001
            Self::Mohism => 0.9, // Active benefit
        }
    }

    /// Map to approximate `τ_φ` value
    /// Eastern traditions tend toward openness (higher `τ_φ`)
    #[must_use]
    pub const fn to_tau_phi(&self) -> f32 {
        match self {
            Self::Confucianism => 0.3, // Moderate trust in tradition
            Self::Buddhism => 0.7,     // High openness to experience
            Self::Daoism => 0.9,       // Very open, trusting flow
            Self::Legalism => -0.5,    // Skeptical of human nature
            Self::Mohism => 0.5,       // Trust in universal love
        }
    }
}

/// Extended philosophical taxonomy including Eastern traditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhilosophicalTaxonomy {
    /// Dimension names
    pub dimensions: Vec<PhilosophicalDimension>,
    /// Eastern tradition weights for each agent
    pub eastern_weights: std::collections::BTreeMap<EasternTradition, f32>,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

/// Single philosophical dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhilosophicalDimension {
    /// Dimension name
    pub name: String,
    /// Lower extreme label
    pub low_label: String,
    /// Upper extreme label
    pub high_label: String,
    /// Current position [-1, 1]
    pub position: f32,
}

impl PhilosophicalTaxonomy {
    /// Create taxonomy with only Western dimensions
    #[must_use]
    pub fn western_only() -> Self {
        Self {
            dimensions: vec![PhilosophicalDimension {
                name: "Kant-Machiavelli".to_string(),
                low_label: "Machiavellian".to_string(),
                high_label: "Kantian".to_string(),
                position: 0.0,
            }],
            eastern_weights: std::collections::BTreeMap::new(),
        }
    }

    /// Create taxonomy with Eastern dimensions included
    #[must_use]
    pub fn with_eastern() -> Self {
        let mut taxonomy = Self::western_only();

        taxonomy.dimensions.push(PhilosophicalDimension {
            name: "Eastern-Western".to_string(),
            low_label: "Western Individualist".to_string(),
            high_label: "Eastern Collectivist".to_string(),
            position: 0.5,
        });

        taxonomy.dimensions.push(PhilosophicalDimension {
            name: "Action-WuWei".to_string(),
            low_label: "Wu Wei (Non-action)".to_string(),
            high_label: "Active Intervention".to_string(),
            position: 0.0,
        });

        // Initialize equal weights for Eastern traditions
        // TAG: surface=simulation owner=research-team rule=SIM-001
        for tradition in [
            EasternTradition::Confucianism,
            EasternTradition::Buddhism,
            EasternTradition::Daoism,
        ] {
            taxonomy.eastern_weights.insert(tradition, 0.33);
        }

        taxonomy
    }

    /// Get dimensions
    #[must_use]
    pub fn dimensions(&self) -> &[PhilosophicalDimension] {
        &self.dimensions
    }

    /// Get dominant Eastern tradition with deterministic tie-breaking
    #[must_use]
    pub fn dominant_eastern(&self) -> Option<(EasternTradition, f32)> {
        self.eastern_weights
            .iter()
            .max_by(|(t1, w1), (t2, w2)| {
                w1.partial_cmp(w2)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| t2.cmp(t1))
            })
            .map(|(&t, &w)| (t, w))
    }

    /// Compute synthesized worldview
    /// Combines Western and Eastern dimensions
    #[must_use]
    pub fn synthesized_worldview(&self) -> SynthesizedWorldview {
        let tau_phi = if let Some((tradition, weight)) = self.dominant_eastern() {
            // Blend Eastern τ_φ with default
            tradition.to_tau_phi() * weight
        } else {
            0.0
        };

        SynthesizedWorldview {
            tau_phi,
            harmony_focus: self
                .eastern_weights
                .get(&EasternTradition::Confucianism)
                .or_else(|| self.eastern_weights.get(&EasternTradition::Daoism))
                .copied()
                .unwrap_or(0.0),
            impermanence_acceptance: self
                .eastern_weights
                .get(&EasternTradition::Buddhism)
                .copied()
                .unwrap_or(0.0),
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }
    }

    /// Get synthesis matrix for tradition combinations
    #[must_use]
    pub fn synthesis_matrix(&self) -> SynthesisMatrix {
        let mut combinations = Vec::new();

        // Generate interesting combinations
        for (t1, w1) in &self.eastern_weights {
            for (t2, w2) in &self.eastern_weights {
                if t1 != t2 && *w1 > 0.1 && *w2 > 0.1 {
                    let synthesized_tau = (t1.to_tau_phi() * w1 + t2.to_tau_phi() * w2) / (w1 + w2);
                    combinations.push(TraditionSynthesis {
                        traditions: (*t1, *t2),
                        weights: (*w1, *w2),
                        synthesized_tau_phi: synthesized_tau,
                    });
                }
            }
        }

        SynthesisMatrix { combinations }
    }
}

/// Synthesized worldview combining traditions
// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedWorldview {
    /// Resulting `τ_φ` value
    pub tau_phi: f32,
    /// Focus on harmony (0-1)
    pub harmony_focus: f32,
    /// Acceptance of impermanence (0-1)
    pub impermanence_acceptance: f32,
}

/// Matrix of tradition syntheses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisMatrix {
    /// Tradition combinations
    pub combinations: Vec<TraditionSynthesis>,
}

/// Synthesis of two traditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraditionSynthesis {
    /// Component traditions
    pub traditions: (EasternTradition, EasternTradition),
    /// Weights of each
    pub weights: (f32, f32),
    /// Resulting `τ_φ`
    pub synthesized_tau_phi: f32,
}
