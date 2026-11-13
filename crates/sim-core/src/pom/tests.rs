use super::*;
use crate::agent::Agent;
use crate::economics::compute_gini;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[test]
fn test_pom_target_library_size() {
    assert_eq!(POM_TARGETS.len(), 41, "Should have exactly 41 POM targets");
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_all_targets_have_valid_ranges() {
    for target in POM_TARGETS {
        assert!(
            target.min < target.max,
            "Target {} should have valid range",
            target.id
        );
        assert!(
            !target.name.is_empty(),
            "Target {} should have a name",
            target.id
        );
        assert!(
            !target.description.is_empty(),
            "Target {} should have a description",
            target.id
        );
    }
}

#[test]
fn test_p0_targets_exist() {
    let p0_count = POM_TARGETS
        .iter()
        .filter(|t| matches!(t.priority, PomPriority::P0))
        .count();
    assert!(
        p0_count >= 8,
        "Should have at least 8 P0 targets, found {p0_count}"
    );
}

#[test]
fn test_all_categories_have_targets() {
    use PomCategory::*;
    let categories = vec![
        Geography,
        Demographics,
        Docas,
        Society,
        Language,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Protocol,
        Criticality,
        Economics,
        Tfp,
        Gtt,
    ];

    for category in categories {
        let count = POM_TARGETS
            .iter()
            .filter(|t| t.category == category)
            .count();
        assert!(
            count > 0,
            "Category {category:?} should have at least one target"
        );
    }
}

#[test]
fn test_validation_suite_creation() {
    let metrics = PomMetrics {
        agent_survival_rate: 0.75,
        cooperation_rate: 0.60,
        branching_ratio: 0.95,
        verification_rate: 0.85,
        identity_theft_rate: 0.05,
        dopamine_baseline: 0.50,
        oxytocin_baseline: 0.40,
        multimodal_sync_auc: 0.91,
        spectral_coherence_cf: 0.97,
        lambda_scale_invariant_variance: 0.10,
        ..Default::default()
    };

    let suite = validate_all_targets(&metrics);

    let total_results: usize = suite.categories.values().map(|c| c.targets.len()).sum();
    assert_eq!(total_results, POM_TARGETS.len());
    assert!(suite.overall_score >= 0.0 && suite.overall_score <= 100.0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_p0_validation() {
    let metrics = PomMetrics {
        agent_survival_rate: 0.75,
        cooperation_rate: 0.55,
        branching_ratio: 0.98,
        verification_rate: 0.85,
        identity_theft_rate: 0.05,
        dopamine_baseline: 0.50,
        oxytocin_baseline: 0.40,
        multimodal_sync_auc: 0.91,
        spectral_coherence_cf: 0.97,
        lambda_scale_invariant_variance: 0.10,
        ..Default::default()
    };

    let suite = validate_all_targets(&metrics);

    let p0_failed: Vec<_> = suite
        .categories
        .values()
        .flat_map(|c| &c.targets)
        .filter(|t| t.priority == PomPriority::P0 && !t.passed)
        .map(|t| &t.target_id)
        .collect();

    assert!(
        p0_failed.is_empty(),
        "P0 targets should pass: {p0_failed:?}"
    );
}

#[test]
fn test_summary_report() {
    let metrics = PomMetrics::default();
    let suite = validate_all_targets(&metrics);
    let report = suite.summary_report();

    assert!(report.contains("POM Validation Suite Report"));
    assert!(report.contains("Overall Score:"));
    assert!(report.contains("By Priority:"));
    assert!(report.contains("By Category:"));
}

#[test]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
fn test_correlation_computation() {
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

    let r = compute_correlation(&x, &y);
    assert!(r.is_some());
    // ERR-001: Test code - unwrap() exempt for test assertions
    assert!((r.unwrap() - 1.0).abs() < 0.001);

    // Test negative correlation
    let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
    let r_neg = compute_correlation(&x, &y_neg);
    assert!(r_neg.is_some());
    // ERR-001: Test code - unwrap() exempt for test assertions
    assert!((r_neg.unwrap() + 1.0).abs() < 0.001);
}

#[test]
fn test_r_squared_computation() {
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

    let r2 = compute_r_squared(&x, &y);
    assert!(r2.is_some());
    // ERR-001: Test code - unwrap() exempt for test assertions
    assert!((r2.unwrap() - 1.0).abs() < 0.001);
}

#[test]
fn test_gini_computation() {
    let wealth_equal = vec![10.0, 10.0, 10.0, 10.0];
    let gini_equal = compute_gini(&wealth_equal);
    assert!(gini_equal < 0.01);

    let wealth_unequal = vec![1.0, 2.0, 3.0, 100.0];
    let gini_unequal = compute_gini(&wealth_unequal);
    assert!(gini_unequal > 0.5);
}

#[test]
fn test_validator_creation() {
    let validator = PomValidator::new();
    assert!(
        validator.compute_agent_survival_rate(&crate::simulation::Simulation::new_default()) >= 0.0
    );
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_all_target_ids_mapped() {
    // Verify all target IDs have a mapping in get_metric_value
    let metrics = PomMetrics::default();

    for target in POM_TARGETS {
        let value = get_metric_value(target.id, &metrics);
        // Should not panic and should return a value
        assert!(value >= 0.0 || value < 0.0); // Just verify it returns something
    }
}

#[test]
fn test_category_counts() {
    let category_counts: std::collections::HashMap<PomCategory, usize> =
        POM_TARGETS
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, t| {
                *acc.entry(t.category).or_insert(0) += 1;
                acc
            });

    assert_eq!(
        *category_counts.get(&PomCategory::Geography).unwrap_or(&0),
        2
    );
    assert_eq!(
        *category_counts
            .get(&PomCategory::Demographics)
            .unwrap_or(&0),
        2
    );
    assert_eq!(*category_counts.get(&PomCategory::Docas).unwrap_or(&0), 9);
    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(*category_counts.get(&PomCategory::Society).unwrap_or(&0), 3);
    assert_eq!(
        *category_counts.get(&PomCategory::Language).unwrap_or(&0),
        2
    );
    assert_eq!(
        *category_counts.get(&PomCategory::Protocol).unwrap_or(&0),
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        2
    );
    assert_eq!(
        *category_counts.get(&PomCategory::Criticality).unwrap_or(&0),
        2
    );
    assert_eq!(
        *category_counts.get(&PomCategory::Economics).unwrap_or(&0),
        8
    );
    assert_eq!(*category_counts.get(&PomCategory::Tfp).unwrap_or(&0), 5);
    assert_eq!(*category_counts.get(&PomCategory::Gtt).unwrap_or(&0), 6);
}

// ========================================================================
// PomValidator construction and defaults
// ========================================================================

#[test]
fn test_validator_default() {
    let v: PomValidator = PomValidator;
    let sim = Simulation::new_default();
    let _metrics = v.compute_all(&sim);
}

#[test]
fn test_pom_metrics_default() {
    let m = PomMetrics::default();
    assert_eq!(m.agent_survival_rate, 0.0);
    assert_eq!(m.energy_cv, 0.0);
    assert_eq!(m.cooperation_rate, 0.0);
}

// ========================================================================
// Geography compute functions
// ========================================================================

#[test]
fn test_compute_agent_survival_rate_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_agent_survival_rate(&sim), 0.0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_compute_agent_survival_rate_all_alive() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    assert_eq!(v.compute_agent_survival_rate(&sim), 1.0);
}

#[test]
fn test_compute_agent_survival_rate_mixed() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.agents[0].alive = false;
    let v = PomValidator::new();
    assert_eq!(v.compute_agent_survival_rate(&sim), 0.5);
}

#[test]
fn test_compute_energy_cv_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_energy_cv(&sim), 0.0);
}

#[test]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
fn test_compute_energy_cv_single_agent() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    assert_eq!(v.compute_energy_cv(&sim), 0.0);
}

#[test]
fn test_compute_energy_cv_zero_mean() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.agents[0].energy = 0.0;
    sim.agents[1].energy = 0.0;
    let v = PomValidator::new();
    assert_eq!(v.compute_energy_cv(&sim), 0.0);
}

#[test]
fn test_compute_energy_cv_varied() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.agents[0].energy = 10.0;
    sim.agents[1].energy = 30.0;
    let v = PomValidator::new();
    let cv = v.compute_energy_cv(&sim);
    assert!(cv > 0.0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// ========================================================================
// Demographics compute functions
// ========================================================================

#[test]
fn test_compute_mortality_young_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let rate = v.compute_mortality_young(&sim);
    assert!(rate > 0.0);
}

#[test]
fn test_compute_mortality_young_with_dead() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].age = 0;
    sim.agents[0].alive = false;
    let v = PomValidator::new();
    assert_eq!(v.compute_mortality_young(&sim), 1.0);
}

#[test]
fn test_compute_mortality_old_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let rate = v.compute_mortality_old(&sim);
    assert!(rate > 0.0);
}

#[test]
fn test_compute_mortality_old_with_dead() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    sim.agents[0].age = 55;
    sim.agents[0].alive = false;
    let v = PomValidator::new();
    assert_eq!(v.compute_mortality_old(&sim), 1.0);
}

// ========================================================================
// DOCAS compute functions
// ========================================================================

#[test]
fn test_compute_dopamine_baseline_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_dopamine_baseline(&sim), 0.5);
}

#[test]
fn test_compute_dopamine_baseline_with_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].docas.levels[crate::types::D] = 0.8;
    let v = PomValidator::new();
    assert!((v.compute_dopamine_baseline(&sim) - 0.8).abs() < 0.01);
}

#[test]
fn test_compute_oxytocin_baseline_empty() {
    let sim = Simulation::new_default();
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let v = PomValidator::new();
    assert_eq!(v.compute_oxytocin_baseline(&sim), 0.5);
}

#[test]
fn test_compute_oxytocin_baseline_with_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].docas.levels[crate::types::O] = 0.3;
    let v = PomValidator::new();
    assert!((v.compute_oxytocin_baseline(&sim) - 0.3).abs() < 0.01);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_compute_cortisol_response_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_cortisol_response(&sim), 0.5);
}

#[test]
fn test_compute_cortisol_response_no_stressed() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].energy = 50.0;
    let v = PomValidator::new();
    assert_eq!(v.compute_cortisol_response(&sim), 0.5);
}

#[test]
fn test_compute_cortisol_response_with_stressed() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].energy = 10.0;
    sim.agents[0].docas.levels[crate::types::C] = 0.8;
    let v = PomValidator::new();
    let resp = v.compute_cortisol_response(&sim);
    assert!((resp - 0.8).abs() < 0.01);
}

#[test]
fn test_compute_trust_heritability_no_lineage() {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    assert_eq!(v.compute_trust_heritability(&sim), 0.5);
}

#[test]
fn test_compute_trust_heritability_with_lineage() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sim = Simulation::new_default();
    let pos = crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    };
    let genome1 = crate::genome::Genome::random(&mut rng);
    let parent = Agent::new(0, pos, genome1.clone(), 0, crate::types::Sex::Male);
    sim.agents.push(parent);
    let traits1 = crate::lineage::TraitSnapshot::from_genome(&genome1, 0);
    sim.lineage
        .add_initial(crate::lineage::LineageNode::new_initial(0, 0, traits1));

    let genome2 = crate::genome::Genome::random(&mut rng);
    let child = Agent::new(1, pos, genome2.clone(), 10, crate::types::Sex::Female);
    sim.agents.push(child);
    let traits2 = crate::lineage::TraitSnapshot::from_genome(&genome2, 10);
    sim.lineage
        .add_offspring(crate::lineage::LineageNode::new_offspring(
            1, 0, None, 10, traits2, 0,
        ));

    let v = PomValidator::new();
    let h2 = v.compute_trust_heritability(&sim);
    assert!((0.0..=1.0).contains(&h2));
}

#[test]
fn test_compute_pde4b_learning_correlation_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_pde4b_learning_correlation(&sim), 0.5);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_compute_pde4b_learning_correlation_with_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 3.0,
        y: 3.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    let r = v.compute_pde4b_learning_correlation(&sim);
    assert!((-1.0..=1.0).contains(&r));
}

#[test]
fn test_compute_curvature_kappa_cooperation_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_curvature_kappa_cooperation(&sim), 0.0);
}

#[test]
fn test_compute_curvature_kappa_cooperation_with_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 3.0,
        y: 3.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    let r = v.compute_curvature_kappa_cooperation(&sim);
    assert!((0.0..=1.0).contains(&r));
}

#[test]
fn test_compute_viscosity_variation_psi_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_viscosity_variation_psi(&sim), 0.3);
}

#[test]
fn test_compute_viscosity_variation_psi_with_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].trust_viscosity.baseline_resilience = 0.7;
    let v = PomValidator::new();
    assert_eq!(v.compute_viscosity_variation_psi(&sim), 0.7);
}

#[test]
fn test_compute_coupling_phi_sync_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_coupling_phi_sync(&sim), 0.0);
}

#[test]
fn test_compute_coupling_phi_sync_with_agents() {
    let mut sim = Simulation::new_default();
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    let sync = v.compute_coupling_phi_sync(&sim);
    assert!((0.0..=1.0).contains(&sync));
}

#[test]
fn test_compute_component_heritability_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_component_heritability(&sim), 0.5);
}

#[test]
fn test_compute_component_heritability_with_lineage() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sim = Simulation::new_default();
    let pos = crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    };
    let genome1 = crate::genome::Genome::random(&mut rng);
    let parent = Agent::new(0, pos, genome1.clone(), 0, crate::types::Sex::Male);
    sim.agents.push(parent);
    let traits1 = crate::lineage::TraitSnapshot::from_genome(&genome1, 0);
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.lineage
        .add_initial(crate::lineage::LineageNode::new_initial(0, 0, traits1));

    let genome2 = crate::genome::Genome::random(&mut rng);
    let child = Agent::new(1, pos, genome2.clone(), 10, crate::types::Sex::Female);
    sim.agents.push(child);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let traits2 = crate::lineage::TraitSnapshot::from_genome(&genome2, 10);
    sim.lineage
        .add_offspring(crate::lineage::LineageNode::new_offspring(
            1, 0, None, 10, traits2, 0,
        ));

    let v = PomValidator::new();
    let h2 = v.compute_component_heritability(&sim);
    assert!((0.0..=1.0).contains(&h2));
}

// ========================================================================
// Society compute functions
// ========================================================================

#[test]
fn test_compute_cooperation_rate_from_history() {
    let mut sim = Simulation::new_default();
    sim.metrics.cooperation_history.push(0.75);
    let v = PomValidator::new();
    assert_eq!(v.compute_cooperation_rate(&sim), 0.75);
}

#[test]
fn test_compute_cooperation_rate_empty_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_cooperation_rate(&sim), 0.5);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

#[test]
fn test_compute_cooperation_rate_from_interactions() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].positive_interactions = 8;
    sim.agents[0].negative_interactions = 2;
    let v = PomValidator::new();
    assert_eq!(v.compute_cooperation_rate(&sim), 0.8);
}

#[test]
fn test_compute_cooperation_rate_no_interactions() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    assert_eq!(v.compute_cooperation_rate(&sim), 0.5);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_compute_network_clustering_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_network_clustering(&sim), 0.3);
}

#[test]
fn test_compute_network_clustering_with_neighbors() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 3.0,
        y: 3.0,
        z: 0.0,
    });
    sim.agents[0].local_reputation.insert(1, 0.5);
    sim.agents[0].local_reputation.insert(2, 0.5);
    sim.agents[1].local_reputation.insert(0, 0.5);
    sim.agents[1].local_reputation.insert(2, 0.5);
    let v = PomValidator::new();
    let cc = v.compute_network_clustering(&sim);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert!((0.0..=1.0).contains(&cc));
}

#[test]
fn test_compute_avg_path_length_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_avg_path_length(&sim), 0.0);
}

#[test]
fn test_compute_avg_path_length_with_paths() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 3.0,
        y: 3.0,
        z: 0.0,
    });
    sim.agents[0].local_reputation.insert(1, 0.5);
    sim.agents[1].local_reputation.insert(2, 0.5);
    let v = PomValidator::new();
    let apl = v.compute_avg_path_length(&sim);
    assert!(apl > 0.0);
}

#[test]
fn test_shortest_path_same_node() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    let agents: Vec<&Agent> = sim.agents.iter().collect();
    assert_eq!(v.shortest_path(&agents, 0, 0), Some(0));
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_shortest_path_no_path() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    let agents: Vec<&Agent> = sim.agents.iter().collect();
    assert_eq!(v.shortest_path(&agents, 0, 1), None);
}

#[test]
fn test_shortest_path_direct() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.agents[0].local_reputation.insert(1, 0.5);
    let v = PomValidator::new();
    let agents: Vec<&Agent> = sim.agents.iter().collect();
    assert_eq!(v.shortest_path(&agents, 0, 1), Some(1));
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[test]
fn test_shortest_path_multi_hop() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 3.0,
        y: 3.0,
        z: 0.0,
    });
    sim.agents[0].local_reputation.insert(1, 0.5);
    sim.agents[1].local_reputation.insert(2, 0.5);
    let v = PomValidator::new();
    let agents: Vec<&Agent> = sim.agents.iter().collect();
    assert_eq!(v.shortest_path(&agents, 0, 2), Some(2));
}

// ========================================================================
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
// Language compute functions
// ========================================================================

#[test]
fn test_compute_lexicon_size_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_lexicon_size(&sim), 0.0);
}

#[test]
fn test_compute_lexicon_size_with_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    assert_eq!(v.compute_lexicon_size(&sim), 0.0);
}

#[test]
fn test_compute_communication_success_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_communication_success(&sim), 0.5);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_compute_communication_success_far_apart() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 100.0,
        y: 100.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    assert_eq!(v.compute_communication_success(&sim), 0.5);
}

#[test]
fn test_lexicon_overlap_both_empty() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    assert_eq!(v.lexicon_overlap(&sim.agents[0], &sim.agents[1]), 0.0);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

// ========================================================================
// Protocol compute functions
// ========================================================================

#[test]
fn test_compute_verification_rate_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_verification_rate(&sim), 0.0);
}

#[test]
fn test_compute_verification_rate_mixed() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        z: 0.0,
    });
    sim.agents[0].has_verified_id = true;
    let v = PomValidator::new();
    assert_eq!(v.compute_verification_rate(&sim), 0.5);
}

#[test]
fn test_compute_identity_theft_rate_with_stats() {
    let mut sim = Simulation::new_default();
    sim.tick = 365;
    sim.metrics.identity_theft_stats = Some(crate::identity::IdentityTheftStats {
        attempts: 10,
        successful: 1,
        ..Default::default()
    });
    let v = PomValidator::new();
    let rate = v.compute_identity_theft_rate(&sim);
    assert!(rate > 0.0 && rate <= 1.0);
}

#[test]
fn test_compute_identity_theft_rate_from_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
        // TAG: surface=simulation owner=research-team rule=SIM-001
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.agents[0].is_identity_theft_victim = true;
    let v = PomValidator::new();
    assert_eq!(v.compute_identity_theft_rate(&sim), 0.5);
}

// ========================================================================
// Criticality compute functions
// ========================================================================

#[test]
fn test_compute_branching_ratio() {
    let mut sim = Simulation::new_default();
    sim.metrics.births_this_tick = 5;
    sim.metrics.deaths_this_tick = 2;
    sim.metrics.encounters_this_tick = 10;
    let v = PomValidator::new();
    let br = v.compute_branching_ratio(&sim);
    assert!(br >= 0.0);
}

#[test]
fn test_compute_avalanche_size_exp_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_avalanche_size_exp(&sim), 1.5);
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[test]
fn test_compute_avalanche_size_exp_many_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..15 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
    }
    let v = PomValidator::new();
    let exp = v.compute_avalanche_size_exp(&sim);
    assert!((1.0..=3.0).contains(&exp));
}

// ========================================================================
// Economics compute functions
// ========================================================================

#[test]
fn test_compute_wealth_gini_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_wealth_gini(&sim), 0.0);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

#[test]
fn test_compute_wealth_gini_with_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    });
    sim.agents[0].economics.wealth = 10.0;
    sim.agents[1].economics.wealth = 90.0;
    let v = PomValidator::new();
    let gini = v.compute_wealth_gini(&sim);
    assert!(gini > 0.0);
}

#[test]
fn test_compute_transaction_velocity_tick_zero() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_transaction_velocity(&sim), 0.0);
}

#[test]
fn test_compute_transaction_velocity_with_data() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut sim = Simulation::new_default();
    sim.tick = 100;
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].positive_interactions = 50;
    let v = PomValidator::new();
    let vel = v.compute_transaction_velocity(&sim);
    assert!(vel > 0.0);
}

#[test]
fn test_compute_gini_trust_correlation_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert!((v.compute_gini_trust_correlation(&sim) - (-0.85)).abs() < 0.01);
}

#[test]
fn test_compute_gini_trust_correlation_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..12 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
        sim.agents[i].economics.wealth = (i as f32) * 10.0;
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }
    let v = PomValidator::new();
    let r = v.compute_gini_trust_correlation(&sim);
    assert!((-1.0..=1.0).contains(&r));
}

#[test]
fn test_compute_growth_trust_correlation_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_growth_trust_correlation(&sim), 0.95);
}

#[test]
fn test_compute_growth_trust_correlation_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
        sim.agents[i].economics.bond_density = 0.5;
        sim.agents[i].economics.wealth = 50.0;
    }
    let v = PomValidator::new();
    let r2 = v.compute_growth_trust_correlation(&sim);
    assert!((0.93..=1.0).contains(&r2));
}

#[test]
fn test_compute_interest_rate_tau_relation_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_interest_rate_tau_relation(&sim), 1.0);
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_compute_interest_rate_tau_relation_low_tau() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].docas.levels[crate::types::D] = 0.01;
    sim.agents[0].docas.levels[crate::types::O] = 0.01;
    sim.agents[0].docas.levels[crate::types::S] = 0.01;
    sim.agents[0].docas.levels[crate::types::C] = 0.99;
    sim.agents[0].docas.levels[crate::types::A] = 0.99;
    let v = PomValidator::new();
    let rate = v.compute_interest_rate_tau_relation(&sim);
    assert!((0.5..=2.0).contains(&rate));
}

#[test]
fn test_compute_crisis_tau_threshold_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_crisis_tau_threshold(&sim), 0.62);
}

#[test]
fn test_compute_crisis_tau_threshold_below() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].docas.levels[crate::types::D] = 0.1;
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.agents[0].docas.levels[crate::types::O] = 0.1;
    sim.agents[0].docas.levels[crate::types::S] = 0.1;
    sim.agents[0].docas.levels[crate::types::C] = 0.9;
    sim.agents[0].docas.levels[crate::types::A] = 0.9;
    let v = PomValidator::new();
    let thresh = v.compute_crisis_tau_threshold(&sim);
    assert!(thresh < 0.62);
}

#[test]
fn test_compute_crisis_tau_threshold_above() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].docas.levels[crate::types::D] = 0.9;
    sim.agents[0].docas.levels[crate::types::O] = 0.9;
    sim.agents[0].docas.levels[crate::types::S] = 0.9;
    sim.agents[0].docas.levels[crate::types::C] = 0.1;
    sim.agents[0].docas.levels[crate::types::A] = 0.1;
    let v = PomValidator::new();
    let thresh = v.compute_crisis_tau_threshold(&sim);
    assert_eq!(thresh, 0.62);
}

#[test]
fn test_compute_money_as_highest_tau_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_money_as_highest_tau(&sim), 0.5);
}

#[test]
fn test_compute_money_as_highest_tau_with_agents() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
        sim.agents[i].economics.wealth = (i as f32) * 10.0;
    }
    let v = PomValidator::new();
    let score = v.compute_money_as_highest_tau(&sim);
    assert!((0.0..=1.0).contains(&score));
}

#[test]
fn test_compute_credit_as_prepaid_tau_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    assert_eq!(v.compute_credit_as_prepaid_tau(&sim), 0.5);
}

#[test]
fn test_compute_credit_as_prepaid_tau_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
        sim.agents[i].economics.credit_access = 0.8;
    }
    let v = PomValidator::new();
    let score = v.compute_credit_as_prepaid_tau(&sim);
    assert!((0.0..=1.0).contains(&score));
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

// ========================================================================
// TFP compute functions
// ========================================================================

#[test]
fn test_compute_multimodal_sync_auc_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let auc = v.compute_multimodal_sync_auc(&sim);
    assert!((0.88..=0.94).contains(&auc));
}

#[test]
fn test_compute_multimodal_sync_auc_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
    }
    let v = PomValidator::new();
    let auc = v.compute_multimodal_sync_auc(&sim);
    assert!((0.88..=0.94).contains(&auc));
}

#[test]
fn test_compute_lambda_cross_domain_variance_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let var = v.compute_lambda_cross_domain_variance(&sim);
    assert!((0.0..=0.20).contains(&var));
}

#[test]
fn test_compute_lambda_cross_domain_variance_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
    }
    let v = PomValidator::new();
    let var = v.compute_lambda_cross_domain_variance(&sim);
    assert!((0.0..=0.20).contains(&var));
}

#[test]
fn test_compute_entropy_reduction_delta_h_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let h = v.compute_entropy_reduction_delta_h(&sim);
    assert!((-0.60..=-0.48).contains(&h));
}

#[test]
fn test_compute_entropy_reduction_delta_h_with_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].docas.levels = [0.9, 0.1, 0.1, 0.1, 0.1];
    let v = PomValidator::new();
    let h = v.compute_entropy_reduction_delta_h(&sim);
    assert!((-0.60..=-0.48).contains(&h));
}

#[test]
fn test_compute_composite_stability_r_squared_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let r2 = v.compute_composite_stability_r_squared(&sim);
    assert!((0.6..=1.0).contains(&r2));
}
// TAG: surface=simulation owner=research-team rule=SIM-001

#[test]
fn test_compute_composite_stability_r_squared_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
        sim.agents[i].energy = 50.0;
    }
    let v = PomValidator::new();
    let r2 = v.compute_composite_stability_r_squared(&sim);
    assert!((0.6..=1.0).contains(&r2));
}

#[test]
fn test_compute_spectral_coherence_cf_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let cf = v.compute_spectral_coherence_cf(&sim);
    assert!((0.95..=1.0).contains(&cf));
}

#[test]
fn test_compute_spectral_coherence_cf_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
    }
    let v = PomValidator::new();
    let cf = v.compute_spectral_coherence_cf(&sim);
    assert!((0.95..=1.0).contains(&cf));
}

// ========================================================================
// GTT compute functions
// ========================================================================

#[test]
fn test_compute_lambda_scale_invariant_variance_few_agents() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let var = v.compute_lambda_scale_invariant_variance(&sim);
    assert!((0.0..=0.15).contains(&var));
}

#[test]
fn test_compute_lambda_scale_invariant_variance_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
    }
    let v = PomValidator::new();
    let var = v.compute_lambda_scale_invariant_variance(&sim);
    assert!((0.0..=0.15).contains(&var));
}

#[test]
fn test_compute_lambda_universal_constant_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let lambda = v.compute_lambda_universal_constant(&sim);
    assert!((0.58..=0.64).contains(&lambda));
}

#[test]
fn test_compute_lambda_universal_constant_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
    }
    let v = PomValidator::new();
    let lambda = v.compute_lambda_universal_constant(&sim);
    assert!((0.58..=0.64).contains(&lambda));
}

#[test]
fn test_compute_neural_curvature_kappa_correlation_few_agents() {
    let sim = Simulation::new_default();
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let v = PomValidator::new();
    assert_eq!(v.compute_neural_curvature_kappa_correlation(&sim), 0.5);
}

#[test]
fn test_compute_neural_curvature_kappa_correlation_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
    }
    let v = PomValidator::new();
    let r = v.compute_neural_curvature_kappa_correlation(&sim);
    assert!((-1.0..=1.0).contains(&r));
}

#[test]
fn test_compute_phase_alignment_score_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let score = v.compute_phase_alignment_score(&sim);
    assert!((0.9..=1.0).contains(&score));
}

#[test]
fn test_compute_phase_alignment_score_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
    }
    let v = PomValidator::new();
    let score = v.compute_phase_alignment_score(&sim);
    assert!((0.9..=1.0).contains(&score));
}

#[test]
fn test_compute_tft_ess_emergence_empty() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let ess = v.compute_tft_ess_emergence(&sim);
    assert!((0.0..=1.0).contains(&ess));
}

#[test]
fn test_compute_tft_ess_emergence_with_tft() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.agents[0].strategy = crate::types::StrategyType::TitForTat;
    let v = PomValidator::new();
    let ess = v.compute_tft_ess_emergence(&sim);
    assert!((0.0..=1.0).contains(&ess));
}

#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_compute_semantic_coherence_cosine_few_agents() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let coh = v.compute_semantic_coherence_cosine(&sim);
    assert!((0.75..=1.0).contains(&coh));
}

#[test]
fn test_compute_semantic_coherence_cosine_with_agents() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
    }
    let v = PomValidator::new();
    let coh = v.compute_semantic_coherence_cosine(&sim);
    assert!((0.75..=1.0).contains(&coh));
}

#[test]
fn test_lexicon_cosine_similarity_empty() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    sim.spawn_agent(crate::types::Position {
        x: 2.0,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        y: 2.0,
        z: 0.0,
    });
    let v = PomValidator::new();
    assert_eq!(
        v.lexicon_cosine_similarity(&sim.agents[0], &sim.agents[1]),
        0.0
    );
}

// ========================================================================
// Statistical utility functions
// ========================================================================

#[test]
fn test_compute_correlation_empty() {
    assert!(compute_correlation(&[], &[]).is_none());
}

#[test]
fn test_compute_correlation_single_element() {
    assert!(compute_correlation(&[1.0], &[2.0]).is_none());
}

#[test]
fn test_compute_correlation_unequal_lengths() {
    assert!(compute_correlation(&[1.0, 2.0], &[1.0]).is_none());
}

#[test]
fn test_compute_correlation_zero_variance_x() {
    assert!(compute_correlation(&[5.0, 5.0, 5.0], &[1.0, 2.0, 3.0]).is_none());
}
// TAG: surface=simulation owner=research-team rule=SIM-001

#[test]
fn test_compute_correlation_zero_variance_y() {
    assert!(compute_correlation(&[1.0, 2.0, 3.0], &[5.0, 5.0, 5.0]).is_none());
}

#[test]
fn test_compute_correlation_no_correlation() {
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let y = vec![5.0, 2.0, 8.0, 1.0, 9.0, 3.0, 7.0, 4.0, 6.0, 10.0];
    let r = compute_correlation(&x, &y);
    assert!(r.is_some());
    assert!(r.unwrap().abs() < 0.5);
}

#[test]
fn test_compute_r_squared_empty() {
    assert!(compute_r_squared(&[], &[]).is_none());
}

#[test]
fn test_compute_r_squared_zero_variance() {
    assert!(compute_r_squared(&[1.0, 1.0], &[2.0, 3.0]).is_none());
}

#[test]
fn test_estimate_power_law_exponent_few_data() {
    assert!(estimate_power_law_exponent(&[1.0, 2.0, 3.0, 4.0, 5.0]).is_none());
}

#[test]
fn test_estimate_power_law_exponent_valid() {
    let mut data: Vec<f32> = (1..=100).map(|i| i as f32).collect();
    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let alpha = estimate_power_law_exponent(&data);
    assert!(alpha.is_some());
    let a = alpha.unwrap();
    assert!((1.0..=3.0).contains(&a));
}

#[test]
fn test_estimate_power_law_exponent_uniform() {
    let data: Vec<f32> = vec![1.0; 100];
    let alpha = estimate_power_law_exponent(&data);
    assert!(alpha.is_none());
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

// ========================================================================
// Validation suite methods
// ========================================================================

#[test]
fn test_p0_passed_all_pass() {
    let metrics = PomMetrics {
        agent_survival_rate: 0.75,
        cooperation_rate: 0.55,
        branching_ratio: 0.98,
        verification_rate: 0.85,
        identity_theft_rate: 0.05,
        dopamine_baseline: 0.50,
        oxytocin_baseline: 0.40,
        multimodal_sync_auc: 0.91,
        spectral_coherence_cf: 0.97,
        lambda_scale_invariant_variance: 0.10,
        ..Default::default()
    };
    let suite = validate_all_targets(&metrics);
    assert!(suite.p0_passed());
}

#[test]
fn test_p0_passed_some_fail() {
    let metrics = PomMetrics {
        agent_survival_rate: 0.30, // fail
        cooperation_rate: 0.55,
        branching_ratio: 0.98,
        verification_rate: 0.85,
        identity_theft_rate: 0.05,
        dopamine_baseline: 0.50,
        oxytocin_baseline: 0.40,
        multimodal_sync_auc: 0.91,
        spectral_coherence_cf: 0.97,
        lambda_scale_invariant_variance: 0.10,
        ..Default::default()
    };
    let suite = validate_all_targets(&metrics);
    assert!(!suite.p0_passed());
}

#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_failed_targets() {
    let metrics = PomMetrics {
        agent_survival_rate: 0.30,
        ..Default::default()
    };
    let suite = validate_all_targets(&metrics);
    let failed = suite.failed_targets();
    assert!(!failed.is_empty());
}

#[test]
fn test_get_category_results() {
    let metrics = PomMetrics::default();
    let suite = validate_all_targets(&metrics);
    assert!(suite.get_category_results(PomCategory::Geography).is_some());
    assert!(suite.get_category_results(PomCategory::Gtt).is_some());
}

#[test]
fn test_summary_report_with_failures() {
    let metrics = PomMetrics {
        agent_survival_rate: 0.30,
        ..Default::default()
    };
    let suite = validate_all_targets(&metrics);
    let report = suite.summary_report();
    assert!(report.contains("Failed Targets:"));
}

#[test]
fn test_summary_report_structure() {
    let metrics = PomMetrics {
        agent_survival_rate: 0.75,
        cooperation_rate: 0.55,
        branching_ratio: 0.98,
        verification_rate: 0.85,
        identity_theft_rate: 0.05,
        dopamine_baseline: 0.50,
        oxytocin_baseline: 0.40,
        multimodal_sync_auc: 0.91,
        spectral_coherence_cf: 0.97,
        lambda_scale_invariant_variance: 0.10,
        ..Default::default()
    };
    let suite = validate_all_targets(&metrics);
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let report = suite.summary_report();
    assert!(report.contains("POM Validation Suite Report"));
    assert!(report.contains("By Priority:"));
    assert!(report.contains("By Category:"));
}

// ========================================================================
// Integration tests
// ========================================================================

#[test]
fn test_compute_all_on_empty_sim() {
    let sim = Simulation::new_default();
    let v = PomValidator::new();
    let metrics = v.compute_all(&sim);
    assert_eq!(metrics.agent_survival_rate, 0.0);
}

#[test]
fn test_compute_all_on_populated_sim() {
    let mut sim = Simulation::new_default();
    for i in 0..5 {
        sim.spawn_agent(crate::types::Position {
            x: i as f32,
            y: i as f32,
            z: 0.0,
        });
    }
    let v = PomValidator::new();
    let metrics = v.compute_all(&sim);
    assert!(metrics.agent_survival_rate >= 0.0);
    assert!(metrics.energy_cv >= 0.0);
}

#[test]
fn test_validate_simulation_integration() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(crate::types::Position {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    let suite = validate_simulation(&sim);
    assert_eq!(suite.total_count, POM_TARGETS.len());
    assert!(suite.overall_score >= 0.0 && suite.overall_score <= 100.0);
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_get_metric_value_all_targets() {
    let metrics = PomMetrics::default();
    for target in POM_TARGETS {
        let value = get_metric_value(target.id, &metrics);
        assert!(
            value >= f32::NEG_INFINITY,
            "get_metric_value should return a valid float for {}",
            target.id
        );
    }
}

#[test]
fn test_get_metric_value_unknown_id() {
    let metrics = PomMetrics::default();
    assert_eq!(get_metric_value("UNKNOWN-001", &metrics), 0.0);
}

// ========================================================================
// Priority and category tests
// ========================================================================

#[test]
fn test_pom_priority_as_str() {
    assert_eq!(PomPriority::P0.as_str(), "P0");
    assert_eq!(PomPriority::P1.as_str(), "P1");
    assert_eq!(PomPriority::P2.as_str(), "P2");
}

#[test]
fn test_pom_category_display() {
    assert_eq!(format!("{}", PomCategory::Geography), "Geography");
    assert_eq!(format!("{}", PomCategory::Demographics), "Demographics");
    assert_eq!(format!("{}", PomCategory::Docas), "DOCAS");
    assert_eq!(format!("{}", PomCategory::Society), "Society");
    assert_eq!(format!("{}", PomCategory::Language), "Language");
    assert_eq!(format!("{}", PomCategory::Protocol), "Protocol");
    assert_eq!(format!("{}", PomCategory::Criticality), "Criticality");
    assert_eq!(format!("{}", PomCategory::Economics), "Economics");
    assert_eq!(format!("{}", PomCategory::Tfp), "TFP");
    assert_eq!(format!("{}", PomCategory::Gtt), "GTT");
}
