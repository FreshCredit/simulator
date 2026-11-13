//! Bass diffusion adoption model for protocol introduction
//!
//! Per `RESEARCH_TEMPORAL_DYNAMICS.md` Section 1.3
//! Based on Bass (1969) and Rai & `McAndrews` (2015)

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
// TAG: surface=simulation owner=research-team rule=SIM-001
/// Bass diffusion model parameters and computation
///
/// The Bass model describes how new products/technologies are adopted
/// through a combination of:
/// - Innovation (external influence, e.g., advertising)
/// - Imitation (internal influence, e.g., word-of-mouth)
///
/// # Mathematical Model
///
/// The differential equation governing adoption:
/// ```text
/// dF/dt = (p + q·F) · (1 - F)
/// ```
///
/// Where:
/// - F(t) = fraction of population that has adopted by time t
/// - p = innovation coefficient (external influence)
/// - q = imitation coefficient (internal influence)
///
/// The analytical solution:
/// ```text
/// F(t) = (1 - exp(-(p+q)t)) / (1 + (q/p)·exp(-(p+q)t))
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BassDiffusionModel {
    /// Innovation coefficient (p)
    /// Represents adoption driven by external factors
    /// Typical range: 0.01 to 0.03
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub innovation_coeff: f32,

    /// Imitation coefficient (q)
    /// Represents adoption driven by social contagion
    /// Typical range: 0.3 to 0.5
    pub imitation_coeff: f32,

    /// Market saturation level (m)
    /// Maximum fraction of population that will adopt
    /// Range: 0.0 to 1.0
    pub saturation: f32,
}

impl BassDiffusionModel {
    /// Create new Bass diffusion model with standard parameters
    ///
    /// Default values from Rai & `McAndrews` (2015) residential solar study:
    /// - p = 0.03 (innovation)
    /// - q = 0.38 (imitation)
    #[must_use]
    pub const fn new() -> Self {
        Self {
            innovation_coeff: 0.03,
            imitation_coeff: 0.38,
            saturation: 0.90,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create model with specific parameters
    #[must_use]
    pub fn with_params(p: f32, q: f32, saturation: f32) -> Self {
        Self {
            innovation_coeff: p,
            imitation_coeff: q,
            saturation: saturation.clamp(0.0, 1.0),
        }
    }

    /// Create model scaled for normalized time [0, 1]
    ///
    /// With standard parameters (p=0.03, q=0.38), the Bass curve
    /// needs ~50 time units to reach saturation. This constructor
    /// scales parameters so the full S-curve fits in [0, 1].
    #[must_use]
    pub fn for_normalized_time() -> Self {
        // Scale so that at t=1, adoption is near saturation (0.9)
        // With p=0.6, q=7.6, the curve completes by t=1
        const TIME_SCALE: f32 = 20.0;
        Self {
            innovation_coeff: 0.03 * TIME_SCALE,
            imitation_coeff: 0.38 * TIME_SCALE,
            saturation: 0.90,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate cumulative adoption fraction at time t
    ///
    /// # Arguments
    /// * `t` - Time (in arbitrary units, scaled by `time_scale`)
    ///
    /// # Returns
    /// Cumulative adoption fraction in [0.0, saturation]
    ///
    /// # Example
    /// ```
    /// use sim_core::temporal::BassDiffusionModel;
    ///
    /// let model = BassDiffusionModel::new();
    ///
    /// // At start: innovators only (~1-2%)
    /// let start = model.cumulative_adoption(0.0);
    /// debug_assert!(start < 0.02);
    ///
    /// // At t=10: significant adoption
    /// let mid = model.cumulative_adoption(10.0);
    /// debug_assert!(mid > 0.3 && mid < 0.9);
    ///
    /// // At large t: near saturation
    /// let end = model.cumulative_adoption(50.0);
    /// debug_assert!((end - 0.9).abs() < 0.05);
    /// ```
    #[must_use]
    pub fn cumulative_adoption(&self, t: f32) -> f32 {
        let p = self.innovation_coeff;
        let q = self.imitation_coeff;
        let m = self.saturation;

        if t <= 0.0 || m <= 0.0 {
            return 0.0;
        }

        if p <= 0.0 {
            // Pure imitation without innovation
            return 0.0;
        }

        // Analytical solution to Bass differential equation
        let exp_term = (-(p + q) * t).exp();
        let numerator = 1.0 - exp_term;
        let denominator = 1.0 + (q / p) * exp_term;

        if denominator <= 0.0 {
            0.0
        } else {
            (m * numerator / denominator).clamp(0.0, m)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate adoption rate (derivative) at time t
    ///
    /// This represents how fast adoption is occurring,
    /// useful for determining new adopters per tick.
    ///
    /// # Arguments
    /// * `t` - Time (in arbitrary units)
    ///
    /// # Returns
    /// Instantaneous adoption rate (fraction of population per time unit)
    #[must_use]
    pub fn adoption_rate(&self, t: f32) -> f32 {
        if self.saturation <= 0.0 {
            return 0.0;
        }
        let p = self.innovation_coeff;
        let q = self.imitation_coeff;
        let f = (self.cumulative_adoption(t) / self.saturation).clamp(0.0, 1.0);

        // dF/dt = (p + q·F) · (1 - F) · m
        self.saturation * (p + q * f) * (1.0 - f)
    }

    /// Find time of peak adoption rate
    ///
    /// Returns time when adoption is fastest.
    /// Occurs at inflection point of S-curve.
    ///
    /// # Returns
    /// Time of peak adoption rate
    #[must_use]
    pub fn peak_adoption_time(&self) -> f32 {
        let p = self.innovation_coeff;
        let q = self.imitation_coeff;

        // Peak occurs when F = (q - p) / (2q) if q > p
        if q <= p {
            return 0.0;
        }

        let f_peak = (q - p) / (2.0 * q);

        // Solve for t when F(t) = f_peak
        // t = ln((q/p) · (f_peak / (1 - f_peak))) / (p + q)
        let ratio = (q / p) * (f_peak / (1.0 - f_peak));
        ratio.ln() / (p + q)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate full adoption curve
    ///
    /// Returns vector of cumulative adoption values
    /// sampled at regular intervals.
    #[must_use]
    pub fn generate_curve(&self, num_points: usize) -> AdoptionCurve {
        let mut points = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let t = i as f32 / (num_points - 1).max(1) as f32;
            points.push(self.cumulative_adoption(t));
        }

        AdoptionCurve { points }
    }
}

impl Default for BassDiffusionModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-computed adoption curve for efficient lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptionCurve {
    points: Vec<f32>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl AdoptionCurve {
    /// Get adoption value at normalized time
    #[must_use]
    pub fn at(&self, t: f32) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        if self.points.len() == 1 {
            return self.points[0];
        }
        if t <= 0.0 {
            return self.points.first().copied().unwrap_or(0.0);
        }
        if t >= 1.0 {
            return self.points.last().copied().unwrap_or(1.0);
        }

        let idx = t * (self.points.len() - 1) as f32;
        let i = idx.floor() as usize;
        let frac = idx - i as f32;

        if i + 1 < self.points.len() {
            let v1 = self.points[i];
            let v2 = self.points[i + 1];
            v1 + frac * (v2 - v1)
        } else {
            self.points[i]
        }
    }

    /// Get raw points
    #[must_use]
    pub fn points(&self) -> &[f32] {
        &self.points
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bass_basic() {
        // Use normalized time model for tests expecting 0-1 range
        let model = BassDiffusionModel::for_normalized_time();

        // Start at 0
        assert_eq!(model.cumulative_adoption(0.0), 0.0);

        // At t=1.0: near saturation (normalized time)
        let end = model.cumulative_adoption(1.0);
        assert!(
            (end - 0.9).abs() < 0.05,
            "Should approach saturation at t=1.0, got {end}"
        );

        // Monotonic increase (0 to 1)
        let mut prev = 0.0;
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let curr = model.cumulative_adoption(t);
            assert!(curr >= prev, "Should be monotonic at t={t}");
            prev = curr;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_adoption_rate() {
        // Use normalized time model
        let model = BassDiffusionModel::for_normalized_time();

        // Rate should be positive throughout
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let rate = model.adoption_rate(t);
            assert!(rate >= 0.0, "Rate should be non-negative at t={t}");
        }

        // Rate should peak at some intermediate point (S-curve inflection)
        let mut max_rate = 0.0;
        let mut max_t = 0.0;
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let rate = model.adoption_rate(t);
            if rate > max_rate {
                max_rate = rate;
                max_t = t;
            }
        }

        // Peak should be somewhere in the middle (with scaled params, peak ~0.5)
        assert!(
            max_t > 0.1 && max_t < 0.9,
            "Peak should be in middle range, got t={max_t}"
        );
    }

    #[test]
    fn test_peak_adoption_time() {
        // Use normalized time model
        let model = BassDiffusionModel::for_normalized_time();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let peak_time = model.peak_adoption_time();

        // With scaled p=0.3, q=3.8, peak should be around t=0.5-0.7
        assert!(
            peak_time > 0.2 && peak_time < 0.9,
            "Peak time should be in reasonable range (around 0.5-0.7), got {peak_time}"
        );

        // Rate at peak time should be near maximum
        let peak_rate = model.adoption_rate(peak_time);
        let mut max_sampled_rate = 0.0;
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let rate = model.adoption_rate(t);
            if rate > max_sampled_rate {
                max_sampled_rate = rate;
            }
        }
        assert!(
            peak_rate >= max_sampled_rate * 0.95,
            "Peak rate should be near maximum: {peak_rate} vs sampled {max_sampled_rate}"
        );
    }

    #[test]
    fn test_adoption_curve() {
        // Use normalized time model
        let model = BassDiffusionModel::for_normalized_time();
        let curve = model.generate_curve(101);

        assert_eq!(curve.points().len(), 101);

        // Check endpoints
        assert!((curve.at(0.0) - 0.0).abs() < 0.001);
        assert!((curve.at(1.0) - 0.9).abs() < 0.01);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Check interpolation
        let t = 0.5;
        let expected = model.cumulative_adoption(t);
        let actual = curve.at(t);
        assert!((actual - expected).abs() < 0.01);
    }

    #[test]
    fn test_custom_params() {
        // Test that higher coefficients lead to faster adoption
        // Fast model: p=0.3, q=1.5 (10x the standard coefficients)
        let fast = BassDiffusionModel::with_params(0.3, 1.5, 0.9);
        // Standard model: p=0.03, q=0.38
        let standard = BassDiffusionModel::new();

        // At same early time, fast model should have higher adoption
        let fast_early = fast.cumulative_adoption(1.0);
        let std_early = standard.cumulative_adoption(1.0);
        assert!(
            fast_early > std_early,
            "Faster model should have higher adoption at t=1: {fast_early} vs {std_early}"
        );

        // Fast model reaches saturation at t=10
        let fast_saturation = fast.cumulative_adoption(10.0);
        assert!(
            fast_saturation > 0.85,
            "Fast model should reach saturation at t=10: {fast_saturation}"
        );

        // Standard model needs more time (t=50) to reach saturation
        let std_saturation = standard.cumulative_adoption(50.0);
        assert!(
            std_saturation > 0.85,
            "Standard model should reach saturation at t=50: {std_saturation}"
        );
    }
}
