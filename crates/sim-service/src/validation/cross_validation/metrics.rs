//! Evaluation metrics for cross-validation
//!
//! Implements common classification and regression metrics.

use super::types::{Metric, Predictions};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Regression metrics
#[derive(Debug)]
pub struct RegressionMetrics;

impl RegressionMetrics {
    /// Mean Squared Error
    #[must_use]
    pub fn mse(predictions: &Predictions) -> f64 {
        let n = predictions
            .true_values
            .len()
            .min(predictions.predicted.len());
        if n == 0 {
            return 0.0;
        }
        let diffs: Vec<f64> = predictions.true_values[..n]
            .iter()
            .zip(predictions.predicted[..n].iter())
            .filter(|(&t, &p)| t.is_finite() && p.is_finite())
            .map(|(&t, &p)| (t - p).powi(2))
            .collect();
        if diffs.is_empty() {
            return 0.0;
        }
        diffs.iter().sum::<f64>() / diffs.len() as f64
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Root Mean Squared Error
    #[must_use]
    pub fn rmse(predictions: &Predictions) -> f64 {
        Self::mse(predictions).sqrt()
    }

    /// Mean Absolute Error
    #[must_use]
    pub fn mae(predictions: &Predictions) -> f64 {
        let n = predictions
            .true_values
            .len()
            .min(predictions.predicted.len());
        if n == 0 {
            return 0.0;
        }
        let diffs: Vec<f64> = predictions.true_values[..n]
            .iter()
            .zip(predictions.predicted[..n].iter())
            .filter(|(&t, &p)| t.is_finite() && p.is_finite())
            .map(|(&t, &p)| (t - p).abs())
            .collect();
        if diffs.is_empty() {
            return 0.0;
        }
        diffs.iter().sum::<f64>() / diffs.len() as f64
    }

    /// R-squared coefficient
    #[must_use]
    pub fn r2(predictions: &Predictions) -> f64 {
        let n = predictions
            .true_values
            .len()
            .min(predictions.predicted.len());
        if n == 0 {
            return 0.0;
        }

        let pairs: Vec<(f64, f64)> = predictions.true_values[..n]
            .iter()
            .zip(predictions.predicted[..n].iter())
            .filter(|(&t, &p)| t.is_finite() && p.is_finite())
            .map(|(&t, &p)| (t, p))
            .collect();

        if pairs.is_empty() {
            return 0.0;
        }

        let n_valid = pairs.len() as f64;
        let mean_true = pairs.iter().map(|(t, _)| *t).sum::<f64>() / n_valid;

        let ss_res: f64 = pairs.iter().map(|(t, p)| (t - p).powi(2)).sum();
        let ss_tot: f64 = pairs.iter().map(|(t, _)| (t - mean_true).powi(2)).sum();

        if ss_tot <= 1e-12 {
            if ss_res <= 1e-12 {
                return 1.0;
            }
            return 0.0;
        }

        (1.0 - (ss_res / ss_tot)).clamp(-1.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Explained variance
    #[must_use]
    pub fn explained_variance(predictions: &Predictions) -> f64 {
        let n = predictions
            .true_values
            .len()
            .min(predictions.predicted.len());
        if n == 0 {
            return 0.0;
        }

        let pairs: Vec<(f64, f64)> = predictions.true_values[..n]
            .iter()
            .zip(predictions.predicted[..n].iter())
            .filter(|(&t, &p)| t.is_finite() && p.is_finite())
            .map(|(&t, &p)| (t, p))
            .collect();

        if pairs.is_empty() {
            return 0.0;
        }

        let n_valid = pairs.len() as f64;
        let mean_true = pairs.iter().map(|(t, _)| *t).sum::<f64>() / n_valid;
        let mean_pred = pairs.iter().map(|(_, p)| *p).sum::<f64>() / n_valid;

        let var_res: f64 = pairs
            .iter()
            .map(|(t, p)| ((t - mean_true) - (p - mean_pred)).powi(2))
            .sum::<f64>()
            / n_valid;

        let var_tot: f64 = pairs
            .iter()
            .map(|(t, _)| (t - mean_true).powi(2))
            .sum::<f64>()
            / n_valid;

        if var_tot <= 1e-12 {
            if var_res <= 1e-12 {
                return 1.0;
            }
            return 0.0;
        }

        (1.0 - (var_res / var_tot)).clamp(0.0, 1.0)
    }

    /// Mean Absolute Percentage Error
    #[must_use]
    pub fn mape(predictions: &Predictions) -> f64 {
        let n = predictions
            .true_values
            .len()
            .min(predictions.predicted.len());
        if n == 0 {
            return 0.0;
        }
        let (sum, valid_count) = predictions.true_values[..n]
            .iter()
            .zip(predictions.predicted[..n].iter())
            .filter(|(&t, &p)| t.is_finite() && p.is_finite() && t.abs() > 1e-10)
            .fold((0.0f64, 0usize), |(acc_sum, count), (&t, &p)| {
                (acc_sum + ((t - p) / t).abs(), count + 1)
            });

        if valid_count == 0 {
            0.0
        } else {
            (sum / valid_count as f64) * 100.0
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Classification metrics
#[derive(Debug)]
pub struct ClassificationMetrics;

impl ClassificationMetrics {
    /// Accuracy
    #[must_use]
    pub fn accuracy(predictions: &Predictions) -> f64 {
        let n = predictions
            .true_values
            .len()
            .min(predictions.predicted.len());
        if n == 0 {
            return 0.0;
        }
        let valid_pairs: Vec<(f64, f64)> = predictions.true_values[..n]
            .iter()
            .zip(predictions.predicted[..n].iter())
            .filter(|(&t, &p)| t.is_finite() && p.is_finite())
            .map(|(&t, &p)| (t, p))
            .collect();

        if valid_pairs.is_empty() {
            return 0.0;
        }

        let correct: usize = valid_pairs
            .iter()
            .filter(|(t, p)| (*t >= 0.5) == (*p >= 0.5))
            .count();

        correct as f64 / valid_pairs.len() as f64
    }

    /// Precision (binary)
    #[must_use]
    pub fn precision(predictions: &Predictions) -> f64 {
        let n = predictions
            .true_values
            .len()
            .min(predictions.predicted.len());
        if n == 0 {
            return 0.0;
        }
        let valid_pairs: Vec<(f64, f64)> = predictions.true_values[..n]
            .iter()
            .zip(predictions.predicted[..n].iter())
            .filter(|(&t, &p)| t.is_finite() && p.is_finite())
            .map(|(&t, &p)| (t, p))
            .collect();

        if valid_pairs.is_empty() {
            return 0.0;
        }

        let true_positives: usize = valid_pairs
            .iter()
            .filter(|(t, p)| *t >= 0.5 && *p >= 0.5)
            .count();

        let predicted_positives: usize = valid_pairs.iter().filter(|(_, p)| *p >= 0.5).count();

        if predicted_positives == 0 {
            0.0
        } else {
            true_positives as f64 / predicted_positives as f64
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Recall (binary)
    #[must_use]
    pub fn recall(predictions: &Predictions) -> f64 {
        let n = predictions
            .true_values
            .len()
            .min(predictions.predicted.len());
        if n == 0 {
            return 0.0;
        }
        let valid_pairs: Vec<(f64, f64)> = predictions.true_values[..n]
            .iter()
            .zip(predictions.predicted[..n].iter())
            .filter(|(&t, &p)| t.is_finite() && p.is_finite())
            .map(|(&t, &p)| (t, p))
            .collect();

        if valid_pairs.is_empty() {
            return 0.0;
        }

        let true_positives: usize = valid_pairs
            .iter()
            .filter(|(t, p)| *t >= 0.5 && *p >= 0.5)
            .count();

        let actual_positives: usize = valid_pairs.iter().filter(|(t, _)| *t >= 0.5).count();

        if actual_positives == 0 {
            0.0
        } else {
            true_positives as f64 / actual_positives as f64
        }
    }

    /// F1 score
    #[must_use]
    pub fn f1(predictions: &Predictions) -> f64 {
        let precision = Self::precision(predictions);
        let recall = Self::recall(predictions);

        if precision + recall < 1e-10 {
            0.0
        } else {
            2.0 * (precision * recall) / (precision + recall)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Area Under ROC Curve (approximation)
    #[must_use]
    pub fn roc_auc(predictions: &Predictions) -> f64 {
        let n = predictions
            .true_values
            .len()
            .min(predictions.predicted.len());
        if n == 0 {
            return 0.5;
        }

        // Extract binary labels and scores (using probabilities if available, else predicted)
        let mut scores: Vec<(f64, bool)> = Vec::with_capacity(n);
        for i in 0..n {
            let label = predictions.true_values[i] >= 0.5;
            let score = if let Some(probs) = &predictions.probabilities {
                if i < probs.len() {
                    if probs[i].len() > 1 {
                        probs[i][1]
                    } else if !probs[i].is_empty() {
                        probs[i][0]
                    } else {
                        predictions.predicted[i]
                    }
                } else {
                    predictions.predicted[i]
                }
            } else {
                predictions.predicted[i]
            };
            if score.is_finite() {
                scores.push((score, label));
            }
        }

        let n_valid = scores.len();
        if n_valid == 0 {
            return 0.5;
        }

        let n_pos = scores.iter().filter(|(_, label)| *label).count() as f64;
        let n_neg = n_valid as f64 - n_pos;

        if n_pos == 0.0 || n_neg == 0.0 {
            return 0.5;
        }

        // Sort ascending by score
        scores.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Exact O(N log N) rank-sum with mid-ranks for ties
        let mut sum_ranks_pos = 0.0f64;
        let mut i = 0;
        while i < n_valid {
            let mut j = i + 1;
            while j < n_valid && (scores[j].0 - scores[i].0).abs() <= 1e-12 {
                j += 1;
            }
            let mid_rank = (i + 1 + j) as f64 / 2.0;
            for item in &scores[i..j] {
                if item.1 {
                    sum_ranks_pos += mid_rank;
                }
            }
            i = j;
        }

        let u1 = sum_ranks_pos - (n_pos * (n_pos + 1.0)) / 2.0;
        (u1 / (n_pos * n_neg)).clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Log loss (uses probabilities if available, falls back to predicted)
    #[must_use]
    pub fn log_loss(predictions: &Predictions) -> f64 {
        let n = predictions
            .true_values
            .len()
            .min(predictions.predicted.len());
        if n == 0 {
            return 0.0;
        }
        let eps = 1e-15;

        let mut sum = 0.0f64;
        let mut count = 0usize;

        for i in 0..n {
            let t = predictions.true_values[i];
            if !t.is_finite() {
                continue;
            }
            let p = if let Some(probs) = &predictions.probabilities {
                if i < probs.len() {
                    if probs[i].len() > 1 {
                        probs[i][1]
                    } else if !probs[i].is_empty() {
                        probs[i][0]
                    } else {
                        predictions.predicted[i]
                    }
                } else {
                    predictions.predicted[i]
                }
            } else {
                predictions.predicted[i]
            };

            if !p.is_finite() {
                continue;
            }

            let prob_class_1 = p.clamp(eps, 1.0 - eps);
            let y = if t >= 0.5 { 1.0 } else { 0.0 };
            sum += -(y * prob_class_1.ln() + (1.0 - y) * (1.0 - prob_class_1).ln());
            count += 1;
        }

        if count == 0 {
            0.0
        } else {
            sum / count as f64
        }
    }
}

/// Create metric from name
#[must_use]
pub fn create_metric(name: &str) -> Box<dyn Metric> {
    match name {
        "mse" => Box::new(MSE),
        "rmse" => Box::new(RMSE),
        "mae" => Box::new(MAE),
        "r2" => Box::new(R2),
        "mape" => Box::new(MAPE),
        "explained_variance" => Box::new(ExplainedVariance),
        "accuracy" => Box::new(Accuracy),
        "precision" => Box::new(Precision),
        "recall" => Box::new(Recall),
        "f1" => Box::new(F1),
        "roc_auc" => Box::new(RocAuc),
        "log_loss" => Box::new(LogLoss),
        _ => Box::new(MSE),
    }
}

// Metric structs for trait implementation
// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug)]
/// MSE data
pub struct MSE;
impl Metric for MSE {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        RegressionMetrics::mse(predictions)
    }
    fn name(&self) -> &'static str {
        "mse"
    }
    fn higher_is_better(&self) -> bool {
        false
    }
}

#[derive(Debug)]
/// RMSE data
pub struct RMSE;
impl Metric for RMSE {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        RegressionMetrics::rmse(predictions)
    }
    fn name(&self) -> &'static str {
        "rmse"
    }
    fn higher_is_better(&self) -> bool {
        false
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug)]
/// MAE data
pub struct MAE;
impl Metric for MAE {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        RegressionMetrics::mae(predictions)
    }
    fn name(&self) -> &'static str {
        "mae"
    }
    fn higher_is_better(&self) -> bool {
        false
    }
}

#[derive(Debug)]
/// R2 data
pub struct R2;
impl Metric for R2 {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        RegressionMetrics::r2(predictions)
    }
    fn name(&self) -> &'static str {
        "r2"
    }
    fn higher_is_better(&self) -> bool {
        true
    }
}

#[derive(Debug)]
/// MAPE data
pub struct MAPE;
impl Metric for MAPE {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        RegressionMetrics::mape(predictions)
    }
    fn name(&self) -> &'static str {
        "mape"
    }
    fn higher_is_better(&self) -> bool {
        false
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug)]
/// Accuracy data
pub struct Accuracy;
impl Metric for Accuracy {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        ClassificationMetrics::accuracy(predictions)
    }
    fn name(&self) -> &'static str {
        "accuracy"
    }
    fn higher_is_better(&self) -> bool {
        true
    }
}

#[derive(Debug)]
/// Precision data
pub struct Precision;
impl Metric for Precision {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        ClassificationMetrics::precision(predictions)
    }
    fn name(&self) -> &'static str {
        "precision"
    }
    fn higher_is_better(&self) -> bool {
        true
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug)]
/// Recall data
pub struct Recall;
impl Metric for Recall {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        ClassificationMetrics::recall(predictions)
    }
    fn name(&self) -> &'static str {
        "recall"
    }
    fn higher_is_better(&self) -> bool {
        true
    }
}

#[derive(Debug)]
/// F1 data
pub struct F1;
impl Metric for F1 {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        ClassificationMetrics::f1(predictions)
    }
    fn name(&self) -> &'static str {
        "f1"
    }
    fn higher_is_better(&self) -> bool {
        true
    }
}

#[derive(Debug)]
/// `RocAuc` data
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
pub struct RocAuc;
impl Metric for RocAuc {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        ClassificationMetrics::roc_auc(predictions)
    }
    fn name(&self) -> &'static str {
        "roc_auc"
    }
    fn higher_is_better(&self) -> bool {
        true
    }
}

#[derive(Debug)]
/// `ExplainedVariance` data
pub struct ExplainedVariance;
impl Metric for ExplainedVariance {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        RegressionMetrics::explained_variance(predictions)
    }
    fn name(&self) -> &'static str {
        "explained_variance"
    }
    fn higher_is_better(&self) -> bool {
        true
    }
}

#[derive(Debug)]
/// `LogLoss` data
pub struct LogLoss;
impl Metric for LogLoss {
    fn calculate(&self, predictions: &Predictions) -> f64 {
        ClassificationMetrics::log_loss(predictions)
    }
    fn name(&self) -> &'static str {
        "log_loss"
    }
    fn higher_is_better(&self) -> bool {
        false
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn create_test_predictions() -> Predictions {
        Predictions {
            true_values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            predicted: vec![1.1, 1.9, 3.2, 3.8, 5.1],
            probabilities: None,
        }
    }

    fn create_binary_predictions() -> Predictions {
        Predictions {
            true_values: vec![0.0, 0.0, 1.0, 1.0, 1.0],
            predicted: vec![0.1, 0.4, 0.6, 0.8, 0.9],
            probabilities: None,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_mse() {
        let preds = create_test_predictions();
        let mse = RegressionMetrics::mse(&preds);
        assert!(mse > 0.0 && mse < 0.1);
    }

    #[test]
    fn test_rmse() {
        let preds = create_test_predictions();
        let rmse = RegressionMetrics::rmse(&preds);
        assert!(rmse > 0.0 && rmse < 0.3);
    }

    #[test]
    fn test_mae() {
        let preds = create_test_predictions();
        let mae = RegressionMetrics::mae(&preds);
        assert!(mae > 0.0 && mae < 0.3);
    }

    #[test]
    fn test_r2() {
        let preds = create_test_predictions();
        let r2 = RegressionMetrics::r2(&preds);
        assert!(r2 > 0.95 && r2 <= 1.0);
    }

    #[test]
    fn test_accuracy() {
        let preds = create_binary_predictions();
        let acc = ClassificationMetrics::accuracy(&preds);
        // All 5 should be correct (differences: 0.1, 0.4, 0.4, 0.2, 0.1 all < 0.5)
        assert_eq!(acc, 1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_precision() {
        let preds = create_binary_predictions();
        let precision = ClassificationMetrics::precision(&preds);
        // 3 predicted positive, all 3 are correct
        assert_eq!(precision, 1.0);
    }

    #[test]
    fn test_recall() {
        let preds = create_binary_predictions();
        let recall = ClassificationMetrics::recall(&preds);
        // 3 actual positive, all 3 found
        assert_eq!(recall, 1.0);
    }

    #[test]
    fn test_f1() {
        let preds = create_binary_predictions();
        let f1 = ClassificationMetrics::f1(&preds);
        assert_eq!(f1, 1.0);
    }

    #[test]
    fn test_log_loss_fallback_when_probabilities_none() {
        let preds = create_binary_predictions();
        let loss = ClassificationMetrics::log_loss(&preds);
        assert!(
            loss > 0.0 && loss.is_finite(),
            "Loss must be positive cross-entropy"
        );
    }

    #[test]
    fn test_metric_trait() {
        let preds = create_test_predictions();
        let mse = create_metric("mse");
        assert_eq!(mse.name(), "mse");
        assert!(!mse.higher_is_better());
        assert!(mse.calculate(&preds) > 0.0);
    }

    #[test]
    fn test_r2_constant_target_exact() {
        let preds = Predictions {
            true_values: vec![42.0, 42.0, 42.0, 42.0],
            predicted: vec![42.0, 42.0, 42.0, 42.0],
            probabilities: None,
        };
        let r2 = RegressionMetrics::r2(&preds);
        assert_eq!(
            r2, 1.0,
            "Exact prediction on constant ground truth must yield R² = 1.0"
        );

        let ev = RegressionMetrics::explained_variance(&preds);
        assert_eq!(
            ev, 1.0,
            "Exact prediction on constant ground truth must yield EV = 1.0"
        );
    }

    #[test]
    fn test_r2_constant_target_divergent() {
        let preds = Predictions {
            true_values: vec![42.0, 42.0, 42.0, 42.0],
            predicted: vec![41.0, 43.0, 40.0, 44.0],
            probabilities: None,
        };
        let r2 = RegressionMetrics::r2(&preds);
        assert_eq!(
            r2, 0.0,
            "Divergent prediction on constant ground truth must yield R² = 0.0"
        );

        let ev = RegressionMetrics::explained_variance(&preds);
        assert_eq!(
            ev, 0.0,
            "Divergent prediction on constant ground truth must yield EV = 0.0"
        );
    }

    #[test]
    fn test_metrics_filter_nan_gracefully() {
        let preds = Predictions {
            true_values: vec![f64::NAN, 1.0, 0.0],
            predicted: vec![f64::NAN, 0.9, 0.1],
            probabilities: None,
        };
        let acc = ClassificationMetrics::accuracy(&preds);
        assert!((acc - 1.0).abs() < 1e-6);

        let mse = RegressionMetrics::mse(&preds);
        assert!(mse < 0.1);

        let all_nan = Predictions {
            true_values: vec![f64::NAN, f64::NAN],
            predicted: vec![f64::NAN, f64::NAN],
            probabilities: None,
        };
        assert_eq!(ClassificationMetrics::accuracy(&all_nan), 0.0);
        assert_eq!(RegressionMetrics::mse(&all_nan), 0.0);
    }
}
