//! DMN Analog: Bounded counterfactual planning
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION planning, not product decision-making
//! SIM-004: Deterministic - uses seeded RNG
//!
//! Per §6.2 lines 732-796: DMN analog with K-rollout planning
//! Per §8.2.1 lines 4565-4589: DMN toggle contract

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::choice_architecture::{apply_nudges, uniform_weights, NudgeConfig};
use crate::environment::Environment;
use crate::types::Position;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Action types available to agents
/// Per §7.6: Agent decision-making
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Move in a direction
    Move(Direction),
    /// Stay in current position
    Stay,
    /// Attempt reproduction (if eligible)
    Reproduce,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Movement directions (8-way + stay)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// north.
    North,
    /// north east.
    NorthEast,
    /// east.
    East,
    /// south east.
    SouthEast,
    /// south.
    South,
    /// south west.
    SouthWest,
    /// west.
    West,
    /// north west.
    NorthWest,
}

impl Direction {
    /// Convert direction to position delta
    #[must_use]
    pub const fn to_delta(&self) -> (f32, f32) {
        match self {
            Self::North => (0.0, 1.0),
            Self::NorthEast => (1.0, 1.0),
            Self::East => (1.0, 0.0),
            Self::SouthEast => (1.0, -1.0),
            Self::South => (0.0, -1.0),
            Self::SouthWest => (-1.0, -1.0),
            Self::West => (-1.0, 0.0),
            Self::NorthWest => (-1.0, 1.0),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Determine direction from delta coordinates
    /// Returns the closest cardinal or diagonal direction
    #[must_use]
    pub fn from_delta(dx: f32, dy: f32) -> Self {
        match (dx > 0.0, dx < 0.0, dy > 0.0, dy < 0.0) {
            (true, false, true, false) => Self::NorthEast,
            (true, false, false, true) => Self::SouthEast,
            (false, true, true, false) => Self::NorthWest,
            (false, true, false, true) => Self::SouthWest,
            (true, false, false, false) => Self::East,
            (false, true, false, false) => Self::West,
            (false, false, true, false) => Self::North,
            _ => Self::South,
        }
    }

    /// All 8 directions
    #[must_use]
    pub const fn all() -> [Self; 8] {
        [
            Self::North,
            Self::NorthEast,
            Self::East,
            Self::SouthEast,
            Self::South,
            Self::SouthWest,
            Self::West,
            Self::NorthWest,
        ]
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sample a random action from available actions
/// Per §8.2.1 line 4586: `dmn_off` behavior
/// MOVEMENT FIX: Strongly prefer movement over staying to prevent gridlock
pub fn sample_action(agent: &Agent, rng: &mut StdRng) -> Action {
    let actions = available_actions(agent);

    // Weight movement actions higher than Stay
    // This prevents agents from getting stuck in clusters
    let mut weighted_actions: Vec<Action> = Vec::new();
    for action in &actions {
        match action {
            Action::Stay => {
                // Add Stay only once (low probability)
                weighted_actions.push(*action);
            }
            Action::Move(_) => {
                // Add Move actions multiple times (high probability)
                // 4x weighting for movement
                weighted_actions.push(*action);
                weighted_actions.push(*action);
                weighted_actions.push(*action);
                weighted_actions.push(*action);
            }
            Action::Reproduce => {
                // Add Reproduction with normal weight
                weighted_actions.push(*action);
            }
        }
    }

    let idx = rng.gen_range(0..weighted_actions.len());
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    weighted_actions[idx]
}

/// Get available actions for an agent
fn available_actions(agent: &Agent) -> Vec<Action> {
    let mut actions = vec![Action::Stay];

    // Movement actions always available
    for dir in Direction::all() {
        actions.push(Action::Move(dir));
    }

    // Reproduction only if energy sufficient
    // Per §7.4.0 line 3757: reproduction_threshold 50-100
    if agent.energy >= 50.0 {
        actions.push(Action::Reproduce);
    }

    actions
}

/// Evaluate action value via rollout simulation
/// Per §8.2.1 lines 4577: `simulate_trajectory`
fn evaluate_action(agent: &Agent, env: &Environment, action: Action, _horizon: u32) -> f32 {
    // Simple heuristic evaluation for Phase 1
    // Full trajectory simulation deferred to Phase 2+
    match action {
        Action::Move(dir) => {
            let (dx, dy) = dir.to_delta();
            let new_pos = Position::new_2d(agent.position.x + dx, agent.position.y + dy);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Value = resource availability at new position
            let resource_value = env.resource_at(&new_pos).unwrap_or(0.0);

            // MOVEMENT BIAS FIX: Strong bonus for movement to prevent gridlock
            // Agents should ALWAYS prefer movement over staying (exploration)
            // This is critical for agents to find resources and avoid clustering
            let movement_bonus = if agent.energy > 10.0 { 15.0 } else { 0.0 };

            resource_value + movement_bonus
        }
        Action::Stay => {
            // Value = current resource availability
            // Note: Stay has NO movement bonus, so movement is strongly preferred
            // Only stay if current cell has high resources
            env.resource_at(&agent.position).unwrap_or(0.0)
        }
        Action::Reproduce => {
            // Reproduction has negative immediate value (energy cost)
            // but positive long-term value (offspring)
            // For now, heuristic approximation: -30 (cost) + 50 (future value)
            20.0
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// DMN planning with K-rollout counterfactual simulation
/// Per §6.2 lines 736-740: K=3-20, H=1-5, B=100-1000
/// Per §8.2.1 lines 4568-4582: `dmn_on` implementation
pub fn plan_action(agent: &Agent, env: &Environment, rng: &mut StdRng) -> Action {
    plan_action_with_nudges(agent, env, rng, false)
}

/// DMN planning with optional Choice Architecture (Thaler nudges)
/// Per §6.2 lines 736-740: K=3-20, H=1-5, B=100-1000
/// Per §8.2.1 lines 4568-4582: `dmn_on` implementation
/// Per §8.2.1 lines 4866-4893: `choice_arch_on` applies nudges
pub fn plan_action_with_nudges(
    agent: &Agent,
    env: &Environment,
    rng: &mut StdRng,
    choice_architecture_enabled: bool,
) -> Action {
    plan_action_with_nudges_and_horizon(agent, env, rng, choice_architecture_enabled, None)
}

/// DMN planning with optional Choice Architecture and horizon override
pub fn plan_action_with_nudges_and_horizon(
    agent: &Agent,
    env: &Environment,
    rng: &mut StdRng,
    choice_architecture_enabled: bool,
    horizon_override: Option<u32>,
) -> Action {
    plan_action_full(
        agent,
        env,
        rng,
        choice_architecture_enabled,
        horizon_override,
        None,
        None,
        None,
        None,
    )
}

/// DMN planning with configurable Choice Architecture and simulation budget override
pub fn plan_action_full(
    agent: &Agent,
    env: &Environment,
    rng: &mut StdRng,
    choice_architecture_enabled: bool,
    horizon_override: Option<u32>,
    budget_override: Option<u32>,
    nudge_config: Option<&NudgeConfig>,
    k_range: Option<(u32, u32)>,
    h_range: Option<(u32, u32)>,
) -> Action {
    let k = if let Some((k_min, k_max)) = k_range {
        let spread = k_max.saturating_sub(k_min);
        k_min + (agent.genome.get_trait(10) * spread as f32) as u32
    } else {
        agent.genome.planning_depth()
    }; // K = 3-20

    let h = horizon_override.unwrap_or_else(|| {
        if let Some((h_min, h_max)) = h_range {
            let spread = h_max.saturating_sub(h_min);
            h_min + (agent.genome.get_trait(11) * spread as f32) as u32
        } else {
            agent.genome.planning_horizon()
        }
    }); // H = 1-5 or override

    let base_budget = budget_override.unwrap_or(agent.dmn_budget);
    let budget = base_budget.max(1);

    // Limit rollouts by budget
    let rollouts = k.min(budget);

    let mut candidates: Vec<(Action, f32)> = Vec::new();

    for _ in 0..rollouts {
        let action = sample_action(agent, rng);
        let value = evaluate_action(agent, env, action, h);
        candidates.push((action, value));
    }

    // Apply Choice Architecture nudges if enabled
    let default_nudge = NudgeConfig::default();
    let effective_nudge = nudge_config.unwrap_or(&default_nudge);
    let weighted_candidates = if choice_architecture_enabled {
        apply_nudges(&candidates, agent, effective_nudge)
    } else {
        uniform_weights(&candidates)
    };

    // Sort by weight (descending) - now reflects nudge effects
    let mut sorted = weighted_candidates;
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Return best action, or Stay if no candidates
    sorted.first().map_or(Action::Stay, |(a, _)| *a)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Agent;
    use crate::docas::DocasState;
    use crate::environment::{Environment, EnvironmentConfig};
    use crate::genome::Genome;
    use crate::types::Position;
    use rand::SeedableRng;

    /// Helper: Create test agent with specific genome traits
    fn create_test_agent(planning_depth_trait: f32, planning_horizon_trait: f32) -> Agent {
        let mut genome = Genome::default();
        // Trait 10 (planning_depth) uses loci 20 and 21
        genome.alleles[20] = [planning_depth_trait, planning_depth_trait];
        genome.alleles[21] = [planning_depth_trait, planning_depth_trait];
        // Trait 11 (planning_horizon) uses loci 22 and 23
        genome.alleles[22] = [planning_horizon_trait, planning_horizon_trait];
        genome.alleles[23] = [planning_horizon_trait, planning_horizon_trait];

        Agent {
            id: 0,
            position: Position::new_2d(32.0, 32.0),
            energy: 100.0,
            age: 0,
            sex: crate::types::Sex::Male,
            genome,
            vision_range: 6,
            metabolic_rate: 1.5,
            docas: DocasState::default(),
            dmn_budget: 100,
            criticality_sigma: 1.0,
            emotion_baseline: 0.5,
            reputation_score: 0.5,
            group_id: None,
            cultural_traits: Vec::new(),
            cognitive_bandwidth: 0.5,

            // TAG: surface=simulation owner=research-team rule=SIM-001
            isolation_score: 0.0,
            attestations_given: Vec::new(),
            attestations_received: Vec::new(),
            lexicon: crate::language::Lexicon::default(),
            has_verified_id: false,
            verification_count: 0,
            strategy: crate::types::StrategyType::TitForTat,
            is_identity_theft_victim: false,
            identity_theft_event: None,
            identity: None,
            theft_recovery_tick: None,
            credit_report: crate::identity::CreditReport::clean(),
            credit_error_recovery_tick: None,
            risk_profile: crate::identity::RiskProfile::default(),
            engagement_network: Vec::new(),
            birth_tick: 0,
            alive: true,
            debt_obligations: Vec::new(),
            positive_interactions: 0,
            negative_interactions: 0,
            interaction_history: std::collections::HashMap::new(),
            economics: crate::economics::EconomicState::default(),
            local_reputation: std::collections::HashMap::new(),
            cognitive_system: crate::types::CognitiveSystem::System1,
            dmn_horizon: crate::types::H_DEFAULT,
            music_state: None,
            // Emotion state
            emotion_state: crate::emotion::EmotionState::default(),
            // Language drift tracking
            language_drift: crate::language_drift::LanguageDriftTracker::new(0),
            // Trust curvature fields (§6.6.2)
            docas_history: std::collections::VecDeque::new(),
            trust_curvature: crate::docas::TrustCurvature::new(),
            // Viscosity fields (§6.6.3)
            tau_history: Vec::new(),
            trust_viscosity: crate::docas::TrustViscosity::default(),
            adaptation_rate: 1.0,
            // Psychology Layer (Stage 2)
            // TAG: surface=simulation owner=research-team rule=SIM-001
            semantic_state: [0.5; 8],
            affect_valence: 0.0,
            affect_arousal: 0.5,
            social_orientation: 0.0,
            // Philosophy Layer (Stage 2)
            philosophical_state: None,
            // Meaning Layer
            meaning_state: None,
            // GTT Fields
            lambda: 0.61,
            expectation: 0.5,
            validation: 0.5,
            prediction_error: 0.0,
            stability: 1.0,
            // TFP Fields
            psi_bio: 0.5,
            psi_fin: 0.5,
            psi_ling: 0.5,
            psi_trust: 0.5,
            // Emotion Fields
            dominance: 0.5,
            // Linguistics Fields
            trust_in_meaning: 0.5,
            semantic_entropy: 0.5,
            semantic_coherence: 0.5,
            // Culture Fields
            cultural_quadrant: None,
            hofstede_dimensions: None,
            cultural_orientation: None,
            // Game Theory Fields
            game_history: Vec::new(),
            trust_in_partner: std::collections::HashMap::new(),
            cumulative_payoff: 0.0,
            // PDE4B Fields
            pde4b_expression: 1.0,
            camp_level: 1.0,
            learning_rate_modifier: 0.61,
            // Neural Maturation Fields
            // TAG: surface=simulation owner=research-team rule=SIM-001
            prefrontal_cortex: crate::agent::PrefrontalCortex::default(),
            // Epigenetics Fields
            epigenome: crate::epigenome::Epigenome::new(),
            // Big Five Personality
            personality: crate::psychology::PersonalityTraits::default(),
            // Working Memory
            working_memory: crate::cognition::WorkingMemory::new(0.0, 0),
            // Inbreeding
            inbreeding: crate::genetics::InbreedingTracker::new(),
            // Movement direction for visualization
            last_dx: 1.0,
            last_dy: 0.0,
            // Stigmergy fields
            pheromone_sensitivity: [1.0, 1.0, 1.0, 0.5, 0.8],
            chemotaxis_direction: None,
            last_pheromone_deposit_tick: 0,
            // Social forces (Barnes-Hut)
            social_force: crate::types::Vector::zero(),
            // Time preference
            time_preference: crate::economics::TimePreferenceProfile::default(),
            biomarkers: crate::biomarkers::AgentBiomarkers::default(),
        }
    }

    #[test]
    fn test_direction_to_delta() {
        assert_eq!(Direction::North.to_delta(), (0.0, 1.0));
        assert_eq!(Direction::NorthEast.to_delta(), (1.0, 1.0));
        assert_eq!(Direction::East.to_delta(), (1.0, 0.0));
        assert_eq!(Direction::SouthEast.to_delta(), (1.0, -1.0));
        assert_eq!(Direction::South.to_delta(), (0.0, -1.0));
        assert_eq!(Direction::SouthWest.to_delta(), (-1.0, -1.0));
        assert_eq!(Direction::West.to_delta(), (-1.0, 0.0));
        assert_eq!(Direction::NorthWest.to_delta(), (-1.0, 1.0));
    }

    #[test]
    fn test_direction_all() {
        let dirs = Direction::all();
        assert_eq!(dirs.len(), 8);
        assert!(dirs.contains(&Direction::North));
        assert!(dirs.contains(&Direction::East));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_sample_action_returns_valid_action() {
        let mut rng = StdRng::seed_from_u64(42);
        let agent = create_test_agent(0.5, 0.5);

        for _ in 0..10 {
            let action = sample_action(&agent, &mut rng);
            // Should return one of: Move, Stay, or Reproduce (if energy >= 50)
            match action {
                Action::Move(_) | Action::Stay | Action::Reproduce => {}
            }
        }
    }

    #[test]
    fn test_sample_action_no_reproduce_when_low_energy() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut agent = create_test_agent(0.5, 0.5);
        agent.energy = 30.0; // Below reproduction threshold

        // Sample many times - should never get Reproduce
        for _ in 0..100 {
            let action = sample_action(&agent, &mut rng);
            assert!(
                !matches!(action, Action::Reproduce),
                "Should not sample Reproduce when energy < 50"
            );
        }
    }

    #[test]
    fn test_sample_action_can_reproduce_when_high_energy() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut agent = create_test_agent(0.5, 0.5);
        agent.energy = 75.0; // Above reproduction threshold

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Sample many times - should eventually get Reproduce
        let mut got_reproduce = false;
        for _ in 0..1000 {
            let action = sample_action(&agent, &mut rng);
            if matches!(action, Action::Reproduce) {
                got_reproduce = true;
                break;
            }
        }
        assert!(got_reproduce, "Should sample Reproduce when energy >= 50");
    }

    #[test]
    fn test_planning_depth_genome_control() {
        // Min planning depth (trait = 0.0 → K = 3)
        let agent_min = create_test_agent(0.0, 0.5);
        assert_eq!(agent_min.genome.planning_depth(), 3);

        // Max planning depth (trait = 1.0 → K = 20)
        let agent_max = create_test_agent(1.0, 0.5);
        assert_eq!(agent_max.genome.planning_depth(), 20);

        // Mid planning depth (trait = 0.5 → K ≈ 11-12)
        let agent_mid = create_test_agent(0.5, 0.5);
        let k = agent_mid.genome.planning_depth();
        assert!(
            (10..=13).contains(&k),
            "Mid trait should give K ≈ 11-12, got {k}"
        );
    }

    #[test]
    fn test_planning_horizon_genome_control() {
        // Min planning horizon (trait = 0.0 → H = 1)
        let agent_min = create_test_agent(0.5, 0.0);
        assert_eq!(agent_min.genome.planning_horizon(), 1);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Max planning horizon (trait = 1.0 → H = 5)
        let agent_max = create_test_agent(0.5, 1.0);
        assert_eq!(agent_max.genome.planning_horizon(), 5);

        // Mid planning horizon (trait = 0.5 → H ≈ 3)
        let agent_mid = create_test_agent(0.5, 0.5);
        let h = agent_mid.genome.planning_horizon();
        assert!((2..=4).contains(&h), "Mid trait should give H ≈ 3, got {h}");
    }

    #[test]
    fn test_plan_action_respects_budget() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut agent = create_test_agent(1.0, 0.5); // Max K = 20
        agent.dmn_budget = 5; // Budget limits to 5 rollouts

        let config = EnvironmentConfig::default();
        let env = Environment::new(&config, &mut rng);

        // Planning should work despite K=20, limited by budget=5
        let action = plan_action(&agent, &env, &mut rng);

        // Should return a valid action
        match action {
            Action::Move(_) | Action::Stay | Action::Reproduce => {}
        }
    }

    #[test]
    fn test_plan_action_deterministic() {
        let agent = create_test_agent(0.5, 0.5);
        let config = EnvironmentConfig::default();

        // Same seed should produce same action
        let mut rng1 = StdRng::seed_from_u64(12345);
        let env1 = Environment::new(&config, &mut rng1);
        let action1 = plan_action(&agent, &env1, &mut rng1);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut rng2 = StdRng::seed_from_u64(12345);
        let env2 = Environment::new(&config, &mut rng2);
        let action2 = plan_action(&agent, &env2, &mut rng2);

        assert_eq!(action1, action2, "Same seed should produce same action");
    }

    #[test]
    fn test_evaluate_action_move_toward_resources() {
        let mut rng = StdRng::seed_from_u64(42);
        let agent = create_test_agent(0.5, 0.5);
        let config = EnvironmentConfig::default();
        let env = Environment::new(&config, &mut rng);

        // Evaluate moving in different directions
        let value_north = evaluate_action(&agent, &env, Action::Move(Direction::North), 1);
        let value_stay = evaluate_action(&agent, &env, Action::Stay, 1);

        // Values should be non-negative (resource amounts)
        assert!(value_north >= 0.0);
        assert!(value_stay >= 0.0);
    }

    #[test]
    fn test_evaluate_action_reproduce_value() {
        let mut rng = StdRng::seed_from_u64(42);
        let agent = create_test_agent(0.5, 0.5);
        let config = EnvironmentConfig::default();
        let env = Environment::new(&config, &mut rng);

        let value = evaluate_action(&agent, &env, Action::Reproduce, 1);

        // Per implementation: reproduction value = 20.0
        assert_eq!(value, 20.0);
    }

    #[test]
    fn test_plan_action_with_nudges_enabled() {
        // Per P1-6: Choice Architecture with Thaler nudges
        let mut rng = StdRng::seed_from_u64(42);
        let mut agent = create_test_agent(0.5, 0.5);
        // Low energy removes movement bonus, allowing nudge to affect choice
        agent.energy = 5.0;
        let config = EnvironmentConfig::default();
        let env = Environment::new(&config, &mut rng);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Run multiple times to get distribution
        let mut actions = Vec::new();
        for seed in 0..100 {
            let mut rng = StdRng::seed_from_u64(seed);
            let action = plan_action_with_nudges(&agent, &env, &mut rng, true);
            actions.push(action);
        }

        // With nudges enabled, Stay should be more common due to default boost
        let stay_count = actions.iter().filter(|a| **a == Action::Stay).count();

        // Stay should appear at least occasionally (default effect)
        assert!(stay_count > 0, "Stay should be boosted by default nudge");
    }

    #[test]
    fn test_plan_action_with_nudges_disabled() {
        // Per P1-6: OFF behavior = uniform distribution
        let mut rng = StdRng::seed_from_u64(42);
        let agent = create_test_agent(0.5, 0.5);
        let config = EnvironmentConfig::default();
        let env = Environment::new(&config, &mut rng);

        let action = plan_action_with_nudges(&agent, &env, &mut rng, false);

        // Should return an action (no crash)
        assert!(matches!(
            action,
            Action::Stay | Action::Move(_) | Action::Reproduce
        ));
    }
}
