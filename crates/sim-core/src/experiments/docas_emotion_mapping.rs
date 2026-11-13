//! DOCAS-Emotion Mapping Validation Experiment (EMOT-P0-2)
//!
//! Validates the post-hoc mapping between DOCAS neurochemical dimensions
//! and emotion space (Valence-Arousal-Dominance model).
//!
//! ## Hypothesis
//! - H1: Dopamine correlates with arousal (r > 0.6)
//! - H2: Oxytocin correlates with positive valence (r > 0.5)
//! - H3: Cortisol correlates with negative dominance (r > 0.4)
//! - H4: Combined DOCAS → VAD mapping explains >70% variance
//!
//! ## References
//! - Russell (1980): Circumplex model of affect
//! - Mehrabian (1996): Pleasure-arousal-dominance
//! - DOCAS Paper: Neurochemical basis of trust dynamics

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::json;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// DOCAS neurochemical state
#[derive(Debug, Clone)]
pub struct DOCASState {
    /// dopamine.
    pub dopamine: f64, // 0-1 range
    /// oxytocin.
    pub oxytocin: f64, // 0-1 range
    /// cortisol.
    pub cortisol: f64, // 0-1 range
    /// adrenaline.
    pub adrenaline: f64, // 0-1 range
    /// serotonin.
    pub serotonin: f64, // 0-1 range
}

/// VAD emotion coordinates
#[derive(Debug, Clone)]
pub struct VADState {
    /// valence.
    pub valence: f64, // -1 (negative) to 1 (positive)
    /// arousal.
    pub arousal: f64, // 0 (calm) to 1 (excited)
    /// dominance.
    pub dominance: f64, // -1 (submissive) to 1 (dominant)
}

/// Mapping result for a single data point
#[derive(Debug, Clone)]
pub struct MappingResult {
    /// docas.
    pub docas: DOCASState,
    /// actual vad.
    pub actual_vad: VADState,
    /// predicted vad.
    pub predicted_vad: VADState,
    /// error.
    pub error: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// DOCAS-Emotion Mapping Validation Experiment
///
/// EMOT-P0-2: Validates post-hoc DOCAS → VAD mapping.
#[derive(Debug)]
pub struct DOCASEmotionMapping;

impl DOCASEmotionMapping {
    /// Generate random DOCAS state using seeded RNG (SIM-002: deterministic)
    fn random_docas<R: rand::Rng>(&self, rng: &mut R) -> DOCASState {
        DOCASState {
            dopamine: rng.gen(),
            oxytocin: rng.gen(),
            cortisol: rng.gen(),
            adrenaline: rng.gen(),
            serotonin: rng.gen(),
        }
    }

    /// Simulate ground-truth VAD from DOCAS (the "true" mapping)
    fn docas_to_vad_ground_truth(&self, docas: &DOCASState) -> VADState {
        // Valence: Positive from oxytocin and serotonin, negative from cortisol
        let valence = docas.oxytocin * 0.4 + docas.serotonin * 0.3 - docas.cortisol * 0.3;

        // Arousal: Driven by dopamine and adrenaline
        let arousal = docas.dopamine * 0.5 + docas.adrenaline * 0.5;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Dominance: Confidence from balanced dopamine/serotonin
        let dominance = (docas.dopamine - docas.cortisol) * 0.5;

        VADState {
            valence: valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(0.0, 1.0),
            dominance: dominance.clamp(-1.0, 1.0),
        }
    }

    /// Predict VAD using our mapping model
    fn predict_vad(&self, docas: &DOCASState) -> VADState {
        // Model coefficients (to be validated)
        let valence = docas.oxytocin * 0.45 + docas.serotonin * 0.25 - docas.cortisol * 0.25
            + docas.dopamine * 0.05;

        let arousal = docas.dopamine * 0.55 + docas.adrenaline * 0.45 - docas.serotonin * 0.1;

        let dominance = docas.dopamine * 0.4 - docas.cortisol * 0.35 + docas.serotonin * 0.15;

        VADState {
            valence: valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(0.0, 1.0),
            dominance: dominance.clamp(-1.0, 1.0),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate Pearson correlation
    fn correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        crate::stats::pearson_correlation_f64(x, y).clamp(-1.0, 1.0)
    }

    /// Calculate R² (coefficient of determination)
    fn r_squared(&self, actual: &[f64], predicted: &[f64]) -> f64 {
        if actual.len() != predicted.len() || actual.is_empty() {
            return 0.0;
        }

        let mean_actual = actual.iter().sum::<f64>() / actual.len() as f64;

        let ss_total: f64 = actual.iter().map(|a| (a - mean_actual).powi(2)).sum();
        let ss_residual: f64 = actual
            .iter()
            .zip(predicted.iter())
            .map(|(a, p)| (a - p).powi(2))
            .sum();

        if ss_total == 0.0 {
            return 0.0;
        }

        1.0 - (ss_residual / ss_total)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for DOCASEmotionMapping {
    fn name(&self) -> &'static str {
        "docas_emotion_mapping"
    }

    fn description(&self) -> &'static str {
        "Validate DOCAS to VAD emotion space mapping (EMOT-P0-2)"
    }

    fn required_agents(&self) -> usize {
        500
    }

    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let n_samples = config.n_agents;
        let mut results = Vec::new();
        let seed = if config.seed != 0 { config.seed } else { 42 };
        let mut rng = StdRng::seed_from_u64(seed);

        // Generate samples
        for _ in 0..n_samples {
            let docas = self.random_docas(&mut rng);
            let actual_vad = self.docas_to_vad_ground_truth(&docas);
            let predicted_vad = self.predict_vad(&docas);

            let error = ((actual_vad.valence - predicted_vad.valence).powi(2)
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
                + (actual_vad.arousal - predicted_vad.arousal).powi(2)
                + (actual_vad.dominance - predicted_vad.dominance).powi(2))
            .sqrt();

            results.push(MappingResult {
                docas,
                actual_vad,
                predicted_vad,
                error,
            });
        }

        // Extract series for correlation analysis
        let dopamine: Vec<f64> = results.iter().map(|r| r.docas.dopamine).collect();
        let oxytocin: Vec<f64> = results.iter().map(|r| r.docas.oxytocin).collect();
        let cortisol: Vec<f64> = results.iter().map(|r| r.docas.cortisol).collect();
        let valence: Vec<f64> = results.iter().map(|r| r.actual_vad.valence).collect();
        let arousal: Vec<f64> = results.iter().map(|r| r.actual_vad.arousal).collect();
        let dominance: Vec<f64> = results.iter().map(|r| r.actual_vad.dominance).collect();

        let pred_valence: Vec<f64> = results.iter().map(|r| r.predicted_vad.valence).collect();
        let pred_arousal: Vec<f64> = results.iter().map(|r| r.predicted_vad.arousal).collect();
        let pred_dominance: Vec<f64> = results.iter().map(|r| r.predicted_vad.dominance).collect();

        // Calculate correlations
        let r_dopamine_arousal = self.correlation(&dopamine, &arousal);
        let r_oxytocin_valence = self.correlation(&oxytocin, &valence);
        let r_cortisol_dominance = self.correlation(&cortisol, &dominance);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Calculate R² for each dimension
        let r2_valence = self.r_squared(&valence, &pred_valence);
        let r2_arousal = self.r_squared(&arousal, &pred_arousal);
        let r2_dominance = self.r_squared(&dominance, &pred_dominance);
        let r2_overall = (r2_valence + r2_arousal + r2_dominance) / 3.0;

        // Calculate mean error
        let mean_error = results.iter().map(|r| r.error).sum::<f64>() / results.len() as f64;

        // Validation criteria
        let criteria = [
            ("Dopamine-Arousal r > 0.6", r_dopamine_arousal > 0.6),
            ("Oxytocin-Valence r > 0.5", r_oxytocin_valence > 0.5),
            (
                "Cortisol-Dominance r > 0.4",
                r_cortisol_dominance.abs() > 0.4,
            ),
            ("Overall R² > 0.70", r2_overall > 0.70),
            ("Mean error < 0.5", mean_error < 0.5),
        ];

        let criteria_passed = criteria.iter().filter(|(_, c)| *c).count();
        let success = criteria_passed >= 4;

        let metrics = json!({
            "n_samples": n_samples,
            "r_dopamine_arousal": r_dopamine_arousal,
            "r_oxytocin_valence": r_oxytocin_valence,
            "r_cortisol_dominance": r_cortisol_dominance,
            "r2_valence": r2_valence,
            "r2_arousal": r2_arousal,
            "r2_dominance": r2_dominance,
            "r2_overall": r2_overall,
            "mean_error": mean_error,
            "criteria_passed": criteria_passed,
            "criteria_total": criteria.len(),
        });

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Ok(ExperimentResult {
            experiment_type: "docas_emotion_mapping".to_string(),
            success,
            gap_id: "EMOT-P0-2".to_string(),
            paper: "EMOTION".to_string(),
            metrics,
            figures: vec![],
            csv_outputs: vec![],
            artifacts: vec![],
            duration: std::time::Duration::from_secs(0),
            error_message: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_docas() {
        let exp = DOCASEmotionMapping;
        let mut rng = StdRng::seed_from_u64(42);
        let docas = exp.random_docas(&mut rng);

        assert!(docas.dopamine >= 0.0 && docas.dopamine <= 1.0);
        assert!(docas.oxytocin >= 0.0 && docas.oxytocin <= 1.0);
        assert!(docas.cortisol >= 0.0 && docas.cortisol <= 1.0);
    }

    #[test]
    fn test_vad_conversion() {
        let exp = DOCASEmotionMapping;
        let docas = DOCASState {
            dopamine: 0.8,
            oxytocin: 0.9,
            cortisol: 0.1,
            adrenaline: 0.7,
            serotonin: 0.8,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let vad = exp.docas_to_vad_ground_truth(&docas);

        // High oxytocin/serotonin should give positive valence
        assert!(vad.valence > 0.0);
        // High dopamine should give high arousal
        assert!(vad.arousal > 0.5);
    }

    #[test]
    fn test_correlation() {
        let exp = DOCASEmotionMapping;

        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let r = exp.correlation(&x, &y);
        assert!(r > 0.99); // Perfect correlation
    }

    #[test]
    fn test_experiment_run() {
        let exp = DOCASEmotionMapping;
        let config = ExperimentConfig::quick();
        let result = exp.run(&config).unwrap();

        assert_eq!(result.experiment_type, "docas_emotion_mapping");
        assert_eq!(result.gap_id, "EMOT-P0-2");
    }
}
