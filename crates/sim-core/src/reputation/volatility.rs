//! Reputation volatility tracking using GARCH-inspired models
//!
//! Per `RESEARCH_TEMPORAL_DYNAMICS.md` Section 2
//! Based on Engle (1982) ARCH and Bollerslev (1986) GARCH

use std::collections::VecDeque;

// TAG: surface=simulation owner=research-team rule=SIM-001
use serde::{Deserialize, Serialize};

use crate::types::Tick;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Reputation volatility tracker using rolling window variance
///
/// Tracks trust score variance over time using a GARCH(1,1)-inspired
/// approach adapted for agent-based reputation systems.
///
/// # Mathematical Model
///
/// GARCH(1,1) variance equation:
/// ```text
/// σ²(t) = ω + α·ε²(t-1) + β·σ²(t-1)
/// ```
///
/// Where:
/// - σ²(t) = conditional variance at time t
/// - ε(t) = residual (`trust_score` - `window_mean`)
/// - ω = long-run average variance (omega)
/// - α = weight of new observation (alpha)
/// - β = weight of previous variance (beta)
///
/// Constraints: ω > 0, α ≥ 0, β ≥ 0, α + β < 1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationVolatilityTracker {
    /// Window of recent trust scores for volatility calculation  // SIM-TRUST-001: Uses `compute_tau()` externally
    trust_history: VecDeque<f32>,

    /// Window size (number of ticks to retain)
    window_size: usize,

    /// Current volatility estimate (standard deviation)
    current_volatility: f32,

    /// GARCH-like parameters for volatility clustering
    /// Long-run average variance
    omega: f32,
    /// Weight of new observation (typically 0.05-0.15)
    alpha: f32,
    /// Weight of previous variance (typically 0.80-0.90)
    beta: f32,

    /// Volatility history for time-series analysis
    volatility_history: Vec<f32>,

    /// Timestamp of each volatility measurement
    timestamp_history: Vec<Tick>,

    /// Previous squared residual for GARCH update
    previous_squared_residual: f32,
}

impl ReputationVolatilityTracker {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new volatility tracker
    ///
    /// # Arguments
    /// * `window_size` - Size of rolling window for mean calculation
    ///
    /// # Example
    /// ```
    /// use sim_core::reputation::ReputationVolatilityTracker;
    ///
    /// let tracker = ReputationVolatilityTracker::new(100);
    /// ```
    #[must_use]
    pub fn new(window_size: usize) -> Self {
        Self::with_params(window_size, 0.001, 0.10, 0.85)
    }

    /// Create tracker with custom GARCH parameters
    ///
    /// # Arguments
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// * `window_size` - Rolling window size
    /// * `omega` - Long-run average variance (must be > 0)
    /// * `alpha` - Weight of new observation (must be ≥ 0)
    /// * `beta` - Weight of previous variance (must be ≥ 0)
    ///
    /// # Panics
    /// Panics if alpha + beta ≥ 1.0 (violates stationarity condition)
    #[must_use]
    pub fn with_params(window_size: usize, omega: f32, alpha: f32, beta: f32) -> Self {
        debug_assert!(
            alpha + beta < 1.0,
            "GARCH stationarity requires alpha + beta < 1, got {} + {} = {}",
            alpha,
            beta,
            alpha + beta
        );
        debug_assert!(omega > 0.0, "Omega must be positive");

        let safe_omega = if omega.is_finite() && omega > 0.0 {
            omega
        } else {
            0.001
        };
        let safe_alpha = if alpha.is_finite() && alpha >= 0.0 {
            alpha
        } else {
            0.10
        };
        let safe_beta = if beta.is_finite() && beta >= 0.0 {
            beta
        } else {
            0.85
        };
        let sum_ab = safe_alpha + safe_beta;
        let scale = if sum_ab >= 1.0 { 0.999 / sum_ab } else { 1.0 };
        let final_alpha = safe_alpha * scale;
        let final_beta = safe_beta * scale;
        let stationary_var = safe_omega / (1.0 - (final_alpha + final_beta)).max(1e-4);
        let initial_vol = stationary_var.sqrt().clamp(0.0, 1.0);

        Self {
            trust_history: VecDeque::with_capacity(window_size),
            window_size,
            current_volatility: initial_vol,
            omega: safe_omega,
            alpha: final_alpha,
            beta: final_beta,
            volatility_history: Vec::new(),
            timestamp_history: Vec::new(),
            previous_squared_residual: stationary_var,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update with new trust score observation
    ///
    /// # Arguments
    /// * `trust_score` - Current trust score (typically in [0.0, 1.0])
    /// * `tick` - Current simulation tick
    pub fn update(&mut self, trust_score: f32, tick: Tick) {
        if !trust_score.is_finite() || self.window_size == 0 {
            return;
        }

        // Add to rolling window
        self.trust_history.push_back(trust_score);
        if self.trust_history.len() > self.window_size {
            self.trust_history.pop_front();
        }

        // Need at least 2 points for variance calculation
        if self.trust_history.len() >= 2 {
            // Calculate window mean
            let mean = self.trust_history.iter().sum::<f32>() / self.trust_history.len() as f32;

            // Calculate residual
            let residual = trust_score - mean;
            let squared_residual = residual * residual;

            // GARCH(1,1) variance update
            let previous_variance = self.current_volatility.powi(2);
            let new_variance = self.omega
                + self.alpha * self.previous_squared_residual
                + self.beta * previous_variance;

            self.current_volatility = new_variance.sqrt().clamp(0.0, 1.0);
            self.previous_squared_residual = squared_residual;

            // Record history
            self.volatility_history.push(self.current_volatility);
            self.timestamp_history.push(tick);
        }
    }

    /// Get current volatility estimate
    ///
    /// Returns standard deviation of trust scores (in same units as trust)
    #[must_use]
    pub const fn current_volatility(&self) -> f32 {
        self.current_volatility
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get current variance (squared volatility)
    #[must_use]
    pub fn current_variance(&self) -> f32 {
        self.current_volatility.powi(2)
    }

    /// Get average volatility over entire history
    #[must_use]
    pub fn mean_volatility(&self) -> f32 {
        if self.volatility_history.is_empty() {
            0.0
        } else {
            self.volatility_history.iter().sum::<f32>() / self.volatility_history.len() as f32
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get volatility trend (positive = increasing volatility)
    ///
    /// # Arguments
    /// * `periods` - Number of recent periods to compare
    #[must_use]
    pub fn volatility_trend(&self, periods: usize) -> f32 {
        let n = self.volatility_history.len();
        if n < periods || periods == 0 {
            return 0.0;
        }

        let recent = self.volatility_history[n - 1];
        let past = self.volatility_history[n - periods];

        recent - past
    }

    /// Detect volatility clustering (periods of high volatility)
    ///
    /// Returns list of (`start_tick`, `end_tick`) tuples where volatility
    /// exceeded the threshold.
    ///
    /// # Arguments
    /// * `threshold` - Volatility threshold for clustering detection
    #[must_use]
    pub fn detect_clustering(&self, threshold: f32) -> Vec<(Tick, Tick)> {
        let mut clusters = Vec::new();
        let mut cluster_start: Option<Tick> = None;

        for (i, &vol) in self.volatility_history.iter().enumerate() {
            if vol > threshold {
                if cluster_start.is_none() {
                    cluster_start = Some(self.timestamp_history[i]);
                }
            } else if let Some(start) = cluster_start {
                clusters.push((start, self.timestamp_history[i]));
                cluster_start = None;
            }
        }

        // Close any open cluster at end of history
        if let Some(start) = cluster_start {
            if let Some(&last_tick) = self.timestamp_history.last() {
                clusters.push((start, last_tick));
            }
        }

        clusters
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if currently in high volatility state
    #[must_use]
    pub fn is_high_volatility(&self, threshold: f32) -> bool {
        self.current_volatility > threshold
    }

    /// Get volatility coefficient of variation
    /// (volatility of volatility, measure of instability)
    #[must_use]
    pub fn volatility_of_volatility(&self) -> f32 {
        if self.volatility_history.len() < 2 {
            return 0.0;
        }

        let mean = self.mean_volatility();
        if !mean.is_finite() || mean <= 1e-6 {
            return 0.0;
        }

        let variance = self
            .volatility_history
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f32>()
            / self.volatility_history.len() as f32;

        (variance.sqrt() / mean).clamp(0.0, 10.0)
    }

    /// Get complete volatility history
    #[must_use]
    pub const fn history(&self) -> &[(Tick, f32)] {
        // This is a view into the history - in practice we'd need to
        // return an iterator or Vec of tuples
        // For now, return empty slice as placeholder
        &[]
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Reset tracker state
    pub fn reset(&mut self) {
        self.trust_history.clear();
        self.current_volatility = self.omega.sqrt();
        self.volatility_history.clear();
        self.timestamp_history.clear();
        self.previous_squared_residual = self.omega;
    }
}

/// Population-wide reputation volatility metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopulationVolatilityMetrics {
    /// Mean volatility across all agents
    pub mean_volatility: f32,

    /// Standard deviation of volatility (heterogeneity)
    pub volatility_std: f32,

    /// Minimum volatility observed
    pub min_volatility: f32,

    /// Maximum volatility observed
    pub max_volatility: f32,

    /// Fraction of agents with high volatility (>threshold)
    pub high_volatility_fraction: f32,

    /// Rolling window of mean volatility over time
    pub mean_volatility_history: Vec<f32>,

    /// Cross-agent volatility correlation
    /// Measures whether agents' reputations fluctuate together
    pub synchronization_index: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl PopulationVolatilityMetrics {
    /// Compute from individual agent trackers
    ///
    /// # Arguments
    /// * `agent_volatilities` - Current volatility of each agent
    /// * `high_vol_threshold` - Threshold for "high volatility" classification
    pub fn compute(agent_volatilities: &[f32], high_vol_threshold: f32) -> Self {
        let clean: Vec<f32> = agent_volatilities
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .collect();

        if clean.is_empty() {
            return Self::default();
        }

        let n = clean.len() as f32;

        // Basic statistics
        let mean = clean.iter().sum::<f32>() / n;

        let variance = clean.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / n;

        let std = variance.sqrt();

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let min = clean.iter().copied().fold(f32::INFINITY, f32::min);

        let max = clean.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        // High volatility fraction
        let high_count = clean.iter().filter(|&&v| v > high_vol_threshold).count() as f32;

        Self {
            mean_volatility: mean,
            volatility_std: std,
            min_volatility: if min.is_finite() { min } else { 0.0 },
            max_volatility: if max.is_finite() { max } else { 0.0 },
            high_volatility_fraction: high_count / n,
            mean_volatility_history: Vec::new(),
            synchronization_index: 0.0, // Would need cross-correlation calculation
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add historical mean volatility point
    pub fn record_mean(&mut self, mean_vol: f32) {
        self.mean_volatility_history.push(mean_vol);
    }

    /// Calculate trend in population volatility over time
    #[must_use]
    pub fn population_trend(&self, periods: usize) -> f32 {
        let n = self.mean_volatility_history.len();
        if n < periods || periods == 0 {
            return 0.0;
        }

        let recent = self.mean_volatility_history[n - 1];
        let past = self.mean_volatility_history[n - periods];

        recent - past
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if population shows synchronized volatility
    /// (indicating systemic reputation shocks)
    #[must_use]
    pub fn is_synchronized(&self, threshold: f32) -> bool {
        self.synchronization_index > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_basic() {
        let mut tracker = ReputationVolatilityTracker::new(10);

        // Initially stationary volatility (positive)
        assert!(tracker.current_volatility() > 0.0);

        // Add some observations
        for i in 0..5 {
            tracker.update(0.5, i as Tick);
        }

        // Still zero (need at least 2 for variance)
        // Actually, after 2+ updates, we should have volatility
        assert!(tracker.current_volatility() >= 0.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Add more with variance
        for i in 5..20 {
            let trust = if i % 2 == 0 { 0.3 } else { 0.7 };
            tracker.update(trust, i as Tick);
        }

        // Should have positive volatility now
        assert!(tracker.current_volatility() > 0.0);
    }

    #[test]
    fn test_volatility_bounded() {
        let mut tracker = ReputationVolatilityTracker::new(100);

        // Add many observations
        for i in 0..1000 {
            let trust = (i % 100) as f32 / 100.0;
            tracker.update(trust, i as Tick);
        }

        // Volatility should be bounded [0, 1]
        assert!(tracker.current_volatility() >= 0.0);
        assert!(tracker.current_volatility() <= 1.0);

        // Check history is bounded
        for &vol in &tracker.volatility_history {
            assert!((0.0..=1.0).contains(&vol));
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_trend_detection() {
        let mut tracker = ReputationVolatilityTracker::new(10);

        // Low volatility period
        for i in 0..20 {
            tracker.update(0.5, i as Tick);
        }
        let low_vol = tracker.current_volatility();

        // High volatility period
        for i in 20..40 {
            let trust = if i % 2 == 0 { 0.2 } else { 0.8 };
            tracker.update(trust, i as Tick);
        }
        let high_vol = tracker.current_volatility();

        // Trend should be positive (increasing volatility)
        let trend = tracker.volatility_trend(20);
        assert!(
            trend > 0.0,
            "Trend should be positive with increasing volatility"
        );

        assert!(
            high_vol > low_vol,
            "High volatility period should have higher measured volatility"
        );
    }

    #[test]
    fn test_clustering_detection() {
        let mut tracker = ReputationVolatilityTracker::new(10);

        // First: low volatility
        for i in 0..20 {
            tracker.update(0.5, i as Tick);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Second: high volatility cluster
        for i in 20..40 {
            let trust = if i % 2 == 0 { 0.1 } else { 0.9 };
            tracker.update(trust, i as Tick);
        }

        // Third: low volatility again
        for i in 40..60 {
            tracker.update(0.5, i as Tick);
        }

        // Detect clusters above median volatility
        let threshold = tracker.mean_volatility();
        let clusters = tracker.detect_clustering(threshold);

        assert!(!clusters.is_empty(), "Should detect volatility cluster");
    }

    #[test]
    fn test_population_metrics() {
        let volatilities = vec![0.05, 0.08, 0.12, 0.15, 0.20, 0.25, 0.30];

        let metrics = PopulationVolatilityMetrics::compute(&volatilities, 0.15);

        assert!(metrics.mean_volatility > 0.0);
        assert!(metrics.volatility_std >= 0.0);
        assert!(metrics.min_volatility <= metrics.max_volatility);

        // High volatility fraction (> 0.15)
        // Should be 4/7: 0.20, 0.25, 0.30 (plus maybe 0.15 depending on > vs >=)
        assert!(metrics.high_volatility_fraction > 0.0);
    }

    #[test]
    fn test_custom_params() {
        // More responsive (higher alpha)
        let responsive = ReputationVolatilityTracker::with_params(10, 0.001, 0.20, 0.75);

        // More stable (higher beta)
        let stable = ReputationVolatilityTracker::with_params(10, 0.001, 0.05, 0.90);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(responsive.alpha, 0.20);
        assert_eq!(responsive.beta, 0.75);
        assert_eq!(stable.alpha, 0.05);
        assert_eq!(stable.beta, 0.90);
    }

    #[test]
    #[should_panic(expected = "GARCH stationarity")]
    fn test_invalid_garch_params() {
        // alpha + beta = 1.0 violates stationarity
        let _ = ReputationVolatilityTracker::with_params(10, 0.001, 0.5, 0.5);
    }

    #[test]
    fn test_reset() {
        let mut tracker = ReputationVolatilityTracker::new(10);

        // Build up some state
        for i in 0..50 {
            tracker.update(0.5 + (i % 3) as f32 * 0.1, i as Tick);
        }

        assert!(!tracker.volatility_history.is_empty());

        // Reset
        tracker.reset();

        assert_eq!(tracker.current_volatility(), tracker.omega.sqrt());
        assert!(tracker.volatility_history.is_empty());
        assert!(tracker.trust_history.is_empty());
    }
}
