//! Experiment Catalog Module
//!
//! Contains the static catalog of all 26 research experiments.
//! Extracted from experiments.rs to reduce file complexity.

use crate::api::experiments::{AcceptanceCriterion, ExperimentConfig, ExperimentInfo};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Complete catalog of all 26 research experiments
#[must_use]
pub fn experiment_catalog() -> Vec<ExperimentInfo> {
    let mut catalog = gtt_experiments();
    catalog.extend(docas_experiments());
    catalog.extend(pde4b_experiments());
    catalog.extend(emotion_experiments());
    catalog.extend(linguistics_experiments());
    catalog.extend(philosophy_experiments());
    catalog.extend(culture_experiments());
    catalog.extend(economics_experiments());
    catalog.extend(game_theory_experiments());
    catalog.extend(tfp_experiments());
    catalog.extend(meaning_experiments());
    catalog.extend(music_experiments());
    catalog.extend(v42_experiments());
    catalog
}

/// GTT (Generalized Trust Theory) experiments.
fn gtt_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === GTT (Generalized Trust Theory) ===
        ExperimentInfo {
            id: "lambda-universality".to_string(),
            name: "Dimensionless Trust Scaling (Pi_1 = lambda * dt)".to_string(),
            paper: "GTT".to_string(),
            paper_full: "The General Theory of Trust (v4.5.0)".to_string(),
            gap_ids: vec!["GTT-P0-1".to_string()],
            description: "Validate dimensionless Buckingham Pi_1 = lambda * dt dynamical scaling and Kalman parameter recovery (Sim W1, MAE <= 0.024). Note: scalar lambda universality in [0.53, 0.69] was retired in v4.5.0 because lambda depends on event cadence.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 365,
                replications: 10_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "lambda_mae".to_string(),
                    description: "Kalman/RTS-EM parameter recovery MAE".to_string(),
                    target_min: Some(0.0),
                    target_max: Some(0.024),
                    unit: "dimensionless".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "stability-law".to_string(),
            name: "Stability Law".to_string(),
            paper: "GTT".to_string(),
            paper_full: "Generalized Trust Theory".to_string(),
            gap_ids: vec!["GTT-P0-6".to_string()],
            description: "Validate stability law R² ≥ 0.60 for trust dynamics. Tests whether trust viscosity follows predicted functional form.".to_string(),
// TAG: surface=simulation owner=research-team rule=SIM-001
            default_config: ExperimentConfig {
                agents: 5_000,
                ticks: 365,
                replications: 10_000,
                seed: 123,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "stability_r2".to_string(),
                    description: "Stability law fit".to_string(),
                    target_min: Some(0.60),
                    target_max: None,
                    unit: "R²".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "cross-domain-correlation".to_string(),
            name: "Cross-Domain Correlation".to_string(),
            paper: "GTT".to_string(),
            paper_full: "Generalized Trust Theory".to_string(),
            gap_ids: vec!["GTT-P0-5".to_string()],
            description: "Validate cross-domain trust correlations r ∈ [0.40, 0.60]. Tests whether trust transfers across DOCAS, Emotion, Language, and Music domains.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 365,
                replications: 10_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "correlation".to_string(),
                    description: "Cross-domain correlation".to_string(),
                    target_min: Some(0.40),
                    target_max: Some(0.60),
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    unit: "Pearson r".to_string(),
                },
            ],
        },

    ]
}

/// DOCAS experiments.
fn docas_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === DOCAS ===
        ExperimentInfo {
            id: "twin-heritability".to_string(),
            name: "Twin Heritability".to_string(),
            paper: "DOCAS".to_string(),
            paper_full: "DOCAS Dynamics".to_string(),
            gap_ids: vec!["DOCAS-P0-2".to_string()],
            description: "Estimate heritability h² ∈ [0.42, 0.56] using Falconer's formula on MZ/DZ twin pairs. Tests genetic contribution to trust phenotypes.".to_string(),
            default_config: ExperimentConfig {
                agents: 2_000, // 1000 MZ pairs + 1000 DZ pairs
                ticks: 365,
                replications: 2_000,
                seed: 2024,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "heritability_h2".to_string(),
                    description: "Heritability estimate".to_string(),
                    target_min: Some(0.42),
                    target_max: Some(0.56),
                    unit: "h²".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "criticality-sigma".to_string(),
            name: "Criticality Sigma".to_string(),
            paper: "DOCAS".to_string(),
            paper_full: "DOCAS Dynamics".to_string(),
            gap_ids: vec!["DOCAS-P0-3".to_string()],
            description: "Measure criticality exponent σ ∈ [0.90, 1.10] from avalanche dynamics. Tests whether trust cascades exhibit self-organized criticality.".to_string(),
            default_config: ExperimentConfig {
                agents: 5_000,
// TAG: surface=simulation owner=research-team rule=SIM-001
                ticks: 365,
                replications: 5_000,
                seed: 999,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "sigma".to_string(),
                    description: "Criticality exponent".to_string(),
                    target_min: Some(0.90),
                    target_max: Some(1.10),
                    unit: "σ".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "sim-o1".to_string(),
            name: "Sim O1: Dopamine Lesion".to_string(),
            paper: "DOCAS".to_string(),
            paper_full: "Optogenetic Trust Intervention".to_string(),
            gap_ids: vec!["DOCAS-v4-O1".to_string()],
            description: "Simulate ArchT 566nm yellow light on Dopamine (D) channel, blocking positive updates and testing trust suppression > 90% (99.97% observed).".to_string(),
            default_config: ExperimentConfig {
                agents: 1_000,
                ticks: 100,
                replications: 1_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "suppression".to_string(),
                    description: "Dopamine lesion suppression".to_string(),
                    target_min: Some(0.90),
                    target_max: None,
                    unit: "%".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "sim-o2".to_string(),
            name: "Sim O2: Cortisol Vulnerability".to_string(),
            paper: "DOCAS".to_string(),
            paper_full: "Optogenetic Stress Excitation".to_string(),
            gap_ids: vec!["DOCAS-v4-O2".to_string()],
            description: "Pulse ChR2 470nm blue light on Cortisol (C) channel, compressing cognitive bandwidth and testing predatory contract uptake surge (1.74x relative risk).".to_string(),
            default_config: ExperimentConfig {
                agents: 1_000,
                ticks: 100,
                replications: 1_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "relative_risk".to_string(),
                    description: "Cortisol stress relative risk".to_string(),
                    target_min: Some(1.40),
                    target_max: None,
                    unit: "RR".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "sim-d16".to_string(),
            name: "Sim D16: Dual-Branch Visual Cloaking".to_string(),
            paper: "DOCAS".to_string(),
            paper_full: "Perceptual Cloaking & Bounded Gating".to_string(),
            gap_ids: vec!["DOCAS-v4-D16".to_string()],
            description: "Evaluate 27 credit offers under Brain-IT Branch A structural layout vs Branch B semantic detail, verifying the 38.9x vulnerability decile odds ratio (D1 vs D10).".to_string(),
            default_config: ExperimentConfig {
                agents: 1_000,
                ticks: 365,
                replications: 1_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "odds_ratio".to_string(),
                    description: "Vulnerability gradient odds ratio D1/D10".to_string(),
                    target_min: Some(3.0),
                    target_max: None,
                    unit: "OR".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "sim-d17".to_string(),
            name: "Sim D17: Weather Gating & Weather-Shield".to_string(),
            paper: "DOCAS".to_string(),
            paper_full: "Atmospheric & Photic Gating".to_string(),
            gap_ids: vec!["DOCAS-v4-D17".to_string()],
            description: "Simulate atmospheric photic-thermal coupling and the 24-hour Weather-Shield decision cooling-off buffer mitigating predatory uptake during heatwaves and winter deficits.".to_string(),
            default_config: ExperimentConfig {
                agents: 1_000,
                ticks: 365,
                replications: 1_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "heatwave_rr".to_string(),
                    description: "Heatwave relative risk".to_string(),
                    target_min: Some(1.02),
                    target_max: None,
                    unit: "RR".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "sim-d18".to_string(),
            name: "Sim D18: Baylor Unconscious Narrative Gating".to_string(),
            paper: "DOCAS".to_string(),
            paper_full: "Baylor Narrative Gating".to_string(),
            gap_ids: vec!["DOCAS-v4-D18".to_string()],
            description: "Simulate Baylor study consciousness mode switching (M in {0,1}), Cortisol threshold suppression (C < 0.12), dopamine-decoupled syntactic schema preservation, and waking attractor drift.".to_string(),
            default_config: ExperimentConfig {
                agents: 1_000,
                ticks: 100,
                replications: 1,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "cortisol_friction_reduction".to_string(),
                    description: "Cortisol gate cognitive friction reduction >= 75%".to_string(),
                    target_min: Some(0.75),
                    target_max: None,
                    unit: "ratio".to_string(),
                },
                AcceptanceCriterion {
                    metric: "syntax_difference_ratio".to_string(),
                    description: "Syntactic perplexity difference ratio <= 5%".to_string(),
                    target_min: None,
                    target_max: Some(0.05),
                    unit: "ratio".to_string(),
                },
            ],
        },



    ]
}

/// PDE4B experiments.
fn pde4b_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === PDE4B ===
        ExperimentInfo {
            id: "pde4b-lambda-correlation".to_string(),
            name: "Omnigenic Striatal Coincidence (Sim GEN-2)".to_string(),
            paper: "TDF / DOCAS".to_string(),
            paper_full: "Omnigenic Striatal Coincidence Genetics".to_string(),
            gap_ids: vec!["PDE4B-P0-1".to_string()],
            description: "Evaluate omnigenic polygenic coincidence score (p-PGS across 100 core regulatory loci, R^2 = 0.0765, p < 1e-80) driving Master Law gain lambda. Single-candidate PDE4B variance was falsified in Sim GEN-1 (R^2 < 0.0005).".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 365,
                replications: 5_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "p_pgs_r2".to_string(),
                    description: "Omnigenic p-PGS variance explained".to_string(),
                    target_min: Some(0.05),
                    target_max: None,
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    unit: "R^2".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "pde4b-pharmacology".to_string(),
            name: "PDE4B Pharmacology".to_string(),
            paper: "PDE4B".to_string(),
            paper_full: "PDE4B Genetics".to_string(),
            gap_ids: vec!["PDE4B-P0-2".to_string()],
            description: "Test pharmacological intervention effect size d > 0.50. Measures impact of PDE4B inhibitors on trust behavior.".to_string(),
            default_config: ExperimentConfig {
                agents: 5_000,
                ticks: 365,
                replications: 2_000,
                seed: 123,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "cohens_d".to_string(),
                    description: "Intervention effect size".to_string(),
                    target_min: Some(0.50),
                    target_max: None,
                    unit: "Cohen's d".to_string(),
                },
            ],
        },

    ]
}

/// Emotion experiments.
fn emotion_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === Emotion ===
        ExperimentInfo {
            id: "emotion-mapping".to_string(),
            name: "Emotion Mapping".to_string(),
            paper: "Emotion".to_string(),
            paper_full: "Emotion-DOCAS Mapping".to_string(),
            gap_ids: vec!["EMOT-P0-1".to_string()],
            description: "Validate DOCAS → VAD emotion mapping R² > 0.75. Tests reverse projection from neuromodulator state to dimensional emotion.".to_string(),
            default_config: ExperimentConfig {
                agents: 5_000,
// TAG: surface=simulation owner=research-team rule=SIM-001
                ticks: 365,
                replications: 5_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "r_squared".to_string(),
                    description: "DOCAS-Emotion fit".to_string(),
                    target_min: Some(0.75),
                    target_max: None,
                    unit: "R²".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "emotion-pharmacology".to_string(),
            name: "Emotion Pharmacology".to_string(),
            paper: "Emotion".to_string(),
            paper_full: "Emotion-DOCAS Mapping".to_string(),
            gap_ids: vec!["EMOT-P0-3".to_string()],
            description: "Test emotion intervention R² > 0.40. Validates pharmacological modulation of emotion-trust relationships.".to_string(),
            default_config: ExperimentConfig {
                agents: 5_000,
                ticks: 365,
                replications: 2_000,
                seed: 123,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "r_squared".to_string(),
                    description: "Intervention model fit".to_string(),
                    target_min: Some(0.40),
                    target_max: None,
                    unit: "R²".to_string(),
                },
            ],
        },

// TAG: surface=simulation owner=research-team rule=SIM-001
    ]
}

/// Linguistics experiments.
fn linguistics_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === Linguistics ===
        ExperimentInfo {
            id: "semantic-network".to_string(),
            name: "Semantic Network".to_string(),
            paper: "Linguistics".to_string(),
            paper_full: "Semantic Trust Networks".to_string(),
            gap_ids: vec!["LING-P0-1".to_string()],
            description: "Measure betweenness centrality BC > 2.0 for trust-weighted semantic networks. Tests network structure of trust-laden terms.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 730,
                replications: 5_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "betweenness_centrality".to_string(),
                    description: "Trust term centrality".to_string(),
                    target_min: Some(2.0),
                    target_max: None,
                    unit: "BC".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "semantic-drift".to_string(),
            name: "Semantic Drift".to_string(),
            paper: "Linguistics".to_string(),
            paper_full: "Semantic Trust Networks".to_string(),
            gap_ids: vec!["LING-P0-2".to_string()],
            description: "Test drift ∝ 1/τ correlation r < -0.30. Validates that semantic change rate correlates with trust decline.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 730,
                replications: 5_000,
// TAG: surface=simulation owner=research-team rule=SIM-001
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "correlation".to_string(),
                    description: "Drift-trust correlation".to_string(),
                    target_min: None,
                    target_max: Some(-0.30),
                    unit: "Pearson r".to_string(),
                },
            ],
        },

    ]
}

/// Philosophy experiments.
fn philosophy_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === Philosophy ===
        ExperimentInfo {
            id: "philosophical-oscillation".to_string(),
            name: "Philosophical Oscillation".to_string(),
            paper: "Philosophy".to_string(),
            paper_full: "Philosophy of Trust".to_string(),
            gap_ids: vec!["PHIL-P0-2".to_string()],
            description: "Measure oscillation period T ∈ [200, 300] years. Tests cyclical theory of philosophical trust frameworks.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 3_650, // 10 years of daily ticks
                replications: 1_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "oscillation_period".to_string(),
                    description: "Philosophy cycle period".to_string(),
                    target_min: Some(200.0),
                    target_max: Some(300.0),
                    unit: "years".to_string(),
                },
            ],
        },

// TAG: surface=simulation owner=research-team rule=SIM-001
    ]
}

/// Culture experiments.
fn culture_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === Culture ===
        ExperimentInfo {
            id: "cultural-quadrants".to_string(),
            name: "Cultural Quadrants".to_string(),
            paper: "Culture".to_string(),
            paper_full: "Cultural Trust Quadrants".to_string(),
            gap_ids: vec!["CULT-P0-1".to_string()],
            description: "Validate quadrant model R² > 0.80. Tests two-dimensional cultural classification (bond density × philosophical axis).".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 1_825, // 5 years
                replications: 2_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "r_squared".to_string(),
                    description: "Quadrant model fit".to_string(),
                    target_min: Some(0.80),
                    target_max: None,
                    unit: "R²".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "hofstede-prediction".to_string(),
            name: "Hofstede Prediction".to_string(),
            paper: "Culture".to_string(),
            paper_full: "Cultural Trust Quadrants".to_string(),
            gap_ids: vec!["CULT-P0-3".to_string()],
            description: "Predict Hofstede dimensions R² > 0.60. Tests whether simulation parameters predict real-world cultural indices.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 1_825,
                replications: 2_000,
// TAG: surface=simulation owner=research-team rule=SIM-001
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "r_squared".to_string(),
                    description: "Hofstede prediction".to_string(),
                    target_min: Some(0.60),
                    target_max: None,
                    unit: "R²".to_string(),
                },
            ],
        },

    ]
}

/// Economics experiments.
fn economics_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === Economics ===
        ExperimentInfo {
            id: "gdp-prediction".to_string(),
            name: "GDP Prediction".to_string(),
            paper: "Economics".to_string(),
            paper_full: "Trust Economics".to_string(),
            gap_ids: vec!["ECON-P0-1".to_string()],
            description: "Predict GDP from trust R² > 0.85. Validates GDP = f(bond_density × τ) model against World Bank data.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 1_825,
                replications: 5_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "r_squared".to_string(),
                    description: "GDP prediction".to_string(),
                    target_min: Some(0.85),
                    target_max: None,
                    unit: "R²".to_string(),
                },
            ],
// TAG: surface=simulation owner=research-team rule=SIM-001
        },
        ExperimentInfo {
            id: "crisis-threshold".to_string(),
            name: "Crisis Threshold".to_string(),
            paper: "Economics".to_string(),
            paper_full: "Trust Economics".to_string(),
            gap_ids: vec!["ECON-P0-2".to_string()],
            description: "Detect crisis threshold τ ∈ [0.25, 0.35]. Identifies trust level below which economic crises emerge.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 1_825,
                replications: 5_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "crisis_threshold".to_string(),
                    description: "Crisis τ threshold".to_string(),
                    target_min: Some(0.25),
                    target_max: Some(0.35),
                    unit: "τ".to_string(),
                },
            ],
        },

    ]
}

/// Game Theory experiments.
fn game_theory_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === Game Theory ===
        ExperimentInfo {
            id: "trust-weighted-pd".to_string(),
            name: "Trust-Weighted Prisoner's Dilemma".to_string(),
            paper: "GameTheory".to_string(),
            paper_full: "Game Theory of Trust".to_string(),
            gap_ids: vec!["GAME-P0-1".to_string()],
            description: "Test cooperation ∝ τ correlation r > 0.50. Validates trust-weighted expected utility model in PD games.".to_string(),
            default_config: ExperimentConfig {
                agents: 1_000,
                ticks: 1_000,
                replications: 10_000,
// TAG: surface=simulation owner=research-team rule=SIM-001
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "correlation".to_string(),
                    description: "Trust-cooperation correlation".to_string(),
                    target_min: Some(0.50),
                    target_max: None,
                    unit: "Pearson r".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "trust-weighted-ultimatum".to_string(),
            name: "Trust-Weighted Ultimatum".to_string(),
            paper: "GameTheory".to_string(),
            paper_full: "Game Theory of Trust".to_string(),
            gap_ids: vec![], // Additional experiment
            description: "Test offer ∝ τ correlation r > 0.40. Extends trust-EU model to ultimatum games.".to_string(),
            default_config: ExperimentConfig {
                agents: 1_000,
                ticks: 1_000,
                replications: 10_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "correlation".to_string(),
                    description: "Trust-offer correlation".to_string(),
                    target_min: Some(0.40),
                    target_max: None,
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    unit: "Pearson r".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "evolutionary-dynamics".to_string(),
            name: "Evolutionary Dynamics".to_string(),
            paper: "GameTheory".to_string(),
            paper_full: "Game Theory of Trust".to_string(),
            gap_ids: vec!["GAME-P0-3".to_string()],
            description: "Test ESS convergence. Validates evolutionary stability of trust-based strategies.".to_string(),
            default_config: ExperimentConfig {
                agents: 5_000,
                ticks: 1_000,
                replications: 5_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "ess_convergence".to_string(),
                    description: "ESS convergence rate".to_string(),
                    target_min: Some(0.95),
                    target_max: None,
                    unit: "proportion".to_string(),
                },
            ],
        },

    ]
}

/// TFP (Trust Field Protocol) experiments.
fn tfp_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === TFP (Trust Field Protocol) ===
        ExperimentInfo {
            id: "psi-trust-auc".to_string(),
            name: "Ψ-Trust AUC".to_string(),
            paper: "TFP".to_string(),
            paper_full: "Trust Field Protocol".to_string(),
            gap_ids: vec!["TFP-P0-3".to_string()],
            description: "Measure Ψ_trust discrimination AUC ≥ 0.80. Tests whether composite trust field predicts betrayal.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 365,
                replications: 10_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
// TAG: surface=simulation owner=research-team rule=SIM-001
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "auc".to_string(),
                    description: "ROC AUC".to_string(),
                    target_min: Some(0.80),
                    target_max: None,
                    unit: "AUC".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "entropy-reduction".to_string(),
            name: "Entropy Reduction".to_string(),
            paper: "TFP".to_string(),
            paper_full: "Trust Field Protocol".to_string(),
            gap_ids: vec![], // Additional experiment
            description: "Measure entropy reduction ΔH < -0.10. Tests whether trust field reduces social uncertainty.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 365,
                replications: 10_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "entropy_delta".to_string(),
                    description: "Entropy change".to_string(),
                    target_min: None,
                    target_max: Some(-0.10),
                    unit: "bits".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "cost-model".to_string(),
            name: "Cost Model".to_string(),
// TAG: surface=simulation owner=research-team rule=SIM-001
            paper: "TFP".to_string(),
            paper_full: "Trust Field Protocol".to_string(),
            gap_ids: vec!["TFP-P0-4".to_string()],
            description: "Validate cost model MC ∈ [0.01, 0.05]. Tests marginal cost of trust field maintenance.".to_string(),
            default_config: ExperimentConfig {
                agents: 100_000,
                ticks: 365,
                replications: 1_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "marginal_cost".to_string(),
                    description: "Marginal cost".to_string(),
                    target_min: Some(0.01),
                    target_max: Some(0.05),
                    unit: "cost/agent/tick".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "sim-o3".to_string(),
            name: "Sim O3: Kuramoto Mutant Fragmentation".to_string(),
            paper: "TFP".to_string(),
            paper_full: "Macroscopic Coherence Resilience".to_string(),
            gap_ids: vec!["TFP-v4-O3".to_string()],
            description: "Optogenetically silence Oxytocin/Serotonin coupling in 10% of agents, testing global Kuramoto coherence order parameter R stability and resilience (< 40% collapse threshold, 11.39% observed).".to_string(),
            default_config: ExperimentConfig {
                agents: 500,
                ticks: 200,
                replications: 1_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "coherence_collapse".to_string(),
                    description: "Global Kuramoto coherence collapse fraction".to_string(),
                    target_min: None,
                    target_max: Some(0.40),
                    unit: "%".to_string(),
                },
            ],
        },


    ]
}

/// Meaning experiments.
fn meaning_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === Meaning ===
        ExperimentInfo {
            id: "percolation-threshold".to_string(),
            name: "Percolation Threshold".to_string(),
            paper: "Meaning".to_string(),
            paper_full: "Meaning and Trust".to_string(),
            gap_ids: vec!["MEAN-P0-3".to_string()],
            description: "Measure percolation threshold τ_c ∈ [0.29, 0.35]. Tests critical trust level for meaning transmission.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 365,
                replications: 5_000,
                seed: 7777,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
// TAG: surface=simulation owner=research-team rule=SIM-001
                AcceptanceCriterion {
                    metric: "percolation_threshold".to_string(),
                    description: "Critical τ for percolation".to_string(),
                    target_min: Some(0.29),
                    target_max: Some(0.35),
                    unit: "τ".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "civilizational-collapse".to_string(),
            name: "Civilizational Collapse".to_string(),
            paper: "Meaning".to_string(),
            paper_full: "Meaning and Trust".to_string(),
            gap_ids: vec!["MEAN-P0-2".to_string()],
            description: "Predict collapse accuracy > 0.70. Validates meaning-field model of societal collapse.".to_string(),
            default_config: ExperimentConfig {
                agents: 10_000,
                ticks: 365,
                replications: 5_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "accuracy".to_string(),
                    description: "Collapse prediction accuracy".to_string(),
                    target_min: Some(0.70),
                    target_max: None,
                    unit: "proportion".to_string(),
                },
            ],
        },

    ]
}

/// Music experiments.
fn music_experiments() -> Vec<ExperimentInfo> {
    vec![
        // === Music ===
        ExperimentInfo {
            id: "spectral-coherence".to_string(),
            name: "Spectral Coherence".to_string(),
// TAG: surface=simulation owner=research-team rule=SIM-001
            paper: "Music".to_string(),
            paper_full: "Music and Trust".to_string(),
            gap_ids: vec!["MUSIC-P0-2".to_string()],
            description: "Measure music-trust coherence > 0.30. Tests frequency-domain correlation between musical features and trust dynamics.".to_string(),
            default_config: ExperimentConfig {
                agents: 5_000,
                ticks: 365,
                replications: 5_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "coherence".to_string(),
                    description: "Spectral coherence".to_string(),
                    target_min: Some(0.30),
                    target_max: None,
                    unit: "magnitude".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "music-trust-correlation".to_string(),
            name: "Music-Trust Correlation".to_string(),
            paper: "Music".to_string(),
            paper_full: "Music and Trust".to_string(),
            gap_ids: vec!["MUSIC-P0-3".to_string()],
            description: "Measure music → trust lead indicator > 0.10. Tests whether musical features predict future trust changes.".to_string(),
            default_config: ExperimentConfig {
                agents: 5_000,
                ticks: 365,
                replications: 5_000,
                seed: 42,
                pde4b_variance: "medium".to_string(),
            },
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "lead_correlation".to_string(),
                    description: "Music-trust lead correlation".to_string(),
                    target_min: Some(0.10),
                    target_max: None,
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    unit: "Pearson r".to_string(),
                },
            ],
        },
    ]
}
/// v4.2 research experiments (TFP, Whitepaper, Lemons ODE).
fn v42_experiments() -> Vec<ExperimentInfo> {
    vec![
        ExperimentInfo {
            id: "sim-p8c".to_string(),
            name: "Sim P8c: Provider-Diversity Entropy Gate".to_string(),
            paper: "TFP".to_string(),
            paper_full: "The Fresh Protocol §3.7".to_string(),
            gap_ids: vec!["TFP-v4-P8c".to_string()],
            description: "Provider-diversity entropy gate scales standing accumulation by eta = 1 - exp(-H(w)). Monopoly colluders (P=1, H=0) are blocked (eta = 0.0).".to_string(),
            default_config: ExperimentConfig::default(),
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "monopoly_efficiency".to_string(),
                    description: "Monopoly colluder efficiency".to_string(),
                    target_min: Some(0.0),
                    target_max: Some(0.0),
                    unit: "eta".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "sim-p8d".to_string(),
            name: "Sim P8d: Economic Collusion Auction".to_string(),
            paper: "TFP".to_string(),
            paper_full: "The Fresh Protocol §3.7".to_string(),
            gap_ids: vec!["TFP-v4-P8d".to_string()],
            description: "Microeconomic collusion auction proving break-even credit limits L*(K) exceed entry micro-credit lines.".to_string(),
            default_config: ExperimentConfig::default(),
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "breakeven_limit".to_string(),
                    description: "Adversary break-even credit limit".to_string(),
                    target_min: Some(2000.0),
                    target_max: None,
                    unit: "USD".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "sim-w8".to_string(),
            name: "Sim W8: Temporal SHAP Adverse Action".to_string(),
            paper: "Whitepaper".to_string(),
            paper_full: "FreshCredit Whitepaper §5".to_string(),
            gap_ids: vec!["WP-v4-W8".to_string()],
            description: "Exact coalitional Shapley values (error < 1e-12) over 6 cash-flow features generating 12 C.F.R. § 1002.9 top-4 reasons and actionable recourse.".to_string(),
            default_config: ExperimentConfig::default(),
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "efficiency_error".to_string(),
                    description: "Shapley efficiency error".to_string(),
                    target_min: None,
                    target_max: Some(1e-12),
                    unit: "error".to_string(),
                },
            ],
        },
        ExperimentInfo {
            id: "sim-w9".to_string(),
            name: "Sim W9: Dynamic Lemons Market Solvency".to_string(),
            paper: "Whitepaper".to_string(),
            paper_full: "FreshCredit Whitepaper §5".to_string(),
            gap_ids: vec!["WP-v4-W9".to_string()],
            description: "Dynamic Lemons Market ODE proving lending pool reserve solvency (R(t) >= $500k) over 36 months under adverse selection.".to_string(),
            default_config: ExperimentConfig::default(),
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    metric: "min_reserve".to_string(),
                    description: "Minimum reserve pool balance".to_string(),
                    target_min: Some(500000.0),
                    target_max: None,
                    unit: "USD".to_string(),
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_length() {
        let catalog = experiment_catalog();
        assert_eq!(catalog.len(), 36);
    }

    #[test]
    fn test_catalog_unique_ids() {
        let catalog = experiment_catalog();
        let ids: Vec<_> = catalog.iter().map(|e| e.id.clone()).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique_ids.len(), ids.len());
    }

    #[test]
    fn test_catalog_contains_lambda_universality() {
        let catalog = experiment_catalog();
        assert!(catalog.iter().any(|e| e.id == "lambda-universality"));
    }

    #[test]
    fn test_catalog_no_empty_descriptions() {
        let catalog = experiment_catalog();
        for exp in &catalog {
            assert!(
                !exp.description.is_empty(),
                "{} has empty description",
                exp.id
            );
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_catalog_acceptance_criteria_ranges() {
        let catalog = experiment_catalog();
        for exp in &catalog {
            for criterion in &exp.acceptance_criteria {
                if let (Some(min), Some(max)) = (criterion.target_min, criterion.target_max) {
                    assert!(min <= max, "{}: min {} > max {}", exp.id, min, max);
                }
            }
        }
    }

    #[test]
    fn test_catalog_config_agents_positive() {
        let catalog = experiment_catalog();
        for exp in &catalog {
            assert!(
                exp.default_config.agents > 0,
                "{}: agents must be positive",
                exp.id
            );
            assert!(
                exp.default_config.ticks > 0,
                "{}: ticks must be positive",
                exp.id
            );
        }
    }
}
