//! Meta-analysis computation
//!
//! Implements fixed-effect and random-effects meta-analysis models.

use super::types::*;
use tracing::{debug, info};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Meta-analysis calculator
#[derive(Debug)]
pub struct MetaAnalyzer {
    config: MetaAnalysisConfig,
}

impl MetaAnalyzer {
    /// Create a new meta-analyzer
    #[must_use]
    pub const fn new(config: MetaAnalysisConfig) -> Self {
        Self { config }
    }

    fn build_empty_result(&self) -> MetaAnalysisResult {
        MetaAnalysisResult {
            config: self.config.clone(),
            k: 0,
            total_n: 0,
            pooled_effect: 0.0,
            pooled_se: 0.0,
            confidence_interval: (0.0, 0.0),
            prediction_interval: None,
            heterogeneity: HeterogeneityStats {
                q: 0.0,
                df: 0,
                q_pvalue: 1.0,
                i_squared: 0.0,
                h_squared: 1.0,
                tau_squared: 0.0,
                tau: 0.0,
            },
            bias_tests: Vec::new(),
            study_results: Vec::new(),
            model_fit: ModelFitStats {
                log_likelihood: None,
                aic: None,
                bic: None,
                r_squared: None,
            },
        }
    }

    fn filter_valid_studies(studies: &[Study]) -> Vec<Study> {
        studies
            .iter()
            .filter(|s| {
                s.effect_size.is_finite() && s.standard_error.is_finite() && s.standard_error > 0.0
            })
            .cloned()
            .collect()
    }

    fn calculate_study_weights(variances: &[f64], tau_squared: f64) -> (Vec<f64>, f64) {
        let weights: Vec<f64> = variances.iter().map(|v| 1.0 / (v + tau_squared)).collect();
        let sum_weights: f64 = weights.iter().sum();
        (weights, sum_weights)
    }

    fn calculate_weighted_mean_effect(studies: &[Study], weights: &[f64], sum_weights: f64) -> f64 {
        if sum_weights > 0.0 {
            studies
                .iter()
                .zip(weights.iter())
                .map(|(s, w)| s.effect_size * w)
                .sum::<f64>()
                / sum_weights
        } else {
            0.0
        }
    }

    fn calculate_pooled_se(sum_weights: f64) -> f64 {
        if sum_weights > 0.0 {
            (1.0 / sum_weights).sqrt()
        } else {
            0.0
        }
    }

    fn calculate_prediction_interval(
        &self,
        pooled_effect: f64,
        pooled_se: f64,
        tau_squared: f64,
        k: usize,
    ) -> Option<(f64, f64)> {
        if matches!(self.config.model, MetaAnalysisModel::RandomEffects) && k >= 3 {
            let pi_se = (pooled_se.powi(2) + tau_squared).sqrt();
            let df = (k - 2) as f64;
            let t_critical =
                self.student_t_quantile(1.0 - (1.0 - self.config.confidence_level) / 2.0, df);
            Some((
                pooled_effect - t_critical * pi_se,
                pooled_effect + t_critical * pi_se,
            ))
        } else {
            None
        }
    }

    fn build_individual_study_results(
        studies: &[Study],
        weights: &[f64],
        sum_weights: f64,
        pooled_effect: f64,
    ) -> Vec<StudyResult> {
        studies
            .iter()
            .zip(weights.iter())
            .map(|(s, w)| {
                let w_percent = 100.0 * w / sum_weights;
                let q_contrib = w * (s.effect_size - pooled_effect).powi(2);
                StudyResult {
                    study_id: s.id.clone(),
                    effect_size: s.effect_size,
                    standard_error: s.standard_error,
                    weight: *w,
                    weight_percent: w_percent,
                    q_contribution: q_contrib,
                }
            })
            .collect()
    }

    /// Run meta-analysis on studies
    pub fn analyze(&self, studies: &[Study]) -> MetaAnalysisResult {
        info!("Running meta-analysis with {} studies", studies.len());

        let valid_studies = Self::filter_valid_studies(studies);
        let studies = if valid_studies.len() < studies.len() {
            &valid_studies[..]
        } else {
            studies
        };

        if studies.is_empty() {
            return self.build_empty_result();
        }

        let k = studies.len();
        let total_n: usize = studies.iter().map(|s| s.sample_size).sum();

        let variances: Vec<f64> = studies
            .iter()
            .map(|s| s.standard_error.max(1e-6).powi(2))
            .collect();

        let tau_squared = match self.config.model {
            MetaAnalysisModel::FixedEffect => 0.0,
            MetaAnalysisModel::RandomEffects | MetaAnalysisModel::Mixed => {
                self.estimate_tau_squared(studies, &variances)
            }
        };

        debug!("Estimated tau² = {}", tau_squared);

        let (weights, sum_weights) = Self::calculate_study_weights(&variances, tau_squared);
        let pooled_effect = Self::calculate_weighted_mean_effect(studies, &weights, sum_weights);
        let pooled_se = Self::calculate_pooled_se(sum_weights);

        let z_critical = self.z_critical(self.config.confidence_level);
        let ci_lower = pooled_effect - z_critical * pooled_se;
        let ci_upper = pooled_effect + z_critical * pooled_se;

        let prediction_interval =
            self.calculate_prediction_interval(pooled_effect, pooled_se, tau_squared, k);

        let heterogeneity =
            self.calculate_heterogeneity(studies, &weights, &variances, tau_squared);

        let study_results =
            Self::build_individual_study_results(studies, &weights, sum_weights, pooled_effect);

        let bias_tests = self.test_publication_bias(studies);
        let model_fit =
            self.calculate_model_fit(studies, &study_results, pooled_effect, tau_squared);

        MetaAnalysisResult {
            config: self.config.clone(),
            k,
            total_n,
            pooled_effect,
            pooled_se,
            confidence_interval: (ci_lower, ci_upper),
            prediction_interval,
            heterogeneity,
            bias_tests,
            study_results,
            model_fit,
        }
    }

    /// Estimate between-study variance (tau²)
    #[must_use]
    pub fn estimate_tau_squared(&self, studies: &[Study], variances: &[f64]) -> f64 {
        super::heterogeneity::estimate_tau_squared(&self.config, studies, variances)
    }

    /// Calculate heterogeneity statistics
    fn calculate_heterogeneity(
        &self,
        studies: &[Study],
        weights: &[f64],
        variances: &[f64],
        tau_squared: f64,
    ) -> HeterogeneityStats {
        super::heterogeneity::calculate_heterogeneity(studies, weights, variances, tau_squared)
    }

    /// Test for publication bias
    fn test_publication_bias(&self, studies: &[Study]) -> Vec<BiasTestResult> {
        super::bias::test_publication_bias(&self.config, studies)
    }

    /// Calculate model fit statistics
    fn calculate_model_fit(
        &self,
        studies: &[Study],
        study_results: &[StudyResult],
        pooled_effect: f64,
        tau_squared: f64,
    ) -> ModelFitStats {
        super::heterogeneity::calculate_model_fit(
            studies,
            study_results,
            pooled_effect,
            tau_squared,
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compare simulation vs empirical meta-analyses
    #[must_use]
    pub fn compare_simulation_empirical(
        &self,
        empirical_studies: &[Study],
        simulation_studies: &[Study],
    ) -> SimulationEmpiricalComparison {
        super::comparison::compare_simulation_empirical(self, empirical_studies, simulation_studies)
    }

    /// Critical z-value using rational approximation of inverse normal CDF
    #[must_use]
    pub fn z_critical(&self, confidence: f64) -> f64 {
        super::dist::z_critical(confidence)
    }

    /// Student's t quantile function
    #[must_use]
    pub fn student_t_quantile(&self, p: f64, df: f64) -> f64 {
        super::dist::student_t_quantile(p, df)
    }

    /// Chi-square cumulative distribution function
    #[must_use]
    pub fn chi_square_cdf(&self, x: f64, df: f64) -> f64 {
        super::dist::chi_square_cdf(x, df)
    }

    /// Student's t cumulative distribution function
    #[must_use]
    pub fn student_t_cdf(&self, t: f64, df: f64) -> f64 {
        super::dist::student_t_cdf(t, df)
    }

    /// Standard normal CDF
    #[must_use]
    pub fn normal_cdf(&self, z: f64) -> f64 {
        super::dist::normal_cdf(z)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for MetaAnalyzer {
    fn default() -> Self {
        Self::new(MetaAnalysisConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_studies() -> Vec<Study> {
        vec![
            Study {
                id: "1".to_string(),
                name: "Study 1".to_string(),
                effect_size: 0.5,
                standard_error: 0.1,
                sample_size: 100,
                confidence_interval: None,
                year: Some(2020),
                source: StudySource::Empirical,
                metadata: HashMap::new(),
            },
            Study {
                id: "2".to_string(),
                name: "Study 2".to_string(),
                effect_size: 0.6,
                standard_error: 0.12,
                sample_size: 80,
                confidence_interval: None,
                year: Some(2021),
                source: StudySource::Empirical,
                metadata: HashMap::new(),
            },
            Study {
                id: "3".to_string(),
                // TAG: surface=simulation owner=research-team rule=SIM-001
                name: "Study 3".to_string(),
                effect_size: 0.4,
                standard_error: 0.08,
                sample_size: 150,
                confidence_interval: None,
                year: Some(2022),
                source: StudySource::Simulation,
                metadata: HashMap::new(),
            },
        ]
    }

    #[test]
    fn test_meta_analysis() {
        let config = MetaAnalysisConfig::default();
        let analyzer = MetaAnalyzer::new(config);
        let studies = create_test_studies();

        let result = analyzer.analyze(&studies);

        assert_eq!(result.k, 3);
        assert!(result.pooled_effect > 0.4 && result.pooled_effect < 0.6);
        assert!(result.heterogeneity.i_squared >= 0.0);
    }

    #[test]
    fn test_simulation_empirical_comparison() {
        let config = MetaAnalysisConfig::default();
        let analyzer = MetaAnalyzer::new(config);

        let empirical = vec![Study {
            id: "e1".to_string(),
            name: "Empirical 1".to_string(),
            effect_size: 0.5,
            standard_error: 0.1,
            sample_size: 100,
            confidence_interval: None,
            year: None,
            source: StudySource::Empirical,
            metadata: HashMap::new(),
        }];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let simulation = vec![Study {
            id: "s1".to_string(),
            name: "Simulation 1".to_string(),
            effect_size: 0.52,
            standard_error: 0.08,
            sample_size: 1000,
            confidence_interval: None,
            year: None,
            source: StudySource::Simulation,
            metadata: HashMap::new(),
        }];

        let comparison = analyzer.compare_simulation_empirical(&empirical, &simulation);

        assert!(comparison.comparison.effect_difference.abs() < 0.1);
    }

    #[test]
    fn test_fixed_effect() {
        let config = MetaAnalysisConfig {
            model: MetaAnalysisModel::FixedEffect,
            ..Default::default()
        };
        let analyzer = MetaAnalyzer::new(config);
        let studies = create_test_studies();

        let result = analyzer.analyze(&studies);
        assert_eq!(result.heterogeneity.tau_squared, 0.0);
        assert!(result.prediction_interval.is_none());
    }

    #[test]
    fn test_random_effects() {
        let config = MetaAnalysisConfig {
            model: MetaAnalysisModel::RandomEffects,
            ..Default::default()
        };
        let analyzer = MetaAnalyzer::new(config);
        let studies = create_test_studies();

        let result = analyzer.analyze(&studies);
        assert!(result.prediction_interval.is_some());
    }

    #[test]
    fn test_prediction_interval_none_when_k_less_than_three() {
        let config = MetaAnalysisConfig {
            model: MetaAnalysisModel::RandomEffects,
            ..Default::default()
        };
        let analyzer = MetaAnalyzer::new(config);
        let studies = vec![
            create_test_studies()[0].clone(),
            create_test_studies()[1].clone(),
        ];
        let result = analyzer.analyze(&studies);
        assert_eq!(result.k, 2);
        assert!(
            result.prediction_interval.is_none(),
            "Random-effects prediction interval requires k >= 3 studies"
        );
    }

    #[test]
    fn test_reml_and_ml_heterogeneity_methods() {
        let studies = create_test_studies();

        let reml_analyzer = MetaAnalyzer::new(MetaAnalysisConfig {
            heterogeneity_method: HeterogeneityMethod::REML,
            ..Default::default()
        });
        let reml_res = reml_analyzer.analyze(&studies);
        assert!(reml_res.heterogeneity.tau_squared >= 0.0);

        let ml_analyzer = MetaAnalyzer::new(MetaAnalysisConfig {
            heterogeneity_method: HeterogeneityMethod::MaximumLikelihood,
            ..Default::default()
        });
        let ml_res = ml_analyzer.analyze(&studies);
        assert!(ml_res.heterogeneity.tau_squared >= 0.0);
    }

    #[test]
    fn test_exact_chisquare_cdf_boundaries() {
        let analyzer = MetaAnalyzer::default();

        // Exact zero at x = 0 for nu = 1 and nu = 2
        assert_eq!(analyzer.chi_square_cdf(0.0, 1.0), 0.0);
        assert_eq!(analyzer.chi_square_cdf(0.0, 2.0), 0.0);
        assert!(analyzer.chi_square_cdf(1e-12, 1.0) < 1e-5);

        // Exact closed form for nu = 2: 1 - exp(-x / 2)
        // At x = 2.0: 1 - exp(-1) = 0.6321205588
        let val2 = analyzer.chi_square_cdf(2.0, 2.0);
        let expected = 1.0 - (-1.0f64).exp();
        assert!((val2 - expected).abs() < 1e-6);

        // For nu = 1: 2 * Phi(1) - 1 at x = 1 (z = 1) -> 0.682689
        let val1 = analyzer.chi_square_cdf(1.0, 1.0);
        assert!((val1 - 0.682689).abs() < 0.005);
    }

    #[test]
    fn test_begg_and_trim_fill_tie_safety() {
        let analyzer = MetaAnalyzer::new(MetaAnalysisConfig {
            bias_test: PublicationBiasTest::Begg,
            ..Default::default()
        });

        // Identical standard errors (tie handling)
        let tied_studies = vec![
            Study {
                id: "1".to_string(),
                name: "1".to_string(),
                effect_size: 0.2,
                standard_error: 0.1,
                sample_size: 100,
                confidence_interval: None,
                year: None,
                source: StudySource::Simulation,
                metadata: HashMap::new(),
            },
            Study {
                id: "2".to_string(),
                name: "2".to_string(),
                effect_size: 0.3,
                standard_error: 0.1,
                sample_size: 100,
                confidence_interval: None,
                year: None,
                source: StudySource::Simulation,
                metadata: HashMap::new(),
            },
            Study {
                id: "3".to_string(),
                name: "3".to_string(),
                effect_size: 0.4,
                standard_error: 0.1,
                sample_size: 100,
                confidence_interval: None,
                year: None,
                source: StudySource::Simulation,
                metadata: HashMap::new(),
            },
        ];

        let res = analyzer.analyze(&tied_studies);
        assert_eq!(res.bias_tests.len(), 1);
        assert_eq!(res.bias_tests[0].statistic, 0.0);
        assert_eq!(res.bias_tests[0].p_value, 1.0);
    }

    #[test]
    fn test_meta_analysis_trim_and_fill_respects_estimator_and_single_study() {
        let single_study = vec![Study {
            id: "1".to_string(),
            name: "1".to_string(),
            effect_size: 0.5,
            standard_error: 0.1,
            sample_size: 100,
            confidence_interval: None,
            year: None,
            source: StudySource::Simulation,
            metadata: HashMap::new(),
        }];

        let analyzer = MetaAnalyzer::new(MetaAnalysisConfig {
            model: MetaAnalysisModel::RandomEffects,
            heterogeneity_method: HeterogeneityMethod::REML,
            bias_test: PublicationBiasTest::TrimAndFill,
            ..Default::default()
        });

        let res = analyzer.analyze(&single_study);
        assert_eq!(res.k, 1);
        assert_eq!(res.heterogeneity.tau_squared, 0.0);
        assert_eq!(res.bias_tests.len(), 1);
        assert!(res.bias_tests[0].interpretation.contains("Insufficient"));

        let studies = create_test_studies();
        let res_reml = analyzer.analyze(&studies);
        assert_eq!(res_reml.k, 3);
        assert!(!res_reml.bias_tests.is_empty());
    }
}
