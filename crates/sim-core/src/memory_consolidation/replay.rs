//! Memory Replay System (P1-AGENT-4)

use serde::{Deserialize, Serialize};

use super::{LongTermStorage, WorkingMemoryItem};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Replay scheduler for memory consolidation
#[derive(Debug, Clone)]
pub struct ReplayScheduler {
    /// config.
    pub config: ReplayConfig,
    /// scheduled events.
    pub scheduled_events: Vec<ReplayEvent>,
    /// replay queue.
    pub replay_queue: Vec<u64>, // Memory IDs to replay
}

/// Configuration for memory replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayConfig {
    /// Probability of replaying a memory
    pub replay_probability: f32,
    /// Number of replays per cycle
    pub replays_per_cycle: usize,
    /// Time compression factor (replay speed)
    pub time_compression: f32,
    /// Importance boost from replay
    pub replay_boost: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            replay_probability: 0.1,
            replays_per_cycle: 5,
            time_compression: 10.0,
            replay_boost: 0.05,
        }
    }
}

/// Replay event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEvent {
    /// memory id.
    pub memory_id: u64,
    /// scheduled time.
    pub scheduled_time: u64,
    /// priority.
    pub priority: ReplayPriority,
    /// replay count.
    pub replay_count: u32,
}

/// Priority for replay
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayPriority {
    /// low.
    Low,
    /// medium.
    Medium,
    /// high.
    High,
    /// critical.
    Critical,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ReplayScheduler {
    /// Create new replay scheduler
    #[must_use]
    pub const fn new(config: ReplayConfig) -> Self {
        Self {
            config,
            scheduled_events: Vec::new(),
            replay_queue: Vec::new(),
        }
    }

    /// Schedule memories for replay
    pub fn schedule_replay(&mut self, memories: &[WorkingMemoryItem]) -> Vec<WorkingMemoryItem> {
        let mut replayed = memories.to_vec();

        for memory in &mut replayed {
            // Higher importance/emotion = higher priority
            let priority = if memory.importance > 0.8 || memory.emotional_valence.abs() > 0.7 {
                ReplayPriority::High
            } else if memory.importance > 0.5 {
                ReplayPriority::Medium
            } else {
                ReplayPriority::Low
            };

            let event = ReplayEvent {
                memory_id: memory.id,
                scheduled_time: memory.timestamp + 100, // Schedule for later
                priority,
                replay_count: 0,
            };

            self.scheduled_events.push(event);
        }

        replayed
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Run scheduled replays
    pub fn run_scheduled_replay(&mut self, storage: &mut LongTermStorage) {
        // Sort by priority and time
        self.scheduled_events.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.scheduled_time.cmp(&b.scheduled_time))
        });

        // Drain the top replays_per_cycle events to prevent duplicate accumulation
        let count = self
            .config
            .replays_per_cycle
            .min(self.scheduled_events.len());
        let to_replay: Vec<_> = self.scheduled_events.drain(..count).collect();

        for mut event in to_replay {
            if let Some(memory) = storage.get_mut(event.memory_id) {
                // Replay strengthens the memory
                memory.strengthen(self.config.replay_boost);
                memory.access_count += 1;

                // Increase replay count
                event.replay_count += 1;

                // Reschedule if still important and below max replays
                if memory.importance > 0.3 && event.replay_count < 5 {
                    event.scheduled_time += 500;
                    self.scheduled_events.push(event);
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Spontaneous replay during rest (hippocampal replay)
    pub fn spontaneous_replay(&mut self, storage: &mut LongTermStorage) -> Vec<u64> {
        let mut replayed = Vec::new();

        // Random selection weighted by importance
        let candidates: Vec<_> = storage
            .all_memories()
            .filter(|m| m.importance > 0.3)
            .map(|m| (m.id, m.importance))
            .collect();

        for (id, importance) in candidates {
            let hash = id.wrapping_mul(0x9e3779b97f4a7c15);
            let p = ((hash >> 32) as u32 as f32) / (u32::MAX as f32);
            if p < importance * self.config.replay_probability {
                if let Some(memory) = storage.get_mut(id) {
                    memory.strengthen(self.config.replay_boost * 0.5);
                    replayed.push(id);
                }
            }
        }

        replayed
    }

    /// Sharp-wave ripple replay (compressed sequential replay)
    pub fn sharp_wave_ripple(
        &mut self,
        storage: &mut LongTermStorage,
        sequence: &[u64],
    ) -> Vec<u64> {
        let mut strengthened = Vec::new();

        // Replay sequence in compressed form
        for &memory_id in sequence {
            if let Some(memory) = storage.get_mut(memory_id) {
                memory.strengthen(self.config.replay_boost * 2.0);
                strengthened.push(memory_id);
            }
        }

        // Create sequence associations
        for window in sequence.windows(2) {
            if let [a, b] = window {
                storage.strengthen_association(*a, *b, 0.5);
            }
        }

        strengthened
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get replay statistics
    #[must_use]
    pub fn statistics(&self) -> ReplayStatistics {
        ReplayStatistics {
            scheduled_count: self.scheduled_events.len(),
            avg_replay_count: if self.scheduled_events.is_empty() {
                0.0
            } else {
                self.scheduled_events
                    .iter()
                    .map(|e| e.replay_count)
                    .sum::<u32>() as f32
                    / self.scheduled_events.len() as f32
            },
            high_priority_count: self
                .scheduled_events
                .iter()
                .filter(|e| {
                    e.priority == ReplayPriority::High || e.priority == ReplayPriority::Critical
                })
                .count(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Replay statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReplayStatistics {
    /// scheduled count.
    pub scheduled_count: usize,
    /// avg replay count.
    pub avg_replay_count: f32,
    /// high priority count.
    pub high_priority_count: usize,
}

#[cfg(test)]
mod tests {
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
            timestamp: 0,
            context: MemoryContext {
                location: "Test".to_string(),
                emotional_state: (0.0, 0.5),
                social_context: vec![],
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            },
        }
    }

    #[test]
    fn test_schedule_replay() {
        let mut scheduler = ReplayScheduler::new(ReplayConfig::default());
        let memories = vec![create_test_memory(1, 0.9), create_test_memory(2, 0.4)];

        let _replayed = scheduler.schedule_replay(&memories);
        assert_eq!(scheduler.scheduled_events.len(), 2);
    }

    #[test]
    fn test_replay_statistics() {
        let mut scheduler = ReplayScheduler::new(ReplayConfig::default());
        let memories = vec![create_test_memory(1, 0.9)];
        scheduler.schedule_replay(&memories);

        let stats = scheduler.statistics();
        assert_eq!(stats.scheduled_count, 1);
    }

    #[test]
    fn test_replay_config_default() {
        let config = ReplayConfig::default();
        assert_eq!(config.replay_probability, 0.1);
        assert_eq!(config.replays_per_cycle, 5);
        assert_eq!(config.time_compression, 10.0);
        assert_eq!(config.replay_boost, 0.05);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_replay_priority_ordering() {
        assert!(ReplayPriority::Low < ReplayPriority::Medium);
        assert!(ReplayPriority::Medium < ReplayPriority::High);
        assert!(ReplayPriority::High < ReplayPriority::Critical);
    }

    #[test]
    fn test_schedule_replay_priority_high() {
        let mut scheduler = ReplayScheduler::new(ReplayConfig::default());
        let mut memory = create_test_memory(1, 0.9);
        memory.emotional_valence = 0.8; // High emotional valence
        scheduler.schedule_replay(&[memory]);

        assert_eq!(scheduler.scheduled_events.len(), 1);
        assert_eq!(scheduler.scheduled_events[0].priority, ReplayPriority::High);
    }

    #[test]
    fn test_schedule_replay_priority_low() {
        let mut scheduler = ReplayScheduler::new(ReplayConfig::default());
        let mut memory = create_test_memory(1, 0.3);
        memory.emotional_valence = 0.1; // Low emotional valence
        scheduler.schedule_replay(&[memory]);

        assert_eq!(scheduler.scheduled_events.len(), 1);
        assert_eq!(scheduler.scheduled_events[0].priority, ReplayPriority::Low);
    }

    #[test]
    fn test_replay_scheduler_new() {
        let config = ReplayConfig::default();
        let scheduler = ReplayScheduler::new(config);
        assert!(scheduler.scheduled_events.is_empty());
        assert!(scheduler.replay_queue.is_empty());
    }

    #[test]
    fn test_replay_drain_prevents_exponential_duplication() {
        let mut scheduler = ReplayScheduler::new(ReplayConfig {
            replays_per_cycle: 5,
            ..Default::default()
        });
        let mut storage = LongTermStorage::new();
        let memory = create_test_memory(42, 0.9);
        storage.store(memory.clone());

        scheduler.schedule_replay(&[memory]);
        assert_eq!(scheduler.scheduled_events.len(), 1);

        // Run 10 cycles; queue length must remain bounded <= 1 (never duplicating to 2^10)
        for _ in 0..10 {
            scheduler.run_scheduled_replay(&mut storage);
            assert!(
                scheduler.scheduled_events.len() <= 1,
                "Queue size {} exceeded bounded limit",
                scheduler.scheduled_events.len()
            );
        }

        // After 5 replays, event must be retired cleanly
        assert_eq!(scheduler.scheduled_events.len(), 0);
    }
}
