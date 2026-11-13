//! Forest plot generation
//!
//! Creates data for forest plot visualization of meta-analysis results.

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Forest plot generator
#[derive(Debug)]
pub struct ForestPlot;

impl ForestPlot {
    /// Create a new forest plot generator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generate forest plot data from meta-analysis result
    #[must_use]
    pub fn generate(&self, result: &MetaAnalysisResult) -> ForestPlotData {
        let z_critical = match result.config.confidence_level {
            c if c >= 0.99 => 2.576,
            c if c >= 0.95 => 1.96,
            c if c >= 0.90 => 1.645,
            _ => 1.96,
        };

        // Individual studies
        let studies: Vec<ForestStudy> = result
            .study_results
            .iter()
            .map(|sr| {
                let ci_lower = sr.effect_size - z_critical * sr.standard_error;
                let ci_upper = sr.effect_size + z_critical * sr.standard_error;

                ForestStudy {
                    id: sr.study_id.clone(),
                    name: sr.study_id.clone(), // Could look up full name
                    effect_size: sr.effect_size,
                    ci_lower,
                    ci_upper,
                    weight: sr.weight_percent,
                    sample_size: 0, // Not stored in result, would need original study
                }
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Pooled effect
        let pooled_effect = ForestPooledEffect {
            effect_size: result.pooled_effect,
            ci_lower: result.confidence_interval.0,
            ci_upper: result.confidence_interval.1,
            pi_lower: result.prediction_interval.map(|(l, _)| l),
            pi_upper: result.prediction_interval.map(|(_, u)| u),
        };

        ForestPlotData {
            studies,
            pooled_effect,
            heterogeneity: result.heterogeneity.clone(),
            model: format!("{:?}", result.config.model),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate sorted forest plot (by effect size with deterministic ID tie-break)
    #[must_use]
    pub fn generate_sorted(&self, result: &MetaAnalysisResult) -> ForestPlotData {
        let mut data = self.generate(result);
        data.studies.sort_by(|a, b| {
            a.effect_size
                .partial_cmp(&b.effect_size)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        data
    }

    /// Generate with weights visual
    #[must_use]
    pub fn generate_with_weights(&self, result: &MetaAnalysisResult) -> ForestPlotData {
        let mut data = self.generate(result);
        // Sort by weight descending for visual impact with deterministic ID tie-break
        data.studies.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        data
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ForestPlot {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_meta_result() -> MetaAnalysisResult {
        MetaAnalysisResult {
            config: MetaAnalysisConfig::default(),
            k: 3,
            total_n: 300,
            pooled_effect: 0.5,
            pooled_se: 0.05,
            confidence_interval: (0.4, 0.6),
            prediction_interval: Some((0.3, 0.7)),
            heterogeneity: HeterogeneityStats {
                q: 5.0,
                df: 2,
                q_pvalue: 0.1,
                i_squared: 25.0,
                h_squared: 1.33,
                tau_squared: 0.01,
                tau: 0.1,
            },
            bias_tests: Vec::new(),
            study_results: vec![
                StudyResult {
                    study_id: "s1".to_string(),
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    effect_size: 0.4,
                    standard_error: 0.1,
                    weight: 0.5,
                    weight_percent: 33.3,
                    q_contribution: 1.0,
                },
                StudyResult {
                    study_id: "s2".to_string(),
                    effect_size: 0.5,
                    standard_error: 0.08,
                    weight: 0.7,
                    weight_percent: 46.7,
                    q_contribution: 0.5,
                },
                StudyResult {
                    study_id: "s3".to_string(),
                    effect_size: 0.6,
                    standard_error: 0.12,
                    weight: 0.3,
                    weight_percent: 20.0,
                    q_contribution: 1.5,
                },
            ],
            model_fit: ModelFitStats {
                log_likelihood: None,
                aic: None,
                bic: None,
                r_squared: None,
            },
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_forest_plot_generation() {
        let plot = ForestPlot::new();
        let result = create_test_meta_result();

        let data = plot.generate(&result);

        assert_eq!(data.studies.len(), 3);
        assert!(data.pooled_effect.effect_size > 0.0);
        assert_eq!(data.model, "RandomEffects");
    }

    #[test]
    fn test_sorted_forest() {
        let plot = ForestPlot::new();
        let result = create_test_meta_result();

        let data = plot.generate_sorted(&result);

        assert!(data.studies[0].effect_size <= data.studies[1].effect_size);
        assert!(data.studies[1].effect_size <= data.studies[2].effect_size);
    }
}
