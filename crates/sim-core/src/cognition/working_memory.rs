//! Working Memory System with Miller's Law Capacity Limits
//!
//! Implements Miller's Law (1956): 7 ± 2 chunks capacity
//! References:
//! - Miller (1956): The magical number seven, plus or minus two
//! - Cowan (2001): The magical number 4 in short-term memory
//! - Baddeley (2000): The episodic buffer
//!
//! SIM-008: Cognitive constraints for agent decision-making

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::types::{AgentId, NormId, Position, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Working memory system with Miller's Law capacity limits
/// Miller (1956): 7 ± 2 chunks capacity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkingMemory {
    /// Current memory items (limited capacity)
    pub items: Vec<MemoryChunk>,

    /// Maximum capacity (individual differences: 5-9)
    /// Determined by dlPFC development and genetics
    pub capacity: usize,

    /// Currently used capacity
    pub used_slots: usize,

    /// Rehearsal buffer for maintaining items (max 3 items)
    pub rehearsal_queue: VecDeque<MemoryChunk>,

    /// Attention focus (limited to ~4 items per Cowan 2001)
    pub focus: Vec<MemoryChunk>,

    /// Central executive control strength [0, 1]
    pub executive_control: f32,

    /// Timestamp of last rehearsal
    pub last_rehearsal_tick: Tick,

    /// Total items ever successfully encoded
    pub total_encoded: u64,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Total items lost due to capacity limits
    pub total_lost: u64,
}

/// Memory chunk - unit of working memory storage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryChunk {
    /// Chunk content (compressed representation)
    pub content: MemoryContent,

    /// Chunk size (in slots)
    /// Simple item = 1 slot, complex concept = 2-3 slots
    pub size: usize,

    /// Priority/relevance [0.0, 1.0]
    pub priority: f32,

    /// Time when added to memory
    pub timestamp: Tick,

    /// Decay rate (how quickly forgotten)
    /// Range: [0.0, 1.0], higher = faster decay
    pub decay_rate: f32,

    /// Number of times rehearsed
    pub rehearsal_count: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MemoryChunk {
    /// Create a new memory chunk
    #[must_use]
    pub fn new(content: MemoryContent, priority: f32, tick: Tick) -> Self {
        let size = content.size_in_slots();
        let base_decay = 0.1; // Base 10% decay per tick

        Self {
            content,
            size,
            priority: priority.clamp(0.0, 1.0),
            timestamp: tick,
            decay_rate: base_decay,
            rehearsal_count: 0,
        }
    }

    /// Create a high-priority chunk (survival-critical)
    #[must_use]
    pub fn high_priority(content: MemoryContent, tick: Tick) -> Self {
        Self::new(content, 0.9, tick)
    }

    /// Create a medium-priority chunk (socially relevant)
    #[must_use]
    pub fn medium_priority(content: MemoryContent, tick: Tick) -> Self {
        Self::new(content, 0.6, tick)
    }

    /// Create a low-priority chunk (incidental)
    #[must_use]
    pub fn low_priority(content: MemoryContent, tick: Tick) -> Self {
        Self::new(content, 0.3, tick)
    }

    /// Check if this chunk should decay given current tick
    #[must_use]
    pub fn should_decay(&self, current_tick: Tick, threshold: f32) -> bool {
        let age = current_tick.saturating_sub(self.timestamp);
        // Exponential decay: P(forget) = 1 - (1 - decay_rate)^age
        let decay_prob = 1.0 - (1.0 - self.decay_rate).powi(age as i32);
        decay_prob > threshold
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get current retention strength
    #[must_use]
    pub fn retention_strength(&self, current_tick: Tick) -> f32 {
        let age = current_tick.saturating_sub(self.timestamp);
        (1.0 - self.decay_rate).powi(age as i32)
    }
}

/// Types of information that can be held in working memory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryContent {
    /// Social information about another agent
    AgentInfo {
        /// agent id.
        agent_id: AgentId,
        /// trust level.
        trust_level: f32,
        /// last encounter.
        last_encounter: Tick,
        /// interaction count.
        interaction_count: u32,
    },

    /// Location/resource information
    SpatialInfo {
        /// position.
        position: Position,
        /// resource level.
        resource_level: f32,
        /// tick.
        tick: Tick,
        /// threat level.
        threat_level: f32,
    },

    /// Economic/decision information
    EconomicInfo {
        /// opportunity id.
        opportunity_id: u64,
        /// opportunity type.
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        opportunity_type: String,
        /// value.
        value: f32,
        /// risk.
        risk: f32,
    },

    /// Cultural/social norm
    NormInfo {
        /// norm id.
        norm_id: NormId,
        /// compliance rate.
        compliance_rate: f32,
        /// importance.
        importance: f32,
    },

    /// Simple fact (generic information)
    Fact(String),

    /// Current goal/intention
    Goal {
        /// description.
        description: String,
        /// priority.
        priority: f32,
        /// deadline.
        deadline: Option<Tick>,
    },

    /// Emotional state snapshot
    EmotionSnapshot {
        /// valence.
        valence: f32,
        /// arousal.
        arousal: f32,
        /// dominant emotion.
        dominant_emotion: String,
        /// trigger.
        trigger: String,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MemoryContent {
    /// Get size in working memory slots
    /// Per Miller: chunks can vary in size (expertise increases chunk size)
    #[must_use]
    pub const fn size_in_slots(&self) -> usize {
        match self {
            // Simple facts: 1 slot
            Self::Fact(_) => 1,

            // Social info: 1-2 slots depending on complexity
            Self::AgentInfo {
                interaction_count, ..
            } => {
                if *interaction_count > 10 {
                    2 // Complex relationship
                } else {
                    1 // Simple acquaintance
                }
            }

            // Spatial: 2 slots (position + context)
            Self::SpatialInfo { .. } => 2,

            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            // Economic: 2 slots (value + risk assessment)
            Self::EconomicInfo { .. } => 2,

            // Norms: 1 slot (compressed through repetition)
            Self::NormInfo { .. } => 1,

            // Goals: 2 slots (complex structure)
            Self::Goal { .. } => 2,

            // Emotions: 1 slot
            Self::EmotionSnapshot { .. } => 1,
        }
    }

    /// Get category for chunking strategies
    #[must_use]
    pub const fn category(&self) -> MemoryCategory {
        match self {
            Self::AgentInfo { .. } => MemoryCategory::Social,
            Self::SpatialInfo { .. } => MemoryCategory::Spatial,
            Self::EconomicInfo { .. } => MemoryCategory::Economic,
            Self::NormInfo { .. } => MemoryCategory::Cultural,
            Self::Fact(_) => MemoryCategory::Semantic,
            Self::Goal { .. } => MemoryCategory::Goal,
            Self::EmotionSnapshot { .. } => MemoryCategory::Emotional,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if two memory contents can be chunked together
    #[must_use]
    pub fn can_chunk_with(&self, other: &Self) -> bool {
        self.category() == other.category()
    }
}

/// Memory content categories for chunking strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryCategory {
    /// social.
    Social,
    /// spatial.
    Spatial,
    /// economic.
    Economic,
    /// cultural.
    Cultural,
    /// semantic.
    Semantic,
    /// goal.
    Goal,
    /// emotional.
    Emotional,
}

/// Result of attempting to add an item to working memory
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkingMemoryResult {
    /// Successfully added
    Added,
    /// Replaced a lower-priority item
    /// replaced count.
    Replaced {
        /// replaced count.
        replaced_count: usize,
    },
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Could not add (memory full, insufficient priority)
    Rejected {
        /// reason.
        reason: RejectionReason,
    },
    /// Merged with existing chunk
    /// with index.
    Merged {
        /// with index.
        with_index: usize,
    },
}

/// Reasons for rejecting a memory item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionReason {
    /// Memory is full and new item priority too low
    InsufficientPriority,
    /// Item is too large even with all replacements
    TooLarge,
    /// Executive control prevented encoding (e.g., during stress)
    ExecutiveSuppression,
}

impl WorkingMemory {
    /// Minimum capacity (children/low development)
    pub const MIN_CAPACITY: usize = 3;

    /// Maximum capacity (adult with high dlPFC development)
    pub const MAX_CAPACITY: usize = 9;

    /// Default adult capacity (Miller's 7)
    pub const DEFAULT_CAPACITY: usize = 7;

    /// Focus of attention limit (Cowan's ~4)
    pub const FOCUS_LIMIT: usize = 4;

    /// Rehearsal buffer size
    pub const REHEARSAL_BUFFER_SIZE: usize = 3;

    /// Decay threshold (items above this are removed)
    pub const DECAY_THRESHOLD: f32 = 0.5;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create working memory with capacity determined by PFC development
    /// Per Miller (1956): Adult capacity 7 ± 2 (range 5-9)
    #[must_use]
    pub fn new(dlpfc_development: f32, _age: Tick) -> Self {
        // Capacity increases with dlPFC development
        // Adult capacity: 7 ± 2 (range 5-9)
        // Child capacity: 3-5 (increases with age)
        let base_capacity = 5;
        let development_bonus = (dlpfc_development * 4.0) as usize; // +0 to +4
        let capacity =
            (base_capacity + development_bonus).clamp(Self::MIN_CAPACITY, Self::MAX_CAPACITY);

        Self {
            items: Vec::with_capacity(capacity),
            capacity,
            used_slots: 0,
            rehearsal_queue: VecDeque::with_capacity(Self::REHEARSAL_BUFFER_SIZE),
            focus: Vec::with_capacity(Self::FOCUS_LIMIT),
            executive_control: 0.5,
            last_rehearsal_tick: 0,
            total_encoded: 0,
            total_lost: 0,
        }
    }

    /// Create working memory with explicit capacity (for testing)
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let clamped = capacity.clamp(Self::MIN_CAPACITY, Self::MAX_CAPACITY);
        Self {
            items: Vec::with_capacity(clamped),
            capacity: clamped,
            used_slots: 0,
            rehearsal_queue: VecDeque::with_capacity(Self::REHEARSAL_BUFFER_SIZE),
            focus: Vec::with_capacity(Self::FOCUS_LIMIT),
            executive_control: 0.5,
            last_rehearsal_tick: 0,
            total_encoded: 0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            total_lost: 0,
        }
    }

    /// Try to add item to working memory
    /// Returns result indicating success/failure
    pub fn add(&mut self, chunk: MemoryChunk) -> WorkingMemoryResult {
        let required_slots = chunk.size;

        // Check if we can merge with existing item
        if let Some(index) = self.find_mergeable(&chunk) {
            return self.merge_chunk(index, &chunk);
        }

        // Check if there's space
        if self.used_slots + required_slots <= self.capacity {
            self.used_slots += required_slots;
            self.items.push(chunk.clone());
            self.add_to_rehearsal_queue(chunk);
            self.total_encoded += 1;
            return WorkingMemoryResult::Added;
        }

        // Memory full - try to make space for the slot deficit
        let deficit = (self.used_slots + required_slots).saturating_sub(self.capacity);
        match self.make_space_for_priority(deficit, chunk.priority) {
            Some(freed_slots) if freed_slots >= deficit => {
                self.used_slots += required_slots;
                self.items.push(chunk.clone());
                self.add_to_rehearsal_queue(chunk);
                self.total_encoded += 1;
                WorkingMemoryResult::Replaced {
                    replaced_count: freed_slots,
                }
            }
            _ => {
                self.total_lost += 1;
                WorkingMemoryResult::Rejected {
                    reason: if chunk.priority < 0.5 {
                        RejectionReason::InsufficientPriority
                    } else {
                        RejectionReason::TooLarge
                    },
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Force add an item, dropping lowest priority items if necessary
    /// Returns false only if item is too large even with empty memory
    pub fn force_add(&mut self, chunk: MemoryChunk) -> bool {
        let required = chunk.size;

        if required > self.capacity {
            return false; // Can never fit
        }

        // Make space if needed
        while self.used_slots + required > self.capacity {
            if self.drop_lowest_priority().is_none() {
                break;
            }
        }

        if self.used_slots + required <= self.capacity {
            self.used_slots += required;
            self.items.push(chunk.clone());
            self.add_to_rehearsal_queue(chunk);
            self.total_encoded += 1;
            true
        } else {
            self.total_lost += 1;
            false
        }
    }

    /// Check if an item can be added without dropping anything
    #[must_use]
    pub const fn can_add(&self, chunk: &MemoryChunk) -> bool {
        self.used_slots + chunk.size <= self.capacity
    }

    /// Check if agent can process complex information
    /// Requires sufficient working memory capacity
    #[must_use]
    pub const fn can_process_complexity(&self, complexity: usize) -> bool {
        self.capacity - self.used_slots >= complexity
    }

    /// Get available slots
    #[must_use]
    pub const fn available_slots(&self) -> usize {
        self.capacity.saturating_sub(self.used_slots)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get utilization ratio
    #[must_use]
    pub fn utilization(&self) -> f32 {
        self.used_slots as f32 / self.capacity as f32
    }

    /// Make space by dropping lowest priority items
    /// Returns number of slots freed
    #[allow(dead_code)]
    fn make_space(&mut self, needed: usize) -> usize {
        let mut freed = 0;
        while freed < needed && !self.items.is_empty() {
            if let Some(removed) = self.drop_lowest_priority() {
                freed += removed;
            } else {
                break;
            }
        }
        freed
    }

    /// Make space only if new item has higher priority than what we'd drop
    fn make_space_for_priority(&mut self, needed: usize, priority: f32) -> Option<usize> {
        // Sort by priority to find what we'd need to drop
        let mut items_with_priority: Vec<(usize, f32, usize)> = self
            .items
            .iter()
            .enumerate()
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .map(|(i, chunk)| (i, chunk.priority, chunk.size))
            .collect();
        items_with_priority
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut freed = 0;
        let mut would_drop_priority = 0.0f32;
        let mut drop_count = 0;

        for &(_idx, item_priority, size) in &items_with_priority {
            if freed >= needed {
                break;
            }
            would_drop_priority = would_drop_priority.max(item_priority);
            freed += size;
            drop_count += 1;
        }

        // Only proceed if new item priority exceeds what we'd drop
        if freed >= needed && priority > would_drop_priority {
            // Actually drop the items
            let mut to_remove: Vec<usize> = items_with_priority[..drop_count]
                .iter()
                .map(|(i, _, _)| *i)
                .collect();
            to_remove.sort_by(|a, b| b.cmp(a)); // Sort descending for safe removal

            for idx in to_remove {
                self.used_slots -= self.items[idx].size;
                self.items.remove(idx);
            }

            Some(freed)
        } else {
            None
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Drop the lowest priority item
    /// Returns size of dropped item, or None if memory empty
    pub fn drop_lowest_priority(&mut self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }

        let min_idx = self
            .items
            .iter()
            .enumerate()
            .min_by(|a, b| {
                a.1.priority
                    .partial_cmp(&b.1.priority)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)?;

        let size = self.items[min_idx].size;
        self.used_slots -= size;
        self.items.remove(min_idx);
        Some(size)
    }

    /// Find an existing chunk that can be merged with the new one
    /// Only merges when content is semantically equivalent (same agent, same position, etc.)
    fn find_mergeable(&self, new_chunk: &MemoryChunk) -> Option<usize> {
        self.items.iter().enumerate().find_map(|(i, existing)| {
            // Only merge if content is actually equivalent (update existing info)
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            if Self::content_equivalent(&existing.content, &new_chunk.content) {
                Some(i)
            } else {
                None
            }
        })
    }

    /// Check if two memory contents are equivalent (same entity/location/concept)
    fn content_equivalent(a: &MemoryContent, b: &MemoryContent) -> bool {
        use MemoryContent::{
            AgentInfo, EconomicInfo, EmotionSnapshot, Fact, Goal, NormInfo, SpatialInfo,
        };
        match (a, b) {
            // Same agent - update trust level
            (AgentInfo { agent_id: id1, .. }, AgentInfo { agent_id: id2, .. }) => id1 == id2,
            // Same position - update resource/threat info
            (SpatialInfo { position: pos1, .. }, SpatialInfo { position: pos2, .. }) => {
                // Positions are equivalent if very close (within 1 unit)
                pos1.distance(pos2) < 1.0
            }
            // Same opportunity
            (
                EconomicInfo {
                    opportunity_id: id1,
                    ..
                },
                EconomicInfo {
                    opportunity_id: id2,
                    ..
                },
            ) => id1 == id2,
            // Same norm
            (NormInfo { norm_id: id1, .. }, NormInfo { norm_id: id2, .. }) => id1 == id2,
            // Same fact (exact string match)
            (Fact(s1), Fact(s2)) => s1 == s2,
            // Same goal
            (
                Goal {
                    description: d1, ..
                },
                Goal {
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    description: d2,
                    ..
                },
            ) => d1 == d2,
            // Same emotion type from same trigger
            (
                EmotionSnapshot {
                    dominant_emotion: e1,
                    trigger: t1,
                    ..
                },
                EmotionSnapshot {
                    dominant_emotion: e2,
                    trigger: t2,
                    ..
                },
            ) => e1 == e2 && t1 == t2,
            // Different categories cannot be equivalent
            _ => false,
        }
    }

    /// Merge a new chunk with an existing one
    fn merge_chunk(&mut self, index: usize, new_chunk: &MemoryChunk) -> WorkingMemoryResult {
        if let Some(existing) = self.items.get_mut(index) {
            let old_size = existing.size;
            let new_size = new_chunk.size;

            if new_size > old_size {
                let extra_slots = new_size - old_size;
                if self.used_slots + extra_slots <= self.capacity {
                    self.used_slots += extra_slots;
                    existing.size = new_size;
                    existing.content = new_chunk.content.clone();
                }
            } else if new_size < old_size {
                let freed = old_size - new_size;
                self.used_slots = self.used_slots.saturating_sub(freed);
                existing.size = new_size;
                existing.content = new_chunk.content.clone();
            } else {
                existing.content = new_chunk.content.clone();
            }

            existing.priority = (existing.priority + new_chunk.priority) / 2.0;
            existing.timestamp = new_chunk.timestamp;
            existing.rehearsal_count += 1;
            existing.decay_rate *= 0.9; // Slower decay after rehearsal
            WorkingMemoryResult::Merged { with_index: index }
        } else {
            WorkingMemoryResult::Rejected {
                reason: RejectionReason::InsufficientPriority,
            }
        }
    }

    /// Add item to rehearsal queue (limited size)
    fn add_to_rehearsal_queue(&mut self, chunk: MemoryChunk) {
        if self.rehearsal_queue.len() >= Self::REHEARSAL_BUFFER_SIZE {
            self.rehearsal_queue.pop_front();
        }
        self.rehearsal_queue.push_back(chunk);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Decay old memories based on time and decay rates
    pub fn decay(&mut self, current_tick: Tick) {
        let to_remove: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, chunk)| chunk.should_decay(current_tick, Self::DECAY_THRESHOLD))
            .map(|(i, _)| i)
            .collect();

        for &i in to_remove.iter().rev() {
            self.used_slots -= self.items[i].size;
            self.items.remove(i);
            self.total_lost += 1;
        }
    }

    /// Rehearse items to prevent decay
    /// Refreshes timestamps and reduces decay rates
    pub fn rehearse(&mut self, current_tick: Tick) {
        let rehearsal_items: Vec<MemoryChunk> = self.rehearsal_queue.iter().cloned().collect();

        for chunk in rehearsal_items {
            if let Some(existing) = self.items.iter_mut().find(|i| i.content == chunk.content) {
                // Refresh timestamp
                existing.timestamp = current_tick;
                // Reduce decay rate (better retention modulated by central executive control)
                let retention_boost = (0.9 - 0.2 * self.executive_control).clamp(0.5, 0.95);
                existing.decay_rate *= retention_boost;
                existing.decay_rate = existing.decay_rate.max(0.01); // Min decay
                                                                     // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                existing.rehearsal_count += 1;
            }
        }

        self.last_rehearsal_tick = current_tick;
    }

    /// Focus attention on subset of items (limited to ~4 per Cowan 2001)
    pub fn focus_attention(&mut self, n: usize) {
        // Sort by priority (highest first)
        self.items.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Focus on top N (max 4)
        let focus_n = n.min(Self::FOCUS_LIMIT).min(self.items.len());
        self.focus = self.items[..focus_n].to_vec();
    }

    /// Focus on specific category
    pub fn focus_on_category(&mut self, category: MemoryCategory) {
        let mut category_items: Vec<&MemoryChunk> = self
            .items
            .iter()
            .filter(|chunk| chunk.content.category() == category)
            .collect();

        // Sort by priority
        category_items.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take up to FOCUS_LIMIT
        self.focus = category_items
            .into_iter()
            .take(Self::FOCUS_LIMIT)
            .cloned()
            .collect();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get items in current focus
    #[must_use]
    pub fn focused_items(&self) -> &[MemoryChunk] {
        &self.focus
    }

    /// Get all items (sorted by priority)
    #[must_use]
    pub fn items_by_priority(&self) -> Vec<&MemoryChunk> {
        let mut items: Vec<&MemoryChunk> = self.items.iter().collect();
        items.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        items
    }

    /// Get items by category
    #[must_use]
    pub fn items_by_category(&self, category: MemoryCategory) -> Vec<&MemoryChunk> {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        self.items
            .iter()
            .filter(|chunk| chunk.content.category() == category)
            .collect()
    }

    /// Chunking strategy - group related items
    /// This is how experts increase effective capacity
    pub fn chunk_related(&mut self) {
        // Group items by category
        let mut by_category: std::collections::HashMap<MemoryCategory, Vec<usize>> =
            std::collections::HashMap::new();

        for (i, chunk) in self.items.iter().enumerate() {
            by_category
                .entry(chunk.content.category())
                .or_default()
                .push(i);
        }

        // For categories with 3+ items, we could create super-chunks
        // (This is a simplified implementation)
        for (_category, indices) in by_category {
            if indices.len() >= 3 {
                // Mark these as chunkable (in a full implementation,
                // would actually merge into hierarchical chunks)
                // For now, just reduce their individual slot costs slightly
                for &idx in &indices {
                    if let Some(chunk) = self.items.get_mut(idx) {
                        // Expertise reduces chunk size
                        chunk.size = chunk.size.max(1);
                    }
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.focus.clear();
        self.rehearsal_queue.clear();
        self.used_slots = 0;
    }

    /// Update executive control based on stress/arousal
    /// High stress reduces executive control
    pub fn update_executive_control(&mut self, stress: f32, arousal: f32) {
        // Stress and arousal both reduce executive control
        let stress_factor = 1.0 - stress.clamp(0.0, 1.0);
        let arousal_factor = 1.0 - (arousal.clamp(0.0, 1.0) * 0.5); // Arousal has less effect
        self.executive_control = (stress_factor * arousal_factor * 0.5 + 0.25).clamp(0.0, 1.0);
    }

    /// Get memory statistics
    #[must_use]
    pub fn stats(&self) -> WorkingMemoryStats {
        WorkingMemoryStats {
            capacity: self.capacity,
            used_slots: self.used_slots,
            available_slots: self.available_slots(),
            item_count: self.items.len(),
            focus_count: self.focus.len(),
            utilization: self.utilization(),
            total_encoded: self.total_encoded,
            total_lost: self.total_lost,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            retention_rate: if self.total_encoded > 0 {
                self.total_encoded.saturating_sub(self.total_lost) as f32
                    / self.total_encoded as f32
            } else {
                1.0
            },
        }
    }
}

/// Working memory statistics
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorkingMemoryStats {
    /// capacity.
    pub capacity: usize,
    /// used slots.
    pub used_slots: usize,
    /// available slots.
    pub available_slots: usize,
    /// item count.
    pub item_count: usize,
    /// focus count.
    pub focus_count: usize,
    /// utilization.
    pub utilization: f32,
    /// total encoded.
    pub total_encoded: u64,
    /// total lost.
    pub total_lost: u64,
    /// retention rate.
    pub retention_rate: f32,
}

impl Default for WorkingMemoryStats {
    fn default() -> Self {
        Self {
            capacity: WorkingMemory::DEFAULT_CAPACITY,
            used_slots: 0,
            available_slots: WorkingMemory::DEFAULT_CAPACITY,
            item_count: 0,
            focus_count: 0,
            utilization: 0.0,
            total_encoded: 0,
            total_lost: 0,
            retention_rate: 1.0,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Decision option for agent decision-making
#[derive(Debug, Clone, PartialEq)]
pub struct DecisionOption {
    /// id.
    pub id: u64,
    /// description.
    pub description: String,
    /// expected value.
    pub expected_value: f32,
    /// complexity.
    pub complexity: usize, // Working memory slots required
    /// risk.
    pub risk: f32,
}

impl DecisionOption {
    /// Create a simple decision option (1 slot)
    #[must_use]
    pub fn simple(id: u64, description: &str, expected_value: f32) -> Self {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Self {
            id,
            description: description.to_string(),
            expected_value,
            complexity: 1,
            risk: 0.0,
        }
    }

    /// Create a complex decision option (multiple slots)
    #[must_use]
    pub fn complex(
        id: u64,
        description: &str,
        expected_value: f32,
        complexity: usize,
        risk: f32,
    ) -> Self {
        Self {
            id,
            description: description.to_string(),
            expected_value,
            complexity,
            risk: risk.clamp(0.0, 1.0),
        }
    }
}

/// Perceived information for encoding into working memory
#[derive(Debug, Clone)]
pub struct PerceivedInfo {
    /// content.
    pub content: MemoryContent,
    /// priority.
    pub priority: f32,
    /// distance.
    pub distance: f32, // Distance to stimulus (affects encoding)
}

impl PerceivedInfo {
    /// Create perceived info with automatic priority calculation
    #[must_use]
    pub fn new(content: MemoryContent, base_priority: f32, distance: f32) -> Self {
        // Priority decreases with distance
        let distance_factor = (1.0 / (1.0 + distance)).clamp(0.1, 1.0);
        let priority = (base_priority * distance_factor).clamp(0.0, 1.0);

        Self {
            content,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            priority,
            distance,
        }
    }
}

impl From<PerceivedInfo> for MemoryChunk {
    fn from(info: PerceivedInfo) -> Self {
        Self::new(info.content, info.priority, 0) // Tick will be set when added
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Miller's Law Tests ===

    #[test]
    fn test_millers_law_capacity_limits() {
        // Test that capacity respects Miller's 7±2
        let wm = WorkingMemory::with_capacity(7);
        assert_eq!(wm.capacity, 7);
        assert!(wm.capacity >= 5 && wm.capacity <= 9);

        // Test clamping
        let too_small = WorkingMemory::with_capacity(2);
        assert_eq!(too_small.capacity, WorkingMemory::MIN_CAPACITY);

        let too_large = WorkingMemory::with_capacity(15);
        assert_eq!(too_large.capacity, WorkingMemory::MAX_CAPACITY);
    }

    #[test]
    fn test_capacity_by_dlpfc_development() {
        // Low development -> low capacity
        let child_wm = WorkingMemory::new(0.0, 0);
        assert!(child_wm.capacity >= WorkingMemory::MIN_CAPACITY);
        assert!(child_wm.capacity <= 6);

        // Medium development -> medium capacity
        let teen_wm = WorkingMemory::new(0.5, 150);
        assert!(teen_wm.capacity >= 5 && teen_wm.capacity <= 8);

        // Full development -> full capacity (7±2)
        let adult_wm = WorkingMemory::new(1.0, 300);
        assert!(adult_wm.capacity >= 7 && adult_wm.capacity <= WorkingMemory::MAX_CAPACITY);
    }

    #[test]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn test_adding_items_respects_capacity() {
        let mut wm = WorkingMemory::with_capacity(5);

        // Add 5 single-slot items (should fit)
        for i in 0..5 {
            let chunk = MemoryChunk::new(MemoryContent::Fact(format!("fact_{i}")), 0.5, 0);
            let result = wm.add(chunk);
            assert!(
                matches!(result, WorkingMemoryResult::Added),
                "Failed to add item {i}: {result:?}"
            );
        }

        assert_eq!(wm.used_slots, 5);
        assert_eq!(wm.available_slots(), 0);

        // 6th item should be rejected (low priority)
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let low_priority = MemoryChunk::new(MemoryContent::Fact("overflow".to_string()), 0.1, 0);
        let result = wm.add(low_priority);
        assert!(matches!(result, WorkingMemoryResult::Rejected { .. }));
    }

    #[test]
    fn test_high_priority_replaces_low_priority() {
        let mut wm = WorkingMemory::with_capacity(3);

        // Add 3 low-priority items
        for i in 0..3 {
            let chunk = MemoryChunk::new(MemoryContent::Fact(format!("low_{i}")), 0.3, 0);
            wm.add(chunk);
        }

        assert_eq!(wm.used_slots, 3);

        // High priority item should replace low priority items
        let high_priority = MemoryChunk::new(MemoryContent::Fact("urgent".to_string()), 0.9, 1);
        let result = wm.add(high_priority);

        assert!(
            matches!(result, WorkingMemoryResult::Replaced { .. }),
            "High priority should replace: {result:?}"
        );
        assert!(wm.items.iter().any(|item| {
            if let MemoryContent::Fact(s) = &item.content {
                s == "urgent"
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_decay_removes_old_memories() {
        let mut wm = WorkingMemory::with_capacity(7);

        // Add item with high decay rate
        let chunk = MemoryChunk {
            content: MemoryContent::Fact("fleeting".to_string()),
            size: 1,
            priority: 0.5,
            timestamp: 0,
            decay_rate: 0.5, // 50% decay per tick
            rehearsal_count: 0,
        };
        wm.add(chunk);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(wm.items.len(), 1);

        // After 2 ticks, should be decayed (1 - (1-0.5)^2 = 0.75 > 0.5 threshold)
        wm.decay(2);
        assert_eq!(wm.items.len(), 0);
    }

    #[test]
    fn test_rehearsal_prevents_decay() {
        let mut wm = WorkingMemory::with_capacity(7);

        // Add item at tick 0
        let chunk = MemoryChunk::new(MemoryContent::Fact("important".to_string()), 0.9, 0);
        wm.add(chunk);

        // Rehearse multiple times to significantly reduce decay rate
        for tick in 1..=3 {
            wm.rehearse(tick);
        }

        // After multiple rehearsals, decay rate should be significantly reduced
        let item = &wm.items[0];
        assert!(
            item.decay_rate < 0.06,
            "Decay rate should be reduced to < 0.06, got {}",
            item.decay_rate
        );

        // Should still be present after many ticks due to reduced decay
        wm.decay(10);
        assert_eq!(
            wm.items.len(),
            1,
            "Item should still be present after decay"
        );
    }

    #[test]
    fn test_focus_limited_to_four() {
        let mut wm = WorkingMemory::with_capacity(9); // Max capacity

        // Add 9 items with different priorities
        for i in 0..9 {
            let chunk =
                MemoryChunk::new(MemoryContent::Fact(format!("item_{i}")), i as f32 / 9.0, 0);
            wm.add(chunk);
        }

        // Focus on top 10 (should be limited to 4)
        wm.focus_attention(10);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(wm.focus.len(), WorkingMemory::FOCUS_LIMIT);

        // Focus should contain highest priority items
        assert!(wm.focus.iter().all(|item| item.priority >= 0.5));
    }

    #[test]
    fn test_chunk_sizes() {
        // Simple fact: 1 slot
        let fact = MemoryContent::Fact("simple".to_string());
        assert_eq!(fact.size_in_slots(), 1);

        // Spatial: 2 slots
        let spatial = MemoryContent::SpatialInfo {
            position: Position::new(0.0, 0.0, 0.0),
            resource_level: 0.5,
            tick: 0,
            threat_level: 0.0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        };
        assert_eq!(spatial.size_in_slots(), 2);

        // Complex agent info: 2 slots
        let complex_agent = MemoryContent::AgentInfo {
            agent_id: 1,
            trust_level: 0.5,
            last_encounter: 0,
            interaction_count: 15, // > 10 = complex
        };
        assert_eq!(complex_agent.size_in_slots(), 2);
    }

    #[test]
    fn test_complexity_requirements() {
        let mut wm = WorkingMemory::with_capacity(5);

        // Fill with 5 single-slot items
        for i in 0..5 {
            wm.add(MemoryChunk::new(
                MemoryContent::Fact(format!("filler_{i}")),
                0.5,
                0,
            ));
        }

        // Should not be able to process complexity requiring 2 slots
        assert!(!wm.can_process_complexity(2));

        // Should be able to process 1 slot
        wm.items.pop(); // Free one slot
        wm.used_slots -= 1;
        assert!(wm.can_process_complexity(1));
    }

    #[test]
    fn test_categorization_and_chunking() {
        let mut wm = WorkingMemory::with_capacity(7);

        // Add items of different categories
        wm.add(MemoryChunk::new(
            MemoryContent::Fact("fact1".to_string()),
            0.5,
            0,
        ));
        wm.add(MemoryChunk::new(
            MemoryContent::Fact("fact2".to_string()),
            0.6,
            0,
            // TAG: surface=simulation owner=research-team rule=SIM-001
        ));
        wm.add(MemoryChunk::new(
            MemoryContent::Fact("fact3".to_string()),
            0.7,
            0,
        ));

        // Should be able to retrieve by category
        let facts = wm.items_by_category(MemoryCategory::Semantic);
        assert_eq!(facts.len(), 3);

        // Focus on semantic category
        wm.focus_on_category(MemoryCategory::Semantic);
        assert_eq!(wm.focus.len(), 3);
    }

    #[test]
    fn test_utilization_tracking() {
        // Use capacity 5 for predictable utilization calculations
        let mut wm = WorkingMemory::with_capacity(5);

        assert_eq!(wm.utilization(), 0.0);

        wm.add(MemoryChunk::new(
            MemoryContent::Fact("1".to_string()),
            0.5,
            0,
        ));
        assert!((wm.utilization() - 0.2).abs() < 0.01); // 1/5 = 0.2

        wm.add(MemoryChunk::new(
            MemoryContent::Fact("2".to_string()),
            0.5,
            0,
        ));
        assert!((wm.utilization() - 0.4).abs() < 0.01); // 2/5 = 0.4
    }

    #[test]
    fn test_force_add() {
        let mut wm = WorkingMemory::with_capacity(3);

        // Fill memory
        for i in 0..3 {
            wm.add(MemoryChunk::new(
                MemoryContent::Fact(format!("item_{i}")),
                0.9, // High priority
                0,
            ));
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Normal add should fail
        let result = wm.add(MemoryChunk::new(
            MemoryContent::Fact("normal".to_string()),
            0.5,
            0,
        ));
        assert!(matches!(result, WorkingMemoryResult::Rejected { .. }));

        // Force add should succeed by dropping lowest priority
        let success = wm.force_add(MemoryChunk::new(
            MemoryContent::Fact("forced".to_string()),
            0.95,
            0,
        ));
        assert!(success);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Should have replaced the lowest priority item
        assert!(wm.items.iter().any(|item| {
            if let MemoryContent::Fact(s) = &item.content {
                s == "forced"
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_stats_accuracy() {
        let mut wm = WorkingMemory::with_capacity(5);

        let stats_before = wm.stats();
        assert_eq!(stats_before.total_encoded, 0);
        assert_eq!(stats_before.total_lost, 0);

        // Fill memory completely with high priority items
        for i in 0..5 {
            wm.add(MemoryChunk::new(
                MemoryContent::Fact(format!("kept_{i}")),
                0.9,
                0,
            ));
        }

        // Try to add low priority items (all should be rejected - memory full)
        for i in 0..5 {
            wm.add(MemoryChunk::new(
                MemoryContent::Fact(format!("lost_{i}")),
                0.1, // Low priority, should be rejected
                0,
            ));
        }

        let stats_after = wm.stats();
        assert_eq!(stats_after.total_encoded, 5); // Only high priority got in
        assert!(
            stats_after.total_lost >= 5,
            "Expected at least 5 lost, got {}",
            stats_after.total_lost
        );
        assert!(stats_after.retention_rate < 1.0);
    }

    #[test]
    fn test_executive_control_update() {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut wm = WorkingMemory::with_capacity(7);

        // Default executive control
        assert_eq!(wm.executive_control, 0.5);

        // High stress reduces control
        wm.update_executive_control(0.9, 0.5);
        assert!(wm.executive_control < 0.5);

        // Low stress maintains control
        wm.update_executive_control(0.1, 0.2);
        assert!(wm.executive_control > 0.3);
    }

    #[test]
    fn test_retention_strength_calculation() {
        let chunk = MemoryChunk {
            content: MemoryContent::Fact("test".to_string()),
            size: 1,
            priority: 0.5,
            timestamp: 0,
            decay_rate: 0.1,
            rehearsal_count: 0,
        };

        // At tick 0, retention should be 1.0
        assert_eq!(chunk.retention_strength(0), 1.0);

        // At tick 5: (1 - 0.1)^5 ≈ 0.59
        let strength_5 = chunk.retention_strength(5);
        assert!((strength_5 - 0.59).abs() < 0.05);

        // At tick 10: (1 - 0.1)^10 ≈ 0.35
        let strength_10 = chunk.retention_strength(10);
        assert!((strength_10 - 0.35).abs() < 0.05);
    }

    // === Decision-Making Tests ===

    #[test]
    fn test_decision_option_creation() {
        let simple = DecisionOption::simple(1, "go_left", 0.5);
        assert_eq!(simple.complexity, 1);
        assert_eq!(simple.risk, 0.0);

        let complex = DecisionOption::complex(2, "trade", 0.8, 3, 0.3);
        assert_eq!(complex.complexity, 3);
        assert_eq!(complex.risk, 0.3);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_perceived_info_priority() {
        // Close stimulus -> high priority
        let close = PerceivedInfo::new(
            MemoryContent::Fact("danger".to_string()),
            0.9,
            1.0, // close
        );

        // Far stimulus -> lower priority
        let far = PerceivedInfo::new(
            MemoryContent::Fact("danger".to_string()),
            0.9,
            10.0, // far
        );

        assert!(close.priority > far.priority);
    }

    #[test]
    fn test_merge_chunk_updates_content_and_preserves_slot_invariant() {
        let mut wm = WorkingMemory::with_capacity(5);

        let initial_agent = MemoryContent::AgentInfo {
            agent_id: 101,
            trust_level: 0.4,
            last_encounter: 10,
            interaction_count: 5,
        };
        let add_res = wm.add(MemoryChunk::new(initial_agent, 0.6, 10));
        assert_eq!(add_res, WorkingMemoryResult::Added);
        assert_eq!(wm.used_slots, 1);

        let updated_agent = MemoryContent::AgentInfo {
            agent_id: 101,
            trust_level: 0.85,
            last_encounter: 25,
            interaction_count: 15,
        };
        let merge_res = wm.add(MemoryChunk::new(updated_agent.clone(), 0.8, 25));
        assert_eq!(merge_res, WorkingMemoryResult::Merged { with_index: 0 });

        assert_eq!(wm.items[0].content, updated_agent);
        assert_eq!(wm.items[0].size, 2);
        assert_eq!(wm.used_slots, 2);
    }
}
