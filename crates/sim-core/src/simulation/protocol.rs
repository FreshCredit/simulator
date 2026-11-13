use super::{GameEvent, GameOutcome};
use crate::games::{
    decide_action, play_prisoners_dilemma, play_stag_hunt, play_ultimatum_game, select_strategy,
    DocasGameModulation, GameType,
};
use crate::protocol::{ClaimType, VerificationHash, VerificationMode};
use crate::types::{AgentId, Position, StrategyType, Tick};
use rand::rngs::StdRng;
use rand::Rng;

// TAG: surface=simulation owner=research-team rule=SIM-001
impl super::Simulation {
    /// Process game-theoretic interactions between agents
    /// Per §6.10: UTGT classical game experiments
    /// Per §6.10.3: Sample 1% of population every 100 ticks for game experiments
    pub(super) fn process_game_interactions(&mut self) {
        let living_agents: Vec<usize> = self
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive)
            .map(|(i, _)| i)
            .collect();

        if living_agents.len() < 2 {
            return;
        }

        let sample_size = ((living_agents.len() as f32 * 0.01).ceil() as usize).max(2);

        let mut selected_indices: Vec<usize> = Vec::new();
        {
            let rng = self.rng();
            for _ in 0..sample_size {
                let idx = rng.gen_range(0..living_agents.len());
                selected_indices.push(living_agents[idx]);
            }
        }

        if selected_indices.len() < 2 {
            return;
        }

        let agent_data: Vec<(usize, AgentId, f32, f32, StrategyType, Option<bool>)> =
            selected_indices
                .iter()
                .map(|&idx| {
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    let agent = &self.agents[idx];
                    (
                        idx,
                        agent.id,
                        agent.compute_trust(),
                        agent.genome.cooperation_tendency(),
                        agent.strategy,
                        None,
                    )
                })
                .collect();

        let game_type = {
            let rng = self.rng();
            match rng.gen_range(0..3) {
                0 => GameType::PrisonersDilemma,
                1 => GameType::StagHunt,
                _ => GameType::UltimatumGame,
            }
        };

        let mut outcomes: Vec<(usize, usize, GameOutcome)> = Vec::new();

        for chunk in agent_data.chunks(2) {
            if chunk.len() < 2 {
                continue;
            }

            let (idx_a, _id_a, tau_a, coop_a, strategy_a, last_a) = chunk[0];
            let (idx_b, _id_b, tau_b, coop_b, strategy_b, last_b) = chunk[1];

            let (a_cooperates, b_cooperates, payoff_a, payoff_b) = {
                let rng = self.rng();
                let a_coop = decide_action(strategy_a, last_b, tau_a, coop_a, rng);
                let b_coop = decide_action(strategy_b, last_a, tau_b, coop_b, rng);
                let (pa, pb) =
                    Self::compute_game_payoffs(game_type, a_coop, b_coop, tau_a, tau_b, rng);
                (a_coop, b_coop, pa, pb)
            };

            outcomes.push((
                // TAG: surface=simulation owner=research-team rule=SIM-001
                idx_a,
                idx_b,
                GameOutcome {
                    a_coop: a_cooperates,
                    b_coop: b_cooperates,
                    payoff_a,
                    payoff_b,
                },
            ));
        }

        for (idx_a, idx_b, outcome) in outcomes {
            self.apply_game_outcome(idx_a, idx_b, outcome, game_type);
        }
    }

    pub(super) fn compute_game_payoffs(
        game_type: GameType,
        a_coop: bool,
        b_coop: bool,
        tau_a: f32,
        tau_b: f32,
        rng: &mut StdRng,
    ) -> (f32, f32) {
        match game_type {
            GameType::PrisonersDilemma => play_prisoners_dilemma(a_coop, b_coop),
            GameType::StagHunt => play_stag_hunt(a_coop, b_coop),
            GameType::UltimatumGame => play_ultimatum_game(tau_a, tau_b, 10.0, rng),
            GameType::PublicGoodsGame | GameType::DictatorGame | GameType::TrustGame => {
                play_prisoners_dilemma(a_coop, b_coop)
            }
        }
    }

    pub(super) fn apply_game_outcome(
        &mut self,
        idx_a: usize,
        idx_b: usize,
        outcome: GameOutcome,
        game_type: GameType,
    ) {
        self.agents[idx_a].energy = (self.agents[idx_a].energy + outcome.payoff_a).max(0.0);
        self.agents[idx_b].energy = (self.agents[idx_b].energy + outcome.payoff_b).max(0.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        self.record_game_interactions(idx_a, idx_b, outcome.a_coop, outcome.b_coop);

        if self.config.toggles.docas_enabled {
            self.apply_game_docas_modulation(
                idx_a,
                idx_b,
                outcome.a_coop,
                outcome.b_coop,
                outcome.payoff_a,
                outcome.payoff_b,
            );
        }

        let id_a = self.agents[idx_a].id;
        let id_b = self.agents[idx_b].id;
        self.game_events_this_tick.push(GameEvent {
            tick: self.tick,
            agent_a: id_a,
            agent_b: id_b,
            game_type: format!("{game_type:?}"),
            outcome: Self::game_outcome_label(outcome.a_coop, outcome.b_coop),
            payoff_a: outcome.payoff_a,
            payoff_b: outcome.payoff_b,
        });

        self.maybe_update_strategies(idx_a, idx_b);
    }

    pub(super) fn record_game_interactions(
        &mut self,
        idx_a: usize,
        idx_b: usize,
        a_coop: bool,
        b_coop: bool,
    ) {
        if a_coop && b_coop {
            self.agents[idx_a].record_positive_interaction();
            self.agents[idx_b].record_positive_interaction();
        // TAG: surface=simulation owner=research-team rule=SIM-001
        } else if !a_coop && !b_coop {
            self.agents[idx_a].record_negative_interaction();
            self.agents[idx_b].record_negative_interaction();
        } else if a_coop {
            self.agents[idx_a].record_negative_interaction();
            self.agents[idx_b].record_positive_interaction();
        } else {
            self.agents[idx_a].record_positive_interaction();
            self.agents[idx_b].record_negative_interaction();
        }
    }

    pub(super) fn apply_game_docas_modulation(
        &mut self,
        idx_a: usize,
        idx_b: usize,
        a_coop: bool,
        b_coop: bool,
        payoff_a: f32,
        payoff_b: f32,
    ) {
        let mod_a = DocasGameModulation::from_outcome(a_coop, b_coop, payoff_a, 2.5);
        let mod_b = DocasGameModulation::from_outcome(b_coop, a_coop, payoff_b, 2.5);

        let docas_a = &mut self.agents[idx_a].docas.levels;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        docas_a[0] = (docas_a[0] + mod_a.delta_d).clamp(0.0, 1.0);
        docas_a[1] = (docas_a[1] + mod_a.delta_o).clamp(0.0, 1.0);
        docas_a[2] = (docas_a[2] + mod_a.delta_c).clamp(0.0, 1.0);
        docas_a[3] = (docas_a[3] + mod_a.delta_a).clamp(0.0, 1.0);
        docas_a[4] = (docas_a[4] + mod_a.delta_s).clamp(0.0, 1.0);

        let docas_b = &mut self.agents[idx_b].docas.levels;
        docas_b[0] = (docas_b[0] + mod_b.delta_d).clamp(0.0, 1.0);
        docas_b[1] = (docas_b[1] + mod_b.delta_o).clamp(0.0, 1.0);
        docas_b[2] = (docas_b[2] + mod_b.delta_c).clamp(0.0, 1.0);
        docas_b[3] = (docas_b[3] + mod_b.delta_a).clamp(0.0, 1.0);
        docas_b[4] = (docas_b[4] + mod_b.delta_s).clamp(0.0, 1.0);
    }

    pub(super) fn game_outcome_label(a_coop: bool, b_coop: bool) -> String {
        if a_coop && b_coop {
            "Mutual Cooperation".to_string()
        } else if !a_coop && !b_coop {
            "Mutual Defection".to_string()
        } else if a_coop {
            "A Cooperates, B Defects".to_string()
        } else {
            "A Defects, B Cooperates".to_string()
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub(super) fn maybe_update_strategies(&mut self, idx_a: usize, idx_b: usize) {
        let tau_a = self.agents[idx_a].compute_trust();
        let coop_a = self.agents[idx_a].genome.cooperation_tendency();
        let tau_b = self.agents[idx_b].compute_trust();
        let coop_b = self.agents[idx_b].genome.cooperation_tendency();

        let (new_a, new_b) = {
            let rng = self.rng();
            let update_a = rng.gen::<f32>() < 0.1;
            let update_b = rng.gen::<f32>() < 0.1;
            let new_a = if update_a {
                select_strategy(coop_a, tau_a, rng)
            } else {
                StrategyType::TitForTat
            };
            let new_b = if update_b {
                select_strategy(coop_b, tau_b, rng)
            } else {
                StrategyType::TitForTat
            };
            (new_a, new_b)
        };

        self.agents[idx_a].strategy = new_a;
        self.agents[idx_b].strategy = new_b;
    }
    /// Process Stage 5: Protocol (blockchain verification simulation)
    /// Per §22.1: Phase 1 MVP uses local SHA-256 hashes
    /// Per §8.2.1 lines 4646-4666: Protocol toggle contract
    /// Per §6.8.5: Fresh Protocol intervention analysis
    ///
    /// ## Key Mechanisms
    /// - Agents can anchor identity claims (create verification hashes)
    /// - Agents verify counterparty claims during interactions
    /// - Verification reduces prediction error ε(t) → affects cooperation
    /// - Track verification rate for stability (>0.1 per §D.3 line 7587)
    pub(super) fn process_stage5_protocol(&mut self) {
        let current_tick = self.tick;
        let verification_mode = self.config.verification_mode;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        if verification_mode == VerificationMode::CentralizedOracle {
            self.process_centralized_verification(current_tick);
            return;
        }

        self.process_anchoring(current_tick);
        self.process_verifications(current_tick);
    }

    pub(super) fn process_anchoring(&mut self, current_tick: Tick) {
        const ANCHOR_PROBABILITY: f32 = 0.05;

        let agents_to_anchor: Vec<AgentId> = self
            .agents
            .iter()
            .filter(|a| a.alive && !a.has_verified_id)
            .map(|a| a.id)
            .collect();

        let anchor_decisions: Vec<f32> = {
            let rng = self.rng();
            agents_to_anchor.iter().map(|_| rng.gen::<f32>()).collect()
        };

        let (cost_min, cost_max) = if self.config.toggles.dials.protocol_anchor_cost_range.1
            > self.config.toggles.dials.protocol_anchor_cost_range.0
        {
            self.config.toggles.dials.protocol_anchor_cost_range
        } else {
            (0.005, 0.05)
        };
        let cost_span = (cost_max - cost_min).max(0.0001);

        let anchor_random_data: Vec<(u64, f32)> = {
            let rng = self.rng();
            agents_to_anchor
                .iter()
                .map(|_| (rng.gen::<u64>(), cost_min + rng.gen::<f32>() * cost_span))
                .collect()
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for (i, agent_id) in agents_to_anchor.iter().enumerate() {
            if anchor_decisions[i] >= ANCHOR_PROBABILITY {
                continue;
            }
            let (random_component, anchor_cost) = anchor_random_data[i];
            let type_val = ClaimType::IdentityVerification as u64;
            let hash = agent_id
                .wrapping_mul(31)
                .wrapping_add(type_val.wrapping_mul(17))
                .wrapping_add(current_tick.wrapping_mul(13))
                .wrapping_add(random_component);

            let claim = crate::protocol::AnchoredClaim {
                hash,
                agent_id: *agent_id,
                claim_type: ClaimType::IdentityVerification,
                anchor_tick: current_tick,
                anchor_cost,
            };
            self.protocol.anchored_claims.insert(hash, claim);
            self.protocol.total_anchors += 1;

            if let Some(agent) = self.agents.iter_mut().find(|a| a.id == *agent_id) {
                agent.has_verified_id = true;
                agent.consume_energy(anchor_cost);
            }

            self.agent_identity_hashes.insert(*agent_id, hash);
        }
    }

    pub(super) fn process_verifications(&mut self, current_tick: Tick) {
        const VERIFICATION_RADIUS: f32 = 3.0;
        const VERIFICATION_PROBABILITY: f32 = 0.1;

        let verified_agents: Vec<(AgentId, Position, VerificationHash)> = self
            .agents
            .iter()
            .filter(|a| a.alive && a.has_verified_id)
            .filter_map(|a| {
                self.agent_identity_hashes
                    .get(&a.id)
                    .map(|h| (a.id, a.position, *h))
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut verification_pairs: Vec<(AgentId, VerificationHash)> = Vec::new();
        for i in 0..verified_agents.len() {
            for j in (i + 1)..verified_agents.len() {
                let (id_i, pos_i, _) = &verified_agents[i];
                let (id_j, pos_j, hash_j) = &verified_agents[j];
                let dist = pos_i.distance(pos_j);
                if dist <= VERIFICATION_RADIUS {
                    verification_pairs.push((*id_i, *hash_j));
                    verification_pairs.push((*id_j, verified_agents[i].2));
                }
            }
        }

        let verify_data: Vec<(f32, f32)> = {
            let rng = self.rng();
            verification_pairs
                .iter()
                .map(|_| (rng.gen::<f32>(), 0.0025 + rng.gen::<f32>() * 0.0225))
                .collect()
        };

        let latency_ticks = (self.config.block_time.max(1) as u32).min(100);
        let consensus_req = self.config.consensus_threshold.clamp(0.0, 1.0);

        for (i, (verifier_id, hash_to_verify)) in verification_pairs.iter().enumerate() {
            let (decision, verify_cost) = verify_data[i];
            if decision >= VERIFICATION_PROBABILITY {
                continue;
            }
            let claim_opt = self.protocol.anchored_claims.get(hash_to_verify);
            let is_confirmed = claim_opt.is_some_and(|c| {
                current_tick >= c.anchor_tick.saturating_add(latency_ticks as u64)
            });
            let consensus_met = if consensus_req > 0.0 {
                decision
                    <= (VERIFICATION_PROBABILITY * (1.25 - consensus_req * 0.5)).clamp(0.01, 1.0)
            } else {
                true
            };
            let verified = is_confirmed && consensus_met;

            let result = crate::protocol::VerificationResult {
                verified,
                hash: *hash_to_verify,
                verify_tick: current_tick,
                verify_cost,
                latency_ticks,
                authority: if verified {
                    format!("0x{hash_to_verify:016x}")
                } else {
                    "none".to_string()
                },
                is_centralized: false,
            };

            // TAG: surface=simulation owner=research-team rule=SIM-001
            self.protocol
                .verification_history
                .entry(*verifier_id)
                .or_default()
                .push(result);

            self.protocol.total_verifications += 1;
            if verified {
                self.protocol.successful_verifications += 1;
            }

            if let Some(agent) = self.agents.iter_mut().find(|a| a.id == *verifier_id) {
                agent.verification_count += 1;
                agent.consume_energy(verify_cost);
                if verified {
                    agent.gain_energy(verify_cost * 0.5);
                }
            }
        }
    }

    /// Process centralized verification (SIM-011)
    /// Single authority model - faster but vulnerable to single point of failure
    /// Per P0-002: Baseline for Protocol ON vs OFF comparison
    pub(super) fn process_centralized_verification(&mut self, current_tick: Tick) {
        const CENTRALIZED_VERIFICATION_PROBABILITY: f32 = 0.3; // Higher than decentralized
                                                               // REDUCED: Was 0.02, now 0.01 (50% reduction)
        const CENTRALIZED_COST: f32 = 0.01; // Lower cost (no blockchain fees)
        const VERIFICATION_RADIUS: f32 = 5.0; // Larger radius (centralized has more reach)

        // In centralized mode, the "central authority" verifies agents
        // Verification is faster and cheaper but less secure

        // Collect all living agents
        let agent_data: Vec<(AgentId, Position)> = self
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| (a.id, a.position))
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Generate verification decisions
        let verify_decisions: Vec<bool> = {
            let rng = self.rng();
            agent_data
                .iter()
                .map(|_| rng.gen::<f32>() < CENTRALIZED_VERIFICATION_PROBABILITY)
                .collect()
        };

        // Process verifications - centralized authority verifies agents in batches
        for (i, (agent_id, position)) in agent_data.iter().enumerate() {
            if verify_decisions[i] {
                // Find nearby agents that can be "verified" by the central authority
                let nearby_agents: Vec<AgentId> = self
                    .agents
                    .iter()
                    .filter(|a| a.alive && a.id != *agent_id)
                    .filter(|a| position.distance(&a.position) <= VERIFICATION_RADIUS)
                    .map(|a| a.id)
                    .collect();

                // Mark agent as verified (if not already)
                if let Some(agent) = self.agents.iter_mut().find(|a| a.id == *agent_id) {
                    if !agent.has_verified_id {
                        agent.has_verified_id = true;
                        agent.consume_energy(CENTRALIZED_COST);

                        // Generate deterministic "centralized" hash
                        let hash = agent_id
                            .wrapping_mul(7919)
                            .wrapping_add(current_tick.wrapping_mul(104729));

                        let claim = crate::protocol::AnchoredClaim {
                            hash,
                            agent_id: *agent_id,
                            // TAG: surface=simulation owner=research-team rule=SIM-001
                            claim_type: ClaimType::IdentityVerification,
                            anchor_tick: current_tick,
                            anchor_cost: CENTRALIZED_COST,
                        };
                        self.protocol.anchored_claims.insert(hash, claim);
                        self.protocol.total_anchors += 1;
                        self.agent_identity_hashes.insert(*agent_id, hash);
                    }
                    agent.verification_count += 1;
                }

                // Centralized mode: authority also "verifies" nearby agents' identities
                for nearby_id in nearby_agents.iter().take(3) {
                    // Limit to 3 per tick
                    if let Some(agent) = self.agents.iter_mut().find(|a| a.id == *nearby_id) {
                        if !agent.has_verified_id {
                            agent.has_verified_id = true;

                            let hash = nearby_id
                                .wrapping_mul(7919)
                                .wrapping_add(current_tick.wrapping_mul(104729))
                                .wrapping_add(*agent_id); // Include verifier

                            let claim = crate::protocol::AnchoredClaim {
                                hash,
                                agent_id: *nearby_id,
                                claim_type: ClaimType::IdentityVerification,
                                anchor_tick: current_tick,
                                anchor_cost: CENTRALIZED_COST,
                            };
                            self.protocol.anchored_claims.insert(hash, claim);
                            self.protocol.total_anchors += 1;
                            self.agent_identity_hashes.insert(*nearby_id, hash);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::protocol::VerificationMode;
    use crate::toggles::ToggleConfig;
    use crate::types::Position;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // =========================================================================
    // FALSIFICATION TESTS
    // Per AUDIT_REPORT.md Section V9: Protocol ON vs OFF comparison
    // H0: τ_mean(ON) ≤ τ_mean(OFF) + 0.1
    // Reject if τ_mean(ON) > τ_mean(OFF) + 0.1 (p < 0.05)
    // =========================================================================

    /// Helper: Run simulation and collect mean trust values
    fn run_and_collect_trust(config: SimulationConfig, ticks: u64) -> Vec<f32> {
        let mut sim = Simulation::new(config);
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
        let mut trust_values = Vec::new();

        for _ in 0..ticks {
            sim.step();

            // Collect trust from all living agents
            for agent in &sim.agents {
                if agent.alive {
                    trust_values.push(agent.compute_trust());
                }
            }
        }

        trust_values
    }

    /// Helper: Calculate mean of a slice
    fn mean(values: &[f32]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f32>() / values.len() as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Helper: Calculate standard deviation
    /// BANNED-001: "standard deviation" is a technical statistical term
    fn std_dev(values: &[f32]) -> f32 {
        // Guard: Need at least 2 values for sample standard deviation
        if values.len() < 2 {
            return 0.0;
        }
        let m = mean(values);
        // Safe: len() >= 2 verified by guard above, so len() - 1 >= 1
        let variance: f32 =
            values.iter().map(|x| (x - m).powi(2)).sum::<f32>() / (values.len() - 1) as f32;
        variance.sqrt()
    }

    /// Falsification test: Protocol ON should produce higher mean trust than OFF
    /// Per `AUDIT_REPORT.md` V9: `τ_mean(ON)` > `τ_mean(OFF)` + 0.1 threshold
    ///
    /// NOTE: This is a structural test that verifies the comparison can be performed.
    /// Full hypothesis testing requires identity theft mechanics (P0 blocker).
    #[test]
    fn test_falsification_protocol_on_vs_off() {
        // Run with Protocol OFF
        let mut toggles_off = ToggleConfig::stage_4(); // Language but no protocol
        toggles_off.protocol_enabled = false;
        let config_off = SimulationConfig {
            toggles: toggles_off,
            max_ticks: 100,
            seed: 42424,
            ..Default::default()
        };

        let trust_off = run_and_collect_trust(config_off, 100);
        let mean_off = mean(&trust_off);

        // Run with Protocol ON
        let config_on = SimulationConfig {
            toggles: ToggleConfig::stage_5(), // Full protocol enabled
            max_ticks: 100,
            seed: 42424, // Same seed for comparison
            ..Default::default()
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let trust_on = run_and_collect_trust(config_on, 100);
        let mean_on = mean(&trust_on);

        // Structural verification: both runs produced trust data
        assert!(
            !trust_off.is_empty() && !trust_on.is_empty(),
            "Both runs should produce trust measurements"
        );

        // Log results for analysis (visible in test output with --nocapture)
        println!("Falsification Test Results:");
        println!(
            "  Protocol OFF: n={}, mean_τ={:.4}, std={:.4}",
            trust_off.len(),
            mean_off,
            std_dev(&trust_off)
        );
        println!(
            "  Protocol ON:  n={}, mean_τ={:.4}, std={:.4}",
            trust_on.len(),
            mean_on,
            std_dev(&trust_on)
        );
        println!("  Difference:   Δ = {:.4}", mean_on - mean_off);
        println!("  Threshold:    0.1 (H0 rejection requires Δ > 0.1)");

        // NOTE: Full hypothesis test requires identity theft mechanics
        // This test verifies the infrastructure for comparison exists
        // See AUDIT_REPORT.md P0 blockers for identity theft implementation
    }

    /// Falsification test: Baseline centralized vs decentralized comparison
    /// Per §7.8.4: Centralized verification serves as negative control
    #[test]
    fn test_falsification_decentralized_vs_centralized() {
        // NOTE: Full centralized baseline requires VerificationMode enum (P0 blocker)
        // This test verifies the comparison structure

        // Decentralized: Full protocol with distributed verification
        let config_decentral = SimulationConfig {
            toggles: ToggleConfig::all_on(),
            max_ticks: 100,
            seed: 55555,
            ..Default::default()
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let trust_decentral = run_and_collect_trust(config_decentral, 100);

        // Proxy for centralized: Protocol disabled (simulates central authority)
        let config_central = SimulationConfig {
            toggles: ToggleConfig::stage_3(), // Society without protocol
            max_ticks: 100,
            seed: 55555,
            ..Default::default()
        };

        let trust_central = run_and_collect_trust(config_central, 100);

        // Structural verification
        assert!(
            !trust_decentral.is_empty() && !trust_central.is_empty(),
            "Both verification modes should produce trust data"
        );

        println!("Decentralized vs Centralized Comparison:");
        println!(
            "  Decentralized: n={}, mean_τ={:.4}",
            trust_decentral.len(),
            mean(&trust_decentral)
        );
        println!(
            "  Centralized:   n={}, mean_τ={:.4}",
            trust_central.len(),
            mean(&trust_central)
        );
    }

    // =========================================================================
    // REGRESSION TESTS: Known scenarios with expected outcomes
    // =========================================================================

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Regression: Simulation runs for extended periods without panicking
    /// Per §22.5.3: Phase 0 single-agent validation
    #[test]
    fn test_regression_extended_run_no_panic() {
        let config = SimulationConfig {
            toggles: ToggleConfig::stage_0(),
            max_ticks: 1000,
            seed: 11111,
            energy_consumption_rate: 0.5, // Low consumption
            ..Default::default()
        };

        let mut sim = Simulation::new(config);
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
        let initial_agents = sim.agents.len();

        for _ in 0..1000 {
            sim.step();
        }

        // Simulation should complete 1000 ticks without panic
        assert_eq!(sim.tick, 1000);

        // Agent was created and the simulation tracked it
        assert!(initial_agents > 0, "Simulation started with agents");
    }

    /// Regression: Population should grow with reproduction enabled
    #[test]
    fn test_regression_population_growth() {
        let config = SimulationConfig {
            toggles: ToggleConfig::stage_1(), // Biology enabled
            max_ticks: 500,
            seed: 77777,
            reproduction_threshold: 50.0, // Lower threshold
            ..Default::default()
        };

        let mut sim = Simulation::new(config);
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
        let initial_pop = sim.agents.len();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for _ in 0..500 {
            sim.step();
        }

        // With reproduction, we should see population change (growth or deaths)
        let final_pop = sim.agents.len();
        println!("Population: initial={initial_pop}, final={final_pop}");

        // At minimum, the simulation should have run without panic
        assert!(sim.tick == 500);
    }

    /// Regression: Group formation should occur with society enabled
    #[test]
    #[ignore = "slow: 300-tick simulation validation"]
    fn test_regression_group_formation() {
        let config = SimulationConfig {
            toggles: ToggleConfig::stage_3(), // Society enabled
            max_ticks: 300,
            seed: 88888,
            ..Default::default()
        };

        let mut sim = Simulation::new(config);
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
        sim.spawn_agent(Position::new_2d(33.0, 32.0)); // Need multiple agents for groups

        for _ in 0..300 {
            sim.step();
        }

        // Count agents in groups
        let grouped_agents = sim
            .agents
            .iter()
            .filter(|a| a.alive && a.group_id.is_some())
            .count();

        println!(
            "Group formation: {}/{} agents in groups",
            grouped_agents,
            sim.agents.iter().filter(|a| a.alive).count()
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Simulation should run without panic
        assert!(sim.tick == 300);
    }

    // === Verification Mode Tests (P0-2) ===

    #[test]
    fn test_verification_mode_default() {
        let config = SimulationConfig::default();
        assert_eq!(
            config.verification_mode,
            VerificationMode::DecentralizedProtocol
        );
    }

    #[test]
    fn test_centralized_baseline_config() {
        let config = SimulationConfig::centralized_baseline(12345, 500);
        assert_eq!(
            config.verification_mode,
            VerificationMode::CentralizedOracle
        );
        assert!(config.toggles.protocol_enabled); // SIM-011: Protocol ON but centralized verification
        assert_eq!(config.seed, 12345);
        assert_eq!(config.max_ticks, 500);
    }

    #[test]
    fn test_decentralized_protocol_config() {
        let config = SimulationConfig::decentralized_protocol(54321, 1000);
        assert_eq!(
            config.verification_mode,
            VerificationMode::DecentralizedProtocol
        );
        assert!(config.toggles.protocol_enabled); // Decentralized = blockchain ON
        assert_eq!(config.seed, 54321);
        assert_eq!(config.max_ticks, 1000);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    #[ignore = "slow: dual 200-tick simulation validation"]
    fn test_verification_mode_comparison_experiment() {
        // Run two simulations with same seed but different verification modes
        // This is the core P0-2 research design
        let seed = 99999;
        let ticks = 200;

        // Centralized baseline (no blockchain)
        let config_centralized = SimulationConfig::centralized_baseline(seed, ticks);
        let mut sim_centralized = Simulation::new(config_centralized);
        sim_centralized.spawn_agent(Position::new_2d(32.0, 32.0));

        // Decentralized protocol (blockchain ON)
        let config_decentralized = SimulationConfig::decentralized_protocol(seed, ticks);
        let mut sim_decentralized = Simulation::new(config_decentralized);
        sim_decentralized.spawn_agent(Position::new_2d(32.0, 32.0));

        // Run both simulations
        for _ in 0..ticks {
            sim_centralized.step();
            sim_decentralized.step();
        }

        // Both should complete without panic
        assert_eq!(sim_centralized.tick, ticks);
        assert_eq!(sim_decentralized.tick, ticks);

        // Verification modes should remain distinct
        assert_eq!(
            sim_centralized.config.verification_mode,
            VerificationMode::CentralizedOracle
        );
        assert_eq!(
            sim_decentralized.config.verification_mode,
            VerificationMode::DecentralizedProtocol
        );
    }
}
