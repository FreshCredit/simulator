//! Memory Types and Data Structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of memory storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    /// Short-term sensory memory
    Sensory,
    /// Short-term working memory
    Working,
    /// Long-term episodic (experiences)
    Episodic,
    /// Long-term semantic (facts/knowledge)
    Semantic,
    /// Procedural (skills/habits)
    Procedural,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Memory importance level for retention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum MemoryImportance {
    /// Trivial
    Trivial = 1,
    /// Low
    Low = 2,
    #[default]
    /// Normal
    Normal = 3,
    /// High
    High = 4,
    /// Critical
    Critical = 5,
}

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique memory ID
    pub id: String,
    /// Agent ID this memory belongs to
    pub agent_id: String,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Memory type
    pub memory_type: MemoryType,
    /// Memory content
    pub content: MemoryContent,
    /// When the memory was formed
    pub timestamp: String,
    /// Simulation tick when formed
    pub tick: u64,
    /// Memory importance for retention
    pub importance: MemoryImportance,
    /// Emotional valence (-1.0 to 1.0)
    pub emotional_valence: f64,
    /// How many times this memory was accessed
    pub access_count: u32,
    /// Last access timestamp
    pub last_accessed: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Related memory IDs
    pub related_memories: Vec<String>,
    /// Whether this memory has been consolidated
    pub is_consolidated: bool,
    /// Source of the memory
    pub source: MemorySource,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Content of a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MemoryContent {
    /// Episodic memory - an experience
    Episodic(EpisodicMemory),
    /// Semantic memory - a fact
    Semantic(SemanticMemory),
    /// Procedural memory - a skill
    Procedural(ProceduralMemory),
    /// Working memory - temporary information
    Working(WorkingMemoryContent),
}

/// Episodic memory - specific experiences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    /// What happened
    pub event: String,
    /// Where it happened
    pub location: Option<String>,
    /// Who else was involved
    pub participants: Vec<String>,
    /// Sensory details
    pub sensory_details: HashMap<String, String>,
    /// Narrative of the experience
    pub narrative: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Semantic memory - facts and knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    /// Subject of the fact
    pub subject: String,
    /// Predicate/relation
    pub predicate: String,
    /// Object/value
    pub object: String,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Source of the knowledge
    pub knowledge_source: String,
}

/// Procedural memory - skills and habits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralMemory {
    /// Name of the skill/action
    pub skill_name: String,
    /// Steps involved
    pub steps: Vec<String>,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Contexts where applicable
    pub contexts: Vec<String>,
}

/// Working memory content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryContent {
    /// Current focus/attention
    pub focus: String,
    /// Recent perceptions
    pub recent_perceptions: Vec<String>,
    /// Current goals
    pub active_goals: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Source of a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemorySource {
    /// Direct perception
    Perception,
    /// Agent action result
    Action,
    /// Communication from another agent
    Communication,
    /// Internal reasoning
    Internal,
    /// Consolidated from working memory
    Consolidation,
    /// Imported/external
    External,
}

/// Query parameters for memory retrieval
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    /// Filter by memory type
    pub memory_type: Option<MemoryType>,
    /// Filter by importance minimum
    pub min_importance: Option<MemoryImportance>,
    /// Filter by tags (all must match)
    pub tags: Option<Vec<String>>,
    /// Time range start
    pub from_tick: Option<u64>,
    /// Time range end
    pub to_tick: Option<u64>,
    /// Text search in content
    pub content_search: Option<String>,
    /// Maximum number of results
    pub limit: usize,
    /// Offset for pagination
    pub offset: usize,
    /// Sort by recency (true) or importance (false)
    pub sort_by_recency: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            memory_type: None,
            min_importance: None,
            tags: None,
            from_tick: None,
            to_tick: None,
            content_search: None,
            limit: 50, // Default limit
            offset: 0,
            sort_by_recency: true,
        }
    }
}

/// Result of a memory retrieval operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRetrievalResult {
    /// Retrieved memories with relevance scores
    pub memories: Vec<ScoredMemory>,
    /// Total matching memories
    pub total: usize,
    /// Query execution time in ms
    pub query_time_ms: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// A memory with a relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemory {
    /// The memory entry
    #[serde(flatten)]
    pub memory: MemoryEntry,
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: f64,
    /// Why this memory was retrieved
    pub match_reasons: Vec<String>,
}

/// Memory statistics for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// agent id
    pub agent_id: String,
    /// total memories
    pub total_memories: usize,
    /// episodic count
    pub episodic_count: usize,
    /// semantic count
    pub semantic_count: usize,
    /// procedural count
    pub procedural_count: usize,
    /// working count
    pub working_count: usize,
    /// memory by importance
    pub memory_by_importance: HashMap<String, usize>,
    /// oldest memory
    pub oldest_memory: Option<String>,
    /// newest memory
    pub newest_memory: Option<String>,
    /// total storage bytes
    pub total_storage_bytes: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Request to store a new memory
#[derive(Debug, Deserialize)]
pub struct StoreMemoryRequest {
    /// agent id
    pub agent_id: String,
    /// memory type
    pub memory_type: MemoryType,
    /// content
    pub content: MemoryContent,
    /// tick
    pub tick: u64,
    /// importance
    pub importance: Option<MemoryImportance>,
    /// emotional valence
    pub emotional_valence: Option<f64>,
    /// tags
    pub tags: Option<Vec<String>>,
    /// source
    pub source: Option<MemorySource>,
}

/// Request to query memories
#[derive(Debug, Deserialize)]
pub struct QueryMemoryRequest {
    /// agent id
    pub agent_id: String,
    /// memory type
    pub memory_type: Option<MemoryType>,
    /// tags
    pub tags: Option<Vec<String>>,
    /// from tick
    pub from_tick: Option<u64>,
    /// to tick
    pub to_tick: Option<u64>,
    /// content search
    pub content_search: Option<String>,
    /// limit
    pub limit: Option<usize>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Response for memory storage
#[derive(Debug, Serialize)]
pub struct StoreMemoryResponse {
    /// memory id
    pub memory_id: String,
    /// agent id
    pub agent_id: String,
    /// stored
    pub stored: bool,
}

/// Response for memory query
#[derive(Debug, Serialize)]
pub struct QueryMemoryResponse {
    /// agent id
    pub agent_id: String,
    /// memories
    pub memories: Vec<ScoredMemory>,
    /// total
    pub total: usize,
}

/// Memory consolidation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationPlan {
    /// Memories to consolidate
    pub memories_to_consolidate: Vec<String>,
    /// New semantic memory to create
    pub new_semantic_memory: Option<SemanticMemory>,
    /// Memories that can be forgotten
    pub memories_to_forget: Vec<String>,
    /// Reason for consolidation
    pub reason: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Forgetting policy configuration
#[derive(Debug, Clone)]
pub struct ForgettingPolicy {
    /// Base decay rate per tick (0.0 to 1.0)
    pub base_decay_rate: f64,
    /// Importance retention factors
    pub importance_factors: HashMap<MemoryImportance, f64>,
    /// Emotional boost to retention
    pub emotional_boost_factor: f64,
    /// Access count boost
    pub access_boost_factor: f64,
    /// Minimum retention ticks
    pub min_retention_ticks: u64,
    /// Maximum memory capacity per agent
    pub max_memories_per_agent: usize,
}

impl Default for ForgettingPolicy {
    fn default() -> Self {
        let mut importance_factors = HashMap::new();
        importance_factors.insert(MemoryImportance::Trivial, 0.1);
        importance_factors.insert(MemoryImportance::Low, 0.3);
        importance_factors.insert(MemoryImportance::Normal, 0.5);
        importance_factors.insert(MemoryImportance::High, 0.8);
        importance_factors.insert(MemoryImportance::Critical, 1.0);

        Self {
            base_decay_rate: 0.001,
            importance_factors,
            emotional_boost_factor: 0.2,
            access_boost_factor: 0.1,
            min_retention_ticks: 100,
            max_memories_per_agent: 10000,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Memory similarity metrics
#[derive(Debug, Clone)]
pub struct SimilarityConfig {
    /// Weight for semantic similarity
    pub semantic_weight: f64,
    /// Weight for temporal proximity
    pub temporal_weight: f64,
    /// Weight for emotional similarity
    pub emotional_weight: f64,
    /// Weight for tag overlap
    pub tag_weight: f64,
}

impl Default for SimilarityConfig {
    fn default() -> Self {
        Self {
            semantic_weight: 0.4,
            temporal_weight: 0.3,
            emotional_weight: 0.2,
            tag_weight: 0.1,
        }
    }
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
impl MemoryEntry {
    /// Create a new memory entry
    #[must_use]
    pub fn new(
        agent_id: String,
        memory_type: MemoryType,
        content: MemoryContent,
        tick: u64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id,
            memory_type,
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
            tick,
            importance: MemoryImportance::Normal,
            emotional_valence: 0.0,
            access_count: 0,
            last_accessed: None,
            tags: Vec::new(),
            related_memories: Vec::new(),
            is_consolidated: false,
            source: MemorySource::Internal,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate retention probability (0.0 to 1.0)
    #[must_use]
    pub fn retention_probability(&self, current_tick: u64, policy: &ForgettingPolicy) -> f64 {
        let age_ticks = current_tick.saturating_sub(self.tick);

        // Don't forget memories that are too new
        if age_ticks < policy.min_retention_ticks {
            return 1.0;
        }

        // Base retention from importance
        let importance_factor = policy
            .importance_factors
            .get(&self.importance)
            .copied()
            .unwrap_or(0.5);

        // Emotional boost
        let emotional_boost = self.emotional_valence.abs() * policy.emotional_boost_factor;

        // Access count boost
        let access_boost = (self.access_count as f64 * policy.access_boost_factor).min(0.5);

        // Calculate decay
        let adjusted_decay = policy.base_decay_rate * (1.0 - importance_factor);
        let retention = (1.0 - adjusted_decay).max(0.0).powf(age_ticks as f64);

        // Apply boosts
        let boosted_retention = retention + emotional_boost + access_boost;

        boosted_retention.clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Mark memory as accessed
    pub fn mark_accessed(&mut self) {
        self.access_count += 1;
        self.last_accessed = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Add a related memory
    pub fn add_related(&mut self, memory_id: String) {
        if !self.related_memories.contains(&memory_id) {
            self.related_memories.push(memory_id);
            self.access_count += 1;
        }
    }

    /// Get memory summary text
    #[must_use]
    pub fn summary(&self) -> String {
        match &self.content {
            MemoryContent::Episodic(ep) => format!(
                "[{}] {} at tick {}: {}",
                self.memory_type_str(),
                self.agent_id,
                self.tick,
                ep.event
            ),
            MemoryContent::Semantic(sm) => format!(
                "[{}] {}: {} {} {} (confidence: {:.2})",
                self.memory_type_str(),
                self.agent_id,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                sm.subject,
                sm.predicate,
                sm.object,
                sm.confidence
            ),
            MemoryContent::Procedural(pm) => format!(
                "[{}] {} knows how to: {} (success: {:.1}%)",
                self.memory_type_str(),
                self.agent_id,
                pm.skill_name,
                pm.success_rate * 100.0
            ),
            MemoryContent::Working(wm) => format!(
                "[{}] {} focus: {}",
                self.memory_type_str(),
                self.agent_id,
                wm.focus
            ),
        }
    }

    const fn memory_type_str(&self) -> &'static str {
        match self.memory_type {
            MemoryType::Sensory => "Sensory",
            MemoryType::Working => "Working",
            MemoryType::Episodic => "Episodic",
            MemoryType::Semantic => "Semantic",
            MemoryType::Procedural => "Procedural",
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MemoryContent {
    /// Get searchable text from content
    #[must_use]
    pub fn searchable_text(&self) -> String {
        match self {
            Self::Episodic(ep) => {
                format!(
                    "{} {} {}",
                    ep.event,
                    ep.narrative,
                    ep.location.as_deref().unwrap_or("")
                )
            }
            Self::Semantic(sm) => {
                format!("{} {} {}", sm.subject, sm.predicate, sm.object)
            }
            Self::Procedural(pm) => {
                format!("{} {}", pm.skill_name, pm.contexts.join(" "))
            }
            Self::Working(wm) => {
                format!("{} {}", wm.focus, wm.active_goals.join(" "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_probability_large_tick_decay() {
        let entry = MemoryEntry::new(
            "agent-1".to_string(),
            MemoryType::Episodic,
            MemoryContent::Episodic(EpisodicMemory {
                event: "Ancient event".to_string(),
                location: None,
                participants: vec![],
                sensory_details: HashMap::new(),
                narrative: "Ancient event".to_string(),
            }),
            0,
        );

        let policy = ForgettingPolicy::default();

        // Very large age ticks (larger than i32::MAX)
        let large_tick = (i32::MAX as u64) + 1_000_000;
        let p = entry.retention_probability(large_tick, &policy);
        assert!(p >= 0.0 && p <= 1.0);
        // Should have fully decayed to near 0 (ignoring boost if valence is 0)
        assert!(p < 0.01);
    }
}
