// TAG: surface=simulation owner=research-team rule=SIM-001
//! Social norms, institutions, encounters, and game theory dynamics

use super::*;
use crate::agent::Agent;
use crate::identity::{ClaimType as IdentityClaimType, Identity, IdentityTheftStats};
use crate::society::{
    check_frequency_dependent_emergence, check_institution_emergence, compute_norm_compliance,
    enforce_group_norms, evolve_institutions, form_groups, Norm, SanctionResult,
};

impl Simulation {
    pub(crate) fn process_stage3_society(&mut self) {
        self.apply_norm_compliance();
        self.process_norm_emergence();
        self.process_norm_enforcement();
        self.process_institution_emergence();
        self.process_institution_evolution();
    }

    pub(crate) fn apply_norm_compliance(&mut self) {
        if !self.config.toggles.norms_enabled || self.active_norms.is_empty() {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            return;
        }
        for agent in &mut self.agents {
            if !agent.alive {
                continue;
            }
            let group_id = agent.group_id;
            let applicable_norms: Vec<&Norm> = self
                .active_norms
                .iter()
                .filter(|n| n.group_id.is_none() || n.group_id == group_id)
                .collect();

            let trust_modifier = (agent.compute_trust() - 1.0) * 0.2;
            let mut total_compliance = 0.0;
            let mut norm_count = 0;
            for norm in &applicable_norms {
                let base_compliance = compute_norm_compliance(agent, norm, 0.5);
                let compliance = (base_compliance + trust_modifier).clamp(0.0, 1.0);
                total_compliance += compliance;
                norm_count += 1;
            }

            if norm_count > 0 {
                let avg_compliance = total_compliance / norm_count as f32;
                if avg_compliance > 0.7 {
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    agent.docas.levels[4] += 0.05;
                } else if avg_compliance < 0.3 {
                    agent.docas.levels[2] += 0.05;
                }
                agent.docas.levels[2] = agent.docas.levels[2].clamp(0.0, 1.0);
                agent.docas.levels[4] = agent.docas.levels[4].clamp(0.0, 1.0);
            }
        }
    }

    pub(crate) fn process_norm_emergence(&mut self) {
        if !self.config.toggles.norms_enabled || self.tick % 100 != 0 {
            return;
        }
        let mut group_agents: std::collections::BTreeMap<Option<GroupId>, Vec<&Agent>> =
            std::collections::BTreeMap::new();
        for agent in &self.agents {
            if agent.alive {
                group_agents.entry(agent.group_id).or_default().push(agent);
            }
        }

        for (group_id, agents) in &group_agents {
            if agents.len() < 5 {
                continue;
                // TAG: surface=simulation owner=research-team rule=SIM-001
            }
            let freq = self.behavior_frequencies.entry(*group_id).or_default();
            if let Some(new_norm) = check_frequency_dependent_emergence(
                freq,
                *group_id,
                self.tick,
                &mut self.next_norm_id,
            ) {
                self.active_norms.push(new_norm);
            }
        }
    }

    pub(crate) fn find_norm_violations(
        agents: &[Agent],
        norms: &[Norm],
        rng: &mut rand::rngs::StdRng,
    ) -> Vec<(AgentId, NormId)> {
        let mut violations = Vec::new();
        for agent in agents {
            if !agent.alive {
                continue;
            }
            for norm in norms {
                if norm.group_id.is_some() && norm.group_id != agent.group_id {
                    continue;
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                }
                let compliance = compute_norm_compliance(agent, norm, 0.5);
                if rng.gen::<f32>() > compliance {
                    violations.push((agent.id, norm.id));
                }
            }
        }
        violations
    }

    pub(crate) fn apply_enforcement_deltas(&mut self, results: &[SanctionResult]) {
        for result in results {
            if let Some(agent) = self.agents.iter_mut().find(|a| a.id == result.violator_id) {
                agent.docas.levels[2] += result.violator_cortisol_delta;
                agent.docas.levels[2] = agent.docas.levels[2].clamp(0.0, 1.0);
            }
            if let Some(punisher_id) = result.punisher_id {
                if let Some(agent) = self.agents.iter_mut().find(|a| a.id == punisher_id) {
                    agent.docas.levels[0] += result.punisher_dopamine_delta;
                    agent.docas.levels[0] = agent.docas.levels[0].clamp(0.0, 1.0);
                }
            }
        }
    }

    pub(crate) fn process_norm_enforcement(&mut self) {
        if !self.config.toggles.norms_enabled || self.tick % 10 != 0 || self.active_norms.is_empty()
        {
            return;
        }

        let norms_clone = self.active_norms.clone();
        let Some(rng) = self.rng.as_mut() else {
            return;
        };

        let violations = Self::find_norm_violations(&self.agents, &norms_clone, rng);
        if violations.is_empty() {
            return;
        }

        let results = enforce_group_norms(&mut self.agents, &norms_clone, &violations, 0.5, rng);
        self.apply_enforcement_deltas(&results);
    }

    pub(crate) fn process_institution_emergence(&mut self) {
        if !self.config.toggles.institutions_enabled || self.tick % 100 != 0 {
            return;
        }
        for (group_id, tracker) in &self.coordination_trackers.clone() {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            let Some(rng) = self.rng.as_mut() else {
                continue;
            };
            check_institution_emergence(tracker, *group_id, self.tick, &mut self.institutions, rng);
        }
        for tracker in self.coordination_trackers.values_mut() {
            tracker.reset(self.tick);
        }
    }

    pub(crate) fn process_institution_evolution(&mut self) {
        if !self.config.toggles.institutions_enabled {
            return;
        }
        const DECAY_RATE: f32 = 0.01;
        if let Some(ref mut rng) = self.rng {
            let _ = evolve_institutions(&mut self.institutions, self.tick, DECAY_RATE, rng);
            // AUDIT-OK(fire-and-forget): in-simulation institution evolution; error only signals nothing-to-evolve
        }
    }

    pub(crate) fn process_group_formation(&mut self) {
        if !self.config.toggles.groups_enabled || self.tick % 50 != 0 {
            return;
        }
        form_groups(&mut self.agents, 5.0, &mut self._next_group_id);
    }

    pub(crate) fn process_identity_theft(&mut self) {
        self.initialize_agent_identities();
        self.process_identity_theft_events();
        self.sync_identity_legacy_fields();
    }

    pub(crate) fn initialize_agent_identities(&mut self) {
        let current_tick = self.tick;
        for agent in &mut self.agents {
            if agent.alive && agent.identity.is_none() {
                let mut identity = Identity::new(agent.id, self.config.seed);
                identity.add_claim(
                    IdentityClaimType::Reputation,
                    agent.reputation_score,
                    current_tick,
                );
                identity.add_claim(
                    IdentityClaimType::Wealth,
                    agent.energy / 100.0,
                    current_tick,
                );
                agent.identity = Some(identity);
            }
        }
    }

    pub(crate) fn process_identity_theft_events(&mut self) {
        let current_tick = self.tick;
        let protocol_enabled = self.config.toggles.protocol_enabled;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut rng = StdRng::seed_from_u64(
            self.config
                .seed
                .wrapping_add(current_tick.wrapping_mul(7919)),
        );
        let mut stats = IdentityTheftStats::default();
        crate::identity::process_identity_theft_population(
            &mut self.agents,
            current_tick,
            protocol_enabled,
            &mut stats,
            &mut rng,
        );
        self.metrics.identity_theft_stats = Some(stats);
    }

    pub(crate) fn sync_identity_legacy_fields(&mut self) {
        for agent in &mut self.agents {
            if let Some(ref identity) = agent.identity {
                agent.is_identity_theft_victim = identity.is_stolen;
                agent.has_verified_id = identity.verification_rate() > 0.5;
                agent.verification_count = identity.verification_count;
                if identity.is_stolen {
                    if let Some(theft_tick) = identity.theft_tick {
                        agent.theft_recovery_tick = Some(theft_tick + 500);
                    }
                } else {
                    agent.theft_recovery_tick = None;
                }
            }
        }
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    /// Process two-agent encounters (SIM-003)
    /// Per SIM-003: 4-step encounter pipeline
    pub(crate) fn process_encounters(&mut self) {
        use crate::encounter::{detect_proximity, process_encounter, PERCEPTION_RADIUS};

        // Reset per-tick metrics
        self.metrics.trade_volume_this_tick = 0.0;
        self.metrics.disputes_this_tick = 0;
        self.metrics.encounters_this_tick = 0;

        // Step 1: Detect agents in proximity
        let encounter_pairs = detect_proximity(&self.agents, PERCEPTION_RADIUS);

        if encounter_pairs.is_empty() {
            // Still record cooperation history even with no encounters
            let coop_rate = self.compute_cooperation_rate();
            self.metrics.cooperation_history.push(coop_rate);
            return;
        }

        // Track encounter outcomes for philosophy updates
        // (agent_slice_idx, outcome, was_cooperative)
        let mut philosophy_updates: Vec<(usize, f32, bool)> = Vec::new();

        // Process each encounter pair
        let current_tick = self.tick;

        for (idx1, idx2) in encounter_pairs {
            // Skip if either agent is dead
            if !self.agents[idx1].alive || !self.agents[idx2].alive {
                continue;
                // TAG: surface=simulation owner=research-team rule=SIM-001
            }

            // Count this encounter
            self.metrics.encounters_this_tick += 1;

            // Process the encounter
            // This requires mutable access to both agents
            // We use split_at_mut to get non-overlapping mutable references
            // Create a temporary RNG from the pre-generated value for determinism
            let mut pair_rng = rand::rngs::StdRng::seed_from_u64(
                (self.config.seed.wrapping_add(current_tick))
                    .wrapping_add(idx1 as u64 * 1000 + idx2 as u64),
            );

            let (result, debt) = if idx1 < idx2 {
                let (left, right) = self.agents.split_at_mut(idx2);
                let agent1 = &mut left[idx1];
                let agent2 = &mut right[0];
                process_encounter(agent1, agent2, current_tick, &mut pair_rng)
            } else {
                let (left, right) = self.agents.split_at_mut(idx1);
                let agent2 = &mut left[idx2];
                let agent1 = &mut right[0];
                process_encounter(agent1, agent2, current_tick, &mut pair_rng)
            };

            // Log encounter result for metrics and track for philosophy updates
            match result {
                crate::encounter::EncounterResult::Avoided => {
                    // Agents avoided each other - slight negative outcome
                    philosophy_updates.push((idx1, -0.1, false));
                    philosophy_updates.push((idx2, -0.1, false));
                }
                // TAG: surface=simulation owner=research-team rule=SIM-001
                crate::encounter::EncounterResult::GameEngagement { .. } => {
                    // Game was played - track debt as trade volume
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    if let Some((_, _, amount)) = debt {
                        self.metrics.trade_volume_this_tick += amount;
                        // Track disputes (defections create debts)
                        self.metrics.disputes_this_tick += 1;

                        // Debt indicates defection - negative outcome for debtor, neutral for creditor
                        philosophy_updates.push((idx1, -0.3, false));
                        philosophy_updates.push((idx2, 0.2, true));
                    } else {
                        // No debt = mutual cooperation
                        philosophy_updates.push((idx1, 0.3, true));
                        philosophy_updates.push((idx2, 0.3, true));
                    }
                }
                crate::encounter::EncounterResult::ReproductionEngagement => {
                    // Reproduction intent signaled - positive cooperative outcome
                    philosophy_updates.push((idx1, 0.4, true));
                    philosophy_updates.push((idx2, 0.4, true));
                }
                crate::encounter::EncounterResult::IdentityTheft { .. } => {
                    // Identity theft handled by encounter module
                    // Track as dispute for metrics
                    self.metrics.disputes_this_tick += 1;
                    // Negative outcome for victim
                    philosophy_updates.push((idx1, -0.5, false));
                    philosophy_updates.push((idx2, -0.5, false));
                }
            }
        }
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Update philosophy based on encounter outcomes in O(1) direct indexing
        for (idx, outcome, was_cooperative) in philosophy_updates {
            if let Some(agent) = self.agents.get_mut(idx) {
                if agent.alive {
                    agent.update_philosophy_from_experience(outcome, was_cooperative);
                }
            }
        }

        // Update cooperation history for POM validation
        let coop_rate = self.compute_cooperation_rate();
        self.metrics.cooperation_history.push(coop_rate);
    }

    /// Compute cooperation rate from agent interactions (POM-SOC-001)
    pub(crate) fn compute_cooperation_rate(&self) -> f32 {
        let (pos, neg): (u32, u32) = self
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| (a.positive_interactions, a.negative_interactions))
            .fold((0, 0), |(p, n), (ap, an)| (p + ap, n + an));

        if pos + neg > 0 {
            pos as f32 / (pos + neg) as f32
        } else {
            0.5 // Neutral default
        }
    }

    /// Regulate system criticality (self-organized criticality)
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Per CRI-001: Maintain σ ≈ 1.0 (critical state)
    /// Uses standardized branching ratio computation from metrics module
    pub(crate) fn regulate_criticality(&mut self) {
        // Compute system-wide branching ratio using standardized function
        let system_sigma = crate::metrics::compute_branching_ratio(
            self.metrics.births_this_tick,
            self.metrics.deaths_this_tick,
            self.metrics.encounters_this_tick,
            self.living_agent_count(),
        ) as f32;

        let target_sigma = if self.config.toggles.dials.criticality_sigma_target > 0.0 {
            self.config.toggles.dials.criticality_sigma_target
        } else {
            1.0f32
        };
        let tolerance = if self.config.toggles.dials.criticality_tolerance > 0.0 {
            self.config.toggles.dials.criticality_tolerance
        } else {
            0.1f32
        };
        let error = system_sigma - target_sigma;

        // Update each agent's criticality_sigma toward system target
        // Agents adapt their activity level based on system state
        for agent in &mut self.agents {
            if !agent.alive {
                continue;
            }

            // Exponential moving average: agent sigma tracks system sigma
            let alpha = 0.1; // Adaptation rate
            agent.criticality_sigma =
                // TAG: surface=simulation owner=research-team rule=SIM-001
                (1.0 - alpha) * agent.criticality_sigma + alpha * system_sigma;

            // Apply homeostatic correction via DOCAS
            // If system is too active (σ > target + tolerance), increase cortisol (inhibition)
            // If system is too quiet (σ < target - tolerance), increase dopamine (activation)
            if error > tolerance {
                // Supercritical: increase inhibition
                agent.docas.levels[2] = (agent.docas.levels[2] + 0.01).min(1.0);
            // Cortisol
            } else if error < -tolerance {
                // Subcritical: increase activation
                agent.docas.levels[0] = (agent.docas.levels[0] + 0.01).min(1.0);
                // Dopamine
            }
        }
    }
}
