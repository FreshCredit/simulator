//! Time Preferences: Hyperbolic and Quasi-Hyperbolic Discounting
//!
//! Pure functions for modeling intertemporal choice.

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
// HYPERBOLIC DISCOUNTING: Time Preferences (Laibson 1997, Phelps & Pollak 1968)
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Hyperbolic discount factor
///
/// **Theoretical Basis:**
/// - People exhibit present bias: prefer immediate rewards over larger future ones
/// - Discount rate declines as delay increases (non-constant impatience)
/// - Standard exponential discounting cannot explain observed behavior
///
/// **Formula (Mazur 1987):**
/// V = A / (1 + k·t)
///
/// Where:
/// - V = present value
/// - A = reward amount
/// - k = impatience parameter (higher = more impatient)
/// - t = delay (ticks)
///
/// **Empirical Values:**
/// - k ≈ 0.1-0.3 for typical agents (Mazur 1987, animal studies)
/// - k ≈ 0.05-0.15 for human intertemporal choices
///
/// **Example:**
/// ```ignore
/// use sim_core::economics::hyperbolic_discount;
///
/// // Value of $100 in 10 ticks (k=0.1)
/// let value = hyperbolic_discount(100.0, 10, 0.1);
/// // value ≈ 50.0 (50% discount over 10 ticks)
/// ```
#[must_use]
pub fn hyperbolic_discount(reward: f32, delay: u32, k: f32) -> f32 {
    if !reward.is_finite() || delay == 0 {
        return if reward.is_finite() { reward } else { 0.0 };
    }
    let t = delay as f32;
    let safe_k = if k.is_finite() { k.max(0.0) } else { 0.0 };
    reward / (1.0 + safe_k * t)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Quasi-hyperbolic (β-δ) discount factor
///
/// **Theoretical Basis:**
/// - Laibson (1997): "Golden Eggs and Hyperbolic Discounting"
/// - Separates present bias (β) from long-term impatience (δ)
/// - Accounts for "sophisticated" vs "naive" agent behavior
///
/// **Formula:**
/// V = A                    if t = 0 (immediate)
/// V = β × δ^t × A          if t > 0 (delayed)
///
/// Where:
/// - β = present bias factor (0 < β ≤ 1, typically 0.7-0.9)
/// - δ = exponential discount factor (0 < δ ≤ 1, typically 0.95-0.99)
/// - t = delay (ticks)
/// - A = reward amount
///
/// **Agent Types:**
/// - Time-consistent: β = 1.0 (standard exponential)
/// - Present-biased: β < 1.0 (hyperbolic discounting)
/// - Highly impatient: low δ (steep long-term discounting)
#[must_use]
pub fn quasi_hyperbolic_discount(reward: f32, delay: u32, beta: f32, delta: f32) -> f32 {
    if !reward.is_finite() || delay == 0 {
        return if reward.is_finite() { reward } else { 0.0 };
    }
    let t = delay as f32;
    let b = if beta.is_finite() {
        beta.clamp(0.0, 1.0)
    } else {
        0.85
    };
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let d = if delta.is_finite() {
        delta.clamp(0.0, 1.0)
    } else {
        0.95
    };
    reward * b * d.powf(t)
}

/// Compare immediate vs delayed rewards under hyperbolic discounting
///
/// **Returns:** true if immediate reward is preferred
///
/// **Example:**
/// ```ignore
/// use sim_core::economics::prefer_immediate_hyperbolic;
///
/// // Prefer $50 now vs $100 in 20 ticks? (k=0.1)
/// let prefer_now = prefer_immediate_hyperbolic(50.0, 0, 100.0, 20, 0.1);
/// // true: 50 > 100/(1+0.1*20) = 33.3
/// ```
#[must_use]
pub fn prefer_immediate_hyperbolic(
    immediate_reward: f32,
    delayed_reward: f32,
    delay: u32,
    k: f32,
) -> bool {
    let immediate_value = immediate_reward;
    let delayed_value = hyperbolic_discount(delayed_reward, delay, k);
    immediate_value > delayed_value
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compare two delayed options under hyperbolic discounting
///
/// Returns the option with higher present value
///
/// **Parameters:**
/// - `reward_a`, `delay_a`: First option
/// - `reward_b`, `delay_b`: Second option
/// - `k`: Impatience parameter
///
/// **Returns:** true if option A has higher present value
#[must_use]
pub fn compare_delayed_options(
    reward_a: f32,
    delay_a: u32,
    reward_b: f32,
    delay_b: u32,
    k: f32,
) -> bool {
    let value_a = hyperbolic_discount(reward_a, delay_a, k);
    let value_b = hyperbolic_discount(reward_b, delay_b, k);
    value_a >= value_b
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute impatience parameter from observed choices
///
/// **Formula:**
/// Given indifference between (A, t₁) and (B, t₂):
/// k = (B - A) / (A·t₂ - B·t₁)
///
/// **Returns:** Estimated k, or None if indifference cannot be determined
#[must_use]
pub fn estimate_impatience(
    reward_a: f32,
    delay_a: u32,
    reward_b: f32,
    delay_b: u32,
) -> Option<f32> {
    if !reward_a.is_finite() || !reward_b.is_finite() || reward_a <= 0.0 || reward_b <= 0.0 {
        return None;
    }
    if delay_a == delay_b {
        // Same delay: cannot estimate k
        return None;
    }

    // Order options so (a, t1) is the earlier option
    let (a, t1, b, t2) = if delay_a < delay_b {
        (reward_a, delay_a as f32, reward_b, delay_b as f32)
    } else {
        (reward_b, delay_b as f32, reward_a, delay_a as f32)
    };

    // Later reward must be strictly greater than earlier reward to compensate for delay
    if b <= a {
        return None;
    }

    let numerator = b - a;
    let denominator = a * t2 - b * t1;

    if denominator <= 1e-6 {
        return None;
    }

    let k = numerator / denominator;
    if k.is_finite() && k >= 0.0 {
        Some(k)
    } else {
        None
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Agent-specific time preference profile
///
/// Captures individual differences in discounting behavior
/// Correlates with DOCAS (particularly dopamine and cortisol)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimePreferenceProfile {
    /// Present bias parameter (β in β-δ model)
    /// Range: [0.5, 1.0], typical: 0.8
    pub present_bias: f32,

    /// Long-term discount factor (δ in β-δ model)
    /// Range: [0.9, 0.99], typical: 0.95
    pub long_term_discount: f32,

    /// Hyperbolic impatience parameter (k)
    /// Range: [0.01, 0.5], typical: 0.1
    pub impatience_k: f32,

    /// Whether agent is "sophisticated" (aware of their bias)
    /// vs "naive" (unaware, constantly planning to start tomorrow)
    pub is_sophisticated: bool,
}

impl Default for TimePreferenceProfile {
    fn default() -> Self {
        Self {
            present_bias: 0.85,       // Typical present bias
            long_term_discount: 0.95, // 5% discount per tick
            impatience_k: 0.1,        // Standard hyperbolic parameter
            is_sophisticated: false,  // Most agents are naive
        }
    }
}

impl TimePreferenceProfile {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create time preference profile from genome (heritable traits)
    ///
    /// **Heritability:**
    /// Uses genome trait indices 20-21 (loci 40-43):
    /// - Trait 20 (loci 40-41): Present bias β [0.5, 1.0]
    /// - Trait 21 (loci 42-43): Impatience k [0.01, 0.5]
    ///
    /// **Note:** Sophistication (awareness of bias) is determined by
    /// vmPFC development rather than genetics, as it requires
    /// metacognitive capacity that emerges from neural maturation.
    ///
    /// **Returns:** `TimePreferenceProfile` with heritable parameters
    #[must_use]
    pub fn from_genome(genome: &crate::genome::Genome) -> Self {
        // Present bias β: higher values = less present bias (more patient)
        // Range: [0.5, 1.0], typical: 0.85
        let beta_trait = genome.get_trait(20);
        let present_bias = 0.5 + beta_trait * 0.5;

        // Long-term discount δ: derived from present bias
        // More present-biased agents also discount long-term more steeply
        // Range: [0.9, 0.99]
        let long_term_discount = 0.9 + (1.0 - beta_trait) * 0.09;

        // Hyperbolic impatience k: higher = more impatient
        // Range: [0.01, 0.5], typical: 0.1
        let k_trait = genome.get_trait(21);
        let impatience_k = 0.01 + k_trait * 0.49;

        // Sophistication: starts as naive, becomes sophisticated with vmPFC development
        // This is NOT genetic - it requires metacognitive capacity
        let is_sophisticated = false;

        Self {
            present_bias: present_bias.clamp(0.5, 1.0),
            long_term_discount: long_term_discount.clamp(0.9, 0.99),
            impatience_k: impatience_k.clamp(0.01, 0.5),
            is_sophisticated,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a time-consistent agent (no present bias)
    #[must_use]
    pub fn time_consistent(delta: f32) -> Self {
        Self {
            present_bias: 1.0,
            long_term_discount: delta.clamp(0.0, 1.0),
            impatience_k: 0.0, // Will use exponential
            is_sophisticated: true,
        }
    }

    /// Create a highly impatient agent (addiction-like behavior)
    #[must_use]
    pub const fn highly_impatient() -> Self {
        Self {
            present_bias: 0.5,       // Severe present bias
            long_term_discount: 0.9, // 10% discount per tick
            impatience_k: 0.3,       // Very impatient
            is_sophisticated: false,
        }
    }

    /// Create a patient, future-oriented agent
    #[must_use]
    pub const fn future_oriented() -> Self {
        Self {
            present_bias: 0.95,       // Minimal present bias
            long_term_discount: 0.99, // 1% discount per tick
            impatience_k: 0.05,       // Very patient
            is_sophisticated: true,
        }
    }

    /// Compute present value using this profile
    /// Uses quasi-hyperbolic formula
    #[must_use]
    pub fn present_value(&self, reward: f32, delay: u32) -> f32 {
        quasi_hyperbolic_discount(reward, delay, self.present_bias, self.long_term_discount)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute effective annual discount rate
    /// For k=0.1: returns ~0.09 (9% annual rate)
    #[must_use]
    pub fn effective_annual_rate(&self) -> f32 {
        // Approximate from long-term discount
        1.0 - self.long_term_discount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperbolic_discount_immediate() {
        let v = hyperbolic_discount(100.0, 0, 0.1);
        assert!((v - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_hyperbolic_discount_delayed() {
        let v = hyperbolic_discount(100.0, 10, 0.1);
        assert!((v - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_hyperbolic_discount_zero_k() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let v = hyperbolic_discount(100.0, 10, 0.0);
        assert!((v - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_quasi_hyperbolic_discount_immediate() {
        let v = quasi_hyperbolic_discount(100.0, 0, 0.8, 0.95);
        assert!((v - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_quasi_hyperbolic_discount_delayed() {
        let v = quasi_hyperbolic_discount(100.0, 10, 0.8, 0.95);
        assert!(v < 100.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_quasi_hyperbolic_clamping() {
        let v = quasi_hyperbolic_discount(100.0, 1, 2.0, 1.5);
        // Beta and delta should be clamped to [0, 1]
        assert!(v <= 100.0);
    }

    #[test]
    fn test_prefer_immediate_hyperbolic() {
        assert!(prefer_immediate_hyperbolic(50.0, 100.0, 20, 0.1));
        assert!(!prefer_immediate_hyperbolic(30.0, 100.0, 20, 0.1));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_compare_delayed_options() {
        assert!(compare_delayed_options(100.0, 10, 80.0, 10, 0.1));
        assert!(!compare_delayed_options(80.0, 10, 100.0, 10, 0.1));
    }

    #[test]
    fn test_estimate_impatience() {
        let k = estimate_impatience(50.0, 0, 100.0, 10);
        assert!(k.is_some());
        assert!((k.unwrap() - 0.1).abs() < 0.05);
    }

    #[test]
    fn test_estimate_impatience_same_delay() {
        let k = estimate_impatience(50.0, 5, 100.0, 5);
        assert!(k.is_none());
    }

    #[test]
    fn test_time_preference_profile_default() {
        let profile = TimePreferenceProfile::default();
        assert_eq!(profile.present_bias, 0.85);
        assert_eq!(profile.long_term_discount, 0.95);
        assert_eq!(profile.impatience_k, 0.1);
        assert!(!profile.is_sophisticated);
    }

    #[test]
    fn test_time_preference_profile_time_consistent() {
        let profile = TimePreferenceProfile::time_consistent(0.95);
        assert_eq!(profile.present_bias, 1.0);
        assert!(profile.is_sophisticated);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_time_preference_profile_highly_impatient() {
        let profile = TimePreferenceProfile::highly_impatient();
        assert_eq!(profile.present_bias, 0.5);
        assert_eq!(profile.impatience_k, 0.3);
    }

    #[test]
    fn test_time_preference_profile_future_oriented() {
        let profile = TimePreferenceProfile::future_oriented();
        assert_eq!(profile.present_bias, 0.95);
        assert_eq!(profile.long_term_discount, 0.99);
    }

    #[test]
    fn test_present_value() {
        let profile = TimePreferenceProfile::default();
        let v = profile.present_value(100.0, 10);
        assert!(v < 100.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_effective_annual_rate() {
        let profile = TimePreferenceProfile {
            long_term_discount: 0.95,
            ..Default::default()
        };
        let rate = profile.effective_annual_rate();
        assert!((rate - 0.05).abs() < 0.01);
    }
}
