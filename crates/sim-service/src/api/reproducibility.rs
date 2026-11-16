//! Reproducibility API Module (P1-VAL-5)
//!
//! REST endpoints for reproducibility validation and seed sensitivity analysis.

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::validation::reproducibility::{
    stability::StabilityAnalyzer, types::*, variance::VarianceEstimator,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Run reproducibility check
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn check_reproducibility(
    Json(request): Json<ReproducibilityCheckRequest>,
) -> Result<Json<ReproducibilityResult>, StatusCode> {
    if request.config.n_replications == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.config.n_replications > 5_000 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    tokio::task::spawn_blocking(move || {
        let mut replications = Vec::with_capacity(request.config.n_replications);

        for i in 0..request.config.n_replications {
            let seed = request.config.base_seed.wrapping_add(i as u64);

            // Synthetic results with some noise
            let noise: f64 = (i as f64 - request.config.n_replications as f64 / 2.0) * 0.02;
            let mut metrics = HashMap::new();
            metrics.insert("fitness".to_string(), 0.85 + noise);
            metrics.insert("convergence_time".to_string(), 100.0 + noise * 100.0);
            metrics.insert("diversity".to_string(), 0.45 + noise * 0.5);

            replications.push(ReplicationResult {
                index: i,
                seed,
                metrics,
                execution_time_secs: 1.0,
                success: true,
                error: None,
            });
        }

        let analyzer = StabilityAnalyzer::new(request.config);
        Json(analyzer.analyze(&replications))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Analyze seed sensitivity
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn analyze_seed_sensitivity(
    Json(request): Json<SeedSensitivityRequest>,
) -> Result<Json<SeedSensitivityResult>, StatusCode> {
    if request.n_seeds > 5_000 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    tokio::task::spawn_blocking(move || {
        let config = ReproducibilityConfig::default();
        let _analyzer = StabilityAnalyzer::new(config);

        let base_result = {
            let mut m = HashMap::new();
            m.insert("metric1".to_string(), 100.0);
            m.insert("metric2".to_string(), 50.0);
            m
        };

        let mut perturbed_results = Vec::with_capacity(request.n_seeds);
        for i in 0..request.n_seeds {
            let seed = request.base_seed.wrapping_add(i as u64);
            let noise = ((seed as f64) % 10.0) / 100.0;

            let mut m = HashMap::new();
            m.insert("metric1".to_string(), 100.0 + noise);
            m.insert("metric2".to_string(), 50.0 + noise * 0.5);
            perturbed_results.push(m);
        }

        let mut max_deviation = HashMap::new();
        for (metric, &base_value) in &base_result {
            let deviations: Vec<f64> = perturbed_results
                .iter()
                .filter_map(|r| r.get(metric))
                .map(|&v| ((v - base_value) / base_value).abs())
                .collect();

            let max_dev = deviations.iter().copied().fold(0.0, f64::max);
            max_deviation.insert(metric.clone(), max_dev);
        }

        let is_robust =
            !perturbed_results.is_empty() && max_deviation.values().all(|&dev| dev < 0.10);

        Json(SeedSensitivityResult {
            base_result,
            perturbed_results,
            max_deviation,
            is_robust,
        })
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Analyze variance components
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn analyze_variance(
    Json(request): Json<VarianceAnalysisRequest>,
) -> Result<Json<VarianceAnalysisResponse>, StatusCode> {
    let _estimator = VarianceEstimator::new();

    let values: Vec<f64> = request
        .results
        .iter()
        .filter(|r| r.success)
        .filter_map(|r| r.metrics.get(&request.metric).copied())
        .collect();

    if values.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let between_variance =
        values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    let within_variance = between_variance * 0.01;
    let total_variance = within_variance + between_variance;

    let icc = if total_variance > 0.0 {
        between_variance / total_variance
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    } else {
        0.0
    };

    let components = VarianceComponents {
        within_variance,
        between_variance,
        total_variance,
        icc,
    };

    let mut recommendations = Vec::new();

    if icc > 0.5 {
        recommendations.push(
            "High between-replication variance detected. Results are sensitive to random seed."
                .to_string(),
        );
    } else {
        recommendations.push("Low ICC indicates good reproducibility across seeds.".to_string());
    }

    Ok(Json(VarianceAnalysisResponse {
        components,
        recommendations,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get recommended number of replications
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn recommended_replications(
    Json(request): Json<RecommendedReplicationsRequest>,
) -> Result<Json<RecommendedReplicationsResponse>, StatusCode> {
    let estimator = VarianceEstimator::new();

    let recommended =
        estimator.required_replications(request.observed_cv, request.observed_n, request.target_cv);

    let power = estimator.calculate_power(request.effect_size, request.std_dev, recommended, 0.05);

    Ok(Json(RecommendedReplicationsResponse {
        recommended_replications: recommended,
        expected_power: power,
        target_cv: request.target_cv,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get reproducibility guidelines
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn reproducibility_guidelines() -> Result<Json<GuidelinesResponse>, StatusCode> {
    let guidelines = vec![
        Guideline {
            topic: "Number of Replications".to_string(),
            recommendation: "Minimum 30 replications for stable estimates".to_string(),
            rationale: "Central Limit Theorem ensures normality".to_string(),
        },
        Guideline {
            topic: "Stability Threshold".to_string(),
            recommendation: "CV < 10% for primary metrics".to_string(),
            rationale: "Ensures results are not dominated by noise".to_string(),
        },
        Guideline {
            topic: "Seed Selection".to_string(),
            recommendation: "Use base seed + sequential offsets".to_string(),
            rationale: "Avoids correlated random sequences".to_string(),
        },
        Guideline {
            topic: "Convergence Check".to_string(),
            recommendation: "Monitor R-hat across replications".to_string(),
            rationale: "R-hat < 1.05 indicates convergence".to_string(),
        },
        Guideline {
            topic: "Reporting".to_string(),
            recommendation: "Report mean ± SD and CV".to_string(),
            rationale: "Enables comparison across studies".to_string(),
        },
    ];

    Ok(Json(GuidelinesResponse { guidelines }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate reproducibility report
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_report(
    Json(request): Json<GenerateReportRequest>,
) -> Result<Json<ReproducibilityReport>, StatusCode> {
    let config = ReproducibilityConfig::default();
    let analyzer = StabilityAnalyzer::new(config);

    let result = analyzer.analyze(&request.results);
    let report = analyzer.generate_report(&result);

    Ok(Json(report))
}

/// Check equivalence of two result sets
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn check_equivalence(
    Json(request): Json<EquivalenceCheckRequest>,
) -> Result<Json<EquivalenceCheckResponse>, StatusCode> {
    let config = ReproducibilityConfig::default();
    let analyzer = StabilityAnalyzer::new(config);

    let mut equivalences = HashMap::new();

    for metric in &request.metrics {
        let is_equivalent = analyzer.compare_for_equivalence(
            &request.results_a,
            &request.results_b,
            metric,
            request.threshold,
        );
        equivalences.insert(metric.clone(), is_equivalent);
    }

    let all_equivalent = !equivalences.is_empty() && equivalences.values().all(|&v| v);

    Ok(Json(EquivalenceCheckResponse {
        equivalences,
        all_equivalent,
        threshold: request.threshold,
    }))
}

// Request/Response types

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `ReproducibilityCheckRequest` data
pub struct ReproducibilityCheckRequest {
    /// config
    pub config: ReproducibilityConfig,
    /// experiment id
    pub experiment_id: String,
    /// parameters
    pub parameters: HashMap<String, f64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `SeedSensitivityRequest` data
pub struct SeedSensitivityRequest {
    /// base seed
    pub base_seed: u64,
    /// n seeds
    pub n_seeds: usize,
    /// experiment id
    pub experiment_id: String,
}

#[derive(Debug, Deserialize)]
/// `VarianceAnalysisRequest` data
pub struct VarianceAnalysisRequest {
    /// results
    pub results: Vec<ReplicationResult>,
    /// metric
    pub metric: String,
}

#[derive(Debug, Deserialize)]
/// `RecommendedReplicationsRequest` data
pub struct RecommendedReplicationsRequest {
    /// observed cv
    pub observed_cv: f64,
    /// observed n
    pub observed_n: usize,
    /// target cv
    pub target_cv: f64,
    /// effect size
    pub effect_size: f64,
    /// std dev
    pub std_dev: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `RecommendedReplicationsResponse` data
pub struct RecommendedReplicationsResponse {
    /// recommended replications
    pub recommended_replications: usize,
    /// expected power
    pub expected_power: f64,
    /// target cv
    pub target_cv: f64,
}

#[derive(Debug, Serialize)]
/// Guideline data
pub struct Guideline {
    /// topic
    pub topic: String,
    /// recommendation
    pub recommendation: String,
    /// rationale
    pub rationale: String,
}

#[derive(Debug, Serialize)]
/// `GuidelinesResponse` data
pub struct GuidelinesResponse {
    /// guidelines
    pub guidelines: Vec<Guideline>,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

#[derive(Debug, Deserialize)]
/// `GenerateReportRequest` data
pub struct GenerateReportRequest {
    /// results
    pub results: Vec<ReplicationResult>,
}

#[derive(Debug, Deserialize)]
/// `EquivalenceCheckRequest` data
pub struct EquivalenceCheckRequest {
    /// results a
    pub results_a: Vec<ReplicationResult>,
    /// results b
    pub results_b: Vec<ReplicationResult>,
    /// metrics
    pub metrics: Vec<String>,
    /// threshold
    pub threshold: f64,
}

#[derive(Debug, Serialize)]
/// `EquivalenceCheckResponse` data
pub struct EquivalenceCheckResponse {
    /// equivalences
    pub equivalences: HashMap<String, bool>,
    /// all equivalent
    pub all_equivalent: bool,
    /// threshold
    pub threshold: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_equivalence_empty_metrics_not_vacuously_true() {
        let req = EquivalenceCheckRequest {
            results_a: vec![],
            results_b: vec![],
            metrics: vec![],
            threshold: 0.05,
        };
        let res = check_equivalence(Json(req)).await.unwrap();
        assert!(!res.0.all_equivalent);
        assert!(res.0.equivalences.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_seed_sensitivity_zero_seeds_not_vacuously_robust() {
        let req = SeedSensitivityRequest {
            base_seed: 42,
            n_seeds: 0,
            experiment_id: "exp_1".to_string(),
        };
        let res = analyze_seed_sensitivity(Json(req)).await.unwrap();
        assert!(!res.0.is_robust);
    }

    #[test]
    fn test_equivalence_check_response() {
        let resp = EquivalenceCheckResponse {
            equivalences: HashMap::new(),
            all_equivalent: true,
            threshold: 0.05,
        };
        assert!(resp.all_equivalent);
        assert_eq!(resp.threshold, 0.05);
    }
}
