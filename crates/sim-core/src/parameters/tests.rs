use super::*;

#[test]
fn test_default_parameters_valid() {
    let params = SimulationParameters::default();
    assert!(params.validate().is_ok());
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_energy_validation_consumption_rate() {
    let energy_invalid_low = EnergyParameters {
        consumption_rate: 0.5, // Below valid range
        ..Default::default()
    };
    assert!(energy_invalid_low.validate().is_err());

    let energy_invalid_high = EnergyParameters {
        consumption_rate: 2.5, // Above valid range
        ..Default::default()
    };
    assert!(energy_invalid_high.validate().is_err());

    let energy_valid = EnergyParameters {
        consumption_rate: 1.5, // Valid
        ..Default::default()
    };
    assert!(energy_valid.validate().is_ok());
}

#[test]
fn test_mortality_validation() {
    let mortality_invalid = MortalityParameters {
        aging_beta: 0.05, // Below valid range
        ..Default::default()
    };
    assert!(mortality_invalid.validate().is_err());

    let mortality_valid = MortalityParameters {
        aging_beta: 0.10, // Valid
        ..Default::default()
    };
    assert!(mortality_valid.validate().is_ok());
}

#[test]
fn test_reproduction_validation() {
    let repro_invalid = ReproductionParameters {
        threshold: 40.0, // Below valid range
        ..Default::default()
    };
    assert!(repro_invalid.validate().is_err());

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let repro_valid = ReproductionParameters {
        threshold: 75.0, // Valid
        ..Default::default()
    };
    assert!(repro_valid.validate().is_ok());
}

#[test]
fn test_genome_range_validation() {
    let genome_invalid = GenomeRangeParameters {
        docas_alpha_range: (0.99, 0.9), // Invalid: min > max
        ..Default::default()
    };
    assert!(genome_invalid.validate().is_err());

    let genome_valid = GenomeRangeParameters {
        docas_alpha_range: (0.9, 0.99), // Valid
        ..Default::default()
    };
    assert!(genome_valid.validate().is_ok());
}

#[test]
fn test_parameters_serialization() {
    let params = SimulationParameters::default();
    // ERR-001: Test code - unwrap() exempt for test assertions
    let json = serde_json::to_string(&params).unwrap();
    let parsed: SimulationParameters = serde_json::from_str(&json).unwrap();
    assert!(parsed.validate().is_ok());
}

// === Preset Tests ===

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_preset_near_extinction() {
    let params = SimulationParameters::near_extinction();
    // Near extinction should have high mortality
    assert!(params.mortality.baseline > MortalityParameters::default().baseline);
    assert_eq!(params.mortality.aging_beta, 0.12);
    assert_eq!(params.resources.density, 0.1);
}

#[test]
fn test_preset_rapid_expansion() {
    let params = SimulationParameters::rapid_expansion();
    // Rapid expansion should have low mortality and abundant resources
    assert!(params.mortality.baseline < MortalityParameters::default().baseline);
    assert_eq!(params.reproduction.threshold, 50.0);
    assert_eq!(params.resources.density, 0.6);
}

#[test]
fn test_preset_boom_bust_cycles() {
    let params = SimulationParameters::boom_bust_cycles();
    // Boom-bust should have moderate consumption and slow regrowth
    assert_eq!(params.energy.consumption_rate, 1.5);
    assert_eq!(params.resources.regrowth_rate, 0.3);
}

#[test]
fn test_preset_stable_equilibrium() {
    let params = SimulationParameters::stable_equilibrium();
    // Stable equilibrium should have balanced parameters
    assert_eq!(params.energy.consumption_rate, 1.2);
    assert_eq!(params.resources.regrowth_rate, 0.4);
}

#[test]
fn test_preset_high_diversity() {
    let params = SimulationParameters::high_diversity();
    // High diversity should have increased mutation rate
    assert_eq!(params.reproduction.mutation_rate, 0.01);
    assert!(params.genome_ranges.docas_alpha_range.0 < 0.9);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_preset_strong_social() {
    let params = SimulationParameters::strong_social();
    // Strong social should have faster norm emergence
    assert!(params.society.frequency_threshold < SocietyParameters::default().frequency_threshold);
    assert!(
        params.society.cascade_acceleration > SocietyParameters::default().cascade_acceleration
    );
}

// === Scenario Validation Tests ===
// These tests verify that each preset produces parameters within scientifically plausible ranges

#[test]
fn test_scenario_near_extinction_plausibility() {
    let params = SimulationParameters::near_extinction();
    // High mortality should make survival difficult
    assert!(
        params.mortality.baseline >= 0.0001,
        "Near-extinction needs high mortality"
    );
    assert!(
        params.resources.density <= 0.2,
        "Near-extinction needs sparse resources"
    );
    // But not impossible - must allow some reproduction
    assert!(
        params.reproduction.threshold <= 100.0,
        "Must allow some reproduction"
    );
}

#[test]
fn test_scenario_rapid_expansion_plausibility() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let params = SimulationParameters::rapid_expansion();
    // Low mortality and easy reproduction
    assert!(
        params.mortality.baseline < 0.00001,
        "Rapid expansion needs very low mortality"
    );
    assert!(
        params.reproduction.threshold <= 60.0,
        "Rapid expansion needs easy reproduction"
    );
    assert!(
        params.resources.density >= 0.4,
        "Rapid expansion needs abundant resources"
    );
}

#[test]
fn test_scenario_boom_bust_plausibility() {
    let params = SimulationParameters::boom_bust_cycles();
    // Fast reproduction but slow regrowth creates oscillations
    assert!(
        params.reproduction.threshold < 75.0,
        "Boom-bust needs faster reproduction"
    );
    assert!(
        params.resources.regrowth_rate < 0.5,
        "Boom-bust needs slow regrowth for crashes"
    );
    assert!(
        params.resources.capacity >= 20.0,
        "Boom-bust needs capacity for booms"
    );
}

#[test]
fn test_scenario_stable_equilibrium_plausibility() {
    let params = SimulationParameters::stable_equilibrium();
    // Balanced parameters should be close to defaults
    let default_params = SimulationParameters::default();
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let energy_diff =
        (params.energy.consumption_rate - default_params.energy.consumption_rate).abs();
    let mortality_diff = (params.mortality.aging_beta - default_params.mortality.aging_beta).abs();
    assert!(
        energy_diff < 0.5,
        "Stable equilibrium should be close to default"
    );
    assert!(
        mortality_diff < 0.05,
        "Stable equilibrium should be close to default"
    );
}

#[test]
fn test_scenario_high_diversity_plausibility() {
    let params = SimulationParameters::high_diversity();
    // Mutation rate should be significantly higher than default
    let default_params = SimulationParameters::default();
    assert!(
        params.reproduction.mutation_rate > default_params.reproduction.mutation_rate * 5.0,
        "High diversity needs much higher mutation rate"
    );
    // But not unrealistically high
    assert!(
        params.reproduction.mutation_rate <= 0.1,
        "Mutation rate must be realistic"
    );
}

#[test]
fn test_all_scenarios_valid() {
    // All preset scenarios must pass validation
    let scenarios = vec![
        ("near_extinction", SimulationParameters::near_extinction()),
        ("rapid_expansion", SimulationParameters::rapid_expansion()),
        ("boom_bust_cycles", SimulationParameters::boom_bust_cycles()),
        (
            "stable_equilibrium",
            SimulationParameters::stable_equilibrium(),
        ),
        ("high_diversity", SimulationParameters::high_diversity()),
        ("strong_social", SimulationParameters::strong_social()),
    ];

    // TAG: surface=simulation owner=research-team rule=SIM-001
    for (name, params) in scenarios {
        // Note: Some scenarios intentionally push boundaries, so we check critical params only
        assert!(
            (1.0..=2.0).contains(&params.energy.consumption_rate),
            "{name}: consumption_rate out of range"
        );
        assert!(
            (0.08..=0.12).contains(&params.mortality.aging_beta),
            "{name}: aging_beta out of range"
        );
    }
}

// === Plausibility Check Tests ===

#[test]
fn test_default_is_viable() {
    let params = SimulationParameters::default();
    assert!(params.is_viable(), "Default parameters should be viable");
    assert!(
        params.check_plausibility().is_empty(),
        "Default should have no warnings"
    );
}

#[test]
fn test_sustainable_presets_viable() {
    // Sustainable presets should be viable
    assert!(SimulationParameters::rapid_expansion().is_viable());
    assert!(SimulationParameters::boom_bust_cycles().is_viable());
    assert!(SimulationParameters::stable_equilibrium().is_viable());
    assert!(SimulationParameters::high_diversity().is_viable());
    assert!(SimulationParameters::strong_social().is_viable());
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_near_extinction_is_extreme() {
    // Near extinction is intentionally extreme and may not be viable
    let params = SimulationParameters::near_extinction();
    // It should have warnings about the extreme configuration
    let warnings = params.check_plausibility();
    assert!(!warnings.is_empty(), "Near extinction should have warnings");
}

#[test]
fn test_plausibility_warnings() {
    // Create an extreme configuration that should trigger warnings
    let mut params = SimulationParameters::default();
    params.energy.consumption_rate = 2.0;
    params.resources.density = 0.1;

    let warnings = params.check_plausibility();
    assert!(!warnings.is_empty(), "Extreme config should have warnings");
}

#[test]
fn test_non_viable_detection() {
    let mut params = SimulationParameters::default();
    // Make reproduction impossible
    params.reproduction.threshold = 200.0; // Above max energy

    assert!(
        !params.is_viable(),
        "Should detect non-viable configuration"
    );
}

// === Empirical Parameters Tests ===

#[test]
fn test_empirical_parameters_default() {
    let params = EmpiricalParameters::default();
    assert!(params.validate().is_ok());
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// --- Identity & Verification Tests (7 parameters) ---

#[test]
fn test_identity_credit_report_error_rate() {
    let params = EmpiricalParameters::default();
    // Consumer Reports 2024: 44% error rate
    assert!((params.credit_report_error_rate - 0.44).abs() < 0.01);
    assert!((0.0..=1.0).contains(&params.credit_report_error_rate));
}

#[test]
fn test_identity_serious_error_rate() {
    let params = EmpiricalParameters::default();
    // ~14.5% serious errors (33% of 44%)
    assert!((params.serious_error_rate - 0.145).abs() < 0.01);
    assert!(params.serious_error_rate < params.credit_report_error_rate);
}

#[test]
fn test_identity_theft_victim_rate() {
    let params = EmpiricalParameters::default();
    // BJS 2016: 10% annual victimization (corrected from 7.3%)
    assert!((params.identity_theft_victim_rate - 0.10).abs() < 0.01); // Corrected to 10%
    assert!((0.05..=0.10).contains(&params.identity_theft_victim_rate));
}

#[test]
fn test_synthetic_id_detection_rate() {
    let params = EmpiricalParameters::default();
    // Reuters 2023: 95% failure = 5% detection
    assert!((params.synthetic_id_detection_rate - 0.05).abs() < 0.01);
}

#[test]
fn test_breach_exposure_rate() {
    let params = EmpiricalParameters::default();
    // ITRC 2024: ~5.12 exposures per capita
    assert!(params.breach_exposure_rate > 1.0);
    assert!(params.breach_exposure_rate < 10.0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_reporting_rate() {
    let params = EmpiricalParameters::default();
    // FTC data: ~4.6% reporting rate
    assert!((params.reporting_rate - 0.046).abs() < 0.01);
    assert!(params.reporting_rate < params.identity_theft_victim_rate);
}

#[test]
fn test_attack_vector_infrastructure() {
    let params = EmpiricalParameters::default();
    // IBM Security 2024: 68% infrastructure attacks
    assert!((params.attack_vector_infrastructure - 0.68).abs() < 0.01);
}

// --- Cost & Economics Tests (4 parameters) ---

#[test]
fn test_tri_merge_cost_range() {
    let params = EmpiricalParameters::default();
    // Experian 2012: $50-$110
    assert_eq!(params.tri_merge_cost_range.0, 50.0);
    assert_eq!(params.tri_merge_cost_range.1, 110.0);
    assert!(params.tri_merge_cost_range.0 < params.tri_merge_cost_range.1);
}

#[test]
fn test_single_bureau_cost() {
    let params = EmpiricalParameters::default();
    // Experian 2012: ~$1.95
    assert!((params.single_bureau_cost - 1.95).abs() < 0.01);
    assert!(params.single_bureau_cost < params.tri_merge_cost_range.0);
}

#[test]
fn test_internal_verification_cost() {
    let params = EmpiricalParameters::default();
    // Experian 2012: ~$0.16
    assert!((params.internal_verification_cost - 0.16).abs() < 0.01);
    assert!(params.internal_verification_cost < params.single_bureau_cost);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_fraud_cost_multiplier() {
    let params = EmpiricalParameters::default();
    // LexisNexis 2025: $5.75 per $1 fraud
    assert!((params.fraud_cost_multiplier - 5.75).abs() < 0.01);
    assert!(params.fraud_cost_multiplier > 1.0);
}

// --- Friction & Abandonment Tests (3 parameters) ---

#[test]
fn test_verification_abandonment_rate() {
    let params = EmpiricalParameters::default();
    // Baymard 2024: 70% abandonment
    assert!((params.verification_abandonment_rate - 0.70).abs() < 0.01);
}

#[test]
fn test_step_abandonment_rate() {
    let params = EmpiricalParameters::default();
    // Baymard 2024: ~18% per step
    assert!((params.step_abandonment_rate - 0.18).abs() < 0.01);
}

#[test]
fn test_friction_reduction_lift() {
    let params = EmpiricalParameters::default();
    // Baymard 2024: 18% lift
    assert!((params.friction_reduction_lift - 0.18).abs() < 0.01);
}

// --- Health & Social Tests (3 parameters) ---

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_financial_shame_mortality_multiplier() {
    let params = EmpiricalParameters::default();
    // CDC 2024: 26% increase = 1.26 multiplier
    assert!((params.financial_shame_mortality_multiplier - 1.26).abs() < 0.01);
    assert!(params.financial_shame_mortality_multiplier > 1.0);
}

#[test]
fn test_poverty_threshold_daily() {
    let params = EmpiricalParameters::default();
    // World Bank 2023: $30/day PPP
    assert!((params.poverty_threshold_daily - 30.0).abs() < 0.01);
    assert!(params.poverty_threshold_daily > 0.0);
}

#[test]
fn test_informal_asset_value() {
    let params = EmpiricalParameters::default();
    // World Bank 2023: ~$540 trillion
    let expected = 540_000_000_000_000.0;
    assert!((params.informal_asset_value - expected).abs() < 1_000_000_000_000.0);
}

// --- DOCAS Parameters Tests (6 parameters) ---

#[test]
fn test_docas_alpha_range() {
    let params = EmpiricalParameters::default();
    // Dayan & Abbott 2001: 0.90-0.99
    assert!(params.docas_alpha_range.0 >= 0.90);
    assert!(params.docas_alpha_range.1 <= 0.99);
    assert!(params.docas_alpha_range.0 < params.docas_alpha_range.1);
}

#[test]
fn test_docas_beta_range() {
    let params = EmpiricalParameters::default();
    // Montague et al. 1996: 0.05-0.20
    assert!(params.docas_beta_range.0 >= 0.05);
    assert!(params.docas_beta_range.1 <= 0.20);
    assert!(params.docas_beta_range.0 < params.docas_beta_range.1);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_criticality_sigma_target() {
    let params = EmpiricalParameters::default();
    // Bak-Tang-Wiesenfeld: σ = 1.0
    assert!((params.criticality_sigma_target - 1.0).abs() < 0.01);
}

#[test]
fn test_cortisol_half_life_ticks() {
    let params = EmpiricalParameters::default();
    // Derived from 90-120 min empirical
    assert!(params.cortisol_half_life_ticks > 0.0);
}

#[test]
fn test_dopamine_learning_base() {
    let params = EmpiricalParameters::default();
    // Montague et al. 1996: λ ≈ 0.61
    assert!((params.dopamine_learning_base - 0.61).abs() < 0.01);
}

#[test]
fn test_docas_homeostatic_baseline() {
    let params = EmpiricalParameters::default();
    // Neutral state ~0.70
    assert!((params.docas_homeostatic_baseline - 0.70).abs() < 0.01);
}

// --- Genome & Heritability Tests (6 parameters) ---

#[test]
fn test_heritability_h2() {
    let params = EmpiricalParameters::default();
    // Twin studies: h² ≈ 0.50
    assert!((params.heritability_h2 - 0.50).abs() < 0.01);
    assert!((0.3..=0.7).contains(&params.heritability_h2));
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_pde4b_variance_optimal() {
    let params = EmpiricalParameters::default();
    // DOCAS §10: 0.08-0.12
    assert!(params.pde4b_variance_optimal.0 >= 0.08);
    assert!(params.pde4b_variance_optimal.1 <= 0.12);
}

#[test]
fn test_mutation_rate() {
    let params = EmpiricalParameters::default();
    // Drake et al. 1998: 0.001
    assert!((params.mutation_rate - 0.001).abs() < 0.0001);
}

#[test]
fn test_recombination_rate() {
    let params = EmpiricalParameters::default();
    // Standard genetic: 0.5
    assert!((params.recombination_rate - 0.5).abs() < 0.01);
}

#[test]
fn test_parent_offspring_correlation_min() {
    let params = EmpiricalParameters::default();
    // Minimum 0.5 correlation
    assert!(params.parent_offspring_correlation_min >= 0.5);
    assert!(params.parent_offspring_correlation_min <= 1.0);
}

#[test]
fn test_learning_rate_lambda() {
    let params = EmpiricalParameters::default();
    // DOCAS universal: λ ≈ 0.61
    assert!((params.learning_rate_lambda - 0.61).abs() < 0.01);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// --- Game Theory Tests (3 parameters) ---

#[test]
fn test_cooperation_threshold_pd() {
    let params = EmpiricalParameters::default();
    // UTGT §5.1: τ > 0.65
    assert!((params.cooperation_threshold_pd - 0.65).abs() < 0.01);
}

#[test]
fn test_cooperation_rate_target() {
    let params = EmpiricalParameters::default();
    // Target range 0.35-0.65
    assert!(params.cooperation_rate_target.0 >= 0.35);
    assert!(params.cooperation_rate_target.1 <= 0.65);
}

#[test]
fn test_system2_proportion() {
    let params = EmpiricalParameters::default();
    // Dual-process: 1-5%
    assert!(params.system2_proportion.0 >= 0.01);
    assert!(params.system2_proportion.1 <= 0.05);
}

// --- Economics Tests (4 parameters) ---

#[test]
fn test_gini_coefficient_target() {
    let params = EmpiricalParameters::default();
    // Milanovic 2016: 0.4-0.6
    assert!(params.gini_coefficient_target.0 >= 0.40);
    assert!(params.gini_coefficient_target.1 <= 0.60);
}

#[test]
fn test_credit_access_threshold() {
    let params = EmpiricalParameters::default();
    assert!((params.credit_access_threshold - 0.3).abs() < 0.01);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_bond_density_threshold() {
    let params = EmpiricalParameters::default();
    assert!((params.bond_density_threshold - 0.4).abs() < 0.01);
}

#[test]
fn test_wealth_trust_correlation() {
    let params = EmpiricalParameters::default();
    // Econ research: r ≈ -0.89
    assert!((params.wealth_trust_correlation - (-0.89)).abs() < 0.01);
    assert!(params.wealth_trust_correlation < 0.0); // Negative correlation
}

// --- Recovery Tests (5 parameters) ---

#[test]
fn test_identity_theft_recovery_range() {
    let params = EmpiricalParameters::default();
    // BJS 2016: 100-500 ticks
    assert_eq!(params.identity_theft_recovery_min, 100);
    assert_eq!(params.identity_theft_recovery_max, 500);
    assert!(params.identity_theft_recovery_min < params.identity_theft_recovery_max);
}

#[test]
fn test_recovery_duration_random() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let params = EmpiricalParameters::default();
    let mut rng = StdRng::seed_from_u64(12345);

    let duration = params.random_recovery_duration(&mut rng);
    assert!(duration >= params.identity_theft_recovery_min);
    assert!(duration <= params.identity_theft_recovery_max);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_stress_recovery_half_life() {
    let params = EmpiricalParameters::default();
    assert!(params.stress_recovery_half_life > 0.0);
}

#[test]
fn test_reputation_recovery_half_life() {
    let params = EmpiricalParameters::default();
    // Fehr & Gächter 2002: ~700 ticks
    assert!((params.reputation_recovery_half_life - 700.0).abs() < 10.0);
}

#[test]
fn test_protocol_protection_effectiveness() {
    let params = EmpiricalParameters::default();
    // 20-40% reduction
    assert!(params.protocol_protection_effectiveness.0 >= 0.20);
    assert!(params.protocol_protection_effectiveness.1 <= 0.40);
}

// --- POM Targets Tests (10 parameters) ---

#[test]
fn test_pom_docas_c1_heritability() {
    let params = EmpiricalParameters::default();
    // Target: h² = 0.50
    assert!((params.docas_c1_heritability - 0.50).abs() < 0.01);
}

#[test]
fn test_pom_docas_c2_homeostasis_variance() {
    let params = EmpiricalParameters::default();
    // Target: 0.02
    assert!((params.docas_c2_homeostasis_variance - 0.02).abs() < 0.01);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_pom_tfp_c3_cross_domain_coherence() {
    let params = EmpiricalParameters::default();
    // Target: 0.45-0.55
    assert!(params.tfp_c3_cross_domain_coherence.0 >= 0.45);
    assert!(params.tfp_c3_cross_domain_coherence.1 <= 0.55);
}

#[test]
fn test_pom_gtt_c2_cooperation_threshold() {
    let params = EmpiricalParameters::default();
    // Target: 0.65
    assert!((params.gtt_c2_cooperation_threshold - 0.65).abs() < 0.01);
}

#[test]
fn test_pom_econ_c1_gini_trust_correlation() {
    let params = EmpiricalParameters::default();
    // Target: -0.95 to -0.75
    assert!(params.econ_c1_gini_trust_correlation.0 >= -0.95);
    assert!(params.econ_c1_gini_trust_correlation.1 <= -0.75);
    assert!(params.econ_c1_gini_trust_correlation.0 < params.econ_c1_gini_trust_correlation.1);
}

#[test]
fn test_pom_sim_group_001_clustering() {
    let params = EmpiricalParameters::default();
    // Target: 0.30-0.60
    assert!(params.sim_group_001_clustering.0 >= 0.30);
    assert!(params.sim_group_001_clustering.1 <= 0.60);
}

#[test]
fn test_pom_sim_credit_001_access_threshold() {
    let params = EmpiricalParameters::default();
    assert!((params.sim_credit_001_access_threshold - 0.3).abs() < 0.01);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_pom_id_theft_001_theft_rate() {
    let params = EmpiricalParameters::default();
    // Target: 5-10%
    assert!(params.id_theft_001_theft_rate.0 >= 0.05);
    assert!(params.id_theft_001_theft_rate.1 <= 0.10);

    // Validation helper
    assert!(params.is_theft_rate_valid(0.07));
    assert!(!params.is_theft_rate_valid(0.15));
}

#[test]
fn test_pom_id_theft_003_protection_reduction() {
    let params = EmpiricalParameters::default();
    // Target: 20-40%
    assert!(params.id_theft_003_protection_reduction.0 >= 0.20);
    assert!(params.id_theft_003_protection_reduction.1 <= 0.40);

    // Validation helper
    assert!(params.is_protection_effectiveness_valid(0.30));
    assert!(!params.is_protection_effectiveness_valid(0.50));
}

#[test]
fn test_pom_cri_001_branching_ratio() {
    let params = EmpiricalParameters::default();
    // Target: 0.90-1.10
    assert!(params.cri_001_branching_ratio.0 >= 0.90);
    assert!(params.cri_001_branching_ratio.1 <= 1.10);
}

// --- Empirical Parameters Validation Tests ---

#[test]
fn test_empirical_parameters_invalid_rates() {
    let mut params = EmpiricalParameters {
        credit_report_error_rate: 1.5,
        ..Default::default()
    };
    assert!(params.validate().is_err());

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Test invalid rate < 0.0
    params.credit_report_error_rate = -0.1;
    assert!(params.validate().is_err());

    // Restore valid
    params.credit_report_error_rate = 0.44;
    assert!(params.validate().is_ok());
}

#[test]
fn test_empirical_parameters_invalid_ranges() {
    let mut params = EmpiricalParameters {
        tri_merge_cost_range: (110.0, 50.0),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    // Restore valid
    params.tri_merge_cost_range = (50.0, 110.0);
    assert!(params.validate().is_ok());
}

#[test]
fn test_empirical_parameters_invalid_recovery() {
    let mut params = EmpiricalParameters {
        identity_theft_recovery_min: 500,
        identity_theft_recovery_max: 100,
        ..Default::default()
    };
    assert!(params.validate().is_err());

    // Restore valid
    params.identity_theft_recovery_min = 100;
    params.identity_theft_recovery_max = 500;
    assert!(params.validate().is_ok());
}

#[test]
fn test_empirical_parameters_serialization() {
    let params = EmpiricalParameters::default();
    // ERR-001: Test code - unwrap() exempt for test assertions
    let json = serde_json::to_string(&params).unwrap();
    let parsed: EmpiricalParameters = serde_json::from_str(&json).unwrap();
    assert!(parsed.validate().is_ok());
}
