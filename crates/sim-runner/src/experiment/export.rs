//! Experiment validation, summary, and artifact export

use super::types::*;
use super::ExperimentRunner;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

impl ExperimentRunner {
    pub(crate) fn compute_summary(replications: &[ReplicationResult]) -> ExperimentSummary {
        if replications.is_empty() {
            return ExperimentSummary::default();
        }

        let metric_names = Self::collect_metric_names(replications);
        let mut means = HashMap::new();
        let mut stds = HashMap::new();
        let mut medians = HashMap::new();
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut mins = HashMap::new();
        let mut maxs = HashMap::new();

        for name in &metric_names {
            if let Some((mean, std, median, min, max)) =
                Self::compute_single_metric_stats(replications, name)
            {
                means.insert(name.clone(), mean);
                stds.insert(name.clone(), std);
                medians.insert(name.clone(), median);
                mins.insert(name.clone(), min);
                maxs.insert(name.clone(), max);
            }
        }

        let successful = replications.iter().filter(|r| r.success).count();

        ExperimentSummary {
            means,
            stds,
            medians,
            mins,
            maxs,
            successful_replications: successful,
            total_replications: replications.len(),
        }
    }

    fn collect_metric_names(replications: &[ReplicationResult]) -> Vec<String> {
        let mut metric_names: Vec<String> = Vec::new();
        for rep in replications {
            for name in rep.metrics.keys() {
                if !metric_names.contains(name) {
                    metric_names.push(name.clone());
                }
            }
        }
        metric_names
    }

    fn compute_single_metric_stats(
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        replications: &[ReplicationResult],
        name: &str,
    ) -> Option<(f64, f64, f64, f64, f64)> {
        let values: Vec<f64> = replications
            .iter()
            .filter_map(|r| r.metrics.get(name).copied())
            .collect();

        if values.is_empty() {
            return None;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std = variance.sqrt();

        let mut sorted = values;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = sorted[sorted.len() / 2];
        let min = *sorted.first()?;
        let max = *sorted.last()?;

        Some((mean, std, median, min, max))
    }

    /// Validate summary against acceptance criteria
    pub(crate) fn validate(
        summary: &ExperimentSummary,
        criteria: &[AcceptanceCriterion],
    ) -> ValidationResult {
        let mut results = Vec::new();
        let mut all_passed = true;
        let mut confidence_intervals = HashMap::new();

        for criterion in criteria {
            let (passed, ci, result) = Self::check_criterion(summary, criterion);
            confidence_intervals.insert(criterion.metric_name.clone(), ci);
            if !passed {
                all_passed = false;
            }
            results.push(result);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        ValidationResult {
            passed: all_passed,
            criteria_results: results,
            confidence_intervals,
        }
    }

    fn check_criterion(
        summary: &ExperimentSummary,
        criterion: &AcceptanceCriterion,
    ) -> (bool, (f64, f64), CriterionResult) {
        let observed = summary
            .means
            .get(&criterion.metric_name)
            .copied()
            .unwrap_or(f64::NAN);
        let std = summary
            .stds
            .get(&criterion.metric_name)
            .copied()
            .unwrap_or(0.0);
        let n = summary.successful_replications as f64;

        let ci_width = 1.96 * std / n.sqrt();
        let ci = (observed - ci_width, observed + ci_width);

        let passed = match criterion.comparison {
            Comparison::InRange | Comparison::CorrelationIn => {
                observed >= criterion.expected_range.0 && observed <= criterion.expected_range.1
            }
            Comparison::GreaterThan => observed >= criterion.expected_range.0,
            Comparison::LessThan => observed <= criterion.expected_range.1,
        };

        let result = CriterionResult {
            criterion: criterion.clone(),
            observed_value: observed,
            passed,
            confidence_interval: ci,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        (passed, ci, result)
    }

    /// Export results to CSV
    pub(crate) fn export_csv(
        result: &ExperimentResult,
        path: &Path,
    ) -> Result<(), ExperimentError> {
        std::fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")))?;

        let mut file = std::fs::File::create(path)?;

        // Write header
        let mut metric_names: Vec<&String> = result.summary.means.keys().collect();
        metric_names.sort();

        writeln!(
            file,
            "rep_id,seed,{},duration_ms,success",
            metric_names
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )?;

        // Write data rows
        for rep in &result.replications {
            let metrics_str = metric_names
                .iter()
                .map(|name| {
                    rep.metrics
                        .get(*name)
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join(",");

            writeln!(
                file,
                "{},{},{},{},{}",
                rep.rep_id,
                // TAG: surface=simulation owner=research-team rule=SIM-001
                rep.seed,
                metrics_str,
                rep.duration_ms,
                i32::from(rep.success)
            )?;
        }

        Ok(())
    }

    /// Export manifest to JSON
    pub(crate) fn export_manifest(
        result: &ExperimentResult,
        path: &Path,
    ) -> Result<(), ExperimentError> {
        std::fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")))?;

        let manifest = serde_json::json!({
            "experiment_name": result.config.name,
            "experiment_type": format!("{:?}", result.config.experiment_type),
            "paper": result.config.experiment_type.paper(),
            "gap_ids": result.config.experiment_type.gap_ids(),
            "config": {
                "agent_count": result.config.agent_count,
                "tick_count": result.config.tick_count,
                "replications": result.config.replications,
                "seed_base": result.config.seed_base,
            },
            "summary": {
                "successful_replications": result.summary.successful_replications,
                "total_replications": result.summary.total_replications,
                "means": result.summary.means,
                "stds": result.summary.stds,
            },
            "validation": {
                "passed": result.validation.passed,
                "criteria_results": result.validation.criteria_results.iter().map(|cr| {
                    serde_json::json!({
                        "metric": cr.criterion.metric_name,
                        "expected": cr.criterion.expected_range,
                        "observed": cr.observed_value,
                        "ci_95": cr.confidence_interval,
                        "passed": cr.passed,
                    })
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                }).collect::<Vec<_>>(),
            },
            "passed": result.passed,
        });

        let json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| ExperimentError::Serialization(e.to_string()))?;

        std::fs::write(path, json)?;

        Ok(())
    }

    /// Generate PNG visualization charts from experiment results
    pub(crate) fn generate_png_charts(
        result: &ExperimentResult,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>, ExperimentError> {
        use crate::visualization::ChartConfig;

        let mut png_paths = Vec::new();

        // Create output directory for charts
        let charts_dir = output_dir.join("charts");
        std::fs::create_dir_all(&charts_dir)?;

        // Generate metric distribution charts (histograms for each metric)
        for metric_name in result.summary.means.keys() {
            let chart_path = charts_dir.join(format!(
                "{}_{}_distribution.png",
                result.config.name, metric_name
            ));

            // Collect values from all replications for this metric
            let values: Vec<f64> = result
                .replications
                .iter()
                .filter_map(|rep| rep.metrics.get(metric_name).copied())
                .collect();

            // TAG: surface=simulation owner=research-team rule=SIM-001
            if values.len() >= 2 {
                let bin_count = (values.len() as f64).sqrt().clamp(5.0, 30.0) as usize;

                let histogram_data = crate::visualization::HistogramData::from_data(
                    &values,
                    bin_count,
                    format!("{metric_name} Distribution"),
                );

                let config =
                    ChartConfig::new(800, 600, format!("{metric_name} Distribution"), &chart_path);

                if let Err(e) = crate::visualization::generate_histogram(&histogram_data, &config) {
                    eprintln!("Warning: Failed to generate histogram for {metric_name}: {e}");
                } else {
                    png_paths.push(chart_path);
                }
            }
        }

        // Generate scatter plots for correlation analysis between metrics
        let metric_names: Vec<&String> = result.summary.means.keys().collect();
        for i in 0..metric_names.len() {
            for j in (i + 1)..metric_names.len() {
                let metric_x = metric_names[i];
                let metric_y = metric_names[j];

                // Collect paired data
                let points: Vec<(f64, f64)> = result
                    .replications
                    .iter()
                    .filter_map(|rep| {
                        let x = rep.metrics.get(metric_x)?;
                        let y = rep.metrics.get(metric_y)?;
                        Some((*x, *y))
                    })
                    .collect();

                if points.len() >= 10 {
                    let chart_path = charts_dir.join(format!(
                        "{}_{}_vs_{}.png",
                        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                        result.config.name,
                        metric_y,
                        metric_x
                    ));

                    let scatter_data = crate::visualization::ScatterData {
                        points,
                        x_label: metric_x.clone(),
                        y_label: metric_y.clone(),
                        title: format!("{metric_y} vs {metric_x}"),
                    };

                    let config = ChartConfig::new(
                        800,
                        600,
                        format!("{metric_y} vs {metric_x}"),
                        &chart_path,
                    );

                    if let Err(e) =
                        crate::visualization::generate_scatter_plot(&scatter_data, &config)
                    {
                        eprintln!("Warning: Failed to generate scatter plot: {e}");
                    } else {
                        png_paths.push(chart_path);
                    }
                }
            }
        }

        if !png_paths.is_empty() {
            println!(
                "  Generated {} PNG charts in {}/",
                png_paths.len(),
                charts_dir.display()
            );
        }

        Ok(png_paths)
    }
}
