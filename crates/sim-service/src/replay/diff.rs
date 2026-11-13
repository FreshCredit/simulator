//! Diff Engine - Compare simulation runs

use std::collections::{HashMap, HashSet};

use super::types::{
    AgentState, ComparisonResult, Difference, DivergencePoint, MetricComparison, ReplayFrame,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Engine for comparing simulation runs
#[derive(Debug)]
pub struct DiffEngine {
    /// Divergence threshold for marking divergence points
    divergence_threshold: f64,
}

impl DiffEngine {
    /// Create a new diff engine with default settings
    #[must_use]
    pub const fn new() -> Self {
        Self {
            divergence_threshold: 0.5,
        }
    }

    /// Create with custom threshold
    #[must_use]
    pub const fn with_threshold(divergence: f64) -> Self {
        Self {
            divergence_threshold: divergence,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compare two runs
    #[must_use]
    pub fn compare_runs(
        &self,
        base_run_id: &str,
        compare_run_id: &str,
        base_frames: &[ReplayFrame],
        compare_frames: &[ReplayFrame],
    ) -> ComparisonResult {
        let mut metric_comparisons = Vec::new();
        let mut significant_differences = Vec::new();
        let mut divergence_points = Vec::new();

        // Compare agent counts over time
        let agent_count_diff = self.compare_agent_counts(base_frames, compare_frames);
        if let Some(diff) = agent_count_diff {
            metric_comparisons.push(diff);
        }

        // Compare metrics
        let metric_diffs = self.compare_metrics(base_frames, compare_frames);
        metric_comparisons.extend(metric_diffs);

        // Find significant differences tick by tick
        for tick in self.get_common_ticks(base_frames, compare_frames) {
            let base_frame = self.get_frame_at_tick(base_frames, tick);
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            let compare_frame = self.get_frame_at_tick(compare_frames, tick);

            if let (Some(base_f), Some(compare_f)) = (base_frame, compare_frame) {
                let frame_diffs = self.compare_frames(&base_f, &compare_f);

                // Check for divergence
                let similarity = self.calculate_frame_similarity(&base_f, &compare_f);
                if similarity < self.divergence_threshold {
                    divergence_points.push(DivergencePoint {
                        tick,
                        description: format!(
                            "Runs diverged significantly (similarity: {similarity:.2})"
                        ),
                        magnitude: 1.0 - similarity,
                    });
                }

                // Collect significant differences
                for diff in frame_diffs {
                    if diff.severity > 0.5 {
                        significant_differences.push(diff);
                    }
                }
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Calculate overall similarity
        let similarity_score = self.calculate_overall_similarity(base_frames, compare_frames);

        ComparisonResult {
            base_run_id: base_run_id.to_string(),
            compare_run_id: compare_run_id.to_string(),
            similarity_score,
            metric_comparisons,
            significant_differences,
            divergence_points,
        }
    }

    /// Compare agent counts
    fn compare_agent_counts(
        &self,
        base_frames: &[ReplayFrame],
        compare_frames: &[ReplayFrame],
    ) -> Option<MetricComparison> {
        if base_frames.is_empty() || compare_frames.is_empty() {
            return None;
        }

        let base_avg = base_frames
            .iter()
            .map(|f| f.agents.len() as f64)
            .sum::<f64>()
            / base_frames.len() as f64;

        let compare_avg = compare_frames
            .iter()
            .map(|f| f.agents.len() as f64)
            .sum::<f64>()
            / compare_frames.len() as f64;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let percent_diff = if base_avg == 0.0 {
            0.0
        } else {
            ((compare_avg - base_avg) / base_avg) * 100.0
        };

        Some(MetricComparison {
            metric: "agent_count".to_string(),
            base_avg,
            compare_avg,
            percent_diff,
            correlation: self.calculate_correlation(
                &base_frames
                    .iter()
                    .map(|f| f.agents.len() as f64)
                    .collect::<Vec<_>>(),
                &compare_frames
                    .iter()
                    .map(|f| f.agents.len() as f64)
                    .collect::<Vec<_>>(),
            ),
        })
    }

    /// Compare metrics between runs
    fn compare_metrics(
        &self,
        base_frames: &[ReplayFrame],
        compare_frames: &[ReplayFrame],
    ) -> Vec<MetricComparison> {
        let mut comparisons = Vec::new();

        // Get all unique metric names
        let mut all_metrics: HashSet<String> = HashSet::new();
        for frame in base_frames {
            all_metrics.extend(frame.metrics.keys().cloned());
        }
        for frame in compare_frames {
            all_metrics.extend(frame.metrics.keys().cloned());
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut sorted_metrics: Vec<String> = all_metrics.into_iter().collect();
        sorted_metrics.sort();
        for metric in sorted_metrics {
            let base_values: Vec<f64> = base_frames
                .iter()
                .filter_map(|f| f.metrics.get(&metric).copied())
                .collect();

            let compare_values: Vec<f64> = compare_frames
                .iter()
                .filter_map(|f| f.metrics.get(&metric).copied())
                .collect();

            if !base_values.is_empty() && !compare_values.is_empty() {
                let base_avg = base_values.iter().sum::<f64>() / base_values.len() as f64;
                let compare_avg = compare_values.iter().sum::<f64>() / compare_values.len() as f64;

                let percent_diff = if base_avg == 0.0 {
                    0.0
                } else {
                    ((compare_avg - base_avg) / base_avg) * 100.0
                };

                comparisons.push(MetricComparison {
                    metric: metric.clone(),
                    base_avg,
                    compare_avg,
                    percent_diff,
                    correlation: self.calculate_correlation(&base_values, &compare_values),
                });
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        comparisons
    }

    /// Compare two frames
    fn compare_frames(&self, base: &ReplayFrame, compare: &ReplayFrame) -> Vec<Difference> {
        let mut differences = Vec::new();

        // Compare agent positions
        let agent_diffs = self.compare_agent_positions(&base.agents, &compare.agents, base.tick);
        differences.extend(agent_diffs);

        // Compare environment
        if base.environment.weather != compare.environment.weather {
            differences.push(Difference {
                tick: base.tick,
                diff_type: "weather_change".to_string(),
                description: format!(
                    "Weather differs: {} vs {}",
                    base.environment.weather, compare.environment.weather
                ),
                severity: 0.7,
                affected: vec![],
            });
        }

        // Compare events
        if base.events.len() != compare.events.len() {
            differences.push(Difference {
                tick: base.tick,
                diff_type: "event_count".to_string(),
                description: format!(
                    "Different number of events: {} vs {}",
                    base.events.len(),
                    compare.events.len()
                ),
                severity: 0.5,
                affected: vec![],
            });
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        differences
    }

    /// Compare agent positions
    fn compare_agent_positions(
        &self,
        base_agents: &[AgentState],
        compare_agents: &[AgentState],
        tick: u64,
    ) -> Vec<Difference> {
        let mut differences = Vec::new();

        // Find agents that moved differently
        let base_map: HashMap<&str, &AgentState> = base_agents
            .iter()
            .map(|a| (a.agent_id.as_str(), a))
            .collect();

        let compare_map: HashMap<&str, &AgentState> = compare_agents
            .iter()
            .map(|a| (a.agent_id.as_str(), a))
            .collect();

        // Check for agents that exist in both runs (sorted by agent_id for SIM-004 determinism)
        let mut sorted_agent_ids: Vec<&&str> = base_map.keys().collect();
        sorted_agent_ids.sort();
        for agent_id in sorted_agent_ids {
            let base_agent = base_map[agent_id];
            if let Some(compare_agent) = compare_map.get(agent_id) {
                let distance = self.euclidean_distance(base_agent.position, compare_agent.position);

                // If positions diverged significantly
                if distance > 5.0 {
                    differences.push(Difference {
                        tick,
                        diff_type: "position_divergence".to_string(),
                        description: format!(
                            "Agent {agent_id} at different positions (distance: {distance:.2})"
                        ),
                        severity: (distance / 20.0).min(1.0),
                        affected: vec![(*agent_id).to_string()],
                    });
                }

                // TAG: surface=simulation owner=research-team rule=SIM-001
                // Check state differences
                if base_agent.state != compare_agent.state {
                    differences.push(Difference {
                        tick,
                        diff_type: "state_difference".to_string(),
                        description: format!(
                            "Agent {} in different states: {} vs {}",
                            agent_id, base_agent.state, compare_agent.state
                        ),
                        severity: 0.6,
                        affected: vec![(*agent_id).to_string()],
                    });
                }
            }
        }

        differences
    }

    /// Calculate frame similarity (0.0 - 1.0)
    fn calculate_frame_similarity(&self, base: &ReplayFrame, compare: &ReplayFrame) -> f64 {
        let mut similarities = Vec::new();

        // Agent count similarity
        let max_agents = base.agents.len().max(compare.agents.len()) as f64;
        let agent_sim = if max_agents > 0.0 {
            let min_agents = base.agents.len().min(compare.agents.len()) as f64;
            min_agents / max_agents
        } else {
            1.0
        };
        similarities.push(agent_sim);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Position similarity for common agents
        let base_map: HashMap<&str, &AgentState> = base
            .agents
            .iter()
            .map(|a| (a.agent_id.as_str(), a))
            .collect();
        let compare_map: HashMap<&str, &AgentState> = compare
            .agents
            .iter()
            .map(|a| (a.agent_id.as_str(), a))
            .collect();

        let mut position_similarities = Vec::new();
        for (agent_id, base_agent) in &base_map {
            if let Some(compare_agent) = compare_map.get(agent_id) {
                let distance = self.euclidean_distance(base_agent.position, compare_agent.position);
                // Convert distance to similarity (closer = more similar)
                let sim = (-distance / 10.0).exp();
                position_similarities.push(sim);
            }
        }

        if !position_similarities.is_empty() {
            let avg_pos_sim: f64 =
                position_similarities.iter().sum::<f64>() / position_similarities.len() as f64;
            similarities.push(avg_pos_sim);
        }

        // Average all similarities
        if similarities.is_empty() {
            1.0
        } else {
            similarities.iter().sum::<f64>() / similarities.len() as f64
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate overall similarity between runs
    fn calculate_overall_similarity(
        &self,
        base_frames: &[ReplayFrame],
        compare_frames: &[ReplayFrame],
    ) -> f64 {
        let common_ticks = self.get_common_ticks(base_frames, compare_frames);
        if common_ticks.is_empty() {
            return 0.0;
        }

        let mut similarities = Vec::new();
        for tick in common_ticks {
            if let (Some(base), Some(compare)) = (
                self.get_frame_at_tick(base_frames, tick),
                self.get_frame_at_tick(compare_frames, tick),
            ) {
                similarities.push(self.calculate_frame_similarity(&base, &compare));
            }
        }

        if similarities.is_empty() {
            0.0
        } else {
            similarities.iter().sum::<f64>() / similarities.len() as f64
        }
    }

    /// Calculate correlation coefficient between two series
    fn calculate_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        let clean: Vec<(f64, f64)> = x
            .iter()
            .zip(y.iter())
            .filter(|(&a, &b)| a.is_finite() && b.is_finite())
            .map(|(&a, &b)| (a, b))
            .collect();

        let n = clean.len();
        if n < 2 {
            return 0.0;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mean_x: f64 = clean.iter().map(|p| p.0).sum::<f64>() / n as f64;
        let mean_y: f64 = clean.iter().map(|p| p.1).sum::<f64>() / n as f64;

        let mut numerator = 0.0;
        let mut denom_x = 0.0;
        let mut denom_y = 0.0;

        for (xi, yi) in clean {
            let diff_x = xi - mean_x;
            let diff_y = yi - mean_y;
            numerator += diff_x * diff_y;
            denom_x += diff_x * diff_x;
            denom_y += diff_y * diff_y;
        }

        let denominator = (denom_x * denom_y).sqrt();
        if denominator < 1e-15 {
            0.0
        } else {
            (numerator / denominator).clamp(-1.0, 1.0)
        }
    }

    /// Calculate Euclidean distance
    fn euclidean_distance(&self, a: (f64, f64), b: (f64, f64)) -> f64 {
        (a.0 - b.0).hypot(a.1 - b.1)
    }

    /// Get set of common ticks between two frame collections
    fn get_common_ticks(
        &self,
        base_frames: &[ReplayFrame],
        compare_frames: &[ReplayFrame],
    ) -> Vec<u64> {
        let base_ticks: HashSet<u64> = base_frames.iter().map(|f| f.tick).collect();
        let compare_ticks: HashSet<u64> = compare_frames.iter().map(|f| f.tick).collect();

        let mut common: Vec<u64> = base_ticks.intersection(&compare_ticks).copied().collect();
        common.sort_unstable();
        common
    }

    /// Get frame at specific tick
    fn get_frame_at_tick(&self, frames: &[ReplayFrame], tick: u64) -> Option<ReplayFrame> {
        frames.iter().find(|f| f.tick == tick).cloned()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for DiffEngine {
    fn default() -> Self {
        Self::new()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Run diff result storage
#[derive(Clone, Debug)]
pub struct RunDiff {
    /// Base run ID
    pub base_run_id: String,
    /// Comparison run ID
    pub compare_run_id: String,
    /// Comparison timestamp
    pub compared_at: String,
    /// Comparison result
    pub result: ComparisonResult,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::types::{AgentState, EnvironmentState};

    fn create_test_frame(tick: u64, agent_offset: f64) -> ReplayFrame {
        ReplayFrame {
            tick,
            timestamp: chrono::Utc::now().to_rfc3339(),
            agents: vec![AgentState {
                agent_id: "agent-1".to_string(),
                position: (tick as f64 + agent_offset, tick as f64 + agent_offset),
                velocity: (1.0, 1.0),
                orientation: 0.0,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                state: "moving".to_string(),
                energy: 100.0,
                health: 100.0,
                properties: HashMap::new(),
            }],
            environment: EnvironmentState {
                time_of_day: 12.0,
                weather: "clear".to_string(),
                temperature: 20.0,
                resources: vec![],
                regions: HashMap::new(),
            },
            events: vec![],
            metrics: {
                let mut m = HashMap::new();
                m.insert("metric1".to_string(), tick as f64);
                m
            },
        }
    }

    #[test]
    fn test_compare_similar_runs() {
        let engine = DiffEngine::new();

        // Create two nearly identical runs
        let base_frames: Vec<_> = (0..10).map(|i| create_test_frame(i, 0.0)).collect();
        let compare_frames: Vec<_> = (0..10).map(|i| create_test_frame(i, 0.1)).collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let result = engine.compare_runs("run-1", "run-2", &base_frames, &compare_frames);

        // Should be highly similar
        assert!(result.similarity_score > 0.9);
        assert!(result.divergence_points.is_empty());
    }

    #[test]
    fn test_compare_divergent_runs() {
        let engine = DiffEngine::new();

        // Create two very different runs
        let base_frames: Vec<_> = (0..10).map(|i| create_test_frame(i, 0.0)).collect();
        let compare_frames: Vec<_> = (0..10).map(|i| create_test_frame(i, 100.0)).collect();

        let result = engine.compare_runs("run-1", "run-2", &base_frames, &compare_frames);

        // Should be less similar
        assert!(result.similarity_score < 0.8);
    }

    #[test]
    fn test_correlation() {
        let engine = DiffEngine::new();

        // Perfect correlation
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let corr = engine.calculate_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 0.001);

        // Negative correlation
        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let corr_neg = engine.calculate_correlation(&x, &y_neg);
        assert!((corr_neg - (-1.0)).abs() < 0.001);
    }
}
