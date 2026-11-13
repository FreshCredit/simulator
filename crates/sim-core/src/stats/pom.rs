// TAG: surface=simulation owner=research-team rule=SIM-001
//! Pattern-Oriented Modeling (POM) validation metrics and acceptance bands

use serde::{Deserialize, Serialize};

/// POM validation module version identifier
#[inline]
#[must_use]
pub const fn pom_validation_version() -> &'static str {
    "0.1.0"
}

/// Default POM acceptance tolerance
#[inline]
#[must_use]
pub const fn default_pom_tolerance() -> f32 {
    0.05
}

/// POM Validation Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomValidationResult {
    /// Pattern name
    pub pattern: String,
    /// Metric name
    pub metric: String,
    /// Observed value
    pub observed: f32,
    /// Acceptance band (min, max)
    pub acceptance_band: (f32, f32),
    /// Whether validation passed
    pub passed: bool,
    /// Effect size (if comparing conditions)
    pub effect_size: Option<f32>,
    /// Confidence interval
    pub confidence_interval: Option<(f32, f32)>,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Data quality report (if available)
    pub data_quality_warnings: Vec<String>,
}

/// Validate POM target
#[must_use]
pub fn validate_pom_target(
    pattern: &str,
    metric: &str,
    observed: f32,
    acceptance_min: f32,
    acceptance_max: f32,
) -> PomValidationResult {
    let mut warnings = Vec::new();
    if observed.is_nan() {
        warnings.push("Observed value is NaN".to_string());
    }
    if observed.is_infinite() {
        warnings.push("Observed value is infinite".to_string());
    }

    PomValidationResult {
        pattern: pattern.to_string(),
        metric: metric.to_string(),
        observed,
        acceptance_band: (acceptance_min, acceptance_max),
        passed: !observed.is_nan()
            && !observed.is_infinite()
            && observed >= acceptance_min
            && observed <= acceptance_max,
        effect_size: None,
        confidence_interval: None,
        data_quality_warnings: warnings,
    }
}

/// Validate POM target with effect size
#[must_use]
pub fn validate_pom_target_with_effect(
    pattern: &str,
    metric: &str,
    observed: f32,
    acceptance_min: f32,
    acceptance_max: f32,
    group1: &[f32],
    group2: &[f32],
) -> PomValidationResult {
    let (d, ci_lower, ci_upper) = super::cohens_d_with_ci(group1, group2);

    let mut warnings = Vec::new();
    if observed.is_nan() {
        warnings.push("Observed value is NaN".to_string());
    }
    if observed.is_infinite() {
        warnings.push("Observed value is infinite".to_string());
    }

    PomValidationResult {
        pattern: pattern.to_string(),
        metric: metric.to_string(),
        observed,
        acceptance_band: (acceptance_min, acceptance_max),
        passed: !observed.is_nan()
            && !observed.is_infinite()
            && observed >= acceptance_min
            && observed <= acceptance_max,
        effect_size: Some(d),
        confidence_interval: Some((ci_lower, ci_upper)),
        data_quality_warnings: warnings,
    }
}
