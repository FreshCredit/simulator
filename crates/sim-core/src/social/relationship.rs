//! Relationship Management (P1-AGENT-5)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::Interaction;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Relationship manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipManager {
    /// relationships.
    pub relationships: HashMap<(u64, u64), Relationship>,
    /// agent relationships.
    pub agent_relationships: HashMap<u64, Vec<u64>>, // agent_id -> list of related agents
}

impl RelationshipManager {
    /// Create new manager
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add agent
    pub fn add_agent(&mut self, agent_id: u64) {
        self.agent_relationships.entry(agent_id).or_default();
    }

    /// Get or create relationship
    fn get_or_create_mut(&mut self, agent_a: u64, agent_b: u64) -> &mut Relationship {
        let key = normalize_key(agent_a, agent_b);

        // Track relationship for both agents (avoid duplicates)
        let related_a = self.agent_relationships.entry(agent_a).or_default();
        if !related_a.contains(&agent_b) {
            related_a.push(agent_b);
        }
        let related_b = self.agent_relationships.entry(agent_b).or_default();
        if !related_b.contains(&agent_a) {
            related_b.push(agent_a);
        }

        self.relationships
            .entry(key)
            .or_insert_with(|| Relationship::new(agent_a, agent_b))
    }

    /// Update relationship from interaction
    pub fn update_from_interaction(&mut self, interaction: &Interaction) {
        let rel = self.get_or_create_mut(interaction.agent_a, interaction.agent_b);
        rel.update_from_interaction(interaction);
    }

    /// Get relationship between two agents
    #[must_use]
    pub fn get_relationship(&self, agent_a: u64, agent_b: u64) -> Option<&Relationship> {
        self.relationships.get(&normalize_key(agent_a, agent_b))
    }

    /// Get mutable relationship between two agents
    pub fn get_mut(&mut self, agent_a: u64, agent_b: u64) -> Option<&mut Relationship> {
        self.relationships.get_mut(&normalize_key(agent_a, agent_b))
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get all relationships for an agent
    #[must_use]
    pub fn get_relationships(&self, agent_id: u64) -> Vec<&Relationship> {
        self.agent_relationships
            .get(&agent_id)
            .map(|related| {
                related
                    .iter()
                    .filter_map(|&other_id| self.get_relationship(agent_id, other_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if relationship exists
    #[must_use]
    pub fn has_relationship(&self, agent_a: u64, agent_b: u64) -> bool {
        self.relationships
            .contains_key(&normalize_key(agent_a, agent_b))
    }

    /// Decay all relationships
    pub fn decay_relationships(&mut self, rate: f32) {
        for rel in self.relationships.values_mut() {
            rel.decay(rate);
        }

        // Remove very weak relationships
        let to_remove: Vec<_> = self
            .relationships
            .iter()
            .filter(|(_, r)| r.strength < 0.1)
            .map(|(k, _)| *k)
            .collect();

        for key in to_remove {
            if let Some(rel) = self.relationships.remove(&key) {
                // Remove from agent tracking
                if let Some(related) = self.agent_relationships.get_mut(&rel.agent_a) {
                    related.retain(|&id| id != rel.agent_b);
                }
                if let Some(related) = self.agent_relationships.get_mut(&rel.agent_b) {
                    related.retain(|&id| id != rel.agent_a);
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get count of relationships
    #[must_use]
    pub fn relationship_count(&self) -> usize {
        self.relationships.len()
    }

    /// Get average relationship strength
    #[must_use]
    pub fn avg_strength(&self) -> f32 {
        if self.relationships.is_empty() {
            return 0.0;
        }
        self.relationships.values().map(|r| r.strength).sum::<f32>()
            / self.relationships.len() as f32
    }
}

/// Relationship between two agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// agent a.
    pub agent_a: u64,
    /// agent b.
    pub agent_b: u64,
    /// strength.
    pub strength: f32, // 0.0 to 1.0
    /// relationship type.
    pub relationship_type: RelationshipType,
    /// trust.
    pub trust: f32,
    /// reciprocity.
    pub reciprocity: f32,
    /// interaction count.
    pub interaction_count: u32,
    /// positive interactions.
    pub positive_interactions: u32,
    /// negative interactions.
    pub negative_interactions: u32,
    /// last interaction.
    pub last_interaction: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Type of relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// stranger.
    Stranger,
    /// acquaintance.
    Acquaintance,
    /// friend.
    Friend,
    /// close friend.
    CloseFriend,
    /// rival.
    Rival,
    /// enemy.
    Enemy,
    /// ally.
    Ally,
}

impl Relationship {
    /// Create new relationship
    #[must_use]
    pub const fn new(agent_a: u64, agent_b: u64) -> Self {
        Self {
            agent_a,
            agent_b,
            strength: 0.0,
            relationship_type: RelationshipType::Stranger,
            trust: 0.5,
            reciprocity: 0.0,
            interaction_count: 0,
            positive_interactions: 0,
            negative_interactions: 0,
            last_interaction: 0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update from interaction
    pub fn update_from_interaction(&mut self, interaction: &Interaction) {
        self.interaction_count += 1;
        self.last_interaction = interaction.timestamp;

        // Update strength based on outcome
        let delta = interaction.outcome * 0.1;
        self.strength = (self.strength + delta).clamp(0.0, 1.0);

        // Track positive/negative
        if interaction.outcome > 0.0 {
            self.positive_interactions += 1;
        } else if interaction.outcome < 0.0 {
            self.negative_interactions += 1;
        }

        // Update trust based on consistency
        let expected_outcome = self.trust * 2.0 - 1.0; // Map [0,1] to [-1,1]
        let consistency = 1.0 - (interaction.outcome - expected_outcome).abs();
        self.trust = (self.trust + consistency * 0.05).clamp(0.0, 1.0);

        // Update reciprocity
        self.reciprocity = self.positive_interactions as f32 / self.interaction_count.max(1) as f32;

        // Update relationship type
        self.update_type();
    }

    /// Update relationship type based on strength
    fn update_type(&mut self) {
        self.relationship_type = match self.strength {
            s if s < 0.2 => RelationshipType::Stranger,
            s if s < 0.4 => RelationshipType::Acquaintance,
            s if s < 0.7 => RelationshipType::Friend,
            s if s >= 0.7 => RelationshipType::CloseFriend,
            _ => RelationshipType::Stranger,
        };

        // Override if many negative interactions
        if self.negative_interactions > self.positive_interactions * 2 {
            self.relationship_type = RelationshipType::Rival;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Decay relationship
    pub fn decay(&mut self, rate: f32) {
        self.strength = (self.strength - rate).max(0.0);
        self.update_type();
    }

    /// Predict likelihood of cooperation
    #[must_use]
    pub fn predict_cooperation(&self) -> f32 {
        self.strength * self.trust * self.reciprocity
    }

    /// Get relationship quality
    #[must_use]
    pub fn quality(&self) -> RelationshipQuality {
        if self.strength > 0.6 && self.trust > 0.6 {
            RelationshipQuality::Positive
        } else if self.strength < 0.2 && self.negative_interactions > 3 {
            RelationshipQuality::Negative
        } else {
            RelationshipQuality::Neutral
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Quality assessment of relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipQuality {
    /// positive.
    Positive,
    /// neutral.
    Neutral,
    /// negative.
    Negative,
}

/// Normalize key order for consistent lookup
const fn normalize_key(a: u64, b: u64) -> (u64, u64) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::super::InteractionType;
    use super::*;

    #[test]
    fn test_relationship_formation() {
        let mut manager = RelationshipManager::new();
        manager.add_agent(1);
        manager.add_agent(2);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let interaction = Interaction {
            agent_a: 1,
            agent_b: 2,
            interaction_type: InteractionType::Cooperation,
            outcome: 0.8,
            timestamp: 0,
        };

        manager.update_from_interaction(&interaction);

        let rel = manager.get_relationship(1, 2);
        assert!(rel.is_some());
        assert!(rel.unwrap().strength > 0.0);
    }

    #[test]
    fn test_relationship_progression() {
        let mut rel = Relationship::new(1, 2);

        // Multiple positive interactions
        for i in 0..10 {
            rel.update_from_interaction(&Interaction {
                agent_a: 1,
                agent_b: 2,
                interaction_type: InteractionType::Cooperation,
                outcome: 0.8,
                timestamp: i as u64,
            });
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(matches!(
            rel.relationship_type,
            RelationshipType::Friend | RelationshipType::CloseFriend
        ));
        assert!(rel.predict_cooperation() > 0.5);
    }

    #[test]
    fn test_relationship_decay() {
        let mut rel = Relationship::new(1, 2);
        rel.strength = 0.8;

        rel.decay(0.1);
        assert!(rel.strength < 0.8);
    }

    #[test]
    fn test_negative_relationship() {
        let mut rel = Relationship::new(1, 2);

        // Multiple negative interactions
        for i in 0..5 {
            rel.update_from_interaction(&Interaction {
                agent_a: 1,
                agent_b: 2,
                interaction_type: InteractionType::Conflict,
                outcome: -0.8,
                timestamp: i as u64,
            });
        }

        assert_eq!(rel.relationship_type, RelationshipType::Rival);
        assert_eq!(rel.quality(), RelationshipQuality::Negative);
    }
}
