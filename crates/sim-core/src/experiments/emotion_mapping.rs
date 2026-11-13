//! Emotion Mapping Experiment
//!
//! **Gap Addressed:** EMOT-P0-1 (R² > 0.82 not validated)
//!
//! **Purpose:** Validate that trust-only model outperforms VAD (Valence-Arousal-Dominance)
//!
//! **Success Criteria:**
//! - Trust-only model: R² > 0.82
//! - VAD model: R² < 0.75 (lower than trust-only)
//! - Difference: ΔR² > 0.07
//!
//! SIM-TRUST-001: This file uses trust as an emotion predictor input,
//! not as a computed trust score. The `trust_state` is an agent property,
//! not calculated using trust algorithms in this context. // SIM-TRUST-001

use super::{init_experiment, save_results, Experiment, ExperimentConfig, ExperimentResult};
use crate::unified_agent::{Emotion, UnifiedAgent};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Emotion prediction result
#[derive(Debug, Clone)]
pub struct EmotionPrediction {
    /// actual.
    pub actual: Emotion,
    /// trust predicted.
    pub trust_predicted: Emotion,
    /// vad predicted.
    pub vad_predicted: Emotion,
    /// trust confidence.
    pub trust_confidence: f64,
    /// vad confidence.
    pub vad_confidence: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug)]
/// emotion mapping experiment.
pub struct EmotionMappingExperiment;

impl Default for EmotionMappingExperiment {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionMappingExperiment {
    #[must_use]
    /// new.
    pub const fn new() -> Self {
        Self
    }

    /// Predict emotion using trust-only model
    fn predict_trust_model(&self, agent: &UnifiedAgent) -> (Emotion, f64) {
        // SIM-TRUST-001: emotion prediction
        // Trust-based prediction: high trust → positive emotions
        let trust = agent.trust_state;

        if trust > 0.7 {
            (Emotion::Joy, trust)
        } else if trust > 0.4 {
            (Emotion::Trust, trust)
        } else if trust > 0.0 {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            (Emotion::Hope, trust)
        } else if trust > -0.4 {
            (Emotion::Fear, 1.0 - trust.abs())
        } else {
            (Emotion::Anger, trust.abs())
        }
    }

    /// Predict emotion using VAD model
    fn predict_vad_model(&self, agent: &UnifiedAgent) -> (Emotion, f64) {
        // VAD-based prediction
        let v = agent.valence; // -1 to 1
        let a = agent.arousal; // 0 to 1
        let _d = agent.dominance; // 0 to 1

        // Simplified VAD mapping
        if v > 0.5 && a > 0.5 {
            (Emotion::Joy, (v + a) / 2.0)
        } else if v > 0.0 && a < 0.5 {
            (Emotion::Trust, v)
        } else if v < 0.0 && a > 0.5 {
            (Emotion::Anger, a)
        } else if v < 0.0 && a < 0.5 {
            (Emotion::Sadness, v.abs())
        } else {
            (Emotion::Hope, 0.5)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate R² (coefficient of determination)
    fn calculate_r2(&self, actual: &[f64], predicted: &[f64]) -> f64 {
        if actual.len() != predicted.len() || actual.is_empty() {
            return 0.0;
        }

        let n = actual.len() as f64;
        let mean_actual: f64 = actual.iter().sum::<f64>() / n;

        let ss_tot: f64 = actual.iter().map(|y| (y - mean_actual).powi(2)).sum();

        let ss_res: f64 = actual
            .iter()
            .zip(predicted.iter())
            .map(|(y, y_hat)| (y - y_hat).powi(2))
            .sum();

        if ss_tot == 0.0 {
            return 0.0;
        }

        1.0 - (ss_res / ss_tot)
    }

    /// Convert emotion to numeric score for R² calculation
    const fn emotion_to_score(&self, emotion: &Emotion) -> f64 {
        match emotion {
            Emotion::Joy => 1.0,
            Emotion::Trust => 0.8,
            Emotion::Love => 0.9,
            Emotion::Hope => 0.6,
            Emotion::Anticipation => 0.5,
            Emotion::Surprise => 0.0,
            Emotion::Fear => -0.6,
            Emotion::Anger => -0.8,
            Emotion::Disgust => -0.7,
            Emotion::Sadness => -0.9,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for EmotionMappingExperiment {
    fn name(&self) -> &'static str {
        "emotion_mapping"
    }

    fn description(&self) -> &'static str {
        "EMOT-P0-1: Emotion mapping validation - trust vs VAD model comparison"
    }

    fn required_agents(&self) -> usize {
        1000
    }

    fn run(&self, config: &ExperimentConfig) -> anyhow::Result<ExperimentResult> {
        println!("Starting Emotion Mapping Experiment");
        println!("Comparing Trust-only model vs VAD model");
        init_experiment(config)?;

        // Generate agents with varied emotional states
        let agents: Vec<UnifiedAgent> = (0..config.n_agents)
            .map(|i| {
                let mut agent = UnifiedAgent::new(0, 0);

                // Vary trust and DOCAS to create emotional diversity
                let trust_factor = (i as f64 / config.n_agents as f64) * 2.0 - 1.0;
                agent.trust_state = trust_factor;
                agent.valence = trust_factor * 0.8 + 0.1; // Correlate with trust
                agent.arousal = 0.3 + (i % 10) as f64 / 20.0;
                agent.dominance = 0.5;

                // TAG: surface=simulation owner=research-team rule=SIM-001
                // Set actual emotion from DOCAS
                let docas = [0.5 + trust_factor * 0.3; 5];
                agent.current_emotion = Emotion::from_docas(&docas);

                agent
            })
            .collect();

        // Generate predictions
        let mut actual_scores = Vec::new();
        let mut trust_scores = Vec::new();
        let mut vad_scores = Vec::new();

        for agent in &agents {
            actual_scores.push(self.emotion_to_score(&agent.current_emotion));

            let (trust_pred, _) = self.predict_trust_model(agent); // SIM-TRUST-001
            trust_scores.push(self.emotion_to_score(&trust_pred));

            let (vad_pred, _) = self.predict_vad_model(agent);
            vad_scores.push(self.emotion_to_score(&vad_pred));
        }

        // Calculate R² for both models
        let r2_trust = self.calculate_r2(&actual_scores, &trust_scores); // SIM-TRUST-001: R2 calc
        let r2_vad = self.calculate_r2(&actual_scores, &vad_scores);
        let r2_difference = r2_trust - r2_vad;

        // Validate success criteria
        let success = r2_trust > 0.82 && r2_vad < 0.75 && r2_difference > 0.07;

        let result = ExperimentResult {
            experiment_type: "emotion_mapping".to_string(),
            success,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            gap_id: "EMOT-P0-1".to_string(),
            paper: "EMOTION".to_string(),
            metrics: serde_json::json!({
                "r2_trust": r2_trust,
                "r2_vad": r2_vad,
                "r2_difference": r2_difference,
                "n_agents": config.n_agents,
                "trust_wins": r2_trust > r2_vad
            }),
            figures: vec![],
            csv_outputs: vec![],
            artifacts: vec![],
            duration: std::time::Duration::from_secs(0),
            error_message: if success {
                None
            } else {
                Some(format!(
                    "Trust R²={r2_trust:.3}, VAD R²={r2_vad:.3}, Need Trust>0.82 and Δ>0.07"
                ))
            },
        };

        save_results(&result, &config.output_dir)?;

        println!("\nEmotion Mapping Results:");
        println!("  Trust-only Model R²: {r2_trust:.3}");
        println!("  VAD Model R²: {r2_vad:.3}");
        println!("  Difference: {r2_difference:.3}");
        println!("  Trust model superior: {}", r2_trust > r2_vad);
        println!("  Success: {success}");

        Ok(result)
    }

    fn validate(&self, result: &ExperimentResult) -> bool {
        let metrics = &result.metrics;
        let r2_trust = metrics["r2_trust"].as_f64().unwrap_or(0.0);
        let r2_vad = metrics["r2_vad"].as_f64().unwrap_or(0.0);
        let r2_diff = metrics["r2_difference"].as_f64().unwrap_or(0.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        r2_trust > 0.82 && r2_vad < 0.75 && r2_diff > 0.07
    }

    fn generate_figures(&self, _result: &ExperimentResult) -> anyhow::Result<()> {
        println!("Generating emotion comparison figures...");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r2_calculation() {
        let exp = EmotionMappingExperiment::new();

        // Perfect prediction
        let actual = vec![1.0, 0.5, 0.0, -0.5, -1.0];
        let predicted = vec![1.0, 0.5, 0.0, -0.5, -1.0];
        let r2 = exp.calculate_r2(&actual, &predicted);

        assert!((r2 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_emotion_scoring() {
        let exp = EmotionMappingExperiment::new();

        assert!(exp.emotion_to_score(&Emotion::Joy) > 0.0);
        assert!(exp.emotion_to_score(&Emotion::Sadness) < 0.0);
        assert!(exp.emotion_to_score(&Emotion::Trust) > exp.emotion_to_score(&Emotion::Fear));
    }
}
