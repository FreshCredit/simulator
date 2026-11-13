//! Experiment Pattern Types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of macro experiment patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExperimentPattern {
    /// Parameter sweep across one or more dimensions
    ParameterSweep(SweepConfig),
    /// A/B test comparing two variants
    ABTest(Box<ABTestConfig>),
    /// Full factorial design
    Factorial(FactorialConfig),
    /// Scenario-based experiment
    Scenario(ScenarioConfig),
    /// Baseline comparison against control
    Baseline(BaselineConfig),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for parameter sweep
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SweepConfig {
    /// Parameters to sweep
    pub parameters: Vec<SweepParameter>,
    /// Number of replications per configuration
    pub replications: u32,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Whether to use grid search or random sampling
    pub sampling: SamplingMethod,
    /// Number of samples (for random sampling)
    pub sample_count: Option<usize>,
}

/// A single parameter to sweep
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SweepParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Start value
    pub start: f64,
    /// End value
    pub end: f64,
    /// Step size (for numeric)
    pub step: Option<f64>,
    /// Discrete values (if applicable)
    pub values: Option<Vec<String>>,
}

/// Parameter type for sweep
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    /// Integer
    Integer,
    /// Float
    Float,
    /// Boolean
    Boolean,
    /// String
    String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sampling method for sweeps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SamplingMethod {
    /// Grid search - all combinations
    Grid,
    /// Random sampling
    Random,
    /// Latin hypercube sampling
    LatinHypercube,
    /// Sobol sequence for quasi-random
    Sobol,
}

/// Configuration for A/B test
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ABTestConfig {
    /// Control configuration
    pub control: ExperimentConfig,
    /// Treatment configuration
    pub treatment: ExperimentConfig,
    /// Number of replications per variant
    pub replications: u32,
    /// Random seed
    pub seed: Option<u64>,
    /// Metrics to compare
    pub metrics: Vec<String>,
    /// Statistical test to use
    pub statistical_test: StatisticalTest,
    /// Significance level
    pub alpha: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Experiment configuration for a variant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentConfig {
    /// Variant name
    pub name: String,
    /// Variant description
    pub description: Option<String>,
    /// Parameter values
    pub parameters: HashMap<String, serde_json::Value>,
    /// Agent configuration
    pub agent_config: Option<AgentConfig>,
    /// Environment configuration
    pub environment_config: Option<EnvironmentConfig>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentConfig {
    /// Number of agents
    pub count: usize,
    /// Agent type
    pub agent_type: String,
    /// Initial properties
    pub initial_properties: HashMap<String, serde_json::Value>,
}

/// Environment configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentConfig {
    /// Map/terrain configuration
    pub terrain: Option<String>,
    /// Resource distribution
    pub resources: Vec<ResourceConfig>,
    /// Weather/climate settings
    pub climate: Option<ClimateConfig>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Resource configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceConfig {
    /// resource type
    pub resource_type: String,
    /// distribution
    pub distribution: String,
    /// density
    pub density: f64,
}

/// Climate configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClimateConfig {
    /// base temperature
    pub base_temperature: f64,
    /// seasonality
    pub seasonality: f64,
    /// variability
    pub variability: f64,
}

/// Statistical test for comparisons
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatisticalTest {
    /// T-test for means
    TTest,
    /// Mann-Whitney U test (non-parametric)
    MannWhitney,
    /// Kolmogorov-Smirnov test for distributions
    KolmogorovSmirnov,
    /// Permutation test
    Permutation,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for factorial design
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactorialConfig {
    /// Factors to vary
    pub factors: Vec<Factor>,
    /// Number of replications per cell
    pub replications: u32,
    /// Random seed
    pub seed: Option<u64>,
    /// Whether to use full or fractional factorial
    pub design: FactorialDesign,
    /// Resolution (for fractional factorial)
    pub resolution: Option<u32>,
}

/// A factor in factorial design
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Factor {
    /// Factor name
    pub name: String,
    /// Factor levels
    pub levels: Vec<serde_json::Value>,
}

/// Factorial design type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FactorialDesign {
    /// Full factorial (all combinations)
    Full,
    /// Fractional factorial
    Fractional,
}

/// Configuration for scenario-based experiment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScenarioConfig {
    /// Base scenario
    pub base: ExperimentConfig,
    /// Alternative scenarios
    pub scenarios: Vec<ScenarioVariant>,
    /// Number of replications per scenario
    pub replications: u32,
    /// Random seed
    pub seed: Option<u64>,
    /// Transition rules between scenarios
    pub transitions: Option<Vec<ScenarioTransition>>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// A scenario variant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioVariant {
    /// Scenario ID
    pub id: String,
    /// Scenario name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Changes from base
    pub changes: HashMap<String, serde_json::Value>,
    /// When this scenario triggers (tick)
    pub trigger_tick: Option<u64>,
    /// Duration in ticks
    pub duration: Option<u64>,
}

/// Transition between scenarios
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScenarioTransition {
    /// From scenario
    pub from: String,
    /// To scenario
    pub to: String,
    /// Condition for transition
    pub condition: TransitionCondition,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Condition for scenario transition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransitionCondition {
    /// Time-based transition
    Tick {
        /// Tick count threshold
        tick: u64,
    },
    /// Metric-based transition
    Metric {
        /// Metric name
        metric: String,
        /// Threshold value
        threshold: f64,
        /// Comparison operator
        operator: String,
    },
    /// Event-based transition
    Event {
        /// Event type
        event_type: String,
    },
}

/// Configuration for baseline comparison
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaselineConfig {
    /// Baseline (control) configuration
    pub baseline: ExperimentConfig,
    /// Alternative configurations to compare
    pub alternatives: Vec<ExperimentConfig>,
    /// Number of replications
    pub replications: u32,
    /// Random seed
    pub seed: Option<u64>,
    /// Metrics to compare
    pub metrics: Vec<String>,
    /// Whether to normalize to baseline
    pub normalize: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Pattern execution configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatternConfig {
    /// Pattern type and configuration
    pub pattern: ExperimentPattern,
    /// Global experiment parameters
    pub global_params: GlobalParams,
    /// Output configuration
    pub output: OutputConfig,
    /// Execution constraints
    pub constraints: ExecutionConstraints,
}

/// Global parameters for pattern execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalParams {
    /// Number of simulation ticks
    pub ticks: u64,
    /// Simulation speed
    pub speed: f64,
    /// Recording interval
    pub record_interval: u32,
    /// Metrics to collect
    pub metrics: Vec<String>,
    /// Whether to record full trajectories
    pub record_trajectories: bool,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputConfig {
    /// Output format
    pub format: OutputFormat,
    /// Destination path/URL
    pub destination: String,
    /// Whether to compress output
    pub compress: bool,
    /// Metrics to include in summary
    pub summary_metrics: Vec<String>,
    /// Visualization options
    pub visualization: Option<VisualizationOutput>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Output format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// Json
    Json,
    /// Csv
    Csv,
    /// Parquet
    Parquet,
    /// Hdf5
    Hdf5,
}

#[allow(clippy::struct_excessive_bools)]
/// Visualization output options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualizationOutput {
    /// Generate trajectory plots
    pub trajectories: bool,
    /// Generate metric time series
    pub time_series: bool,
    /// Generate heatmaps
    pub heatmaps: bool,
    /// Generate comparison plots
    pub comparisons: bool,
}

/// Execution constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionConstraints {
    /// Maximum wall clock time (seconds)
    pub max_duration_secs: u64,
    /// Maximum number of parallel runs
    pub max_parallelism: usize,
    /// Memory limit per run (MB)
    pub memory_limit_mb: usize,
    /// Retry failed runs
    pub retry_failed: bool,
    /// Maximum retries per run
    pub max_retries: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ExecutionConstraints {
    fn default() -> Self {
        Self {
            max_duration_secs: 3600,
            max_parallelism: 4,
            memory_limit_mb: 4096,
            retry_failed: true,
            max_retries: 3,
        }
    }
}

/// Result of pattern execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternResult {
    /// Pattern ID
    pub pattern_id: String,
    /// Pattern type
    pub pattern_type: String,
    /// Execution status
    pub status: ExecutionStatus,
    /// Total number of runs
    pub total_runs: usize,
    /// Completed runs
    pub completed_runs: usize,
    /// Failed runs
    pub failed_runs: usize,
    /// Individual run results
    pub run_results: Vec<RunResult>,
    /// Aggregated results by configuration
    pub aggregated: Vec<AggregatedResult>,
    /// Statistical comparisons (if applicable)
    pub comparisons: Option<Vec<ComparisonResult>>,
    /// Start time
    pub started_at: String,
    /// End time
    pub completed_at: Option<String>,
    /// Total execution time (seconds)
    pub execution_time_secs: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    /// Pending
    Pending,
    /// Running
    Running,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

/// Result of a single run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    /// Run ID
    pub run_id: String,
    /// Configuration used
    pub configuration: HashMap<String, serde_json::Value>,
    /// Status
    pub status: RunStatus,
    /// Metrics collected
    pub metrics: HashMap<String, f64>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution time (seconds)
    pub execution_time_secs: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Run status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// Pending
    Pending,
    /// Running
    Running,
    /// Completed
    Completed,
    /// Failed
    Failed,
}

/// Aggregated results for a configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    /// Configuration identifier
    pub config_id: String,
    /// Configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Number of successful runs
    pub run_count: usize,
    /// Mean values for each metric
    pub means: HashMap<String, f64>,
    /// Standard deviations
    pub std_devs: HashMap<String, f64>,
    /// Confidence intervals (95%)
    pub confidence_intervals: HashMap<String, (f64, f64)>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Comparison result between configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Comparison name
    pub name: String,
    /// Base configuration
    pub base_config: String,
    /// Comparison configuration
    pub compare_config: String,
    /// Metric comparisons
    pub metric_comparisons: Vec<MetricComparisonResult>,
}

/// Metric comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricComparisonResult {
    /// Metric name
    pub metric: String,
    /// Base mean
    pub base_mean: f64,
    /// Compare mean
    pub compare_mean: f64,
    /// Absolute difference
    pub diff: f64,
    /// Percentage difference
    pub percent_diff: f64,
    /// P-value from statistical test
    pub p_value: f64,
    /// Whether difference is significant
    pub significant: bool,
    /// Effect size (Cohen's d)
    pub effect_size: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Pattern execution request
#[derive(Debug, Deserialize)]
pub struct ExecutePatternRequest {
    /// name
    pub name: String,
    /// description
    pub description: Option<String>,
    /// config
    pub config: PatternConfig,
}

/// Pattern execution response
#[derive(Debug, Serialize)]
pub struct ExecutePatternResponse {
    /// pattern id
    pub pattern_id: String,
    /// status
    pub status: String,
    /// estimated runs
    pub estimated_runs: usize,
    /// estimated duration secs
    pub estimated_duration_secs: u64,
}

/// Pattern status response
#[derive(Debug, Serialize)]
pub struct PatternStatusResponse {
    /// pattern id
    pub pattern_id: String,
    /// status
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub status: String,
    /// progress percent
    pub progress_percent: f64,
    /// completed runs
    pub completed_runs: usize,
    /// total runs
    pub total_runs: usize,
    /// current run
    pub current_run: Option<String>,
    /// eta secs
    pub eta_secs: Option<u64>,
}

/// Pattern summary for listing
#[derive(Debug, Serialize)]
pub struct PatternSummary {
    /// pattern id
    pub pattern_id: String,
    /// name
    pub name: String,
    /// pattern type
    pub pattern_type: String,
    /// status
    pub status: String,
    /// created at
    pub created_at: String,
    /// completed at
    pub completed_at: Option<String>,
    /// total runs
    pub total_runs: usize,
}
