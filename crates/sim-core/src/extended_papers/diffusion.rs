//! Diffusion Models (P2-RES-1)
//!
//! Complex contagions and innovation diffusion (Centola & Macy 2007)

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Diffusion model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffusionConfig {
    /// Threshold for adoption (number of neighbors needed)
    pub threshold: usize,
    /// Whether contagion requires clustered neighbors (complex vs simple)
    pub requires_clustering: bool,
    /// Decay rate for influence
    pub decay_rate: f32,
}

impl Default for DiffusionConfig {
    fn default() -> Self {
        Self {
            threshold: 2,
            requires_clustering: true,
            decay_rate: 0.1,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Innovation being diffused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Innovation {
    /// id.
    pub id: String,
    /// name.
    pub name: String,
    /// Relative advantage
    pub advantage: f32,
    /// Compatibility with existing practices
    pub compatibility: f32,
    /// Complexity (inverse of ease of use)
    pub complexity: f32,
    /// Trialability
    pub trialability: f32,
    /// Observability of results
    pub observability: f32,
}

impl Innovation {
    /// Calculate overall adoption propensity
    #[must_use]
    pub fn adoption_propensity(&self) -> f32 {
        let idt_score =
            (self.advantage + self.compatibility + self.trialability + self.observability) / 4.0
                - self.complexity * 0.5;
        idt_score.clamp(0.0, 1.0)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Agent state in diffusion process
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdoptionState {
    /// unaware.
    Unaware,
    /// aware.
    Aware,
    /// adopted.
    Adopted,
    /// rejected.
    Rejected,
}

/// Network edge for diffusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffusionEdge {
    /// from.
    pub from: u64,
    /// to.
    pub to: u64,
    /// weight.
    pub weight: f32,
    /// is clustered.
    pub is_clustered: bool, // Part of triad?
}

/// Diffusion network
#[derive(Debug, Clone)]
pub struct DiffusionNetwork {
    /// config.
    pub config: DiffusionConfig,
    /// edges.
    pub edges: Vec<DiffusionEdge>,
    /// agent states.
    pub agent_states: HashMap<u64, AdoptionState>,
    /// exposure counts.
    pub exposure_counts: HashMap<u64, u32>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DiffusionNetwork {
    #[must_use]
    /// new.
    pub fn new(config: DiffusionConfig) -> Self {
        Self {
            config,
            edges: Vec::new(),
            agent_states: HashMap::new(),
            exposure_counts: HashMap::new(),
        }
    }

    /// Add edge
    pub fn add_edge(&mut self, edge: DiffusionEdge) {
        self.edges.push(edge);
    }

    /// Initialize all agents
    pub fn add_agent(&mut self, agent_id: u64, initial_state: AdoptionState) {
        self.agent_states.insert(agent_id, initial_state);
    }

    /// Get neighbors
    #[must_use]
    pub fn neighbors(&self, agent_id: u64) -> Vec<u64> {
        self.edges
            .iter()
            .filter(|e| e.from == agent_id || e.to == agent_id)
            .map(|e| if e.from == agent_id { e.to } else { e.from })
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get adopted neighbors
    #[must_use]
    pub fn adopted_neighbors(&self, agent_id: u64) -> Vec<u64> {
        self.neighbors(agent_id)
            .into_iter()
            .filter(|&n| matches!(self.agent_states.get(&n), Some(AdoptionState::Adopted)))
            .collect()
    }

    /// Check if neighbors form a cluster
    #[must_use]
    pub fn is_clustered(&self, agent_id: u64) -> bool {
        let adopted = self.adopted_neighbors(agent_id);
        if adopted.len() < 2 {
            return false;
        }

        // Check if any two adopted neighbors are connected
        let neighbor_set: HashSet<u64> = adopted.iter().copied().collect();
        self.edges.iter().any(|e| {
            let a = e.from;
            let b = e.to;
            neighbor_set.contains(&a) && neighbor_set.contains(&b)
        })
    }

    /// Step diffusion
    pub fn step(&mut self, innovation: &Innovation) -> DiffusionStepResult {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut adoptions = 0;
        let mut rejections = 0;
        let mut awareness = 0;

        let mut agent_ids: Vec<u64> = self.agent_states.keys().copied().collect();
        agent_ids.sort_unstable();

        let state_snapshot = self.agent_states.clone();
        let mut state_updates = Vec::new();

        for &agent_id in &agent_ids {
            let current_state = match state_snapshot.get(&agent_id) {
                Some(s) => *s,
                None => continue,
            };

            if current_state == AdoptionState::Adopted || current_state == AdoptionState::Rejected {
                continue;
            }

            // Count adopted neighbors in frozen snapshot
            let exposure = self
                .neighbors(agent_id)
                .into_iter()
                .filter(|&n| matches!(state_snapshot.get(&n), Some(AdoptionState::Adopted)))
                .count();

            if exposure > 0 {
                let exposure_count = self.exposure_counts.entry(agent_id).or_insert(0);
                *exposure_count += 1;
                let total_exposures = *exposure_count;

                // Move to aware state
                if current_state == AdoptionState::Unaware {
                    state_updates.push((agent_id, AdoptionState::Aware));
                    awareness += 1;
                }

                // TAG: surface=simulation owner=research-team rule=SIM-001
                // Check adoption criteria
                let meets_threshold = exposure >= self.config.threshold;
                let meets_clustering =
                    !self.config.requires_clustering || self.is_clustered(agent_id);

                if meets_threshold && meets_clustering {
                    // Probabilistic adoption with Centola & Macy (2007) exposure decay
                    let propensity = innovation.adoption_propensity();
                    let social_influence = (exposure as f32 * 0.1).min(0.5);
                    let excess_exposure = exposure.saturating_sub(self.config.threshold) as i32;
                    let decay =
                        (1.0 - self.config.decay_rate.clamp(0.0, 1.0)).powi(excess_exposure);
                    let adoption_prob =
                        (propensity + social_influence * (1.0 - propensity)) * decay;

                    let hash = agent_id
                        .wrapping_mul(0x9e3779b97f4a7c15)
                        .wrapping_add(exposure as u64);
                    let p = ((hash >> 32) as u32 as f32) / (u32::MAX as f32);

                    if p < adoption_prob {
                        state_updates.push((agent_id, AdoptionState::Adopted));
                        adoptions += 1;
                    } else if total_exposures >= (self.config.threshold * 4) as u32 {
                        state_updates.push((agent_id, AdoptionState::Rejected));
                        rejections += 1;
                    }
                }
            }
        }

        for (id, state) in state_updates {
            self.agent_states.insert(id, state);
        }

        DiffusionStepResult {
            adoptions,
            rejections,
            awareness,
            total_adopted: self.adopted_count(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Count adopted agents
    #[must_use]
    pub fn adopted_count(&self) -> usize {
        self.agent_states
            .values()
            .filter(|s| matches!(s, AdoptionState::Adopted))
            .count()
    }

    /// Calculate adoption rate
    #[must_use]
    pub fn adoption_rate(&self) -> f32 {
        if self.agent_states.is_empty() {
            return 0.0;
        }
        self.adopted_count() as f32 / self.agent_states.len() as f32
    }
}

impl Default for DiffusionNetwork {
    fn default() -> Self {
        Self::new(DiffusionConfig::default())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Result of a diffusion step
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffusionStepResult {
    /// adoptions.
    pub adoptions: u32,
    /// rejections.
    pub rejections: u32,
    /// awareness.
    pub awareness: u32,
    /// total adopted.
    pub total_adopted: usize,
}

/// Simple vs complex contagion comparison
#[derive(Debug, Clone)]
pub struct ContagionComparison {
    /// simple contagion rate.
    pub simple_contagion_rate: f32,
    /// complex contagion rate.
    pub complex_contagion_rate: f32,
    /// network type.
    pub network_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_innovation() -> Innovation {
        Innovation {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            id: "test".to_string(),
            name: "Test Innovation".to_string(),
            advantage: 0.8,
            compatibility: 0.7,
            complexity: 0.3,
            trialability: 0.9,
            observability: 0.8,
        }
    }

    #[test]
    fn test_innovation_propensity() {
        let innovation = create_test_innovation();
        let propensity = innovation.adoption_propensity();
        assert!(propensity > 0.0 && propensity <= 1.0);
    }

    #[test]
    fn test_diffusion_network() {
        let mut network = DiffusionNetwork::default();

        // Add 5 agents
        for i in 0..5 {
            network.add_agent(
                i,
                if i == 0 {
                    AdoptionState::Adopted
                } else {
                    AdoptionState::Unaware
                },
            );
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Create edges (clustered around agent 0)
        network.add_edge(DiffusionEdge {
            from: 0,
            to: 1,
            weight: 1.0,
            is_clustered: true,
        });
        network.add_edge(DiffusionEdge {
            from: 0,
            to: 2,
            weight: 1.0,
            is_clustered: true,
        });
        network.add_edge(DiffusionEdge {
            from: 1,
            to: 2,
            weight: 1.0,
            is_clustered: true,
        }); // Cluster

        assert_eq!(network.adopted_count(), 1);
    }

    #[test]
    fn test_cluster_detection() {
        let mut network = DiffusionNetwork::default();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for i in 0..3 {
            network.add_agent(
                i,
                if i < 2 {
                    AdoptionState::Adopted
                } else {
                    AdoptionState::Unaware
                },
            );
        }

        // Clustered
        network.add_edge(DiffusionEdge {
            from: 0,
            to: 2,
            weight: 1.0,
            is_clustered: true,
        });
        network.add_edge(DiffusionEdge {
            from: 1,
            to: 2,
            weight: 1.0,
            is_clustered: true,
        });
        network.add_edge(DiffusionEdge {
            from: 0,
            to: 1,
            weight: 1.0,
            is_clustered: true,
        });

        // Agent 2's adopted neighbors (0 and 1) are connected
        assert!(network.is_clustered(2));
    }
}
