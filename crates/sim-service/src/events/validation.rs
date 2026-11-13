//! Event Validation
//!
//! Validates external events before injection to ensure they meet
//! simulation constraints and don't cause errors.

use std::collections::HashSet;

use super::types::{
    AgentAction, CommunicationEvent, EnvironmentChangeType, ExternalEvent, MessageType,
    SystemCommand, UrgencyLevel,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Result of event validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Normalized/validated event
    pub normalized_event: Option<ExternalEvent>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ValidationResult {
    /// Create a successful result
    #[must_use]
    pub const fn success(event: ExternalEvent) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            normalized_event: Some(event),
        }
    }

    /// Create a failed result
    #[must_use]
    pub const fn failure(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
            normalized_event: None,
        }
    }

    /// Add a warning
    #[must_use]
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Check if there are any blocking errors
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Event validator with configurable rules
#[derive(Debug)]
pub struct EventValidator {
    /// Maximum agents per agent event
    max_agents_per_event: usize,
    /// Maximum message length
    max_message_length: usize,
    /// Allowed agent actions
    allowed_agent_actions: HashSet<AgentAction>,
    /// Allowed environment changes
    allowed_env_changes: HashSet<EnvironmentChangeType>,
    /// Allowed system commands
    allowed_system_commands: HashSet<SystemCommand>,
    /// Maximum resource change amount
    max_resource_change: f64,
}

impl EventValidator {
    /// Create a new validator with default rules
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_agents_per_event: 1000,
            max_message_length: 10000,
            allowed_agent_actions: HashSet::from([
                AgentAction::SetState,
                AgentAction::TriggerBehavior,
                AgentAction::SetProperty,
                AgentAction::Teleport,
                AgentAction::ModifyTrait,
                AgentAction::InfluenceEmotion,
            ]),
            allowed_env_changes: HashSet::from([
                EnvironmentChangeType::WeatherChange,
                EnvironmentChangeType::TimeChange,
                EnvironmentChangeType::SocialEvent,
                EnvironmentChangeType::PolicyChange,
            ]),
            allowed_system_commands: HashSet::from([
                SystemCommand::Pause,
                SystemCommand::Resume,
                SystemCommand::SpeedUp,
                SystemCommand::SlowDown,
                SystemCommand::Checkpoint,
            ]),
            max_resource_change: 1000000.0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a permissive validator (allows more event types)
    #[must_use]
    pub fn permissive() -> Self {
        let mut validator = Self::new();
        validator.allowed_env_changes.extend([
            EnvironmentChangeType::Disaster,
            EnvironmentChangeType::EconomicShift,
        ]);
        validator
            .allowed_system_commands
            .extend([SystemCommand::LoadScenario, SystemCommand::InjectScenario]);
        validator.max_agents_per_event = 10000;
        validator.max_message_length = 100000;
        validator.max_resource_change = 10000000.0;
        validator
    }

    /// Validate an external event
    #[must_use]
    pub fn validate(&self, event: &ExternalEvent) -> ValidationResult {
        match event {
            ExternalEvent::AgentEvent(agent_event) => self.validate_agent_event(agent_event),
            ExternalEvent::EnvironmentEvent(env_event) => {
                self.validate_environment_event(env_event)
            }
            ExternalEvent::NarrativeEvent(narr_event) => self.validate_narrative_event(narr_event),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            ExternalEvent::CommunicationEvent(comm_event) => {
                self.validate_communication_event(comm_event)
            }
            ExternalEvent::ResourceEvent(res_event) => self.validate_resource_event(res_event),
            ExternalEvent::SystemEvent(sys_event) => self.validate_system_event(sys_event),
        }
    }

    /// Validate agent event
    fn validate_agent_event(&self, event: &super::types::AgentEvent) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if event.target_agents.len() > self.max_agents_per_event {
            errors.push(format!(
                "Too many target agents: {} (max: {})",
                event.target_agents.len(),
                self.max_agents_per_event
            ));
        }
        if !self.allowed_agent_actions.contains(&event.action) {
            errors.push(format!("Agent action not allowed: {:?}", event.action));
        }
        Self::check_agent_action_params(event, &mut errors, &mut warnings);
        Self::result_or_failure(ExternalEvent::AgentEvent(event.clone()), errors, &warnings)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn check_agent_action_params(
        event: &super::types::AgentEvent,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        match event.action {
            AgentAction::Teleport => {
                if !event.parameters.contains_key("x") || !event.parameters.contains_key("y") {
                    errors.push("Teleport action requires 'x' and 'y' coordinates".to_string());
                }
            }
            AgentAction::SetProperty => {
                if !event.parameters.contains_key("property")
                    || !event.parameters.contains_key("value")
                {
                    errors.push(
                        "SetProperty action requires 'property' and 'value' parameters".to_string(),
                    );
                }
            }
            AgentAction::InfluenceEmotion => {
                if let Some(val) = event.parameters.get("valence") {
                    if let Some(val_num) = val.as_f64() {
                        if !(-1.0..=1.0).contains(&val_num) {
                            errors
                                .push("Emotional valence must be between -1.0 and 1.0".to_string());
                        }
                    }
                } else {
                    warnings.push("No valence specified for emotion influence".to_string());
                }
            }
            _ => {}
        }
    }

    /// Validate environment event
    fn validate_environment_event(
        &self,
        event: &super::types::EnvironmentEvent,
    ) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        if !self.allowed_env_changes.contains(&event.change_type) {
            errors.push(format!(
                "Environment change type not allowed: {:?}",
                event.change_type
            ));
        }
        Self::check_disaster_params(event, &mut warnings);
        Self::check_time_change(event, &mut errors);
        Self::result_or_failure(
            ExternalEvent::EnvironmentEvent(event.clone()),
            errors,
            &warnings,
        )
    }

    fn check_disaster_params(event: &super::types::EnvironmentEvent, warnings: &mut Vec<String>) {
        if event.change_type != EnvironmentChangeType::Disaster {
            return;
        }
        if !event.parameters.contains_key("severity") {
            warnings.push("Disaster event should specify severity".to_string());
        }
        if event.parameters.contains_key("affected_agents") && event.regions.is_empty() {
            warnings.push("Disaster affects specific agents but no region specified".to_string());
        }
    }

    fn check_time_change(event: &super::types::EnvironmentEvent, errors: &mut Vec<String>) {
        if event.change_type != EnvironmentChangeType::TimeChange {
            return;
        }
        if let Some(time_val) = event.parameters.get("time_of_day") {
            if let Some(hour) = time_val.as_f64() {
                if !(0.0..=24.0).contains(&hour) {
                    errors.push("Time of day must be between 0 and 24".to_string());
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Validate narrative event
    fn validate_narrative_event(&self, event: &super::types::NarrativeEvent) -> ValidationResult {
        let mut errors = Vec::new();
        if event.beat_id.is_empty() {
            errors.push("Narrative beat ID cannot be empty".to_string());
        }
        if event.story_context.is_empty() {
            errors.push("Story context cannot be empty".to_string());
        }
        Self::result_or_failure(ExternalEvent::NarrativeEvent(event.clone()), errors, &[])
    }

    /// Validate communication event
    fn validate_communication_event(&self, event: &CommunicationEvent) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if event.sender.is_empty() {
            errors.push("Communication event requires a sender".to_string());
        }
        if event.recipients.is_empty() {
            errors.push("Communication event requires at least one recipient".to_string());
        }
        if event.content.len() > self.max_message_length {
            errors.push(format!(
                "Message too long: {} characters (max: {})",
                // TAG: surface=simulation owner=research-team rule=SIM-001
                event.content.len(),
                self.max_message_length
            ));
        }
        if event.content.is_empty() {
            errors.push("Message content cannot be empty".to_string());
        }
        if event.urgency == UrgencyLevel::Critical && event.message_type != MessageType::Warning {
            warnings.push("Critical urgency used with non-warning message type".to_string());
        }
        if event.message_type == MessageType::Command && event.recipients.len() > 10 {
            warnings.push(
                "Command sent to many recipients; consider using SystemEvent instead".to_string(),
            );
        }

        Self::result_or_failure(
            ExternalEvent::CommunicationEvent(event.clone()),
            errors,
            &warnings,
        )
    }

    /// Validate resource event
    fn validate_resource_event(&self, event: &super::types::ResourceEvent) -> ValidationResult {
        let mut errors = Vec::new();
        if event.resource_type.is_empty() {
            errors.push("Resource type cannot be empty".to_string());
        }
        if event.amount.abs() > self.max_resource_change {
            errors.push(format!(
                "Resource change too large: {} (max: {})",
                event.amount, self.max_resource_change
            ));
        }
        if event.amount == 0.0 {
            errors.push("Resource change amount cannot be zero".to_string());
        }
        if event.targets.is_empty() {
            errors.push("Resource event requires at least one target".to_string());
        }
        Self::result_or_failure(ExternalEvent::ResourceEvent(event.clone()), errors, &[])
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Validate system event
    fn validate_system_event(&self, event: &super::types::SystemEvent) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if !self.allowed_system_commands.contains(&event.command) {
            errors.push(format!("System command not allowed: {:?}", event.command));
        }
        Self::check_system_command(event, &mut errors, &mut warnings);
        Self::result_or_failure(ExternalEvent::SystemEvent(event.clone()), errors, &warnings)
    }

    fn check_system_command(
        event: &super::types::SystemEvent,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        match event.command {
            SystemCommand::LoadScenario => {
                if !event.parameters.contains_key("scenario_id") {
                    errors.push("LoadScenario requires 'scenario_id' parameter".to_string());
                }
            }
            SystemCommand::InjectScenario => {
                if !event.parameters.contains_key("scenario_id") {
                    errors.push("InjectScenario requires 'scenario_id' parameter".to_string());
                }
                warnings.push("Scenario injection may disrupt ongoing simulation".to_string());
            }
            SystemCommand::SpeedUp | SystemCommand::SlowDown => {
                // TAG: surface=simulation owner=research-team rule=SIM-001
                if let Some(factor) = event.parameters.get("factor") {
                    if let Some(f) = factor.as_f64() {
                        if !(0.1..=10.0).contains(&f) {
                            errors.push("Speed factor must be between 0.1 and 10.0".to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Build result from errors and warnings
    fn result_or_failure(
        event: ExternalEvent,
        errors: Vec<String>,
        warnings: &[String],
    ) -> ValidationResult {
        if errors.is_empty() {
            ValidationResult::success(event).with_warning(warnings.join("; "))
        } else {
            ValidationResult::failure(errors)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update validation rules
    pub fn allow_agent_action(&mut self, action: AgentAction) {
        self.allowed_agent_actions.insert(action);
    }

    /// Disallow an agent action
    pub fn disallow_agent_action(&mut self, action: &AgentAction) {
        self.allowed_agent_actions.remove(action);
    }

    /// Allow an environment change
    pub fn allow_env_change(&mut self, change: EnvironmentChangeType) {
        self.allowed_env_changes.insert(change);
    }

    /// Set max agents per event
    pub fn set_max_agents(&mut self, max: usize) {
        self.max_agents_per_event = max;
    }

    /// Set max message length
    pub fn set_max_message_length(&mut self, max: usize) {
        self.max_message_length = max;
    }
}

impl Default for EventValidator {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::types::{
        AgentAction, AgentEvent, CommunicationEvent, EnvironmentChangeType, EnvironmentEvent,
        MessageType, UrgencyLevel,
    };
    use std::collections::HashMap;

    #[test]
    fn test_validate_valid_agent_event() {
        let validator = EventValidator::new();

        let event = ExternalEvent::AgentEvent(AgentEvent {
            target_agents: vec!["agent-1".to_string()],
            action: AgentAction::SetProperty,
            parameters: {
                let mut p = HashMap::new();
                p.insert("property".to_string(), serde_json::json!("energy"));
                p.insert("value".to_string(), serde_json::json!(100));
                p
            },
        });

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let result = validator.validate(&event);
        assert!(result.valid);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_validate_invalid_teleport() {
        let validator = EventValidator::new();

        let event = ExternalEvent::AgentEvent(AgentEvent {
            target_agents: vec!["agent-1".to_string()],
            action: AgentAction::Teleport,
            parameters: HashMap::new(), // Missing x, y
        });

        let result = validator.validate(&event);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains('x') && e.contains('y')));
    }

    #[test]
    fn test_validate_too_many_agents() {
        let validator = EventValidator::new();

        let event = ExternalEvent::AgentEvent(AgentEvent {
            target_agents: (0..2000).map(|i| format!("agent-{i}")).collect(),
            action: AgentAction::SetState,
            parameters: HashMap::new(),
        });

        let result = validator.validate(&event);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Too many")));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_validate_communication_empty_content() {
        let validator = EventValidator::new();

        let event = ExternalEvent::CommunicationEvent(CommunicationEvent {
            sender: "agent-1".to_string(),
            recipients: vec!["agent-2".to_string()],
            message_type: MessageType::Conversation,
            content: String::new(),
            urgency: UrgencyLevel::Normal,
            metadata: HashMap::new(),
        });

        let result = validator.validate(&event);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("empty")));
    }

    #[test]
    fn test_permissive_validator() {
        let validator = EventValidator::permissive();

        // Should allow disasters
        let event = ExternalEvent::EnvironmentEvent(EnvironmentEvent {
            change_type: EnvironmentChangeType::Disaster,
            regions: vec!["region-1".to_string()],
            parameters: HashMap::new(),
        });

        let result = validator.validate(&event);
        assert!(result.valid);
    }
}
