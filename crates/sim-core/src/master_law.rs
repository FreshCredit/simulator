// TAG: surface=simulation owner=research-team rule=SIM-001
//! Master Update Law & Closed-Loop Damping Architecture (OSF v3.0 GTT / DOCAS / TFP)
//!
//! Formally verified in Lean 4 (`TrustStack.MasterLaw`):
//! - M1: Physics reduction at (λ⁺=λ⁻=λ, r=1, δ=0, S_base=0) recovers Rescorla–Wagner ΔS = λε
//! - M2: Geometric contraction to baseline S_t - S_base = (1 - δ)^t (S_0 - S_base) under zero input
//! - M3: BIBO stability: |S_t - S_base| ≤ (1-δ)^t |S_0 - S_base| + (λ⁺ + λ⁻)E/δ
//!
//! Replaces retired κ = ∇·T with Liouville transverse contraction (-μ)
//! and closed-loop characteristic damping analysis (overdamped vs underdamped ringing at λ* = 1/4).

use serde::{Deserialize, Serialize};

/// Master update law configuration preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MasterLawPreset {
    /// Pure integrator preset (r=1, δ=0, λ⁺=λ⁻=λ, S_base=0): GTT physics limit
    PhysicsLeakFree,
    /// Homeostatic baseline-restoring preset (r ∈ [0,1], δ=0.01, S_base=0.5): DOCAS biology
    #[default]
    BiologyHomeostatic,
    /// Asymmetric break/build preset (λ⁻ > λ⁺, r=1, δ=0.01, S_base=0.5): TFP protocol
    ProtocolAsymmetric,
    /// User-configured custom parameters
    Custom,
}

/// Master update law configuration parameters
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MasterLawConfig {
    /// Active preset
    pub preset: MasterLawPreset,
    /// Sensitivity to positive mismatch (validation > expectation)
    pub lambda_plus: f64,
    /// Sensitivity to negative mismatch (validation < expectation)
    pub lambda_minus: f64,
    /// Recognition gating multiplier r_t in [0, 1]
    pub recognition_gate: f64,
    /// Baseline-restoring leak rate delta_S
    pub leak_delta: f64,
    /// Baseline prior S_base
    pub s_base: f64,
    /// Normalization root-mean-square scale s_d
    pub scale_s_d: f64,
    /// Corrective dissipation bandwidth mu
    pub corrective_bandwidth_mu: f64,
}

impl Default for MasterLawConfig {
    fn default() -> Self {
        Self::for_preset(MasterLawPreset::BiologyHomeostatic)
    }
}

impl MasterLawConfig {
    /// Construct a Master Law configuration matching a canonical research preset
    #[must_use]
    pub fn for_preset(preset: MasterLawPreset) -> Self {
        match preset {
            MasterLawPreset::PhysicsLeakFree => Self {
                preset,
                lambda_plus: 0.35,
                lambda_minus: 0.35,
                recognition_gate: 1.0,
                leak_delta: 0.0,
                s_base: 0.0,
                scale_s_d: 1.0,
                corrective_bandwidth_mu: 1.0,
            },
            MasterLawPreset::BiologyHomeostatic => Self {
                preset,
                lambda_plus: 0.01,
                lambda_minus: 0.01,
                recognition_gate: 1.0,
                leak_delta: 0.01,
                s_base: 0.5,
                scale_s_d: 1.0,
                corrective_bandwidth_mu: 1.0,
            },
            MasterLawPreset::ProtocolAsymmetric => Self {
                preset,
                lambda_plus: 0.005,
                lambda_minus: 0.02,
                recognition_gate: 1.0,
                leak_delta: 0.01,
                s_base: 0.5,
                scale_s_d: 1.0,
                corrective_bandwidth_mu: 1.0,
            },
            MasterLawPreset::Custom => Self {
                preset,
                lambda_plus: 0.01,
                lambda_minus: 0.04,
                recognition_gate: 1.0,
                leak_delta: 0.01,
                s_base: 0.5,
                scale_s_d: 1.0,
                corrective_bandwidth_mu: 1.0,
            },
        }
    }

    /// Compute Definition 2a normalized mismatch: ε̃ = (V - E) / s_d
    #[must_use]
    pub fn compute_normalized_mismatch(&self, validation: f64, expectation: f64) -> f64 {
        let s_d = if self.scale_s_d > 1e-6 {
            self.scale_s_d
        } else {
            1.0
        };
        (validation - expectation) / s_d
    }

    /// Advance trust standing by one step of the Master Law
    #[must_use]
    pub fn step(&self, current_s: f64, normalized_epsilon: f64) -> f64 {
        let eps = normalized_epsilon;
        let pos_term = self.lambda_plus * eps.max(0.0) * self.recognition_gate;
        let neg_term = self.lambda_minus * eps.min(0.0) * self.recognition_gate;
        let leak_term = self.leak_delta * (current_s - self.s_base);
        current_s + pos_term + neg_term - leak_term
    }

    /// Advance trust standing directly from raw validation and expectation signals
    #[must_use]
    pub fn step_with_signals(&self, current_s: f64, validation: f64, expectation: f64) -> f64 {
        let eps = self.compute_normalized_mismatch(validation, expectation);
        self.step(current_s, eps)
    }

    /// Exact fixed-point integer Master Stability Law step (`FIXED_SCALE = 10^9`, `div_euclid`),
    /// matching `fresh-protocol-spec::MasterLawParams::step_fixed_i64`, `uniffi_bridge`, and `ZK-STP v8` bit-for-bit.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn step_fixed_i64(
        &self,
        current_s_fixed: i64,
        v_fixed: i64,
        e_fixed: i64,
        r_fixed: i64,
    ) -> i64 {
        const FIXED_SCALE: i128 = 1_000_000_000;
        let lp = (self.lambda_plus * (FIXED_SCALE as f64)).round() as i128;
        let ln = (self.lambda_minus * (FIXED_SCALE as f64)).round() as i128;
        let delta = (self.leak_delta * (FIXED_SCALE as f64)).round() as i128;
        let s_base = (self.s_base * (FIXED_SCALE as f64)).round() as i128;
        let s = i128::from(current_s_fixed);
        let v = i128::from(v_fixed);
        let e = i128::from(e_fixed);
        let r = i128::from(r_fixed);
        let eps = v - e;
        let (eps_p, eps_n) = if eps >= 0 { (eps, 0) } else { (0, -eps) };
        let q_d = ((lp * eps_p - ln * eps_n) * r).div_euclid(FIXED_SCALE * FIXED_SCALE);
        let q_l = (delta * (s - s_base)).div_euclid(FIXED_SCALE);
        let clamped = (s + q_d - q_l).clamp(0, FIXED_SCALE);
        i64::try_from(clamped).unwrap_or(0)
    }

    /// Z3-verified (v4.5) asymmetric shifted BIBO invariant interval under unit mismatch |ε̃| ≤ 1:
    /// `[S_base - λ⁻/δ_S, S_base + λ⁺/δ_S]` when `δ_S > 0`. Returns `None` in the leak-free (`δ_S = 0`) limit.
    #[must_use]
    pub fn asymmetric_invariant_bounds(&self) -> Option<(f64, f64)> {
        if self.leak_delta <= 1e-12 {
            None
        } else {
            let lower = self.s_base - (self.lambda_minus / self.leak_delta);
            let upper = self.s_base + (self.lambda_plus / self.leak_delta);
            Some((lower, upper))
        }
    }

    /// Clamp a state value into the Z3-verified asymmetric BIBO invariant corridor when `δ_S > 0`
    #[must_use]
    pub fn clamp_to_invariant_bounds(&self, s: f64) -> f64 {
        if let Some((lower, upper)) = self.asymmetric_invariant_bounds() {
            s.clamp(lower, upper)
        } else {
            s
        }
    }

    /// Open-loop Liouville transverse contraction rate κ = -μ everywhere
    #[must_use]
    pub fn open_loop_divergence(&self) -> f64 {
        -self
            .corrective_bandwidth_mu
            .abs()
            .max(self.leak_delta.min(1e-6))
    }

    /// Analyze closed-loop recurrence stability and damping regime for sensitivity parameter λ
    #[must_use]
    pub fn analyze_closed_loop_damping(&self, lambda: f64) -> ClosedLoopDampingAnalysis {
        let lambda_val = if lambda > 0.0 {
            lambda.abs()
        } else {
            self.lambda_plus.abs()
        };
        let is_stable_1step = lambda_val > 0.0 && lambda_val < 2.0;
        let boundary_2step = 2.0 / (1.0 + 5.0_f64.sqrt()); // 1/phi ≈ 0.61803398875
        let is_stable_2step = lambda_val > 0.0 && lambda_val < boundary_2step;

        let regime = if lambda_val <= 0.0 {
            DampingRegime::MarginalStatic
        } else if lambda_val < 0.25 {
            DampingRegime::Overdamped
        } else if (lambda_val - 0.25).abs() < 1e-6 {
            DampingRegime::CriticallyDamped
        } else if lambda_val < 1.0 {
            DampingRegime::UnderdampedRinging
        } else {
            DampingRegime::Unstable
        };

        let ringing_period_steps = if lambda_val > 0.25 && lambda_val < 1.0 {
            let disc = (4.0 * lambda_val - 1.0).sqrt();
            let angle = disc.atan();
            if angle > 1e-6 {
                Some(2.0 * std::f64::consts::PI / angle)
            } else {
                None
            }
        } else {
            None
        };

        ClosedLoopDampingAnalysis {
            lambda: lambda_val,
            critical_lambda: 0.25,
            regime,
            is_stable_1step,
            is_stable_2step,
            boundary_2step,
            ringing_period_steps,
        }
    }
}

/// Closed-loop recurrence damping regime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DampingRegime {
    /// Zero sensitivity (pure persistence)
    MarginalStatic,
    /// Overdamped (monotonic decay without oscillation)
    Overdamped,
    /// Critically damped (fastest decay without ringing)
    CriticallyDamped,
    /// Underdamped ringing (damped harmonic oscillations)
    UnderdampedRinging,
    /// Unstable (divergent oscillations)
    Unstable,
}

/// Diagnostic report for closed-loop trust recurrence
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClosedLoopDampingAnalysis {
    /// Sensitivity parameter lambda
    pub lambda: f64,
    /// Theoretical critical damping boundary lambda* = 1/4
    pub critical_lambda: f64,
    /// Inferred damping regime
    pub regime: DampingRegime,
    /// Whether 1-step recurrence is stable (lambda in (0, 2))
    pub is_stable_1step: bool,
    /// Whether 2-step recurrence is stable (lambda in (0, 1/phi))
    pub is_stable_2step: bool,
    /// 2-step closure boundary 1/phi ≈ 0.618034
    pub boundary_2step: f64,
    /// Oscillation period T(lambda) in steps if underdamped
    pub ringing_period_steps: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_step_physics_reduction() {
        let config = MasterLawConfig::for_preset(MasterLawPreset::PhysicsLeakFree);
        let s0 = 10.0;
        let eps = 2.0;
        let s1 = config.step(s0, eps);
        assert!((s1 - 10.70).abs() < 1e-9);
    }

    #[test]
    fn test_master_step_contraction_to_baseline() {
        let config = MasterLawConfig::for_preset(MasterLawPreset::BiologyHomeostatic);
        let mut s = 0.0;
        for _ in 0..200 {
            s = config.step(s, 0.0);
        }
        assert!((s - 0.5).abs() < 0.07);
    }

    #[test]
    fn test_closed_loop_ringing_at_lambda_0_61() {
        let config = MasterLawConfig::default();
        let analysis = config.analyze_closed_loop_damping(0.61);
        assert_eq!(analysis.regime, DampingRegime::UnderdampedRinging);
        assert!(analysis.is_stable_1step);
        assert!(analysis.is_stable_2step);
        let period = analysis
            .ringing_period_steps
            .expect("ringing period exists");
        assert!((period - 7.17).abs() < 0.15);
    }

    #[test]
    fn test_liouville_transverse_divergence_is_minus_mu() {
        let config = MasterLawConfig {
            corrective_bandwidth_mu: 1.5,
            ..Default::default()
        };
        assert_eq!(config.open_loop_divergence(), -1.5);
    }

    #[test]
    fn test_master_law_golden_ratio_boundary_and_kuramoto_phase_wrapping() {
        let config = MasterLawConfig::default();
        let inv_phi = 2.0 / (1.0 + 5.0_f64.sqrt()); // ≈ 0.61803398875

        // Exactly below boundary: stable 2-step feedback
        let analysis_below = config.analyze_closed_loop_damping(inv_phi - 0.005);
        assert!(analysis_below.is_stable_2step);

        // Above boundary: unstable 2-step recurrence
        let analysis_above = config.analyze_closed_loop_damping(inv_phi + 0.005);
        assert!(
            !analysis_above.is_stable_2step,
            "Gain above 1/phi must be unstable for 2-step recurrence"
        );

        // Kuramoto phase wrapping preserves circle group bound for extreme values
        let wrapped_pos = crate::kuramoto_leakage::wrap_phase_pi(50.0 * std::f64::consts::PI);
        assert!(wrapped_pos >= -std::f64::consts::PI && wrapped_pos <= std::f64::consts::PI);

        let wrapped_neg = crate::kuramoto_leakage::wrap_phase_pi(-100.5 * std::f64::consts::PI);
        assert!(wrapped_neg >= -std::f64::consts::PI && wrapped_neg <= std::f64::consts::PI);
    }

    #[test]
    fn test_asymmetric_shifted_bibo_invariance_z3_parity() {
        let cfg = MasterLawConfig {
            preset: MasterLawPreset::ProtocolAsymmetric,
            lambda_plus: 0.02,
            lambda_minus: 0.08,
            recognition_gate: 1.0,
            leak_delta: 0.10,
            s_base: 0.50,
            scale_s_d: 1.0,
            corrective_bandwidth_mu: 1.0,
        };
        let (lower, upper) = cfg
            .asymmetric_invariant_bounds()
            .expect("Positive leak_delta must yield invariant bounds");
        assert!((lower - (-0.30)).abs() < 1e-12);
        assert!((upper - 0.70).abs() < 1e-12);

        for s_init in [lower, 0.0, cfg.s_base, upper] {
            for eps in [-1.0, -0.5, 0.0, 0.5, 1.0] {
                let s_next = cfg.step(s_init, eps);
                assert!(
                    s_next >= lower - 1e-12 && s_next <= upper + 1e-12,
                    "Step from {s_init} with eps={eps} escaped [{lower}, {upper}]: got {s_next}"
                );
            }
        }
    }

    #[test]
    fn test_step_fixed_i64_golden_parity() {
        let cfg = MasterLawConfig {
            preset: MasterLawPreset::ProtocolAsymmetric,
            lambda_plus: 0.228,
            lambda_minus: 0.912,
            recognition_gate: 1.0,
            leak_delta: 0.010,
            s_base: 0.500,
            scale_s_d: 1.0,
            corrective_bandwidth_mu: 1.0,
        };
        let s1 = cfg.step_fixed_i64(500_000_000, 850_000_000, 500_000_000, 1_000_000_000);
        assert_eq!(s1, 579_800_000);
        let s2 = cfg.step_fixed_i64(s1, 100_000_000, 750_000_000, 1_000_000_000);
        assert_eq!(s2, 0);
        let s3 = cfg.step_fixed_i64(430_802_000, 500_000_000, 500_000_000, 1_000_000_000);
        assert_eq!(s3, 431_493_980);
    }
}
