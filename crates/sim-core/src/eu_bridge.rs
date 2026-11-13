// TAG: surface=simulation owner=research-team rule=SIM-001
//! Esscher Exponential-Tilt Expected Utility Bridge & Active-Inference Readout (OSF v3.0 GTT §2.3 / DOCAS)
//!
//! Formally verified in Lean 4 (`TrustStack.Tilt`):
//! 1. P̃ is a probability vector (positivity, sums to 1)
//! 2. Neutral reduction: at trust index ι = 0, P̃ = P (standard expected utility)
//! 3. Monotone optimism: tilted expected utility is non-decreasing in ι (s ≥ t ⟹ EU(s) ≥ EU(t))
//! 4. Bounded distortion: |log(P̃(o)/P(o))| ≤ |γ ι| (max U - min U) ≤ 2γ max|û|
//!
//! Active-Inference readout:
//! τ = -EFE (negative Expected Free Energy)
//! Valence = sign(Δτ), Arousal = |Δτ|, Dominance = d(Δτ)/dt

use serde::{Deserialize, Serialize};

/// Configuration for Esscher exponential probability tilt
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExponentialTiltConfig {
    /// Tilt scaling amplitude gamma >= 0
    pub gamma_amplitude: f64,
}

impl Default for ExponentialTiltConfig {
    fn default() -> Self {
        Self {
            gamma_amplitude: 1.0,
        }
    }
}

/// Compute trust index ι(T) = tanh(w · T) ∈ (-1, 1)
#[must_use]
pub fn compute_trust_index(docas_levels: &[f64; 5]) -> f64 {
    // Canonical weights: positive expectation (D), validation (O), stabilization (S);
    // negative threat/stress (C), action urgency (A)
    let w = [0.35, 0.40, -0.45, -0.20, 0.50];
    let mut dot = 0.0;
    for i in 0..5 {
        let level = if docas_levels[i].is_finite() {
            docas_levels[i]
        } else {
            0.5
        };
        dot += w[i] * (level - 0.5);
    }
    if dot.is_finite() {
        dot.tanh()
    } else {
        0.0
    }
}

/// Apply Esscher exponential tilt to baseline outcome probabilities P(o)
/// P̃(o) = P(o) * exp(γ * ι * u(o)) / Z
#[must_use]
pub fn apply_exponential_tilt(
    base_probs: &[f64],
    utilities: &[f64],
    trust_index_iota: f64,
    gamma: f64,
) -> Vec<f64> {
    let len = base_probs.len().min(utilities.len());
    if len == 0 {
        return Vec::new();
    }

    let gamma_safe = if gamma.is_finite() && gamma >= 0.0 {
        gamma
    } else {
        1.0
    };
    let iota_safe = if trust_index_iota.is_finite() {
        trust_index_iota.clamp(-1.0, 1.0)
    } else {
        0.0
    };

    let mut weights = Vec::with_capacity(len);
    let mut z = 0.0;

    for i in 0..len {
        let p = if base_probs[i].is_finite() {
            base_probs[i].max(0.0)
        } else {
            0.0
        };
        let u = if utilities[i].is_finite() {
            utilities[i]
        } else {
            0.0
        };
        let product = gamma_safe * iota_safe * u;
        let exponent = if product.is_finite() {
            product.clamp(-50.0, 50.0)
        } else {
            0.0
        };
        let w = p * exponent.exp();
        if w.is_finite() && w >= 0.0 {
            weights.push(w);
            z += w;
        } else {
            weights.push(0.0);
        }
    }

    if !z.is_finite() || z <= 0.0 {
        let uniform = 1.0 / len as f64;
        return vec![uniform; len];
    }

    weights
        .into_iter()
        .map(|w| (w / z).clamp(0.0, 1.0))
        .collect()
}

/// Evaluate expected utility under trust-tilted probabilities: EU(a | T) = Σ P̃(o) U(o)
#[must_use]
pub fn tilted_expected_utility(
    base_probs: &[f64],
    utilities: &[f64],
    trust_index_iota: f64,
    gamma: f64,
) -> f64 {
    let tilted_probs = apply_exponential_tilt(base_probs, utilities, trust_index_iota, gamma);
    let mut eu = 0.0;
    let len = tilted_probs.len().min(utilities.len());
    for i in 0..len {
        let u = if utilities[i].is_finite() {
            utilities[i]
        } else {
            0.0
        };
        eu += tilted_probs[i] * u;
    }
    if eu.is_finite() {
        eu
    } else {
        0.0
    }
}

/// Active-inference emotion state readout: τ = -EFE
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ActiveInferenceAffect {
    /// Current confidence / negative free energy tau
    pub tau_current: f64,
    /// Rate of change delta_tau
    pub delta_tau: f64,
    /// Emotional valence: sign(delta_tau) in {-1, 0, 1}
    pub valence: f64,
    /// Emotional arousal: |delta_tau| >= 0
    pub arousal: f64,
    /// Emotional dominance: d(delta_tau)/dt
    pub dominance: f64,
}

impl ActiveInferenceAffect {
    /// Compute active-inference affect dimensions from temporal tau dynamics
    #[must_use]
    pub fn compute(tau_prev: f64, tau_curr: f64, dt: f64, prev_delta_tau: f64) -> Self {
        let tau_curr_safe = if tau_curr.is_finite() { tau_curr } else { 0.0 };
        let tau_prev_safe = if tau_prev.is_finite() { tau_prev } else { 0.0 };
        let prev_delta_safe = if prev_delta_tau.is_finite() {
            prev_delta_tau
        } else {
            0.0
        };
        let delta_tau = tau_curr_safe - tau_prev_safe;
        let valence = if delta_tau.abs() < 1e-7 {
            0.0
        } else {
            delta_tau.signum()
        };
        let arousal = delta_tau.abs();
        let dt_safe = if dt.is_finite() && dt > 1e-6 { dt } else { 1.0 };
        let dominance = (delta_tau - prev_delta_safe) / dt_safe;

        Self {
            tau_current: tau_curr_safe,
            delta_tau,
            valence,
            arousal,
            dominance: if dominance.is_finite() {
                dominance
            } else {
                0.0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neutral_reduction() {
        let base_p = [0.2, 0.5, 0.3];
        let u = [-2.0, 1.0, 5.0];
        let tilted = apply_exponential_tilt(&base_p, &u, 0.0, 1.5);
        for i in 0..base_p.len() {
            assert!((tilted[i] - base_p[i]).abs() < 1e-9);
        }
    }

    #[test]
    fn test_monotone_optimism() {
        let base_p = [0.25, 0.25, 0.25, 0.25];
        let u = [-10.0, 0.0, 5.0, 20.0];
        let eu_pessimistic = tilted_expected_utility(&base_p, &u, -0.8, 1.0);
        let eu_neutral = tilted_expected_utility(&base_p, &u, 0.0, 1.0);
        let eu_optimistic = tilted_expected_utility(&base_p, &u, 0.8, 1.0);

        assert!(eu_optimistic > eu_neutral);
        assert!(eu_neutral > eu_pessimistic);
    }

    #[test]
    fn test_active_inference_readout() {
        let affect = ActiveInferenceAffect::compute(0.5, 0.8, 1.0, 0.1);
        assert_eq!(affect.valence, 1.0);
        assert!((affect.arousal - 0.3).abs() < 1e-7);
        // Dominance = (0.3 - 0.1)/1.0 = 0.2
        assert!((affect.dominance - 0.2).abs() < 1e-7);
    }

    #[test]
    fn test_nan_inputs_resilient_and_finite() {
        let trust_nan = compute_trust_index(&[f64::NAN, f64::NAN, f64::NAN, f64::NAN, f64::NAN]);
        assert_eq!(trust_nan, 0.0);

        let base_p = [0.5, 0.5];
        let u_nan = [f64::NAN, 2.0];
        let tilted = apply_exponential_tilt(&base_p, &u_nan, f64::NAN, f64::NAN);
        assert_eq!(tilted.len(), 2);
        let sum: f64 = tilted.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-9,
            "Simplex probability must sum to 1.0"
        );
        assert!(tilted.iter().all(|&p| p >= 0.0 && p.is_finite()));

        let eu = tilted_expected_utility(&base_p, &u_nan, 0.5, 1.0);
        assert!(eu.is_finite());

        let affect = ActiveInferenceAffect::compute(f64::NAN, 0.5, 0.0, f64::NAN);
        assert!(affect.tau_current.is_finite());
        assert!(affect.delta_tau.is_finite());
        assert!(affect.valence.is_finite());
        assert!(affect.arousal.is_finite());
        assert!(affect.dominance.is_finite());
    }
}
