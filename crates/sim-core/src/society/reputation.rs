//! Reputation subsystem
//!
//! Per §6.22.2: Network-based reputation computation and propagation

use crate::agent::Agent;
use crate::society::attestations::Attestation;
use crate::types::{AgentId, GroupId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute reputation from attestation network
/// Per §6.22.2 lines 2976-3003
#[must_use]
pub fn compute_reputation(
    agent_id: AgentId,
    attestations: &[Attestation],
    agents: &[Agent],
    current_tick: Tick,
) -> f32 {
    // Filter attestations about this agent
    let relevant: Vec<_> = attestations
        .iter()
        .filter(|a| a.subject_id == agent_id)
        .filter(|a| {
            // Not expired
            a.tick_expires.is_none_or(|exp| current_tick < exp)
        })
        .collect();

    if relevant.is_empty() {
        return 0.5; // Neutral reputation for new agents
    }

    // Weighted by attestor's own reputation (web of trust)
    let weighted_sum: f32 = relevant
        .iter()
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .map(|a| {
            let attestor_rep = agents
                .iter()
                .find(|ag| ag.id == a.attestor_id)
                .map_or(0.1, |ag| ag.reputation_score.max(0.1));

            // Recency decay: older attestations matter less
            let age = current_tick.saturating_sub(a.tick_created);
            let recency_weight = (-0.001 * age as f32).exp(); // Half-life ~700 ticks

            a.value * attestor_rep * recency_weight * (1.0 + a.stake) // Stake amplifies
        })
        .sum();

    let total_weight: f32 = relevant.len() as f32 + 0.01; // Avoid divide by zero

    (weighted_sum / total_weight).clamp(-1.0, 1.0)
}

// CODE-001: Deprecated with migration path
// Migration: Use agent.compute_trust() for the canonical trust computation
// or agent.unified_trust() for the comprehensive multi-domain trust estimate.
// This function is maintained for backward compatibility in society module tests.
#[deprecated(
    since = "0.2.0",
    note = "Use agent.compute_trust() or agent.unified_trust() instead"
)]
// TAG: surface=simulation owner=research-team rule=SIM-001
#[must_use]
/// compute pairwise trust.
pub fn compute_pairwise_trust(agent_a: &Agent, _agent_b: &Agent) -> f32 {
    // Delegate to canonical implementation
    agent_a.compute_trust()
}

// ============================================================================
// REPUTATION PROPAGATION (SIM-009)
// Per §6.22: Network-based reputation diffusion through social network
// ============================================================================

/// Reputation propagation configuration
#[derive(Debug, Clone, Copy)]
pub struct ReputationPropagationConfig {
    /// Decay factor for each hop (0.0-1.0)
    pub decay_factor: f32,
    /// Maximum propagation distance (hops)
    pub max_hops: u32,
    /// Minimum reputation threshold to propagate
    pub min_reputation_threshold: f32,
    /// Attestation network boost factor
    pub attestation_boost: f32,
}

impl Default for ReputationPropagationConfig {
    fn default() -> Self {
        Self {
            decay_factor: 0.5,             // 50% decay per hop
            max_hops: 3,                   // Max 3 hops
            min_reputation_threshold: 0.2, // Only propagate |reputation| > 0.2
            attestation_boost: 1.5,        // 50% boost through attestation links
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Propagate reputation through social network (SIM-009)
///
/// Uses network diffusion model where reputation spreads through:
/// 1. Direct interaction history (`local_reputation` `HashMap`)
/// 2. Attestation network (`attestations_given/received`)
/// 3. Group membership (shared `group_id`)
///
/// Returns number of reputation updates performed
pub fn propagate_reputation_network(
    agents: &mut [Agent],
    attestations: &[Attestation],
    config: &ReputationPropagationConfig,
) -> usize {
    let mut update_count = 0;

    // Build social network adjacency list
    let network = build_social_network(agents, attestations);

    // For each agent, propagate their reputation to connected agents
    for i in 0..agents.len() {
        let agent_id = agents[i].id;
        let agent_reputation = agents[i].reputation_score;

        // Only propagate significant reputations
        if agent_reputation.abs() < config.min_reputation_threshold {
            continue;
        }

        // BFS propagation through network
        let updates = propagate_from_source(agents, &network, agent_id, agent_reputation, config);

        update_count += updates;
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    update_count
}

/// Build social network from agent interactions and attestations
pub(crate) fn build_social_network(
    agents: &[Agent],
    attestations: &[Attestation],
) -> std::collections::HashMap<AgentId, Vec<(AgentId, f32)>> {
    let mut network: std::collections::HashMap<AgentId, Vec<(AgentId, f32)>> =
        std::collections::HashMap::new();

    // Add edges from local reputation (prior interactions)
    for agent in agents {
        for (&other_id, &reputation) in &agent.local_reputation {
            // Edge weight based on interaction strength
            let weight = reputation.abs();
            network
                .entry(agent.id)
                .or_default()
                .push((other_id, weight));
        }
    }

    // Add edges from attestations (stronger connections)
    for attestation in attestations {
        let weight = attestation.value.abs() * attestation.stake;
        network
            .entry(attestation.attestor_id)
            .or_default()
            .push((attestation.subject_id, weight));
        network
            .entry(attestation.subject_id)
            .or_default()
            .push((attestation.attestor_id, weight));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Add edges from shared group membership (only for agents assigned to a group)
    let group_members: std::collections::HashMap<GroupId, Vec<AgentId>> = agents
        .iter()
        .filter_map(|a| a.group_id.map(|group| (group, a.id)))
        .fold(std::collections::HashMap::new(), |mut acc, (group, id)| {
            acc.entry(group).or_default().push(id);
            acc
        });

    for (_group, members) in group_members {
        if members.len() > 1 {
            for i in 0..members.len() {
                for j in (i + 1)..members.len() {
                    // Group membership creates weak connection
                    let weight = 0.3;
                    network
                        .entry(members[i])
                        .or_default()
                        .push((members[j], weight));
                    network
                        .entry(members[j])
                        .or_default()
                        .push((members[i], weight));
                }
            }
        }
    }

    network
}

/// Propagate reputation from a source agent through network
fn propagate_from_source(
    // TAG: surface=simulation owner=research-team rule=SIM-001
    agents: &mut [Agent],
    network: &std::collections::HashMap<AgentId, Vec<(AgentId, f32)>>,
    source_id: AgentId,
    source_reputation: f32,
    config: &ReputationPropagationConfig,
) -> usize {
    let mut visited = std::collections::HashSet::new();
    let mut queue: std::collections::VecDeque<(AgentId, u32, f32)> =
        std::collections::VecDeque::new();
    let mut update_count = 0;

    // Start BFS from source's neighbors
    if let Some(neighbors) = network.get(&source_id) {
        for &(neighbor_id, weight) in neighbors {
            if neighbor_id != source_id {
                let initial_signal = source_reputation * weight * config.decay_factor;
                queue.push_back((neighbor_id, 1, initial_signal));
            }
        }
    }

    visited.insert(source_id);

    while let Some((current_id, hops, signal)) = queue.pop_front() {
        if visited.contains(&current_id) || hops > config.max_hops {
            continue;
        }

        visited.insert(current_id);

        // Find agent index
        if let Some(agent_idx) = agents.iter().position(|a| a.id == current_id) {
            // Update local reputation for source
            let current_reputation = agents[agent_idx].get_local_reputation(source_id);
            let new_reputation = (current_reputation + signal).clamp(-1.0, 1.0);

            if (new_reputation - current_reputation).abs() > 0.001 {
                agents[agent_idx].update_local_reputation(source_id, signal);
                update_count += 1;
            }

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Continue propagation if signal is still strong
            let next_signal = signal * config.decay_factor;
            if next_signal.abs() >= config.min_reputation_threshold {
                if let Some(neighbors) = network.get(&current_id) {
                    for &(neighbor_id, weight) in neighbors {
                        if !visited.contains(&neighbor_id) && neighbor_id != source_id {
                            queue.push_back((neighbor_id, hops + 1, next_signal * weight));
                        }
                    }
                }
            }
        }
    }

    update_count
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute network-based reputation metrics
#[must_use]
pub fn compute_reputation_network_metrics(agents: &[Agent]) -> ReputationNetworkMetrics {
    let mut total_connections = 0;
    let mut total_reputation_variance = 0.0;

    for agent in agents {
        let local_rep_count = agent.local_reputation.len();
        total_connections += local_rep_count;

        // Compute variance in local reputations
        if local_rep_count > 1 {
            let values: Vec<f32> = agent.local_reputation.values().copied().collect();
            let mean = values.iter().sum::<f32>() / values.len() as f32;
            let variance =
                values.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;
            total_reputation_variance += variance;
        }
    }

    let agent_count = agents.len() as f32;
    let avg_connections = if agents.is_empty() {
        0.0
    } else {
        total_connections as f32 / agent_count
    };

    ReputationNetworkMetrics {
        avg_connections,
        total_connections,
        avg_reputation_variance: if agents.is_empty() {
            0.0
        } else {
            total_reputation_variance / agent_count
        },
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Metrics for reputation network structure
#[derive(Debug, Clone, Default)]
pub struct ReputationNetworkMetrics {
    /// Average number of reputation connections per agent
    pub avg_connections: f32,
    /// Total number of reputation connections
    pub total_connections: usize,
    /// Average variance in reputation values
    pub avg_reputation_variance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Agent;
    use crate::genome::Genome;
    use crate::society::attestations::{AnchorStatus, Attestation, ClaimType};
    use crate::types::{Position, Sex};

    fn create_test_agent(id: u64) -> Agent {
        let genome = Genome::default();
        Agent::new(id, Position::new_2d(0.0, 0.0), genome, 0, Sex::Male)
    }

    fn create_attestation(
        id: u64,
        attestor_id: u64,
        subject_id: u64,
        value: f32,
        tick_created: Tick,
    ) -> Attestation {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Attestation {
            id,
            attestor_id,
            subject_id,
            claim_type: ClaimType::Cooperation,
            value,
            tick_created,
            tick_expires: None,
            stake: 0.1,
            attestation_hash: None,
            anchor_tx_id: None,
            anchor_block: None,
            anchor_status: AnchorStatus::None,
        }
    }

    #[test]
    fn test_compute_reputation_no_attestations() {
        let agent = create_test_agent(1);
        let agents = vec![agent];
        let attestations: Vec<Attestation> = vec![];
        let rep = compute_reputation(1, &attestations, &agents, 10);
        assert_eq!(rep, 0.5);
    }

    #[test]
    fn test_compute_reputation_with_positive_attestation() {
        let agent = create_test_agent(1);
        let mut attestor = create_test_agent(2);
        attestor.reputation_score = 0.8;
        let agents = vec![agent, attestor];
        let attestations = vec![create_attestation(1, 2, 1, 1.0, 0)];
        let rep = compute_reputation(1, &attestations, &agents, 10);
        assert!(rep > 0.5);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_compute_reputation_expired_filtered() {
        let attestor = create_test_agent(2);
        let subject = create_test_agent(1);
        let agents = vec![subject, attestor];
        let attestations = vec![Attestation {
            id: 1,
            attestor_id: 2,
            subject_id: 1,
            claim_type: ClaimType::Cooperation,
            value: 1.0,
            tick_created: 0,
            tick_expires: Some(5),
            stake: 0.1,
            attestation_hash: None,
            anchor_tx_id: None,
            anchor_block: None,
            anchor_status: AnchorStatus::None,
        }];
        let rep = compute_reputation(1, &attestations, &agents, 10);
        assert_eq!(rep, 0.5); // Expired, so neutral
    }

    #[test]
    fn test_compute_reputation_network_metrics_empty() {
        let agents: Vec<Agent> = vec![];
        let metrics = compute_reputation_network_metrics(&agents);
        assert_eq!(metrics.avg_connections, 0.0);
        assert_eq!(metrics.total_connections, 0);
        assert_eq!(metrics.avg_reputation_variance, 0.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_compute_reputation_network_metrics_with_connections() {
        let mut agent1 = create_test_agent(1);
        let mut agent2 = create_test_agent(2);
        agent1.update_local_reputation(2, 0.8);
        agent2.update_local_reputation(1, 0.6);
        let agents = vec![agent1, agent2];
        let metrics = compute_reputation_network_metrics(&agents);
        assert_eq!(metrics.total_connections, 2);
        assert_eq!(metrics.avg_connections, 1.0);
    }

    #[test]
    fn test_build_social_network_from_local_reputation() {
        let mut agent1 = create_test_agent(1);
        let mut agent2 = create_test_agent(2);
        agent1.update_local_reputation(2, 0.8);
        agent2.update_local_reputation(1, 0.6);
        let agents = vec![agent1, agent2];
        let attestations: Vec<Attestation> = vec![];
        let network = build_social_network(&agents, &attestations);
        assert!(network.contains_key(&1));
        assert!(network.contains_key(&2));
    }

    #[test]
    fn test_build_social_network_from_group_membership() {
        let mut agent1 = create_test_agent(1);
        let mut agent2 = create_test_agent(2);
        agent1.group_id = Some(100);
        agent2.group_id = Some(100);
        let agents = vec![agent1, agent2];
        let attestations: Vec<Attestation> = vec![];
        let network = build_social_network(&agents, &attestations);
        // Should have edges from shared group membership
        assert!(network.get(&1).unwrap().iter().any(|(id, _)| *id == 2));
        assert!(network.get(&2).unwrap().iter().any(|(id, _)| *id == 1));
    }

    #[test]
    fn test_ungrouped_agents_do_not_form_false_group_clique() {
        let mut agent1 = create_test_agent(1);
        let mut agent2 = create_test_agent(2);
        let mut agent3 = create_test_agent(3);
        agent1.group_id = None;
        agent2.group_id = None;
        agent3.group_id = None;
        let agents = vec![agent1, agent2, agent3];
        let attestations: Vec<Attestation> = vec![];
        let network = build_social_network(&agents, &attestations);
        assert!(!network.contains_key(&1));
        assert!(!network.contains_key(&2));
        assert!(!network.contains_key(&3));
    }
}
