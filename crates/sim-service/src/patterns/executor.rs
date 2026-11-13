//! Pattern Executor - Executes macro experiment patterns

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Executes experiment patterns
#[derive(Debug)]
pub struct PatternExecutor {
    /// Active pattern executions
    executions: Arc<RwLock<HashMap<String, PatternExecution>>>,
    /// Execution counter for IDs
    execution_counter: Arc<RwLock<u64>>,
}

/// Internal execution state
#[derive(Clone, Debug)]
struct PatternExecution {
    pattern_id: String,
    config: PatternConfig,
    status: ExecutionStatus,
    run_results: Vec<RunResult>,
    started_at: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl PatternExecutor {
    /// Create a new pattern executor
    #[must_use]
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
            execution_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Execute a pattern
    pub async fn execute(&self, request: ExecutePatternRequest) -> ExecutePatternResponse {
        // Generate pattern ID
        let pattern_id = {
            let mut counter = self.execution_counter.write().await;
            *counter += 1;
            format!("pattern-{}-{}", *counter, uuid::Uuid::new_v4())
        };

        // Generate all run configurations
        let run_configs = self.generate_run_configs(&request.config);
        let estimated_runs = run_configs.len();

        info!(
            pattern_id = %pattern_id,
            pattern_type = ?request.config.pattern,
            estimated_runs = %estimated_runs,
            "Starting pattern execution"
        );

        // Create execution record
        let execution = PatternExecution {
            pattern_id: pattern_id.clone(),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            config: request.config.clone(),
            status: ExecutionStatus::Pending,
            run_results: Vec::new(),
            started_at: chrono::Utc::now().to_rfc3339(),
        };

        {
            let mut executions = self.executions.write().await;
            executions.insert(pattern_id.clone(), execution);
        }

        // Start execution in background
        let executor_clone = self.clone();
        let pattern_id_clone = pattern_id.clone();
        let config_clone = request.config.clone();

        tokio::spawn(async move {
            executor_clone
                .run_executions(pattern_id_clone, config_clone, run_configs)
                .await;
        });

        // Estimate duration
        let estimated_duration_secs =
            (estimated_runs as u64) * request.config.global_params.ticks / 10; // Rough estimate

        ExecutePatternResponse {
            pattern_id,
            status: "running".to_string(),
            estimated_runs,
            estimated_duration_secs,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get pattern execution status
    pub async fn get_status(&self, pattern_id: &str) -> Option<PatternStatusResponse> {
        let executions = self.executions.read().await;
        executions.get(pattern_id).map(|exec| {
            let completed = exec
                .run_results
                .iter()
                .filter(|r| r.status == RunStatus::Completed)
                .count();
            let total = exec.run_results.len();
            let progress = if total > 0 {
                (completed as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            PatternStatusResponse {
                pattern_id: pattern_id.to_string(),
                status: format!("{:?}", exec.status).to_lowercase(),
                progress_percent: progress,
                completed_runs: completed,
                total_runs: total,
                current_run: None, // Would track in real implementation
                eta_secs: None,    // Would calculate in real implementation
            }
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get pattern execution result
    pub async fn get_result(&self, pattern_id: &str) -> Option<PatternResult> {
        let executions = self.executions.read().await;
        executions
            .get(pattern_id)
            .map(|exec| self.build_result(exec))
    }

    /// List all pattern executions
    pub async fn list_executions(&self) -> Vec<PatternSummary> {
        let executions = self.executions.read().await;
        let mut summaries: Vec<PatternSummary> = executions
            .values()
            .map(|exec| PatternSummary {
                pattern_id: exec.pattern_id.clone(),
                name: "Pattern".to_string(), // Would store name in execution
                pattern_type: self.get_pattern_type_name(&exec.config.pattern),
                status: format!("{:?}", exec.status).to_lowercase(),
                created_at: exec.started_at.clone(),
                completed_at: None, // Would track in real implementation
                total_runs: exec.run_results.len(),
            })
            .collect();
        summaries.sort_by(|a, b| a.pattern_id.cmp(&b.pattern_id));
        summaries
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Cancel a pattern execution
    pub async fn cancel(&self, pattern_id: &str) -> bool {
        let mut executions = self.executions.write().await;
        if let Some(exec) = executions.get_mut(pattern_id) {
            exec.status = ExecutionStatus::Cancelled;
            info!(pattern_id = %pattern_id, "Pattern execution cancelled");
            true
        } else {
            false
        }
    }

    /// Generate run configurations based on pattern type
    fn generate_run_configs(
        &self,
        config: &PatternConfig,
    ) -> Vec<HashMap<String, serde_json::Value>> {
        match &config.pattern {
            ExperimentPattern::ParameterSweep(sweep) => self.generate_sweep_configs(sweep),
            ExperimentPattern::ABTest(ab) => self.generate_ab_configs(ab),
            ExperimentPattern::Factorial(factorial) => self.generate_factorial_configs(factorial),
            ExperimentPattern::Scenario(scenario) => self.generate_scenario_configs(scenario),
            ExperimentPattern::Baseline(baseline) => self.generate_baseline_configs(baseline),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
    }

    /// Generate sweep configurations
    fn generate_sweep_configs(
        &self,
        config: &SweepConfig,
    ) -> Vec<HashMap<String, serde_json::Value>> {
        let mut configs;
        match config.sampling {
            SamplingMethod::Grid => {
                // Generate grid of all combinations
                configs = self.generate_grid_configs(&config.parameters);
            }
            SamplingMethod::Random => {
                // Generate random samples
                let count = config.sample_count.unwrap_or(10);
                configs = self.generate_random_configs(&config.parameters, count, config.seed);
            }
            _ => {
                // Default to grid for other methods
                configs = self.generate_grid_configs(&config.parameters);
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Expand for replications
        if config.replications > 1 {
            let mut replicated = Vec::new();
            for (i, cfg) in configs.iter().enumerate() {
                for rep in 0..config.replications {
                    let mut cfg_with_rep = cfg.clone();
                    cfg_with_rep.insert("replication".to_string(), serde_json::json!(rep));
                    cfg_with_rep.insert(
                        "config_id".to_string(),
                        serde_json::json!(format!("{}-{}", i, rep)),
                    );
                    replicated.push(cfg_with_rep);
                }
            }
            configs = replicated;
        }

        configs
    }

    /// Generate grid configurations with exact index quantization to prevent cumulative float drift
    fn generate_grid_configs(
        &self,
        parameters: &[SweepParameter],
    ) -> Vec<HashMap<String, serde_json::Value>> {
        let mut configs = vec![HashMap::new()];

        for param in parameters {
            let mut new_configs = Vec::new();

            for cfg in &configs {
                // Generate values for this parameter
                let values = if let Some(ref discrete) = param.values {
                    discrete.iter().map(|v| serde_json::json!(v)).collect()
                } else {
                    let step = param.step.unwrap_or(1.0);
                    let mut vals = Vec::new();
                    if step > 0.0 && param.start <= param.end {
                        let span = param.end - param.start;
                        let n_steps = ((span / step).round() as usize).min(10_000);
                        for k in 0..=n_steps {
                            let val = param.start + (k as f64) * step;
                            match param.param_type {
                                ParameterType::Integer => {
                                    vals.push(serde_json::json!(val.round() as i64));
                                }
                                ParameterType::Boolean => vals.push(serde_json::json!(val != 0.0)),
                                _ => vals.push(serde_json::json!(val)),
                            }
                        }
                    } else {
                        match param.param_type {
                            ParameterType::Integer => {
                                vals.push(serde_json::json!(param.start as i64));
                            }
                            ParameterType::Boolean => {
                                vals.push(serde_json::json!(param.start != 0.0));
                            }
                            _ => vals.push(serde_json::json!(param.start)),
                        }
                    }
                    vals
                };

                // TAG: surface=simulation owner=research-team rule=SIM-001
                // Create new config for each value
                for v in values {
                    let mut new_cfg = cfg.clone();
                    new_cfg.insert(param.name.clone(), v);
                    new_configs.push(new_cfg);
                }
            }

            configs = new_configs;
        }

        configs
    }

    /// Generate random configurations with panic bounds guards and deterministic default seed (SIM-004)
    fn generate_random_configs(
        &self,
        parameters: &[SweepParameter],
        count: usize,
        seed: Option<u64>,
    ) -> Vec<HashMap<String, serde_json::Value>> {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::seed_from_u64(0),
        };

        let mut configs = Vec::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for _ in 0..count {
            let mut cfg = HashMap::new();
            for param in parameters {
                let (min_val, max_val) = if param.start <= param.end {
                    (param.start, param.end)
                } else {
                    (param.end, param.start)
                };
                let value = match param.param_type {
                    ParameterType::Integer => {
                        let val = rng.gen_range(min_val as i64..=(max_val as i64));
                        serde_json::json!(val)
                    }
                    ParameterType::Float => {
                        let val = rng.gen_range(min_val..=max_val);
                        serde_json::json!(val)
                    }
                    ParameterType::Boolean => {
                        serde_json::json!(rng.gen_bool(0.5))
                    }
                    ParameterType::String => {
                        if let Some(ref values) = param.values {
                            if !values.is_empty() {
                                let idx = rng.gen_range(0..values.len());
                                serde_json::json!(values[idx])
                            } else {
                                serde_json::json!("")
                            }
                        } else {
                            serde_json::json!("")
                        }
                    }
                };
                cfg.insert(param.name.clone(), value);
            }
            configs.push(cfg);
        }

        configs
    }

    /// Generate A/B test configurations
    fn generate_ab_configs(
        &self,
        config: &ABTestConfig,
    ) -> Vec<HashMap<String, serde_json::Value>> {
        let mut configs = Vec::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Control runs
        for rep in 0..config.replications {
            let mut cfg = HashMap::new();
            cfg.insert("variant".to_string(), serde_json::json!("control"));
            cfg.insert(
                "variant_name".to_string(),
                serde_json::json!(&config.control.name),
            );
            cfg.insert("replication".to_string(), serde_json::json!(rep));
            cfg.insert(
                "parameters".to_string(),
                serde_json::json!(&config.control.parameters),
            );
            configs.push(cfg);
        }

        // Treatment runs
        for rep in 0..config.replications {
            let mut cfg = HashMap::new();
            cfg.insert("variant".to_string(), serde_json::json!("treatment"));
            cfg.insert(
                "variant_name".to_string(),
                serde_json::json!(&config.treatment.name),
            );
            cfg.insert("replication".to_string(), serde_json::json!(rep));
            cfg.insert(
                "parameters".to_string(),
                serde_json::json!(&config.treatment.parameters),
            );
            configs.push(cfg);
        }

        configs
    }

    /// Generate factorial design configurations
    fn generate_factorial_configs(
        &self,
        config: &FactorialConfig,
    ) -> Vec<HashMap<String, serde_json::Value>> {
        let mut configs = vec![HashMap::new()];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for factor in &config.factors {
            let mut new_configs = Vec::new();
            for cfg in &configs {
                for (i, level) in factor.levels.iter().enumerate() {
                    let mut new_cfg = cfg.clone();
                    new_cfg.insert(factor.name.clone(), level.clone());
                    new_cfg.insert(format!("{}_level", factor.name), serde_json::json!(i));
                    new_configs.push(new_cfg);
                }
            }
            configs = new_configs;
        }

        // Add replications
        let mut replicated = Vec::new();
        for (i, cfg) in configs.iter().enumerate() {
            for rep in 0..config.replications {
                let mut cfg_with_rep = cfg.clone();
                cfg_with_rep.insert("replication".to_string(), serde_json::json!(rep));
                cfg_with_rep.insert(
                    "cell_id".to_string(),
                    serde_json::json!(format!("{}-{}", i, rep)),
                );
                replicated.push(cfg_with_rep);
            }
        }

        replicated
    }

    /// Generate scenario configurations
    fn generate_scenario_configs(
        &self,
        config: &ScenarioConfig,
    ) -> Vec<HashMap<String, serde_json::Value>> {
        let mut configs = Vec::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Base scenario
        for rep in 0..config.replications {
            let mut cfg = HashMap::new();
            cfg.insert("scenario".to_string(), serde_json::json!("base"));
            cfg.insert("replication".to_string(), serde_json::json!(rep));
            cfg.insert(
                "parameters".to_string(),
                serde_json::json!(&config.base.parameters),
            );
            configs.push(cfg);
        }

        // Alternative scenarios
        for scenario in &config.scenarios {
            for rep in 0..config.replications {
                let mut cfg = HashMap::new();
                cfg.insert("scenario".to_string(), serde_json::json!(&scenario.id));
                cfg.insert(
                    "scenario_name".to_string(),
                    serde_json::json!(&scenario.name),
                );
                cfg.insert("replication".to_string(), serde_json::json!(rep));

                // Merge base with changes
                let mut params = config.base.parameters.clone();
                params.extend(scenario.changes.clone());
                cfg.insert("parameters".to_string(), serde_json::json!(params));

                configs.push(cfg);
            }
        }

        configs
    }

    /// Generate baseline comparison configurations
    fn generate_baseline_configs(
        &self,
        config: &BaselineConfig,
    ) -> Vec<HashMap<String, serde_json::Value>> {
        let mut configs = Vec::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Baseline runs
        for rep in 0..config.replications {
            let mut cfg = HashMap::new();
            cfg.insert("variant".to_string(), serde_json::json!("baseline"));
            cfg.insert(
                "variant_name".to_string(),
                serde_json::json!(&config.baseline.name),
            );
            cfg.insert("replication".to_string(), serde_json::json!(rep));
            cfg.insert(
                "parameters".to_string(),
                serde_json::json!(&config.baseline.parameters),
            );
            configs.push(cfg);
        }

        // Alternative runs
        for alt in &config.alternatives {
            for rep in 0..config.replications {
                let mut cfg = HashMap::new();
                cfg.insert("variant".to_string(), serde_json::json!("alternative"));
                cfg.insert("variant_name".to_string(), serde_json::json!(&alt.name));
                cfg.insert("replication".to_string(), serde_json::json!(rep));
                cfg.insert("parameters".to_string(), serde_json::json!(&alt.parameters));
                configs.push(cfg);
            }
        }

        configs
    }

    /// Run all executions for a pattern
    async fn run_executions(
        // TAG: surface=simulation owner=research-team rule=SIM-001
        &self,
        pattern_id: String,
        _config: PatternConfig,
        run_configs: Vec<HashMap<String, serde_json::Value>>,
    ) {
        // Update status to running
        {
            let mut executions = self.executions.write().await;
            if let Some(exec) = executions.get_mut(&pattern_id) {
                exec.status = ExecutionStatus::Running;
            }
        }

        // Execute runs (simplified - would use batch executor in real implementation)
        for (i, cfg) in run_configs.iter().enumerate() {
            // Check if cancelled
            {
                let executions = self.executions.read().await;
                if let Some(exec) = executions.get(&pattern_id) {
                    if exec.status == ExecutionStatus::Cancelled {
                        break;
                    }
                }
            }

            // Create run result
            let run_result = RunResult {
                run_id: format!("{pattern_id}-run-{i}"),
                configuration: cfg.clone(),
                status: RunStatus::Completed,
                metrics: HashMap::new(), // Would be populated from actual run
                error: None,
                execution_time_secs: 1.0, // Placeholder
            };

            // Store result
            {
                let mut executions = self.executions.write().await;
                if let Some(exec) = executions.get_mut(&pattern_id) {
                    exec.run_results.push(run_result);
                }
            }

            // TAG: surface=simulation owner=research-team rule=SIM-001
            debug!(run_idx = %i, "Completed run");
        }

        // Update final status
        {
            let mut executions = self.executions.write().await;
            if let Some(exec) = executions.get_mut(&pattern_id) {
                if exec.status != ExecutionStatus::Cancelled {
                    exec.status = ExecutionStatus::Completed;
                }
            }
        }

        info!(pattern_id = %pattern_id, "Pattern execution completed");
    }

    /// Build pattern result from execution
    fn build_result(&self, exec: &PatternExecution) -> PatternResult {
        let completed = exec
            .run_results
            .iter()
            .filter(|r| r.status == RunStatus::Completed)
            .count();
        let failed = exec
            .run_results
            .iter()
            .filter(|r| r.status == RunStatus::Failed)
            .count();

        // Aggregate results by configuration
        let aggregated = self.aggregate_results(&exec.run_results);

        PatternResult {
            pattern_id: exec.pattern_id.clone(),
            pattern_type: self.get_pattern_type_name(&exec.config.pattern),
            // TAG: surface=simulation owner=research-team rule=SIM-001
            status: exec.status,
            total_runs: exec.run_results.len(),
            completed_runs: completed,
            failed_runs: failed,
            run_results: exec.run_results.clone(),
            aggregated,
            comparisons: None, // Would compute if A/B test
            started_at: exec.started_at.clone(),
            completed_at: Some(chrono::Utc::now().to_rfc3339()),
            execution_time_secs: 0.0, // Would calculate
        }
    }

    /// Aggregate results by configuration
    fn aggregate_results(&self, run_results: &[RunResult]) -> Vec<AggregatedResult> {
        let mut groups: std::collections::BTreeMap<String, Vec<&RunResult>> =
            std::collections::BTreeMap::new();
        for result in run_results {
            // Canonical sorted configuration representation ensures deterministic grouping (SIM-004)
            let sorted_cfg: std::collections::BTreeMap<_, _> =
                result.configuration.iter().collect();
            let key = serde_json::to_string(&sorted_cfg).unwrap_or_default();
            groups.entry(key).or_default().push(result);
        }

        groups
            .into_iter()
            .enumerate()
            .map(|(idx, (_, runs))| Self::aggregate_group(&runs, idx))
            .collect()
    }

    fn aggregate_group(runs: &[&RunResult], group_idx: usize) -> AggregatedResult {
        let params = runs
            .first()
            .map(|r| r.configuration.clone())
            .unwrap_or_default();
        let mut metric_names: Vec<String> = runs
            .iter()
            .flat_map(|r| r.metrics.keys().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        metric_names.sort(); // SIM-004: deterministic metric order

        let mut means = HashMap::new();
        let mut std_devs = HashMap::new();
        let mut confidence_intervals = HashMap::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for metric in metric_names {
            let values: Vec<f64> = runs
                .iter()
                .filter_map(|r| r.metrics.get(&metric).copied())
                .filter(|v| v.is_finite())
                .collect();
            if values.is_empty() {
                continue;
            }
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            means.insert(metric.clone(), mean);

            if values.len() > 1 {
                let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                    / (values.len() - 1) as f64;
                let std_dev = variance.sqrt();
                std_devs.insert(metric.clone(), std_dev);
                let margin = 1.96 * std_dev / (values.len() as f64).sqrt();
                confidence_intervals.insert(metric.clone(), (mean - margin, mean + margin));
            }
        }

        AggregatedResult {
            config_id: format!("config-{group_idx}"),
            parameters: params,
            run_count: runs.len(),
            means,
            std_devs,
            confidence_intervals,
        }
    }

    /// Get pattern type name
    fn get_pattern_type_name(&self, pattern: &ExperimentPattern) -> String {
        match pattern {
            ExperimentPattern::ParameterSweep(_) => "parameter_sweep".to_string(),
            ExperimentPattern::ABTest(_) => "ab_test".to_string(),
            ExperimentPattern::Factorial(_) => "factorial".to_string(),
            ExperimentPattern::Scenario(_) => "scenario".to_string(),
            ExperimentPattern::Baseline(_) => "baseline".to_string(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for PatternExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PatternExecutor {
    fn clone(&self) -> Self {
        Self {
            executions: Arc::clone(&self.executions),
            execution_counter: Arc::clone(&self.execution_counter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sweep_generation() {
        let executor = PatternExecutor::new();

        let sweep = SweepConfig {
            parameters: vec![
                SweepParameter {
                    name: "param1".to_string(),
                    param_type: ParameterType::Integer,
                    start: 1.0,
                    end: 3.0,
                    step: Some(1.0),
                    values: None,
                },
                SweepParameter {
                    name: "param2".to_string(),
                    param_type: ParameterType::Integer,
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    start: 10.0,
                    end: 20.0,
                    step: Some(10.0),
                    values: None,
                },
            ],
            replications: 2,
            seed: None,
            sampling: SamplingMethod::Grid,
            sample_count: None,
        };

        let configs = executor.generate_sweep_configs(&sweep);

        // 3 values for param1 * 2 values for param2 * 2 replications = 12
        assert_eq!(configs.len(), 12);
    }

    #[tokio::test]
    async fn test_ab_test_generation() {
        let executor = PatternExecutor::new();

        let ab = ABTestConfig {
            control: ExperimentConfig {
                name: "Control".to_string(),
                description: None,
                parameters: HashMap::new(),
                agent_config: None,
                environment_config: None,
            },
            treatment: ExperimentConfig {
                name: "Treatment".to_string(),
                description: None,
                parameters: HashMap::new(),
                agent_config: None,
                environment_config: None,
            },
            replications: 5,
            seed: None,
            metrics: vec!["metric1".to_string()],
            statistical_test: StatisticalTest::TTest,
            alpha: 0.05,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let configs = executor.generate_ab_configs(&ab);

        // 5 control + 5 treatment = 10
        assert_eq!(configs.len(), 10);
    }

    #[tokio::test]
    async fn test_factorial_generation() {
        let executor = PatternExecutor::new();

        let factorial = FactorialConfig {
            factors: vec![
                Factor {
                    name: "A".to_string(),
                    levels: vec![serde_json::json!("low"), serde_json::json!("high")],
                },
                Factor {
                    name: "B".to_string(),
                    levels: vec![serde_json::json!("low"), serde_json::json!("high")],
                },
            ],
            replications: 3,
            seed: None,
            design: FactorialDesign::Full,
            resolution: None,
        };

        let configs = executor.generate_factorial_configs(&factorial);

        // 2 * 2 * 3 = 12
        assert_eq!(configs.len(), 12);
    }

    #[tokio::test]
    async fn test_pattern_executor_clone_shared_execution() {
        let executor = PatternExecutor::new();
        let request = ExecutePatternRequest {
            name: "TestExecution".to_string(),
            description: None,
            config: PatternConfig {
                pattern: ExperimentPattern::Baseline(BaselineConfig {
                    baseline: ExperimentConfig {
                        name: "Baseline".to_string(),
                        description: None,
                        parameters: HashMap::new(),
                        agent_config: None,
                        environment_config: None,
                    },
                    alternatives: vec![],
                    replications: 1,
                    seed: Some(42),
                    metrics: vec!["metric".to_string()],
                    normalize: false,
                }),
                global_params: GlobalParams {
                    ticks: 10,
                    speed: 1.0,
                    record_interval: 1,
                    metrics: vec!["metric".to_string()],
                    record_trajectories: false,
                },
                output: OutputConfig {
                    format: OutputFormat::Json,
                    destination: "memory".to_string(),
                    compress: false,
                    summary_metrics: vec![],
                    visualization: None,
                },
                constraints: ExecutionConstraints::default(),
            },
        };

        let response = executor.execute(request).await;
        let pattern_id = response.pattern_id;

        // Poll for completion (background worker updates shared Arc state)
        let mut completed = false;
        for _ in 0..50 {
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            if let Some(status) = executor.get_status(&pattern_id).await {
                if status.status == "completed" {
                    completed = true;
                    break;
                }
            }
        }
        assert!(
            completed,
            "Background execution must complete on shared Arc state"
        );

        let result = executor.get_result(&pattern_id).await;
        assert!(
            result.is_some(),
            "Execution result must be accessible via shared state"
        );
    }

    #[tokio::test]
    async fn test_aggregate_results_canonical_grouping() {
        let executor = PatternExecutor::new();

        // Config 1 with insertion order: "a", then "b"
        let mut cfg1 = HashMap::new();
        cfg1.insert("a".to_string(), serde_json::json!(1));
        cfg1.insert("b".to_string(), serde_json::json!(2));

        // Config 2 with insertion order: "b", then "a"
        let mut cfg2 = HashMap::new();
        cfg2.insert("b".to_string(), serde_json::json!(2));
        cfg2.insert("a".to_string(), serde_json::json!(1));

        let mut metrics1 = HashMap::new();
        metrics1.insert("score".to_string(), 10.0);

        let mut metrics2 = HashMap::new();
        metrics2.insert("score".to_string(), 20.0);

        let run1 = RunResult {
            run_id: "r1".to_string(),
            configuration: cfg1,
            status: RunStatus::Completed,
            metrics: metrics1,
            error: None,
            execution_time_secs: 0.1,
        };
        let run2 = RunResult {
            run_id: "r2".to_string(),
            configuration: cfg2,
            status: RunStatus::Completed,
            metrics: metrics2,
            error: None,
            execution_time_secs: 0.1,
        };

        let aggregated = executor.aggregate_results(&[run1, run2]);
        // Must be aggregated into 1 single canonical group instead of 2 fractured groups
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].run_count, 2);
        assert_eq!(*aggregated[0].means.get("score").unwrap(), 15.0);
    }
}
