//! Social Network Formation Module (P1-AGENT-5)
//!
//! Dynamic relationship building between agents.

pub mod dynamics;
pub mod network;
pub mod relationship;

pub use dynamics::{NetworkDynamics, TieFormation};
pub use network::{NetworkMetrics, SocialNetwork};
pub use relationship::{Relationship, RelationshipManager, RelationshipType};

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Social system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialConfig {
    /// Maximum number of ties an agent can maintain
    pub max_ties: usize,
    /// Decay rate for inactive relationships
    pub tie_decay_rate: f32,
    /// Threshold for considering someone a friend
    pub friendship_threshold: f32,
    /// Homophily strength
    pub homophily: f32,
    /// Reciprocity expectation
    pub reciprocity: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for SocialConfig {
    fn default() -> Self {
        Self {
            max_ties: 50,
            tie_decay_rate: 0.01,
            friendship_threshold: 0.5,
            homophily: 0.3,
            reciprocity: 0.8,
        }
    }
}

/// Social system managing all agent relationships
#[derive(Debug, Clone)]
pub struct SocialSystem {
    /// config.
    pub config: SocialConfig,
    /// networks.
    pub networks: Vec<SocialNetwork>,
    /// relationship manager.
    pub relationship_manager: RelationshipManager,
    /// dynamics.
    pub dynamics: NetworkDynamics,
}

impl SocialSystem {
    /// Create new social system
    #[must_use]
    pub fn new(config: SocialConfig) -> Self {
        Self {
            config: config.clone(),
            networks: Vec::new(),
            relationship_manager: RelationshipManager::new(),
            dynamics: NetworkDynamics::new(config),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add agent to social system
    pub fn add_agent(&mut self, agent_id: u64) {
        self.relationship_manager.add_agent(agent_id);
    }

    /// Update all relationships
    pub fn update(&mut self, interactions: &[Interaction]) {
        // Update relationships based on interactions
        for interaction in interactions {
            self.relationship_manager
                .update_from_interaction(interaction);
        }

        // Apply network dynamics
        self.dynamics
            .update_networks(&mut self.relationship_manager);

        // Decay old relationships
        self.relationship_manager
            .decay_relationships(self.config.tie_decay_rate);
    }

    /// Get network for specific type
    #[must_use]
    pub fn get_network(&self, network_type: NetworkType) -> Option<&SocialNetwork> {
        self.networks
            .iter()
            .find(|n| n.network_type == network_type)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get agent's social position
    #[must_use]
    pub fn get_social_position(&self, agent_id: u64) -> Option<SocialPosition> {
        let relationships = self.relationship_manager.get_relationships(agent_id);
        if relationships.is_empty() {
            return None;
        }

        let degree = relationships.len();
        let strength_sum: f32 = relationships.iter().map(|r| r.strength).sum();
        let avg_strength = strength_sum / degree as f32;

        Some(SocialPosition {
            agent_id,
            degree,
            avg_relationship_strength: avg_strength,
            clustering_coefficient: self.calculate_clustering(agent_id),
        })
    }

    /// Calculate clustering coefficient for agent
    fn calculate_clustering(&self, agent_id: u64) -> f32 {
        let neighbors: Vec<u64> = self
            .relationship_manager
            .get_relationships(agent_id)
            .iter()
            .map(|r| {
                if r.agent_a == agent_id {
                    r.agent_b
                } else {
                    r.agent_a
                }
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            })
            .collect();

        if neighbors.len() < 2 {
            return 0.0;
        }

        let mut edges_between_neighbors = 0;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                if self
                    .relationship_manager
                    .has_relationship(neighbors[i], neighbors[j])
                {
                    edges_between_neighbors += 1;
                }
            }
        }

        // P1-FIX: Use checked arithmetic to prevent overflow
        let max_edges = neighbors
            .len()
            .checked_mul(neighbors.len().saturating_sub(1))
            .map_or(0, |v| v / 2);
        if max_edges == 0 {
            0.0
        } else {
            edges_between_neighbors as f32 / max_edges as f32
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for SocialSystem {
    fn default() -> Self {
        Self::new(SocialConfig::default())
    }
}

/// Type of social network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkType {
    /// friendship.
    Friendship,
    /// trust.
    Trust,
    /// advice.
    Advice,
    /// cooperation.
    Cooperation,
    /// influence.
    Influence,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Social interaction event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// agent a.
    pub agent_a: u64,
    /// agent b.
    pub agent_b: u64,
    /// interaction type.
    pub interaction_type: InteractionType,
    /// outcome.
    pub outcome: f32, // -1.0 (negative) to 1.0 (positive)
    /// timestamp.
    pub timestamp: u64,
}

/// Type of interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionType {
    /// cooperation.
    Cooperation,
    /// competition.
    Competition,
    /// exchange.
    Exchange,
    /// communication.
    Communication,
    /// conflict.
    Conflict,
    /// support.
    Support,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Social position of an agent
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SocialPosition {
    /// agent id.
    pub agent_id: u64,
    /// degree.
    pub degree: usize,
    /// avg relationship strength.
    pub avg_relationship_strength: f32,
    /// clustering coefficient.
    pub clustering_coefficient: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_system() {
        let mut system = SocialSystem::default();
        system.add_agent(1);
        system.add_agent(2);

        // Multiple interactions to build relationship strength
        let interactions: Vec<Interaction> = (0..5)
            .map(|i| Interaction {
                agent_a: 1,
                agent_b: 2,
                interaction_type: InteractionType::Cooperation,
                outcome: 0.8,
                timestamp: i as u64,
            })
            .collect();

        system.update(&interactions);

        let position = system.get_social_position(1);
        assert!(position.is_some());
    }

    #[test]
    fn test_clustering_coefficient() {
        let mut system = SocialSystem::default();
        system.add_agent(1);
        system.add_agent(2);
        system.add_agent(3);

        // Create triangle with multiple interactions
        let interactions: Vec<Interaction> = (0..5)
            // TAG: surface=simulation owner=research-team rule=SIM-001
            .flat_map(|i| {
                vec![
                    Interaction {
                        agent_a: 1,
                        agent_b: 2,
                        interaction_type: InteractionType::Cooperation,
                        outcome: 0.8,
                        timestamp: i as u64,
                    },
                    Interaction {
                        agent_a: 2,
                        agent_b: 3,
                        interaction_type: InteractionType::Cooperation,
                        outcome: 0.8,
                        timestamp: i as u64,
                    },
                    Interaction {
                        agent_a: 1,
                        agent_b: 3,
                        interaction_type: InteractionType::Cooperation,
                        outcome: 0.8,
                        timestamp: i as u64,
                    },
                ]
            })
            .collect();
        system.update(&interactions);

        let position = system.get_social_position(1).unwrap();
        assert_eq!(position.clustering_coefficient, 1.0); // Complete triangle
    }
}
