//! Prospect Theory: Behavioral Economics (Kahneman & Tversky, 1979)
//!
//! Pure functions for prospect theory calculations.

// PROSPECT THEORY: Behavioral Economics (Kahneman & Tversky, 1979)
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Prospect Theory value function
///
/// **Theoretical Basis:**
/// - Kahneman & Tversky (1979): "Prospect Theory: An Analysis of Decision under Risk"
/// - Value is defined on gains/losses relative to reference point (not final wealth)
/// - Concave for gains (risk averse), convex for losses (risk seeking)
/// - Loss aversion: losses loom larger than gains (λ ≈ 2.25)
///
/// **Formula:**
/// - v(x) = x^α if x ≥ 0 (gains)
/// - v(x) = -λ(-x)^β if x < 0 (losses)
///
/// **Parameters:**
/// - α = 0.88 (curvature for gains)
/// - β = 0.88 (curvature for losses)
/// - λ = 2.25 (loss aversion coefficient)
#[must_use]
pub fn prospect_value(gain_or_loss: f32) -> f32 {
    if !gain_or_loss.is_finite() {
        return 0.0;
    }
    const ALPHA: f32 = 0.88; // Curvature parameter (empirical)
    const BETA: f32 = 0.88; // Curvature for losses (empirical)
    const LAMBDA: f32 = 2.25; // Loss aversion coefficient (empirical)

    if gain_or_loss >= 0.0 {
        // Gains: concave (risk averse)
        gain_or_loss.max(0.0).powf(ALPHA)
    } else {
        // Losses: convex (risk seeking), amplified by loss aversion
        -LAMBDA * (-gain_or_loss).max(0.0).powf(BETA)
    }
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

/// Prospect Theory probability weighting function
///
/// **Theoretical Basis:**
/// - People overweight small probabilities and underweight large probabilities
/// - Inverse-S shaped weighting function
///
/// **Formula (Tversky & Kahneman, 1992):**
/// π(p) = p^γ / (p^γ + (1-p)^γ)^(1/γ)
///
/// **Parameter:**
/// - γ = 0.61 (elevation parameter, empirical)
#[must_use]
pub fn probability_weight(probability: f32) -> f32 {
    if !probability.is_finite() || probability <= 0.0 {
        return 0.0;
    }
    if probability >= 1.0 {
        return 1.0;
    }
    const GAMMA: f32 = 0.61; // Probability weighting parameter (empirical)

    let p = probability.clamp(0.0, 1.0);

    let numerator = p.max(0.0).powf(GAMMA);
    let p_gamma = p.max(0.0).powf(GAMMA);
    let one_minus_p_gamma = (1.0 - p).max(0.0).powf(GAMMA);
    let denominator = (p_gamma + one_minus_p_gamma).max(0.0).powf(1.0 / GAMMA);

    if denominator <= 1e-12 {
        return p;
    }

    (numerator / denominator).clamp(0.0, 1.0)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute prospect-weighted expected value
///
/// **Formula:**
/// PT-EV = Σ `π(p_i)` × `v(x_i - r)`
///
/// Where:
/// - `π(p_i)` is the weighted probability
/// - `v(x_i - r)` is the prospect value of outcome `x_i` relative to reference point `r`
///
/// **Parameters:**
/// - `outcomes`: Vec of (probability, `payoff_or_wealth`) pairs
/// - `reference_point`: Reference wealth level (outcomes are evaluated relative to this)
#[must_use]
pub fn prospect_expected_value(
    outcomes: &[(f32, f32)], // (probability, payoff_or_wealth)
    reference_point: f32,
) -> f32 {
    outcomes
        .iter()
        .map(|(prob, payoff)| {
            let weighted_prob = probability_weight(*prob);
            let deviation = if reference_point.is_finite() {
                payoff - reference_point
            } else {
                *payoff
            };
            let value = prospect_value(deviation);
            weighted_prob * value
        })
        .sum()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Decision utility under Prospect Theory
///
/// Compares two choices using PT framework and returns the preferred choice
///
/// **Returns:**
/// - `true` if choice A is preferred
/// - `false` if choice B is preferred
///
/// **Example:**
/// ```ignore
/// use sim_core::economics::prospect_choose;
///
/// // Choice A: 50% chance of $100 gain
/// // Choice B: Certain $40 gain
/// let choice_a = vec![(0.5, 100.0), (0.5, 0.0)];
/// let choice_b = vec![(1.0, 40.0)];
/// let prefer_a = prospect_choose(&choice_a, &choice_b, 0.0);
/// ```
#[must_use]
pub fn prospect_choose(
    choice_a: &[(f32, f32)], // (probability, gain_or_loss) for choice A
    choice_b: &[(f32, f32)], // (probability, gain_or_loss) for choice B
    reference_point: f32,
) -> bool {
    let ev_a = prospect_expected_value(choice_a, reference_point);
    let ev_b = prospect_expected_value(choice_b, reference_point);

    ev_a >= ev_b
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute certainty equivalent under Prospect Theory
///
/// The certain amount that is equivalent in value to a risky prospect
///
/// **Returns:** Certainty equivalent amount (can be positive or negative)
#[must_use]
pub fn certainty_equivalent(
    outcomes: &[(f32, f32)], // (probability, gain_or_loss)
    reference_point: f32,
) -> f32 {
    let pt_value = prospect_expected_value(outcomes, reference_point);

    // Invert the value function to get certainty equivalent deviation
    let ce_deviation = if pt_value >= 0.0 {
        pt_value.max(0.0).powf(1.0 / 0.88) // Inverse of gain function
    } else {
        -(-pt_value / 2.25).max(0.0).powf(1.0 / 0.88) // Inverse of loss function
    };

    if reference_point.is_finite() {
        reference_point + ce_deviation
    } else {
        ce_deviation
    }
}

// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prospect_value_gain() {
        let v = prospect_value(100.0);
        assert!(v > 0.0);
        assert!(v < 100.0); // Concave for gains
                            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    #[test]
    fn test_prospect_value_loss() {
        let v = prospect_value(-100.0);
        assert!(v < 0.0);
        assert!(v.abs() > 100.0); // Loss aversion: losses weighted more
    }

    #[test]
    fn test_prospect_value_zero() {
        let v = prospect_value(0.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn test_probability_weight_small() {
        let w = probability_weight(0.05);
        assert!(w > 0.05); // Overweight small probabilities
    }

    #[test]
    fn test_probability_weight_large() {
        let w = probability_weight(0.95);
        assert!(w < 0.95); // Underweight large probabilities
    }

    #[test]
    fn test_probability_weight_bounds() {
        assert_eq!(probability_weight(0.0), 0.0);
        assert_eq!(probability_weight(1.0), 1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_prospect_expected_value() {
        let outcomes = vec![(0.5, 100.0), (0.5, 0.0)];
        let ev = prospect_expected_value(&outcomes, 0.0);
        assert!(ev > 0.0);
    }

    #[test]
    fn test_prospect_choose() {
        let choice_a = vec![(0.5, 100.0), (0.5, 0.0)];
        let choice_b = vec![(1.0, 40.0)];
        // In prospect theory, agents often prefer certain $40 over 50% chance of $100
        let prefer_a = prospect_choose(&choice_a, &choice_b, 0.0);
        // Just verify it doesn't panic and returns a bool
        let _ = prefer_a;
    }

    #[test]
    fn test_certainty_equivalent_positive() {
        let outcomes = vec![(1.0, 100.0)];
        let ce = certainty_equivalent(&outcomes, 0.0);
        assert!(ce > 0.0);
    }

    #[test]
    fn test_certainty_equivalent_negative() {
        let outcomes = vec![(1.0, -100.0)];
        let ce = certainty_equivalent(&outcomes, 0.0);
        assert!(ce < 0.0);
    }

    #[test]
    fn test_prospect_theory_reference_point_shift_and_nan_immunity() {
        // 1. Shift covariance: certainty equivalent with reference point 1000
        let outcomes = vec![(1.0, 1100.0)]; // $100 gain above $1000 reference
        let ce_1000 = certainty_equivalent(&outcomes, 1000.0);
        assert!((ce_1000 - 1100.0).abs() < 1e-3);

        // 2. Nan immunity
        assert_eq!(prospect_value(f32::NAN), 0.0);
        assert_eq!(probability_weight(f32::NAN), 0.0);
        let nan_outcomes = vec![(f32::NAN, 100.0), (0.5, f32::NAN)];
        let ev = prospect_expected_value(&nan_outcomes, 0.0);
        assert!(ev.is_finite());
    }
}
