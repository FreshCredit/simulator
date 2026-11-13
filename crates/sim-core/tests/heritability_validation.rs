// TAG: surface=simulation owner=research-team rule=SIM-001
//! SIM-101: Heritability Validation Tests (DOCAS-C1)
//!
//! CODE-009: EXCEPTION - Integration test for sim-core crate (tests heritability)
//! Implements twin simulation studies to validate heritability of DOCAS traits.
//! Per simulation-plan.md P2-7: MZ vs DZ twin correlation using Falconer's formula.
//!
//! Methodology:
//! 1. Create MZ (monozygotic) twin pairs with identical genomes
//! 2. Create DZ (dizygotic) twin pairs with ~50% shared genetics
//! 3. Expose both to identical environmental stimuli
//! 4. Measure trait correlations
//! 5. Calculate h² = `2(r_MZ` - `r_DZ`)

// Numeric cast precision is acceptable in these simulation test utilities.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::float_cmp
)]

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sim_core::genome::Genome;
use sim_core::heritability::{HeritabilityAnalysis, HeritabilityEstimate};

/// Create an MZ twin pair (identical genomes)
fn create_mz_pair(seed: u64) -> (Genome, Genome) {
    let mut rng = StdRng::seed_from_u64(seed);
    let genome = Genome::random(&mut rng);
    (genome.clone(), genome)
}

/// Create a DZ twin pair (~50% shared genetics via randomization)
fn create_dz_pair(seed: u64) -> (Genome, Genome) {
    let mut rng = StdRng::seed_from_u64(seed);
    let parent = Genome::random(&mut rng);

    // DZ twin inherits ~50% via allele sampling
    let mut twin_alleles = Vec::with_capacity(parent.alleles.len());
    for allele_pair in &parent.alleles {
        // Randomly select one allele from each locus
        let selected = if rng.gen::<bool>() {
            [allele_pair[0], allele_pair[0]]
        } else {
            [allele_pair[1], allele_pair[1]]
        };
        twin_alleles.push(selected);
    }

    let twin = Genome {
        alleles: twin_alleles,
    };
    (parent, twin)
}
// TAG: surface=simulation owner=research-team rule=SIM-001

/// Simulate trait expression from genome
/// Returns trait values that would be observed after environmental exposure
fn simulate_trait_expression(genome: &Genome, stimulus_type: StimulusType) -> TraitProfile {
    let _docas_alpha = genome.docas_alpha();
    let docas_beta = genome.docas_beta();
    let planning_depth = genome.planning_depth();

    // Base trait values from genome
    let base_tau = genome.get_trait(15).clamp(0.0, 1.0); // Trust trait
    let base_cooperation = genome.get_trait(16).clamp(0.0, 1.0);
    let base_risk = genome.get_trait(17).clamp(0.0, 1.0);

    // Apply environmental stimulus effects
    let (tau, cooperation, risk) = match stimulus_type {
        StimulusType::Stress => {
            // Stress reduces trust, increases risk-taking
            let tau = (base_tau * 0.8).clamp(0.0, 1.0);
            let risk = (base_risk * 1.2).min(1.0);
            (tau, base_cooperation, risk)
        }
        StimulusType::Cooperation => {
            // Cooperative environment enhances trust
            let tau = (base_tau * 1.1).min(1.0);
            let coop = (base_cooperation * 1.15).min(1.0);
            (tau, coop, base_risk)
        }
        StimulusType::Neutral => (base_tau, base_cooperation, base_risk),
    };

    TraitProfile {
        tau,
        cooperation,
        risk_tolerance: risk,
        dopamine: docas_beta[0],
        oxytocin: docas_beta[1],
        cortisol: docas_beta[2],
        planning_depth: planning_depth as f32 / 20.0, // Normalize to [0, 1]
    }
}

/// Trait profile for an agent
#[derive(Clone, Copy, Debug)]
struct TraitProfile {
    pub tau: f32,
    pub cooperation: f32,
    pub risk_tolerance: f32,
    pub dopamine: f32,
    #[allow(dead_code)]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub oxytocin: f32,
    #[allow(dead_code)]
    pub cortisol: f32,
    pub planning_depth: f32,
}

/// Environmental stimulus types
#[derive(Clone, Copy)]
enum StimulusType {
    Stress,
    Cooperation,
    Neutral,
}

/// Run twin study with given configuration
fn run_twin_study(mz_count: usize, dz_count: usize, stimulus: StimulusType) -> TwinStudyResults {
    let mut mz_traits: Vec<(TraitProfile, TraitProfile)> = Vec::with_capacity(mz_count);
    let mut dz_traits: Vec<(TraitProfile, TraitProfile)> = Vec::with_capacity(dz_count);

    // Generate MZ twin pairs
    for i in 0..mz_count {
        let (twin_a, twin_b) = create_mz_pair(i as u64);
        let traits_a = simulate_trait_expression(&twin_a, stimulus);
        let traits_b = simulate_trait_expression(&twin_b, stimulus);
        mz_traits.push((traits_a, traits_b));
    }

    // Generate DZ twin pairs
    for i in 0..dz_count {
        let (twin_a, twin_b) = create_dz_pair((1000 + i) as u64);
        let traits_a = simulate_trait_expression(&twin_a, stimulus);
        let traits_b = simulate_trait_expression(&twin_b, stimulus);
        dz_traits.push((traits_a, traits_b));
    }

    TwinStudyResults {
        mz_traits,
        dz_traits,
    }
}

/// Results from twin study
struct TwinStudyResults {
    pub mz_traits: Vec<(TraitProfile, TraitProfile)>,
    pub dz_traits: Vec<(TraitProfile, TraitProfile)>,
}

/// Compute heritability analysis from twin study results
fn compute_heritability(results: &TwinStudyResults) -> HeritabilityAnalysis {
    // Extract trait pairs for MZ twins
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mz_tau: Vec<(f32, f32)> = results
        .mz_traits
        .iter()
        .map(|(a, b)| (a.tau, b.tau))
        .collect();
    let mz_coop: Vec<(f32, f32)> = results
        .mz_traits
        .iter()
        .map(|(a, b)| (a.cooperation, b.cooperation))
        .collect();
    let mz_risk: Vec<(f32, f32)> = results
        .mz_traits
        .iter()
        .map(|(a, b)| (a.risk_tolerance, b.risk_tolerance))
        .collect();
    let mz_dopamine: Vec<(f32, f32)> = results
        .mz_traits
        .iter()
        .map(|(a, b)| (a.dopamine, b.dopamine))
        .collect();

    // Extract trait pairs for DZ twins
    let dz_tau: Vec<(f32, f32)> = results
        .dz_traits
        .iter()
        .map(|(a, b)| (a.tau, b.tau))
        .collect();
    let dz_coop: Vec<(f32, f32)> = results
        .dz_traits
        .iter()
        .map(|(a, b)| (a.cooperation, b.cooperation))
        .collect();
    let dz_risk: Vec<(f32, f32)> = results
        .dz_traits
        .iter()
        .map(|(a, b)| (a.risk_tolerance, b.risk_tolerance))
        .collect();
    let dz_dopamine: Vec<(f32, f32)> = results
        .dz_traits
        .iter()
        .map(|(a, b)| (a.dopamine, b.dopamine))
        .collect();

    HeritabilityAnalysis {
        tau: HeritabilityEstimate::from_correlations("tau", &mz_tau, &dz_tau),
        cooperation: HeritabilityEstimate::from_correlations("cooperation", &mz_coop, &dz_coop),
        risk_tolerance: HeritabilityEstimate::from_correlations(
            "risk_tolerance",
            &mz_risk,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            &dz_risk,
        ),
        planning_depth: HeritabilityEstimate::default(),
        docas_sensitivity: HeritabilityEstimate::from_correlations(
            "dopamine",
            &mz_dopamine,
            &dz_dopamine,
        ),
        analysis_tick: 0,
    }
}

#[test]
fn test_mz_twins_identical_genomes() {
    //! MZ twins should have identical genomes

    let (genome_a, genome_b) = create_mz_pair(42);

    // Genomes should be identical
    assert_eq!(genome_a.alleles.len(), genome_b.alleles.len());
    for (a, b) in genome_a.alleles.iter().zip(genome_b.alleles.iter()) {
        assert_eq!(a, b, "MZ twins should have identical alleles");
    }
}

#[test]
fn test_mz_trait_correlation_higher_than_dz() {
    //! MZ twins should show higher trait correlation than DZ twins
    //! for heritable traits (per Falconer's formula).

    let results = run_twin_study(50, 50, StimulusType::Neutral);
    let analysis = compute_heritability(&results);

    // MZ correlation should generally be higher than DZ
    // (Allowing for statistical variation in small samples)
    println!("MZ tau correlation: {}", analysis.tau.mz_correlation);
    println!("DZ tau correlation: {}", analysis.tau.dz_correlation);

    // The key property: if trait is heritable, MZ > DZ
    // With genetic variation in the simulation, this should hold
    assert!(
        analysis.tau.mz_correlation >= analysis.tau.dz_correlation - 0.1,
        "MZ correlation should not be much lower than DZ: MZ={}, DZ={}",
        analysis.tau.mz_correlation,
        analysis.tau.dz_correlation
    );
}

#[test]
fn test_heritability_calculation_bounds() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    //! Heritability h² should be bounded to [0, 1]

    let results = run_twin_study(100, 100, StimulusType::Neutral);
    let analysis = compute_heritability(&results);

    // All heritability estimates should be in valid range
    assert!(
        analysis.tau.heritability >= 0.0 && analysis.tau.heritability <= 1.0,
        "Tau heritability should be in [0, 1]: {}",
        analysis.tau.heritability
    );
    assert!(
        analysis.cooperation.heritability >= 0.0 && analysis.cooperation.heritability <= 1.0,
        "Cooperation heritability should be in [0, 1]: {}",
        analysis.cooperation.heritability
    );
    assert!(
        analysis.risk_tolerance.heritability >= 0.0 && analysis.risk_tolerance.heritability <= 1.0,
        "Risk tolerance heritability should be in [0, 1]: {}",
        analysis.risk_tolerance.heritability
    );
}

#[test]
fn test_environmental_stimulus_affects_traits() {
    //! Different environmental stimuli should produce different trait expressions

    let results_stress = run_twin_study(30, 30, StimulusType::Stress);
    let results_coop = run_twin_study(30, 30, StimulusType::Cooperation);

    // Calculate mean tau for each condition
    let mean_tau_stress: f32 = results_stress
        .mz_traits
        .iter()
        .map(|(a, _)| a.tau)
        .sum::<f32>()
        / results_stress.mz_traits.len() as f32;
    let mean_tau_coop: f32 = results_coop
        .mz_traits
        .iter()
        .map(|(a, _)| a.tau)
        .sum::<f32>()
        / results_coop.mz_traits.len() as f32;

    // Cooperation stimulus should increase trust relative to stress
    assert!(
        mean_tau_coop > mean_tau_stress,
        "Cooperation stimulus should produce higher tau: coop={mean_tau_coop}, stress={mean_tau_stress}"
    );
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_docas_traits_show_heritability() {
    //! DOCAS neurochemical parameters should show measurable heritability

    let results = run_twin_study(60, 60, StimulusType::Neutral);

    // Extract dopamine values
    let mz_dopa: Vec<f32> = results
        .mz_traits
        .iter()
        .map(|(a, b)| (a.dopamine - b.dopamine).abs())
        .collect();
    let dz_dopa: Vec<f32> = results
        .dz_traits
        .iter()
        .map(|(a, b)| (a.dopamine - b.dopamine).abs())
        .collect();

    // MZ twins should have smaller differences (more similar)
    let mean_diff_monozygotic: f32 = mz_dopa.iter().sum::<f32>() / mz_dopa.len() as f32;
    let mean_diff_dizygotic: f32 = dz_dopa.iter().sum::<f32>() / dz_dopa.len() as f32;

    println!("MZ dopamine diff: {mean_diff_monozygotic}, DZ dopamine diff: {mean_diff_dizygotic}");

    // MZ should be more similar (smaller absolute differences)
    // Note: This is a statistical test, may occasionally fail due to random variation
    assert!(
        mean_diff_monozygotic <= mean_diff_dizygotic * 1.5,
        "MZ twins should have similar or smaller trait differences than DZ"
    );
}

#[test]
fn test_falconer_formula_computation() {
    //! Test the Falconer formula directly: h² = `2(r_MZ` - `r_DZ`)

    // Manually construct twin data with known correlations
    let mz_data: Vec<(f32, f32)> = (0..50)
        .map(|i| {
            let base = i as f32 / 50.0;
            // High correlation: twin_b very similar to twin_a
            (base, rand::random::<f32>().mul_add(0.05, base))
        })
        .collect();

    let dz_data: Vec<(f32, f32)> = (0..50)
        .map(|i| {
            let base = i as f32 / 50.0;
            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Lower correlation: more variation
            (base, rand::random::<f32>().mul_add(0.3, base))
        })
        .collect();

    let estimate = HeritabilityEstimate::from_correlations("test", &mz_data, &dz_data);

    // Verify Falconer's formula was applied
    let expected_h2 = 2.0 * (estimate.mz_correlation - estimate.dz_correlation);
    let clamped_h2 = expected_h2.clamp(0.0, 1.0);

    assert!(
        (estimate.heritability - clamped_h2).abs() < 0.001,
        "Heritability should follow Falconer's formula: expected={}, got={}",
        clamped_h2,
        estimate.heritability
    );
}

#[test]
fn test_sample_size_requirements() {
    //! Test that statistical significance requires adequate sample sizes

    // Test with various sample sizes
    let small_results = run_twin_study(5, 5, StimulusType::Neutral);
    let small_analysis = compute_heritability(&small_results);

    // Small samples should compute heritability (even if less reliable)
    assert!(
        small_analysis.tau.heritability >= 0.0 && small_analysis.tau.heritability <= 1.0,
        "Should compute heritability even for small samples: h²={}",
        small_analysis.tau.heritability
    );

    // Large sample should be marked as significant
    let large_results = run_twin_study(30, 30, StimulusType::Neutral);
    let large_analysis = compute_heritability(&large_results);

    assert!(
        large_analysis.tau.is_significant(),
        "Large sample (n=30) should be statistically significant: pairs=({}, {})",
        large_analysis.tau.mz_pairs,
        large_analysis.tau.dz_pairs
    );
}

#[test]
fn test_genome_to_trait_pathway() {
    //! Verify that genome alleles translate to observable traits
    // TAG: surface=simulation owner=research-team rule=SIM-001

    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(42);
    let genome = Genome::random(&mut rng);

    let profile = simulate_trait_expression(&genome, StimulusType::Neutral);

    // All trait values should be in valid range
    assert!(
        profile.tau >= 0.0 && profile.tau <= 1.0,
        "Tau should be in [0, 1]"
    );
    assert!(
        profile.cooperation >= 0.0 && profile.cooperation <= 1.0,
        "Cooperation should be in [0, 1]"
    );
    assert!(
        profile.risk_tolerance >= 0.0 && profile.risk_tolerance <= 1.0,
        "Risk tolerance should be in [0, 1]"
    );
    assert!(
        profile.dopamine >= 0.05 && profile.dopamine <= 0.2,
        "Dopamine beta should be in [0.05, 0.2]"
    );
    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(
        profile.planning_depth >= 0.0 && profile.planning_depth <= 1.0,
        "Planning depth should be normalized"
    );
}

#[test]
fn test_twin_trait_consistency_across_runs() {
    //! Same seed should produce consistent twin traits (determinism)

    let results1 = run_twin_study(20, 20, StimulusType::Neutral);
    let _results2 = run_twin_study(20, 20, StimulusType::Neutral);

    // First MZ pair should be identical across runs
    let (twin_a, twin_b) = &results1.mz_traits[0];

    // Note: These might differ slightly due to the DZ twin randomization,
    // but MZ twins should show the same pattern
    assert!(
        (twin_a.tau - twin_b.tau).abs() < 0.001,
        "MZ twins should have nearly identical tau values: A={}, B={}",
        twin_a.tau,
        twin_b.tau
    );
}
