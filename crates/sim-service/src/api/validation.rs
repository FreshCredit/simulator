//! Validation API Module (P1-VAL-1)
//!
//! REST endpoints for statistical power analysis and sample size calculations.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::validation::power::{
    calculator::PowerCalculator, effect_size::EffectSizeConverter,
    sample_size::SampleSizeCalculator, types::*,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validation API state
#[derive(Clone, Debug)]
pub struct ValidationState {
    power_calc: Arc<PowerCalculator>,
    sample_size_calc: Arc<SampleSizeCalculator>,
}

impl ValidationState {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[must_use]
    /// new
    pub fn new() -> Self {
        Self {
            power_calc: Arc::new(PowerCalculator::new()),
            sample_size_calc: Arc::new(SampleSizeCalculator::new()),
        }
    }
}

impl Default for ValidationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate statistical power
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn calculate_power(
    State(state): State<ValidationState>,
    Json(request): Json<PowerCalculationRequest>,
) -> Result<Json<PowerCalculationResponse>, StatusCode> {
    let config = request.config;
    let n = request.n_per_group;

    let result = state.power_calc.calculate_power(&config, n);

    Ok(Json(PowerCalculationResponse {
        power: result.power,
        power_level: format!("{:?}", result.power_level).to_lowercase(),
        power_description: result.power_level.description().to_string(),
        achieved_target: result.power >= config.target_power,
        recommendations: result.recommendations,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Calculate required sample size
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn calculate_sample_size(
    State(state): State<ValidationState>,
    Json(request): Json<SampleSizeRequest>,
) -> Result<Json<SampleSizeResponse>, StatusCode> {
    let result = state
        .sample_size_calc
        .calculate_sample_size(&request.config);

    // Determine if attrition adjustment is needed
    let with_attrition = if let Some(rate) = request.attrition_rate {
        let adjusted = state
            .sample_size_calc
            .calculate_with_attrition(&request.config, rate);
        Some(AdjustedSampleSize {
            attrition_rate: rate,
            n_per_group: adjusted.n_per_group,
            total_n: adjusted.total_n,
        })
    } else {
        None
    };

    Ok(Json(SampleSizeResponse {
        n_per_group: result.n_per_group,
        total_n: result.total_n,
        n_for_80_power: result.n_for_80_power,
        n_for_90_power: result.n_for_90_power,
        n_for_95_power: result.n_for_95_power,
        power_curve: result.power_curve,
        with_attrition,
        interpretation: format!(
            "To detect a {} effect with {}% power at α={}",
            classify_effect_size(request.config.effect_size.magnitude),
            (request.config.target_power * 100.0) as i32,
            request.config.alpha
        ),
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate power curve
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_power_curve(
    State(state): State<ValidationState>,
    Json(request): Json<PowerCurveRequest>,
) -> Result<Json<PowerCurveResponse>, StatusCode> {
    let curve = state.power_calc.generate_power_curve(
        &request.config,
        request.n_min,
        request.n_max,
        request.n_step,
    );

    // Find sample size for target power
    let target_n = state
        .power_calc
        .find_minimum_n(&request.config, request.config.target_power, request.n_max)
        .unwrap_or(request.n_max);

    Ok(Json(PowerCurveResponse {
        curve,
        target_n,
        target_power: request.config.target_power,
    }))
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Validate proposed sample size
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn validate_sample_size(
    State(state): State<ValidationState>,
    Json(request): Json<ValidateSampleSizeRequest>,
) -> Result<Json<ValidateSampleSizeResponse>, StatusCode> {
    let validation = state
        .sample_size_calc
        .validate_calculation(&request.config, request.proposed_n);

    Ok(Json(ValidateSampleSizeResponse {
        proposed_n: validation.proposed_n,
        achieved_power: validation.achieved_power,
        target_power: validation.target_power,
        is_adequate: validation.is_adequate,
        power_deficit: validation.power_deficit,
        recommendation: validation.recommendation,
        power_level: format!("{:?}", PowerLevel::from_power(validation.achieved_power))
            .to_lowercase(),
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get effect size conventions
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_effect_size_conventions() -> Result<Json<EffectSizeConventionsResponse>, StatusCode>
{
    let mut conventions = HashMap::new();

    for effect_type in [
        EffectSizeType::CohensD,
        EffectSizeType::PearsonsR,
        EffectSizeType::EtaSquared,
        EffectSizeType::OddsRatio,
    ] {
        conventions.insert(
            format!("{effect_type:?}").to_lowercase(),
            EffectSizeConvention {
                small: conventions::small(effect_type),
                medium: conventions::medium(effect_type),
                large: conventions::large(effect_type),
            },
        );
    }

    Ok(Json(EffectSizeConventionsResponse { conventions }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Convert between effect sizes
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn convert_effect_size(
    Json(request): Json<ConvertEffectSizeRequest>,
) -> Result<Json<ConvertEffectSizeResponse>, StatusCode> {
    let converted = EffectSizeConverter::convert(request.value, request.from_type, request.to_type);

    Ok(Json(ConvertEffectSizeResponse {
        original_value: request.value,
        original_type: format!("{:?}", request.from_type).to_lowercase(),
        converted_value: converted,
        converted_type: format!("{:?}", request.to_type).to_lowercase(),
    }))
}

/// Get power analysis summary for experiment planning
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_power_analysis_summary(
    State(state): State<ValidationState>,
    Json(request): Json<PowerSummaryRequest>,
) -> Result<Json<PowerSummaryResponse>, StatusCode> {
    let mut results = Vec::new();

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    for config in &request.configs {
        let sample_size = state.sample_size_calc.calculate_sample_size(config);
        let power_result = state
            .power_calc
            .calculate_power(config, sample_size.n_per_group);

        results.push(SingleAnalysisSummary {
            test_type: format!("{:?}", config.test).to_lowercase(),
            effect_size: config.effect_size.magnitude,
            effect_size_type: format!("{:?}", config.effect_size.effect_type).to_lowercase(),
            target_power: config.target_power,
            required_n: sample_size.n_per_group,
            achieved_power: power_result.power,
            power_level: format!("{:?}", power_result.power_level).to_lowercase(),
        });
    }

    // Calculate summary statistics before moving results
    let total_metrics = results.len();
    let max_required_n = results.iter().map(|r| r.required_n).max().unwrap_or(0);
    let min_required_n = results.iter().map(|r| r.required_n).min().unwrap_or(0);

    // Find the most demanding analysis (largest sample size)
    let most_demanding = results.iter().max_by_key(|r| r.required_n).map(|r| {
        format!(
            "{} requires largest sample (n={} per group)",
            r.test_type, r.required_n
        )
    });

    // TAG: surface=simulation owner=research-team rule=SIM-001
    Ok(Json(PowerSummaryResponse {
        analyses: results,
        total_metrics,
        max_required_n,
        min_required_n,
        overall_recommendation: most_demanding.unwrap_or_default(),
    }))
}

// Helper function to classify effect sizes
fn classify_effect_size(d: f64) -> &'static str {
    match d.abs() {
        d if d < 0.2 => "negligible",
        d if d < 0.5 => "small",
        d if d < 0.8 => "medium",
        d if d < 1.2 => "large",
        _ => "very large",
    }
}

// Request/Response types

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `PowerCalculationRequest` data
pub struct PowerCalculationRequest {
    /// config
    pub config: PowerAnalysisConfig,
    /// n per group
    pub n_per_group: usize,
}

#[derive(Debug, Serialize)]
/// `PowerCalculationResponse` data
pub struct PowerCalculationResponse {
    /// power
    pub power: f64,
    /// power level
    pub power_level: String,
    /// power description
    pub power_description: String,
    /// achieved target
    pub achieved_target: bool,
    /// recommendations
    pub recommendations: Vec<String>,
}

#[derive(Debug, Deserialize)]
/// `SampleSizeRequest` data
pub struct SampleSizeRequest {
    /// config
    pub config: PowerAnalysisConfig,
    /// attrition rate
    pub attrition_rate: Option<f64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `SampleSizeResponse` data
pub struct SampleSizeResponse {
    /// n per group
    pub n_per_group: usize,
    /// total n
    pub total_n: usize,
    /// n for 80 power
    pub n_for_80_power: usize,
    /// n for 90 power
    pub n_for_90_power: usize,
    /// n for 95 power
    pub n_for_95_power: usize,
    /// power curve
    pub power_curve: Vec<PowerCurvePoint>,
    /// with attrition
    pub with_attrition: Option<AdjustedSampleSize>,
    /// interpretation
    pub interpretation: String,
}

#[derive(Debug, Serialize)]
/// `AdjustedSampleSize` data
pub struct AdjustedSampleSize {
    /// attrition rate
    pub attrition_rate: f64,
    /// n per group
    pub n_per_group: usize,
    /// total n
    pub total_n: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `ValidateSampleSizeRequest` data
pub struct ValidateSampleSizeRequest {
    /// config
    pub config: PowerAnalysisConfig,
    /// proposed n
    pub proposed_n: usize,
}

#[derive(Debug, Serialize)]
/// `ValidateSampleSizeResponse` data
pub struct ValidateSampleSizeResponse {
    /// proposed n
    pub proposed_n: usize,
    /// achieved power
    pub achieved_power: f64,
    /// target power
    pub target_power: f64,
    /// is adequate
    pub is_adequate: bool,
    /// power deficit
    pub power_deficit: f64,
    /// recommendation
    pub recommendation: String,
    /// power level
    pub power_level: String,
}

#[derive(Debug, Serialize)]
/// `EffectSizeConvention` data
pub struct EffectSizeConvention {
    /// small
    pub small: f64,
    /// medium
    pub medium: f64,
    /// large
    pub large: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `EffectSizeConventionsResponse` data
pub struct EffectSizeConventionsResponse {
    /// conventions
    pub conventions: HashMap<String, EffectSizeConvention>,
}

#[derive(Debug, Deserialize)]
/// `ConvertEffectSizeRequest` data
pub struct ConvertEffectSizeRequest {
    /// value
    pub value: f64,
    /// from type
    pub from_type: EffectSizeType,
    /// to type
    pub to_type: EffectSizeType,
}

#[derive(Debug, Serialize)]
/// `ConvertEffectSizeResponse` data
pub struct ConvertEffectSizeResponse {
    /// original value
    pub original_value: f64,
    /// original type
    pub original_type: String,
    /// converted value
    pub converted_value: f64,
    /// converted type
    pub converted_type: String,
}

#[derive(Debug, Deserialize)]
/// `PowerSummaryRequest` data
pub struct PowerSummaryRequest {
    /// configs
    pub configs: Vec<PowerAnalysisConfig>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `PowerSummaryResponse` data
pub struct PowerSummaryResponse {
    /// analyses
    pub analyses: Vec<SingleAnalysisSummary>,
    /// total metrics
    pub total_metrics: usize,
    /// max required n
    pub max_required_n: usize,
    /// min required n
    pub min_required_n: usize,
    /// overall recommendation
    pub overall_recommendation: String,
}

#[derive(Debug, Serialize)]
/// `SingleAnalysisSummary` data
pub struct SingleAnalysisSummary {
    /// test type
    pub test_type: String,
    /// effect size
    pub effect_size: f64,
    /// effect size type
    pub effect_size_type: String,
    /// target power
    pub target_power: f64,
    /// required n
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub required_n: usize,
    /// achieved power
    pub achieved_power: f64,
    /// power level
    pub power_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::power::types::{
        EffectSizeConfig, EffectSizeSource, EffectSizeType, PowerAnalysisConfig, StatisticalTest,
    };

    fn test_config() -> PowerAnalysisConfig {
        PowerAnalysisConfig {
            alpha: 0.05,
            target_power: 0.8,
            test: StatisticalTest::TTestIndependent,
            effect_size: EffectSizeConfig {
                effect_type: EffectSizeType::CohensD,
                magnitude: 0.5,
                source: EffectSizeSource::Convention,
                ci: None,
            },
            num_groups: 2,
            allocation_ratio: None,
            one_tailed: false,
            min_detectable_effect: None,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_calculate_power() {
        let state = ValidationState::new();
        let request = PowerCalculationRequest {
            config: test_config(),
            n_per_group: 64,
        };

        let result = calculate_power(State(state), Json(request)).await.unwrap();
        assert!(result.power > 0.0 && result.power <= 1.0);
    }

    #[tokio::test]
    async fn test_calculate_sample_size() {
        let state = ValidationState::new();
        let request = SampleSizeRequest {
            config: test_config(),
            attrition_rate: None,
        };

        let result = calculate_sample_size(State(state), Json(request))
            .await
            .unwrap();
        assert!(result.n_per_group > 0);
        assert!(result.total_n > 0);
    }

    #[tokio::test]
    async fn test_get_effect_size_conventions() {
        let result = get_effect_size_conventions().await.unwrap();
        assert!(!result.conventions.is_empty());
    }
}
