//! Memory Consolidation - Converts episodic memories to semantic knowledge

use std::collections::HashMap;
use tracing::{debug, info, trace};

use super::types::{ConsolidationPlan, MemoryContent, MemoryEntry, MemoryType, SemanticMemory};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Memory consolidator that converts episodic memories to semantic knowledge
#[derive(Debug)]
pub struct MemoryConsolidator {
    /// Minimum number of similar episodes before consolidation
    min_episodes: usize,
    /// Similarity threshold for grouping episodes
    similarity_threshold: f64,
    /// How old (in ticks) memories must be before consolidation
    min_age_ticks: u64,
}

impl MemoryConsolidator {
    /// Create a new consolidator with default settings
    #[must_use]
    pub const fn new() -> Self {
        Self {
            min_episodes: 3,
            similarity_threshold: 0.7,
            min_age_ticks: 1000,
        }
    }

    /// Create with custom settings
    #[must_use]
    pub const fn with_settings(
        min_episodes: usize,
        similarity_threshold: f64,
        min_age_ticks: u64,
    ) -> Self {
        Self {
            min_episodes,
            similarity_threshold,
            min_age_ticks,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Analyze memories and create a consolidation plan
    #[must_use]
    pub fn plan_consolidation(
        &self,
        memories: &[MemoryEntry],
        current_tick: u64,
    ) -> Vec<ConsolidationPlan> {
        let mut plans = Vec::new();

        // Group episodic memories by agent
        let mut by_agent: HashMap<String, Vec<&MemoryEntry>> = HashMap::new();
        for mem in memories {
            if mem.memory_type == MemoryType::Episodic && !mem.is_consolidated {
                by_agent.entry(mem.agent_id.clone()).or_default().push(mem);
            }
        }

        // For each agent, look for patterns (sorted by agent_id for SIM-004 determinism)
        let mut sorted_agents: Vec<String> = by_agent.keys().cloned().collect();
        sorted_agents.sort();
        for agent_id in sorted_agents {
            if let Some(agent_memories) = by_agent.get(&agent_id) {
                let agent_plans =
                    self.analyze_agent_memories(&agent_id, agent_memories, current_tick);
                plans.extend(agent_plans);
            }
        }

        plans
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Execute a consolidation plan
    pub fn execute_plan(
        &self,
        plan: &ConsolidationPlan,
        memories: &mut HashMap<String, MemoryEntry>,
    ) -> Option<MemoryEntry> {
        // Mark memories as consolidated
        for mem_id in &plan.memories_to_consolidate {
            if let Some(mem) = memories.get_mut(mem_id) {
                mem.is_consolidated = true;
            }
        }

        // Create new semantic memory if specified
        if let Some(ref semantic) = plan.new_semantic_memory {
            let agent_id = plan
                .memories_to_consolidate
                .first()
                .and_then(|id| memories.get(id))
                .map(|m| m.agent_id.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let max_tick = plan
                .memories_to_consolidate
                .iter()
                .filter_map(|id| memories.get(id))
                .map(|m| m.tick)
                .max()
                .unwrap_or(0);

            let new_memory = MemoryEntry::new(
                agent_id,
                MemoryType::Semantic,
                MemoryContent::Semantic(semantic.clone()),
                max_tick,
            );

            debug!(
                "Created semantic memory: {} {} {}",
                semantic.subject, semantic.predicate, semantic.object
            );

            return Some(new_memory);
        }

        None
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Find duplicate/redundant memories that can be merged
    #[must_use]
    pub fn find_redundancies(&self, memories: &[MemoryEntry]) -> Vec<Vec<String>> {
        let mut groups: Vec<Vec<String>> = Vec::new();
        let mut processed = std::collections::HashSet::new();

        for mem in memories {
            if processed.contains(&mem.id) {
                continue;
            }

            let mut group = vec![mem.id.clone()];
            processed.insert(mem.id.clone());

            // Find similar memories
            for other in memories {
                if processed.contains(&other.id) || mem.id == other.id {
                    continue;
                }

                if self.are_redundant(mem, other) {
                    group.push(other.id.clone());
                    processed.insert(other.id.clone());
                }
            }

            if group.len() > 1 {
                groups.push(group);
            }
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        groups
    }

    /// Summarize a group of episodic memories into a semantic fact
    #[must_use]
    pub fn summarize_episodes(&self, episodes: &[&MemoryEntry]) -> Option<SemanticMemory> {
        if episodes.is_empty() {
            return None;
        }

        let mut location_counts: HashMap<String, usize> = HashMap::new();
        for ep in episodes.iter().filter_map(|e| Self::episodic(e)) {
            if let Some(ref loc) = ep.location {
                *location_counts.entry(loc.clone()).or_insert(0) += 1;
            }
        }
        let most_common_location = location_counts
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
            .map(|(loc, _)| loc);

        let mut participant_counts: HashMap<String, usize> = HashMap::new();
        for ep in episodes.iter().filter_map(|e| Self::episodic(e)) {
            for p in &ep.participants {
                *participant_counts.entry(p.clone()).or_insert(0) += 1;
            }
        }
        let most_common_participant = participant_counts
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
            .map(|(p, _)| p);

        let _event_types: Vec<_> = episodes
            .iter()
            .filter_map(|e| Self::episodic(e).map(|ep| ep.event.clone()))
            .collect();

        most_common_participant.map(|participant| SemanticMemory {
            subject: participant,
            predicate: "often_met_at".to_string(),
            object: most_common_location.unwrap_or_else(|| "various_places".to_string()),
            confidence: (episodes.len() as f64 * 0.1).min(1.0),
            knowledge_source: "consolidated".to_string(),
        })
    }

    const fn episodic(entry: &MemoryEntry) -> Option<&super::types::EpisodicMemory> {
        match &entry.content {
            MemoryContent::Episodic(ep) => Some(ep),
            _ => None,
        }
    }

    /// Analyze agent memories for consolidation opportunities
    fn analyze_agent_memories(
        &self,
        agent_id: &str,
        memories: &[&MemoryEntry],
        current_tick: u64,
    ) -> Vec<ConsolidationPlan> {
        let mut plans = Vec::new();

        // Group similar episodes
        let groups = self.group_similar_episodes(memories);

        for group in groups {
            if group.len() >= self.min_episodes {
                // Check if memories are old enough
                let old_enough = group
                    .iter()
                    .all(|m| current_tick.saturating_sub(m.tick) >= self.min_age_ticks);

                // TAG: surface=simulation owner=research-team rule=SIM-001
                if old_enough {
                    let episode_refs: Vec<_> = group.clone();

                    if let Some(semantic) = self.summarize_episodes(&episode_refs) {
                        let plan = ConsolidationPlan {
                            memories_to_consolidate: group.iter().map(|m| m.id.clone()).collect(),
                            new_semantic_memory: Some(semantic),
                            memories_to_forget: Vec::new(),
                            reason: format!(
                                "Consolidated {} similar episodic memories into semantic knowledge",
                                group.len()
                            ),
                        };
                        plans.push(plan);
                    }
                }
            }
        }

        trace!(
            agent_id = %agent_id,
            plan_count = %plans.len(),
            "Consolidation analysis complete"
        );

        plans
    }

    /// Group similar episodic memories together
    fn group_similar_episodes<'a>(
        &self,
        memories: &[&'a MemoryEntry],
    ) -> Vec<Vec<&'a MemoryEntry>> {
        let mut groups: Vec<Vec<&MemoryEntry>> = Vec::new();
        let mut assigned = std::collections::HashSet::new();

        for mem in memories {
            if assigned.contains(&mem.id) {
                continue;
            }

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let mut group = vec![*mem];
            assigned.insert(mem.id.clone());

            for other in memories {
                if assigned.contains(&other.id) || mem.id == other.id {
                    continue;
                }

                if self.episode_similarity(mem, other) >= self.similarity_threshold {
                    group.push(*other);
                    assigned.insert(other.id.clone());
                }
            }

            if group.len() >= 2 {
                groups.push(group);
            }
        }

        groups
    }

    /// Calculate similarity between two episodic memories
    fn episode_similarity(&self, a: &MemoryEntry, b: &MemoryEntry) -> f64 {
        match (&a.content, &b.content) {
            (MemoryContent::Episodic(ep_a), MemoryContent::Episodic(ep_b)) => {
                let mut score = 0.0;
                let mut factors = 0;

                // Event similarity
                let event_sim = Self::text_similarity(&ep_a.event, &ep_b.event);
                score += event_sim;
                factors += 1;

                // Location match
                if ep_a.location == ep_b.location {
                    score += 1.0;
                }
                factors += 1;

                // TAG: surface=simulation owner=research-team rule=SIM-001
                // Participant overlap
                let a_participants: std::collections::HashSet<_> =
                    ep_a.participants.iter().cloned().collect();
                let b_participants: std::collections::HashSet<_> =
                    ep_b.participants.iter().cloned().collect();
                let overlap: std::collections::HashSet<_> = a_participants
                    .intersection(&b_participants)
                    .cloned()
                    .collect();
                if !a_participants.is_empty() && !b_participants.is_empty() {
                    score += overlap.len() as f64
                        / a_participants.union(&b_participants).count().max(1) as f64;
                }
                factors += 1;

                // Temporal proximity
                let time_diff = a.tick.abs_diff(b.tick) as f64;
                let time_sim = (-time_diff / 1000.0).exp();
                score += time_sim;
                factors += 1;

                score / factors as f64
            }
            _ => 0.0,
        }
    }

    /// Check if two memories are redundant (nearly identical)
    fn are_redundant(&self, a: &MemoryEntry, b: &MemoryEntry) -> bool {
        if a.memory_type != b.memory_type {
            return false;
        }

        match (&a.content, &b.content) {
            (MemoryContent::Semantic(sa), MemoryContent::Semantic(sb)) => {
                sa.subject == sb.subject && sa.predicate == sb.predicate && sa.object == sb.object
            }
            (MemoryContent::Episodic(ea), MemoryContent::Episodic(eb)) => {
                Self::text_similarity(&ea.event, &eb.event) > 0.9 && ea.location == eb.location
            }
            _ => false,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Simple text similarity (word overlap)
    fn text_similarity(a: &str, b: &str) -> f64 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        let words_a: std::collections::HashSet<_> = a_lower.split_whitespace().collect();
        let words_b: std::collections::HashSet<_> = b_lower.split_whitespace().collect();

        if words_a.is_empty() && words_b.is_empty() {
            return 1.0;
        }
        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }

        let intersection: std::collections::HashSet<_> =
            words_a.intersection(&words_b).copied().collect();
        let union: std::collections::HashSet<_> = words_a.union(&words_b).copied().collect();

        intersection.len() as f64 / union.len() as f64
    }

    /// Find most common word in a list of strings
    #[allow(dead_code)]
    fn most_common_word(&self, texts: &[String]) -> Option<String> {
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for text in texts {
            for word in text.to_lowercase().split_whitespace() {
                *word_counts.entry(word.to_string()).or_insert(0) += 1;
            }
        }

        word_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(word, _)| word)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for MemoryConsolidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory maintenance scheduler
#[derive(Debug)]
pub struct MemoryMaintenance {
    consolidator: MemoryConsolidator,
    last_consolidation_tick: u64,
    consolidation_interval: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MemoryMaintenance {
    /// Create new maintenance scheduler
    #[must_use]
    pub const fn new() -> Self {
        Self {
            consolidator: MemoryConsolidator::new(),
            last_consolidation_tick: 0,
            consolidation_interval: 5000, // Every 5000 ticks
        }
    }

    /// Check if consolidation should run
    #[must_use]
    pub const fn should_consolidate(&self, current_tick: u64) -> bool {
        current_tick.saturating_sub(self.last_consolidation_tick) >= self.consolidation_interval
    }

    /// Run consolidation and return number of memories processed
    pub fn run_consolidation(
        &mut self,
        memories: &[MemoryEntry],
        current_tick: u64,
    ) -> (usize, Vec<ConsolidationPlan>) {
        let plans = self.consolidator.plan_consolidation(memories, current_tick);
        let processed = plans.iter().map(|p| p.memories_to_consolidate.len()).sum();

        self.last_consolidation_tick = current_tick;

        info!(
            processed_memories = %processed,
            plans_created = %plans.len(),
            "Memory consolidation completed"
        );

        (processed, plans)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get redundancy groups for cleanup
    #[must_use]
    pub fn find_redundancies(&self, memories: &[MemoryEntry]) -> Vec<Vec<String>> {
        self.consolidator.find_redundancies(memories)
    }
}

impl Default for MemoryMaintenance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::memory::types::{EpisodicMemory, MemoryType};

    fn create_episodic(id: &str, event: &str, tick: u64) -> MemoryEntry {
        let mut entry = MemoryEntry::new(
            "agent-1".to_string(),
            MemoryType::Episodic,
            MemoryContent::Episodic(EpisodicMemory {
                event: event.to_string(),
                location: Some("Market".to_string()),
                participants: vec!["Bob".to_string()],
                sensory_details: HashMap::new(),
                narrative: event.to_string(),
            }),
            tick,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        );
        entry.id = id.to_string();
        entry
    }

    #[test]
    fn test_consolidation_planning() {
        let consolidator = MemoryConsolidator::with_settings(3, 0.5, 100);

        let memories = vec![
            create_episodic("1", "Met Bob at the market", 100),
            create_episodic("2", "Met Bob at the market", 150),
            create_episodic("3", "Met Bob at the market", 200),
            create_episodic("4", "Went fishing alone", 300),
        ];

        let plans = consolidator.plan_consolidation(&memories, 2000);

        // Should create at least one consolidation plan for the similar episodes
        assert!(!plans.is_empty());
    }

    #[test]
    fn test_redundancy_detection() {
        let consolidator = MemoryConsolidator::new();

        let memories = vec![
            create_episodic("1", "Same event", 100),
            create_episodic("2", "Same event", 101),
            create_episodic("3", "Different event", 200),
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let redundancies = consolidator.find_redundancies(&memories);

        assert_eq!(redundancies.len(), 1);
        assert_eq!(redundancies[0].len(), 2);
    }

    #[test]
    fn test_episode_summarization() {
        let consolidator = MemoryConsolidator::new();

        let episodes = [
            create_episodic("1", "Met Bob at the market", 100),
            create_episodic("2", "Met Bob at the market", 200),
            create_episodic("3", "Met Bob at the market", 300),
        ];

        let refs: Vec<_> = episodes.iter().collect();
        let semantic = consolidator.summarize_episodes(&refs);

        assert!(semantic.is_some());
        let s = semantic.unwrap();
        assert_eq!(s.subject, "Bob");
    }

    #[test]
    fn test_text_similarity() {
        let sim =
            MemoryConsolidator::text_similarity("Met Bob at the market", "Met Bob at the market");
        assert_eq!(sim, 1.0);

        let sim2 =
            MemoryConsolidator::text_similarity("Met Bob at the market", "Met Alice at the store");
        assert!(sim2 > 0.0 && sim2 < 1.0);
    }
}
