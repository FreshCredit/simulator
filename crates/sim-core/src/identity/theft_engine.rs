// TAG: surface=simulation owner=research-team rule=SIM-001
//! Identity theft mechanics, opportunistic theft, and breach propagation

use super::*;
use rand::Rng;

/// Attempt to execute identity theft between two agents
pub fn attempt_identity_theft(
    thief: &mut crate::agent::Agent,
    victim: &mut crate::agent::Agent,
    current_tick: Tick,
    protocol_enabled: bool,
    global_stats: &mut IdentityTheftStats,
    rng: &mut impl Rng,
) -> TheftResult {
    // Increment attempt counter
    if let Some(identity) = &mut victim.identity {
        identity.theft_attempts += 1;
    }
    global_stats.attempts += 1;

    // Check if victim is already compromised
    if let Some(ref identity) = victim.identity {
        if identity.is_compromised {
            global_stats.record_failure(&TheftFailureReason::AlreadyCompromised);
            return TheftResult::Failed {
                reason: TheftFailureReason::AlreadyCompromised,
                thief_penalty: 0.0,
            };
        }
    }

    // Calculate base success probability
    // 5-10% annual rate = ~0.014-0.027% per tick (assuming 365 ticks/year)
    // At 1000 ticks/year: 0.5-1.0% per tick
    let base_success_rate = 0.0075; // 0.75% per tick = ~7.5% annual at 1000 ticks

    // Modify based on victim's security (reputation and verification)
    let victim_security = if let Some(ref identity) = victim.identity {
        let verification_bonus = if protocol_enabled {
            identity.verification_rate() * 0.5 // Up to 50% protection from verification
                                               // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        } else {
            0.0
        };
        let anchored_bonus = if identity.anchored_claims.is_empty() {
            0.0
        } else {
            0.2 // 20% reduction from anchored identity
        };
        let reputation_factor = victim.reputation_score * 0.3; // Up to 30% from reputation
        verification_bonus + anchored_bonus + reputation_factor
    } else {
        0.0
    };

    // Modify based on thief's skill (reputation-based)
    let thief_skill = thief.reputation_score * 0.1; // Small bonus/penalty

    let success_probability = (base_success_rate + thief_skill - victim_security).clamp(0.001, 0.5); // Never 0% or above 50%

    // Determine if theft succeeds
    let roll: f32 = rng.gen();

    if roll > success_probability {
        // Theft failed
        let failure_reason = if protocol_enabled && roll > success_probability * 2.0 {
            if victim
                .identity
                .as_ref()
                .is_some_and(|i| !i.anchored_claims.is_empty())
            {
                TheftFailureReason::AnchoredProtection
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            } else {
                TheftFailureReason::ProtocolVerification
            }
        } else if victim.reputation_score > 0.7 {
            TheftFailureReason::StrongSecurity
        } else {
            TheftFailureReason::RandomFailure
        };

        let thief_penalty = 0.1; // Reputation penalty for failed theft attempt
        thief.reputation_score = (thief.reputation_score - thief_penalty).max(0.0);

        global_stats.record_failure(&failure_reason);

        return TheftResult::Failed {
            reason: failure_reason,
            thief_penalty,
        };
    }

    // Theft succeeded
    if let Some(ref mut identity) = victim.identity {
        identity.mark_stolen(current_tick);

        // Determine what was stolen
        let stolen_claims: Vec<ClaimType> = identity.claims
            .iter()
            .filter(|c| c.value > 0.5) // High-value claims targeted
            .map(|c| c.claim_type)
            .collect();

        // Calculate reputation damage
        let base_damage = 0.3;
        let damage_multiplier = 1.0 - victim.reputation_score * 0.5; // Less damage if high rep
        let reputation_damage = base_damage * damage_multiplier;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        victim.reputation_score = (victim.reputation_score - reputation_damage).max(0.0);

        // Update victim's DOCAS (distress)
        // Access levels array directly: [D, O, C, A, S]
        victim.docas.levels[2] += 0.3; // Cortisol - Stress from theft
        victim.docas.levels[3] += 0.2; // Adrenaline - Arousal/alert
        victim.docas.levels[1] = (victim.docas.levels[1] - 0.2).max(0.0); // Oxytocin - Trust damaged

        global_stats.record_success(
            TheftMethod::SocialEngineering,
            identity.recovery_duration as f32,
        );
        global_stats.total_reputation_damage += reputation_damage;

        // Thief gains some benefit (but also risk)
        let theft_benefit = 0.1;
        thief.energy += theft_benefit * 100.0; // Energy boost from successful theft

        // Update victim's theft status
        victim.is_identity_theft_victim = true;

        TheftResult::Success {
            stolen_claims,
            reputation_damage,
            method: TheftMethod::SocialEngineering,
        }
    } else {
        // Victim has no identity - cannot steal
        global_stats.record_failure(&TheftFailureReason::RandomFailure);
        TheftResult::Failed {
            reason: TheftFailureReason::RandomFailure,
            thief_penalty: 0.0,
        }
    }
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Process identity theft for entire population - HYBRID MODEL
///
/// Implements three mechanisms:
/// - 20% Network breach propagation (through engagement network)
/// - 60% Opportunistic theft (risk-weighted selection)
/// - 20% Mega-breach residual (large-scale events)
///
/// Per BJS 2016: 10% annual victimization with risk clustering
pub fn process_identity_theft_population(
    agents: &mut [crate::agent::Agent],
    current_tick: Tick,
    protocol_enabled: bool,
    global_stats: &mut IdentityTheftStats,
    rng: &mut impl Rng,
) {
    // Update recovery progress for all agents
    for agent in agents.iter_mut() {
        if let Some(ref mut identity) = agent.identity {
            identity.update_recovery(current_tick);

            // Clear victim status if recovered
            if !identity.is_compromised && agent.is_identity_theft_victim {
                agent.is_identity_theft_victim = false;
            }
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Update risk profiles based on agent state
        update_risk_profile(agent, current_tick);
    }

    // === HYBRID MODEL: Three theft mechanisms ===

    // 1. NETWORK BREACH PROPAGATION (20% of thefts)
    // Propagation probability: p = 0.08 per edge
    process_network_breach_propagation(agents, current_tick, protocol_enabled, global_stats, rng);

    // 2. OPPORTUNISTIC THEFT (60% of thefts)
    // Risk-weighted victim selection (not random)
    process_opportunistic_theft(agents, current_tick, protocol_enabled, global_stats, rng);

    // 3. MEGA-BREACH RESIDUAL (20% of thefts)
    // Large-scale centralized events
    process_mega_breach(agents, current_tick, protocol_enabled, global_stats, rng);

    // Update current theft rate
    let affected_agents = agents.iter().filter(|a| a.is_identity_theft_victim).count();
    global_stats.update_current_rate(affected_agents, agents.len());
    global_stats.protocol_protection_rate = global_stats.compute_protection_effectiveness();
}

/// Update risk profile based on agent demographics and state
fn update_risk_profile(agent: &mut crate::agent::Agent, _current_tick: Tick) {
    // Income proxy: Based on energy/wealth (higher = more risk per BJS)
    let wealth_factor = agent.economics.wealth / 1000.0; // Normalize
    agent.risk_profile.income_proxy = wealth_factor.min(1.0);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Age risk: Peak at middle age (35-64 equivalent in simulation)
    let age_normalized = (agent.age as f32 / 100.0).min(1.0); // 100 ticks = middle age
    agent.risk_profile.age_risk = if age_normalized > 0.3 && age_normalized < 0.7 {
        0.8 // Peak risk age
    } else {
        0.4 // Lower risk for young/old
    };

    // Network exposure: Based on engagement network size
    let network_size = agent.engagement_network.len() as f32;
    agent.risk_profile.network_exposure = (network_size / 20.0).min(1.0); // 20+ connections = max exposure

    // Protocol protection: Based on verification rate
    agent.risk_profile.protocol_protection = agent
        .identity
        .as_ref()
        .map_or(0.0, Identity::verification_rate);
}

/// Process network breach propagation (20% of thefts)
/// Spreads through engagement network edges
fn process_network_breach_propagation(
    agents: &mut [crate::agent::Agent],
    _current_tick: Tick,
    _protocol_enabled: bool,
    global_stats: &mut IdentityTheftStats,
    rng: &mut impl Rng,
) {
    // Breach initiation rate: λ = 0.032 per tick for N=1000 to achieve ~10% annual
    let breach_initiation_rate = 0.032 * (agents.len() as f32 / 1000.0);

    if rng.gen::<f32>() >= breach_initiation_rate {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        return; // No breach this tick
    }

    // Select origin agent (random)
    let origin_idx = rng.gen_range(0..agents.len());

    // Propagation probability per edge: p = 0.08
    const PROPAGATION_PROB: f32 = 0.08;
    const MAX_CASCADE_DEPTH: u32 = 3;

    // BFS for cascade propagation
    let mut affected: Vec<(usize, u32)> = vec![(origin_idx, 0)]; // (agent_idx, depth)
    let mut visited = std::collections::HashSet::new();
    visited.insert(origin_idx);

    let mut i = 0;
    // P10-R4: bounded — BFS index walk; each iteration consumes one affected entry and i only advances.
    while i < affected.len() {
        let (current_idx, depth) = affected[i];
        i += 1;

        if depth >= MAX_CASCADE_DEPTH {
            continue;
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Get engagement network of current agent
        let connections: Vec<AgentId> = if let Some(agent) = agents.get(current_idx) {
            agent.engagement_network.clone()
        } else {
            continue;
        };

        // Try to propagate to each connection
        for &connected_id in &connections {
            // Find index of connected agent
            if let Some(connected_idx) = agents.iter().position(|a| a.id == connected_id) {
                if visited.contains(&connected_idx) {
                    continue;
                }

                // Propagation attempt
                if rng.gen::<f32>() < PROPAGATION_PROB {
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    visited.insert(connected_idx);
                    affected.push((connected_idx, depth + 1));
                }
            }
        }
    }

    // Execute theft on affected agents
    for (victim_idx, _depth) in affected {
        if victim_idx >= agents.len() {
            continue;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Check if victim is vulnerable
        let victim_vulnerable = agents[victim_idx]
            .identity
            .as_ref()
            .is_some_and(Identity::is_vulnerable);

        if !victim_vulnerable {
            continue;
        }

        // Execute theft (simplified - no specific thief for network breach)
        let victim = &mut agents[victim_idx];

        // Apply network breach theft
        if victim.identity.is_some() {
            let base_damage = 0.3;
            let damage_multiplier = 1.0 - victim.reputation_score * 0.5;
            let reputation_damage = base_damage * damage_multiplier;

            victim.reputation_score = (victim.reputation_score - reputation_damage).max(-1.0);
            victim.risk_profile.record_victimization();

            // Record in stats
            global_stats.record_success(TheftMethod::AccountTakeover, 300.0);
            global_stats.total_reputation_damage += reputation_damage;

            // DOCAS impact
            victim.docas.levels[2] += 0.3; // Cortisol
            victim.docas.levels[3] += 0.2; // Adrenaline
            victim.docas.levels[1] = (victim.docas.levels[1] - 0.2).max(0.0); // Oxytocin
        }
    }
}

/// Process opportunistic theft (60% of thefts)
/// Risk-weighted victim selection (NOT random)
fn process_opportunistic_theft(
    agents: &mut [crate::agent::Agent],
    current_tick: Tick,
    protocol_enabled: bool,
    global_stats: &mut IdentityTheftStats,
    rng: &mut impl Rng,
) {
    // Theft attempt rate calibrated for 60% of total 10% annual rate
    let base_rate = 0.10; // 10% annual target
    let opportunistic_weight = 0.60; // 60% of thefts
    let ticks_per_year = 1000.0;
    let attempt_rate = (base_rate * opportunistic_weight) / ticks_per_year;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let num_attempts = (agents.len() as f32 * attempt_rate) as usize;
    if num_attempts == 0 {
        return;
    }

    // Build risk-weighted victim pool
    let mut victim_pool: Vec<(usize, f32)> = agents
        .iter()
        .enumerate()
        .filter(|(_, a)| a.identity.as_ref().is_some_and(Identity::is_vulnerable))
        .map(|(idx, a)| {
            let risk = a.risk_profile.compute_risk_score();
            (idx, risk)
        })
        .collect();

    if victim_pool.is_empty() {
        return;
    }

    // Sort by risk (highest first)
    victim_pool.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Select thieves (low reputation agents)
    let mut thief_indices: Vec<usize> = (0..agents.len()).collect();
    thief_indices.sort_by(|&i, &j| {
        let rep_i = agents[i].reputation_score;
        let rep_j = agents[j].reputation_score;
        rep_i
            .partial_cmp(&rep_j)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Execute thefts
    for &thief_idx in thief_indices.iter().take(num_attempts) {
        // Select victim using risk-weighted probability
        let total_risk: f32 = victim_pool.iter().map(|(_, r)| r).sum();
        if total_risk <= 0.0 {
            break;
        }

        let mut selector = rng.gen::<f32>() * total_risk;
        let mut victim_idx = victim_pool[0].0;

        for (idx, risk) in &victim_pool {
            selector -= risk;
            if selector <= 0.0 {
                victim_idx = *idx;
                break;
            }
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }

        if victim_idx == thief_idx {
            continue; // Can't steal from self
        }

        // Execute theft
        let (thief, victim) = if thief_idx < victim_idx {
            let (left, right) = agents.split_at_mut(victim_idx);
            (&mut left[thief_idx], &mut right[0])
        } else {
            let (left, right) = agents.split_at_mut(thief_idx);
            (&mut right[0], &mut left[victim_idx])
        };

        attempt_identity_theft(
            thief,
            victim,
            current_tick,
            protocol_enabled,
            global_stats,
            rng,
        );
    }
}

/// Process mega-breach residual (20% of thefts)
/// Large-scale centralized events affecting multiple agents
fn process_mega_breach(
    agents: &mut [crate::agent::Agent],
    _current_tick: Tick,
    _protocol_enabled: bool,
    global_stats: &mut IdentityTheftStats,
    rng: &mut impl Rng,
) {
    // Mega-breach rate: 20% of total 10% annual rate
    let base_rate = 0.10;
    let mega_weight = 0.20;
    let ticks_per_year = 1000.0;
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let breach_rate = (base_rate * mega_weight) / ticks_per_year;

    // Scale by population
    let effective_rate = breach_rate * (agents.len() as f32 / 1000.0);

    if rng.gen::<f32>() >= effective_rate {
        return; // No mega-breach this tick
    }

    // Mega-breach affects 10-30% of population (like Equifax: 147M/330M = ~45%)
    let affected_fraction = 0.10 + rng.gen::<f32>() * 0.20; // 10-30%
    let num_affected = (agents.len() as f32 * affected_fraction) as usize;

    // Select random victims
    let mut victim_indices: Vec<usize> = (0..agents.len()).collect();
    // Shuffle using Fisher-Yates
    for i in (1..victim_indices.len()).rev() {
        let j = rng.gen_range(0..=i);
        victim_indices.swap(i, j);
    }

    // Execute mega-breach thefts
    for &victim_idx in victim_indices.iter().take(num_affected) {
        let victim = &mut agents[victim_idx];

        // Check if vulnerable
        let vulnerable = victim
            .identity
            .as_ref()
            .is_some_and(Identity::is_vulnerable);

        if !vulnerable {
            continue;
        }

        // Apply mega-breach theft (more severe due to scale)
        if victim.identity.is_some() {
            let reputation_damage = 0.35; // Higher damage for mega-breach

            victim.reputation_score = (victim.reputation_score - reputation_damage).max(-1.0);
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            victim.risk_profile.record_victimization();

            // Record in stats
            global_stats.record_success(TheftMethod::CredentialTheft, 400.0);
            global_stats.total_reputation_damage += reputation_damage;

            // Severe DOCAS impact
            victim.docas.levels[2] += 0.4; // High cortisol
            victim.docas.levels[3] += 0.3; // High adrenaline
            victim.docas.levels[1] = (victim.docas.levels[1] - 0.3).max(0.0); // Damaged trust
            victim.docas.levels[4] -= 0.2; // Reduced serotonin
        }
    }
}
