//! Group formation subsystem
//!
//! SIM-GROUP-001: Groups form from spatial proximity + genome similarity
//! Per §4.4.2: Groups form from repeated local interactions

use crate::agent::Agent;
use crate::genome::Genome;
use crate::types::GroupId;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Group formation based on proximity AND genome trait compatibility
/// SIM-GROUP-001: Groups form from spatial proximity + genome similarity
/// Per §4.4.2: Groups form from repeated local interactions
pub fn form_groups(agents: &mut [Agent], proximity_threshold: f32, next_group_id: &mut GroupId) {
    form_groups_with_compatibility(agents, proximity_threshold, 0.3, next_group_id);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Group formation with configurable genome compatibility threshold
/// SIM-GROUP-001: Groups require BOTH spatial proximity AND genome trait compatibility
/// Per §4.4.2: Groups form from repeated local interactions
/// Per §7.5 lines 3927-3960: Genome similarity influences group formation
pub fn form_groups_with_compatibility(
    agents: &mut [Agent],
    proximity_threshold: f32,
    compatibility_threshold: f32,
    next_group_id: &mut GroupId,
) {
    for i in 0..agents.len() {
        if !agents[i].alive || agents[i].group_id.is_some() {
            continue; // Skip dead or already grouped agents
        }

        // Find nearby agents without groups that are genome-compatible
        let agent_pos = agents[i].position;
        let agent_genome = &agents[i].genome;
        // P1 FIX: Pre-allocate with estimated capacity to reduce reallocations
        let estimated_nearby = agents.len().saturating_sub(i + 1) / 4;
        let mut nearby_compatible = Vec::with_capacity(estimated_nearby.max(4));

        for (j, agent) in agents.iter().enumerate() {
            if i == j || !agent.is_alive_and_ungrouped() {
                continue;
            }

            let distance = agent_pos.distance(&agent.position);
            if distance <= proximity_threshold {
                // SIM-GROUP-001: Check genome trait compatibility
                let compatibility = genome_compatibility(agent_genome, &agent.genome);
                if compatibility >= compatibility_threshold {
                    nearby_compatible.push(j);
                }
            }
        }

        // If enough nearby compatible agents, form a group
        if nearby_compatible.len() >= 2 {
            let new_group_id = *next_group_id;
            *next_group_id += 1;

            agents[i].group_id = Some(new_group_id);
            for &j in &nearby_compatible {
                agents[j].group_id = Some(new_group_id);
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute genome trait compatibility between two agents
/// SIM-GROUP-001: Used for group formation decisions
/// Returns similarity score [0.0, 1.0] where 1.0 = identical genomes
#[must_use]
pub fn genome_compatibility(genome_a: &Genome, genome_b: &Genome) -> f32 {
    // Compare cooperation tendency (trait 14) and social traits (O gain = trait 6)
    // These are the most relevant for group formation
    let coop_a = genome_a.cooperation_tendency();
    let coop_b = genome_b.cooperation_tendency();
    let coop_diff = (coop_a - coop_b).abs();

    // Oxytocin gain (social affiliation trait)
    let o_gain_a = genome_a.docas_beta()[1]; // O is index 1
    let o_gain_b = genome_b.docas_beta()[1];
    let o_diff = (o_gain_a - o_gain_b).abs();

    // Risk tolerance similarity
    let risk_a = genome_a.risk_tolerance();
    let risk_b = genome_b.risk_tolerance();
    let risk_diff = (risk_a - risk_b).abs();

    // Compatibility = 1 - weighted average of differences
    // Higher weight on cooperation (most important for group cohesion)
    let weighted_diff = 0.5 * coop_diff + 0.3 * o_diff + 0.2 * risk_diff;
    (1.0 - weighted_diff).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Agent;
    use crate::genome::Genome;
    use crate::types::{Position, Sex};

    fn create_test_agent(id: u64, x: f32, y: f32) -> Agent {
        let genome = Genome::default();
        Agent::new(id, Position::new_2d(x, y), genome, 0, Sex::Male)
    }

    #[test]
    fn test_genome_compatibility_identical() {
        let genome = Genome::default();
        let compat = genome_compatibility(&genome, &genome);
        assert!((compat - 1.0).abs() < 0.001);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_genome_compatibility_range() {
        let genome_a = Genome::default();
        let genome_b = Genome::default();
        let compat = genome_compatibility(&genome_a, &genome_b);
        assert!((0.0..=1.0).contains(&compat));
    }

    #[test]
    fn test_form_groups_with_compatibility_no_nearby() {
        let mut agents = vec![
            create_test_agent(1, 0.0, 0.0),
            create_test_agent(2, 100.0, 100.0),
            create_test_agent(3, 200.0, 200.0),
        ];
        let mut next_group_id = 1;
        form_groups_with_compatibility(&mut agents, 10.0, 0.3, &mut next_group_id);
        assert!(agents.iter().all(|a| a.group_id.is_none()));
    }

    #[test]
    fn test_form_groups_with_compatibility_nearby_form_group() {
        let mut agents = vec![
            create_test_agent(1, 0.0, 0.0),
            create_test_agent(2, 1.0, 0.0),
            create_test_agent(3, 0.0, 1.0),
        ];
        let mut next_group_id = 1;
        form_groups_with_compatibility(&mut agents, 10.0, 0.3, &mut next_group_id);
        // All identical genomes should be compatible and form a group
        assert!(agents.iter().all(|a| a.group_id.is_some()));
        assert_eq!(agents[0].group_id, agents[1].group_id);
        assert_eq!(agents[1].group_id, agents[2].group_id);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_form_groups_skips_dead() {
        let mut agents = vec![
            create_test_agent(1, 0.0, 0.0),
            create_test_agent(2, 1.0, 0.0),
            create_test_agent(3, 0.0, 1.0),
        ];
        agents[0].alive = false;
        let mut next_group_id = 1;
        form_groups_with_compatibility(&mut agents, 10.0, 0.3, &mut next_group_id);
        assert!(agents[0].group_id.is_none());
    }

    #[test]
    fn test_form_groups_skips_already_grouped() {
        let mut agents = vec![
            create_test_agent(1, 0.0, 0.0),
            create_test_agent(2, 1.0, 0.0),
            create_test_agent(3, 0.0, 1.0),
        ];
        agents[0].group_id = Some(99);
        let mut next_group_id = 1;
        form_groups_with_compatibility(&mut agents, 10.0, 0.3, &mut next_group_id);
        assert_eq!(agents[0].group_id, Some(99));
    }
}
