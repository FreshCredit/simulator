//! Types for cross-validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cross-validation method
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CrossValidationMethod {
    /// Standard k-fold
    KFold,
    /// Stratified k-fold (preserves class distribution)
    StratifiedKFold,
    /// Time series split (respects temporal order)
    TimeSeriesSplit,
    /// Group k-fold (keeps groups together)
    GroupKFold,
    /// Shuffle split (random train/test splits)
    ShuffleSplit,
    /// Leave-one-out
    LeaveOneOut,
    /// Leave-p-out
    LeavePOut,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for cross-validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossValidationConfig {
    /// CV method
    pub method: CrossValidationMethod,
    /// Number of folds
    pub n_folds: usize,
    /// Random seed
    pub seed: Option<u64>,
    /// Shuffle data before splitting
    pub shuffle: bool,
    /// Stratification column (for classification)
    pub stratify_column: Option<String>,
    /// Group column (for group k-fold)
    pub group_column: Option<String>,
    /// Test size for shuffle split
    pub test_size: Option<f64>,
    /// Time column (for time series)
    pub time_column: Option<String>,
}

impl Default for CrossValidationConfig {
    fn default() -> Self {
        Self {
            method: CrossValidationMethod::KFold,
            n_folds: 5,
            seed: None,
            shuffle: true,
            stratify_column: None,
            group_column: None,
            test_size: None,
            time_column: None,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Dataset for cross-validation
#[derive(Debug, Clone)]
pub struct Dataset {
    /// Features (row-oriented)
    pub features: Vec<Vec<f64>>,
    /// Target values
    pub target: Vec<f64>,
    /// Sample weights (optional)
    pub sample_weights: Option<Vec<f64>>,
    /// Group labels (optional)
    pub groups: Option<Vec<String>>,
    /// Stratification labels (optional)
    pub stratify: Option<Vec<String>>,
}

/// A single fold in cross-validation
#[derive(Debug, Clone)]
pub struct Fold {
    /// Fold index
    pub index: usize,
    /// Training indices
    pub train_indices: Vec<usize>,
    /// Validation indices
    pub val_indices: Vec<usize>,
}

/// Cross-validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossValidationResult {
    /// Configuration used
    pub config: CrossValidationConfig,
    /// Results for each fold
    pub fold_results: Vec<FoldResult>,
    /// Aggregated metrics
    pub aggregated_metrics: AggregatedMetrics,
    /// Total computation time (seconds)
    pub computation_time_secs: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Result for a single fold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldResult {
    /// Fold number
    pub fold: usize,
    /// Training set size
    pub train_size: usize,
    /// Validation set size
    pub val_size: usize,
    /// Metrics on training set
    pub train_metrics: HashMap<String, f64>,
    /// Metrics on validation set
    pub val_metrics: HashMap<String, f64>,
    /// Time to fit model
    pub fit_time_secs: f64,
    /// Time to predict
    pub predict_time_secs: f64,
}

/// Aggregated metrics across folds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Mean metrics
    pub mean: HashMap<String, f64>,
    /// Standard deviation
    pub std: HashMap<String, f64>,
    /// Min values
    pub min: HashMap<String, f64>,
    /// Max values
    pub max: HashMap<String, f64>,
    /// 95% confidence intervals
    pub ci_95: HashMap<String, (f64, f64)>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Prediction result
#[derive(Debug, Clone)]
pub struct Predictions {
    /// True values
    pub true_values: Vec<f64>,
    /// Predicted values
    pub predicted: Vec<f64>,
    /// Predicted probabilities (for classification)
    pub probabilities: Option<Vec<Vec<f64>>>,
}

/// Model trait for cross-validation
pub trait Model: Send + Sync {
    /// Fit the model on training data
    fn fit(&mut self, features: &[Vec<f64>], target: &[f64]);
    /// Make predictions
    fn predict(&self, features: &[Vec<f64>]) -> Vec<f64>;
    /// Get model name
    fn name(&self) -> &str;
}

/// Metric trait
pub trait Metric: Send + Sync {
    /// Calculate metric from predictions
    fn calculate(&self, predictions: &Predictions) -> f64;
    /// Get metric name
    fn name(&self) -> &str;
    /// Whether higher is better
    fn higher_is_better(&self) -> bool;
}

/// Cross-validation request for API
#[derive(Debug, Deserialize)]
pub struct CrossValidationRequest {
    /// config
    pub config: CrossValidationConfig,
    /// experiment id
    pub experiment_id: Option<String>,
    /// metrics
    pub metrics: Vec<String>,
    /// model type
    pub model_type: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cross-validation response
#[derive(Debug, Serialize)]
pub struct CrossValidationResponse {
    /// cv id
    pub cv_id: String,
    /// status
    pub status: String,
    /// estimated duration secs
    pub estimated_duration_secs: u64,
    /// n folds
    pub n_folds: usize,
}

/// Detailed CV result response
#[derive(Debug, Serialize)]
pub struct CrossValidationDetailResponse {
    /// cv id
    pub cv_id: String,
    /// result
    pub result: CrossValidationResult,
    /// learning curves
    pub learning_curves: Vec<LearningCurvePoint>,
}

/// Learning curve data point
#[derive(Debug, Serialize)]
pub struct LearningCurvePoint {
    /// train size
    pub train_size: usize,
    /// train score
    pub train_score: f64,
    /// val score
    pub val_score: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Model comparison request
#[derive(Debug, Deserialize)]
pub struct ModelComparisonRequest {
    /// config
    pub config: CrossValidationConfig,
    /// models
    pub models: Vec<String>,
    /// dataset id
    pub dataset_id: String,
}

#[derive(Debug, Serialize)]
/// `ModelComparisonResponse` data
pub struct ModelComparisonResponse {
    /// results
    pub results: HashMap<String, CrossValidationResult>,
    /// rankings
    pub rankings: Vec<ModelRanking>,
    /// best model
    pub best_model: String,
}

/// Model ranking
#[derive(Debug, Serialize)]
pub struct ModelRanking {
    /// model
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub model: String,
    /// rank
    pub rank: usize,
    /// mean score
    pub mean_score: f64,
    /// std score
    pub std_score: f64,
}

/// Nested cross-validation config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedCVConfig {
    /// Outer loop config
    pub outer_cv: CrossValidationConfig,
    /// Inner loop config (for hyperparameter tuning)
    pub inner_cv: CrossValidationConfig,
}

/// Nested CV result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedCVResult {
    /// outer results
    pub outer_results: Vec<CrossValidationResult>,
    /// unbiased performance
    pub unbiased_performance: AggregatedMetrics,
}
