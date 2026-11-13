// TAG: surface=blockchain owner=blockchain-team rule=BC-001
//! Metrics collection and statistical analysis for `FreshCredit` testnet
//!
//! Provides:
//! - Mann-Whitney U test for comparing simulation groups
//! - Granger causality test for temporal relationships
//! - Block anchoring metrics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Metrics collected from agent simulations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub simulation_id: String,
    pub agent_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metrics: HashMap<String, f64>,
    pub fraud_score: Option<f64>,
    pub block_number: Option<u32>,
    pub extrinsic_hash: Option<String>,
}

/// Statistical test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTestResult {
    pub test_name: String,
    pub statistic: f64,
    pub p_value: f64,
    pub significant: bool,
    pub interpretation: String,
}

/// Mann-Whitney U test for comparing two independent samples
/// Used to compare fraud scores between control and treatment groups
// Allow: sample sizes and ranks are tiny compared to 2^52, so the usize→f64
// conversion is exact for every realistic input.
#[allow(clippy::cast_precision_loss)]
pub fn mann_whitney_u_test(sample_a: &[f64], sample_b: &[f64]) -> StatisticalTestResult {
    use statrs::distribution::{ContinuousCDF, Normal};
    #[allow(unused_imports)]
    use statrs::statistics::Statistics;

    let valid_a: Vec<f64> = sample_a.iter().copied().filter(|x| x.is_finite()).collect();
    let valid_b: Vec<f64> = sample_b.iter().copied().filter(|x| x.is_finite()).collect();

    let n1 = valid_a.len() as f64;
    let n2 = valid_b.len() as f64;

    if n1 < 2.0 || n2 < 2.0 {
        return StatisticalTestResult {
            test_name: "Mann-Whitney U".to_string(),
            statistic: 0.0,
            p_value: 1.0,
            significant: false,
            // TAG: surface=blockchain owner=blockchain-team rule=BC-001
            interpretation: "Insufficient samples for test".to_string(),
        };
    }

    // Combine and rank samples with mid-rank assignment for ties
    let mut combined: Vec<(f64, usize)> = valid_a
        .iter()
        .map(|&x| (x, 0))
        .chain(valid_b.iter().map(|&x| (x, 1)))
        .collect();

    combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let n_total = combined.len();
    let mut sum_ranks_a = 0.0f64;
    let mut sum_ranks_b = 0.0f64;
    let mut tie_sum = 0.0f64;

    let mut i = 0;
    while i < n_total {
        let mut j = i + 1;
        while j < n_total && (combined[j].0 - combined[i].0).abs() <= 1e-12 {
            j += 1;
        }
        let tie_count = (j - i) as f64;
        let mid_rank = (i + 1 + j) as f64 / 2.0;

        if tie_count > 1.0 {
            tie_sum += tie_count.powi(3) - tie_count;
        }

        for item in &combined[i..j] {
            if item.1 == 0 {
                sum_ranks_a += mid_rank;
            } else {
                sum_ranks_b += mid_rank;
            }
        }
        i = j;
    }

    // Calculate U statistics
    let u1 = sum_ranks_a - (n1 * (n1 + 1.0)) / 2.0;
    let u2 = sum_ranks_b - (n2 * (n2 + 1.0)) / 2.0;
    let u = u1.min(u2);

    // Normal approximation with Lehmann/Mann-Whitney tie correction
    let mean_u = (n1 * n2) / 2.0;
    let n = n1 + n2;
    let tie_correction = if n > 1.0 {
        tie_sum / (n * (n - 1.0))
    } else {
        0.0
    };
    let var_u = ((n1 * n2) / 12.0) * (n + 1.0 - tie_correction);
    let std_u = if var_u > 0.0 { var_u.sqrt() } else { 0.0 };

    // Two-tailed p-value with degenerate-variance protection
    let p_value = if std_u <= 1e-12 {
        1.0
    } else {
        let z = (u - mean_u) / std_u;
        if let Ok(normal) = Normal::new(0.0, 1.0) {
            (2.0 * (1.0 - normal.cdf(z.abs()))).clamp(0.0, 1.0)
        } else {
            1.0
        }
    };

    let significant = p_value < 0.05;
    let interpretation = if significant {
        format!(
            "Significant difference detected (p={p_value:.4}). Groups likely have different distributions.",
        )
    } else {
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        format!(
            "No significant difference (p={p_value:.4}). Groups likely have similar distributions.",
        )
    };

    StatisticalTestResult {
        test_name: "Mann-Whitney U".to_string(),
        statistic: u,
        p_value,
        significant,
        interpretation,
    }
}

/// Granger causality test (simplified)
#[allow(dead_code)]
/// Tests if time series X "Granger-causes" time series Y
/// Used to detect if simulation parameters predict fraud scores
// Allow: sample lengths and lags are tiny compared to 2^52, so the
// usize→f64 conversion is exact for every realistic input.
#[allow(clippy::cast_precision_loss)]
pub fn granger_causality_test(x: &[f64], y: &[f64], max_lag: usize) -> StatisticalTestResult {
    if x.len() != y.len() || max_lag == 0 || x.len() <= 2 * max_lag + 1 {
        return StatisticalTestResult {
            test_name: "Granger Causality".to_string(),
            statistic: 0.0,
            p_value: 1.0,
            significant: false,
            interpretation: "Insufficient data for test".to_string(),
        };
    }

    // Simplified F-test implementation
    // Full implementation would use regression models
    let n = x.len() as f64;
    let k = max_lag as f64;

    // Calculate simple correlation at different lags
    let mut max_correlation: f64 = 0.0;
    let mut best_lag = 0;

    for lag in 1..=max_lag {
        if x.len() <= lag {
            break;
        }
        let x_lagged = &x[..x.len() - lag];
        let y_shifted = &y[lag..];

        let correlation = calculate_correlation(x_lagged, y_shifted);
        if correlation.abs() > max_correlation.abs() {
            max_correlation = correlation;
            best_lag = lag;
            // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        }
    }

    // Approximate F-statistic with non-negative degrees-of-freedom guarantee
    let r_squared = (max_correlation * max_correlation).clamp(0.0, 1.0 - 1e-12);
    let df2 = (n - 2.0 * k - 1.0).max(1.0);
    let f_statistic = ((r_squared / k) / ((1.0 - r_squared) / df2)).max(0.0);

    // Approximate p-value (simplified)
    let p_value = if f_statistic > 3.0 {
        0.01
    } else if f_statistic > 2.0 {
        0.05
    } else if f_statistic > 1.0 {
        0.10
    } else {
        0.50
    };

    let significant = p_value < 0.05;
    let interpretation = if significant {
        format!(
            "X Granger-causes Y at lag {best_lag} (F={f_statistic:.2}, p={p_value:.4}). X contains predictive information for Y.",
        )
    } else {
        format!(
            "No Granger causality detected (F={f_statistic:.2}, p={p_value:.4}). X does not predict Y.",
        )
    };

    StatisticalTestResult {
        test_name: "Granger Causality".to_string(),
        statistic: f_statistic,
        p_value,
        significant,
        interpretation,
    }
}

/// Calculate Pearson correlation coefficient
// Allow: sample lengths are tiny compared to 2^52, so the usize→f64
// conversion is exact for every realistic input.
#[allow(dead_code, clippy::cast_precision_loss)]
fn calculate_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }

    let pairs: Vec<(f64, f64)> = x
        .iter()
        .zip(y.iter())
        .filter(|(&a, &b)| a.is_finite() && b.is_finite())
        .map(|(&a, &b)| (a, b))
        .collect();

    if pairs.len() < 2 {
        return 0.0;
    }

    let n = pairs.len() as f64;
    let mean_x: f64 = pairs.iter().map(|p| p.0).sum::<f64>() / n;
    let mean_y: f64 = pairs.iter().map(|p| p.1).sum::<f64>() / n;

    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    let mut numerator = 0.0;
    let mut denom_x = 0.0;
    let mut denom_y = 0.0;

    for &(xi, yi) in &pairs {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        numerator += dx * dy;
        denom_x += dx * dx;
        denom_y += dy * dy;
    }

    if denom_x <= 1e-12 || denom_y <= 1e-12 {
        return 0.0;
    }

    let denom = (denom_x * denom_y).sqrt();
    if denom <= 1e-12 {
        0.0
    } else {
        (numerator / denom).clamp(-1.0, 1.0)
    }
}

/// Metrics collector for batching and analysis
pub struct MetricsCollector {
    pub metrics: Vec<SimulationMetrics>,
    max_size: usize,
}

impl MetricsCollector {
    pub fn new(max_size: usize) -> Self {
        Self {
            metrics: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add(&mut self, metric: SimulationMetrics) {
        if self.max_size == 0 {
            return;
        }
        if self.metrics.len() >= self.max_size {
            self.metrics.remove(0);
        }
        self.metrics.push(metric);
    }

    pub fn get_fraud_scores_by_agent(&self, agent_id: &str) -> Vec<f64> {
        self.metrics
            .iter()
            .filter(|m| m.agent_id == agent_id)
            .filter_map(|m| m.fraud_score)
            .collect()
    }

    #[allow(dead_code)]
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    pub fn get_time_series(&self, metric_name: &str) -> Vec<(chrono::DateTime<chrono::Utc>, f64)> {
        self.metrics
            .iter()
            .filter_map(|m| m.metrics.get(metric_name).map(|&v| (m.timestamp, v)))
            .collect()
    }

    pub fn compare_agents(&self, agent_a: &str, agent_b: &str) -> Option<StatisticalTestResult> {
        let scores_a = self.get_fraud_scores_by_agent(agent_a);
        let scores_b = self.get_fraud_scores_by_agent(agent_b);

        if scores_a.len() < 5 || scores_b.len() < 5 {
            warn!("Insufficient data for comparison");
            return None;
        }

        info!(
            "Comparing agents: {} (n={}) vs {} (n={})",
            agent_a,
            scores_a.len(),
            agent_b,
            scores_b.len()
        );

        Some(mann_whitney_u_test(&scores_a, &scores_b))
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.metrics.clear();
    }

    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }
}

#[cfg(test)]
// Allow: tests assert exact sentinel values (0.0 correlations, 1.0
// p-values for insufficient-data paths) where approximate comparison
// would weaken the assertion.
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_mann_whitney_u() {
        // Control group (lower fraud scores)
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        let control = vec![0.1, 0.15, 0.12, 0.18, 0.11, 0.14, 0.13, 0.16];

        // Treatment group (higher fraud scores)
        let treatment = vec![0.45, 0.52, 0.48, 0.55, 0.50, 0.47, 0.53, 0.49];

        let result = mann_whitney_u_test(&control, &treatment);

        assert!(
            result.p_value < 0.05,
            "Should detect significant difference"
        );
        assert!(result.significant);
    }

    #[test]
    fn test_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let corr = calculate_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 0.001, "Perfect correlation expected");
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new(100);

        // Add some metrics
        for i in 0..10 {
            collector.add(SimulationMetrics {
                simulation_id: format!("sim-{i}"),
                agent_id: "agent-1".to_string(),
                timestamp: chrono::Utc::now(),
                metrics: HashMap::new(),
                fraud_score: Some(f64::from(i) * 0.1),
                block_number: Some(u32::try_from(i).unwrap_or(0)),
                extrinsic_hash: None,
            });
        }

        assert_eq!(collector.len(), 10);

        let scores = collector.get_fraud_scores_by_agent("agent-1");
        assert_eq!(scores.len(), 10);
    }

    #[test]
    fn test_mann_whitney_u_insufficient_samples() {
        let a = vec![0.5];
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        let b = vec![0.6];
        let result = mann_whitney_u_test(&a, &b);
        assert!(!result.significant);
        assert_eq!(result.p_value, 1.0);
        assert!(result.interpretation.contains("Insufficient"));
    }

    #[test]
    fn test_mann_whitney_u_no_significant_difference() {
        // Same distribution samples
        let sample_a = vec![0.3, 0.31, 0.29, 0.32, 0.30, 0.28, 0.33, 0.31];
        let sample_b = vec![0.30, 0.32, 0.28, 0.31, 0.29, 0.33, 0.30, 0.31];
        let result = mann_whitney_u_test(&sample_a, &sample_b);
        assert!(
            !result.significant,
            "Should not detect significant difference for same distribution"
        );
        assert!(result.p_value >= 0.05);
    }

    #[test]
    fn test_correlation_empty() {
        let corr = calculate_correlation(&[], &[]);
        assert_eq!(corr, 0.0);
    }

    #[test]
    fn test_correlation_mismatched_lengths() {
        let x = vec![1.0, 2.0];
        let y = vec![1.0];
        let corr = calculate_correlation(&x, &y);
        assert_eq!(corr, 0.0);
    }

    #[test]
    fn test_correlation_perfect_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let corr = calculate_correlation(&x, &y);
        assert!(
            (corr - (-1.0)).abs() < 0.001,
            "Perfect negative correlation expected, got {corr}"
        );
    }

    #[test]
    fn test_correlation_zero_variance() {
        let x = vec![1.0, 1.0, 1.0];
        let y = vec![1.0, 2.0, 3.0];
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        let corr = calculate_correlation(&x, &y);
        assert_eq!(corr, 0.0);
    }

    #[test]
    fn test_metrics_collector_max_size_eviction() {
        let mut collector = MetricsCollector::new(5);
        for i in 0..10 {
            collector.add(SimulationMetrics {
                simulation_id: format!("sim-{i}"),
                agent_id: "agent-1".to_string(),
                timestamp: chrono::Utc::now(),
                metrics: HashMap::new(),
                fraud_score: Some(f64::from(i)),
                block_number: Some(u32::try_from(i).unwrap_or(0)),
                extrinsic_hash: None,
            });
        }
        assert_eq!(collector.len(), 5);
        // Oldest entries should have been evicted, so fraud scores should be [5,6,7,8,9]
        let scores = collector.get_fraud_scores_by_agent("agent-1");
        assert_eq!(scores, vec![5.0, 6.0, 7.0, 8.0, 9.0]);
    }

    #[test]
    fn test_metrics_collector_get_fraud_scores_filters_by_agent() {
        let mut collector = MetricsCollector::new(100);
        for i in 0..5 {
            collector.add(SimulationMetrics {
                simulation_id: format!("sim-{i}"),
                agent_id: "agent-1".to_string(),
                timestamp: chrono::Utc::now(),
                metrics: HashMap::new(),
                fraud_score: Some(f64::from(i)),
                block_number: Some(u32::try_from(i).unwrap_or(0)),
                extrinsic_hash: None,
            });
        }
        for i in 0..3 {
            collector.add(SimulationMetrics {
                simulation_id: format!("sim-b-{i}"),
                agent_id: "agent-2".to_string(),
                timestamp: chrono::Utc::now(),
                metrics: HashMap::new(),
                fraud_score: Some(f64::from(i + 10)),
                block_number: Some(u32::try_from(i).unwrap_or(0)),
                extrinsic_hash: None,
            });
        }
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        let scores_1 = collector.get_fraud_scores_by_agent("agent-1");
        assert_eq!(scores_1.len(), 5);
        assert_eq!(scores_1, vec![0.0, 1.0, 2.0, 3.0, 4.0]);

        let scores_2 = collector.get_fraud_scores_by_agent("agent-2");
        assert_eq!(scores_2.len(), 3);
        assert_eq!(scores_2, vec![10.0, 11.0, 12.0]);

        let scores_3 = collector.get_fraud_scores_by_agent("nonexistent");
        assert!(scores_3.is_empty());
    }

    #[test]
    fn test_metrics_collector_compare_agents_insufficient_data() {
        let mut collector = MetricsCollector::new(100);
        for i in 0..3 {
            collector.add(SimulationMetrics {
                simulation_id: format!("sim-{i}"),
                agent_id: "agent-a".to_string(),
                timestamp: chrono::Utc::now(),
                metrics: HashMap::new(),
                fraud_score: Some(f64::from(i)),
                block_number: Some(u32::try_from(i).unwrap_or(0)),
                extrinsic_hash: None,
            });
        }
        for i in 0..3 {
            collector.add(SimulationMetrics {
                simulation_id: format!("sim-{i}"),
                agent_id: "agent-b".to_string(),
                timestamp: chrono::Utc::now(),
                metrics: HashMap::new(),
                fraud_score: Some(f64::from(i)),
                block_number: Some(u32::try_from(i).unwrap_or(0)),
                extrinsic_hash: None,
            });
        }
        let result = collector.compare_agents("agent-a", "agent-b");
        assert!(result.is_none());
    }

    #[test]
    fn test_metrics_collector_get_time_series() {
        let mut collector = MetricsCollector::new(100);
        let now = chrono::Utc::now();
        for i in 0..3 {
            let mut metrics = HashMap::new();
            metrics.insert("trust_score".to_string(), f64::from(i * 10));
            metrics.insert("other_metric".to_string(), 99.0);
            // TAG: surface=blockchain owner=blockchain-team rule=BC-001
            collector.add(SimulationMetrics {
                simulation_id: format!("sim-{i}"),
                agent_id: "agent-1".to_string(),
                timestamp: now + chrono::Duration::seconds(i64::from(i)),
                metrics,
                fraud_score: None,
                block_number: Some(u32::try_from(i).unwrap_or(0)),
                extrinsic_hash: None,
            });
        }
        let series = collector.get_time_series("trust_score");
        assert_eq!(series.len(), 3);
        assert_eq!(
            series.iter().map(|(_, v)| *v).collect::<Vec<_>>(),
            vec![0.0, 10.0, 20.0]
        );

        let empty_series = collector.get_time_series("nonexistent");
        assert!(empty_series.is_empty());
    }

    #[test]
    fn test_metrics_collector_clear() {
        let mut collector = MetricsCollector::new(100);
        collector.add(SimulationMetrics {
            simulation_id: "sim-1".to_string(),
            agent_id: "agent-1".to_string(),
            timestamp: chrono::Utc::now(),
            metrics: HashMap::new(),
            fraud_score: Some(0.5),
            block_number: Some(1),
            extrinsic_hash: None,
        });
        assert_eq!(collector.len(), 1);
        assert!(!collector.is_empty());
        collector.clear();
        assert_eq!(collector.len(), 0);
        assert!(collector.is_empty());
    }

    #[test]
    fn test_granger_causality_insufficient_data() {
        let x = vec![1.0, 2.0];
        let y = vec![1.0, 2.0];
        let result = granger_causality_test(&x, &y, 2);
        assert!(!result.significant);
        assert_eq!(result.p_value, 1.0);
        assert!(result.interpretation.contains("Insufficient"));
    }

    #[test]
    fn test_mann_whitney_identical_tied_samples_not_significant() {
        let sample_a = vec![1.0; 10];
        let sample_b = vec![1.0; 10];
        let res = mann_whitney_u_test(&sample_a, &sample_b);
        assert!(!res.significant);
        assert_eq!(res.p_value, 1.0);
        assert_eq!(res.statistic, 50.0);
    }

    #[test]
    fn test_granger_causality_small_sample_df_guard() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        // max_lag = 3 with n = 5: n <= 2*3 + 1 (5 <= 7), must return insufficient data rather than negative F-statistic
        let res = granger_causality_test(&x, &y, 3);
        assert!(!res.significant);
        assert_eq!(res.p_value, 1.0);
        assert_eq!(res.statistic, 0.0);
    }
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
}
