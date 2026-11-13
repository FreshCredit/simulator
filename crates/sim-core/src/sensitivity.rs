//! Morris Method Screening for Parameter Sensitivity Analysis
//! CODE-006: Module size justified - complete Morris Method algorithm
//! CODE-010: Minimal references justified - self-contained statistical module
//!
//! Implements the Morris Method (Morris 1991; Campolongo et al. 2007) for
// TAG: surface=simulation owner=research-team rule=SIM-001
//! identifying "inactive" parameters in complex agent-based models.
//!
//! Per simulation-plan.md §4.4.1: Phase 0 requirement for parameter pruning
//! before full factorial experiments.
//!
//! ## Key Concepts
//! - **Elementary Effects (EE)**: Measure of parameter influence
//! - **μ* (mu-star)**: Mean of absolute elementary effects (importance ranking)
//! - **σ (sigma)**: Standard deviation of elementary effects (nonlinearity/interaction)
//! - **Classification**: Active, Inactive, Nonlinear based on μ* and σ
//!
//! # Compliance Justification (CODE-006, CODE-010)
//!
//! This module is intentionally large (~550 lines) as it implements the complete
//! Morris Method sensitivity analysis algorithm per Campolongo et al. (2007).
//! Per research methodology requirements (simulation-plan.md §4.4), this module
//! must contain the full implementation including:
//! - Trajectory generation and sampling
//! - Elementary effect computation
//! - Statistical aggregation (μ*, σ, confidence intervals)
//! - Parameter classification logic
//!
//! This is core research functionality for parameter screening, not technical debt.
//! The algorithm is self-contained and splitting would harm readability.
//!
//! See: `docs/claims/sensitivity_module_justification.md`

use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Morris Method configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorrisConfig {
    /// Number of trajectories (r in Morris notation)
    /// Default: 10 (minimum for statistical validity)
    pub trajectories: usize,
    /// Number of levels per parameter (p in Morris notation)
    /// Default: 4 (common choice)
    pub levels: usize,
    /// Parameter bounds: (min, max) for each parameter
    pub parameter_bounds: HashMap<String, (f32, f32)>,
    /// Agent count for screening runs
    pub agent_count: usize,
    /// Tick count for screening runs
    pub tick_count: u64,
    /// Random seed for reproducibility
    pub seed: u64,
}

impl Default for MorrisConfig {
    fn default() -> Self {
        Self {
            trajectories: 10,
            levels: 4,
            parameter_bounds: Self::default_parameter_bounds(),
            agent_count: 1000,
            tick_count: 2000,
            seed: 42,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MorrisConfig {
    /// Default parameter bounds for simulator parameters
    /// Per simulation-plan.md §4.4.1: Genome (40 loci), DOCAS (10 params), DMN (3 params)  // SIM-GENOME-001: Sensitivity analysis parameters
    fn default_parameter_bounds() -> HashMap<String, (f32, f32)> {
        let mut bounds = HashMap::new();

        // Genome parameters (20 traits across 40 loci)
        // Key traits expected to be active per §4.4.1
        bounds.insert("docas_gain_D".to_string(), (0.05, 0.2)); // SIM-GENOME-001: Sensitivity param
        bounds.insert("docas_gain_O".to_string(), (0.05, 0.2)); // SIM-GENOME-001: Sensitivity param
        bounds.insert("docas_gain_C".to_string(), (0.05, 0.2)); // SIM-GENOME-001: Sensitivity param
        bounds.insert("docas_gain_A".to_string(), (0.05, 0.2)); // SIM-GENOME-001: Sensitivity param
        bounds.insert("docas_gain_S".to_string(), (0.05, 0.2)); // SIM-GENOME-001: Sensitivity param
        bounds.insert("planning_depth".to_string(), (0.0, 1.0));
        bounds.insert("cooperation_threshold".to_string(), (0.3, 0.8));
        bounds.insert("risk_aversion".to_string(), (0.0, 1.0));
        bounds.insert("social_learning_rate".to_string(), (0.1, 0.9));
        bounds.insert("mutation_rate".to_string(), (0.0001, 0.01));

        // DOCAS parameters (α decay, β sensitivity per component)  // SIM-GENOME-001: Sensitivity analysis
        bounds.insert("docas_alpha_D".to_string(), (0.9, 0.99)); // SIM-GENOME-001: Sensitivity param
        bounds.insert("docas_alpha_O".to_string(), (0.9, 0.99)); // SIM-GENOME-001: Sensitivity param
        bounds.insert("docas_alpha_C".to_string(), (0.9, 0.99)); // SIM-GENOME-001: Sensitivity param
        bounds.insert("docas_alpha_A".to_string(), (0.9, 0.99)); // SIM-GENOME-001: Sensitivity param
        bounds.insert("docas_alpha_S".to_string(), (0.9, 0.99)); // SIM-GENOME-001: Sensitivity param

        // DMN parameters
        bounds.insert("dmn_K".to_string(), (3.0, 20.0));
        bounds.insert("dmn_H".to_string(), (1.0, 5.0));
        bounds.insert("dmn_B".to_string(), (100.0, 1000.0));

        // Choice Architecture parameters
        bounds.insert("default_effect".to_string(), (0.3, 0.4));
        bounds.insert("friction_sensitivity".to_string(), (0.1, 0.3));
        bounds.insert("loss_aversion".to_string(), (2.0, 2.5));

        bounds
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get parameter names as ordered list
    #[must_use]
    pub fn parameter_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.parameter_bounds.keys().cloned().collect();
        names.sort();
        names
    }
}

/// Elementary Effect for a single parameter trajectory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementaryEffect {
    /// Parameter name
    pub parameter: String,
    /// Change in output per unit change in parameter
    pub ee: f32,
    /// Delta (step size) used
    pub delta: f32,
    /// Direction of change (+1 or -1)
    pub direction: i8,
}

/// Morris screening results for a single parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterScreeningResult {
    /// Parameter name
    pub parameter: String,
    /// Mean of elementary effects (signed)
    pub mu: f32,
    /// Mean of absolute elementary effects (importance measure)
    pub mu_star: f32,
    /// Standard deviation of elementary effects (nonlinearity measure)
    pub sigma: f32,
    /// Number of trajectories used
    pub trajectory_count: usize,
    /// All elementary effects collected
    pub elementary_effects: Vec<f32>,
    /// Parameter classification
    pub classification: ParameterClassification,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Parameter classification based on Morris metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterClassification {
    /// High μ*, low σ: Important and linear
    ActiveLinear,
    /// High μ*, high σ: Important and nonlinear/interactive
    ActiveNonlinear,
    /// Low μ*, high σ: Nonlinear but not important
    Nonlinear,
    /// Low μ*, low σ: Not important (can be fixed)
    Inactive,
}

impl ParameterClassification {
    /// Classify parameter based on μ* and σ
    /// Per Campolongo et al. 2007
    #[must_use]
    pub fn classify(mu_star: f32, sigma: f32, threshold: f32) -> Self {
        if !mu_star.is_finite() || !sigma.is_finite() || !threshold.is_finite() {
            return Self::Inactive;
        }
        if mu_star > threshold && sigma < threshold {
            Self::ActiveLinear
        } else if mu_star > threshold && sigma >= threshold {
            Self::ActiveNonlinear
        } else if mu_star <= threshold && sigma >= threshold {
            Self::Nonlinear
        } else {
            Self::Inactive
        }
    }

    /// Get description of classification
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::ActiveLinear => "Important linear parameter (include in analysis)",
            Self::ActiveNonlinear => {
                "Important nonlinear/interactive parameter (needs interaction analysis)"
            }
            Self::Nonlinear => "Nonlinear but not important (monitor)",
            Self::Inactive => "Inactive parameter (can be fixed to default)",
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Complete Morris screening results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorrisScreeningResults {
    /// Configuration used
    pub config: MorrisConfig,
    /// Results for each parameter
    pub parameter_results: Vec<ParameterScreeningResult>,
    /// Threshold used for classification
    pub threshold: f32,
    /// Recommended parameters to include in full factorial
    pub recommended_active: Vec<String>,
    /// Parameters that can be fixed to default
    pub recommended_inactive: Vec<String>,
    /// Timestamp
    pub completed_at: String,
}

impl MorrisScreeningResults {
    /// Get results sorted by μ* (importance)
    #[must_use]
    pub fn sorted_by_importance(&self) -> Vec<&ParameterScreeningResult> {
        let mut results: Vec<_> = self.parameter_results.iter().collect();
        results.sort_by(|a, b| {
            b.mu_star
                .partial_cmp(&a.mu_star)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.parameter.cmp(&b.parameter))
        });
        results
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get μ* threshold as fraction of maximum
    pub fn compute_threshold(&self, fraction: f32) -> f32 {
        let max_mu_star = self
            .parameter_results
            .iter()
            .map(|r| r.mu_star)
            .fold(0.0, f32::max);
        max_mu_star * fraction
    }

    /// Generate pruning recommendations
    pub fn generate_recommendations(&mut self) {
        let threshold = self.compute_threshold(0.1); // 10% of max μ*
        self.threshold = threshold;

        self.recommended_active.clear();
        self.recommended_inactive.clear();

        for result in &mut self.parameter_results {
            if threshold > 0.0 {
                result.classification =
                    ParameterClassification::classify(result.mu_star, result.sigma, threshold);
            }
            match result.classification {
                ParameterClassification::ActiveLinear
                | ParameterClassification::ActiveNonlinear => {
                    self.recommended_active.push(result.parameter.clone());
                }
                ParameterClassification::Inactive => {
                    self.recommended_inactive.push(result.parameter.clone());
                }
                _ => {} // Nonlinear: monitor
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate Morris trajectory sampling
/// Per Morris 1991: OAT (One-At-a-Time) design
/// # Panics
///
/// Panics if preconditions are not met.
#[allow(clippy::implicit_hasher)]
pub fn generate_trajectory(
    parameter_names: &[String],
    bounds: &HashMap<String, (f32, f32)>,
    levels: usize,
    rng: &mut impl Rng,
) -> Vec<HashMap<String, f32>> {
    let _k = parameter_names.len();
    let safe_levels = levels.max(2);
    let delta = 1.0 / (safe_levels as f32 - 1.0);

    // Random base point in grid
    let mut base_point: HashMap<String, f32> = HashMap::new();
    for param in parameter_names {
        let raw_b = bounds.get(param).copied().unwrap_or((0.0, 1.0));
        let min = raw_b.0.min(raw_b.1);
        let max = raw_b.0.max(raw_b.1);
        let level = rng.gen_range(0..safe_levels) as f32;
        let value = min + (max - min) * level * delta;
        base_point.insert(param.clone(), value);
    }

    // Generate trajectory: k+1 points, changing one parameter at a time
    let mut trajectory = vec![base_point.clone()];
    let mut current_point = base_point;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Random order of parameter changes
    let mut param_order: Vec<_> = parameter_names.to_vec();
    // Shuffle using Fisher-Yates
    for i in (1..param_order.len()).rev() {
        let j = rng.gen_range(0..=i);
        param_order.swap(i, j);
    }

    for param in param_order {
        let raw_b = bounds.get(&param).copied().unwrap_or((0.0, 1.0));
        let min = raw_b.0.min(raw_b.1);
        let max = raw_b.0.max(raw_b.1);
        let current_val = current_point.get(&param).copied().unwrap_or(min);

        // Compute discrete current level in [0, safe_levels - 1]
        let current_level = ((current_val - min) / (max - min).max(1e-12) / delta).round();
        // Determine direction (+1.0 or -1.0), reflecting if boundary would be exceeded
        let mut direction = if rng.gen::<bool>() { 1.0 } else { -1.0 };
        if current_level + direction >= safe_levels as f32 {
            direction = -1.0;
        } else if current_level + direction < 0.0 {
            direction = 1.0;
        }

        let new_level = (current_level + direction).clamp(0.0, (safe_levels - 1) as f32);
        let new_val = min + (max - min) * new_level * delta;

        current_point.insert(param, new_val);
        trajectory.push(current_point.clone());
    }

    trajectory
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute elementary effects for a trajectory
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn compute_elementary_effects(
    trajectory: &[HashMap<String, f32>],
    outputs: &[f32],
    parameter_names: &[String],
    bounds: &HashMap<String, (f32, f32)>,
) -> Vec<ElementaryEffect> {
    let mut effects = Vec::new();

    // Guard against empty or single-element trajectory
    if trajectory.len() < 2 || outputs.len() < 2 {
        return effects;
    }

    for i in 0..trajectory.len().saturating_sub(1) {
        if i + 1 >= outputs.len() {
            break;
        }
        let point_i = &trajectory[i];
        let point_j = &trajectory[i + 1];

        // Find which parameter changed
        let changed_param = parameter_names
            .iter()
            .find(|&p| {
                let v1 = point_i.get(p).copied().unwrap_or(0.0);
                let v2 = point_j.get(p).copied().unwrap_or(0.0);
                (v1 - v2).abs() > 1e-10
            })
            .cloned();

        if let Some(param) = changed_param {
            let raw_b = bounds.get(&param).copied().unwrap_or((0.0, 1.0));
            let min = raw_b.0.min(raw_b.1);
            let max = raw_b.0.max(raw_b.1);
            let range = (max - min).max(1e-12);
            let val_i = point_i.get(&param).copied().unwrap_or(0.0);
            let val_j = point_j.get(&param).copied().unwrap_or(0.0);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let delta_x = (val_j - val_i) / range;
            let delta_y = outputs[i + 1] - outputs[i];

            if delta_x.abs() > 1e-10 {
                effects.push(ElementaryEffect {
                    parameter: param,
                    ee: delta_y / delta_x,
                    delta: delta_x.abs(),
                    direction: if delta_x > 0.0 { 1 } else { -1 },
                });
            }
        }
    }

    effects
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Run Morris screening analysis
/// Returns screening results for all parameters
/// # Panics
///
/// Panics if preconditions are not met.
pub fn run_morris_screening<F>(config: &MorrisConfig, run_simulation: F) -> MorrisScreeningResults
where
    F: Fn(&HashMap<String, f32>) -> f32,
{
    let mut parameter_names: Vec<_> = config.parameter_bounds.keys().cloned().collect();
    parameter_names.sort();
    let mut rng = rand::rngs::StdRng::seed_from_u64(config.seed);

    // Collect elementary effects for all trajectories
    let mut all_effects: HashMap<String, Vec<f32>> = HashMap::new();
    for param in &parameter_names {
        all_effects.insert(param.clone(), Vec::new());
    }

    // Run trajectories
    for _ in 0..config.trajectories {
        let trajectory = generate_trajectory(
            &parameter_names,
            &config.parameter_bounds,
            config.levels,
            &mut rng,
        );

        // Run simulation for each point in trajectory
        #[allow(clippy::redundant_closure)]
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let outputs: Vec<f32> = trajectory.iter().map(|p| run_simulation(p)).collect();

        // Compute elementary effects
        let effects = compute_elementary_effects(
            &trajectory,
            &outputs,
            &parameter_names,
            &config.parameter_bounds,
        );

        // Collect effects by parameter
        for effect in effects {
            if let Some(vec) = all_effects.get_mut(&effect.parameter) {
                vec.push(effect.ee);
            }
        }
    }

    // Compute statistics for each parameter
    let mut parameter_results = Vec::new();
    let default_effects = Vec::new();
    for param in &parameter_names {
        let effects = all_effects.get(param).unwrap_or(&default_effects);

        let mu = if effects.is_empty() {
            0.0
        } else {
            effects.iter().sum::<f32>() / effects.len() as f32
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mu_star = if effects.is_empty() {
            0.0
        } else {
            effects.iter().map(|e| e.abs()).sum::<f32>() / effects.len() as f32
        };

        let sigma = if effects.len() < 2 {
            0.0
        } else {
            let mean = mu;
            let variance = effects.iter().map(|e| (e - mean).powi(2)).sum::<f32>()
                / (effects.len() - 1) as f32;
            variance.sqrt()
        };

        let classification = ParameterClassification::classify(mu_star, sigma, 0.05);

        parameter_results.push(ParameterScreeningResult {
            parameter: param.clone(),
            mu,
            mu_star,
            sigma,
            trajectory_count: config.trajectories,
            elementary_effects: effects.clone(),
            classification,
        });
    }

    let mut results = MorrisScreeningResults {
        config: config.clone(),
        parameter_results,
        threshold: 0.0,
        recommended_active: Vec::new(),
        recommended_inactive: Vec::new(),
        completed_at: chrono::Utc::now().to_rfc3339(),
    };

    results.generate_recommendations();
    results
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Format screening results as markdown report
#[must_use]
pub fn format_screening_report(results: &MorrisScreeningResults) -> String {
    let mut report = String::new();

    report.push_str("# Morris Method Screening Results\n\n");
    report.push_str(&format!("**Completed:** {}\n\n", results.completed_at));
    report.push_str("**Configuration:**\n");
    report.push_str(&format!(
        "- Trajectories: {}\n",
        results.config.trajectories
    ));
    report.push_str(&format!("- Levels: {}\n", results.config.levels));
    report.push_str(&format!("- Agents: {}\n", results.config.agent_count));
    report.push_str(&format!("- Ticks: {}\n\n", results.config.tick_count));

    report.push_str(&format!(
        "**Threshold (10% of max μ*):** {:.4}\n\n",
        results.threshold
    ));

    report.push_str("## Parameter Rankings (by μ*)\n\n");
    report.push_str("| Rank | Parameter | μ* | σ | Classification | Action |\n");
    report.push_str("|------|-----------|-----|---|----------------|--------|\n");

    let sorted = results.sorted_by_importance();
    for (i, result) in sorted.iter().enumerate() {
        let action = match result.classification {
            ParameterClassification::ActiveLinear | ParameterClassification::ActiveNonlinear => {
                "✅ Include"
            }
            ParameterClassification::Inactive => "❌ Fix to default",
            ParameterClassification::Nonlinear => "⚠️ Monitor",
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        report.push_str(&format!(
            "| {} | {} | {:.4} | {:.4} | {:?} | {} |\n",
            i + 1,
            result.parameter,
            result.mu_star,
            result.sigma,
            result.classification,
            action
        ));
    }

    report.push_str("\n## Recommendations\n\n");
    report.push_str(&format!(
        "**Active Parameters ({}):** {}\n\n",
        results.recommended_active.len(),
        results.recommended_active.join(", ")
    ));
    report.push_str(&format!(
        "**Inactive Parameters ({}):** {}\n\n",
        results.recommended_inactive.len(),
        results.recommended_inactive.join(", ")
    ));

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_classification() {
        // High μ*, low σ -> Active Linear
        assert_eq!(
            ParameterClassification::classify(0.5, 0.02, 0.1),
            ParameterClassification::ActiveLinear
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // High μ*, high σ -> Active Nonlinear
        assert_eq!(
            ParameterClassification::classify(0.5, 0.2, 0.1),
            ParameterClassification::ActiveNonlinear
        );

        // Low μ*, high σ -> Nonlinear
        assert_eq!(
            ParameterClassification::classify(0.05, 0.2, 0.1),
            ParameterClassification::Nonlinear
        );

        // Low μ*, low σ -> Inactive
        assert_eq!(
            ParameterClassification::classify(0.05, 0.02, 0.1),
            ParameterClassification::Inactive
        );
    }

    #[test]
    fn test_generate_trajectory() {
        let mut bounds = HashMap::new();
        bounds.insert("param1".to_string(), (0.0, 1.0));
        bounds.insert("param2".to_string(), (0.0, 1.0));

        let param_names = vec!["param1".to_string(), "param2".to_string()];
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let trajectory = generate_trajectory(&param_names, &bounds, 4, &mut rng);

        // Should have k+1 = 3 points for 2 parameters
        assert_eq!(trajectory.len(), 3);

        // Each point should have all parameters
        for point in &trajectory {
            assert!(point.contains_key("param1"));
            assert!(point.contains_key("param2"));
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_compute_elementary_effects() {
        let mut point1 = HashMap::new();
        point1.insert("p1".to_string(), 0.0);
        point1.insert("p2".to_string(), 0.5);

        let mut point2 = HashMap::new();
        point2.insert("p1".to_string(), 0.25); // Changed
        point2.insert("p2".to_string(), 0.5); // Same

        let trajectory = vec![point1, point2];
        let outputs = vec![1.0, 1.5]; // Output changes by 0.5

        let param_names = vec!["p1".to_string(), "p2".to_string()];
        let mut bounds = HashMap::new();
        bounds.insert("p1".to_string(), (0.0, 1.0));
        bounds.insert("p2".to_string(), (0.0, 1.0));

        let effects = compute_elementary_effects(&trajectory, &outputs, &param_names, &bounds);

        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].parameter, "p1");
        // EE = delta_y / delta_x = 0.5 / 0.25 = 2.0
        assert!((effects[0].ee - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_morris_config_default() {
        let config = MorrisConfig::default();
        assert_eq!(config.trajectories, 10);
        assert_eq!(config.levels, 4);
        assert_eq!(config.agent_count, 1000);
        assert_eq!(config.tick_count, 2000);
        assert!(!config.parameter_bounds.is_empty());
    }

    #[test]
    fn test_morris_trajectory_never_drops_boundary_elementary_effects() {
        let mut bounds = HashMap::new();
        bounds.insert("p1".to_string(), (0.0, 10.0));
        bounds.insert("p2".to_string(), (-5.0, 5.0));
        let names = vec!["p1".to_string(), "p2".to_string()];
        let mut rng = rand::rngs::StdRng::seed_from_u64(999);

        // With safe boundary reflection, every step in k+1 points changes a parameter by exactly delta_x
        for _ in 0..50 {
            let traj = generate_trajectory(&names, &bounds, 4, &mut rng);
            assert_eq!(traj.len(), 3);
            let dummy_outputs = vec![1.0, 2.0, 3.0];
            let effects = compute_elementary_effects(&traj, &dummy_outputs, &names, &bounds);
            assert_eq!(
                effects.len(),
                2,
                "Both parameters must produce non-zero elementary effects with reflected steps"
            );
        }
    }

    #[test]
    fn test_morris_parameter_names_sorted_deterministic() {
        let config = MorrisConfig::default();
        let names = config.parameter_names();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(
            names, sorted,
            "parameter_names must be deterministically sorted"
        );
    }
}
