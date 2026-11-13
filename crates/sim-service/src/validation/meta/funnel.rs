//! Funnel plot generation
//!
//! Creates data for funnel plot visualization to assess publication bias.

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Funnel plot generator
#[derive(Debug)]
pub struct FunnelPlot;

impl FunnelPlot {
    /// Create a new funnel plot generator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generate funnel plot data
    #[must_use]
    pub fn generate(&self, result: &MetaAnalysisResult) -> FunnelPlotData {
        // Study points
        let studies: Vec<FunnelStudy> = result
            .study_results
            .iter()
            .map(|sr| FunnelStudy {
                id: sr.study_id.clone(),
                effect_size: sr.effect_size,
                standard_error: sr.standard_error,
                precision: if sr.standard_error <= 0.0 {
                    0.0
                } else {
                    1.0 / sr.standard_error
                },
            })
            .collect();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Generate pseudo-confidence intervals for funnel lines
        let pooled = result.pooled_effect;
        let se_values: Vec<f64> = (1..=20).map(|i| i as f64 * 0.01).collect();

        let pseudo_ci_lower: Vec<(f64, f64)> = se_values
            .iter()
            .map(|se| (*se, pooled - 1.96 * se))
            .collect();
        let pseudo_ci_upper: Vec<(f64, f64)> = se_values
            .iter()
            .map(|se| (*se, pooled + 1.96 * se))
            .collect();

        // Asymmetry test result
        let asymmetry_test = result.bias_tests.first().cloned();

        FunnelPlotData {
            studies,
            pooled_effect: pooled,
            pseudo_ci_lower,
            pseudo_ci_upper,
            asymmetry_test,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate with contour-enhanced funnel plot
    #[must_use]
    pub fn generate_contour(
        &self,
        result: &MetaAnalysisResult,
        contours: &[f64],
    ) -> FunnelPlotData {
        // For now, same as regular funnel but could add significance contours
        let _ = contours;
        self.generate(result)
    }

    /// Calculate trim-and-fill adjusted studies
    #[must_use]
    pub fn trim_and_fill(&self, result: &MetaAnalysisResult) -> Vec<FunnelStudy> {
        // Simplified trim-and-fill implementation
        // In practice, this would iterate to find and impute missing studies

        let original: Vec<FunnelStudy> = result
            .study_results
            .iter()
            .map(|sr| FunnelStudy {
                id: sr.study_id.clone(),
                effect_size: sr.effect_size,
                standard_error: sr.standard_error,
                precision: if sr.standard_error <= 0.0 {
                    0.0
                } else {
                    1.0 / sr.standard_error
                },
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            })
            .collect();

        // Detect asymmetry and add imputed studies
        let asymmetry_score = self.detect_asymmetry(&original);

        if asymmetry_score > 0.5 && !original.is_empty() {
            // Determine side of asymmetry: are studies skewed to the right (positive) or left (negative) of pooled_effect?
            let pooled = result.pooled_effect;
            let right_count = original.iter().filter(|s| s.effect_size > pooled).count();
            let left_count = original.iter().filter(|s| s.effect_size < pooled).count();
            let overrepresented_right = right_count >= left_count;

            // Collect and sort candidate studies on the overrepresented side by distance from pooled_effect descending
            let mut candidates: Vec<&FunnelStudy> = original
                .iter()
                .filter(|s| {
                    if overrepresented_right {
                        s.effect_size > pooled
                    } else {
                        s.effect_size < pooled
                    }
                })
                .collect();
            candidates.sort_by(|a, b| {
                let dist_a = (a.effect_size - pooled).abs();
                let dist_b = (b.effect_size - pooled).abs();
                dist_b.total_cmp(&dist_a).then_with(|| a.id.cmp(&b.id))
            });

            let n_imputed = if candidates.is_empty() {
                0
            } else {
                ((original.len() as f64 * 0.2).ceil() as usize)
                    .min(candidates.len())
                    .max(1)
            };
            let mut imputed = original.clone();

            for (i, study) in candidates.into_iter().take(n_imputed).enumerate() {
                imputed.push(FunnelStudy {
                    id: format!("imputed_{i}"),
                    effect_size: pooled - (study.effect_size - pooled),
                    standard_error: study.standard_error,
                    precision: study.precision,
                });
            }
            imputed
        } else {
            original
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Detect asymmetry in funnel plot
    fn detect_asymmetry(&self, studies: &[FunnelStudy]) -> f64 {
        if studies.len() < 5 {
            return 0.0;
        }

        // Simple correlation between effect and SE
        let mean_es = studies.iter().map(|s| s.effect_size).sum::<f64>() / studies.len() as f64;
        let mean_se = studies.iter().map(|s| s.standard_error).sum::<f64>() / studies.len() as f64;

        let numerator: f64 = studies
            .iter()
            .map(|s| (s.effect_size - mean_es) * (s.standard_error - mean_se))
            .sum();

        let denom_es: f64 = studies
            .iter()
            .map(|s| (s.effect_size - mean_es).powi(2))
            .sum::<f64>()
            .sqrt();
        let denom_se: f64 = studies
            .iter()
            .map(|s| (s.standard_error - mean_se).powi(2))
            .sum::<f64>()
            .sqrt();

        if denom_es > 0.0 && denom_se > 0.0 {
            (numerator / (denom_es * denom_se)).abs()
        } else {
            0.0
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate radial plot data (alternative to funnel)
    #[must_use]
    pub fn radial_plot(&self, result: &MetaAnalysisResult) -> Vec<(f64, f64)> {
        // Convert to radial coordinates
        result
            .study_results
            .iter()
            .map(|sr| {
                let se = if sr.standard_error <= 0.0 {
                    1e-6
                } else {
                    sr.standard_error
                };
                let theta = (sr.effect_size - result.pooled_effect) / se;
                let r = 1.0 / se;
                (theta, r)
            })
            .collect()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for FunnelPlot {
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
            k: 5,
            total_n: 500,
            pooled_effect: 0.5,
            pooled_se: 0.05,
            confidence_interval: (0.4, 0.6),
            prediction_interval: None,
            heterogeneity: HeterogeneityStats {
                q: 8.0,
                df: 4,
                q_pvalue: 0.1,
                i_squared: 30.0,
                h_squared: 1.4,
                tau_squared: 0.02,
                tau: 0.14,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            },
            bias_tests: vec![BiasTestResult {
                test: "Egger".to_string(),
                statistic: 1.5,
                p_value: 0.2,
                interpretation: "No significant bias".to_string(),
                adjusted_estimate: None,
            }],
            study_results: (0..5)
                .map(|i| StudyResult {
                    study_id: format!("s{i}"),
                    effect_size: 0.4 + i as f64 * 0.05,
                    standard_error: 0.1 - i as f64 * 0.01,
                    weight: 0.2,
                    weight_percent: 20.0,
                    q_contribution: 1.0,
                })
                .collect(),
            model_fit: ModelFitStats {
                log_likelihood: None,
                aic: None,
                bic: None,
                r_squared: None,
            },
        }
    }

    #[test]
    fn test_funnel_plot_generation() {
        let plot = FunnelPlot::new();
        let result = create_test_meta_result();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let data = plot.generate(&result);

        assert_eq!(data.studies.len(), 5);
        assert!(!data.pseudo_ci_lower.is_empty());
        assert!(!data.pseudo_ci_upper.is_empty());
        assert!(data.asymmetry_test.is_some());
    }

    #[test]
    fn test_trim_and_fill() {
        let plot = FunnelPlot::new();
        let result = create_test_meta_result();

        let adjusted = plot.trim_and_fill(&result);

        // Should have original studies
        assert!(adjusted.len() >= 5);
    }

    #[test]
    fn test_radial_plot() {
        let plot = FunnelPlot::new();
        let result = create_test_meta_result();

        let radial = plot.radial_plot(&result);

        assert_eq!(radial.len(), 5);
    }
}
