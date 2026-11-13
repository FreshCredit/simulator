//! Representative agent sampling for neural validation
//!
//! Implements stratified sampling strategies to select representative
//! agents for neural simulation, ensuring coverage of the agent population.

use crate::agent::Agent;
use crate::types::AgentId;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sampling strategy
#[derive(Debug, Clone)]
pub enum SamplingStrategy {
    /// Random sampling
    Random {
        /// count.
        count: usize,
    },
    /// Stratified by DOCAS state
    StratifiedByDocas {
        /// count.
        count: usize,
    },
    /// Representative sampling (default)
    Representative {
        /// count.
        count: usize,
    },
    /// Coverage-based sampling
    Coverage {
        /// min per cluster.
        min_per_cluster: usize,
        /// max clusters.
        max_clusters: usize,
    },
    /// Importance sampling (focus on high-variance agents)
    Importance {
        /// count.
        count: usize,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Agent sampler
#[derive(Debug)]
pub struct AgentSampler;

/// Sampled agent information
#[derive(Debug, Clone)]
pub struct SampledAgent {
    /// agent id.
    pub agent_id: AgentId,
    /// agent.
    pub agent: Agent,
    /// weight.
    pub weight: f32,
    /// stratum.
    pub stratum: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sampling statistics
#[derive(Debug, Clone, Default)]
pub struct SamplingStats {
    /// total agents.
    pub total_agents: usize,
    /// sampled agents.
    pub sampled_agents: usize,
    /// strata distribution.
    pub strata_distribution: HashMap<String, usize>,
    /// coverage score.
    pub coverage_score: f32,
}

impl AgentSampler {
    /// Sample agents according to strategy
    pub fn sample(agents: &[Agent], strategy: &SamplingStrategy, rng: &mut StdRng) -> Vec<Agent> {
        match strategy {
            SamplingStrategy::Random { count } => Self::sample_random(agents, *count, rng),
            SamplingStrategy::StratifiedByDocas { count } => {
                Self::sample_stratified_docas(agents, *count, rng)
            }
            SamplingStrategy::Representative { count } => {
                Self::sample_representative(agents, *count, rng)
            }
            SamplingStrategy::Coverage {
                min_per_cluster,
                max_clusters,
            } => Self::sample_coverage(agents, *min_per_cluster, *max_clusters, rng),
            SamplingStrategy::Importance { count } => Self::sample_importance(agents, *count, rng),
        }
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Simple random sampling
    fn sample_random(agents: &[Agent], count: usize, rng: &mut StdRng) -> Vec<Agent> {
        let count = count.min(agents.len());
        let indices: Vec<usize> = (0..agents.len()).collect();
        indices
            .choose_multiple(rng, count)
            .copied()
            .map(|i| agents[i].clone())
            .collect()
    }

    /// Stratified sampling by DOCAS quadrant
    fn sample_stratified_docas(agents: &[Agent], count: usize, rng: &mut StdRng) -> Vec<Agent> {
        // Define strata based on DOCAS state
        let mut strata: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, agent) in agents.iter().enumerate() {
            let docas = &agent.docas;
            let quadrant = Self::docas_quadrant(docas);
            strata.entry(quadrant).or_default().push(i);
        }

        // Sample proportionally from each stratum
        let n_strata = strata.len().max(1);
        let per_stratum = count / n_strata;
        let extra = count % n_strata;

        let mut sampled = Vec::new();
        let stratum_iter = strata.iter().enumerate();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for (stratum_idx, (_name, indices)) in stratum_iter {
            let n_to_sample = per_stratum + usize::from(stratum_idx < extra);
            let n_to_sample = n_to_sample.min(indices.len());

            let selected: Vec<usize> = indices.choose_multiple(rng, n_to_sample).copied().collect();
            for idx in selected {
                sampled.push(agents[idx].clone());
            }
        }

        sampled
    }

    /// Representative sampling (default strategy)
    fn sample_representative(agents: &[Agent], count: usize, rng: &mut StdRng) -> Vec<Agent> {
        if agents.len() <= count {
            return agents.to_vec();
        }

        // Use k-means++ style initialization for representative sampling
        let mut selected = Vec::with_capacity(count);
        let mut selected_indices: Vec<usize> = Vec::new();

        // Pick first agent randomly
        let first_idx = rng.gen_range(0..agents.len());
        selected.push(agents[first_idx].clone());
        selected_indices.push(first_idx);

        // Greedy k-means++: pick points farthest from existing selections
        // P10-R4: bounded — selected grows to count, one point per iteration.
        while selected.len() < count {
            let mut max_min_dist = 0.0;
            let mut max_idx = 0;

            for (i, agent) in agents.iter().enumerate() {
                if selected_indices.contains(&i) {
                    continue;
                }

                let min_dist = selected_indices
                    .iter()
                    .map(|&j| Self::agent_distance(agent, &agents[j]))
                    .fold(f32::INFINITY, f32::min);

                // TAG: surface=simulation owner=research-team rule=SIM-001
                if min_dist > max_min_dist {
                    max_min_dist = min_dist;
                    max_idx = i;
                }
            }

            selected.push(agents[max_idx].clone());
            selected_indices.push(max_idx);
        }

        selected
    }

    /// Coverage-based sampling
    fn sample_coverage(
        agents: &[Agent],
        min_per_cluster: usize,
        max_clusters: usize,
        rng: &mut StdRng,
    ) -> Vec<Agent> {
        // Simple k-means clustering on DOCAS features
        let k = (agents.len() / (min_per_cluster * 2)).clamp(2, max_clusters);
        let clusters = Self::kmeans_clustering(agents, k, rng);

        // Sample from each cluster
        let mut sampled = Vec::new();
        for cluster in clusters {
            let n_to_sample = min_per_cluster.min(cluster.len());
            let selected: Vec<usize> = cluster.choose_multiple(rng, n_to_sample).copied().collect();
            for idx in selected {
                sampled.push(agents[idx].clone());
            }
        }

        sampled
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Importance sampling (focus on high-variance agents)
    fn sample_importance(agents: &[Agent], count: usize, rng: &mut StdRng) -> Vec<Agent> {
        // Compute importance weights based on DOCAS variance
        let mut weights: Vec<f32> = agents
            .iter()
            .map(|a| {
                let variance = a
                    .docas
                    .levels
                    .iter()
                    .map(|&x| (x - 0.5).powi(2))
                    .sum::<f32>();
                variance.sqrt() + 0.1 // Add small constant for exploration
            })
            .collect();

        // Normalize weights
        let total_weight: f32 = weights.iter().sum();
        for w in &mut weights {
            *w /= total_weight;
        }

        // Weighted sampling without replacement
        let count = count.min(agents.len());
        let mut selected = Vec::with_capacity(count);
        let mut remaining: Vec<usize> = (0..agents.len()).collect();
        let mut remaining_weights: Vec<f32> = weights.clone();

        for _ in 0..count {
            if remaining.is_empty() {
                break;
            }

            // Weighted random selection
            let total: f32 = remaining_weights.iter().sum();
            let mut rand_val: f32 = rng.gen::<f32>() * total;

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let mut selected_idx = 0;
            for (i, &weight) in remaining_weights.iter().enumerate() {
                rand_val -= weight;
                if rand_val <= 0.0 {
                    selected_idx = i;
                    break;
                }
            }

            selected.push(agents[remaining[selected_idx]].clone());
            remaining.remove(selected_idx);
            remaining_weights.remove(selected_idx);
        }

        selected
    }

    /// Compute DOCAS quadrant for an agent
    fn docas_quadrant(docas: &crate::docas::DocasState) -> String {
        let d = docas.levels[0]; // Dopamine
        let c = docas.levels[2]; // Cortisol

        match (d > 0.5, c > 0.5) {
            (true, false) => "high_dop_low_stress".to_string(),
            (true, true) => "high_dop_high_stress".to_string(),
            (false, false) => "low_dop_low_stress".to_string(),
            (false, true) => "low_dop_high_stress".to_string(),
        }
    }

    /// Compute distance between two agents (for clustering)
    fn agent_distance(a1: &Agent, a2: &Agent) -> f32 {
        let docas_dist: f32 = a1
            .docas
            .levels
            .iter()
            .zip(a2.docas.levels.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let affect_dist: f32 = (a1.affect_valence - a2.affect_valence).powi(2)
            + (a1.affect_arousal - a2.affect_arousal).powi(2);

        (docas_dist + affect_dist).sqrt()
    }

    /// Simple k-means clustering
    fn kmeans_clustering(agents: &[Agent], k: usize, rng: &mut StdRng) -> Vec<Vec<usize>> {
        // Random initialization
        let mut centroids: Vec<[f32; 5]> = (0..k)
            .map(|_| {
                let idx = rng.gen_range(0..agents.len());
                agents[idx].docas.levels
            })
            .collect();

        let mut assignments = vec![0; agents.len()];
        let max_iters = 20;

        for _ in 0..max_iters {
            // Assign each agent to nearest centroid
            let mut changed = false;
            for (i, agent) in agents.iter().enumerate() {
                let mut min_dist = f32::INFINITY;
                let mut min_cluster = 0;

                for (j, centroid) in centroids.iter().enumerate() {
                    let dist: f32 = agent
                        .docas
                        .levels
                        .iter()
                        .zip(centroid.iter())
                        .map(|(a, c)| (a - c).powi(2))
                        .sum::<f32>();

                    if dist < min_dist {
                        min_dist = dist;
                        min_cluster = j;
                    }
                }

                // TAG: surface=simulation owner=research-team rule=SIM-001
                if assignments[i] != min_cluster {
                    changed = true;
                    assignments[i] = min_cluster;
                }
            }

            if !changed {
                break;
            }

            // Update centroids
            #[allow(clippy::needless_range_loop)]
            for j in 0..k {
                let cluster_members: Vec<usize> = assignments
                    .iter()
                    .enumerate()
                    .filter(|(_, &a)| a == j)
                    .map(|(i, _)| i)
                    .collect();

                if !cluster_members.is_empty() {
                    for m in 0..5 {
                        let sum: f32 = cluster_members
                            .iter()
                            .map(|&i| agents[i].docas.levels[m])
                            .sum();
                        centroids[j][m] = sum / cluster_members.len() as f32;
                    }
                }
            }
        }

        // Build cluster vectors
        let mut clusters: Vec<Vec<usize>> = (0..k).map(|_| Vec::new()).collect();
        for (i, &cluster) in assignments.iter().enumerate() {
            clusters[cluster].push(i);
        }

        clusters.into_iter().filter(|c| !c.is_empty()).collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute sampling statistics
    #[must_use]
    pub fn compute_stats(agents: &[Agent], sampled: &[Agent]) -> SamplingStats {
        let strata_distribution: HashMap<String, usize> = sampled
            .iter()
            .map(|a| (Self::docas_quadrant(&a.docas), 1))
            .fold(HashMap::new(), |mut acc, (k, v)| {
                *acc.entry(k).or_insert(0) += v;
                acc
            });

        let total_strata = 4; // 2x2 quadrant grid
        let covered_strata = strata_distribution.len();
        let coverage_score = covered_strata as f32 / total_strata as f32;

        SamplingStats {
            total_agents: agents.len(),
            sampled_agents: sampled.len(),
            strata_distribution,
            coverage_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn create_test_agents(n: usize) -> Vec<Agent> {
        let mut agents = Vec::with_capacity(n);
        let mut rng = StdRng::seed_from_u64(42);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for i in 0..n {
            use crate::genome::Genome;
            use crate::types::{Position, Sex};

            let genome = Genome::random(&mut rng);
            let mut agent = Agent::new(
                i as u64,
                Position::new(i as f32, i as f32, 0.0),
                genome,
                0,
                if i % 2 == 0 { Sex::Male } else { Sex::Female },
            );

            // Randomize DOCAS levels
            agent.docas.levels = [
                rng.gen::<f32>(),
                rng.gen::<f32>(),
                rng.gen::<f32>(),
                rng.gen::<f32>(),
                rng.gen::<f32>(),
            ];

            agents.push(agent);
        }

        agents
    }

    #[test]
    fn test_random_sampling() {
        let agents = create_test_agents(100);
        let mut rng = StdRng::seed_from_u64(42);

        let sampled =
            AgentSampler::sample(&agents, &SamplingStrategy::Random { count: 10 }, &mut rng);

        assert_eq!(sampled.len(), 10);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_representative_sampling() {
        let agents = create_test_agents(100);
        let mut rng = StdRng::seed_from_u64(42);

        let sampled = AgentSampler::sample(
            &agents,
            &SamplingStrategy::Representative { count: 10 },
            &mut rng,
        );

        assert_eq!(sampled.len(), 10);

        // Check coverage
        let stats = AgentSampler::compute_stats(&agents, &sampled);
        assert!(stats.coverage_score > 0.0);
    }

    #[test]
    fn test_stratified_sampling() {
        let agents = create_test_agents(100);
        let mut rng = StdRng::seed_from_u64(42);

        let sampled = AgentSampler::sample(
            &agents,
            &SamplingStrategy::StratifiedByDocas { count: 20 },
            &mut rng,
        );

        assert_eq!(sampled.len(), 20);

        let stats = AgentSampler::compute_stats(&agents, &sampled);
        assert!(stats.strata_distribution.len() >= 2);
    }
}
