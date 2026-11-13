//! Collective Intentionality Module: Group Agency and We-Intentions (Searle 1990)
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION collective intentionality, not product features
//!
//! Based on John Searle's concept of collective intentionality (Searle 1990):
//! Groups can have shared intentions (we-intentions) not reducible to individual intentions.
//!
//! Key concepts:
//! - We-intentions: Shared intentions that are irreducibly collective
//! - Constitutive rules: Rules of the form "X counts as Y in context C"
//! - Institutional facts: Facts that exist by collective agreement

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{AgentId, GroupId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// A collective goal that a group pursues together
/// Per Searle 1990: Collective goals are not reducible to individual goals
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Goal {
    /// Acquire specific resources
    /// Field.
    /// resource type.
    ResourceAcquisition {
        /// resource type.
        resource_type: String,
        /// amount.
        amount: f32,
    },
    /// Defend territory or group members
    Defense {
        /// target.
        target: String,
    },
    /// Build infrastructure or institutions
    Construction {
        /// project.
        project: String,
    },
    /// Social coordination for mutual benefit
    Coordination {
        /// activity.
        activity: String,
    },
    /// Exploration or information gathering
    Exploration {
        /// area.
        area: String,
    },
    /// Custom goal with description
    Custom {
        /// description.
        description: String,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Goal {
    /// Get a string representation of the goal for we-intention formation
    #[must_use]
    pub fn to_action_string(&self) -> String {
        match self {
            Self::ResourceAcquisition {
                resource_type,
                amount,
            } => {
                format!("Acquire {amount} units of {resource_type}")
            }
            Self::Defense { target } => format!("Defend {target}"),
            Self::Construction { project } => format!("Build {project}"),
            Self::Coordination { activity } => activity.clone(),
            Self::Exploration { area } => format!("Explore {area}"),
            Self::Custom { description } => description.clone(),
        }
    }

    /// Check if this goal requires mutual belief among participants
    #[must_use]
    pub const fn requires_mutual_belief(&self) -> bool {
        matches!(
            self,
            Self::Defense { .. } | Self::Coordination { .. } | Self::Construction { .. }
        )
    }

    /// Check if this goal requires common knowledge among participants
    #[must_use]
    pub const fn requires_common_knowledge(&self) -> bool {
        matches!(self, Self::Defense { .. } | Self::Coordination { .. })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for Goal {
    fn default() -> Self {
        Self::Custom {
            description: "Unspecified collective goal".to_string(),
        }
    }
}

/// A collective group with shared intentionality
/// Per Searle 1990: Groups are constituted by we-intentions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveGroup {
    /// Unique identifier for the group
    pub id: GroupId,
    /// Members of the group
    pub members: Vec<AgentId>,
    /// Collective goal the group pursues
    pub collective_goal: Goal,
    /// Commitment level of the group [0.0, 1.0]
    /// Higher values indicate stronger collective commitment
    pub commitment_level: f32,
    /// When the group was formed
    pub formed_tick: Tick,
    /// Whether the group has achieved mutual belief about the goal
    pub mutual_belief_established: bool,
    /// Whether the group has achieved common knowledge about the goal
    pub common_knowledge_established: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl CollectiveGroup {
    /// Create a new collective group
    #[must_use]
    pub const fn new(
        id: GroupId,
        members: Vec<AgentId>,
        collective_goal: Goal,
        formed_tick: Tick,
    ) -> Self {
        let requires_mutual = collective_goal.requires_mutual_belief();
        let requires_common = collective_goal.requires_common_knowledge();

        Self {
            id,
            members,
            collective_goal,
            commitment_level: 0.5,
            formed_tick,
            mutual_belief_established: !requires_mutual, // Auto-true if not required
            common_knowledge_established: !requires_common, // Auto-true if not required
        }
    }

    /// Update commitment level based on member participation
    pub fn update_commitment(&mut self, participation_rate: f32) {
        // Commitment increases with participation, bounded by [0.1, 1.0]
        self.commitment_level = (0.3 + 0.7 * participation_rate).clamp(0.1, 1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if the group has achieved collective intentionality
    /// Per Searle: Requires both mutual belief and commitment
    #[must_use]
    pub fn has_collective_intentionality(&self) -> bool {
        !self.members.is_empty() && self.mutual_belief_established && self.commitment_level > 0.3
    }

    /// Get the number of members in the group
    #[must_use]
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Check if an agent is a member of this group
    #[must_use]
    pub fn has_member(&self, agent_id: AgentId) -> bool {
        self.members.contains(&agent_id)
    }

    /// Add a member to the group
    pub fn add_member(&mut self, agent_id: AgentId) {
        if !self.members.contains(&agent_id) {
            self.members.push(agent_id);
            self.commitment_level = (self.commitment_level * 0.95).max(0.1);
        }
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Remove a member from the group
    pub fn remove_member(&mut self, agent_id: AgentId) {
        self.members.retain(|&id| id != agent_id);
        self.commitment_level = (self.commitment_level * 0.9).max(0.1);
    }
}

/// We-intention: Collective intention that is irreducibly plural
/// Per Searle 1990: "We-intend" is not reducible to "I-intend" + beliefs about others
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeIntention {
    /// The collective action intended
    pub action: String,
    /// Participants sharing this intention
    pub participants: Vec<AgentId>,
    /// Whether mutual belief has been established
    /// Mutual belief: Each participant believes the goal, and believes others believe it
    pub mutual_belief: bool,
    /// Whether common knowledge has been established
    /// Common knowledge: Everyone knows, everyone knows everyone knows, etc.
    pub common_knowledge: bool,
    /// Strength of the we-intention [0.0, 1.0]
    pub strength: f32,
    /// When the we-intention was formed
    pub formed_tick: Tick,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl WeIntention {
    /// Create a new we-intention
    #[must_use]
    pub const fn new(action: String, participants: Vec<AgentId>, formed_tick: Tick) -> Self {
        Self {
            action,
            participants,
            mutual_belief: false,
            common_knowledge: false,
            strength: 0.5,
            formed_tick,
        }
    }

    /// Check if this we-intention is valid (has participants and action)
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.participants.is_empty() && !self.action.is_empty()
    }

    /// Check if this we-intention satisfies Searle's conditions
    /// Per Searle 1990: Requires mutual belief for basic collective action
    #[must_use]
    pub fn satisfies_searle_conditions(&self) -> bool {
        self.mutual_belief && self.strength > 0.3 && !self.participants.is_empty()
    }

    /// Update strength based on participant commitment
    pub fn update_strength(&mut self, avg_commitment: f32) {
        self.strength = avg_commitment.clamp(0.0, 1.0);
    }

    /// Establish mutual belief among participants
    pub fn establish_mutual_belief(&mut self) {
        self.mutual_belief = true;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Establish common knowledge among participants
    /// Per Searle: Common knowledge is stronger than mutual belief
    pub fn establish_common_knowledge(&mut self) {
        self.mutual_belief = true;
        self.common_knowledge = true;
    }

    /// Check if a specific agent participates in this we-intention
    #[must_use]
    pub fn has_participant(&self, agent_id: AgentId) -> bool {
        self.participants.contains(&agent_id)
    }
}

/// Constitutive rule: "X counts as Y in context C"
/// Per Searle 1995: Constitutive rules create institutional facts
/// Example: "This piece of paper counts as money in the US"
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstitutiveRule {
    /// Brute fact (the physical/material reality)
    /// Example: "`mutual_trust_above_threshold`"
    pub brute_fact: String,
    /// Institutional fact (the social reality created)
    /// Example: "`social_contract_exists`"
    pub institutional_fact: String,
    /// Context in which the rule applies
    /// Example: "`group_formation`"
    pub context: String,
    /// Threshold for applying the rule [0.0, 1.0]
    pub threshold: f32,
    /// Confidence level in the institutional fact [0.0, 1.0]
    pub confidence_factor: f32,
}

impl ConstitutiveRule {
    /// Create a new constitutive rule
    pub fn new(
        brute_fact: impl Into<String>,
        institutional_fact: impl Into<String>,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        context: impl Into<String>,
    ) -> Self {
        Self {
            brute_fact: brute_fact.into(),
            institutional_fact: institutional_fact.into(),
            context: context.into(),
            threshold: 0.5,
            confidence_factor: 0.9,
        }
    }

    /// Check if the rule applies given a brute fact strength
    #[must_use]
    pub fn applies(&self, brute_strength: f32) -> bool {
        brute_strength > self.threshold
    }

    /// Apply the rule to get the institutional fact strength
    #[must_use]
    pub fn apply(&self, brute_strength: f32) -> Option<f32> {
        if self.applies(brute_strength) {
            Some(brute_strength * self.confidence_factor)
        } else {
            None
        }
    }
}

/// Collective intentionality system implementing Searle's framework
/// Manages groups, we-intentions, and constitutive rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveIntentionality {
    /// All collective groups
    pub groups: Vec<CollectiveGroup>,
    /// We-intentions by group ID
    pub we_intentions: HashMap<GroupId, WeIntention>,
    /// Constitutive rules for institutional fact creation
    pub constitutive_rules: Vec<ConstitutiveRule>,
    /// Next available group ID
    next_group_id: GroupId,
    /// Current tick (for timestamping)
    current_tick: Tick,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl CollectiveIntentionality {
    /// Create a new collective intentionality system
    #[must_use]
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            we_intentions: HashMap::new(),
            constitutive_rules: Self::default_rules(),
            next_group_id: 0 as GroupId,
            current_tick: 0,
        }
    }

    /// Create with a specific starting tick
    #[must_use]
    pub fn with_tick(start_tick: Tick) -> Self {
        Self {
            groups: Vec::new(),
            we_intentions: HashMap::new(),
            constitutive_rules: Self::default_rules(),
            next_group_id: 0 as GroupId,
            current_tick: start_tick,
        }
    }

    /// Update the current tick
    pub fn set_tick(&mut self, tick: Tick) {
        self.current_tick = tick;
    }

    /// Get the current tick
    #[must_use]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub const fn current_tick(&self) -> Tick {
        self.current_tick
    }

    /// Form a new collective group with a shared goal
    /// Creates a we-intention for the group
    pub fn form_group(&mut self, members: Vec<AgentId>, goal: &Goal) -> GroupId {
        let id = self.next_group_id;
        self.next_group_id += 1;

        // Create the collective group with belief status based on goal requirements
        let mut group = CollectiveGroup::new(id, members.clone(), goal.clone(), self.current_tick);

        // Create the corresponding we-intention
        let mut we_intention =
            WeIntention::new(goal.to_action_string(), members, self.current_tick);

        // Establish mutual belief for goals that require it
        if goal.requires_mutual_belief() {
            we_intention.establish_mutual_belief();
            group.mutual_belief_established = true;
        }

        // Establish common knowledge for goals that require it
        if goal.requires_common_knowledge() {
            we_intention.establish_common_knowledge();
            group.common_knowledge_established = true;
        }

        self.groups.push(group);
        self.we_intentions.insert(id, we_intention);

        id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Dissolve a collective group
    pub fn dissolve_group(&mut self, group_id: GroupId) -> bool {
        let initial_len = self.groups.len();
        self.groups.retain(|g| g.id != group_id);
        self.we_intentions.remove(&group_id);
        self.groups.len() < initial_len
    }

    /// Get a group by ID
    #[must_use]
    pub fn get_group(&self, group_id: GroupId) -> Option<&CollectiveGroup> {
        self.groups.iter().find(|g| g.id == group_id)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a group by ID (mutable)
    pub fn get_group_mut(&mut self, group_id: GroupId) -> Option<&mut CollectiveGroup> {
        self.groups.iter_mut().find(|g| g.id == group_id)
    }

    /// Get the we-intention for a group
    #[must_use]
    pub fn get_we_intention(&self, group_id: GroupId) -> Option<&WeIntention> {
        self.we_intentions.get(&group_id)
    }

    /// Get the we-intention for a group (mutable)
    pub fn get_we_intention_mut(&mut self, group_id: GroupId) -> Option<&mut WeIntention> {
        self.we_intentions.get_mut(&group_id)
    }

    /// Apply constitutive rules to brute facts
    /// Per Searle 1995: Creates institutional facts from brute facts
    /// Returns map of institutional facts with their strengths
    #[must_use]
    pub fn apply_constitutive_rules(
        &self,
        brute_facts: &HashMap<String, f32>,
    ) -> HashMap<String, f32> {
        let mut institutional = HashMap::new();

        for rule in &self.constitutive_rules {
            if let Some(&strength) = brute_facts.get(&rule.brute_fact) {
                if let Some(institutional_strength) = rule.apply(strength) {
                    institutional.insert(rule.institutional_fact.clone(), institutional_strength);
                }
            }
        }

        institutional
    }

    /// Add a custom constitutive rule
    pub fn add_constitutive_rule(&mut self, rule: ConstitutiveRule) {
        self.constitutive_rules.push(rule);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get all groups that an agent belongs to
    #[must_use]
    pub fn get_agent_groups(&self, agent_id: AgentId) -> Vec<&CollectiveGroup> {
        self.groups
            .iter()
            .filter(|g| g.has_member(agent_id))
            .collect()
    }

    /// Get all we-intentions that an agent participates in
    #[must_use]
    pub fn get_agent_intentions(&self, agent_id: AgentId) -> Vec<&WeIntention> {
        self.we_intentions
            .values()
            .filter(|wi| wi.has_participant(agent_id))
            .collect()
    }

    /// Check if an agent has collective intentionality (participates in valid we-intentions)
    #[must_use]
    pub fn has_collective_intentionality(&self, agent_id: AgentId) -> bool {
        self.get_agent_intentions(agent_id)
            .iter()
            .any(|wi| wi.satisfies_searle_conditions())
    }

    /// Update group commitment levels based on participation
    pub fn update_group_commitments(&mut self, participation_rates: &HashMap<GroupId, f32>) {
        for (group_id, rate) in participation_rates {
            if let Some(group) = self.get_group_mut(*group_id) {
                group.update_commitment(*rate);
            }
            if let Some(we_intention) = self.get_we_intention_mut(*group_id) {
                we_intention.update_strength(*rate);
            }
        }
    }

    /// Establish mutual belief for a group
    pub fn establish_mutual_belief(&mut self, group_id: GroupId) -> bool {
        if let Some(group) = self.get_group_mut(group_id) {
            group.mutual_belief_established = true;
        }
        if let Some(we_intention) = self.get_we_intention_mut(group_id) {
            we_intention.establish_mutual_belief();
            return true;
        }
        false
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Establish common knowledge for a group
    pub fn establish_common_knowledge(&mut self, group_id: GroupId) -> bool {
        if let Some(group) = self.get_group_mut(group_id) {
            group.common_knowledge_established = true;
            group.mutual_belief_established = true;
        }
        if let Some(we_intention) = self.get_we_intention_mut(group_id) {
            we_intention.establish_common_knowledge();
            return true;
        }
        false
    }

    /// Get statistics about the collective intentionality system
    #[must_use]
    pub fn stats(&self) -> CollectiveIntentionalityStats {
        let total_groups = self.groups.len();
        let groups_with_mutual_belief = self
            .groups
            .iter()
            .filter(|g| g.mutual_belief_established)
            .count();
        let groups_with_common_knowledge = self
            .groups
            .iter()
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .filter(|g| g.common_knowledge_established)
            .count();

        let valid_we_intentions = self
            .we_intentions
            .values()
            .filter(|wi| wi.satisfies_searle_conditions())
            .count();

        CollectiveIntentionalityStats {
            total_groups,
            groups_with_mutual_belief,
            groups_with_common_knowledge,
            total_we_intentions: self.we_intentions.len(),
            valid_we_intentions,
            total_constitutive_rules: self.constitutive_rules.len(),
            average_commitment: if total_groups > 0 {
                self.groups.iter().map(|g| g.commitment_level).sum::<f32>() / total_groups as f32
            } else {
                0.0
            },
        }
    }

    /// Default constitutive rules for the simulation
    fn default_rules() -> Vec<ConstitutiveRule> {
        vec![
            ConstitutiveRule {
                brute_fact: "mutual_trust_above_threshold".to_string(),
                institutional_fact: "social_contract_exists".to_string(),
                // TAG: surface=simulation owner=research-team rule=SIM-001
                context: "group_formation".to_string(),
                threshold: 0.5,
                confidence_factor: 0.9,
            },
            ConstitutiveRule {
                brute_fact: "repeated_exchange".to_string(),
                institutional_fact: "economic_relationship".to_string(),
                context: "market".to_string(),
                threshold: 0.5,
                confidence_factor: 0.85,
            },
            ConstitutiveRule {
                brute_fact: "collective_commitment".to_string(),
                institutional_fact: "group_identity".to_string(),
                context: "identity".to_string(),
                threshold: 0.6,
                confidence_factor: 0.95,
            },
            ConstitutiveRule {
                brute_fact: "shared_language_use".to_string(),
                institutional_fact: "communication_community".to_string(),
                context: "language".to_string(),
                threshold: 0.4,
                confidence_factor: 0.8,
            },
        ]
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for CollectiveIntentionality {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for the collective intentionality system
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CollectiveIntentionalityStats {
    /// Total number of collective groups
    pub total_groups: usize,
    /// Groups that have established mutual belief
    pub groups_with_mutual_belief: usize,
    /// Groups that have established common knowledge
    pub groups_with_common_knowledge: usize,
    /// Total number of we-intentions
    pub total_we_intentions: usize,
    /// We-intentions that satisfy Searle's conditions
    pub valid_we_intentions: usize,
    /// Total constitutive rules in effect
    pub total_constitutive_rules: usize,
    /// Average commitment level across groups
    pub average_commitment: f32,
}

#[cfg(test)]
mod tests {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    use super::*;

    // === Test 1: Basic CollectiveGroup Creation ===
    #[test]
    fn test_collective_group_creation() {
        let goal = Goal::ResourceAcquisition {
            resource_type: "food".to_string(),
            amount: 100.0,
        };
        let group = CollectiveGroup::new(1, vec![1, 2, 3], goal, 0);

        assert_eq!(group.id, 1);
        assert_eq!(group.members, vec![1, 2, 3]);
        assert_eq!(group.size(), 3);
        assert!(group.has_member(1));
        assert!(group.has_member(2));
        assert!(!group.has_member(4));
        assert!(group.mutual_belief_established); // Not required for resource acquisition
    }

    // === Test 2: Goal Action String ===
    #[test]
    fn test_goal_to_action_string() {
        let goal1 = Goal::Defense {
            target: "territory".to_string(),
        };
        assert_eq!(goal1.to_action_string(), "Defend territory");

        let goal2 = Goal::Coordination {
            activity: "hunt".to_string(),
        };
        assert_eq!(goal2.to_action_string(), "hunt");

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let goal3 = Goal::Custom {
            description: "custom action".to_string(),
        };
        assert_eq!(goal3.to_action_string(), "custom action");
    }

    // === Test 3: WeIntention Creation and Validation ===
    #[test]
    fn test_we_intention_creation() {
        let wi = WeIntention::new("defend village".to_string(), vec![1, 2, 3], 0);

        assert_eq!(wi.action, "defend village");
        assert_eq!(wi.participants, vec![1, 2, 3]);
        assert!(!wi.mutual_belief);
        assert!(!wi.common_knowledge);
        assert_eq!(wi.strength, 0.5);
        assert!(wi.is_valid());
        assert!(!wi.satisfies_searle_conditions()); // No mutual belief yet

        // Establish mutual belief
        let mut wi2 = wi;
        wi2.establish_mutual_belief();
        assert!(wi2.mutual_belief);
        assert!(!wi2.common_knowledge);
        assert!(wi2.satisfies_searle_conditions());

        // Establish common knowledge
        let mut wi3 = wi2.clone();
        wi3.establish_common_knowledge();
        assert!(wi3.mutual_belief);
        assert!(wi3.common_knowledge);
    }

    // === Test 4: ConstitutiveRule Application ===
    #[test]
    fn test_constitutive_rule() {
        let rule = ConstitutiveRule::new("high_trust", "social_contract", "community");

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(rule.brute_fact, "high_trust");
        assert_eq!(rule.threshold, 0.5);

        // Below threshold - doesn't apply
        assert!(!rule.applies(0.3));
        assert!(rule.apply(0.3).is_none());

        // Above threshold - applies
        assert!(rule.applies(0.7));
        assert_eq!(rule.apply(0.7), Some(0.7 * 0.9));
    }

    // === Test 5: CollectiveIntentionality System - Form Group ===
    #[test]
    fn test_form_group() {
        let mut ci = CollectiveIntentionality::new();
        let goal = Goal::Defense {
            target: "village".to_string(),
        };

        let group_id = ci.form_group(vec![1, 2, 3], &goal);
        assert_eq!(group_id, 0);

        let group = ci.get_group(group_id).unwrap();
        assert_eq!(group.size(), 3);
        assert!(group.mutual_belief_established); // Defense requires mutual belief

        let we_intention = ci.get_we_intention(group_id).unwrap();
        assert!(we_intention.mutual_belief);
        assert_eq!(we_intention.action, "Defend village");
    }

    // === Test 6: Apply Constitutive Rules ===
    #[test]
    fn test_apply_constitutive_rules() {
        let ci = CollectiveIntentionality::new();
        let mut brute_facts = HashMap::new();
        brute_facts.insert("mutual_trust_above_threshold".to_string(), 0.8);
        brute_facts.insert("repeated_exchange".to_string(), 0.3); // Below threshold

        let institutional = ci.apply_constitutive_rules(&brute_facts);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // First rule should apply (0.8 > 0.5)
        assert!(institutional.contains_key("social_contract_exists"));
        assert_eq!(
            institutional.get("social_contract_exists"),
            Some(&(0.8 * 0.9))
        );

        // Second rule should not apply (0.3 < 0.5)
        assert!(!institutional.contains_key("economic_relationship"));
    }

    // === Test 7: Agent Group Membership ===
    #[test]
    fn test_agent_group_membership() {
        let mut ci = CollectiveIntentionality::new();
        let goal1 = Goal::Defense {
            target: "area1".to_string(),
        };
        let goal2 = Goal::Exploration {
            area: "area2".to_string(),
        };

        ci.form_group(vec![1, 2], &goal1);
        ci.form_group(vec![2, 3], &goal2);

        let agent1_groups = ci.get_agent_groups(1);
        assert_eq!(agent1_groups.len(), 1);

        let agent2_groups = ci.get_agent_groups(2);
        assert_eq!(agent2_groups.len(), 2);

        let agent3_groups = ci.get_agent_groups(3);
        assert_eq!(agent3_groups.len(), 1);

        let agent4_groups = ci.get_agent_groups(4);
        assert_eq!(agent4_groups.len(), 0);
    }

    // === Test 8: Update Commitment ===
    #[test]
    fn test_update_commitment() {
        let mut ci = CollectiveIntentionality::new();
        let goal = Goal::Custom {
            description: "test".to_string(),
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let group_id = ci.form_group(vec![1, 2], &goal);

        let group = ci.get_group(group_id).unwrap();
        assert_eq!(group.commitment_level, 0.5);

        let mut participation_rates = HashMap::new();
        participation_rates.insert(group_id, 0.8);
        ci.update_group_commitments(&participation_rates);

        let group = ci.get_group(group_id).unwrap();
        assert!(group.commitment_level > 0.5);
    }

    // === Test 9: Statistics ===
    #[test]
    fn test_statistics() {
        let mut ci = CollectiveIntentionality::new();
        let goal = Goal::Defense {
            target: "test".to_string(),
        };

        let group_id = ci.form_group(vec![1, 2, 3], &goal);
        ci.establish_common_knowledge(group_id);

        let stats = ci.stats();
        assert_eq!(stats.total_groups, 1);
        assert_eq!(stats.groups_with_mutual_belief, 1);
        assert_eq!(stats.groups_with_common_knowledge, 1);
        assert_eq!(stats.total_we_intentions, 1);
        assert!(stats.valid_we_intentions >= 1);
    }

    // === Test 10: Collective Intentionality Check ===
    #[test]
    fn test_collective_intentionality_check() {
        let mut ci = CollectiveIntentionality::new();
        let goal = Goal::Defense {
            target: "village".to_string(),
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let group_id = ci.form_group(vec![1, 2], &goal);

        // Agent 1 should have collective intentionality
        assert!(ci.has_collective_intentionality(1));

        // Agent 3 should not
        assert!(!ci.has_collective_intentionality(3));

        // Dissolve group
        ci.dissolve_group(group_id);

        // Now agent 1 should not have collective intentionality
        assert!(!ci.has_collective_intentionality(1));
    }

    // === Test 11: Goal Requirements ===
    #[test]
    fn test_goal_requirements() {
        let defense = Goal::Defense {
            target: "test".to_string(),
        };
        let coord = Goal::Coordination {
            activity: "test".to_string(),
        };
        let resource = Goal::ResourceAcquisition {
            resource_type: "food".to_string(),
            amount: 10.0,
        };

        assert!(defense.requires_mutual_belief());
        assert!(defense.requires_common_knowledge());

        assert!(coord.requires_mutual_belief());
        assert!(coord.requires_common_knowledge());

        assert!(!resource.requires_mutual_belief());
        assert!(!resource.requires_common_knowledge());
    }
}
