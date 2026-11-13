use crate::types::{AgentId, Tick};
use serde::{Deserialize, Serialize};

use super::issue::{Issue, IssueCategory};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Recurring pattern detected in issues
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RecurringPattern {
    /// pattern id.
    pub pattern_id: u64,
    /// description.
    pub description: String,
    /// category.
    pub category: IssueCategory,
    /// frequency.
    pub frequency: usize,
    /// affected agents.
    pub affected_agents: Vec<AgentId>,
    /// first occurrence.
    pub first_occurrence: Tick,
    /// last occurrence.
    pub last_occurrence: Tick,
    /// severity trend.
    pub severity_trend: f32, // Increasing or decreasing
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl RecurringPattern {
    /// Calculate average severity from related issues
    #[must_use]
    pub fn average_severity(&self, issues: &[Issue]) -> f32 {
        let related: Vec<_> = issues
            .iter()
            .filter(|i| i.category == self.category)
            .collect();

        if related.is_empty() {
            return 0.0;
        }

        related
            .iter()
            .map(|i| i.severity.level() as f32)
            .sum::<f32>()
            / related.len() as f32
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// A lesson learned from resolving an issue
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Lesson {
    /// lesson.
    pub lesson: String,
    /// source issue.
    pub source_issue: u64,
    /// tick learned.
    pub tick_learned: Tick,
    /// application count.
    pub application_count: usize, // How many times applied
    /// effectiveness.
    pub effectiveness: f32, // 0.0 to 1.0
    /// category.
    pub category: IssueCategory,
}

impl Lesson {
    /// Create a new lesson
    #[must_use]
    pub fn new(lesson: &str, source_issue: u64, tick: Tick, category: IssueCategory) -> Self {
        Self {
            lesson: lesson.to_string(),
            source_issue,
            tick_learned: tick,
            application_count: 0,
            effectiveness: 0.5, // Start neutral
            category,
        }
    }

    /// Apply the lesson and update effectiveness
    pub fn apply(&mut self, effective: bool) {
        self.application_count += 1;

        // Update effectiveness using exponential moving average
        let alpha = 0.3;
        let outcome = if effective { 1.0 } else { 0.0 };
        self.effectiveness = (1.0 - alpha) * self.effectiveness + alpha * outcome;

        // Clamp to [0.0, 1.0]
        self.effectiveness = self.effectiveness.clamp(0.0, 1.0);
    }
}
