//! Memory Retrieval with Similarity Search

use std::collections::HashMap;

use super::types::{MemoryContent, MemoryEntry, ScoredMemory, SimilarityConfig};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Advanced memory retriever with similarity-based search
#[derive(Debug)]
pub struct MemoryRetriever {
    config: SimilarityConfig,
}

impl MemoryRetriever {
    /// Create a new retriever with default config
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SimilarityConfig::default(),
        }
    }

    /// Create with custom config
    #[must_use]
    pub const fn with_config(config: SimilarityConfig) -> Self {
        Self { config }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Retrieve memories similar to a cue memory
    #[must_use]
    pub fn retrieve_similar(
        &self,
        cue: &MemoryEntry,
        candidates: &[MemoryEntry],
        top_k: usize,
    ) -> Vec<ScoredMemory> {
        let mut scored: Vec<_> = candidates
            .iter()
            .filter(|m| m.id != cue.id) // Don't return the cue itself
            .map(|m| {
                let score = self.calculate_similarity(cue, m);
                ScoredMemory {
                    memory: m.clone(),
                    relevance_score: score,
                    match_reasons: vec!["similarity".to_string()],
                }
            })
            .collect();

        // Sort by score descending (with memory.id tie-breaker for SIM-004)
        scored.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.memory.id.cmp(&b.memory.id))
        });

        scored.into_iter().take(top_k).collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Retrieve memories by associative chain
    /// Starting from a seed memory, follow related memory links
    #[must_use]
    pub fn retrieve_associative(
        &self,
        seed: &MemoryEntry,
        all_memories: &HashMap<String, MemoryEntry>,
        max_depth: u32,
    ) -> Vec<ScoredMemory> {
        if max_depth == 0 {
            return Vec::new();
        }

        let mut results = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut current_level = vec![(seed.id.clone(), 1.0, 0)];

        visited.insert(seed.id.clone());

        while let Some((current_id, base_score, depth)) = current_level.pop() {
            if depth >= max_depth {
                continue;
            }

            let Some(current) = all_memories.get(&current_id) else {
                continue;
            };

            // Add to results if not the seed
            if current_id != seed.id {
                results.push(ScoredMemory {
                    memory: current.clone(),
                    relevance_score: base_score * (1.0 - depth as f64 / max_depth as f64),
                    match_reasons: vec![format!("association_depth_{}", depth)],
                });
            }

            // Follow related memories
            for related_id in &current.related_memories {
                if !visited.contains(related_id) {
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    visited.insert(related_id.clone());
                    let decay = 0.8f64.powi(depth as i32);
                    current_level.push((related_id.clone(), base_score * decay, depth + 1));
                }
            }

            // Also follow by temporal proximity
            let temporal_neighbors: Vec<_> = all_memories
                .values()
                .filter(|m| {
                    m.agent_id == current.agent_id
                        && m.id != current.id
                        && !visited.contains(&m.id)
                        && m.tick.abs_diff(current.tick) < 50
                })
                .map(|m| m.id.clone())
                .collect();

            for neighbor_id in temporal_neighbors {
                if !visited.contains(&neighbor_id) {
                    visited.insert(neighbor_id.clone());
                    let temporal_score = base_score * 0.5 * (1.0 - depth as f64 / max_depth as f64);
                    current_level.push((neighbor_id, temporal_score, depth + 1));
                }
            }
        }

        // Sort and deduplicate (with memory.id tie-breaker for SIM-004)
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.memory.id.cmp(&b.memory.id))
        });

        results
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Spreading activation retrieval
    /// Activate memories based on keyword activation spreading
    #[must_use]
    pub fn spreading_activation(
        &self,
        keywords: &[String],
        all_memories: &[MemoryEntry],
        activation_threshold: f64,
    ) -> Vec<ScoredMemory> {
        let mut activation: HashMap<String, f64> = HashMap::new();

        // Initial activation from keywords
        for memory in all_memories {
            let text = memory.content.searchable_text().to_lowercase();
            let initial_activation: f64 = keywords
                .iter()
                .map(|k| {
                    if text.contains(&k.to_lowercase()) {
                        1.0
                    } else {
                        0.0
                    }
                })
                .sum();

            if initial_activation > 0.0 {
                activation.insert(memory.id.clone(), initial_activation);
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Spread activation (simplified - one iteration, sorted for SIM-004 determinism)
        let mut sorted_activation_keys: Vec<String> = activation.keys().cloned().collect();
        sorted_activation_keys.sort();
        let spread_activation = activation.clone();
        for id in &sorted_activation_keys {
            if let Some(&act) = spread_activation.get(id) {
                if let Some(memory) = all_memories.iter().find(|m| &m.id == id) {
                    for related_id in &memory.related_memories {
                        let entry = activation.entry(related_id.clone()).or_insert(0.0);
                        *entry += act * 0.3; // Spread factor
                    }
                }
            }
        }

        // Collect results above threshold
        let mut results: Vec<_> = activation
            .into_iter()
            .filter_map(|(id, act)| {
                if act >= activation_threshold {
                    all_memories
                        .iter()
                        .find(|m| m.id == id)
                        .map(|m| ScoredMemory {
                            memory: m.clone(),
                            relevance_score: act.min(1.0),
                            match_reasons: vec!["spreading_activation".to_string()],
                        })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.memory.id.cmp(&b.memory.id))
        });

        results
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Context-aware retrieval
    /// Retrieve memories that match the current context
    #[must_use]
    pub fn retrieve_by_context(
        &self,
        context: &RetrievalContext,
        all_memories: &[MemoryEntry],
        top_k: usize,
    ) -> Vec<ScoredMemory> {
        let mut scored: Vec<_> = all_memories
            .iter()
            .map(|m| {
                let score = self.calculate_context_match(m, context);
                ScoredMemory {
                    memory: m.clone(),
                    relevance_score: score,
                    match_reasons: vec!["context_match".to_string()],
                }
            })
            .filter(|s| s.relevance_score > 0.1)
            .collect();

        scored.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.memory.id.cmp(&b.memory.id))
        });

        scored.into_iter().take(top_k).collect()
    }

    /// Calculate similarity between two memories
    fn calculate_similarity(&self, a: &MemoryEntry, b: &MemoryEntry) -> f64 {
        // Semantic similarity based on content
        let semantic_sim = self.semantic_similarity(&a.content, &b.content);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Temporal similarity (closer in time = more similar)
        let temporal_sim = self.temporal_similarity(a.tick, b.tick);

        // Emotional similarity: emotional_valence in [-1.0, 1.0], difference bounded in [0.0, 2.0]
        let emotional_diff = (a.emotional_valence - b.emotional_valence).abs();
        let emotional_sim = (1.0 - (emotional_diff / 2.0)).clamp(0.0, 1.0);

        // Tag overlap
        let tag_sim = self.tag_overlap(&a.tags, &b.tags);

        // Weighted combination
        (semantic_sim * self.config.semantic_weight
            + temporal_sim * self.config.temporal_weight
            + emotional_sim * self.config.emotional_weight
            + tag_sim * self.config.tag_weight)
            .clamp(0.0, 1.0)
    }

    /// Calculate semantic similarity between memory contents
    fn semantic_similarity(&self, a: &MemoryContent, b: &MemoryContent) -> f64 {
        let text_a = a.searchable_text().to_lowercase();
        let text_b = b.searchable_text().to_lowercase();

        // Simple word overlap similarity
        let words_a: std::collections::HashSet<_> = text_a.split_whitespace().collect();
        let words_b: std::collections::HashSet<_> = text_b.split_whitespace().collect();

        if words_a.is_empty() && words_b.is_empty() {
            return 1.0;
        }
        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }

        let intersection: std::collections::HashSet<_> =
            words_a.intersection(&words_b).copied().collect();
        let union: std::collections::HashSet<_> = words_a.union(&words_b).copied().collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        intersection.len() as f64 / union.len() as f64
    }

    /// Calculate temporal similarity (exponential decay with time)
    fn temporal_similarity(&self, tick_a: u64, tick_b: u64) -> f64 {
        let diff = tick_a.abs_diff(tick_b) as f64;
        (-diff / 100.0).exp() // Decay factor
    }

    /// Calculate tag overlap similarity
    fn tag_overlap(&self, a: &[String], b: &[String]) -> f64 {
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let set_a: std::collections::HashSet<_> = a.iter().cloned().collect();
        let set_b: std::collections::HashSet<_> = b.iter().cloned().collect();

        let intersection: std::collections::HashSet<_> =
            set_a.intersection(&set_b).cloned().collect();

        intersection.len() as f64 / set_a.union(&set_b).count().max(1) as f64
    }

    /// Calculate context match score
    fn calculate_context_match(&self, memory: &MemoryEntry, context: &RetrievalContext) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Location match
        if let Some(ref location) = context.location {
            if let MemoryContent::Episodic(ref ep) = memory.content {
                if ep.location.as_ref() == Some(location) {
                    score += 1.0;
                }
            }
            factors += 1;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Participant match
        if !context.participants.is_empty() {
            if let MemoryContent::Episodic(ref ep) = memory.content {
                let overlap: std::collections::HashSet<_> = ep
                    .participants
                    .iter()
                    .filter(|p| context.participants.contains(p))
                    .collect();
                score += overlap.len() as f64 / context.participants.len().max(1) as f64;
            }
            factors += 1;
        }

        // Time proximity
        if let Some(time) = context.current_tick {
            let diff = memory.tick.abs_diff(time) as f64;
            score += (-diff / 100.0).exp();
            factors += 1;
        }

        // Emotional context match
        score += 1.0 - (memory.emotional_valence - context.emotional_state).abs();
        factors += 1;

        if factors > 0 {
            score / factors as f64
        } else {
            0.0
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for MemoryRetriever {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for context-aware retrieval
#[derive(Debug, Clone, Default)]
pub struct RetrievalContext {
    /// Current location
    pub location: Option<String>,
    /// Current participants
    pub participants: Vec<String>,
    /// Current tick
    pub current_tick: Option<u64>,
    /// Current emotional state (-1.0 to 1.0)
    pub emotional_state: f64,
    /// Active goals
    pub active_goals: Vec<String>,
    /// Recent perceptions
    pub recent_perceptions: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl RetrievalContext {
    /// Create a new context
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set location
    #[must_use]
    pub fn at_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set participants
    #[must_use]
    pub fn with_participants(mut self, participants: Vec<String>) -> Self {
        self.participants = participants;
        self
    }

    /// Set current tick
    #[must_use]
    pub const fn at_tick(mut self, tick: u64) -> Self {
        self.current_tick = Some(tick);
        self
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Set emotional state
    #[must_use]
    pub fn feeling(mut self, valence: f64) -> Self {
        self.emotional_state = valence.clamp(-1.0, 1.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::types::{EpisodicMemory, MemoryType};

    fn create_test_episodic(id: &str, event: &str, tick: u64) -> MemoryEntry {
        let mut entry = MemoryEntry::new(
            "agent-1".to_string(),
            MemoryType::Episodic,
            MemoryContent::Episodic(EpisodicMemory {
                event: event.to_string(),
                location: Some("Town Square".to_string()),
                participants: vec!["Alice".to_string()],
                sensory_details: HashMap::new(),
                narrative: event.to_string(),
            }),
            tick,
        );
        entry.id = id.to_string();
        entry
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_similarity_calculation() {
        let retriever = MemoryRetriever::new();

        let mem_a = create_test_episodic("1", "Met Alice at the market", 100);
        let mem_b = create_test_episodic("2", "Met Alice at the market", 105);

        let sim = retriever.calculate_similarity(&mem_a, &mem_b);
        assert!(sim > 0.5); // Should be quite similar
    }

    #[test]
    fn test_temporal_similarity() {
        let retriever = MemoryRetriever::new();

        let sim_close = retriever.temporal_similarity(100, 105);
        let sim_far = retriever.temporal_similarity(100, 500);

        assert!(sim_close > sim_far);
        assert!(sim_close <= 1.0);
        assert!(sim_far >= 0.0);
    }

    #[test]
    fn test_spreading_activation() {
        let retriever = MemoryRetriever::new();

        let memories = vec![
            create_test_episodic("1", "Alice gave me an apple", 100),
            create_test_episodic("2", "Bob went to the store", 101),
            create_test_episodic("3", "I ate an apple with Alice", 102),
        ];

        let results = retriever.spreading_activation(&["apple".to_string()], &memories, 0.1);

        assert_eq!(results.len(), 2); // Memories 1 and 3 contain "apple"
    }

    #[test]
    fn test_emotional_valence_similarity_bounded() {
        let retriever = MemoryRetriever::new();

        let mut mem_pos = create_test_episodic("pos", "Event", 100);
        mem_pos.emotional_valence = 1.0;

        let mut mem_neg = create_test_episodic("neg", "Event", 100);
        mem_neg.emotional_valence = -1.0;

        let diff_opposing = (mem_pos.emotional_valence - mem_neg.emotional_valence).abs();
        let sim_opposing = (1.0 - (diff_opposing / 2.0)).clamp(0.0, 1.0);
        assert_eq!(sim_opposing, 0.0);

        let overall_sim = retriever.calculate_similarity(&mem_pos, &mem_neg);
        assert!(overall_sim >= 0.0 && overall_sim <= 1.0);

        let mut mem_pos2 = create_test_episodic("pos2", "Event", 100);
        mem_pos2.emotional_valence = 1.0;
        let diff_same = (mem_pos.emotional_valence - mem_pos2.emotional_valence).abs();
        let sim_same = (1.0 - (diff_same / 2.0)).clamp(0.0, 1.0);
        assert_eq!(sim_same, 1.0);
    }
}
