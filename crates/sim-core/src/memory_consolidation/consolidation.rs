//! Memory Consolidation Process (P1-AGENT-4)

use serde::{Deserialize, Serialize};

use super::{ConsolidatedMemory, LongTermStorage, WorkingMemoryItem};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for memory consolidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    /// Number of memories to consolidate per cycle
    pub batch_size: usize,
    /// Minimum importance threshold for consolidation
    pub importance_threshold: f32,
    /// Emotional boost factor
    pub emotional_boost: f32,
    /// Rehearsal requirement
    pub min_rehearsals: u32,
    /// Retention threshold for pruning
    pub retention_threshold: f32,
    /// Integration rate for pattern learning
    pub integration_rate: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            importance_threshold: 0.3,
            emotional_boost: 0.3,
            min_rehearsals: 2,
            retention_threshold: 0.1,
            integration_rate: 0.1,
        }
    }
}

/// Memory consolidator
#[derive(Debug, Clone)]
pub struct MemoryConsolidator {
    /// config.
    pub config: ConsolidationConfig,
    /// consolidation queue.
    pub consolidation_queue: Vec<WorkingMemoryItem>,
}

impl MemoryConsolidator {
    /// Create new consolidator
    #[must_use]
    pub const fn new(config: ConsolidationConfig) -> Self {
        Self {
            config,
            consolidation_queue: Vec::new(),
        }
    }

    /// Select memories for consolidation
    pub fn select_for_consolidation(
        &mut self,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        working_memories: &[WorkingMemoryItem],
    ) -> Vec<WorkingMemoryItem> {
        let mut candidates: Vec<_> = working_memories
            .iter()
            .filter(|m| self.meets_criteria(m))
            .cloned()
            .collect();

        // Sort by consolidation score
        candidates.sort_by(|a, b| {
            let score_a = self.consolidation_score(a);
            let score_b = self.consolidation_score(b);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top batch_size
        candidates.truncate(self.config.batch_size);
        candidates
    }

    /// Check if memory meets consolidation criteria
    fn meets_criteria(&self, memory: &WorkingMemoryItem) -> bool {
        memory.importance.is_finite()
            && memory.importance >= self.config.importance_threshold
            && memory.rehearsal_count >= self.config.min_rehearsals
    }

    /// Calculate consolidation score
    fn consolidation_score(&self, memory: &WorkingMemoryItem) -> f32 {
        if !memory.importance.is_finite() || !memory.emotional_valence.is_finite() {
            return 0.0;
        }
        let emotional_component = memory.emotional_valence.abs() * self.config.emotional_boost;
        let rehearsal_component = (memory.rehearsal_count as f32 * 0.1).min(0.5);

        (memory.importance + emotional_component + rehearsal_component).max(0.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Integrate patterns across memories
    pub fn integrate_patterns(&self, storage: &mut LongTermStorage) {
        // Find similar memories and strengthen connections
        let memories: Vec<_> = storage.all_memories().cloned().collect();

        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                let similarity = self.calculate_similarity(&memories[i], &memories[j]);
                if similarity > 0.7 {
                    storage.strengthen_association(memories[i].id, memories[j].id, similarity);
                }
            }
        }
    }

    /// Calculate similarity between memories
    fn calculate_similarity(&self, a: &ConsolidatedMemory, b: &ConsolidatedMemory) -> f32 {
        let temporal_sim = if a.timestamp.abs_diff(b.timestamp) < 100 {
            0.3
        } else {
            0.0
        };

        let context_sim = if a.context.location == b.context.location {
            0.4
        } else {
            0.0
        };

        let valence_diff = if a.emotional_valence.is_finite() && b.emotional_valence.is_finite() {
            (a.emotional_valence - b.emotional_valence).abs()
        } else {
            2.0
        };
        let emotional_sim = (1.0 - valence_diff * 0.5).clamp(0.0, 1.0);

        (temporal_sim + context_sim + emotional_sim * 0.3).clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create schema from repeated patterns
    #[must_use]
    pub fn extract_schema(&self, memories: &[ConsolidatedMemory]) -> Option<MemorySchema> {
        if memories.len() < 3 {
            return None;
        }

        // Simple schema extraction
        let avg_importance =
            memories.iter().map(|m| m.importance).sum::<f32>() / memories.len() as f32;
        let common_location = self.find_common_location(memories);

        Some(MemorySchema {
            pattern_type: "generalized".to_string(),
            importance: avg_importance,
            typical_location: common_location,
            instance_count: memories.len(),
        })
    }

    /// Find most common location
    fn find_common_location(&self, memories: &[ConsolidatedMemory]) -> Option<String> {
        let mut location_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for memory in memories {
            *location_counts
                .entry(memory.context.location.clone())
                .or_insert(0) += 1;
        }

        location_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(loc, _)| loc)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Extracted memory schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySchema {
    /// pattern type.
    pub pattern_type: String,
    /// importance.
    pub importance: f32,
    /// typical location.
    pub typical_location: Option<String>,
    /// instance count.
    pub instance_count: usize,
}

/// Consolidation process state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsolidationProcess {
    /// encoding.
    Encoding,
    /// stabilization.
    Stabilization,
    /// integration.
    Integration,
    /// retrieval.
    Retrieval,
}

#[cfg(test)]
mod tests {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    use super::super::{MemoryContent, MemoryContext};
    use super::*;

    fn create_test_memory(id: u64, importance: f32) -> WorkingMemoryItem {
        WorkingMemoryItem {
            id,
            content: MemoryContent::Fact {
                statement: "Test".to_string(),
                confidence: 0.9,
            },
            emotional_valence: 0.5,
            importance,
            rehearsal_count: 3,
            timestamp: id * 10,
            context: MemoryContext {
                location: "Test".to_string(),
                emotional_state: (0.0, 0.5),
                social_context: vec![],
            },
        }
    }

    #[test]
    fn test_select_for_consolidation() {
        let mut consolidator = MemoryConsolidator::new(ConsolidationConfig::default());

        let memories = vec![
            create_test_memory(1, 0.9),
            create_test_memory(2, 0.2), // Below threshold
            create_test_memory(3, 0.8),
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let selected = consolidator.select_for_consolidation(&memories);
        assert_eq!(selected.len(), 2); // Only high importance
    }

    #[test]
    fn test_consolidation_score() {
        let config = ConsolidationConfig::default();
        let consolidator = MemoryConsolidator::new(config);

        let memory = create_test_memory(1, 0.5);
        let score = consolidator.consolidation_score(&memory);
        assert!(score > 0.5);
    }

    #[test]
    fn test_schema_extraction() {
        let consolidator = MemoryConsolidator::new(ConsolidationConfig::default());

        let memories: Vec<ConsolidatedMemory> = (0..5)
            .map(|i| ConsolidatedMemory {
                id: i as u64,
                content: super::super::MemoryContent::Fact {
                    statement: format!("Fact {i}"),
                    confidence: 0.9,
                },
                emotional_valence: 0.5,
                importance: 0.7,
                timestamp: i as u64 * 10,
                context: MemoryContext {
                    location: "Common".to_string(),
                    emotional_state: (0.0, 0.5),
                    social_context: vec![],
                },
                access_count: 1,
                last_accessed: i as u64 * 10,
                strength: 0.8,
                associations: vec![],
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let schema = consolidator.extract_schema(&memories);
        assert!(schema.is_some());
        assert_eq!(schema.unwrap().typical_location, Some("Common".to_string()));
    }

    #[test]
    fn test_consolidation_config_default() {
        let config = ConsolidationConfig::default();
        assert_eq!(config.batch_size, 10);
        assert_eq!(config.importance_threshold, 0.3);
        assert_eq!(config.emotional_boost, 0.3);
        assert_eq!(config.min_rehearsals, 2);
        assert_eq!(config.retention_threshold, 0.1);
        assert_eq!(config.integration_rate, 0.1);
    }

    #[test]
    fn test_meets_criteria_false() {
        let consolidator = MemoryConsolidator::new(ConsolidationConfig::default());
        let memory = create_test_memory(1, 0.1); // Below threshold
        assert!(!consolidator.meets_criteria(&memory));
    }

    #[test]
    fn test_meets_criteria_true() {
        let consolidator = MemoryConsolidator::new(ConsolidationConfig::default());
        let memory = create_test_memory(1, 0.9); // Above threshold
        assert!(consolidator.meets_criteria(&memory));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_consolidation_score_higher_with_rehearsals() {
        let config = ConsolidationConfig::default();
        let consolidator = MemoryConsolidator::new(config);

        let mut low_rehearsal = create_test_memory(1, 0.5);
        low_rehearsal.rehearsal_count = 2;
        let score_low = consolidator.consolidation_score(&low_rehearsal);

        let mut high_rehearsal = create_test_memory(2, 0.5);
        high_rehearsal.rehearsal_count = 10;
        let score_high = consolidator.consolidation_score(&high_rehearsal);

        assert!(score_high > score_low);
    }

    #[test]
    fn test_extract_schema_insufficient() {
        let consolidator = MemoryConsolidator::new(ConsolidationConfig::default());
        let memories: Vec<ConsolidatedMemory> = (0..2)
            .map(|i| ConsolidatedMemory {
                id: i as u64,
                content: super::super::MemoryContent::Fact {
                    statement: format!("Fact {i}"),
                    confidence: 0.9,
                },
                emotional_valence: 0.5,
                importance: 0.7,
                timestamp: i as u64 * 10,
                context: MemoryContext {
                    location: "Common".to_string(),
                    emotional_state: (0.0, 0.5),
                    social_context: vec![],
                },
                access_count: 1,
                last_accessed: i as u64 * 10,
                strength: 0.8,
                associations: vec![],
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let schema = consolidator.extract_schema(&memories);
        assert!(schema.is_none());
    }

    #[test]
    fn test_consolidation_process_enum() {
        let processes = [
            ConsolidationProcess::Encoding,
            ConsolidationProcess::Stabilization,
            ConsolidationProcess::Integration,
            ConsolidationProcess::Retrieval,
        ];
        assert_eq!(processes.len(), 4);
    }

    #[test]
    fn test_memory_consolidator_new() {
        let config = ConsolidationConfig::default();
        let consolidator = MemoryConsolidator::new(config);
        assert!(consolidator.consolidation_queue.is_empty());
    }

    #[test]
    fn test_similarity_calculation_extreme_timestamp_and_valence_bounds() {
        let consolidator = MemoryConsolidator::new(ConsolidationConfig::default());

        let m1 = ConsolidatedMemory {
            id: 1,
            content: super::super::MemoryContent::Fact {
                statement: "A".to_string(),
                confidence: 0.9,
            },
            emotional_valence: 1.0,
            importance: 0.9,
            timestamp: 0,
            context: MemoryContext {
                location: "Loc".to_string(),
                emotional_state: (1.0, 0.5),
                social_context: vec![],
            },
            access_count: 1,
            last_accessed: 0,
            strength: 0.9,
            associations: vec![],
        };

        let mut m2 = m1.clone();
        m2.id = 2;
        // Large timestamp difference must not overflow
        m2.timestamp = u64::MAX;
        m2.emotional_valence = -1.0;

        let sim = consolidator.calculate_similarity(&m1, &m2);
        assert!(
            sim >= 0.0 && sim <= 1.0,
            "Similarity {sim} must be bounded in [0.0, 1.0]"
        );
        assert!(sim.is_finite());
    }
}
