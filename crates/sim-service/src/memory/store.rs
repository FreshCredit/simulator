//! Memory Store - In-memory storage for agent memories

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, trace, warn};

use super::types::{
    EpisodicMemory, ForgettingPolicy, MemoryContent, MemoryEntry, MemoryImportance, MemoryQuery,
    MemorySource, MemoryStats, MemoryType, ScoredMemory, SemanticMemory, StoreMemoryRequest,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Storage for all agent memories
#[derive(Clone, Debug)]
pub struct MemoryStore {
    /// All memories indexed by memory ID
    memories: Arc<RwLock<HashMap<String, MemoryEntry>>>,
    /// Index: `agent_id` -> Vec<`memory_ids`>
    agent_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Index: tag -> Vec<`memory_ids`>
    tag_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Forgetting policy
    policy: ForgettingPolicy,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Per-agent memory view
#[derive(Clone, Debug)]
pub struct AgentMemory {
    /// agent id
    pub agent_id: String,
    store: MemoryStore,
}

impl MemoryStore {
    /// Create a new memory store with default policy
    #[must_use]
    pub fn new() -> Self {
        Self::with_policy(ForgettingPolicy::default())
    }

    /// Create with custom forgetting policy
    #[must_use]
    pub fn with_policy(policy: ForgettingPolicy) -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
            agent_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            policy,
        }
    }

    /// Store a new memory
    pub async fn store(&self, request: StoreMemoryRequest) -> String {
        let mut entry = MemoryEntry::new(
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            request.agent_id.clone(),
            request.memory_type,
            request.content,
            request.tick,
        );

        // Apply optional fields
        if let Some(importance) = request.importance {
            entry.importance = importance;
        }
        if let Some(valence) = request.emotional_valence {
            entry.emotional_valence = valence.clamp(-1.0, 1.0);
        }
        if let Some(tags) = request.tags {
            entry.tags = tags;
        }
        if let Some(source) = request.source {
            entry.source = source;
        }

        let memory_id = entry.id.clone();
        let agent_id = entry.agent_id.clone();

        // Store the memory
        {
            let mut memories = self.memories.write().await;
            memories.insert(memory_id.clone(), entry);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Update indexes
        {
            let mut agent_idx = self.agent_index.write().await;
            agent_idx
                .entry(agent_id.clone())
                .or_default()
                .push(memory_id.clone());
        }

        // Update tag index
        {
            let memories = self.memories.read().await;
            if let Some(memory) = memories.get(&memory_id) {
                let mut tag_idx = self.tag_index.write().await;
                for tag in &memory.tags {
                    tag_idx
                        .entry(tag.clone())
                        .or_default()
                        .push(memory_id.clone());
                }
            }
        }

        trace!(
            memory_id = %memory_id,
            agent_id = %agent_id,
            "Memory stored"
        );

        memory_id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a memory by ID
    pub async fn get(&self, memory_id: &str) -> Option<MemoryEntry> {
        let memories = self.memories.read().await;
        memories.get(memory_id).cloned()
    }

    /// Get a memory and mark it as accessed
    pub async fn get_and_access(&self, memory_id: &str) -> Option<MemoryEntry> {
        let mut memories = self.memories.write().await;
        if let Some(memory) = memories.get_mut(memory_id) {
            memory.mark_accessed();
            Some(memory.clone())
        } else {
            None
        }
    }
    /// Link two memories as related
    pub async fn link_memories(&self, memory_id: &str, related_id: &str) {
        let mut memories = self.memories.write().await;
        if let Some(entry) = memories.get_mut(memory_id) {
            if !entry.related_memories.iter().any(|id| id == related_id) {
                entry.related_memories.push(related_id.to_string());
            }
        }
        if let Some(related_entry) = memories.get_mut(related_id) {
            if !related_entry
                .related_memories
                .iter()
                .any(|id| id == memory_id)
            {
                related_entry.related_memories.push(memory_id.to_string());
            }
        }
    }

    /// Query memories for an agent
    pub async fn query(&self, agent_id: &str, query: MemoryQuery) -> Vec<ScoredMemory> {
        let agent_idx = self.agent_index.read().await;
        let memory_ids = match agent_idx.get(agent_id) {
            Some(ids) => ids.clone(),
            None => return Vec::new(),
        };
        drop(agent_idx);

        let memories = self.memories.read().await;
        let mut results: Vec<ScoredMemory> = memory_ids
            .iter()
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .filter_map(|id| memories.get(id))
            .filter(|m| self.matches_query(m, &query))
            .map(|m| ScoredMemory {
                memory: m.clone(),
                relevance_score: self.calculate_relevance(m, &query),
                match_reasons: self.get_match_reasons(m, &query),
            })
            .collect();
        drop(memories);

        // Sort
        if query.sort_by_recency {
            results.sort_by(|a, b| b.memory.tick.cmp(&a.memory.tick));
        } else {
            // Sort by relevance (importance * access_count factor)
            results.sort_by(|a, b| {
                let score_a = a.relevance_score * (a.memory.importance as i32 as f64);
                let score_b = b.relevance_score * (b.memory.importance as i32 as f64);
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        // Apply pagination
        results
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get memory statistics for an agent
    pub async fn get_stats(&self, agent_id: &str) -> Option<MemoryStats> {
        let agent_idx = self.agent_index.read().await;
        let memory_ids = agent_idx.get(agent_id)?.clone();
        drop(agent_idx);

        let memories = self.memories.read().await;

        let mut stats = MemoryStats {
            agent_id: agent_id.to_string(),
            total_memories: 0,
            episodic_count: 0,
            semantic_count: 0,
            procedural_count: 0,
            working_count: 0,
            memory_by_importance: HashMap::new(),
            oldest_memory: None,
            newest_memory: None,
            total_storage_bytes: 0,
        };

        let mut oldest_tick = u64::MAX;
        let mut newest_tick = 0u64;

        for id in &memory_ids {
            if let Some(memory) = memories.get(id) {
                stats.total_memories += 1;

                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                match memory.memory_type {
                    MemoryType::Episodic => stats.episodic_count += 1,
                    MemoryType::Semantic => stats.semantic_count += 1,
                    MemoryType::Procedural => stats.procedural_count += 1,
                    MemoryType::Working => stats.working_count += 1,
                    MemoryType::Sensory => {}
                }

                let importance_key = format!("{:?}", memory.importance).to_lowercase();
                *stats
                    .memory_by_importance
                    .entry(importance_key)
                    .or_insert(0) += 1;

                if memory.tick < oldest_tick {
                    oldest_tick = memory.tick;
                    stats.oldest_memory = Some(memory.timestamp.clone());
                }

                if memory.tick > newest_tick {
                    newest_tick = memory.tick;
                    stats.newest_memory = Some(memory.timestamp.clone());
                }

                // Rough estimation of storage size
                stats.total_storage_bytes +=
                    std::mem::size_of::<MemoryEntry>() + memory.content.searchable_text().len();
            }
        }

        Some(stats)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get all memories for an agent
    pub async fn get_agent_memories(&self, agent_id: &str) -> Vec<MemoryEntry> {
        let agent_idx = self.agent_index.read().await;
        let memory_ids = match agent_idx.get(agent_id) {
            Some(ids) => ids.clone(),
            None => return Vec::new(),
        };
        drop(agent_idx);

        let memories = self.memories.read().await;
        memory_ids
            .iter()
            .filter_map(|id| memories.get(id))
            .cloned()
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Delete a specific memory
    pub async fn delete(&self, memory_id: &str) -> bool {
        let mut memories = self.memories.write().await;

        let memory = match memories.get(memory_id) {
            Some(m) => m.clone(),
            None => return false,
        };

        memories.remove(memory_id);
        drop(memories);

        // Update indexes
        let mut agent_idx = self.agent_index.write().await;
        if let Some(ids) = agent_idx.get_mut(&memory.agent_id) {
            ids.retain(|id| id != memory_id);
        }
        drop(agent_idx);

        let mut tag_idx = self.tag_index.write().await;
        for tag in &memory.tags {
            if let Some(ids) = tag_idx.get_mut(tag) {
                ids.retain(|id| id != memory_id);
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            }
        }

        trace!(memory_id = %memory_id, "Memory deleted");
        true
    }

    /// Delete all memories for an agent
    pub async fn delete_agent_memories(&self, agent_id: &str) -> usize {
        let agent_idx = self.agent_index.read().await;
        let memory_ids = match agent_idx.get(agent_id) {
            Some(ids) => ids.clone(),
            None => return 0,
        };
        drop(agent_idx);

        let count = memory_ids.len();

        let del_futs = memory_ids.iter().map(|id| self.delete(id));
        futures::future::join_all(del_futs).await;

        // Remove agent from index
        let mut agent_idx = self.agent_index.write().await;
        agent_idx.remove(agent_id);

        info!(agent_id = %agent_id, count = %count, "All agent memories deleted");
        count
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply forgetting - remove old/low-importance memories
    pub async fn apply_forgetting(&self, current_tick: u64) -> usize {
        let memories = self.memories.read().await;
        let to_forget: Vec<String> = memories
            .values()
            .filter(|m| {
                let retention = m.retention_probability(current_tick, &self.policy);
                // Use random factor for stochastic forgetting
                retention < 0.1 // Very low retention -> forget
            })
            .map(|m| m.id.clone())
            .collect();
        drop(memories);

        let count = to_forget.len();
        let del_futs = to_forget.iter().map(|id| self.delete(id));
        futures::future::join_all(del_futs).await;

        if count > 0 {
            debug!(count = %count, "Applied forgetting");
        }

        count
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Enforce capacity limits per agent
    pub async fn enforce_capacity(&self) -> usize {
        let agent_idx = self.agent_index.read().await;
        let agents: Vec<String> = agent_idx.keys().cloned().collect();
        drop(agent_idx);

        let enforce_futs = agents.into_iter().map(|agent_id| async move {
            let memories_guard = self.memories.read().await;
            let agent_idx_guard = self.agent_index.read().await;

            let Some(memory_ids) = agent_idx_guard.get(&agent_id) else {
                return 0;
            };

            if memory_ids.len() <= self.policy.max_memories_per_agent {
                return 0;
            }

            // Get memories sorted by importance (ascending) and age (oldest first)
            let mut agent_memories: Vec<_> = memory_ids
                .iter()
                .filter_map(|id| memories_guard.get(id))
                .map(|m| (m.id.clone(), m.importance, m.tick))
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                .collect();
            drop(memories_guard);
            drop(agent_idx_guard);

            // Sort by importance (ascending) then by tick (ascending - oldest first)
            agent_memories.sort_by(|a, b| {
                a.1.cmp(&b.1) // importance
                    .then_with(|| a.2.cmp(&b.2)) // tick (age)
            });

            let excess = agent_memories.len() - self.policy.max_memories_per_agent;
            let to_remove: Vec<_> = agent_memories
                .into_iter()
                .take(excess)
                .map(|(id, _, _)| id)
                .collect();

            let del_futs = to_remove.iter().map(|id| self.delete(id));
            let results = futures::future::join_all(del_futs).await;
            let removed = results.into_iter().filter(|&ok| ok).count();

            if excess > 0 {
                warn!(
                    agent_id = %agent_id,
                    removed = %excess,
                    "Enforced memory capacity limit"
                );
            }
            removed
        });

        let results = futures::future::join_all(enforce_futs).await;
        results.into_iter().sum()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get agent memory view
    pub fn for_agent(&self, agent_id: impl Into<String>) -> AgentMemory {
        AgentMemory {
            agent_id: agent_id.into(),
            store: self.clone(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create episodic memory quickly
    pub async fn create_episodic(
        &self,
        agent_id: impl Into<String>,
        event: impl Into<String>,
        tick: u64,
        importance: MemoryImportance,
    ) -> String {
        let request = StoreMemoryRequest {
            agent_id: agent_id.into(),
            memory_type: MemoryType::Episodic,
            content: MemoryContent::Episodic(EpisodicMemory {
                event: event.into(),
                location: None,
                participants: Vec::new(),
                sensory_details: HashMap::new(),
                narrative: String::new(),
            }),
            tick,
            importance: Some(importance),
            emotional_valence: None,
            tags: None,
            source: Some(MemorySource::Perception),
        };

        self.store(request).await
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Create semantic memory quickly
    pub async fn create_semantic(
        &self,
        agent_id: impl Into<String>,
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
        tick: u64,
        confidence: f64,
    ) -> String {
        let request = StoreMemoryRequest {
            agent_id: agent_id.into(),
            memory_type: MemoryType::Semantic,
            content: MemoryContent::Semantic(SemanticMemory {
                subject: subject.into(),
                predicate: predicate.into(),
                object: object.into(),
                confidence: confidence.clamp(0.0, 1.0),
                knowledge_source: "learned".to_string(),
            }),
            tick,
            importance: Some(MemoryImportance::Normal),
            emotional_valence: None,
            tags: None,
            source: Some(MemorySource::Internal),
        };

        self.store(request).await
    }

    /// Check if a memory matches query criteria
    fn matches_query(&self, memory: &MemoryEntry, query: &MemoryQuery) -> bool {
        Self::matches_type(memory, query)
            && Self::matches_importance(memory, query)
            && Self::matches_tags(memory, query)
            && Self::matches_time_range(memory, query)
            && Self::matches_content(memory, query)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn matches_type(memory: &MemoryEntry, query: &MemoryQuery) -> bool {
        match query.memory_type {
            Some(ref t) => memory.memory_type == *t,
            None => true,
        }
    }

    fn matches_importance(memory: &MemoryEntry, query: &MemoryQuery) -> bool {
        match query.min_importance {
            Some(ref min) => memory.importance >= *min,
            None => true,
        }
    }

    fn matches_tags(memory: &MemoryEntry, query: &MemoryQuery) -> bool {
        match query.tags {
            Some(ref tags) => tags.iter().all(|t| memory.tags.contains(t)),
            None => true,
        }
    }

    const fn matches_time_range(memory: &MemoryEntry, query: &MemoryQuery) -> bool {
        match query.from_tick {
            Some(from) if memory.tick < from => return false,
            _ => {}
        }
        match query.to_tick {
            Some(to) => memory.tick <= to,
            None => true,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn matches_content(memory: &MemoryEntry, query: &MemoryQuery) -> bool {
        match query.content_search {
            Some(ref search) => {
                let text = memory.content.searchable_text().to_lowercase();
                text.contains(&search.to_lowercase())
            }
            None => true,
        }
    }

    /// Calculate relevance score for a memory
    const fn calculate_relevance(&self, _memory: &MemoryEntry, _query: &MemoryQuery) -> f64 {
        // Simple implementation - could be enhanced with vector similarity
        1.0
    }

    /// Get reasons why memory matched
    fn get_match_reasons(&self, memory: &MemoryEntry, query: &MemoryQuery) -> Vec<String> {
        let mut reasons = Vec::new();

        if query.memory_type.is_some() {
            reasons.push(format!("type:{:?}", memory.memory_type));
        }
        if query.tags.is_some() {
            reasons.push("tag_match".to_string());
        }
        if query.content_search.is_some() {
            reasons.push("content_match".to_string());
        }

        reasons
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentMemory {
    /// Store a memory for this agent
    pub async fn store(
        &self,
        memory_type: MemoryType,
        content: MemoryContent,
        tick: u64,
    ) -> String {
        let request = StoreMemoryRequest {
            agent_id: self.agent_id.clone(),
            memory_type,
            content,
            tick,
            importance: None,
            emotional_valence: None,
            tags: None,
            source: None,
        };
        self.store.store(request).await
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Query this agent's memories
    pub async fn query(&self, query: MemoryQuery) -> Vec<ScoredMemory> {
        self.store.query(&self.agent_id, query).await
    }

    /// Get memory statistics
    pub async fn stats(&self) -> Option<MemoryStats> {
        self.store.get_stats(&self.agent_id).await
    }

    /// Get all memories
    pub async fn all(&self) -> Vec<MemoryEntry> {
        self.store.get_agent_memories(&self.agent_id).await
    }

    /// Clear all memories
    pub async fn clear(&self) -> usize {
        self.store.delete_agent_memories(&self.agent_id).await
    }
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let store = MemoryStore::new();

        let request = StoreMemoryRequest {
            agent_id: "agent-1".to_string(),
            memory_type: MemoryType::Episodic,
            content: MemoryContent::Episodic(EpisodicMemory {
                event: "Met Alice".to_string(),
                location: Some("Town Square".to_string()),
                participants: vec!["Alice".to_string()],
                sensory_details: HashMap::new(),
                narrative: "I met Alice in the town square.".to_string(),
            }),
            tick: 100,
            importance: Some(MemoryImportance::High),
            emotional_valence: Some(0.5),
            tags: Some(vec!["social".to_string(), "meeting".to_string()]),
            source: Some(MemorySource::Perception),
        };

        let id = store.store(request).await;

        let retrieved = store.get(&id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().agent_id, "agent-1");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_query_by_type() {
        let store = MemoryStore::new();

        // Store episodic memory
        store
            .create_episodic("agent-2", "Went to market", 100, MemoryImportance::Normal)
            .await;

        // Store semantic memory
        store
            .create_semantic("agent-2", "Apple", "is_a", "Fruit", 101, 0.9)
            .await;

        let query = MemoryQuery {
            memory_type: Some(MemoryType::Semantic),
            ..Default::default()
        };

        let results = store.query("agent-2", query).await;
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_agent_memory_view() {
        let store = MemoryStore::new();
        let agent_memory = store.for_agent("agent-3");

        agent_memory
            .store(
                MemoryType::Episodic,
                MemoryContent::Episodic(EpisodicMemory {
                    event: "Test event".to_string(),
                    location: None,
                    participants: vec![],
                    sensory_details: HashMap::new(),
                    narrative: "Test narrative".to_string(),
                }),
                200,
            )
            .await;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let stats = agent_memory.stats().await;
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().total_memories, 1);
    }

    #[tokio::test]
    async fn test_forgetting() {
        let policy = ForgettingPolicy {
            base_decay_rate: 0.5, // Fast decay for testing
            min_retention_ticks: 10,
            ..Default::default()
        };
        let store = MemoryStore::with_policy(policy);

        // Create a trivial memory
        let id = store
            .create_episodic("agent-4", "Minor event", 0, MemoryImportance::Trivial)
            .await;

        // Apply forgetting at tick 100
        let forgotten = store.apply_forgetting(100).await;
        assert!(forgotten > 0);

        // Memory should be gone
        assert!(store.get(&id).await.is_none());
    }
}
