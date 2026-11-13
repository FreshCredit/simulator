//! Network Dynamics (P1-AGENT-5)

use serde::{Deserialize, Serialize};

use super::{Interaction, InteractionType, RelationshipManager, SocialConfig};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Network dynamics engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDynamics {
    /// config.
    pub config: SocialConfig,
    /// tie formation.
    pub tie_formation: TieFormation,
}

impl NetworkDynamics {
    /// Create new dynamics engine
    #[must_use]
    pub const fn new(config: SocialConfig) -> Self {
        Self {
            config,
            tie_formation: TieFormation::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update networks based on relationships
    pub fn update_networks(&mut self, relationships: &mut RelationshipManager) {
        // Homophily: similar agents form stronger ties
        self.apply_homophily(relationships);

        // Triadic closure: friends of friends become friends
        self.apply_triadic_closure(relationships);

        // Preferential attachment: popular agents gain more ties
        self.apply_preferential_attachment(relationships);

        // Reciprocity: maintain balanced relationships
        self.enforce_reciprocity(relationships);
    }

    /// Apply homophily (similarity attraction)
    fn apply_homophily(&self, _relationships: &mut RelationshipManager) {
        // In a real implementation, would compare agent traits
        // For now, relationships strengthen based on positive interactions
    }

    /// Apply triadic closure
    fn apply_triadic_closure(&self, relationships: &mut RelationshipManager) {
        // Find potential triangles
        let agents: Vec<u64> = relationships.agent_relationships.keys().copied().collect();

        for &agent_a in &agents {
            let neighbors: Vec<u64> = relationships
                .get_relationships(agent_a)
                .iter()
                .filter(|r| r.strength > 0.3)
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                .map(|r| {
                    if r.agent_a == agent_a {
                        r.agent_b
                    } else {
                        r.agent_a
                    }
                })
                .collect();

            for i in 0..neighbors.len() {
                for j in (i + 1)..neighbors.len() {
                    let b = neighbors[i];
                    let c = neighbors[j];

                    // If b and c don't know each other, introduce them
                    if !relationships.has_relationship(b, c) {
                        let prob =
                            self.tie_formation
                                .formation_probability(self.config.homophily, 0.5, 1);
                        if prob > 0.1 {
                            // Potential for new tie
                            let interaction = Interaction {
                                agent_a: b,
                                agent_b: c,
                                interaction_type: InteractionType::Communication,
                                outcome: 0.3, // Neutral-positive introduction
                                timestamp: 0,
                            };
                            relationships.update_from_interaction(&interaction);
                        }
                    }
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply preferential attachment
    fn apply_preferential_attachment(&self, _relationships: &mut RelationshipManager) {
        // Agents with many ties have higher probability of gaining new ties
        // This is naturally modeled by being more likely to interact with them
    }

    /// Enforce reciprocity
    fn enforce_reciprocity(&self, relationships: &mut RelationshipManager) {
        // Find unbalanced relationships
        let rels: Vec<_> = relationships.relationships.values().cloned().collect();

        for rel in rels {
            let reciprocity = rel.reciprocity;
            if reciprocity < self.config.reciprocity && rel.interaction_count > 5 {
                // Imbalanced relationship decays faster
                if let Some(r) = relationships.get_mut(rel.agent_a, rel.agent_b) {
                    r.decay(0.02);
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate tie strength based on factors
    #[must_use]
    pub fn calculate_tie_strength(
        &self,
        interaction_count: u32,
        positive_ratio: f32,
        recency: f32, // 0.0 to 1.0 (1.0 = recent)
        emotional_valence: f32,
    ) -> f32 {
        let frequency = (interaction_count as f32 / 100.0).min(1.0);
        let quality = positive_ratio;

        (frequency * 0.3 + quality * 0.4 + recency * 0.2 + emotional_valence.abs() * 0.1)
            .clamp(0.0, 1.0)
    }
}

/// Tie formation model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieFormation {
    /// formation rate.
    pub formation_rate: f32,
    /// dissolution rate.
    pub dissolution_rate: f32,
    /// homophily weight.
    pub homophily_weight: f32,
    /// proximity weight.
    pub proximity_weight: f32,
}

impl TieFormation {
    /// Create new tie formation model
    #[must_use]
    pub const fn new() -> Self {
        Self {
            formation_rate: 0.1,
            dissolution_rate: 0.05,
            homophily_weight: 0.4,
            proximity_weight: 0.3,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate probability of tie formation
    #[must_use]
    pub fn formation_probability(
        &self,
        similarity: f32,
        proximity: f32,
        mutual_friends: usize,
    ) -> f32 {
        let similarity_component = similarity * self.homophily_weight;
        let proximity_component = proximity * self.proximity_weight;
        let social_proof = (mutual_friends as f32 / 10.0).min(0.3);

        let probability =
            self.formation_rate + similarity_component + proximity_component + social_proof;
        probability.min(1.0)
    }

    /// Calculate probability of tie dissolution
    #[must_use]
    pub fn dissolution_probability(&self, tie_strength: f32, time_since_last_contact: u32) -> f32 {
        let strength_factor = (1.0 - tie_strength) * 0.5;
        let time_factor = (time_since_last_contact as f32 / 100.0).min(0.5);

        (self.dissolution_rate + strength_factor + time_factor).min(1.0)
    }
}

impl Default for TieFormation {
    fn default() -> Self {
        Self::new()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Network evolution over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvolution {
    /// snapshots.
    pub snapshots: Vec<NetworkSnapshot>,
}

/// Snapshot of network at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    /// timestamp.
    pub timestamp: u64,
    /// node count.
    pub node_count: usize,
    /// edge count.
    pub edge_count: usize,
    /// avg clustering.
    pub avg_clustering: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tie_formation() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let formation = TieFormation::new();

        let prob = formation.formation_probability(0.8, 0.9, 3);
        assert!(prob > 0.0 && prob <= 1.0);
    }

    #[test]
    fn test_tie_dissolution() {
        let formation = TieFormation::new();

        // Strong tie with recent contact
        let prob_strong = formation.dissolution_probability(0.9, 5);

        // Weak tie with old contact
        let prob_weak = formation.dissolution_probability(0.2, 100);

        assert!(prob_weak > prob_strong);
    }

    #[test]
    fn test_calculate_tie_strength() {
        let dynamics = NetworkDynamics::new(SocialConfig::default());

        let strong = dynamics.calculate_tie_strength(50, 0.9, 1.0, 0.8);
        let weak = dynamics.calculate_tie_strength(5, 0.5, 0.1, 0.2);

        assert!(strong > weak);
    }
}
