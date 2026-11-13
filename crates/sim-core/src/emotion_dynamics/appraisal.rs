//! Event Appraisal (P1-AGENT-3)

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Appraisal engine for evaluating events
#[derive(Debug, Clone)]
pub struct AppraisalEngine;

impl AppraisalEngine {
    #[must_use]
    /// new.
    pub const fn new() -> Self {
        Self
    }

    /// Appraise an event
    #[must_use]
    pub fn appraise(&self, event: &EventAppraisal) -> AppraisalResult {
        AppraisalResult {
            pleasantness: event.pleasantness(),
            goal_relevance: event.goal_relevance(),
            coping_potential: event.coping_potential(),
            urgency: event.urgency(),
            novelty: event.novelty(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for AppraisalEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of event appraisal
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AppraisalResult {
    /// pleasantness.
    pub pleasantness: f32, // -1.0 to 1.0
    /// goal relevance.
    pub goal_relevance: f32, // 0.0 to 1.0
    /// coping potential.
    pub coping_potential: f32, // 0.0 to 1.0
    /// urgency.
    pub urgency: f32, // 0.0 to 1.0
    /// novelty.
    pub novelty: f32, // 0.0 to 1.0
}

/// An event to be appraised
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAppraisal {
    /// event type.
    pub event_type: EventType,
    /// magnitude.
    pub magnitude: f32,
    /// target.
    pub target: Option<u64>, // Target agent (if any)
    /// context.
    pub context: EventContext,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Type of emotional event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// reward.
    Reward,
    /// punishment.
    Punishment,
    /// cooperation.
    Cooperation,
    /// betrayal.
    Betrayal,
    /// threat.
    Threat,
    /// opportunity.
    Opportunity,
    /// loss.
    Loss,
    /// gain.
    Gain,
    /// surprise.
    Surprise,
    /// social acceptance.
    SocialAcceptance,
    /// social rejection.
    SocialRejection,
}

impl EventAppraisal {
    /// Pleasantness of the event
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn pleasantness(&self) -> f32 {
        match self.event_type {
            EventType::Reward | EventType::Gain | EventType::Opportunity => {
                0.5 + self.magnitude * 0.5
            }
            EventType::Punishment | EventType::Loss | EventType::Threat => {
                -(0.5 + self.magnitude * 0.5)
            }
            EventType::Cooperation => 0.3 + self.magnitude * 0.4,
            EventType::Betrayal => -(0.4 + self.magnitude * 0.5),
            EventType::Surprise => 0.0,
            EventType::SocialAcceptance => 0.4 + self.magnitude * 0.3,
            EventType::SocialRejection => -(0.4 + self.magnitude * 0.3),
        }
    }

    /// Goal relevance
    #[must_use]
    pub const fn goal_relevance(&self) -> f32 {
        match self.event_type {
            EventType::Reward | EventType::Punishment => 0.9,
            EventType::Cooperation | EventType::Betrayal => 0.8,
            EventType::Threat | EventType::Opportunity => 0.7,
            EventType::Loss | EventType::Gain => 0.6,
            EventType::Surprise => 0.4,
            EventType::SocialAcceptance | EventType::SocialRejection => 0.5,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Coping potential
    #[must_use]
    pub const fn coping_potential(&self) -> f32 {
        match self.event_type {
            EventType::Cooperation => 0.9,
            EventType::Opportunity => 0.8,
            EventType::Reward | EventType::Gain => 0.7,
            EventType::Threat => 0.4,
            EventType::Punishment | EventType::Loss => 0.3,
            EventType::Betrayal => 0.2,
            _ => 0.5,
        }
    }

    /// Urgency of response
    #[must_use]
    pub const fn urgency(&self) -> f32 {
        match self.event_type {
            EventType::Threat => 0.9,
            EventType::Surprise => 0.8,
            EventType::Betrayal => 0.7,
            EventType::Opportunity => 0.6,
            EventType::Punishment => 0.5,
            _ => 0.3,
        }
    }

    /// Novelty of event
    #[must_use]
    pub const fn novelty(&self) -> f32 {
        match self.event_type {
            EventType::Surprise => 1.0,
            _ => 1.0 - self.context.expectedness,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Context for event appraisal
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EventContext {
    /// expectedness.
    pub expectedness: f32, // 0.0 (unexpected) to 1.0 (expected)
    /// controllability.
    pub controllability: f32, // 0.0 (uncontrollable) to 1.0 (controllable)
    /// agency.
    pub agency: AgencyType,
}

impl Default for EventContext {
    fn default() -> Self {
        Self {
            expectedness: 0.5,
            controllability: 0.5,
            agency: AgencyType::Other,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Agency type for event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgencyType {
    /// self agent.
    SelfAgent,
    /// other.
    Other,
    /// nature.
    Nature,
    /// chance.
    Chance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_appraisal() {
        let event = EventAppraisal {
            event_type: EventType::Reward,
            magnitude: 0.8,
            target: None,
            context: EventContext::default(),
        };

        assert!(event.pleasantness() > 0.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(event.goal_relevance() > 0.5);
    }

    #[test]
    fn test_threat_appraisal() {
        let event = EventAppraisal {
            event_type: EventType::Threat,
            magnitude: 0.9,
            target: None,
            context: EventContext::default(),
        };

        assert!(event.pleasantness() < 0.0);
        assert!(event.urgency() > 0.5);
    }

    #[test]
    fn test_appraisal_engine() {
        let engine = AppraisalEngine::new();
        let event = EventAppraisal {
            event_type: EventType::Cooperation,
            magnitude: 0.7,
            target: None,
            context: EventContext::default(),
        };

        let result = engine.appraise(&event);
        assert!(result.pleasantness > 0.0);
        assert!(result.coping_potential > 0.5);
    }
}
