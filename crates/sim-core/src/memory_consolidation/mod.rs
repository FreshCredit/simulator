//! Memory Consolidation Module (P1-AGENT-4)
//!
//! Long-term memory formation through consolidation.

pub mod consolidation;
pub mod replay;
pub mod storage;

pub use consolidation::{ConsolidationConfig, ConsolidationProcess, MemoryConsolidator};
pub use replay::{ReplayConfig, ReplayEvent, ReplayScheduler};
pub use storage::{ConsolidatedMemory, LongTermStorage, MemoryIndex};

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Memory consolidation system
#[derive(Debug, Clone)]
pub struct MemoryConsolidationSystem {
    /// config.
    pub config: ConsolidationConfig,
    /// consolidator.
    pub consolidator: MemoryConsolidator,
    /// replay.
    pub replay: ReplayScheduler,
    /// storage.
    pub storage: LongTermStorage,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MemoryConsolidationSystem {
    /// Create new consolidation system
    #[must_use]
    pub fn new(config: ConsolidationConfig) -> Self {
        Self {
            config: config.clone(),
            consolidator: MemoryConsolidator::new(config),
            replay: ReplayScheduler::new(ReplayConfig::default()),
            storage: LongTermStorage::new(),
        }
    }

    /// Process working memory for consolidation
    pub fn consolidate(
        &mut self,
        working_memories: &[WorkingMemoryItem],
    ) -> Vec<ConsolidatedMemory> {
        // 1. Select memories for consolidation
        let selected = self.consolidator.select_for_consolidation(working_memories);

        // 2. Replay selected memories
        let replayed = self.replay.schedule_replay(&selected);

        // 3. Integrate into long-term storage
        let consolidated: Vec<_> = replayed
            .into_iter()
            .map(|item| self.storage.store(item))
            .collect();

        consolidated
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update during sleep/rest periods
    pub fn update_during_rest(&mut self) {
        // 1. Replay recent important memories
        self.replay.run_scheduled_replay(&mut self.storage);

        // 2. Integrate replayed patterns
        self.consolidator.integrate_patterns(&mut self.storage);

        // 3. Prune low-value memories
        self.storage.prune(self.config.retention_threshold);
    }

    /// Retrieve consolidated memory
    #[must_use]
    pub fn retrieve(&self, query: &MemoryQuery) -> Vec<&ConsolidatedMemory> {
        self.storage.retrieve(query)
    }
}

impl Default for MemoryConsolidationSystem {
    fn default() -> Self {
        Self::new(ConsolidationConfig::default())
    }
}

/// Working memory item (pre-consolidation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryItem {
    /// id.
    pub id: u64,
    /// content.
    pub content: MemoryContent,
    /// emotional valence.
    pub emotional_valence: f32,
    /// importance.
    pub importance: f32,
    /// rehearsal count.
    pub rehearsal_count: u32,
    /// timestamp.
    pub timestamp: u64,
    /// context.
    pub context: MemoryContext,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Content of a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryContent {
    /// event.
    Event {
        /// description.
        description: String,
        /// location.
        location: String,
        /// participants.
        participants: Vec<u64>,
    },
    /// fact.
    Fact {
        /// statement.
        statement: String,
        /// confidence.
        confidence: f32,
    },
    /// skill.
    Skill {
        /// action.
        action: String,
        /// success rate.
        success_rate: f32,
    },
    /// relationship.
    Relationship {
        /// agent id.
        agent_id: u64,
        /// trait observations.
        trait_observations: Vec<String>,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Context for memory encoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    /// location.
    pub location: String,
    /// emotional state.
    pub emotional_state: (f32, f32), // valence, arousal
    /// social context.
    pub social_context: Vec<u64>, // Present agents
}

/// Query for memory retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// content type.
    pub content_type: Option<String>,
    /// time range.
    pub time_range: Option<(u64, u64)>,
    /// location.
    pub location: Option<String>,
    /// agent filter.
    pub agent_filter: Option<Vec<u64>>,
    /// min importance.
    pub min_importance: f32,
    /// emotional tone.
    pub emotional_tone: Option<(f32, f32)>, // target valence, tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_consolidation_system() {
        let system = MemoryConsolidationSystem::default();
        assert!(system.storage.memory_count() == 0);
    }

    #[test]
    fn test_working_memory_item() {
        let item = WorkingMemoryItem {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            id: 1,
            content: MemoryContent::Fact {
                statement: "Test".to_string(),
                confidence: 0.9,
            },
            emotional_valence: 0.5,
            importance: 0.7,
            rehearsal_count: 3,
            timestamp: 0,
            context: MemoryContext {
                location: "Test location".to_string(),
                emotional_state: (0.0, 0.5),
                social_context: vec![],
            },
        };
        assert_eq!(item.importance, 0.7);
    }

    #[test]
    fn test_memory_content_variants() {
        let event = MemoryContent::Event {
            description: "Test event".to_string(),
            location: "Here".to_string(),
            participants: vec![1, 2],
        };
        assert!(matches!(event, MemoryContent::Event { .. }));

        let skill = MemoryContent::Skill {
            action: "Jump".to_string(),
            success_rate: 0.8,
        };
        assert!(matches!(skill, MemoryContent::Skill { .. }));

        let relationship = MemoryContent::Relationship {
            agent_id: 5,
            trait_observations: vec!["kind".to_string()],
        };
        assert!(matches!(relationship, MemoryContent::Relationship { .. }));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_memory_query_default() {
        let query = MemoryQuery {
            content_type: None,
            time_range: None,
            location: None,
            agent_filter: None,
            min_importance: 0.5,
            emotional_tone: None,
        };
        assert_eq!(query.min_importance, 0.5);
    }

    #[test]
    fn test_consolidation_system_default() {
        let system = MemoryConsolidationSystem::default();
        assert_eq!(system.storage.memory_count(), 0);
        assert_eq!(system.config.batch_size, 10);
    }

    #[test]
    fn test_consolidate_empty() {
        let mut system = MemoryConsolidationSystem::default();
        let result = system.consolidate(&[]);
        assert!(result.is_empty());
    }
}
