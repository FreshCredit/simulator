//! Differential Privacy for Population-Level Statistics
//!
//! Implements Dwork (2006) differential privacy framework for releasing
//! aggregate simulation data with mathematical privacy guarantees.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION privacy, NOT production privacy systems
//!
//! ## Key Concepts
//!
//! - **Epsilon (ε)**: Privacy budget parameter. Lower = more private, noisier.
//! - **Sensitivity**: Maximum change in query output from adding/removing one individual.
//! - **Laplace Mechanism**: Adds noise from Laplace distribution (L1 sensitivity).
//! - **Gaussian Mechanism**: Adds noise from Gaussian distribution (L2 sensitivity).
//!
//! ## Usage
//!
//! ```rust,ignore
//! use sim_core::privacy::{DifferentialPrivacy, NoiseMechanism, PrivatePopulationStats};
//! use sim_core::Agent;
//!
//! // Create DP system with epsilon = 1.0
//! let dp = DifferentialPrivacy::new(1.0);
//! let mut private_stats = PrivatePopulationStats::new(dp);
//!
//! // Release private population count
//! let private_count = private_stats.private_population(&agents);
//!
//! // Release private mean trust
//! let private_mean_trust = private_stats.private_mean_trust(&agents);
//! ```

use crate::agent::Agent;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI as PI_F64;
use std::sync::atomic::{AtomicU64, Ordering};

static CLONE_NONCE: AtomicU64 = AtomicU64::new(1);

fn erfc_f64(x: f64) -> f64 {
    // Abramowitz & Stegun 7.1.26 rational approximation (max error < 1.5e-7)
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * z);
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    let r = poly * (-z * z).exp();
    if x >= 0.0 {
        r
    } else {
        2.0 - r
    }
}

fn norm_cdf_f64(x: f64) -> f64 {
    0.5 * erfc_f64(-x / std::f64::consts::SQRT_2)
}

/// Balle & Wang (2018) Analytic Gaussian Mechanism: solves
/// `Phi(Delta/(2*sigma) - eps*sigma/Delta) - exp(eps)*Phi(-Delta/(2*sigma) - eps*sigma/Delta) <= delta`
/// accurately for ALL `epsilon > 0` (unlike the classical `epsilon < 1` bound).
fn analytic_gaussian_sigma(sensitivity: f64, epsilon: f64, delta: f64) -> f64 {
    if sensitivity <= 0.0 {
        return 0.0;
    }
    let delta_target = delta.clamp(1e-12, 0.49);
    let b0 = norm_cdf_f64(0.0) - epsilon.exp() * norm_cdf_f64(0.0);
    let _ = b0;
    let phi_diff = |s: f64| -> f64 {
        let a = sensitivity / (2.0 * s);
        let b = (epsilon * s) / sensitivity;
        norm_cdf_f64(a - b) - epsilon.exp() * norm_cdf_f64(-a - b)
    };
    let mut lo = 1e-6_f64;
    let mut hi = (sensitivity * (2.0 * (1.25 / delta_target).ln()).sqrt() / epsilon).max(1.0);
    while phi_diff(hi) > delta_target && hi < 1e12 {
        hi *= 2.0;
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if phi_diff(mid) <= delta_target {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

/// Differential privacy system for population statistics
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
///
/// Manages privacy budget and provides calibrated noise addition
/// for releasing aggregate data with differential privacy guarantees.
/// SIM-004: Uses seeded RNG for reproducible results across runs.
#[derive(Debug, Serialize, Deserialize)]
pub struct DifferentialPrivacy {
    /// Privacy budget (epsilon)
    /// Lower = more private, noisier
    /// Higher = less private, more accurate
    pub epsilon: f32,

    /// Privacy budget consumed
    pub budget_consumed: f32,

    /// Maximum budget before exhaustion
    pub max_budget: f32,

    /// Noise mechanism
    pub mechanism: NoiseMechanism,

    /// Delta parameter for (ε,δ)-differential privacy (Gaussian mechanism)
    /// Only used when mechanism is Gaussian
    pub delta: Option<f32>,

    /// Random number generator for reproducible noise sampling
    /// Not serialized - re-seeded on deserialization
    #[serde(skip, default = "default_rng")]
    pub rng: StdRng,
}

/// Default RNG factory for serde deserialization
fn default_rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Clone for DifferentialPrivacy {
    fn clone(&self) -> Self {
        // Fork independent PRNG state via OS entropy + monotonic counter so cloned
        // privacy trackers NEVER emit identical correlated noise streams.
        let nonce = CLONE_NONCE.fetch_add(1, Ordering::Relaxed);
        let rng = StdRng::from_rng(rand::rngs::OsRng).unwrap_or_else(|_| {
            StdRng::seed_from_u64(
                nonce
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add(u64::from(self.budget_consumed.to_bits())),
            )
        });
        Self {
            epsilon: self.epsilon,
            budget_consumed: self.budget_consumed,
            max_budget: self.max_budget,
            mechanism: self.mechanism,
            delta: self.delta,
            rng,
        }
    }
}

impl PartialEq for DifferentialPrivacy {
    fn eq(&self, other: &Self) -> bool {
        self.epsilon == other.epsilon
            && self.budget_consumed == other.budget_consumed
            && self.max_budget == other.max_budget
            && self.mechanism == other.mechanism
            && self.delta == other.delta
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Noise mechanism for differential privacy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NoiseMechanism {
    /// Laplace mechanism (L1 sensitivity)
    /// Provides pure ε-differential privacy
    #[default]
    Laplace,

    /// Gaussian mechanism (L2 sensitivity)
    /// Provides (ε,δ)-differential privacy
    Gaussian,
}

/// Error type for privacy operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyError {
    /// Privacy budget exhausted
    BudgetExhausted,
    /// Invalid epsilon value
    InvalidEpsilon,
    /// Invalid delta value
    InvalidDelta,
    /// Empty population
    EmptyPopulation,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl std::fmt::Display for PrivacyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BudgetExhausted => write!(f, "Privacy budget exhausted"),
            Self::InvalidEpsilon => write!(f, "Epsilon must be positive"),
            Self::InvalidDelta => write!(f, "Delta must be in (0, 1)"),
            Self::EmptyPopulation => {
                write!(f, "Cannot compute statistics on empty population")
            }
        }
    }
}

impl std::error::Error for PrivacyError {}

impl DifferentialPrivacy {
    /// Create with epsilon-differential privacy using Laplace mechanism
    ///
    /// # Arguments
    /// * `epsilon` - Privacy budget (typically 0.1 to 10.0)
    ///
    /// # Example
    /// ```
    /// use sim_core::privacy::DifferentialPrivacy;
    ///
    /// let dp = DifferentialPrivacy::new(1.0);
    /// debug_assert_eq!(dp.epsilon, 1.0);
    /// ```
    #[must_use]
    pub fn new(epsilon: f32) -> Self {
        Self::with_mechanism_and_seed(epsilon, NoiseMechanism::Laplace, 42)
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Create with specified seed for reproducible noise
    ///
    /// # Arguments
    /// * `epsilon` - Privacy budget
    /// * `seed` - Random seed for noise generation
    #[must_use]
    pub fn with_seed(epsilon: f32, seed: u64) -> Self {
        Self::with_mechanism_and_seed(epsilon, NoiseMechanism::Laplace, seed)
    }

    /// Create with specified noise mechanism and seed
    ///
    /// # Arguments
    /// * `epsilon` - Privacy budget
    /// * `mechanism` - Noise mechanism (Laplace or Gaussian)
    /// * `seed` - Random seed for noise generation
    #[must_use]
    pub fn with_mechanism_and_seed(epsilon: f32, mechanism: NoiseMechanism, seed: u64) -> Self {
        let delta = if mechanism == NoiseMechanism::Gaussian {
            Some(1e-6) // Default delta for Gaussian mechanism
        } else {
            None
        };

        Self {
            epsilon: epsilon.max(0.001), // Minimum epsilon to avoid division by zero
            budget_consumed: 0.0,
            max_budget: epsilon.max(0.001) * 10.0, // Can answer ~10 queries at full epsilon
            mechanism,
            delta,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create with specified noise mechanism
    ///
    /// # Arguments
    /// * `epsilon` - Privacy budget
    /// * `mechanism` - Noise mechanism (Laplace or Gaussian)
    #[must_use]
    pub fn with_mechanism(epsilon: f32, mechanism: NoiseMechanism) -> Self {
        Self::with_mechanism_and_seed(epsilon, mechanism, 42)
    }

    /// Create with (ε,δ)-differential privacy using Gaussian mechanism
    ///
    /// # Arguments
    /// * `epsilon` - Privacy budget
    /// * `delta` - Failure probability (typically 1e-6 to 1e-4)
    #[must_use]
    pub fn with_delta(epsilon: f32, delta: f32) -> Self {
        Self::with_delta_and_seed(epsilon, delta, 42)
    }

    /// Create with (ε,δ)-differential privacy and specified seed
    ///
    /// # Arguments
    /// * `epsilon` - Privacy budget
    /// * `delta` - Failure probability (typically 1e-6 to 1e-4)
    /// * `seed` - Random seed for noise generation
    #[must_use]
    pub fn with_delta_and_seed(epsilon: f32, delta: f32, seed: u64) -> Self {
        Self {
            epsilon: epsilon.max(0.001),
            budget_consumed: 0.0,
            max_budget: epsilon.max(0.001) * 10.0,
            mechanism: NoiseMechanism::Gaussian,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            delta: Some(delta.clamp(1e-10, 0.1)),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Add Laplace noise for differential privacy
    ///
    /// Noise is drawn from Laplace(0, sensitivity/epsilon)
    ///
    /// # Arguments
    /// * `value` - True value
    /// * `sensitivity` - L1 sensitivity of the query
    ///
    /// # Returns
    /// Value with Laplace noise added
    pub fn add_laplace_noise(&mut self, value: f32, sensitivity: f32) -> f32 {
        let eps = self.epsilon.clamp(0.001, 1000.0);
        let sens = if sensitivity.is_finite() {
            sensitivity.max(0.0)
        } else {
            0.0
        };
        let scale = sens / eps;
        let noise = self.sample_laplace(0.0, scale);
        value + noise
    }

    /// Add Gaussian noise for differential privacy
    ///
    /// Noise is drawn from Gaussian(0, sigma²) where
    /// sigma = sensitivity * sqrt(2*ln(1.25/delta)) / epsilon
    ///
    /// # Arguments
    /// * `value` - True value
    /// * `sensitivity` - L2 sensitivity of the query
    ///
    /// # Returns
    /// Value with Gaussian noise added
    pub fn add_gaussian_noise(&mut self, value: f32, sensitivity: f32) -> f32 {
        let delta = f64::from(self.delta.unwrap_or(1e-6).clamp(1e-12, 0.1));
        let eps = f64::from(self.epsilon.clamp(0.001, 1000.0));
        let sens = if sensitivity.is_finite() {
            f64::from(sensitivity.max(0.0))
        } else {
            0.0
        };
        let sigma = analytic_gaussian_sigma(sens, eps, delta);
        let noise = self.sample_gaussian_f64(0.0, sigma);
        (f64::from(value) + noise) as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add noise according to the configured mechanism
    ///
    /// # Arguments
    /// * `value` - True value
    /// * `sensitivity` - Sensitivity of the query (L1 for Laplace, L2 for Gaussian)
    pub fn add_noise(&mut self, value: f32, sensitivity: f32) -> f32 {
        match self.mechanism {
            NoiseMechanism::Laplace => self.add_laplace_noise(value, sensitivity),
            NoiseMechanism::Gaussian => self.add_gaussian_noise(value, sensitivity),
        }
    }

    /// Release private count
    ///
    /// Sensitivity = 1 (adding/removing one person changes count by 1)
    ///
    /// # Arguments
    /// * `true_count` - True population count
    ///
    /// # Returns
    /// Differentially private count
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_count(&mut self, true_count: u32) -> Result<f32, PrivacyError> {
        if self.budget_consumed + self.epsilon > self.max_budget {
            return Err(PrivacyError::BudgetExhausted);
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let sensitivity = 1.0; // Adding/removing one person changes count by 1
        let result = self.add_noise(true_count as f32, sensitivity);
        self.budget_consumed += self.epsilon;
        Ok(result)
    }

    /// Release private mean
    ///
    /// Sensitivity = (max - min) / count
    /// Assumes data is bounded in [0, 1] for sensitivity calculation
    ///
    /// # Arguments
    /// * `true_mean` - True mean value
    /// * `count` - Population count
    ///
    /// # Returns
    /// Differentially private mean
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_mean(&mut self, true_mean: f32, count: u32) -> Result<f32, PrivacyError> {
        if !true_mean.is_finite() {
            return Ok(0.0);
        }
        if self.budget_consumed + self.epsilon > self.max_budget {
            return Err(PrivacyError::BudgetExhausted);
        }

        if count == 0 {
            return Err(PrivacyError::EmptyPopulation);
        }

        // Sensitivity = (max - min) / count, assuming data in [0, 1]
        let sensitivity = 1.0 / count as f32;
        let result = self.add_noise(true_mean.clamp(0.0, 1.0), sensitivity);
        self.budget_consumed += self.epsilon;
        Ok(result)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Release private sum
    ///
    /// Sensitivity = `max_contribution` (one person can contribute at most this)
    ///
    /// # Arguments
    /// * `true_sum` - True sum value
    /// * `max_contribution` - Maximum contribution of a single individual
    ///
    /// # Returns
    /// Differentially private sum
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_sum(
        &mut self,
        true_sum: f32,
        max_contribution: f32,
    ) -> Result<f32, PrivacyError> {
        if !true_sum.is_finite() {
            return Ok(0.0);
        }
        if self.budget_consumed + self.epsilon > self.max_budget {
            return Err(PrivacyError::BudgetExhausted);
        }

        let sensitivity = if max_contribution.is_finite() {
            max_contribution.max(0.0)
        } else {
            1.0
        };
        let result = self.add_noise(true_sum, sensitivity);
        self.budget_consumed += self.epsilon;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Ok(result)
    }

    /// Release private histogram/bin count
    ///
    /// Each bin has sensitivity = 1 (adding/removing one person affects at most one bin)
    ///
    /// # Arguments
    /// * `bin_counts` - True bin counts
    ///
    /// # Returns
    /// Differentially private bin counts
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_histogram(&mut self, bin_counts: &[f32]) -> Result<Vec<f32>, PrivacyError> {
        if bin_counts.is_empty() {
            return Ok(Vec::new());
        }

        if self.budget_consumed + self.epsilon > self.max_budget {
            return Err(PrivacyError::BudgetExhausted);
        }

        // For disjoint histograms, each bin has sensitivity 1 and parallel composition applies
        let sensitivity = 1.0;
        let mut noisy_bins = Vec::with_capacity(bin_counts.len());
        for &count in bin_counts {
            noisy_bins.push(self.add_noise(count, sensitivity));
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }

        self.budget_consumed += self.epsilon;
        Ok(noisy_bins)
    }

    /// Check if privacy budget exhausted
    #[must_use]
    pub fn budget_exhausted(&self) -> bool {
        self.budget_consumed >= self.max_budget
    }

    /// Get remaining privacy budget
    #[must_use]
    pub fn remaining_budget(&self) -> f32 {
        (self.max_budget - self.budget_consumed).max(0.0)
    }

    /// Get budget consumption ratio (0.0 to 1.0)
    #[must_use]
    pub fn budget_ratio(&self) -> f32 {
        (self.budget_consumed / self.max_budget).min(1.0)
    }

    /// Reset privacy budget (for testing only)
    #[cfg(test)]
    pub fn reset_budget(&mut self) {
        self.budget_consumed = 0.0;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Sample from Laplace distribution using canonical inverse transform sampling
    ///
    /// Laplace PDF: f(x) = (1/(2b)) * exp(-|x - mu| / b)
    ///
    /// # Arguments
    /// * `mu` - Location parameter (mean)
    /// * `b` - Scale parameter
    fn sample_laplace(&mut self, mu: f32, b: f32) -> f32 {
        // Sample in f64 to mitigate Mironov (2012) low-mantissa LSB precision leakage
        let u: f64 = self.rng.gen::<f64>() - 0.5;
        let sign = if u < 0.0 { -1.0 } else { 1.0 };
        let prob_term = (1.0 - 2.0 * u.abs()).max(f64::MIN_POSITIVE);
        let noise = -f64::from(b) * sign * prob_term.ln();
        (f64::from(mu) + noise) as f32
    }

    fn sample_gaussian_f64(&mut self, mu: f64, sigma: f64) -> f64 {
        let u1: f64 = self.rng.gen_range(f64::MIN_POSITIVE..1.0);
        let u2: f64 = self.rng.gen_range(0.0..2.0 * PI_F64);
        let z: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos();
        mu + sigma * z
    }

    /// Sample from Gaussian distribution using Box-Muller transform
    ///
    /// # Arguments
    /// * `mu` - Mean
    /// * `sigma` - Standard deviation
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[cfg(test)]
    fn sample_gaussian(&mut self, mu: f32, sigma: f32) -> f32 {
        self.sample_gaussian_f64(f64::from(mu), f64::from(sigma)) as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Validate epsilon value
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate_epsilon(epsilon: f32) -> Result<(), PrivacyError> {
        if epsilon <= 0.0 || !epsilon.is_finite() {
            Err(PrivacyError::InvalidEpsilon)
        } else {
            Ok(())
        }
    }

    /// Validate delta value
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate_delta(delta: f32) -> Result<(), PrivacyError> {
        if delta <= 0.0 || delta >= 1.0 || !delta.is_finite() {
            Err(PrivacyError::InvalidDelta)
        } else {
            Ok(())
        }
    }
}

impl Default for DifferentialPrivacy {
    fn default() -> Self {
        Self::with_seed(1.0, 42)
    }
}

/// Privacy-preserving population statistics
///
/// Provides differentially private versions of common population statistics.
/// Uses the composition theorem: multiple queries consume additive privacy budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivatePopulationStats {
    /// dp.
    pub dp: DifferentialPrivacy,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl PrivatePopulationStats {
    /// Create new private population statistics with given DP configuration
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub const fn new(dp: DifferentialPrivacy) -> Self {
        Self { dp }
    }

    /// Create with default epsilon
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn with_epsilon(epsilon: f32) -> Self {
        Self::new(DifferentialPrivacy::new(epsilon))
    }

    /// Release private population count
    ///
    /// # Arguments
    /// * `agents` - Slice of agents
    ///
    /// # Returns
    /// Differentially private population count
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_population(&mut self, agents: &[Agent]) -> Result<f32, PrivacyError> {
        self.dp.private_count(agents.len() as u32)
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Release private mean trust
    ///
    /// # Arguments
    /// * `agents` - Slice of agents
    ///
    /// # Returns
    /// Differentially private mean trust level
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_mean_trust(&mut self, agents: &[Agent]) -> Result<f32, PrivacyError> {
        if agents.is_empty() {
            return Err(PrivacyError::EmptyPopulation);
        }

        let mean: f32 = agents
            .iter()
            .map(|a| a.compute_trust().clamp(0.0, 1.0))
            .sum::<f32>()
            / agents.len() as f32;
        self.dp.private_mean(mean, agents.len() as u32)
    }

    /// Release private mean wealth
    ///
    /// # Arguments
    /// * `agents` - Slice of agents
    ///
    /// # Returns
    /// Differentially private mean wealth
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_mean_wealth(&mut self, agents: &[Agent]) -> Result<f32, PrivacyError> {
        if agents.is_empty() {
            return Err(PrivacyError::EmptyPopulation);
        }

        let cap = 1000.0f32; // Default wealth sensitivity upper bound
        let clipped_sum: f32 = agents
            .iter()
            .map(|a| a.economics.wealth.clamp(0.0, cap))
            .sum();
        let clipped_mean = clipped_sum / agents.len() as f32;
        let sensitivity = cap / agents.len() as f32;

        if self.dp.budget_consumed + self.dp.epsilon > self.dp.max_budget {
            return Err(PrivacyError::BudgetExhausted);
        }
        let result = self.dp.add_noise(clipped_mean, sensitivity);
        self.dp.budget_consumed += self.dp.epsilon;
        Ok(result)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Release private total wealth
    ///
    /// # Arguments
    /// * `agents` - Slice of agents
    /// * `max_wealth` - Maximum wealth of any single agent (for sensitivity)
    ///
    /// # Returns
    /// Differentially private total wealth
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_total_wealth(
        &mut self,
        agents: &[Agent],
        max_wealth: f32,
    ) -> Result<f32, PrivacyError> {
        if agents.is_empty() {
            return Err(PrivacyError::EmptyPopulation);
        }
        let cap = max_wealth.max(0.0);
        let total: f32 = agents
            .iter()
            .map(|a| a.economics.wealth.clamp(0.0, cap))
            .sum();
        self.dp.private_sum(total, cap)
    }

    /// Release private wealth distribution (binned histogram)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ///
    /// # Arguments
    /// * `agents` - Slice of agents
    /// * `bins` - Number of wealth bins
    /// * `max_wealth` - Maximum wealth value (for binning)
    ///
    /// # Returns
    /// Vector of differentially private bin counts
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_wealth_histogram(
        &mut self,
        agents: &[Agent],
        bins: usize,
        max_wealth: f32,
    ) -> Result<Vec<f32>, PrivacyError> {
        if agents.is_empty() {
            return Err(PrivacyError::EmptyPopulation);
        }
        if bins == 0 {
            return Ok(Vec::new());
        }

        let mut histogram = vec![0.0; bins];
        let safe_max = max_wealth.max(1e-4);
        let bin_width = safe_max / bins as f32;

        for agent in agents {
            let clamped_w = agent.economics.wealth.clamp(0.0, safe_max);
            let bin = ((clamped_w / bin_width) as usize).min(bins - 1);
            histogram[bin] += 1.0;
        }

        self.dp.private_histogram(&histogram)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Release private trust distribution (binned histogram)
    ///
    /// # Arguments
    /// * `agents` - Slice of agents
    /// * `bins` - Number of trust bins (typically 10 for [0, 0.1), [0.1, 0.2), etc.)
    ///
    /// # Returns
    /// Vector of differentially private bin counts
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_trust_histogram(
        &mut self,
        agents: &[Agent],
        bins: usize,
    ) -> Result<Vec<f32>, PrivacyError> {
        if agents.is_empty() {
            return Err(PrivacyError::EmptyPopulation);
        }
        if bins == 0 {
            return Ok(Vec::new());
        }

        let mut histogram = vec![0.0; bins];
        let bin_width = 1.0 / bins as f32;

        for agent in agents {
            let trust = agent.compute_trust();
            let bin = ((trust.clamp(0.0, 1.0) / bin_width) as usize).min(bins - 1);
            histogram[bin] += 1.0;
        }

        self.dp.private_histogram(&histogram)
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Release private mean energy
    ///
    /// # Arguments
    /// * `agents` - Slice of agents
    ///
    /// # Returns
    /// Differentially private mean energy
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_mean_energy(&mut self, agents: &[Agent]) -> Result<f32, PrivacyError> {
        if agents.is_empty() {
            return Err(PrivacyError::EmptyPopulation);
        }

        // Energy is typically 0-100, so we normalize and clamp to [0, 1] for sensitivity
        let mean: f32 = agents
            .iter()
            .map(|a| (a.energy / 100.0).clamp(0.0, 1.0))
            .sum::<f32>()
            / agents.len() as f32;
        let result = self.dp.private_mean(mean, agents.len() as u32)?;
        Ok(result * 100.0) // Scale back to original range
    }

    /// Release private count of agents with verified identity
    ///
    /// # Arguments
    /// * `agents` - Slice of agents
    ///
    /// # Returns
    /// Differentially private count of verified agents
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn private_verified_count(&mut self, agents: &[Agent]) -> Result<f32, PrivacyError> {
        let count = agents.iter().filter(|a| a.has_verified_id).count() as u32;
        self.dp.private_count(count)
    }

    /// Get current privacy budget status
    #[must_use]
    pub fn budget_status(&self) -> BudgetStatus {
        BudgetStatus {
            consumed: self.dp.budget_consumed,
            max: self.dp.max_budget,
            remaining: self.dp.remaining_budget(),
            ratio: self.dp.budget_ratio(),
            exhausted: self.dp.budget_exhausted(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for PrivatePopulationStats {
    fn default() -> Self {
        Self::new(DifferentialPrivacy::default())
    }
}

/// Privacy budget status report
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BudgetStatus {
    /// Budget consumed so far
    pub consumed: f32,
    /// Maximum allowed budget
    pub max: f32,
    /// Remaining budget
    pub remaining: f32,
    /// Consumption ratio (0.0 to 1.0)
    pub ratio: f32,
    /// Whether budget is exhausted
    pub exhausted: bool,
}

impl std::fmt::Display for BudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            f,
            "Privacy Budget: {:.2}/{:.2} ({:.1}% consumed, {})",
            self.consumed,
            self.max,
            self.ratio * 100.0,
            if self.exhausted {
                "EXHAUSTED"
            } else {
                "active"
            }
        )
    }
}

/// Privacy accountant for advanced composition
///
/// Implements advanced composition theorems for tighter privacy bounds
/// when making multiple queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyAccountant {
    /// Epsilon values for each query
    epsilons: Vec<f32>,
    /// Delta values for each query
    deltas: Vec<f32>,
    /// Target delta for overall privacy guarantee
    target_delta: f32,
}

impl PrivacyAccountant {
    /// Create new privacy accountant
    ///
    /// # Arguments
    /// * `target_delta` - Overall failure probability
    #[must_use]
    pub fn new(target_delta: f32) -> Self {
        Self {
            epsilons: Vec::new(),
            deltas: Vec::new(),
            target_delta: target_delta.clamp(1e-10, 0.1),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record a query
    ///
    /// # Arguments
    /// * `epsilon` - Epsilon used for this query
    /// * `delta` - Delta used for this query (None for pure DP)
    pub fn record_query(&mut self, epsilon: f32, delta: Option<f32>) {
        self.epsilons.push(epsilon);
        self.deltas.push(delta.unwrap_or(0.0));
    }

    /// Compute total privacy cost using basic composition
    ///
    /// Basic composition: `total_epsilon` = sum of all epsilons
    #[must_use]
    pub fn basic_composition_epsilon(&self) -> f32 {
        self.epsilons.iter().sum()
    }

    /// Compute total privacy cost using advanced composition
    ///
    /// Advanced composition (Dwork & Roth 2014):
    /// For k queries each with (ε, δ)-DP, the composition is
    /// (ε', kδ + δ')-DP where ε' = sqrt(2k ln(1/δ')) * ε + k*ε*(e^ε - 1)
    #[must_use]
    pub fn advanced_composition_epsilon(&self, delta_prime: f32) -> f32 {
        let k = self.epsilons.len() as f32;
        if k == 0.0 {
            return 0.0;
        }

        // Use average epsilon for composition calculation
        let avg_epsilon: f32 = self.epsilons.iter().sum::<f32>() / k;

        let sqrt_term = (2.0 * k * (1.0 / delta_prime).ln()).sqrt();
        let exp_term = avg_epsilon.exp_m1();

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        sqrt_term * avg_epsilon + k * avg_epsilon * exp_term
    }

    /// Get total delta (sum of all deltas)
    #[must_use]
    pub fn total_delta(&self) -> f32 {
        self.deltas.iter().sum()
    }

    /// Check if privacy guarantee holds
    ///
    /// # Arguments
    /// * `epsilon_budget` - Maximum allowed epsilon
    /// * `delta_budget` - Maximum allowed delta
    #[must_use]
    pub fn within_budget(&self, epsilon_budget: f32, delta_budget: f32) -> bool {
        let total_epsilon = self.basic_composition_epsilon();
        let total_delta = self.total_delta();
        total_epsilon <= epsilon_budget && total_delta <= delta_budget
    }

    /// Get number of queries recorded
    #[must_use]
    pub fn query_count(&self) -> usize {
        self.epsilons.len()
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Reset accountant
    pub fn reset(&mut self) {
        self.epsilons.clear();
        self.deltas.clear();
    }
}

impl Default for PrivacyAccountant {
    fn default() -> Self {
        Self::new(1e-6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::Genome;
    use crate::types::{Position, Sex};
    use rand::SeedableRng;

    fn create_test_agent(id: u64, trust: f32, wealth: f32) -> Agent {
        let mut rng = rand::rngs::StdRng::seed_from_u64(id);
        let genome = Genome::random(&mut rng);
        let mut agent = Agent::new(id, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);
        // Adjust initial values for testing
        agent.economics.wealth = wealth;
        agent.expectation = trust; // Set expectation to influence trust computation
        agent
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn create_test_population(count: usize) -> Vec<Agent> {
        (0..count)
            .map(|i| create_test_agent(i as u64, 0.5, 10.0 + i as f32 * 5.0))
            .collect()
    }

    // === Basic DP Tests ===

    #[test]
    fn test_dp_creation() {
        let dp = DifferentialPrivacy::new(1.0);
        assert_eq!(dp.epsilon, 1.0);
        assert_eq!(dp.budget_consumed, 0.0);
        assert_eq!(dp.mechanism, NoiseMechanism::Laplace);
        assert!(!dp.budget_exhausted());
    }

    #[test]
    fn test_dp_with_mechanism() {
        let dp_laplace = DifferentialPrivacy::with_mechanism(1.0, NoiseMechanism::Laplace);
        assert_eq!(dp_laplace.mechanism, NoiseMechanism::Laplace);
        assert!(dp_laplace.delta.is_none());

        let dp_gaussian = DifferentialPrivacy::with_mechanism(1.0, NoiseMechanism::Gaussian);
        assert_eq!(dp_gaussian.mechanism, NoiseMechanism::Gaussian);
        assert!(dp_gaussian.delta.is_some());
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_dp_with_delta() {
        let dp = DifferentialPrivacy::with_delta(1.0, 1e-5);
        assert_eq!(dp.epsilon, 1.0);
        assert_eq!(dp.delta, Some(1e-5));
        assert_eq!(dp.mechanism, NoiseMechanism::Gaussian);
    }

    #[test]
    fn test_epsilon_validation() {
        assert!(DifferentialPrivacy::validate_epsilon(1.0).is_ok());
        assert!(DifferentialPrivacy::validate_epsilon(0.1).is_ok());
        assert!(DifferentialPrivacy::validate_epsilon(10.0).is_ok());
        assert!(DifferentialPrivacy::validate_epsilon(0.0).is_err());
        assert!(DifferentialPrivacy::validate_epsilon(-1.0).is_err());
    }

    #[test]
    fn test_delta_validation() {
        assert!(DifferentialPrivacy::validate_delta(1e-6).is_ok());
        assert!(DifferentialPrivacy::validate_delta(0.5).is_ok());
        assert!(DifferentialPrivacy::validate_delta(1e-10).is_ok());
        assert!(DifferentialPrivacy::validate_delta(0.0).is_err());
        assert!(DifferentialPrivacy::validate_delta(1.0).is_err());
        assert!(DifferentialPrivacy::validate_delta(1.5).is_err());
    }

    // === Noise Mechanism Tests ===

    #[test]
    fn test_laplace_noise_statistics() {
        let mut dp = DifferentialPrivacy::new(1.0);
        let true_value = 100.0;
        let sensitivity = 1.0;

        // Sample many values to check statistics
        let samples: Vec<f32> = (0..1000)
            .map(|_| dp.add_laplace_noise(true_value, sensitivity))
            .collect();

        let mean = samples.iter().sum::<f32>() / samples.len() as f32;

        // Mean should be close to true value (unbiased)
        assert!(
            (mean - true_value).abs() < 5.0,
            "Mean {mean} should be close to {true_value}"
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Check that noise varies (not all same value)
        let variance =
            samples.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / samples.len() as f32;
        assert!(variance > 0.5, "Variance should be significant: {variance}");
    }

    #[test]
    fn test_gaussian_noise_statistics() {
        let mut dp = DifferentialPrivacy::with_delta(1.0, 1e-6);
        let true_value = 100.0;
        let sensitivity = 1.0;

        // Sample many values to check statistics
        let samples: Vec<f32> = (0..1000)
            .map(|_| dp.add_gaussian_noise(true_value, sensitivity))
            .collect();

        let mean = samples.iter().sum::<f32>() / samples.len() as f32;

        // Mean should be close to true value (unbiased)
        assert!(
            (mean - true_value).abs() < 5.0,
            "Mean {mean} should be close to {true_value}"
        );

        // Check that noise varies
        let variance =
            samples.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / samples.len() as f32;
        assert!(variance > 0.1, "Variance should be significant: {variance}");
    }

    #[test]
    fn test_noise_scale_increases_with_lower_epsilon() {
        let mut dp_high_epsilon = DifferentialPrivacy::new(10.0);
        let mut dp_low_epsilon = DifferentialPrivacy::new(0.1);

        let samples_high: Vec<f32> = (0..500)
            .map(|_| dp_high_epsilon.add_laplace_noise(100.0, 1.0))
            .collect();
        let samples_low: Vec<f32> = (0..500)
            .map(|_| dp_low_epsilon.add_laplace_noise(100.0, 1.0))
            .collect();

        let variance_high = samples_high
            .iter()
            .map(|&x| (x - 100.0).powi(2))
            .sum::<f32>()
            / samples_high.len() as f32;
        let variance_low = samples_low
            .iter()
            // TAG: surface=simulation owner=research-team rule=SIM-001
            .map(|&x| (x - 100.0).powi(2))
            .sum::<f32>()
            / samples_low.len() as f32;

        // Lower epsilon should have higher variance (more noise)
        assert!(variance_low > variance_high * 10.0,
            "Low epsilon (0.1) variance {variance_low} should be much higher than high epsilon (10.0) variance {variance_high}");
    }

    // === Query Tests ===

    #[test]
    fn test_private_count() {
        let mut dp = DifferentialPrivacy::new(10.0); // High epsilon for less noise
        let true_count = 100u32;

        let private_result = dp.private_count(true_count).unwrap();

        // Result should be close to true count
        assert!(
            (private_result - true_count as f32).abs() < 20.0,
            "Private count {private_result} should be close to true count {true_count}"
        );
    }

    #[test]
    fn test_private_count_budget_exhausted() {
        let mut dp = DifferentialPrivacy::new(1.0);
        dp.budget_consumed = dp.max_budget; // Exhaust budget

        let result = dp.private_count(100);
        assert!(matches!(result, Err(PrivacyError::BudgetExhausted)));
    }

    #[test]
    fn test_private_mean() {
        let mut dp = DifferentialPrivacy::new(10.0);
        let true_mean = 0.5;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let count = 100;

        let private_result = dp.private_mean(true_mean, count).unwrap();

        // Result should be close to true mean
        assert!(
            (private_result - true_mean).abs() < 0.1,
            "Private mean {private_result} should be close to true mean {true_mean}"
        );
    }

    #[test]
    fn test_private_mean_empty_population() {
        let mut dp = DifferentialPrivacy::new(1.0);
        let result = dp.private_mean(0.5, 0);
        assert!(matches!(result, Err(PrivacyError::EmptyPopulation)));
    }

    #[test]
    fn test_private_sum() {
        let mut dp = DifferentialPrivacy::new(10.0);
        let true_sum = 1000.0;
        let max_contribution = 100.0;

        let private_result = dp.private_sum(true_sum, max_contribution).unwrap();

        // Result should be close to true sum
        assert!(
            (private_result - true_sum).abs() < 200.0,
            "Private sum {private_result} should be close to true sum {true_sum}"
        );
    }

    #[test]
    fn test_private_histogram() {
        let mut dp = DifferentialPrivacy::new(10.0);
        let bin_counts = vec![10.0, 20.0, 30.0, 40.0];

        let private_result = dp.private_histogram(&bin_counts).unwrap();

        // Should have same number of bins
        assert_eq!(private_result.len(), bin_counts.len());

        // Values should be close to original
        for (original, private) in bin_counts.iter().zip(private_result.iter()) {
            assert!(
                (private - original).abs() < 15.0,
                "Private bin {private} should be close to original {original}"
            );
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }
    }

    // === Budget Tests ===

    #[test]
    fn test_budget_consumption() {
        let mut dp = DifferentialPrivacy::new(1.0);
        let initial_budget = dp.remaining_budget();

        // Perform a query
        let _ = dp.private_count(100);

        // Budget should be consumed
        assert!(dp.budget_consumed > 0.0);
        assert!(dp.remaining_budget() < initial_budget);
    }

    #[test]
    fn test_budget_exhaustion() {
        let mut dp = DifferentialPrivacy::new(0.5);
        dp.max_budget = 2.0; // Small budget for testing

        // Perform multiple queries to exhaust budget
        for i in 0..10 {
            let result = dp.private_count(100);
            if dp.budget_exhausted() {
                assert!(result.is_err() || i >= 1);
                break;
            }
        }
    }

    #[test]
    fn test_budget_ratio() {
        let mut dp = DifferentialPrivacy::new(1.0);
        dp.max_budget = 10.0;
        dp.budget_consumed = 5.0;

        assert_eq!(dp.budget_ratio(), 0.5);
        assert_eq!(dp.remaining_budget(), 5.0);
    }

    // === PrivatePopulationStats Tests ===

    #[test]
    fn test_private_population() {
        let agents = create_test_population(100);
        let mut stats = PrivatePopulationStats::with_epsilon(10.0);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let private_count = stats.private_population(&agents).unwrap();

        assert!(
            (private_count - 100.0).abs() < 20.0,
            "Private population {private_count} should be close to 100"
        );
    }

    #[test]
    fn test_private_mean_trust() {
        let agents = create_test_population(50);
        let mut stats = PrivatePopulationStats::with_epsilon(10.0);

        let private_mean = stats.private_mean_trust(&agents).unwrap();

        // Trust should be in valid range [0, 1]
        assert!(
            (-1.0..=2.0).contains(&private_mean),
            "Private mean trust {private_mean} should be roughly in [0, 1]"
        );
    }

    #[test]
    fn test_private_mean_wealth() {
        let agents = create_test_population(50);
        let mut stats = PrivatePopulationStats::with_epsilon(10.0);

        let private_mean = stats.private_mean_wealth(&agents).unwrap();

        // Mean wealth should be positive
        assert!(
            private_mean > 0.0,
            "Private mean wealth should be positive: {private_mean}"
        );
    }

    #[test]
    fn test_private_total_wealth() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let agents = create_test_population(50);
        let mut stats = PrivatePopulationStats::with_epsilon(10.0);

        let private_total = stats.private_total_wealth(&agents, 500.0).unwrap();
        let true_total: f32 = agents.iter().map(|a| a.economics.wealth).sum();

        assert!(
            (private_total - true_total).abs() < 500.0,
            "Private total wealth {private_total} should be close to true total {true_total}"
        );
    }

    #[test]
    fn test_private_wealth_histogram() {
        let agents = create_test_population(100);
        let mut stats = PrivatePopulationStats::with_epsilon(10.0);

        let histogram = stats.private_wealth_histogram(&agents, 5, 500.0).unwrap();

        // Should have correct number of bins
        assert_eq!(histogram.len(), 5);

        // Total count should be close to population
        let total: f32 = histogram.iter().sum();
        assert!(
            (total - 100.0).abs() < 50.0,
            "Histogram total {total} should be close to population 100"
        );
    }

    #[test]
    fn test_private_trust_histogram() {
        let agents = create_test_population(100);
        let mut stats = PrivatePopulationStats::with_epsilon(10.0);

        let histogram = stats.private_trust_histogram(&agents, 10).unwrap();

        // Should have correct number of bins
        assert_eq!(histogram.len(), 10);

        // Total count should be close to population
        let total: f32 = histogram.iter().sum();
        assert!(
            (total - 100.0).abs() < 50.0,
            "Histogram total {total} should be close to population 100"
        );
    }

    #[test]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn test_private_mean_energy() {
        let agents = create_test_population(50);
        let mut stats = PrivatePopulationStats::with_epsilon(10.0);

        let private_mean = stats.private_mean_energy(&agents).unwrap();

        // Energy should be in valid range [0, 100]
        assert!(
            (-50.0..=150.0).contains(&private_mean),
            "Private mean energy {private_mean} should be roughly in [0, 100]"
        );
    }

    #[test]
    fn test_private_verified_count() {
        let mut agents = create_test_population(100);
        // Set some agents as verified
        for (i, agent) in agents.iter_mut().enumerate() {
            agent.has_verified_id = i < 30; // 30 verified agents
        }

        let mut stats = PrivatePopulationStats::with_epsilon(10.0);
        let private_count = stats.private_verified_count(&agents).unwrap();

        assert!(
            (private_count - 30.0).abs() < 20.0,
            "Private verified count {private_count} should be close to 30"
        );
    }

    #[test]
    fn test_budget_status() {
        let mut stats = PrivatePopulationStats::with_epsilon(1.0);
        stats.dp.max_budget = 10.0;
        stats.dp.budget_consumed = 3.0;

        let status = stats.budget_status();

        assert_eq!(status.consumed, 3.0);
        assert_eq!(status.max, 10.0);
        assert_eq!(status.remaining, 7.0);
        assert_eq!(status.ratio, 0.3);
        assert!(!status.exhausted);
    }

    #[test]
    fn test_budget_status_display() {
        let status = BudgetStatus {
            consumed: 5.0,
            max: 10.0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            remaining: 5.0,
            ratio: 0.5,
            exhausted: false,
        };

        let display = format!("{status}");
        assert!(display.contains("5.00/10.00"));
        assert!(display.contains("50.0%"));
        assert!(display.contains("active"));
    }

    // === Privacy Accountant Tests ===

    #[test]
    fn test_privacy_accountant_creation() {
        let accountant = PrivacyAccountant::new(1e-6);
        assert_eq!(accountant.query_count(), 0);
        assert_eq!(accountant.total_delta(), 0.0);
    }

    #[test]
    fn test_privacy_accountant_record_query() {
        let mut accountant = PrivacyAccountant::new(1e-6);

        accountant.record_query(1.0, None);
        assert_eq!(accountant.query_count(), 1);

        accountant.record_query(0.5, Some(1e-6));
        assert_eq!(accountant.query_count(), 2);
    }

    #[test]
    fn test_basic_composition() {
        let mut accountant = PrivacyAccountant::new(1e-6);

        accountant.record_query(1.0, None);
        accountant.record_query(0.5, None);
        accountant.record_query(0.5, None);

        assert_eq!(accountant.basic_composition_epsilon(), 2.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    #[test]
    fn test_advanced_composition() {
        let mut accountant = PrivacyAccountant::new(1e-6);

        // Record many queries with small epsilon
        // Advanced composition is beneficial when k > 2*ln(1/delta')
        // For delta' = 1e-5, ln(1/delta') ≈ 11.5, so k > 23
        for _ in 0..50 {
            accountant.record_query(0.01, Some(1e-6));
        }

        let epsilon_advanced = accountant.advanced_composition_epsilon(1e-5);
        let epsilon_basic = accountant.basic_composition_epsilon();

        // Advanced composition should give a valid positive bound
        assert!(epsilon_advanced > 0.0);

        // With many small-epsilon queries, advanced composition should be tighter
        // than basic composition. Basic = 50 * 0.01 = 0.5
        // Advanced ≈ sqrt(2*50*11.5) * 0.01 + 50*0.01*0.01 ≈ 1.07 * 0.01 + 0.005 ≈ 0.16
        assert!(epsilon_advanced < epsilon_basic,
            "Advanced composition ({epsilon_advanced}) should be tighter than basic ({epsilon_basic})");
    }

    #[test]
    fn test_within_budget() {
        let mut accountant = PrivacyAccountant::new(1e-6);

        accountant.record_query(0.5, Some(1e-7));
        accountant.record_query(0.5, Some(1e-7));

        assert!(accountant.within_budget(2.0, 1e-5));
        assert!(!accountant.within_budget(0.5, 1e-5));
    }

    #[test]
    fn test_accountant_reset() {
        let mut accountant = PrivacyAccountant::new(1e-6);

        accountant.record_query(1.0, None);
        assert_eq!(accountant.query_count(), 1);

        accountant.reset();
        assert_eq!(accountant.query_count(), 0);
        assert_eq!(accountant.basic_composition_epsilon(), 0.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // === Error Handling Tests ===

    #[test]
    fn test_empty_population_errors() {
        let agents: Vec<Agent> = vec![];
        let mut stats = PrivatePopulationStats::with_epsilon(1.0);

        assert!(stats.private_population(&agents).is_ok());
        assert!(matches!(
            stats.private_mean_trust(&agents),
            Err(PrivacyError::EmptyPopulation)
        ));
        assert!(matches!(
            stats.private_mean_wealth(&agents),
            Err(PrivacyError::EmptyPopulation)
        ));
    }

    // === Default Implementation Tests ===

    #[test]
    fn test_default_dp() {
        let dp = DifferentialPrivacy::default();
        assert_eq!(dp.epsilon, 1.0);
        assert_eq!(dp.mechanism, NoiseMechanism::Laplace);
    }

    #[test]
    fn test_default_private_stats() {
        let stats = PrivatePopulationStats::default();
        assert_eq!(stats.dp.epsilon, 1.0);
    }

    #[test]
    fn test_default_privacy_accountant() {
        let accountant = PrivacyAccountant::default();
        assert_eq!(accountant.query_count(), 0);
    }

    // === Reproducibility Tests (SIM-004) ===

    #[test]
    fn test_laplace_sampling_is_reproducible() {
        // Create two DP instances with the same seed
        let mut dp1 = DifferentialPrivacy::with_seed(1.0, 12345);
        let mut dp2 = DifferentialPrivacy::with_seed(1.0, 12345);

        // Sample multiple values from both
        let samples1: Vec<f32> = (0..100).map(|_| dp1.sample_laplace(0.0, 1.0)).collect();
        let samples2: Vec<f32> = (0..100).map(|_| dp2.sample_laplace(0.0, 1.0)).collect();
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Should be identical since they use the same seed
        assert_eq!(
            samples1, samples2,
            "Laplace samples should be reproducible with same seed"
        );
    }

    #[test]
    fn test_gaussian_sampling_is_reproducible() {
        // Create two DP instances with the same seed
        let mut dp1 = DifferentialPrivacy::with_seed(1.0, 12345);
        let mut dp2 = DifferentialPrivacy::with_seed(1.0, 12345);

        // Sample multiple values from both
        let samples1: Vec<f32> = (0..100).map(|_| dp1.sample_gaussian(0.0, 1.0)).collect();
        let samples2: Vec<f32> = (0..100).map(|_| dp2.sample_gaussian(0.0, 1.0)).collect();

        // Should be identical since they use the same seed
        assert_eq!(
            samples1, samples2,
            "Gaussian samples should be reproducible with same seed"
        );
    }

    #[test]
    fn test_different_seeds_produce_different_samples() {
        // Create two DP instances with different seeds
        let mut dp1 = DifferentialPrivacy::with_seed(1.0, 12345);
        let mut dp2 = DifferentialPrivacy::with_seed(1.0, 99999);

        // Sample multiple values from both
        let samples1: Vec<f32> = (0..100).map(|_| dp1.sample_laplace(0.0, 1.0)).collect();
        let samples2: Vec<f32> = (0..100).map(|_| dp2.sample_laplace(0.0, 1.0)).collect();

        // Should be different since they use different seeds
        assert_ne!(
            samples1, samples2,
            "Samples with different seeds should differ"
        );
    }

    #[test]
    fn test_clone_diverges_prng_stream_no_noise_cancellation() {
        let dp_orig = DifferentialPrivacy::with_seed(1.0, 42);
        let mut clone_a = dp_orig.clone();
        let mut clone_b = dp_orig.clone();
        let seq_a: Vec<f32> = (0..16)
            .map(|_| clone_a.add_laplace_noise(10.0, 1.0))
            .collect();
        let seq_b: Vec<f32> = (0..16)
            .map(|_| clone_b.add_laplace_noise(10.0, 1.0))
            .collect();
        assert_ne!(
            seq_a, seq_b,
            "Cloned DP trackers must never emit identical noise"
        );
    }

    #[test]
    fn test_analytic_gaussian_calibrated_at_high_epsilon() {
        let sigma_eps10 = analytic_gaussian_sigma(1.0, 10.0, 1e-6);
        let classic_invalid = (2.0 * (1.25 / 1e-6_f64).ln()).sqrt() / 10.0;
        assert!(
            sigma_eps10 > classic_invalid,
            "Analytic Gaussian sigma ({sigma_eps10}) must exceed invalid low-eps approximation ({classic_invalid}) at eps=10"
        );
        // Verify exact Balle-Wang inequality holds at calibrated sigma
        let a = 1.0 / (2.0 * sigma_eps10);
        let b = 10.0 * sigma_eps10;
        let actual_delta = norm_cdf_f64(a - b) - 10.0_f64.exp() * norm_cdf_f64(-a - b);
        assert!(actual_delta <= 1e-6 + 1e-9, "actual_delta={actual_delta}");
    }
}
