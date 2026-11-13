//! Opinion Dynamics (P2-RES-1)
//!
//! Bounded confidence and opinion formation (Hegselmann & Krause 2002)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Bounded confidence model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundedConfidenceConfig {
    /// Confidence bound (agents interact if opinions differ by less than this)
    pub epsilon: f32,
    /// Influence weight (how much agents update)
    pub mu: f32,
    /// Number of agents
    pub n_agents: usize,
}

impl Default for BoundedConfidenceConfig {
    fn default() -> Self {
        Self {
            epsilon: 0.2,
            mu: 0.5,
            n_agents: 100,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Agent opinion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opinion {
    /// agent id.
    pub agent_id: u64,
    /// value.
    pub value: f32, // 0-1 range
    /// confidence.
    pub confidence: f32, // Self-confidence in opinion
    /// persuasiveness.
    pub persuasiveness: f32, // Ability to influence others
}

/// Bounded confidence model (Hegselmann-Krause)
#[derive(Debug, Clone)]
pub struct BoundedConfidence {
    /// config.
    pub config: BoundedConfidenceConfig,
    /// opinions.
    pub opinions: Vec<Opinion>,
    /// step count.
    pub step_count: u64,
}

impl BoundedConfidence {
    #[must_use]
    /// new.
    pub fn new(config: BoundedConfidenceConfig) -> Self {
        let mut opinions = Vec::with_capacity(config.n_agents);
        for i in 0..config.n_agents {
            opinions.push(Opinion {
                agent_id: i as u64,
                value: rand::random::<f32>(),
                confidence: 0.5 + rand::random::<f32>() * 0.5,
                persuasiveness: 0.3 + rand::random::<f32>() * 0.7,
            });
        }

        Self {
            config,
            opinions,
            step_count: 0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get neighbors (agents within confidence bound)
    #[must_use]
    pub fn get_neighbors(&self, agent_id: u64) -> Vec<&Opinion> {
        let Some(agent) = self.opinions.iter().find(|o| o.agent_id == agent_id) else {
            return Vec::new();
        };
        self.opinions
            .iter()
            .filter(|o| {
                o.agent_id != agent_id && (o.value - agent.value).abs() <= self.config.epsilon
            })
            .collect()
    }

    /// Single step update
    pub fn step(&mut self) -> OpinionDynamicsResult {
        let mut new_values: HashMap<u64, f32> = HashMap::new();

        for opinion in &self.opinions {
            let neighbors = self.get_neighbors(opinion.agent_id);

            if neighbors.is_empty() {
                new_values.insert(opinion.agent_id, opinion.value);
                continue;
            }

            // Calculate weighted average of neighbors
            let mut weighted_sum = opinion.value * opinion.confidence;
            let mut total_weight = opinion.confidence;

            for neighbor in neighbors {
                let weight = neighbor.persuasiveness;
                weighted_sum += neighbor.value * weight;
                total_weight += weight;
            }

            let average = weighted_sum / total_weight;
            let new_value = opinion.value + self.config.mu * (average - opinion.value);

            new_values.insert(opinion.agent_id, new_value.clamp(0.0, 1.0));
        }

        // Update all opinions
        for opinion in &mut self.opinions {
            if let Some(&new_value) = new_values.get(&opinion.agent_id) {
                opinion.value = new_value;
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        self.step_count += 1;

        OpinionDynamicsResult {
            step: self.step_count,
            consensus_reached: self.check_consensus(0.01),
            num_clusters: self.count_clusters(),
            opinion_range: self.opinion_range(),
        }
    }

    /// Check if consensus reached
    fn check_consensus(&self, tolerance: f32) -> bool {
        if self.opinions.is_empty() {
            return true;
        }
        let values: Vec<f32> = self.opinions.iter().map(|o| o.value).collect();
        let min = values.iter().copied().fold(f32::INFINITY, f32::min);
        let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (max - min) <= tolerance
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Count opinion clusters
    #[must_use]
    pub fn count_clusters(&self) -> usize {
        if self.opinions.is_empty() {
            return 0;
        }

        let mut values: Vec<f32> = self.opinions.iter().map(|o| o.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut clusters = 1;
        for i in 1..values.len() {
            if (values[i] - values[i - 1]) > self.config.epsilon {
                clusters += 1;
            }
        }

        clusters
    }

    /// Get opinion range
    pub fn opinion_range(&self) -> f32 {
        if self.opinions.is_empty() {
            return 0.0;
        }
        let values: Vec<f32> = self.opinions.iter().map(|o| o.value).collect();
        let min = values.iter().copied().fold(f32::INFINITY, f32::min);
        let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (max - min).max(0.0)
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Run until convergence
    pub fn run_until_convergence(&mut self, max_steps: u64, tolerance: f32) -> ConvergenceResult {
        for _ in 0..max_steps {
            let result = self.step();

            if self.check_consensus(tolerance) {
                return ConvergenceResult {
                    converged: true,
                    steps: result.step,
                    final_clusters: result.num_clusters,
                    final_range: result.opinion_range,
                };
            }
        }

        ConvergenceResult {
            converged: false,
            steps: max_steps,
            final_clusters: self.count_clusters(),
            final_range: self.opinion_range(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for BoundedConfidence {
    fn default() -> Self {
        Self::new(BoundedConfidenceConfig::default())
    }
}

/// Opinion dynamics model (general)
#[derive(Debug, Clone)]
pub struct OpinionDynamicsModel {
    /// bounded confidence.
    pub bounded_confidence: BoundedConfidence,
}

impl OpinionDynamicsModel {
    #[must_use]
    /// new.
    pub fn new(config: BoundedConfidenceConfig) -> Self {
        Self {
            bounded_confidence: BoundedConfidence::new(config),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Run model
    pub fn run(&mut self, steps: u64) -> Vec<OpinionDynamicsResult> {
        (0..steps).map(|_| self.bounded_confidence.step()).collect()
    }
}

/// Result of opinion dynamics step
#[derive(Debug, Clone, Copy)]
pub struct OpinionDynamicsResult {
    /// step.
    pub step: u64,
    /// consensus reached.
    pub consensus_reached: bool,
    /// num clusters.
    pub num_clusters: usize,
    /// opinion range.
    pub opinion_range: f32,
}

/// Convergence result
#[derive(Debug, Clone, Copy)]
pub struct ConvergenceResult {
    /// converged.
    pub converged: bool,
    /// steps.
    pub steps: u64,
    /// final clusters.
    pub final_clusters: usize,
    /// final range.
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub final_range: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_confidence() {
        let mut model = BoundedConfidence::default();

        let initial_range = model.opinion_range();

        // Run several steps
        for _ in 0..10 {
            model.step();
        }

        let final_range = model.opinion_range();
        // Range should decrease or stay same
        assert!(final_range <= initial_range);
    }

    #[test]
    fn test_cluster_formation() {
        let mut model = BoundedConfidence::new(BoundedConfidenceConfig {
            epsilon: 0.15,
            mu: 0.5,
            n_agents: 50,
        });

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let result = model.run_until_convergence(100, 0.05);

        // Should converge to one or few clusters
        assert!(result.final_clusters <= 5);
    }

    #[test]
    fn test_neighbors() {
        let mut model = BoundedConfidence::new(BoundedConfidenceConfig {
            epsilon: 0.1, // Smaller epsilon for precise control
            mu: 0.5,
            n_agents: 10,
        });

        // Set specific values
        model.opinions[0].value = 0.5;
        model.opinions[1].value = 0.55; // Within epsilon (0.05 < 0.1)
        model.opinions[2].value = 0.9; // Outside epsilon

        // Set all other agents far away
        for i in 3..10 {
            model.opinions[i].value = 0.9 + (i as f32 * 0.01);
        }

        let neighbors = model.get_neighbors(0);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].agent_id, 1);
    }
}
