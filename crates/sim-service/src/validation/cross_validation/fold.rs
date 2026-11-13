//! Cross-validation orchestration
//!
//! Coordinates splitting, training, evaluation across folds.

use std::collections::HashMap;

use super::metrics::create_metric;
use super::splitter::*;
use super::types::*;
use tracing::{debug, info};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cross-validation orchestrator
#[derive(Debug)]
pub struct CrossValidator {
    config: CrossValidationConfig,
}

impl CrossValidator {
    /// Create a new cross-validator
    #[must_use]
    pub const fn new(config: CrossValidationConfig) -> Self {
        Self { config }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Run cross-validation with a model
    pub fn validate<M: Model>(
        &self,
        dataset: &Dataset,
        model_factory: &dyn Fn() -> M,
        metric_names: &[String],
    ) -> CrossValidationResult {
        let start_time = std::time::Instant::now();

        // Generate folds
        let folds = self.generate_folds(dataset);
        info!("Generated {} folds", folds.len());

        let mut fold_results = Vec::new();

        for fold in &folds {
            debug!("Processing fold {}/{}", fold.index + 1, folds.len());

            // Split data
            let (train_features, train_target) =
                self.extract_fold_data(dataset, &fold.train_indices);
            let (val_features, val_target) = self.extract_fold_data(dataset, &fold.val_indices);

            // Create and train model
            let fold_start = std::time::Instant::now();
            let mut model = model_factory();
            model.fit(&train_features, &train_target);
            let fit_time = fold_start.elapsed().as_secs_f64();

            // Predict
            let pred_start = std::time::Instant::now();
            let train_preds = model.predict(&train_features);
            let val_preds = model.predict(&val_features);
            let predict_time = pred_start.elapsed().as_secs_f64();

            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            // Calculate metrics
            let train_predictions = Predictions {
                true_values: train_target.clone(),
                predicted: train_preds,
                probabilities: None,
            };
            let val_predictions = Predictions {
                true_values: val_target.clone(),
                predicted: val_preds,
                probabilities: None,
            };

            let train_metrics = self.calculate_metrics(&train_predictions, metric_names);
            let val_metrics = self.calculate_metrics(&val_predictions, metric_names);

            fold_results.push(FoldResult {
                fold: fold.index,
                train_size: train_target.len(),
                val_size: val_target.len(),
                train_metrics,
                val_metrics,
                fit_time_secs: fit_time,
                predict_time_secs: predict_time,
            });
        }

        let aggregated_metrics = self.aggregate_metrics(&fold_results, metric_names);

        CrossValidationResult {
            config: self.config.clone(),
            fold_results,
            aggregated_metrics,
            computation_time_secs: start_time.elapsed().as_secs_f64(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate folds based on configuration
    #[must_use]
    pub fn generate_folds(&self, dataset: &Dataset) -> Vec<Fold> {
        match self.config.method {
            CrossValidationMethod::KFold => {
                let splitter = KFold {
                    n_splits: self.config.n_folds,
                    shuffle: self.config.shuffle,
                    seed: self.config.seed,
                };
                splitter.split(dataset)
            }
            CrossValidationMethod::StratifiedKFold => {
                let splitter = StratifiedKFold {
                    n_splits: self.config.n_folds,
                    shuffle: self.config.shuffle,
                    seed: self.config.seed,
                };
                splitter.split(dataset)
            }
            CrossValidationMethod::GroupKFold => {
                let splitter = GroupKFold {
                    n_splits: self.config.n_folds,
                };
                // TAG: surface=simulation owner=research-team rule=SIM-001
                splitter.split(dataset)
            }
            CrossValidationMethod::TimeSeriesSplit => {
                let splitter = TimeSeriesSplit {
                    n_splits: self.config.n_folds,
                    max_train_size: None,
                    test_size: None,
                };
                splitter.split(dataset)
            }
            CrossValidationMethod::ShuffleSplit => {
                let splitter = TrainTestSplit {
                    test_size: self.config.test_size.unwrap_or(0.2),
                    shuffle: self.config.shuffle,
                    seed: self.config.seed,
                };
                splitter.split(dataset)
            }
            CrossValidationMethod::LeaveOneOut => {
                let splitter = LeaveOneOut;
                splitter.split(dataset)
            }
            CrossValidationMethod::LeavePOut => {
                // Leave-p-out not implemented, fallback to KFold
                let splitter = KFold {
                    n_splits: self.config.n_folds,
                    shuffle: self.config.shuffle,
                    seed: self.config.seed,
                };
                splitter.split(dataset)
            }
        }
    }

    /// Extract data for given indices
    fn extract_fold_data(&self, dataset: &Dataset, indices: &[usize]) -> (Vec<Vec<f64>>, Vec<f64>) {
        let max_n = dataset.features.len().min(dataset.target.len());
        let features: Vec<_> = indices
            .iter()
            .filter(|&&i| i < max_n)
            .map(|&i| dataset.features[i].clone())
            .collect();
        let target: Vec<_> = indices
            .iter()
            .filter(|&&i| i < max_n)
            .map(|&i| dataset.target[i])
            .collect();
        (features, target)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate metrics for predictions
    fn calculate_metrics(
        &self,
        predictions: &Predictions,
        metric_names: &[String],
    ) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        for name in metric_names {
            let metric = create_metric(name);
            let value = metric.calculate(predictions);
            metrics.insert(name.clone(), value);
        }

        metrics
    }

    /// Aggregate metrics across folds
    fn aggregate_metrics(
        &self,
        fold_results: &[FoldResult],
        metric_names: &[String],
    ) -> AggregatedMetrics {
        let mut mean = HashMap::new();
        let mut std = HashMap::new();
        let mut min = HashMap::new();
        let mut max = HashMap::new();
        let mut ci_95 = HashMap::new();

        for name in metric_names {
            let values: Vec<f64> = fold_results
                .iter()
                .filter_map(|f| f.val_metrics.get(name).copied())
                .filter(|v| v.is_finite())
                .collect();

            // TAG: surface=simulation owner=research-team rule=SIM-001
            if !values.is_empty() {
                let n = values.len();
                let m = values.iter().sum::<f64>() / n as f64;
                let variance = if n > 1 {
                    values.iter().map(|&v| (v - m).powi(2)).sum::<f64>() / (n - 1) as f64
                } else {
                    0.0
                };
                let s = variance.sqrt();
                let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
                let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

                // 95% CI using Student-t distribution critical values
                let df = n.saturating_sub(1);
                let t_crit = match df {
                    0 => 1.96,
                    1 => 12.706,
                    2 => 4.303,
                    3 => 3.182,
                    4 => 2.776,
                    5 => 2.571,
                    6 => 2.447,
                    7 => 2.365,
                    8 => 2.306,
                    9 => 2.262,
                    10..=14 => 2.201,
                    15..=19 => 2.110,
                    20..=29 => 2.064,
                    _ => 1.96,
                };
                let ci_margin = if n > 0 {
                    t_crit * s / (n as f64).sqrt()
                } else {
                    0.0
                };

                mean.insert(name.clone(), m);
                std.insert(name.clone(), s);
                min.insert(name.clone(), min_val);
                max.insert(name.clone(), max_val);
                ci_95.insert(name.clone(), (m - ci_margin, m + ci_margin));
            }
        }

        AggregatedMetrics {
            mean,
            std,
            min,
            max,
            ci_95,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate learning curves
    pub fn learning_curve<M: Model>(
        &self,
        dataset: &Dataset,
        model_factory: &dyn Fn() -> M,
        metric_name: &str,
        train_sizes: &[f64],
    ) -> Vec<LearningCurvePoint> {
        let mut curve = Vec::new();
        let n = dataset.target.len();
        if n < 2 {
            return curve;
        }

        let splitter = TrainTestSplit {
            test_size: 0.2,
            shuffle: true,
            seed: self.config.seed,
        };
        let folds = splitter.split(dataset);
        if folds.is_empty() || folds[0].train_indices.is_empty() {
            return curve;
        }
        let test_indices = folds[0].val_indices.clone();
        let train_indices_all = folds[0].train_indices.clone();

        let (test_features, test_target) = self.extract_fold_data(dataset, &test_indices);

        for &fraction in train_sizes {
            if !fraction.is_finite() || fraction <= 0.0 {
                continue;
            }
            let train_size =
                ((n as f64 * fraction * 0.8) as usize).clamp(1, train_indices_all.len());
            let train_indices = &train_indices_all[..train_size.min(train_indices_all.len())];

            let (train_features, train_target) = self.extract_fold_data(dataset, train_indices);

            // Train model
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            let mut model = model_factory();
            model.fit(&train_features, &train_target);

            // Predict on train and test
            let train_preds = model.predict(&train_features);
            let test_preds = model.predict(&test_features);

            let metric = create_metric(metric_name);

            let train_score = metric.calculate(&Predictions {
                true_values: train_target.clone(),
                predicted: train_preds,
                probabilities: None,
            });

            let test_score = metric.calculate(&Predictions {
                true_values: test_target.clone(),
                predicted: test_preds,
                probabilities: None,
            });

            curve.push(LearningCurvePoint {
                train_size,
                train_score,
                val_score: test_score,
            });
        }

        curve
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Nested cross-validation
    pub fn nested_validate<M: Model>(
        &self,
        dataset: &Dataset,
        inner_cv: &CrossValidationConfig,
        model_factory: &dyn Fn() -> M,
        metric_names: &[String],
    ) -> NestedCVResult {
        // Outer loop
        let outer_folds = self.generate_folds(dataset);
        let mut outer_results = Vec::new();
        let mut outer_val_results = Vec::new();

        for fold in &outer_folds {
            let (train_features, train_target) =
                self.extract_fold_data(dataset, &fold.train_indices);
            let inner_dataset = Dataset {
                features: train_features.clone(),
                target: train_target.clone(),
                sample_weights: dataset
                    .sample_weights
                    .as_ref()
                    .map(|sw| fold.train_indices.iter().map(|&i| sw[i]).collect()),
                groups: dataset
                    .groups
                    .as_ref()
                    .map(|g| fold.train_indices.iter().map(|&i| g[i].clone()).collect()),
                stratify: dataset
                    .stratify
                    .as_ref()
                    .map(|st| fold.train_indices.iter().map(|&i| st[i].clone()).collect()),
            };

            let inner_validator = Self::new(inner_cv.clone());
            let inner_result =
                inner_validator.validate(&inner_dataset, model_factory, metric_names);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            outer_results.push(inner_result);

            // Fit model on outer training fold and evaluate on held-out outer validation fold
            let (val_features, val_target) = self.extract_fold_data(dataset, &fold.val_indices);
            let mut outer_model = model_factory();
            outer_model.fit(&train_features, &train_target);
            let val_preds = outer_model.predict(&val_features);
            let val_predictions = Predictions {
                true_values: val_target.clone(),
                predicted: val_preds,
                probabilities: None,
            };
            let outer_fold_val_metrics = self.calculate_metrics(&val_predictions, metric_names);
            outer_val_results.push(outer_fold_val_metrics);
        }

        // Aggregate unbiased performance
        let unbiased_performance = self.aggregate_nested_results(&outer_val_results, metric_names);

        NestedCVResult {
            outer_results,
            unbiased_performance,
        }
    }

    fn aggregate_nested_results(
        &self,
        fold_metrics: &[HashMap<String, f64>],
        metric_names: &[String],
    ) -> AggregatedMetrics {
        let mut mean = HashMap::new();
        let mut std = HashMap::new();
        let mut min = HashMap::new();
        let mut max = HashMap::new();
        let mut ci_95 = HashMap::new();

        for name in metric_names {
            let values: Vec<f64> = fold_metrics
                .iter()
                .filter_map(|m| m.get(name).copied())
                .filter(|v| v.is_finite())
                .collect();

            if !values.is_empty() {
                let n = values.len();
                let m = values.iter().sum::<f64>() / n as f64;
                let variance = if n > 1 {
                    values.iter().map(|&v| (v - m).powi(2)).sum::<f64>() / (n - 1) as f64
                } else {
                    0.0
                };
                let s = variance.sqrt();

                // TAG: surface=simulation owner=research-team rule=SIM-001
                mean.insert(name.clone(), m);
                std.insert(name.clone(), s);
                min.insert(
                    name.clone(),
                    values.iter().copied().fold(f64::INFINITY, f64::min),
                );
                max.insert(
                    name.clone(),
                    values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                );

                let ci_margin = if n > 0 {
                    1.96 * s / (n as f64).sqrt()
                } else {
                    0.0
                };
                ci_95.insert(name.clone(), (m - ci_margin, m + ci_margin));
            }
        }

        AggregatedMetrics {
            mean,
            std,
            min,
            max,
            ci_95,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for CrossValidator {
    fn default() -> Self {
        Self::new(CrossValidationConfig::default())
    }
}

/// Simple baseline model for testing
#[derive(Debug)]
pub struct BaselineModel {
    mean_target: f64,
}

impl BaselineModel {
    #[must_use]
    /// fn
    pub const fn new() -> Self {
        Self { mean_target: 0.0 }
    }
}

impl Default for BaselineModel {
    fn default() -> Self {
        Self::new()
    }
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
impl Model for BaselineModel {
    fn fit(&mut self, _features: &[Vec<f64>], target: &[f64]) {
        if !target.is_empty() {
            self.mean_target = target.iter().sum::<f64>() / target.len() as f64;
        }
    }

    fn predict(&self, features: &[Vec<f64>]) -> Vec<f64> {
        vec![self.mean_target; features.len()]
    }

    fn name(&self) -> &'static str {
        "Baseline"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dataset(n: usize) -> Dataset {
        Dataset {
            features: (0..n).map(|i| vec![i as f64, (i * 2) as f64]).collect(),
            target: (0..n).map(|i| (i as f64) * 2.0).collect(),
            sample_weights: None,
            groups: None,
            stratify: None,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_cross_validation() {
        let config = CrossValidationConfig {
            method: CrossValidationMethod::KFold,
            n_folds: 3,
            seed: Some(42),
            shuffle: true,
            ..Default::default()
        };

        let validator = CrossValidator::new(config);
        let dataset = create_test_dataset(30);

        let result = validator.validate(&dataset, &|| BaselineModel::new(), &["mse".to_string()]);

        assert_eq!(result.fold_results.len(), 3);
        assert!(result.aggregated_metrics.mean.contains_key("mse"));
    }

    #[test]
    fn test_learning_curve() {
        let config = CrossValidationConfig::default();
        let validator = CrossValidator::new(config);
        let dataset = create_test_dataset(100);

        let curve =
            validator.learning_curve(&dataset, &|| BaselineModel::new(), "mse", &[0.2, 0.5, 0.8]);

        assert_eq!(curve.len(), 3);
        assert!(curve[0].train_size < curve[1].train_size);
    }
}
