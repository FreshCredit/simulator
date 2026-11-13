//! Reputation Systems (P2-RES-1)
//!
//! Trust and reputation in social networks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Reputation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    /// reputation decay.
    pub reputation_decay: f32,
    /// rating weight.
    pub rating_weight: f32,
    /// transitive trust.
    pub transitive_trust: bool,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            reputation_decay: 0.01,
            rating_weight: 0.5,
            transitive_trust: true,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Reputation score for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reputation {
    /// agent id.
    pub agent_id: u64,
    /// global score.
    pub global_score: f32, // Overall reputation (0-1)
    /// ratings received.
    pub ratings_received: u32,
    /// ratings given.
    pub ratings_given: u32,
    /// category scores.
    pub category_scores: HashMap<String, f32>,
}

/// Rating from one agent to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    /// from.
    pub from: u64,
    /// to.
    pub to: u64,
    /// score.
    pub score: f32, // -1 to 1
    /// category.
    pub category: String,
    /// timestamp.
    pub timestamp: u64,
}

/// Reputation system
#[derive(Debug, Clone, Default)]
pub struct ReputationSystem {
    /// config.
    pub config: ReputationConfig,
    /// reputations.
    pub reputations: HashMap<u64, Reputation>,
    /// ratings.
    pub ratings: Vec<Rating>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ReputationSystem {
    #[must_use]
    /// new.
    pub fn new(config: ReputationConfig) -> Self {
        Self {
            config,
            reputations: HashMap::new(),
            ratings: Vec::new(),
        }
    }

    /// Add rating
    pub fn add_rating(&mut self, rating: &Rating) {
        self.ratings.push(rating.clone());

        // Update recipient's reputation
        let rep = self
            .reputations
            .entry(rating.to)
            .or_insert_with(|| Reputation {
                agent_id: rating.to,
                global_score: 0.5,
                ratings_received: 0,
                ratings_given: 0,
                category_scores: HashMap::new(),
            });

        rep.ratings_received += 1;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Update category score
        let current = rep
            .category_scores
            .get(&rating.category)
            .copied()
            .unwrap_or(0.5);
        let new_score = current * (1.0 - self.config.rating_weight)
            + (rating.score + 1.0) / 2.0 * self.config.rating_weight;
        rep.category_scores
            .insert(rating.category.clone(), new_score);

        // Update global score
        rep.global_score =
            rep.category_scores.values().sum::<f32>() / rep.category_scores.len() as f32;

        // Update rater's count
        let rater = self
            .reputations
            .entry(rating.from)
            .or_insert_with(|| Reputation {
                agent_id: rating.from,
                global_score: 0.5,
                ratings_received: 0,
                ratings_given: 0,
                category_scores: HashMap::new(),
            });
        rater.ratings_given += 1;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get reputation
    #[must_use]
    pub fn get_reputation(&self, agent_id: u64) -> Option<&Reputation> {
        self.reputations.get(&agent_id)
    }

    /// Calculate transitive trust (A trusts B, B trusts C -> A somewhat trusts C)
    #[must_use]
    pub fn transitive_trust(&self, a: u64, c: u64, degree: usize) -> f32 {
        if degree == 0 || !self.config.transitive_trust {
            return 0.0;
        }

        // Find intermediate agents
        let intermediates: Vec<u64> = self
            .ratings
            .iter()
            .filter(|r| r.from == a && r.score > 0.0)
            .map(|r| r.to)
            .collect();

        if intermediates.is_empty() {
            return 0.0;
        }

        // Average trust through intermediates
        let trust_sum: f32 = intermediates
            .iter()
            .map(|&b| {
                let a_to_b = self.direct_trust(a, b);
                let b_to_c = self.direct_trust(b, c);
                a_to_b * b_to_c * 0.5 // Discount transitive trust
            })
            .sum();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        trust_sum / intermediates.len() as f32
    }

    /// Direct trust based on ratings
    fn direct_trust(&self, from: u64, to: u64) -> f32 {
        let ratings: Vec<&Rating> = self
            .ratings
            .iter()
            .filter(|r| r.from == from && r.to == to)
            .collect();

        if ratings.is_empty() {
            return 0.5; // Neutral if no interaction
        }

        let sum: f32 = ratings.iter().map(|r| (r.score + 1.0) / 2.0).sum();
        sum / ratings.len() as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Decay old reputations
    pub fn decay(&mut self) {
        for rep in self.reputations.values_mut() {
            rep.global_score = rep.global_score * (1.0 - self.config.reputation_decay)
                + 0.5 * self.config.reputation_decay;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_rating() {
        let mut system = ReputationSystem::default();

        system.add_rating(&Rating {
            from: 1,
            to: 2,
            score: 0.8,
            category: "trust".to_string(),
            timestamp: 0,
        });

        let rep = system.get_reputation(2).unwrap();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(rep.global_score > 0.5);
    }

    #[test]
    fn test_transitive_trust() {
        let mut system = ReputationSystem::default();

        // A rates B positively
        system.add_rating(&Rating {
            from: 1,
            to: 2,
            score: 0.9,
            category: "trust".to_string(),
            timestamp: 0,
        });
        // B rates C positively
        system.add_rating(&Rating {
            from: 2,
            to: 3,
            score: 0.8,
            category: "trust".to_string(),
            timestamp: 0,
        });

        let transitive = system.transitive_trust(1, 3, 2);
        assert!(transitive > 0.0);
    }
}
