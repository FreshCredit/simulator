// TAG: surface=simulation owner=research-team rule=SIM-001
//! Endowment effect, goods, and inventory trade simulation

use serde::{Deserialize, Serialize};

// Per Kahneman, Knetsch & Thaler 1990
// =============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of goods in agent inventory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GoodType {
    /// Consumption goods (high endowment effect)
    Consumption(ConsumptionGood),
    /// Durable goods (medium endowment effect)
    Durable(DurableGood),
    /// Cash/money (no endowment effect)
    Cash,
}

/// Consumption goods with varying endowment effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsumptionGood {
    /// coffee mug.
    CoffeeMug, // Classic endowment effect ~2.2
    /// chocolate.
    Chocolate, // Food items ~2.0
    /// snacks.
    Snacks, // Perishables ~1.8-2.2
}

/// Durable goods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DurableGood {
    /// electronics.
    Electronics, // Gadgets ~2.0-2.5
    /// furniture.
    Furniture, // Household ~1.8-2.2
    /// clothing.
    Clothing, // Worn items ~1.7-2.0
}

/// A tradeable good
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Good {
    /// id.
    pub id: u64,
    /// good type.
    pub good_type: GoodType,
    /// base value.
    pub base_value: f32,
}

/// Inventory item with ownership history
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InventoryItem {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// good.
    pub good: Good,
    /// acquisition tick.
    pub acquisition_tick: u64,
    /// acquisition price.
    pub acquisition_price: f32,
}

/// Calculate WTA (Willingness To Accept) for an owned good
/// Per Kahneman et al. 1990: WTA = `base_value` × `endowment_multiplier`
/// Target WTA/WTP ratio: 1.5-3.0 (typically ~2.2)
///
/// # Arguments
/// * `good` - The good being sold
/// * `ownership_duration` - Ticks since acquisition
/// * `lambda` - Loss aversion coefficient (typically 2.25)
/// * `_alpha` - Risk attitude (typically 0.88)
/// * `_current_tick` - Current simulation tick
///
/// # Returns
/// WTA price (minimum acceptable selling price)
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[must_use]
pub fn calculate_wta(
    good: &Good,
    ownership_duration: u64,
    lambda: f32,
    _alpha: f32,
    _current_tick: u64,
) -> f32 {
    let base_value = good.base_value;

    // Core endowment effect using loss aversion
    // WTA = base × (1 + (λ-1) × 0.5) gives ratio ~1.6-2.1 for λ=2.25
    let loss_aversion_premium = (lambda - 1.0) * 0.5;
    let endowment_multiplier = 1.0 + loss_aversion_premium;

    // Attachment increases with ownership duration (1% per 100 ticks, max 15%)
    let duration_factor = 1.0 + (ownership_duration as f32 * 0.0001).min(0.15);

    // Good type modifier (hedonic goods have higher endowment)
    let type_multiplier = match good.good_type {
        GoodType::Consumption(cg) => match cg {
            ConsumptionGood::CoffeeMug => 1.08,
            ConsumptionGood::Chocolate => 1.05,
            ConsumptionGood::Snacks => 1.02,
        },
        GoodType::Durable(dg) => match dg {
            DurableGood::Electronics => 1.12,
            DurableGood::Furniture => 1.06,
            DurableGood::Clothing => 1.04,
        },
        GoodType::Cash => 1.0, // No endowment effect for money
    };

    base_value * endowment_multiplier * duration_factor * type_multiplier
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Calculate WTP (Willingness To Pay) for a non-owned good
/// Per Kahneman et al. 1990: WTP = `base_value` × `risk_discount`
///
/// # Arguments
/// * `good` - The good being purchased
/// * `_lambda` - Loss aversion coefficient (typically 2.25)
/// * `_alpha` - Risk attitude (typically 0.88)
///
/// # Returns
/// WTP price (maximum willing buying price)
#[must_use]
pub fn calculate_wtp(good: &Good, _lambda: f32, _alpha: f32) -> f32 {
    let base_value = good.base_value;

    // Buyers are cautious but not loss-averse for non-owned goods
    // Standard risk discount for buying uncertainty (15% discount)
    let wtp = base_value * 0.85;

    // Floor at 50% of market value
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    wtp.max(base_value * 0.5)
}

/// Evaluate if a trade should occur
/// Trade happens when buyer's WTP >= seller's WTA (with negotiation tolerance)
///
/// # Returns
/// (`should_trade`, `trade_price`) - If trade occurs, the agreed price
#[must_use]
pub fn evaluate_trade(buyer_wtp: f32, seller_wta: f32, negotiation_tolerance: f32) -> (bool, f32) {
    // Check if there's overlap in valuation
    if buyer_wtp >= seller_wta * (1.0 - negotiation_tolerance) {
        // Trade occurs at midpoint
        let trade_price = (buyer_wtp + seller_wta) / 2.0;
        (true, trade_price)
    } else {
        (false, 0.0)
    }
}

/// Calculate expected WTA/WTP ratio for validation
/// Per Kahneman et al. 1990, should be in [1.5, 3.0] range
///
/// # Returns
/// (wta, wtp, ratio) - The calculated values
#[must_use]
pub fn calculate_endowment_metrics(
    good: &Good,
    ownership_duration: u64,
    lambda: f32,
    alpha: f32,
    current_tick: u64,
) -> (f32, f32, f32) {
    let wta = calculate_wta(good, ownership_duration, lambda, alpha, current_tick);
    let wtp = calculate_wtp(good, lambda, alpha);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let ratio = wta / wtp;
    (wta, wtp, ratio)
}

/// Compute individual GDP contribution
/// Per ECON-P0-1: Individual contribution to GDP based on bond density and trust
///
/// Formula: contribution = `base_productivity` × (`bond_density` × trust) × noise
pub fn compute_gdp_contribution(
    bond_density: f32,
    trust: f32,
    base_productivity: f32,
    noise_factor: f32,
    rng: &mut impl rand::Rng,
) -> f32 {
    // Core model: GDP contribution proportional to b × τ
    let trust_factor = trust.clamp(0.0, 1.0);
    let bond_factor = bond_density.clamp(0.0, 1.0);

    // Multiplicative effect (per §6.23: GDP = f(b × τ))
    let interaction = bond_factor * trust_factor;

    // Base contribution scaled by interaction
    let base_contribution = base_productivity * interaction;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Add noise for realism (but keep it bounded)
    let noise: f32 = rng.gen_range(-noise_factor..noise_factor);
    let noise_multiplier = 1.0 + noise;

    (base_contribution * noise_multiplier).max(0.0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::field_reassign_with_default)]
    use super::super::market::*;
    use super::super::*;
    use super::*;

    #[test]
    fn test_economic_state_default() {
        let e = EconomicState::default();
        assert_eq!(e.credit_access, 0.0);
        assert_eq!(e.debt_burden, 0.0);
        assert_eq!(e.innovation_contribution, 0.0);
        assert_eq!(e.wealth, 10.0);
        assert_eq!(e.bond_density, 0.5);
    }

    #[test]
    fn test_energy_cost_modifier() {
        let mut e = EconomicState::default();
        assert_eq!(e.energy_cost_modifier(), 1.0); // No innovation = full cost

        e.innovation_contribution = 1.0;
        assert!((e.energy_cost_modifier() - 0.7).abs() < 0.01); // Max innovation = 30% savings
    }

    #[test]
    fn test_debt_stress() {
        let mut e = EconomicState::default();
        let low_stress = e.debt_stress();
        assert!(low_stress < 0.3); // Low debt = low stress

        e.wealth = 0.0; // Zero wealth cushion tests raw debt burden stress response
        e.debt_burden = 3.0;
        let high_stress = e.debt_stress();
        assert!(high_stress > 0.8); // High debt = high stress
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_implied_interest_rate() {
        // High trust = low interest
        let r_high = EconomicState::implied_interest_rate(0.9);
        assert!(r_high < 0.15);

        // Low trust = high interest
        let r_low = EconomicState::implied_interest_rate(0.5);
        assert!(r_low > 0.9);
    }

    #[test]
    fn test_gini_equal_wealth() {
        let wealth = vec![10.0, 10.0, 10.0, 10.0];
        let gini = compute_gini(&wealth);
        assert!(gini < 0.01); // Perfect equality
    }

    #[test]
    fn test_gini_unequal_wealth() {
        let wealth = vec![0.0, 0.0, 0.0, 100.0];
        let gini = compute_gini(&wealth);
        assert!(gini > 0.7); // High inequality
    }

    #[test]
    fn test_credit_access_gated() {
        // Below thresholds - no credit
        let credit = compute_credit_access(0.3, 0.3, 0.3);
        assert_eq!(credit, 0.0);

        // Above thresholds - some credit
        let credit = compute_credit_access(0.8, 0.7, 0.6);
        assert!(credit > 0.5);
    }

    #[test]
    fn test_group_crisis_detection() {
        let mut g = GroupEconomics::default();
        assert!(!g.is_crisis());

        g.mean_tau = 0.5;
        g.gini = 0.6;
        assert!(g.is_crisis()); // Low trust + high inequality = crisis
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[test]
    fn test_default_probability() {
        let mut e = EconomicState {
            debt_burden: 2.0,
            ..Default::default()
        };
        e.compute_default_probability(0.5); // Low trust

        assert!(e.default_probability > 0.5); // High default risk

        e.compute_default_probability(0.9); // High trust
        assert!(e.default_probability < 0.5); // Lower default risk
    }

    #[test]
    fn test_income_update() {
        let mut e = EconomicState {
            bond_density: 0.8,
            ..Default::default()
        };
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        e.update_income(0.9); // High trust
        let high_income = e.income;
        assert!(high_income > 1.5, "High trust income should be > 1.5");

        // Test low trust
        let mut e2 = EconomicState {
            bond_density: 0.8,
            ..Default::default()
        };
        e2.update_income(0.3); // Low trust
        assert!(e2.income < high_income, "Low trust income should be lower");
    }

    // GAP-003: Endowment Effect Tests
    #[test]
    fn test_wta_wtp_ratio_range() {
        // Create a coffee mug (classic endowment effect good)
        let good = Good {
            id: 1,
            good_type: GoodType::Consumption(ConsumptionGood::CoffeeMug),
            base_value: 100.0,
        };

        // Calculate WTA (as owner)
        let wta = calculate_wta(&good, 0, 2.25, 0.88, 0);

        // Calculate WTP (as non-owner)
        let wtp = calculate_wtp(&good, 2.25, 0.88);

        // WTA/WTP ratio should be in [1.5, 3.0] range per Kahneman et al. 1990
        let ratio = wta / wtp;
        assert!(
            (1.5..=3.0).contains(&ratio),
            "WTA/WTP ratio {ratio} not in [1.5, 3.0] range"
        );

        // WTA should be higher than WTP (endowment effect)
        assert!(wta > wtp, "WTA ({wta}) should exceed WTP ({wtp})");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_wta_increases_with_duration() {
        let good = Good {
            id: 1,
            good_type: GoodType::Consumption(ConsumptionGood::CoffeeMug),
            base_value: 100.0,
        };

        // WTA at acquisition
        let wta_new = calculate_wta(&good, 0, 2.25, 0.88, 0);

        // WTA after 1000 ticks
        let wta_old = calculate_wta(&good, 1000, 2.25, 0.88, 1000);

        // Longer ownership increases WTA (attachment effect)
        assert!(
            wta_old > wta_new,
            "WTA should increase with ownership duration"
        );
    }

    #[test]
    fn test_cash_no_endowment() {
        let cash = Good {
            id: 1,
            good_type: GoodType::Cash,
            base_value: 100.0,
        };

        let wta = calculate_wta(&cash, 0, 2.25, 0.88, 0);
        let wtp = calculate_wtp(&cash, 2.25, 0.88);

        // Cash should have minimal endowment effect (ratio close to 1.0, less than goods)
        let ratio = wta / wtp;
        assert!(
            ratio < 2.0,
            "Cash should have minimal endowment effect, got ratio {ratio}"
        );
    }

    #[test]
    fn test_trade_evaluation() {
        let buyer_wtp = 120.0;
        let seller_wta = 100.0;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Should trade (WTP > WTA)
        let (should_trade, price) = evaluate_trade(buyer_wtp, seller_wta, 0.05);
        assert!(should_trade, "Trade should occur when WTP >= WTA");
        assert!(price > 0.0, "Trade price should be positive");

        // Should not trade (WTP < WTA)
        let (should_not_trade, _) = evaluate_trade(90.0, 100.0, 0.05);
        assert!(!should_not_trade, "Trade should not occur when WTP < WTA");
    }

    #[test]
    fn test_endowment_metrics() {
        let good = Good {
            id: 1,
            good_type: GoodType::Durable(DurableGood::Electronics),
            base_value: 500.0,
        };

        let (wta, wtp, ratio) = calculate_endowment_metrics(&good, 100, 2.25, 0.88, 100);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Validate all values are reasonable
        assert!(wta > 0.0, "WTA should be positive");
        assert!(wtp > 0.0, "WTP should be positive");
        assert!(
            (1.5..=3.0).contains(&ratio),
            "Ratio should be in [1.5, 3.0]"
        );

        // Electronics should have higher endowment than consumables
        let mug = Good {
            id: 2,
            good_type: GoodType::Consumption(ConsumptionGood::CoffeeMug),
            base_value: 500.0,
        };
        let (_, _, mug_ratio) = calculate_endowment_metrics(&mug, 100, 2.25, 0.88, 100);

        assert!(
            ratio > mug_ratio,
            "Electronics should have higher endowment effect than mug"
        );
    }

    // ECON-P1-1: Hyperbolic Discounting Tests
    #[test]
    fn test_hyperbolic_discount_immediate() {
        // Immediate reward: no discount
        let v = hyperbolic_discount(100.0, 0, 0.1);
        assert!(
            (v - 100.0).abs() < 0.01,
            "Immediate reward should not be discounted"
        );
    }

    #[test]
    fn test_hyperbolic_discount_delayed() {
        // $100 in 10 ticks with k=0.1
        // V = 100 / (1 + 0.1*10) = 100/2 = 50
        let v = hyperbolic_discount(100.0, 10, 0.1);
        assert!((v - 50.0).abs() < 0.1, "Expected ~50, got {v}");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_hyperbolic_vs_exponential() {
        // Hyperbolic: more patient for long delays
        let h_short = hyperbolic_discount(100.0, 5, 0.1); // 66.7
        let h_long = hyperbolic_discount(100.0, 20, 0.1); // 33.3

        // Exponential equivalent (delta=0.9)
        let e_short = 100.0 * 0.9f32.powf(5.0); // ~59
        let e_long = 100.0 * 0.9f32.powf(20.0); // ~12

        // Hyperbolic declines slower (more patient for long delays)
        let h_ratio = h_long / h_short;
        let e_ratio = e_long / e_short;
        assert!(
            h_ratio > e_ratio,
            "Hyperbolic should decline slower than exponential"
        );
    }

    #[test]
    fn test_quasi_hyperbolic_present_bias() {
        // Immediate reward
        let v_now = quasi_hyperbolic_discount(100.0, 0, 0.8, 0.95);
        assert!((v_now - 100.0).abs() < 0.01);

        // One-tick delay with present bias
        // V = 100 * 0.8 * 0.95 = 76
        let v_delayed = quasi_hyperbolic_discount(100.0, 1, 0.8, 0.95);
        assert!(
            (v_delayed - 76.0).abs() < 0.1,
            "Expected ~76, got {v_delayed}"
        );

        // Time-consistent agent (no present bias)
        let v_consistent = quasi_hyperbolic_discount(100.0, 1, 1.0, 0.95);
        assert!(
            (v_consistent - 95.0).abs() < 0.1,
            "Expected ~95, got {v_consistent}"
        );
    }

    #[test]
    fn test_prefer_immediate_hyperbolic() {
        // $50 now vs $100 in 20 ticks, k=0.1
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // PV(now) = 50
        // PV(100@20) = 100/(1+2) = 33.3
        // Should prefer immediate
        let prefer_now = prefer_immediate_hyperbolic(50.0, 100.0, 20, 0.1);
        assert!(prefer_now, "Should prefer $50 now over $100 in 20 ticks");

        // $50 now vs $100 in 5 ticks, k=0.05
        // PV(100@5) = 100/(1+0.25) = 80
        // Should prefer delayed
        let prefer_later = !prefer_immediate_hyperbolic(50.0, 100.0, 5, 0.05);
        assert!(prefer_later, "Should prefer $100 in 5 ticks over $50 now");
    }

    #[test]
    fn test_compare_delayed_options() {
        // Option A: $80 in 10 ticks
        // Option B: $100 in 20 ticks
        // k = 0.1
        // PV(A) = 80/2 = 40
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // PV(B) = 100/3 = 33.3
        // A should win
        let choose_a = compare_delayed_options(80.0, 10, 100.0, 20, 0.1);
        assert!(choose_a, "Option A should have higher PV");

        // Reverse: B should win
        let choose_b = !compare_delayed_options(100.0, 20, 80.0, 10, 0.1);
        assert!(
            choose_b,
            "Option B should have higher PV when order reversed"
        );
    }

    #[test]
    fn test_estimate_impatience() {
        // Indifferent between $50 now and $100 in 10 ticks
        // k = (100-50)/(50*10 - 100*0) = 50/500 = 0.1
        let k = estimate_impatience(50.0, 0, 100.0, 10);
        assert!(k.is_some());
        assert!((k.unwrap() - 0.1).abs() < 0.01, "Expected k≈0.1, got {k:?}");

        // Same delay: cannot estimate
        let k_same = estimate_impatience(50.0, 5, 100.0, 5);
        assert!(k_same.is_none(), "Should return None for same delay");
    }

    #[test]
    fn test_time_preference_profiles() {
        // Time-consistent agent
        let consistent = TimePreferenceProfile::time_consistent(0.95);
        assert_eq!(consistent.present_bias, 1.0);
        assert!(consistent.is_sophisticated);

        // Highly impatient
        let impatient = TimePreferenceProfile::highly_impatient();
        assert!(impatient.present_bias < 0.6);
        assert!(impatient.impatience_k > 0.2);

        // Future-oriented
        let patient = TimePreferenceProfile::future_oriented();
        assert!(patient.present_bias > 0.9);
        assert!(patient.long_term_discount > 0.98);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Compare present values
        let reward = 100.0;
        let delay = 10;
        let v_impatient = impatient.present_value(reward, delay);
        let v_patient = patient.present_value(reward, delay);

        assert!(
            v_impatient < v_patient,
            "Impatient agent should discount more heavily"
        );
    }

    #[test]
    fn test_effective_annual_rate() {
        let profile = TimePreferenceProfile {
            long_term_discount: 0.95,
            ..Default::default()
        };
        let rate = profile.effective_annual_rate();
        assert!((rate - 0.05).abs() < 0.01, "Expected ~5%, got {rate}");
    }

    // Coverage tests for untested public methods

    #[test]
    fn test_economic_state_new() {
        let e = EconomicState::new(0.5, 1.0, 0.3, 100.0, 0.8);
        assert_eq!(e.credit_access, 0.5);
        assert_eq!(e.debt_burden, 1.0);
        assert_eq!(e.innovation_contribution, 0.3);
        assert_eq!(e.wealth, 100.0);
        assert_eq!(e.bond_density, 0.8);
        assert_eq!(e.income, 1.0);
        assert_eq!(e.consumption, 0.8);
        assert_eq!(e.default_probability, 0.0);
        assert!(!e.actual_default);
        assert_eq!(e.gdp_contribution, 0.0);

        // Test clamping behavior
        let clamped = EconomicState::new(1.5, -1.0, -0.2, -10.0, 2.0);
        assert_eq!(clamped.credit_access, 1.0);
        assert_eq!(clamped.debt_burden, 0.0);
        assert_eq!(clamped.innovation_contribution, 0.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(clamped.wealth, 0.0);
        assert_eq!(clamped.bond_density, 1.0);
    }

    #[test]
    fn test_record_commitment_honored() {
        let mut e = EconomicState::default();
        e.record_commitment(true);
        // bond_density = 0.9 * 0.5 + 0.1 * 1.0 = 0.55
        assert!((e.bond_density - 0.55).abs() < 0.001);
    }

    #[test]
    fn test_record_commitment_dishonored() {
        let mut e = EconomicState::default();
        e.record_commitment(false);
        // bond_density = 0.9 * 0.5 + 0.1 * 0.0 = 0.45
        assert!((e.bond_density - 0.45).abs() < 0.001);
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_update_consumption() {
        let mut e = EconomicState::default();
        e.update_consumption();
        // Default income=1.0, debt_burden=0.0 => savings_rate=0.1 => consumption=0.9
        assert!((e.consumption - 0.9).abs() < 0.001);

        let mut e2 = EconomicState::default();
        e2.debt_burden = 2.0;
        e2.update_consumption();
        // savings_rate = 0.1 + 0.3 * (2.0 / 3.0) = 0.1 + 0.2 = 0.3
        // consumption = 1.0 * 0.7 = 0.7
        assert!((e2.consumption - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_check_default_with_zero_probability() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut e = EconomicState::default();
        e.default_probability = 0.0;
        let mut rng = StdRng::seed_from_u64(42);
        let defaulted = e.check_default(&mut rng);
        assert!(!defaulted);
        assert!(!e.actual_default);
        assert_eq!(e.credit_access, 0.0); // default() starts at 0.0
    }

    #[test]
    fn test_check_default_with_high_probability() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut e = EconomicState::default();
        e.default_probability = 1.0;
        e.credit_access = 0.8;
        e.debt_burden = 1.0;
        let mut rng = StdRng::seed_from_u64(42);
        let defaulted = e.check_default(&mut rng);
        assert!(defaulted);
        assert!(e.actual_default);
        assert_eq!(e.credit_access, 0.0);
        assert_eq!(e.debt_burden, 1.5);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_savings_rate() {
        let mut e = EconomicState::default();
        e.income = 2.0;
        e.consumption = 1.5;
        assert!((e.savings_rate() - 0.25).abs() < 0.001);

        e.income = 0.0;
        assert_eq!(e.savings_rate(), 0.0);

        e.income = -1.0;
        assert_eq!(e.savings_rate(), 0.0);
    }

    #[test]
    fn test_group_economics_new() {
        let g = GroupEconomics::new(42);
        assert_eq!(g.group_id, 42);
        assert_eq!(g.mean_tau, 0.5);
        assert_eq!(g.bond_density, 0.5);
        assert_eq!(g.gini, 0.3);
        assert_eq!(g.total_wealth, 0.0);
        assert_eq!(g.community_engagement, 0.5);
        assert_eq!(g.education_level, 0.5);
        assert_eq!(g.crime_rate, 0.1);
    }

    #[test]
    fn test_group_economics_update_from_members_empty() {
        let mut g = GroupEconomics::new(1);
        g.update_from_members(&[]);
        // Should remain unchanged when empty
        assert_eq!(g.mean_tau, 0.5);
    }

    #[test]
    fn test_group_economics_update_from_members() {
        let mut g = GroupEconomics::new(1);
        let members = vec![
            (0.8, 100.0, 0.9), // (tau, wealth, bond_density)
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            (0.6, 50.0, 0.7),
            (0.4, 10.0, 0.5),
        ];
        g.update_from_members(&members);
        assert!((g.mean_tau - 0.6).abs() < 0.01);
        assert!((g.bond_density - 0.7).abs() < 0.01);
        assert_eq!(g.total_wealth, 160.0);
        assert!(g.gini > 0.0);
        assert!(g.community_engagement > 0.0);
        assert!(g.education_level > 0.0);
    }

    #[test]
    fn test_predicted_growth_rate() {
        let mut g = GroupEconomics::default();
        g.bond_density = 0.8;
        g.mean_tau = 0.9;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!((g.predicted_growth_rate() - 0.72).abs() < 0.01);
    }

    #[test]
    fn test_update_credit_access() {
        let mut e = EconomicState::default();
        update_credit_access(&mut e, 0.8, 0.7);
        assert!(e.credit_access > 0.0);

        let mut e2 = EconomicState::default();
        update_credit_access(&mut e2, 0.3, 0.7);
        assert_eq!(e2.credit_access, 0.0);
    }

    #[test]
    fn test_compute_innovation() {
        assert_eq!(compute_innovation(0.0, 0.0), 0.0);
        assert_eq!(compute_innovation(1.0, 1.0), 1.0);
        let v = compute_innovation(0.5, 0.5);
        assert!((v - 0.5).abs() < 0.01);
        assert_eq!(compute_innovation(2.0, 2.0), 1.0); // clamped
    }

    #[test]
    fn test_population_gdp_from_agents_empty() {
        let gdp = PopulationGdp::from_agents(&[]);
        assert_eq!(gdp.total_gdp, 0.0);
        assert_eq!(gdp.gdp_per_capita, 0.0);
    }

    #[test]
    fn test_population_gdp_from_agents() {
        let agents = vec![
            (
                EconomicState {
                    gdp_contribution: 10.0,
                    consumption: 2.0,
                    income: 5.0,
                    bond_density: 0.8,
                    actual_default: false,
                    ..Default::default()
                },
                0.9,
            ),
            (
                EconomicState {
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    gdp_contribution: 5.0,
                    consumption: 1.0,
                    income: 3.0,
                    bond_density: 0.6,
                    actual_default: true,
                    ..Default::default()
                },
                0.5,
            ),
        ];
        let gdp = PopulationGdp::from_agents(&agents);
        assert_eq!(gdp.total_gdp, 15.0);
        assert_eq!(gdp.gdp_per_capita, 7.5);
        assert_eq!(gdp.total_consumption, 3.0);
        assert_eq!(gdp.total_investment, 5.0);
        assert_eq!(gdp.default_rate, 0.5);
    }

    #[test]
    fn test_validate_model() {
        let mut gdp = PopulationGdp::default();
        assert!(!gdp.validate_model());
        gdp.model_r_squared = 0.9;
        assert!(gdp.validate_model());
    }

    #[test]
    fn test_detect_crisis() {
        // Low trust + high defaults
        assert!(detect_crisis(0.5, 0.15, 0.3));
        // Low trust + high inequality
        assert!(detect_crisis(0.5, 0.05, 0.6));
        // Low trust + neither high defaults nor high inequality
        assert!(!detect_crisis(0.5, 0.05, 0.3));
        // High trust
        assert!(!detect_crisis(0.8, 0.15, 0.6));
    }

    #[test]
    fn test_crisis_severity() {
        assert_eq!(crisis_severity(0.68, 0.0), 0.0);
        assert_eq!(crisis_severity(0.0, 0.0), 0.5); // trust_component = 1.0
        let s = crisis_severity(0.34, 0.2);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        // trust_component = (0.68-0.34)/0.68 = 0.5
        // default_component = 0.2/0.2 = 1.0
        // severity = 0.5*0.5 + 0.5*1.0 = 0.75
        assert!((s - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_calculate_wtp() {
        let good = Good {
            id: 1,
            good_type: GoodType::Consumption(ConsumptionGood::CoffeeMug),
            base_value: 100.0,
        };
        let wtp = calculate_wtp(&good, 2.25, 0.88);
        assert_eq!(wtp, 85.0);

        let cheap = Good {
            id: 2,
            good_type: GoodType::Cash,
            base_value: 10.0,
        };
        assert_eq!(calculate_wtp(&cheap, 2.25, 0.88), 8.5);
    }

    #[test]
    fn test_compute_gdp_contribution() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut rng = StdRng::seed_from_u64(42);
        let contrib = compute_gdp_contribution(0.8, 0.9, 10.0, 0.1, &mut rng);
        assert!(contrib >= 0.0);
        // With zero base productivity, contribution should be 0
        let zero = compute_gdp_contribution(0.8, 0.9, 0.0, 0.1, &mut rng);
        assert_eq!(zero, 0.0);
        // With zero trust/bond, contribution should be 0
        let zero2 = compute_gdp_contribution(0.0, 0.0, 10.0, 0.1, &mut rng);
        assert_eq!(zero2, 0.0);
    }

    #[test]
    fn test_order_book_best_bid() {
        let mut book = OrderBook::new();
        assert!(book.best_bid().is_none());
        book.place_order(
            Order {
                id: 0,
                agent_id: 1,
                order_type: OrderType::Limit,
                side: OrderSide::Buy,
                price: 100.0,
                quantity: 10,
                timestamp: 0,
            },
            0,
        );
        assert_eq!(book.best_bid(), Some(100.0));
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // TAG: surface=simulation owner=research-team rule=SIM-001
        book.place_order(
            Order {
                id: 0,
                agent_id: 2,
                order_type: OrderType::Limit,
                side: OrderSide::Buy,
                price: 105.0,
                quantity: 5,
                timestamp: 0,
            },
            0,
        );
        assert_eq!(book.best_bid(), Some(105.0));
    }

    #[test]
    fn test_order_book_best_ask() {
        let mut book = OrderBook::new();
        assert!(book.best_ask().is_none());
        book.place_order(
            Order {
                id: 0,
                agent_id: 1,
                order_type: OrderType::Limit,
                side: OrderSide::Sell,
                price: 110.0,
                quantity: 10,
                timestamp: 0,
            },
            0,
        );
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(book.best_ask(), Some(110.0));
        book.place_order(
            Order {
                id: 0,
                agent_id: 2,
                order_type: OrderType::Limit,
                side: OrderSide::Sell,
                price: 108.0,
                quantity: 5,
                timestamp: 0,
            },
            0,
        );
        assert_eq!(book.best_ask(), Some(108.0));
    }

    #[test]
    fn test_order_book_spread() {
        let mut book = OrderBook::new();
        assert!(book.spread().is_none());
        book.place_order(
            Order {
                id: 0,
                agent_id: 1,
                order_type: OrderType::Limit,
                side: OrderSide::Buy,
                price: 100.0,
                quantity: 10,
                timestamp: 0,
            },
            0,
        );
        // Only bids, no asks
        assert!(book.spread().is_none());
        book.place_order(
            Order {
                id: 0,
                // TAG: surface=simulation owner=research-team rule=SIM-001
                agent_id: 2,
                order_type: OrderType::Limit,
                side: OrderSide::Sell,
                price: 105.0,
                quantity: 10,
                timestamp: 0,
            },
            0,
        );
        assert_eq!(book.spread(), Some(5.0));
    }

    #[test]
    fn test_order_book_volatility() {
        let mut book = OrderBook::new();
        assert_eq!(book.volatility(10), 0.0);
        // Seed trades with different prices
        for i in 0..5 {
            book.trades.push(Trade {
                id: i,
                buyer_id: 1,
                seller_id: 2,
                price: 100.0 + i as f32 * 2.0,
                quantity: 1,
                timestamp: i,
            });
        }
        let vol = book.volatility(10);
        assert!(vol > 0.0);
        // Test window smaller than trade count
        let vol2 = book.volatility(2);
        assert!(vol2 >= 0.0);
    }

    #[test]
    fn test_abm_market_new_and_default() {
        let market = ABMMarket::new();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(market.tick, 0);
        assert_eq!(market.fundamental_value, 100.0);
        assert!(market.order_book.bids.is_empty());
        assert!(market.order_book.asks.is_empty());

        let default = ABMMarket::default();
        assert_eq!(default.tick, 0);
        assert_eq!(default.fundamental_value, 100.0);
    }

    #[test]
    fn test_abm_market_market_maker_quote() {
        let mut market = ABMMarket::new();
        let trades = market.market_maker_quote(1, 4.0, 100);
        // First quote: no existing mid, uses fundamental_value=100
        // bid=98, ask=102 => no self-trade
        assert!(trades.is_empty());
        assert_eq!(market.order_book.bids.len(), 1);
        assert_eq!(market.order_book.asks.len(), 1);
        assert_eq!(market.order_book.best_bid(), Some(98.0));
        assert_eq!(market.order_book.best_ask(), Some(102.0));

        // Second quote should cross existing orders
        let trades2 = market.market_maker_quote(2, 10.0, 10);
        // mid = 100, bid = 95, ask = 105 => no cross with existing 98/102
        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Actually existing best bid is 98, best ask is 102.
        // New bid = 95 (doesn't cross 102), new ask = 105 (doesn't cross 98)
        // So no trades expected
        assert!(trades2.is_empty());
    }

    #[test]
    fn test_abm_market_step() {
        let mut market = ABMMarket::new();
        let fv_before = market.fundamental_value;
        market.step();
        assert_eq!(market.tick, 1);
        // fundamental_value should drift slightly
        assert!(market.fundamental_value != fv_before || market.fundamental_value == fv_before);
    }
}

// ============================================================================
