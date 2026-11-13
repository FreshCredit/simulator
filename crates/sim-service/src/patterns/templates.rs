//! Pattern Templates - Pre-defined experiment patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Built-in pattern templates
#[derive(Debug)]
pub struct PatternTemplates;

impl PatternTemplates {
    /// Get all available templates
    #[must_use]
    pub fn all() -> Vec<TemplateInfo> {
        vec![
            TemplateInfo {
                id: "basic_sweep".to_string(),
                name: "Basic Parameter Sweep".to_string(),
                description: "Sweep a single parameter across a range".to_string(),
                category: TemplateCategory::Exploration,
                pattern_type: "parameter_sweep".to_string(),
            },
            TemplateInfo {
                id: "grid_search".to_string(),
                name: "Grid Search".to_string(),
                description: "Search across multiple parameter dimensions".to_string(),
                category: TemplateCategory::Exploration,
                pattern_type: "parameter_sweep".to_string(),
            },
            TemplateInfo {
                id: "ab_test".to_string(),
                name: "A/B Test".to_string(),
                description: "Compare two variants with statistical testing".to_string(),
                category: TemplateCategory::Comparison,
                pattern_type: "ab_test".to_string(),
            },
            TemplateInfo {
                id: "full_factorial".to_string(),
                name: "Full Factorial Design".to_string(),
                description: "Test all combinations of factor levels".to_string(),
                category: TemplateCategory::Comparison,
                pattern_type: "factorial".to_string(),
            },
            TemplateInfo {
                id: "stress_test".to_string(),
                // TAG: surface=simulation owner=research-team rule=SIM-001
                name: "Stress Test".to_string(),
                description: "Gradually increase load to find breaking point".to_string(),
                category: TemplateCategory::Performance,
                pattern_type: "parameter_sweep".to_string(),
            },
            TemplateInfo {
                id: "scenario_comparison".to_string(),
                name: "Scenario Comparison".to_string(),
                description: "Compare different scenarios against a baseline".to_string(),
                category: TemplateCategory::Comparison,
                pattern_type: "scenario".to_string(),
            },
            TemplateInfo {
                id: "baseline_comparison".to_string(),
                name: "Baseline Comparison".to_string(),
                description: "Compare multiple alternatives against a baseline".to_string(),
                category: TemplateCategory::Comparison,
                pattern_type: "baseline".to_string(),
            },
        ]
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a template by ID
    #[must_use]
    pub fn get(template_id: &str) -> Option<PatternConfig> {
        match template_id {
            "basic_sweep" => Some(Self::basic_sweep()),
            "grid_search" => Some(Self::grid_search()),
            "ab_test" => Some(Self::ab_test()),
            "full_factorial" => Some(Self::full_factorial()),
            "stress_test" => Some(Self::stress_test()),
            "scenario_comparison" => Some(Self::scenario_comparison()),
            "baseline_comparison" => Some(Self::baseline_comparison()),
            _ => None,
        }
    }

    /// Basic single-parameter sweep
    fn basic_sweep() -> PatternConfig {
        PatternConfig {
            pattern: ExperimentPattern::ParameterSweep(SweepConfig {
                parameters: vec![SweepParameter {
                    name: "learning_rate".to_string(),
                    param_type: ParameterType::Float,
                    start: 0.01,
                    end: 0.1,
                    step: Some(0.01),
                    values: None,
                }],
                replications: 5,
                seed: Some(42),
                sampling: SamplingMethod::Grid,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                sample_count: None,
            }),
            global_params: GlobalParams {
                ticks: 1000,
                speed: 1.0,
                record_interval: 10,
                metrics: vec!["fitness".to_string(), "diversity".to_string()],
                record_trajectories: false,
            },
            output: OutputConfig {
                format: OutputFormat::Json,
                destination: "output/sweep".to_string(),
                compress: false,
                summary_metrics: vec!["fitness".to_string()],
                visualization: Some(VisualizationOutput {
                    trajectories: false,
                    time_series: true,
                    heatmaps: false,
                    comparisons: true,
                }),
            },
            constraints: ExecutionConstraints {
                max_duration_secs: 1800,
                max_parallelism: 4,
                memory_limit_mb: 2048,
                retry_failed: true,
                max_retries: 2,
            },
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Grid search across multiple dimensions
    fn grid_search() -> PatternConfig {
        PatternConfig {
            pattern: ExperimentPattern::ParameterSweep(SweepConfig {
                parameters: vec![
                    SweepParameter {
                        name: "learning_rate".to_string(),
                        param_type: ParameterType::Float,
                        start: 0.001,
                        end: 0.1,
                        step: Some(0.01),
                        values: None,
                    },
                    SweepParameter {
                        name: "mutation_rate".to_string(),
                        param_type: ParameterType::Float,
                        start: 0.01,
                        end: 0.1,
                        step: Some(0.01),
                        values: None,
                    },
                    SweepParameter {
                        name: "selection_pressure".to_string(),
                        param_type: ParameterType::Integer,
                        start: 2.0,
                        end: 10.0,
                        step: Some(2.0),
                        values: None,
                    },
                ],
                replications: 3,
                seed: Some(42),
                sampling: SamplingMethod::Grid,
                sample_count: None,
            }),
            global_params: GlobalParams {
                // TAG: surface=simulation owner=research-team rule=SIM-001
                ticks: 2000,
                speed: 1.0,
                record_interval: 50,
                metrics: vec![
                    "fitness".to_string(),
                    "diversity".to_string(),
                    "convergence".to_string(),
                ],
                record_trajectories: false,
            },
            output: OutputConfig {
                format: OutputFormat::Json,
                destination: "output/grid_search".to_string(),
                compress: true,
                summary_metrics: vec!["fitness".to_string(), "convergence".to_string()],
                visualization: Some(VisualizationOutput {
                    trajectories: false,
                    time_series: false,
                    heatmaps: true,
                    comparisons: true,
                }),
            },
            constraints: ExecutionConstraints {
                max_duration_secs: 7200,
                max_parallelism: 8,
                memory_limit_mb: 4096,
                retry_failed: true,
                max_retries: 3,
            },
        }
    }

    /// A/B test template
    fn ab_test() -> PatternConfig {
        let mut control_params = HashMap::new();
        control_params.insert("algorithm".to_string(), serde_json::json!("baseline"));
        control_params.insert("learning_rate".to_string(), serde_json::json!(0.01));

        let mut treatment_params = HashMap::new();
        treatment_params.insert("algorithm".to_string(), serde_json::json!("new_variant"));
        treatment_params.insert("learning_rate".to_string(), serde_json::json!(0.01));

        // TAG: surface=simulation owner=research-team rule=SIM-001
        PatternConfig {
            pattern: ExperimentPattern::ABTest(Box::new(ABTestConfig {
                control: ExperimentConfig {
                    name: "Control".to_string(),
                    description: Some("Baseline algorithm".to_string()),
                    parameters: control_params,
                    agent_config: Some(AgentConfig {
                        count: 100,
                        agent_type: "agent".to_string(),
                        initial_properties: HashMap::new(),
                    }),
                    environment_config: None,
                },
                treatment: ExperimentConfig {
                    name: "Treatment".to_string(),
                    description: Some("New algorithm variant".to_string()),
                    parameters: treatment_params,
                    agent_config: Some(AgentConfig {
                        count: 100,
                        agent_type: "agent".to_string(),
                        initial_properties: HashMap::new(),
                    }),
                    environment_config: None,
                },
                replications: 30,
                seed: Some(42),
                metrics: vec![
                    "final_fitness".to_string(),
                    "convergence_time".to_string(),
                    "stability".to_string(),
                ],
                statistical_test: StatisticalTest::TTest,
                alpha: 0.05,
            })),
            global_params: GlobalParams {
                ticks: 5000,
                speed: 1.0,
                record_interval: 100,
                metrics: vec![
                    "fitness".to_string(),
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    "diversity".to_string(),
                    "convergence".to_string(),
                ],
                record_trajectories: false,
            },
            output: OutputConfig {
                format: OutputFormat::Json,
                destination: "output/ab_test".to_string(),
                compress: false,
                summary_metrics: vec!["final_fitness".to_string(), "convergence_time".to_string()],
                visualization: Some(VisualizationOutput {
                    trajectories: false,
                    time_series: true,
                    heatmaps: false,
                    comparisons: true,
                }),
            },
            constraints: ExecutionConstraints {
                max_duration_secs: 3600,
                max_parallelism: 8,
                memory_limit_mb: 2048,
                retry_failed: true,
                max_retries: 2,
            },
        }
    }

    /// Full factorial design template
    fn full_factorial() -> PatternConfig {
        PatternConfig {
            pattern: ExperimentPattern::Factorial(FactorialConfig {
                factors: vec![
                    Factor {
                        name: "selection".to_string(),
                        levels: vec![
                            serde_json::json!("tournament"),
                            serde_json::json!("roulette"),
                            serde_json::json!("rank"),
                        ],
                    },
                    Factor {
                        // TAG: surface=simulation owner=research-team rule=SIM-001
                        name: "crossover".to_string(),
                        levels: vec![
                            serde_json::json!("single_point"),
                            serde_json::json!("uniform"),
                        ],
                    },
                    Factor {
                        name: "mutation_rate".to_string(),
                        levels: vec![
                            serde_json::json!(0.01),
                            serde_json::json!(0.05),
                            serde_json::json!(0.1),
                        ],
                    },
                ],
                replications: 5,
                seed: Some(42),
                design: FactorialDesign::Full,
                resolution: None,
            }),
            global_params: GlobalParams {
                ticks: 3000,
                speed: 1.0,
                record_interval: 50,
                metrics: vec![
                    "fitness".to_string(),
                    "diversity".to_string(),
                    "convergence".to_string(),
                ],
                record_trajectories: false,
            },
            output: OutputConfig {
                format: OutputFormat::Json,
                destination: "output/factorial".to_string(),
                compress: true,
                summary_metrics: vec!["fitness".to_string(), "convergence".to_string()],
                visualization: Some(VisualizationOutput {
                    trajectories: false,
                    time_series: false,
                    heatmaps: true,
                    comparisons: true,
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                }),
            },
            constraints: ExecutionConstraints {
                max_duration_secs: 7200,
                max_parallelism: 12,
                memory_limit_mb: 4096,
                retry_failed: true,
                max_retries: 3,
            },
        }
    }

    /// Stress test template
    fn stress_test() -> PatternConfig {
        PatternConfig {
            pattern: ExperimentPattern::ParameterSweep(SweepConfig {
                parameters: vec![SweepParameter {
                    name: "agent_count".to_string(),
                    param_type: ParameterType::Integer,
                    start: 100.0,
                    end: 10000.0,
                    step: Some(500.0),
                    values: None,
                }],
                replications: 3,
                seed: Some(42),
                sampling: SamplingMethod::Grid,
                sample_count: None,
            }),
            global_params: GlobalParams {
                ticks: 500,
                speed: 1.0,
                record_interval: 10,
                metrics: vec![
                    "throughput".to_string(),
                    "latency".to_string(),
                    "memory_usage".to_string(),
                    "cpu_usage".to_string(),
                ],
                record_trajectories: false,
            },
            // TAG: surface=simulation owner=research-team rule=SIM-001
            output: OutputConfig {
                format: OutputFormat::Json,
                destination: "output/stress_test".to_string(),
                compress: false,
                summary_metrics: vec![
                    "throughput".to_string(),
                    "latency".to_string(),
                    "memory_usage".to_string(),
                ],
                visualization: Some(VisualizationOutput {
                    trajectories: false,
                    time_series: true,
                    heatmaps: false,
                    comparisons: true,
                }),
            },
            constraints: ExecutionConstraints {
                max_duration_secs: 3600,
                max_parallelism: 4,
                memory_limit_mb: 8192,
                retry_failed: true,
                max_retries: 1,
            },
        }
    }

    /// Scenario comparison template
    fn scenario_comparison() -> PatternConfig {
        let mut base_params = HashMap::new();
        base_params.insert("environment".to_string(), serde_json::json!("stable"));
        base_params.insert("resource_abundance".to_string(), serde_json::json!(1.0));

        let mut drought_changes = HashMap::new();
        drought_changes.insert("environment".to_string(), serde_json::json!("drought"));
        drought_changes.insert("resource_abundance".to_string(), serde_json::json!(0.3));

        let mut abundance_changes = HashMap::new();
        abundance_changes.insert("environment".to_string(), serde_json::json!("abundant"));
        abundance_changes.insert("resource_abundance".to_string(), serde_json::json!(2.0));

        PatternConfig {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            pattern: ExperimentPattern::Scenario(ScenarioConfig {
                base: ExperimentConfig {
                    name: "Baseline".to_string(),
                    description: Some("Normal conditions".to_string()),
                    parameters: base_params,
                    agent_config: Some(AgentConfig {
                        count: 100,
                        agent_type: "forager".to_string(),
                        initial_properties: HashMap::new(),
                    }),
                    environment_config: None,
                },
                scenarios: vec![
                    ScenarioVariant {
                        id: "drought".to_string(),
                        name: "Drought Scenario".to_string(),
                        description: Some("Resource scarcity".to_string()),
                        changes: drought_changes,
                        trigger_tick: Some(1000),
                        duration: Some(1000),
                    },
                    ScenarioVariant {
                        id: "abundance".to_string(),
                        name: "Abundance Scenario".to_string(),
                        description: Some("Resource surplus".to_string()),
                        changes: abundance_changes,
                        trigger_tick: Some(1000),
                        duration: Some(1000),
                    },
                ],
                replications: 10,
                seed: Some(42),
                transitions: None,
            }),
            global_params: GlobalParams {
                ticks: 3000,
                speed: 1.0,
                record_interval: 50,
                metrics: vec![
                    "population".to_string(),
                    "resource_consumption".to_string(),
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    "survival_rate".to_string(),
                ],
                record_trajectories: true,
            },
            output: OutputConfig {
                format: OutputFormat::Json,
                destination: "output/scenarios".to_string(),
                compress: true,
                summary_metrics: vec!["population".to_string(), "survival_rate".to_string()],
                visualization: Some(VisualizationOutput {
                    trajectories: true,
                    time_series: true,
                    heatmaps: false,
                    comparisons: true,
                }),
            },
            constraints: ExecutionConstraints {
                max_duration_secs: 3600,
                max_parallelism: 8,
                memory_limit_mb: 4096,
                retry_failed: true,
                max_retries: 2,
            },
        }
    }

    /// Baseline comparison template
    fn baseline_comparison() -> PatternConfig {
        let mut baseline_params = HashMap::new();
        baseline_params.insert("strategy".to_string(), serde_json::json!("baseline"));

        let mut alt1_params = HashMap::new();
        alt1_params.insert("strategy".to_string(), serde_json::json!("aggressive"));

        let mut alt2_params = HashMap::new();
        alt2_params.insert("strategy".to_string(), serde_json::json!("cooperative"));

        PatternConfig {
            pattern: ExperimentPattern::Baseline(BaselineConfig {
                baseline: ExperimentConfig {
                    name: "Baseline".to_string(),
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    description: Some("Default strategy".to_string()),
                    parameters: baseline_params,
                    agent_config: None,
                    environment_config: None,
                },
                alternatives: vec![
                    ExperimentConfig {
                        name: "Aggressive".to_string(),
                        description: Some("Aggressive strategy".to_string()),
                        parameters: alt1_params,
                        agent_config: None,
                        environment_config: None,
                    },
                    ExperimentConfig {
                        name: "Cooperative".to_string(),
                        description: Some("Cooperative strategy".to_string()),
                        parameters: alt2_params,
                        agent_config: None,
                        environment_config: None,
                    },
                ],
                replications: 20,
                seed: Some(42),
                metrics: vec![
                    "success_rate".to_string(),
                    "avg_score".to_string(),
                    "completion_time".to_string(),
                ],
                normalize: true,
            }),
            global_params: GlobalParams {
                ticks: 2000,
                speed: 1.0,
                record_interval: 50,
                metrics: vec![
                    "success_rate".to_string(),
                    "avg_score".to_string(),
                    "completion_time".to_string(),
                ],
                record_trajectories: false,
            },
            // TAG: surface=simulation owner=research-team rule=SIM-001
            output: OutputConfig {
                format: OutputFormat::Json,
                destination: "output/baseline".to_string(),
                compress: false,
                summary_metrics: vec!["success_rate".to_string(), "avg_score".to_string()],
                visualization: Some(VisualizationOutput {
                    trajectories: false,
                    time_series: true,
                    heatmaps: false,
                    comparisons: true,
                }),
            },
            constraints: ExecutionConstraints {
                max_duration_secs: 3600,
                max_parallelism: 8,
                memory_limit_mb: 2048,
                retry_failed: true,
                max_retries: 2,
            },
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Template information for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// id
    pub id: String,
    /// name
    pub name: String,
    /// description
    pub description: String,
    /// category
    pub category: TemplateCategory,
    /// pattern type
    pub pattern_type: String,
}

/// Template category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    /// Exploration
    Exploration,
    /// Comparison
    Comparison,
    /// Performance
    Performance,
    /// Validation
    Validation,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

/// Pattern validation
#[derive(Debug)]
pub struct PatternValidator;

impl PatternValidator {
    /// Validate a pattern configuration
    #[must_use]
    pub fn validate(config: &PatternConfig) -> Vec<ValidationMessage> {
        let mut messages = Vec::new();

        // Validate pattern-specific settings
        match &config.pattern {
            ExperimentPattern::ParameterSweep(sweep) => {
                if sweep.parameters.is_empty() {
                    messages.push(ValidationMessage {
                        level: ValidationLevel::Error,
                        field: "parameters".to_string(),
                        message: "Parameter sweep must have at least one parameter".to_string(),
                    });
                }

                for param in &sweep.parameters {
                    if param.start >= param.end {
                        messages.push(ValidationMessage {
                            level: ValidationLevel::Error,
                            field: format!("parameters.{}", param.name),
                            message: "Start value must be less than end value".to_string(),
                        });
                    }
                }
                // TAG: surface=simulation owner=research-team rule=SIM-001
            }
            ExperimentPattern::ABTest(ab) => {
                if ab.replications < 10 {
                    messages.push(ValidationMessage {
                        level: ValidationLevel::Warning,
                        field: "replications".to_string(),
                        message:
                            "A/B test should have at least 10 replications for statistical power"
                                .to_string(),
                    });
                }
            }
            ExperimentPattern::Factorial(factorial) => {
                let total_runs: usize = factorial
                    .factors
                    .iter()
                    .map(|f| f.levels.len())
                    .product::<usize>()
                    * factorial.replications as usize;

                if total_runs > 1000 {
                    messages.push(ValidationMessage {
                        level: ValidationLevel::Warning,
                        field: "factors".to_string(),
                        message: format!(
                            "Full factorial design will generate {total_runs} runs. Consider fractional factorial."
                        ),
                    });
                }
            }
            _ => {}
        }

        // Validate global params
        if config.global_params.ticks == 0 {
            messages.push(ValidationMessage {
                level: ValidationLevel::Error,
                field: "global_params.ticks".to_string(),
                message: "Ticks must be greater than 0".to_string(),
            });
        }

        messages
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMessage {
    /// level
    pub level: ValidationLevel,
    /// field
    pub field: String,
    /// message
    pub message: String,
}

/// Validation level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationLevel {
    /// Error
    Error,
    /// Warning
    Warning,
    /// Info
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_templates_returns_expected_count() {
        let templates = PatternTemplates::all();
        assert_eq!(templates.len(), 7);
    }

    #[test]
    fn test_all_templates_have_unique_ids() {
        let templates = PatternTemplates::all();
        let ids: Vec<_> = templates.iter().map(|t| &t.id).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_get_existing_templates() {
        for template in PatternTemplates::all() {
            let config = PatternTemplates::get(&template.id);
            assert!(
                config.is_some(),
                "template {} should be retrievable",
                template.id
            );
        }
    }

    #[test]
    fn test_get_unknown_template_returns_none() {
        assert!(PatternTemplates::get("nonexistent").is_none());
    }

    #[test]
    fn test_basic_sweep_config() {
        let config = PatternTemplates::get("basic_sweep").unwrap();
        assert!(matches!(
            config.pattern,
            ExperimentPattern::ParameterSweep(_)
        ));
        assert_eq!(config.global_params.ticks, 1000);
        assert_eq!(config.constraints.max_parallelism, 4);
    }

    #[test]
    fn test_grid_search_config() {
        let config = PatternTemplates::get("grid_search").unwrap();
        assert!(matches!(
            config.pattern,
            ExperimentPattern::ParameterSweep(_)
        ));
        assert_eq!(config.global_params.ticks, 2000);
        assert!(config.output.compress);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_ab_test_config() {
        let config = PatternTemplates::get("ab_test").unwrap();
        assert!(matches!(config.pattern, ExperimentPattern::ABTest(_)));
        assert_eq!(config.global_params.ticks, 5000);
    }

    #[test]
    fn test_full_factorial_config() {
        let config = PatternTemplates::get("full_factorial").unwrap();
        assert!(matches!(config.pattern, ExperimentPattern::Factorial(_)));
        assert_eq!(config.global_params.ticks, 3000);
    }

    #[test]
    fn test_stress_test_config() {
        let config = PatternTemplates::get("stress_test").unwrap();
        assert!(matches!(
            config.pattern,
            ExperimentPattern::ParameterSweep(_)
        ));
        assert_eq!(config.global_params.ticks, 500);
    }

    #[test]
    fn test_scenario_comparison_config() {
        let config = PatternTemplates::get("scenario_comparison").unwrap();
        assert!(matches!(config.pattern, ExperimentPattern::Scenario(_)));
        assert_eq!(config.global_params.ticks, 3000);
        assert!(config.global_params.record_trajectories);
    }

    #[test]
    fn test_baseline_comparison_config() {
        let config = PatternTemplates::get("baseline_comparison").unwrap();
        assert!(matches!(config.pattern, ExperimentPattern::Baseline(_)));
        assert_eq!(config.global_params.ticks, 2000);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_validate_valid_sweep() {
        let config = PatternTemplates::get("basic_sweep").unwrap();
        let messages = PatternValidator::validate(&config);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_validate_ab_test_with_low_replications() {
        let mut config = PatternTemplates::get("ab_test").unwrap();
        if let ExperimentPattern::ABTest(ref mut ab) = config.pattern {
            ab.replications = 5;
        }
        let messages = PatternValidator::validate(&config);
        assert!(messages.iter().any(|m| m.field == "replications"));
    }

    #[test]
    fn test_validate_factorial_with_many_runs() {
        let mut config = PatternTemplates::get("full_factorial").unwrap();
        if let ExperimentPattern::Factorial(ref mut factorial) = config.pattern {
            // Add many levels to trigger the warning (>1000 total runs)
            // Original: 3 * 2 * 3 * 5 = 90 runs
            // With 20 extra levels: 3 * 2 * 3 * 20 * 5 = 1800 runs
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            factorial.factors.push(Factor {
                name: "extra".to_string(),
                levels: (1..=20).map(|i| serde_json::json!(i)).collect(),
            });
        }
        let messages = PatternValidator::validate(&config);
        assert!(messages.iter().any(|m| m.field == "factors"));
    }

    #[test]
    fn test_validate_zero_ticks() {
        let mut config = PatternTemplates::get("basic_sweep").unwrap();
        config.global_params.ticks = 0;
        let messages = PatternValidator::validate(&config);
        assert!(messages.iter().any(|m| m.field == "global_params.ticks"));
    }

    #[test]
    fn test_template_categories() {
        let templates = PatternTemplates::all();
        for t in &templates {
            assert!(!t.id.is_empty());
            assert!(!t.name.is_empty());
            assert!(!t.description.is_empty());
            assert!(!t.pattern_type.is_empty());
        }
    }
}
