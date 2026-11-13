//! Long-Term Memory Storage (P1-AGENT-4)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{MemoryContent, MemoryContext, MemoryQuery};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Long-term memory storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermStorage {
    memories: HashMap<u64, ConsolidatedMemory>,
    associations: HashMap<(u64, u64), f32>, // (memory_a, memory_b) -> strength
    index: MemoryIndex,
    next_id: u64,
}

impl LongTermStorage {
    /// Create new storage
    #[must_use]
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            associations: HashMap::new(),
            index: MemoryIndex::new(),
            next_id: 1,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Store a consolidated memory
    pub fn store(&mut self, item: super::WorkingMemoryItem) -> ConsolidatedMemory {
        let memory = ConsolidatedMemory {
            id: self.next_id,
            content: item.content,
            emotional_valence: item.emotional_valence,
            importance: item.importance,
            timestamp: item.timestamp,
            context: item.context,
            access_count: 0,
            last_accessed: item.timestamp,
            strength: 0.5 + item.importance * 0.5,
            associations: vec![],
        };

        self.index.add(&memory);
        self.memories.insert(self.next_id, memory.clone());
        self.next_id += 1;

        memory
    }

    /// Get memory by ID
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&ConsolidatedMemory> {
        self.memories.get(&id)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get mutable memory by ID
    pub fn get_mut(&mut self, id: u64) -> Option<&mut ConsolidatedMemory> {
        self.memories.get_mut(&id)
    }

    /// Retrieve memories matching query
    #[must_use]
    pub fn retrieve(&self, query: &MemoryQuery) -> Vec<&ConsolidatedMemory> {
        self.memories
            .values()
            .filter(|m| self.matches_query(m, query))
            .collect()
    }

    /// Check if memory matches query
    fn matches_query(&self, memory: &ConsolidatedMemory, query: &MemoryQuery) -> bool {
        // Importance filter
        if memory.importance < query.min_importance {
            return false;
        }

        // Location filter
        if let Some(ref loc) = query.location {
            if &memory.context.location != loc {
                return false;
            }
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Time range filter
        if let Some((start, end)) = query.time_range {
            if memory.timestamp < start || memory.timestamp > end {
                return false;
            }
        }

        // Emotional tone filter
        if let Some((target, tolerance)) = query.emotional_tone {
            if (memory.emotional_valence - target).abs() > tolerance {
                return false;
            }
        }

        // Agent filter
        if let Some(ref agents) = query.agent_filter {
            let has_agent = memory
                .context
                .social_context
                .iter()
                .any(|a| agents.contains(a));
            if !has_agent {
                return false;
            }
        }

        true
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Strengthen association between memories
    pub fn strengthen_association(&mut self, a: u64, b: u64, amount: f32) {
        if a == b || !amount.is_finite() || amount <= 0.0 {
            return;
        }
        let key = if a < b { (a, b) } else { (b, a) };
        let current = self.associations.get(&key).copied().unwrap_or(0.0);
        self.associations
            .insert(key, (current + amount).clamp(0.0, 1.0));
    }

    /// Get associated memories
    #[must_use]
    pub fn get_associated(&self, id: u64, min_strength: f32) -> Vec<(u64, f32)> {
        self.associations
            .iter()
            .filter(|((a, b), strength)| (*a == id || *b == id) && **strength >= min_strength)
            .map(|((a, b), strength)| {
                if *a == id {
                    (*b, *strength)
                } else {
                    (*a, *strength)
                }
            })
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Prune weak memories
    pub fn prune(&mut self, threshold: f32) -> usize {
        let to_remove: Vec<u64> = self
            .memories
            .iter()
            .filter(|(_, m)| m.strength < threshold && m.importance < 0.3)
            .map(|(id, _)| *id)
            .collect();

        let count = to_remove.len();
        for id in to_remove {
            self.memories.remove(&id);
            self.index.remove(id);
            // Remove associations
            self.associations.retain(|(a, b), _| *a != id && *b != id);
        }

        count
    }

    /// Get all memories
    pub fn all_memories(&self) -> impl Iterator<Item = &ConsolidatedMemory> {
        self.memories.values()
    }

    /// Count memories
    #[must_use]
    pub fn memory_count(&self) -> usize {
        self.memories.len()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get storage statistics
    #[must_use]
    pub fn statistics(&self) -> StorageStatistics {
        let total = self.memories.len() as f32;
        StorageStatistics {
            memory_count: self.memories.len(),
            association_count: self.associations.len(),
            avg_strength: if total > 0.0 {
                self.memories.values().map(|m| m.strength).sum::<f32>() / total
            } else {
                0.0
            },
            avg_importance: if total > 0.0 {
                self.memories.values().map(|m| m.importance).sum::<f32>() / total
            } else {
                0.0
            },
        }
    }
}

impl Default for LongTermStorage {
    fn default() -> Self {
        Self::new()
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Consolidated memory in long-term storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedMemory {
    /// id.
    pub id: u64,
    /// content.
    pub content: MemoryContent,
    /// emotional valence.
    pub emotional_valence: f32,
    /// importance.
    pub importance: f32,
    /// timestamp.
    pub timestamp: u64,
    /// context.
    pub context: MemoryContext,
    /// access count.
    pub access_count: u32,
    /// last accessed.
    pub last_accessed: u64,
    /// strength.
    pub strength: f32, // 0.0 to 1.0
    /// associations.
    pub associations: Vec<u64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ConsolidatedMemory {
    /// Strengthen the memory
    pub fn strengthen(&mut self, amount: f32) {
        if amount.is_finite() && amount > 0.0 {
            self.strength = (self.strength + amount).clamp(0.0, 1.0);
        }
    }

    /// Decay memory strength
    pub fn decay(&mut self, rate: f32) {
        if rate.is_finite() && rate > 0.0 {
            self.strength = (self.strength - rate).clamp(0.0, 1.0);
        }
    }
}

/// Memory index for efficient retrieval
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct MemoryIndex {
    by_location: HashMap<String, Vec<u64>>,
    by_time: Vec<(u64, u64)>,    // (timestamp, memory_id)
    by_emotion: Vec<(f32, u64)>, // (valence, memory_id)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MemoryIndex {
    #[must_use]
    /// new.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add memory to index
    pub fn add(&mut self, memory: &ConsolidatedMemory) {
        self.by_location
            .entry(memory.context.location.clone())
            .or_default()
            .push(memory.id);

        self.by_time.push((memory.timestamp, memory.id));
        self.by_emotion.push((memory.emotional_valence, memory.id));
    }

    /// Remove memory from index
    pub fn remove(&mut self, id: u64) {
        for list in self.by_location.values_mut() {
            list.retain(|&m_id| m_id != id);
        }
        self.by_time.retain(|&(_, m_id)| m_id != id);
        self.by_emotion.retain(|&(_, m_id)| m_id != id);
    }

    /// Get memories by location
    #[must_use]
    pub fn by_location(&self, location: &str) -> &[u64] {
        self.by_location
            .get(location)
            .map_or(&[], std::vec::Vec::as_slice)
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Storage statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StorageStatistics {
    /// memory count.
    pub memory_count: usize,
    /// association count.
    pub association_count: usize,
    /// avg strength.
    pub avg_strength: f32,
    /// avg importance.
    pub avg_importance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memory(id: u64) -> ConsolidatedMemory {
        ConsolidatedMemory {
            id,
            content: MemoryContent::Fact {
                statement: "Test".to_string(),
                confidence: 0.9,
            },
            emotional_valence: 0.5,
            importance: 0.7,
            timestamp: 0,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            context: MemoryContext {
                location: "Test".to_string(),
                emotional_state: (0.0, 0.5),
                social_context: vec![],
            },
            access_count: 0,
            last_accessed: 0,
            strength: 0.5,
            associations: vec![],
        }
    }

    #[test]
    fn test_storage() {
        let mut storage = LongTermStorage::new();
        assert_eq!(storage.memory_count(), 0);

        let item = super::super::WorkingMemoryItem {
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
                location: "Test".to_string(),
                emotional_state: (0.0, 0.5),
                social_context: vec![],
            },
        };

        let memory = storage.store(item);
        assert_eq!(storage.memory_count(), 1);
        assert_eq!(memory.id, 1);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_associations() {
        let mut storage = LongTermStorage::new();
        storage.strengthen_association(1, 2, 0.5);
        storage.strengthen_association(1, 2, 0.3);

        let associated = storage.get_associated(1, 0.0);
        assert_eq!(associated.len(), 1);
        assert!(associated[0].1 >= 0.7); // Strengthened twice
    }

    #[test]
    fn test_prune() {
        let mut storage = LongTermStorage::new();

        // Create memory directly and index it
        let m = create_test_memory(1);
        storage.index.add(&m);
        storage.memories.insert(1, m);
        storage.memories.get_mut(&1).unwrap().strength = 0.05; // Very weak
        storage.memories.get_mut(&1).unwrap().importance = 0.2;

        assert_eq!(storage.index.by_location("Test").len(), 1);

        let pruned = storage.prune(0.1);
        assert_eq!(pruned, 1);
        assert_eq!(storage.memory_count(), 0);
        assert_eq!(
            storage.index.by_location("Test").len(),
            0,
            "Pruned memory must be removed from index"
        );
    }

    #[test]
    fn test_memory_strengthen() {
        let mut memory = create_test_memory(1);
        let initial = memory.strength;
        memory.strengthen(0.2);
        assert!(memory.strength > initial);
    }

    #[test]
    fn test_memory_decay() {
        let mut memory = create_test_memory(1);
        memory.strength = 0.5;
        memory.decay(0.2);
        assert!(memory.strength < 0.5);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_storage_get_none() {
        let storage = LongTermStorage::new();
        assert!(storage.get(999).is_none());
    }

    #[test]
    fn test_storage_retrieve_empty() {
        let storage = LongTermStorage::new();
        let query = MemoryQuery {
            content_type: None,
            time_range: None,
            location: None,
            agent_filter: None,
            min_importance: 0.0,
            emotional_tone: None,
        };
        let results = storage.retrieve(&query);
        assert!(results.is_empty());
    }

    #[test]
    fn test_storage_statistics_empty() {
        let storage = LongTermStorage::new();
        let stats = storage.statistics();
        assert_eq!(stats.memory_count, 0);
        assert_eq!(stats.association_count, 0);
        assert_eq!(stats.avg_strength, 0.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_storage_prune_none() {
        let mut storage = LongTermStorage::new();
        let pruned = storage.prune(0.0);
        assert_eq!(pruned, 0);
    }

    #[test]
    fn test_memory_index_by_location_empty() {
        let index = MemoryIndex::new();
        let result = index.by_location("nowhere");
        assert!(result.is_empty());
    }

    #[test]
    fn test_storage_all_memories_empty() {
        let storage = LongTermStorage::new();
        let count = storage.all_memories().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_consolidated_memory_strengthen_clamp() {
        let mut memory = create_test_memory(1);
        memory.strength = 0.95;
        memory.strengthen(0.1);
        assert_eq!(memory.strength, 1.0);
    }

    #[test]
    fn test_memory_decay_clamp() {
        let mut memory = create_test_memory(1);
        memory.strength = 0.1;
        memory.decay(0.2);
        assert_eq!(memory.strength, 0.0);
    }
}
