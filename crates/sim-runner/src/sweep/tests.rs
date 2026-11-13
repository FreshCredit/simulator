// TAG: surface=simulation owner=research-team rule=SIM-001
//! Tests for parameter sweep execution.

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::module_inception)]
mod tests {
    use super::super::grid::*;
    use super::super::monte_carlo::*;
    use super::super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_linear_values() {
        let values = SweepRunner::linear_values(0.0, 10.0, 5);
        assert_eq!(values.len(), 5);
        assert!((values[0] - 0.0).abs() < 1e-10);
        assert!((values[4] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_log_values() {
        let values = SweepRunner::log_values(1.0, 100.0, 3);
        assert_eq!(values.len(), 3);
        assert!((values[0] - 1.0).abs() < 1e-10);
        assert!((values[2] - 100.0).abs() < 1e-10);
        // Middle value should be ~10 (geometric mean)
        assert!((values[1] - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_sweep_runner_creation() {
        let config = RunConfig::default();
        let runner = SweepRunner::new(config);
        // Just test creation doesn't panic
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let _ = runner;
    }

    #[test]
    fn test_morris_screener_creation() {
        let config = RunConfig::default();
        let screener = MorrisScreener::new(config);
        assert!(!screener.parameters.is_empty());
    }
    #[test]
    fn test_morris_screener_trajectory_diversity() {
        let mut config = RunConfig::default();
        config.simulation.max_ticks = 5;
        config.simulation.agent_count = 5;
        let mut screener = MorrisScreener::new(config);
        screener.parameter_ranges.clear();
        screener.parameters.clear();
        screener.set_range("agent_count", 5.0, 10.0);
        let res = screener.screen(2, 4, 1).expect("screen");
        assert_eq!(res.trajectories, 2);
        assert_eq!(res.parameters.len(), 1);
    }

    // =========================================================================
    // Grid Sweep Tests
    // =========================================================================

    #[test]
    fn test_grid_sweep_config_creation() {
        let base_config = RunConfig::default();
        let grid_config = GridSweepConfig::new(base_config, 5);

        assert!(grid_config.parameters.is_empty());
        assert_eq!(grid_config.runs_per_config, 5);
        assert_eq!(grid_config.total_points(), 0);
        assert_eq!(grid_config.total_runs(), 0);
    }

    #[test]
    fn test_grid_sweep_config_add_parameter() {
        let base_config = RunConfig::default();
        let mut grid_config = GridSweepConfig::new(base_config, 3);

        grid_config.add_parameter("agent_count", vec![50.0, 100.0, 200.0]);
        assert_eq!(grid_config.parameters.len(), 1);
        assert_eq!(grid_config.total_points(), 3);
        assert_eq!(grid_config.total_runs(), 9);

        grid_config.add_parameter("max_ticks", vec![500.0, 1000.0]);
        assert_eq!(grid_config.parameters.len(), 2);
        assert_eq!(grid_config.total_points(), 6); // 3 * 2
        assert_eq!(grid_config.total_runs(), 18); // 6 * 3
    }

    #[test]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn test_grid_sweep_config_builder_pattern() {
        let base_config = RunConfig::default();
        let grid_config = GridSweepConfig::new(base_config, 2)
            .add_parameter("agent_count", vec![50.0, 100.0])
            .add_parameter("grid_width", vec![32.0, 64.0])
            .clone();

        assert_eq!(grid_config.total_points(), 4);
        assert_eq!(grid_config.total_runs(), 8);
    }

    #[test]
    fn test_grid_point_creation() {
        let mut values = HashMap::new();
        values.insert("agent_count".to_string(), 100.0);
        values.insert("max_ticks".to_string(), 1000.0);

        let point = GridPoint::new(values.clone());

        assert_eq!(point.get("agent_count"), Some(100.0));
        assert_eq!(point.get("max_ticks"), Some(1000.0));
        assert_eq!(point.get("nonexistent"), None);
        assert!(point.run_indices.is_empty());
    }

    #[test]
    fn test_grid_sweep_runner_combinations() {
        let base_config = RunConfig::default();
        let grid_config = GridSweepConfig::new(base_config, 1)
            .add_parameter("param_a", vec![1.0, 2.0])
            .add_parameter("param_b", vec![10.0, 20.0, 30.0])
            .clone();

        let runner = GridSweepRunner::new(grid_config);
        let combinations = runner.generate_combinations();

        // 2 * 3 = 6 combinations
        assert_eq!(combinations.len(), 6);

        // Verify each combination has both parameters
        for combo in &combinations {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            assert!(combo.contains_key("param_a"));
            assert!(combo.contains_key("param_b"));
        }
    }

    #[test]
    fn test_grid_sweep_runner_single_parameter() {
        let base_config = RunConfig::default();
        let grid_config = GridSweepConfig::new(base_config, 1)
            .add_parameter("agent_count", vec![50.0, 100.0, 150.0])
            .clone();

        let runner = GridSweepRunner::new(grid_config);
        let combinations = runner.generate_combinations();

        assert_eq!(combinations.len(), 3);

        // ERR-001: Test code - unwrap() exempt for test assertions
        let values: Vec<f32> = combinations
            .iter()
            .map(|c| *c.get("agent_count").unwrap())
            .collect();

        assert!(values.contains(&50.0));
        assert!(values.contains(&100.0));
        assert!(values.contains(&150.0));
    }

    #[test]
    fn test_grid_sweep_result_stats() {
        // Create mock results for testing
        let point1 = GridPoint::new({
            let mut m = HashMap::new();
            m.insert("agent_count".to_string(), 50.0);
            m
        });
        let point2 = GridPoint::new({
            let mut m = HashMap::new();
            m.insert("agent_count".to_string(), 100.0);
            m
        });

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // We can't easily create BatchResult here, so we'll just verify the structure
        let result = GridSweepResult {
            points: vec![point1, point2],
            results: vec![], // Empty for this test
            parameter_names: vec!["agent_count".to_string()],
        };

        assert_eq!(result.points.len(), 2);
        assert_eq!(result.parameter_names, vec!["agent_count"]);
    }

    #[test]
    fn test_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        assert_eq!(percentile(&values, 0.0), 1.0);
        assert_eq!(percentile(&values, 50.0), 3.0);
        assert_eq!(percentile(&values, 100.0), 5.0);
    }

    #[test]
    fn test_percentile_empty() {
        let values: Vec<f64> = vec![];
        assert_eq!(percentile(&values, 50.0), 0.0);
    }

    #[test]
    fn test_heatmap_data_csv() {
        let heatmap = HeatmapData {
            x_param: "param_x".to_string(),
            y_param: "param_y".to_string(),
            x_values: vec![1.0, 2.0],
            y_values: vec![10.0, 20.0],
            grid: vec![vec![Some(1.5), Some(2.5)], vec![Some(3.5), None]],
        };

        let csv = heatmap.to_csv("metric");
        let lines: Vec<&str> = csv.lines().collect();

        // Header
        assert_eq!(lines[0], "param_x,param_y,metric");
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Check we have the right number of data rows (2x2 = 4)
        assert_eq!(lines.len(), 5);

        // Check NA handling
        assert!(csv.contains("NA"));
    }

    #[test]
    fn test_grid_sweep_error_display() {
        let err = GridSweepError::UnknownParameter("test".to_string());
        assert!(err.to_string().contains("Unknown parameter"));

        let err = GridSweepError::InvalidData("bad data".to_string());
        assert!(err.to_string().contains("Invalid data"));
    }

    #[test]
    fn test_empty_grid_sweep() {
        let base_config = RunConfig::default();
        let grid_config = GridSweepConfig::new(base_config, 1);

        let runner = GridSweepRunner::new(grid_config);
        let combinations = runner.generate_combinations();

        assert!(combinations.is_empty());
    }

    #[test]
    fn test_grid_sweep_cartesian_product() {
        // Test 3 parameters with 2, 3, and 2 values respectively
        let base_config = RunConfig::default();
        let grid_config = GridSweepConfig::new(base_config, 1)
            .add_parameter("p1", vec![1.0, 2.0])
            .add_parameter("p2", vec![10.0, 20.0, 30.0])
            .add_parameter("p3", vec![100.0, 200.0])
            .clone();

        let runner = GridSweepRunner::new(grid_config);
        let combinations = runner.generate_combinations();

        // 2 * 3 * 2 = 12 combinations
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(combinations.len(), 12);

        // Verify all combinations are unique
        let unique_combos: std::collections::HashSet<String> = combinations
            .iter()
            // ERR-001: Test code - unwrap() exempt for test assertions
            .map(|c| {
                format!(
                    "{},{},{}",
                    c.get("p1").unwrap(),
                    c.get("p2").unwrap(),
                    c.get("p3").unwrap()
                )
            })
            .collect();
        assert_eq!(unique_combos.len(), 12);
    }

    // =========================================================================
    // Monte Carlo Tests
    // =========================================================================

    #[test]
    fn test_monte_carlo_config_creation() {
        let base_config = RunConfig::default();
        let mc_config = MonteCarloConfig::new(100, base_config, 42);

        assert_eq!(mc_config.n_samples, 100);
        assert_eq!(mc_config.seed, 42);
        assert!(mc_config.parameter_bounds.is_empty());
    }

    #[test]
    fn test_monte_carlo_config_add_parameter() {
        let base_config = RunConfig::default();
        let mut mc_config = MonteCarloConfig::new(100, base_config, 42);

        mc_config.add_parameter("agent_count", 50.0, 500.0);
        mc_config.add_parameter("max_ticks", 100.0, 1000.0);

        assert_eq!(mc_config.parameter_bounds.len(), 2);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(
            mc_config.parameter_bounds[0],
            ("agent_count".to_string(), 50.0, 500.0)
        );
    }

    #[test]
    fn test_sample_generator_uniform() {
        let bounds = vec![
            ("param1".to_string(), 0.0f32, 10.0f32),
            ("param2".to_string(), 5.0f32, 15.0f32),
        ];

        let mut gen = SampleGenerator::new(42, DistributionType::Uniform, 10);
        let sample = gen.sample(&bounds);

        assert_eq!(sample.len(), 2);
        // ERR-001: Test code - unwrap() exempt for test assertions
        let v1 = sample.get("param1").unwrap();
        let v2 = sample.get("param2").unwrap();

        assert!(*v1 >= 0.0 && *v1 <= 10.0);
        assert!(*v2 >= 5.0 && *v2 <= 15.0);
    }

    #[test]
    fn test_sample_generator_normal() {
        let bounds = vec![("param1".to_string(), 0.0f32, 100.0f32)];

        let mut gen = SampleGenerator::new(
            42,
            DistributionType::Normal {
                mean: 50.0,
                std_dev: 10.0,
            },
            10,
        );
        let sample = gen.sample(&bounds);

        // ERR-001: Test code - unwrap() exempt for test assertions
        let v1 = sample.get("param1").unwrap();
        // Value should be within bounds (clamped)
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(*v1 >= 0.0 && *v1 <= 100.0);
    }

    #[test]
    fn test_sample_generator_log_uniform() {
        let bounds = vec![("param1".to_string(), 1.0f32, 1000.0f32)];

        let mut gen = SampleGenerator::new(42, DistributionType::LogUniform, 10);
        let sample = gen.sample(&bounds);

        // ERR-001: Test code - unwrap() exempt for test assertions
        let v1 = sample.get("param1").unwrap();
        assert!(*v1 >= 1.0 && *v1 <= 1000.0);
    }

    #[test]
    fn test_sample_generator_reproducibility() {
        let bounds = vec![
            ("param1".to_string(), 0.0f32, 10.0f32),
            ("param2".to_string(), 5.0f32, 15.0f32),
        ];

        let mut gen1 = SampleGenerator::new(42, DistributionType::Uniform, 10);
        let mut gen2 = SampleGenerator::new(42, DistributionType::Uniform, 10);

        let sample1 = gen1.sample(&bounds);
        let sample2 = gen2.sample(&bounds);

        assert_eq!(sample1.get("param1"), sample2.get("param1"));
        assert_eq!(sample1.get("param2"), sample2.get("param2"));
    }

    #[test]
    fn test_latin_hypercube_sampling() {
        let bounds = vec![
            ("param1".to_string(), 0.0f32, 10.0f32),
            ("param2".to_string(), 0.0f32, 10.0f32),
        ];

        let n_samples = 10;
        let mut gen = SampleGenerator::new(42, DistributionType::LatinHypercube, n_samples);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Generate all samples
        // P1 FIX: Pre-allocate with known capacity
        let mut samples = Vec::with_capacity(n_samples);
        for _ in 0..n_samples {
            samples.push(gen.sample(&bounds));
        }

        // Check that we have the right number of samples
        assert_eq!(samples.len(), n_samples);

        // Check that all values are within bounds
        for sample in &samples {
            // ERR-001: Test code - unwrap() exempt for test assertions
            let v1 = sample.get("param1").unwrap();
            let v2 = sample.get("param2").unwrap();
            assert!(*v1 >= 0.0 && *v1 <= 10.0);
            assert!(*v2 >= 0.0 && *v2 <= 10.0);
        }
    }

    #[test]
    fn test_latin_hypercube_sampling_n_one_and_inverted_bounds() {
        let bounds = vec![
            ("inverted".to_string(), 10.0f32, 2.0f32),
            ("zero_width".to_string(), 5.0f32, 5.0f32),
        ];

        let mut gen = SampleGenerator::new(42, DistributionType::LatinHypercube, 1);
        let sample = gen.sample(&bounds);

        let inv = *sample.get("inverted").unwrap();
        let zero = *sample.get("zero_width").unwrap();

        assert!(
            inv >= 2.0 && inv <= 10.0,
            "inverted bound should be respected: {inv}"
        );
        assert_eq!(zero, 5.0, "zero-width bound should produce exact value");
    }

    #[test]
    fn test_sample_parameters_function() {
        let bounds = vec![
            ("param1".to_string(), 0.0f32, 10.0f32),
            ("param2".to_string(), 5.0f32, 15.0f32),
        ];

        let mut rng = StdRng::seed_from_u64(42);
        let sample = sample_parameters(&bounds, DistributionType::Uniform, &mut rng);

        assert_eq!(sample.len(), 2);
        // ERR-001: Test code - unwrap() exempt for test assertions
        let v1 = sample.get("param1").unwrap();
        let v2 = sample.get("param2").unwrap();

        assert!(*v1 >= 0.0 && *v1 <= 10.0);
        assert!(*v2 >= 5.0 && *v2 <= 15.0);
    }

    #[test]
    fn test_pearson_correlation() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let corr = pearson_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 1e-10);

        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let corr_neg = pearson_correlation(&x, &y_neg);
        assert!((corr_neg - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_spearman_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let corr = spearman_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rank_data() {
        let data = vec![3.0, 1.0, 4.0, 1.0, 5.0];
        let ranks = rank_data(&data);

        // 1.0 appears twice (positions 1 and 3), tied for rank 1-2, average = 1.5
        // 3.0 is rank 3
        // 4.0 is rank 4
        // 5.0 is rank 5
        assert_eq!(ranks[1], 1.5); // First 1.0
        assert_eq!(ranks[3], 1.5); // Second 1.0
        assert_eq!(ranks[0], 3.0); // 3.0
        assert_eq!(ranks[2], 4.0); // 4.0
        assert_eq!(ranks[4], 5.0); // 5.0
    }

    #[test]
    fn test_monte_carlo_runner_validation() {
        let base_config = RunConfig::default();
        let mc_config = MonteCarloConfig::new(0, base_config, 42); // n_samples = 0
        let runner = MonteCarloRunner::new(mc_config);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let result = runner.run(1);
        assert!(result.is_err()); // Should fail due to n_samples = 0
    }

    #[test]
    fn test_distribution_type_default() {
        let default = DistributionType::default();
        assert_eq!(default, DistributionType::Uniform);
    }

    #[test]
    fn test_monte_carlo_config_with_distribution() {
        let base_config = RunConfig::default();
        let mc_config = MonteCarloConfig::new(100, base_config, 42)
            .with_distribution(DistributionType::LogUniform);

        assert_eq!(mc_config.distribution, DistributionType::LogUniform);
    }

    #[test]
    fn test_compute_std() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let std = compute_std(&values, mean);

        // Population std ≈ 2.0, sample std ≈ 2.138
        assert!(std > 2.0 && std < 2.5);
    }

    #[test]
    fn test_sample_generator_reset() {
        let bounds = vec![("param1".to_string(), 0.0f32, 10.0f32)];

        let mut gen = SampleGenerator::new(42, DistributionType::Uniform, 10);
        let sample1 = gen.sample(&bounds);

        gen.reset(42);
        let sample2 = gen.sample(&bounds);

        assert_eq!(sample1.get("param1"), sample2.get("param1"));
    }

    #[test]
    fn test_lhs_stratification_bijection() {
        use super::super::monte_carlo::sample_lhs_parameters;
        use std::collections::HashSet;
        let bounds = vec![("x".to_string(), 0.0f32, 100.0f32)];
        let n_samples = 20;
        let sweep_seed = 12345;

        let mut observed_strata = HashSet::new();
        for sample_idx in 0..n_samples {
            let mut rng = StdRng::seed_from_u64(sweep_seed + sample_idx as u64);
            let sample =
                sample_lhs_parameters(&bounds, n_samples, sample_idx, sweep_seed, &mut rng);
            let val = *sample.get("x").unwrap();
            let stratum = (val / (100.0 / n_samples as f32)).floor() as usize;
            assert!(stratum < n_samples);
            assert!(
                observed_strata.insert(stratum),
                "Stratum {stratum} must be sampled exactly once in LHS"
            );
        }
        assert_eq!(observed_strata.len(), n_samples);
    }
}
