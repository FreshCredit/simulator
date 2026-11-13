//! Cross-Validation API Module (P1-VAL-3)
//!
//! REST endpoints for k-fold cross-validation.

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::validation::cross_validation::{
    fold::{BaselineModel, CrossValidator},
    metrics::create_metric,
    types::{CrossValidationConfig, CrossValidationResult, Dataset, LearningCurvePoint},
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Run cross-validation
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn run_cross_validation(
    Json(request): Json<RunCrossValidationRequest>,
) -> Result<Json<CrossValidationResult>, StatusCode> {
    let validator = CrossValidator::new(request.config.clone());

    // For demo, use simple dataset
    let dataset = Dataset {
        features: request.features,
        target: request.target,
        sample_weights: None,
        groups: request.groups,
        stratify: request.stratify,
    };

    let result = validator.validate(&dataset, &|| BaselineModel::new(), &request.metrics);

    Ok(Json(result))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get available CV methods
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_cv_methods() -> Result<Json<CvMethodsResponse>, StatusCode> {
    let methods = vec![
        CvMethodInfo {
            id: "kfold".to_string(),
            name: "K-Fold".to_string(),
            description: "Standard k-fold cross-validation".to_string(),
            use_case: "General purpose, balanced data".to_string(),
        },
        CvMethodInfo {
            id: "stratified".to_string(),
            name: "Stratified K-Fold".to_string(),
            description: "Preserves class distribution".to_string(),
            use_case: "Classification with imbalanced classes".to_string(),
        },
        CvMethodInfo {
            id: "group".to_string(),
            name: "Group K-Fold".to_string(),
            description: "Keeps groups together".to_string(),
            use_case: "Grouped data (e.g., multiple samples per subject)".to_string(),
        },
        CvMethodInfo {
            id: "timeseries".to_string(),
            name: "Time Series Split".to_string(),
            description: "Respects temporal order".to_string(),
            use_case: "Time series forecasting".to_string(),
        },
        CvMethodInfo {
            id: "shuffle".to_string(),
            name: "Shuffle Split".to_string(),
            description: "Random train/test splits".to_string(),
            use_case: "Large datasets, quick evaluation".to_string(),
        },
        CvMethodInfo {
            id: "loo".to_string(),
            name: "Leave-One-Out".to_string(),
            description: "Each sample used once for validation".to_string(),
            use_case: "Small datasets".to_string(),
        },
    ];

    Ok(Json(CvMethodsResponse { methods }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get available metrics
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_metrics() -> Result<Json<MetricsResponse>, StatusCode> {
    let metrics = vec![
        MetricInfo {
            id: "mse".to_string(),
            name: "Mean Squared Error".to_string(),
            category: "regression".to_string(),
            higher_is_better: false,
        },
        MetricInfo {
            id: "rmse".to_string(),
            name: "Root Mean Squared Error".to_string(),
            category: "regression".to_string(),
            higher_is_better: false,
        },
        MetricInfo {
            id: "mae".to_string(),
            name: "Mean Absolute Error".to_string(),
            category: "regression".to_string(),
            higher_is_better: false,
        },
        MetricInfo {
            id: "r2".to_string(),
            name: "R-squared".to_string(),
            category: "regression".to_string(),
            higher_is_better: true,
        },
        MetricInfo {
            id: "mape".to_string(),
            name: "Mean Absolute Percentage Error".to_string(),
            category: "regression".to_string(),
            higher_is_better: false,
        },
        MetricInfo {
            id: "explained_variance".to_string(),
            name: "Explained Variance".to_string(),
            category: "regression".to_string(),
            higher_is_better: true,
        },
        // TAG: surface=simulation owner=research-team rule=SIM-001
        MetricInfo {
            id: "accuracy".to_string(),
            name: "Accuracy".to_string(),
            category: "classification".to_string(),
            higher_is_better: true,
        },
        MetricInfo {
            id: "precision".to_string(),
            name: "Precision".to_string(),
            category: "classification".to_string(),
            higher_is_better: true,
        },
        MetricInfo {
            id: "recall".to_string(),
            name: "Recall".to_string(),
            category: "classification".to_string(),
            higher_is_better: true,
        },
        MetricInfo {
            id: "f1".to_string(),
            name: "F1 Score".to_string(),
            category: "classification".to_string(),
            higher_is_better: true,
        },
        MetricInfo {
            id: "roc_auc".to_string(),
            name: "ROC AUC".to_string(),
            category: "classification".to_string(),
            higher_is_better: true,
        },
        MetricInfo {
            id: "log_loss".to_string(),
            name: "Log Loss".to_string(),
            category: "classification".to_string(),
            higher_is_better: false,
        },
    ];

    Ok(Json(MetricsResponse { metrics }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate learning curve
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn learning_curve(
    Json(request): Json<LearningCurveRequest>,
) -> Result<Json<LearningCurveResponse>, StatusCode> {
    let config = CrossValidationConfig::default();
    let validator = CrossValidator::new(config);

    let dataset = Dataset {
        features: request.features,
        target: request.target,
        sample_weights: None,
        groups: None,
        stratify: None,
    };

    let curve = validator.learning_curve(
        &dataset,
        &|| BaselineModel::new(),
        &request.metric,
        &request.train_sizes,
    );

    Ok(Json(LearningCurveResponse {
        curve,
        metric: request.metric,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compare models using CV
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn compare_models(
    Json(request): Json<CompareModelsRequest>,
) -> Result<Json<ModelComparisonResult>, StatusCode> {
    if request.models.is_empty() || request.metrics.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut results = HashMap::new();

    // For demo, just run baseline multiple times with different configs
    for model_name in &request.models {
        let validator = CrossValidator::new(request.config.clone());

        let dataset = Dataset {
            features: request.features.clone(),
            target: request.target.clone(),
            sample_weights: None,
            groups: None,
            stratify: None,
        };

        let result = validator.validate(&dataset, &|| BaselineModel::new(), &request.metrics);
        results.insert(model_name.clone(), result);
    }

    // Rank models by primary metric
    let primary_metric = request.metrics[0].clone();
    let higher_is_better = create_metric(&primary_metric).higher_is_better();

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let mut rankings: Vec<_> = results
        .iter()
        .map(|(name, result)| {
            let mean_score = *result
                .aggregated_metrics
                .mean
                .get(&primary_metric)
                .unwrap_or(&0.0);
            let std_score = *result
                .aggregated_metrics
                .std
                .get(&primary_metric)
                .unwrap_or(&0.0);
            (name.clone(), mean_score, std_score)
        })
        .collect();

    // Sort by score (descending if higher is better) with deterministic name tie-breaker
    rankings.sort_by(|a, b| {
        let cmp = if higher_is_better {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        };
        cmp.then_with(|| a.0.cmp(&b.0))
    });

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let best_model = rankings.first().map(|r| r.0.clone()).unwrap_or_default();

    let rankings: Vec<ModelRankingInfo> = rankings
        .into_iter()
        .enumerate()
        .map(|(i, (model, mean, std))| ModelRankingInfo {
            model,
            rank: i + 1,
            mean_score: mean,
            std_score: std,
        })
        .collect();

    Ok(Json(ModelComparisonResult {
        results,
        rankings,
        best_model,
        primary_metric,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get fold statistics
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn fold_statistics(
    Json(request): Json<FoldStatsRequest>,
) -> Result<Json<FoldStatsResponse>, StatusCode> {
    // Generate folds and return statistics
    let dataset = Dataset {
        features: request.features,
        target: request.target,
        sample_weights: None,
        groups: request.groups,
        stratify: request.stratify,
    };

    let validator = CrossValidator::new(request.config);
    let folds = validator.generate_folds(&dataset);

    let fold_stats: Vec<FoldStats> = folds
        .iter()
        .map(|fold| {
            let train_mean = fold
                .train_indices
                .iter()
                .map(|&i| dataset.target[i])
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
                .sum::<f64>()
                / fold.train_indices.len() as f64;
            let val_mean = fold
                .val_indices
                .iter()
                .map(|&i| dataset.target[i])
                .sum::<f64>()
                / fold.val_indices.len() as f64;

            FoldStats {
                fold_index: fold.index,
                train_size: fold.train_indices.len(),
                val_size: fold.val_indices.len(),
                train_target_mean: train_mean,
                val_target_mean: val_mean,
            }
        })
        .collect();

    Ok(Json(FoldStatsResponse {
        n_folds: folds.len(),
        total_samples: dataset.target.len(),
        fold_stats,
    }))
}

// Request/Response types

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `RunCrossValidationRequest` data
pub struct RunCrossValidationRequest {
    /// config
    pub config: CrossValidationConfig,
    /// features
    pub features: Vec<Vec<f64>>,
    /// target
    pub target: Vec<f64>,
    /// metrics
    pub metrics: Vec<String>,
    /// groups
    pub groups: Option<Vec<String>>,
    /// stratify
    pub stratify: Option<Vec<String>>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `CvMethodInfo` data
pub struct CvMethodInfo {
    /// id
    pub id: String,
    /// name
    pub name: String,
    /// description
    pub description: String,
    /// Re-export
    pub use_case: String,
}

#[derive(Debug, Serialize)]
/// `CvMethodsResponse` data
pub struct CvMethodsResponse {
    /// methods
    pub methods: Vec<CvMethodInfo>,
}

#[derive(Debug, Serialize)]
/// `MetricInfo` data
pub struct MetricInfo {
    /// id
    pub id: String,
    /// name
    pub name: String,
    /// category
    pub category: String,
    /// higher is better
    pub higher_is_better: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `MetricsResponse` data
pub struct MetricsResponse {
    /// metrics
    pub metrics: Vec<MetricInfo>,
}

#[derive(Debug, Deserialize)]
/// `LearningCurveRequest` data
pub struct LearningCurveRequest {
    /// features
    pub features: Vec<Vec<f64>>,
    /// target
    pub target: Vec<f64>,
    /// metric
    pub metric: String,
    /// train sizes
    pub train_sizes: Vec<f64>,
}

#[derive(Debug, Serialize)]
/// `LearningCurveResponse` data
pub struct LearningCurveResponse {
    /// curve
    pub curve: Vec<LearningCurvePoint>,
    /// metric
    pub metric: String,
}

#[derive(Debug, Deserialize)]
/// `CompareModelsRequest` data
pub struct CompareModelsRequest {
    /// config
    pub config: CrossValidationConfig,
    /// models
    pub models: Vec<String>,
    /// features
    pub features: Vec<Vec<f64>>,
    /// target
    pub target: Vec<f64>,
    /// metrics
    pub metrics: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `ModelRankingInfo` data
pub struct ModelRankingInfo {
    /// model
    pub model: String,
    /// rank
    pub rank: usize,
    /// mean score
    pub mean_score: f64,
    /// std score
    pub std_score: f64,
}

#[derive(Debug, Serialize)]
/// `ModelComparisonResult` data
pub struct ModelComparisonResult {
    /// results
    pub results: HashMap<String, CrossValidationResult>,
    /// rankings
    pub rankings: Vec<ModelRankingInfo>,
    /// best model
    pub best_model: String,
    /// primary metric
    pub primary_metric: String,
}

#[derive(Debug, Deserialize)]
/// `FoldStatsRequest` data
pub struct FoldStatsRequest {
    /// config
    pub config: CrossValidationConfig,
    /// features
    pub features: Vec<Vec<f64>>,
    /// target
    pub target: Vec<f64>,
    /// groups
    pub groups: Option<Vec<String>>,
    /// stratify
    pub stratify: Option<Vec<String>>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `FoldStats` data
pub struct FoldStats {
    /// fold index
    pub fold_index: usize,
    /// train size
    pub train_size: usize,
    /// val size
    pub val_size: usize,
    /// train target mean
    pub train_target_mean: f64,
    /// val target mean
    pub val_target_mean: f64,
}

#[derive(Debug, Serialize)]
/// `FoldStatsResponse` data
pub struct FoldStatsResponse {
    /// n folds
    pub n_folds: usize,
    /// total samples
    pub total_samples: usize,
    /// fold stats
    pub fold_stats: Vec<FoldStats>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_cv_methods() {
        let response = list_cv_methods().await.unwrap();
        assert_eq!(response.methods.len(), 6);
        assert_eq!(response.methods[0].id, "kfold");
    }

    #[tokio::test]
    async fn test_list_metrics() {
        let response = list_metrics().await.unwrap();
        assert!(!response.metrics.is_empty());
        assert_eq!(response.metrics[0].id, "mse");
        assert!(response
            .metrics
            .iter()
            .any(|m| m.id == "explained_variance"));
        assert!(response.metrics.iter().any(|m| m.id == "log_loss"));
    }

    #[tokio::test]
    async fn test_compare_models_empty_request_guards() {
        // Empty models
        let req_empty_models = CompareModelsRequest {
            config: CrossValidationConfig::default(),
            models: vec![],
            features: vec![vec![1.0], vec![2.0]],
            target: vec![1.0, 2.0],
            metrics: vec!["mse".to_string()],
        };
        let err1 = compare_models(Json(req_empty_models)).await.unwrap_err();
        assert_eq!(err1, StatusCode::BAD_REQUEST);

        // Empty metrics
        let req_empty_metrics = CompareModelsRequest {
            config: CrossValidationConfig::default(),
            models: vec!["baseline".to_string()],
            features: vec![vec![1.0], vec![2.0]],
            target: vec![1.0, 2.0],
            metrics: vec![],
        };
        let err2 = compare_models(Json(req_empty_metrics)).await.unwrap_err();
        assert_eq!(err2, StatusCode::BAD_REQUEST);
    }
}
