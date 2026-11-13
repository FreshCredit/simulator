use crate::types::{AgentId, DebtRecord};

// TAG: surface=simulation owner=research-team rule=SIM-001
impl super::Simulation {
    /// Collect debt settlement decisions using calibrated Ganong & Noel (2023) structural channels
    fn collect_debt_settlements(&self) -> Vec<(AgentId, usize, bool)> {
        let mut debt_settlements = Vec::new();
        for agent in &self.agents {
            if !agent.alive {
                continue;
            }
            let reputation = (agent.reputation_score + 1.0) / 2.0;

            for (debt_idx, debt) in agent.debt_obligations.iter().enumerate() {
                if debt.is_fully_paid() {
                    continue;
                }
                let debt_amount = debt.remaining_amount();
                let reputation_value =
                    10.0 * (reputation + 0.5) * (1.0 + agent.economics.bond_density);
                let delay_ticks: u32 = debt
                    .tick_due
                    .map_or(10, |due| due.saturating_sub(self.tick) as u32);

                // Calibrated Capacity Confound & Default Triggers (Ganong & Noel 2023):
                // 1. Liquidity Shock (~70% share): Insufficient cash-flow buffer
                let liquidity_shock = agent.economics.cashflow_buffer_days < 1.0
                    || agent.economics.wealth < debt_amount * 0.5;
                // 2. Double-Trigger (~24% share): Severe debt burden + depleted cushion
                let double_trigger =
                    agent.economics.debt_burden > 1.2 && agent.economics.cashflow_buffer_days < 3.0;
                // 3. Strategic Default (~6% share): Hyperbolic discounting expectation mismatch
                let strategic_default =
                    agent.decide_debt_default(debt_amount, reputation_value, delay_ticks);

                let should_default = liquidity_shock || double_trigger || strategic_default;
                debt_settlements.push((agent.id, debt_idx, !should_default));
            }
        }
        debt_settlements
    }

    /// Apply debt settlement outcomes (repayment vs default) to agents
    fn apply_debt_settlements(&mut self, debt_settlements: Vec<(AgentId, usize, bool)>) {
        let cost = self.config.transaction_cost;
        for (agent_id, debt_idx, will_pay) in debt_settlements {
            if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
                if will_pay {
                    if let Some(debt) = agent.debt_obligations.get_mut(debt_idx) {
                        let payment = debt.remaining_amount().min(agent.economics.wealth);
                        debt.record_payment(payment);
                        agent.economics.wealth = (agent.economics.wealth - payment - cost).max(0.0);
                        agent.economics.record_commitment(true);
                        agent.economics.on_time_payment_rate =
                            (agent.economics.on_time_payment_rate * 0.95 + 0.05).min(1.0);
                        agent.economics.market_trust_score =
                            (agent.economics.market_trust_score + 0.02).min(1.0);
                        agent.record_positive_interaction();
                        agent.docas.levels[4] = (agent.docas.levels[4] + 0.05).min(1.0);
                        agent.docas.levels[0] = (agent.docas.levels[0] + 0.03).min(1.0);
                    }
                } else {
                    agent.economics.record_commitment(false);
                    agent.economics.nsf_overdraft_count += 1;
                    agent.economics.on_time_payment_rate =
                        (agent.economics.on_time_payment_rate * 0.90).max(0.0);
                    agent.economics.market_trust_score =
                        (agent.economics.market_trust_score - 0.08).max(0.0);
                    agent.record_negative_interaction();
                    agent.economics.actual_default = true;
                    agent.docas.levels[2] = (agent.docas.levels[2] + 0.1).min(1.0);
                    agent.docas.levels[0] = (agent.docas.levels[0] - 0.05).max(0.0);
                }
            }
        }
    }

    /// Apply economic credit, debt burden, innovation, and cloaking updates across all living agents
    fn update_agent_economics(&mut self) {
        use crate::economics::compute_credit_access;
        let total_agents = self.agents.len();
        let min_rep = self.config.reputation_threshold;
        let credit_limit = self.config.credit_limit;
        let weather_enabled = self.config.environment.weather_enabled;
        let weather_shield = self.config.weather_shield_enabled;
        let weather_thermal = self.environment.weather.thermal_stress_factor();

        for agent in &mut self.agents {
            if !agent.alive {
                continue;
            }

            let reputation = (agent.reputation_score + 1.0) / 2.0;

            // Plaid liquidity buffer in days of consumption
            let burn_rate = (agent.economics.consumption * 1.5).max(0.1);
            agent.economics.cashflow_buffer_days =
                (agent.economics.wealth / burn_rate).clamp(0.0, 100.0);

            // Dual-layer Concordance Statistic: correlation between biological trust and financial market score
            let bio_tau = (agent.compute_trust() / 2.0).clamp(0.0, 1.0);
            let diff = (bio_tau - agent.economics.market_trust_score).abs();
            agent.economics.concordance_mantel_r = (1.0 - diff).clamp(0.0, 1.0);

            // Strict Sequestration Wall: Credit access is evaluated from market_trust_score and bond density,
            // never directly conditioning on internal biological DOCAS or wearable biomarkers.
            let raw_access = compute_credit_access(
                agent.economics.market_trust_score,
                agent.economics.bond_density,
                reputation,
            );
            let effective_access = if reputation >= min_rep {
                raw_access * (credit_limit / 100.0).max(0.1)
            } else {
                0.0
            };

            // Civilization Challenge: Bank run sunspot panic dampens credit availability
            let bank_run_penalty =
                if self.config.bank_run_sensitivity > 0.0 && self.metrics.disputes_this_tick > 0 {
                    (1.0 - (self.metrics.disputes_this_tick as f32
                        * 0.05
                        * self.config.bank_run_sensitivity))
                        .max(0.1)
                } else {
                    1.0
                };
            agent.economics.credit_access = effective_access * bank_run_penalty;

            // Apply base interest rate to existing debts
            if self.config.interest_rate_base > 0.0 {
                let interest = self.config.interest_rate_base * 0.01;
                for debt in &mut agent.debt_obligations {
                    if !debt.is_fully_paid() {
                        debt.amount += debt.remaining_amount() * interest;
                    }
                }
            }

            let total_debt: f32 = agent
                .debt_obligations
                .iter()
                .map(DebtRecord::remaining_amount)
                .sum();
            let income = agent.economics.wealth.max(1.0);
            agent.economics.debt_burden = total_debt / income;

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let stress = agent.economics.debt_stress();
            if stress > 0.5 {
                agent.docas.levels[2] = (agent.docas.levels[2] + stress * 0.05).min(1.0);
            }

            let innovation_input =
                agent.economics.market_trust_score * (1.0 - agent.economics.debt_burden.min(1.0));
            agent.economics.innovation_contribution =
                (0.9 * agent.economics.innovation_contribution + 0.1 * innovation_input)
                    .clamp(0.0, 1.0);

            let total_interactions = agent.positive_interactions + agent.negative_interactions;
            if total_interactions > 0 {
                let honor_rate = agent.positive_interactions as f32 / total_interactions as f32;
                agent.economics.record_commitment(honor_rate > 0.5);
            }

            agent.update_time_preference();

            let weather_mod = if weather_enabled {
                self.environment
                    .weather
                    .cognitive_bandwidth_weather_modifier()
            } else {
                1.0
            };
            let decile = ((agent.cognitive_bandwidth.clamp(0.0, 1.0) * 10.0).floor() as usize + 1)
                .clamp(1, 10);
            let is_perturbed =
                self.config
                    .optogenetics
                    .is_agent_perturbed(agent.id, decile, total_agents);
            let opto_stress_mod = if is_perturbed {
                self.config
                    .optogenetics
                    .modulate_cognitive_bandwidth(1.0, true)
            } else {
                1.0
            };
            let eff_bandwidth = agent.effective_bandwidth(weather_mod, opto_stress_mod);
            let pseudo_r =
                ((agent.id as f32 * std::f32::consts::PI + self.tick as f32 * 0.1).fract()).abs();
            let u_res = 0.10 + 0.04 * (reputation - 0.5);
            let predatory_stimulus = if weather_shield && weather_thermal > 1.30 {
                crate::visual_cloaking::VisualAgreementStimulus::fair_benchmark()
            } else {
                self.config.visual_cloaking.clone()
            };
            let accepted_cloaked =
                predatory_stimulus.evaluate_agent_decision(eff_bandwidth, u_res, pseudo_r);
            if accepted_cloaked && predatory_stimulus.cloaking_ratio > 1.5 {
                agent.docas.levels[crate::types::C] =
                    (agent.docas.levels[crate::types::C] + 0.08).min(1.0);
            }
        }
    }

    /// Process economics: credit access, innovation, debt burden updates
    /// Per economics.md: Credit-Innovation feedback loop
    pub(super) fn process_economics(&mut self) {
        let debt_settlements = self.collect_debt_settlements();
        self.apply_debt_settlements(debt_settlements);
        self.update_agent_economics();

        // Civilization Challenge: Wealth redistribution tax
        if self.config.tax_redistribution_rate > 0.0 {
            let tax_rate = self.config.tax_redistribution_rate.clamp(0.0, 0.50);
            let mut collected_tax = 0.0f32;
            for agent in &mut self.agents {
                if agent.alive && agent.economics.wealth > 20.0 {
                    let tax = (agent.economics.wealth - 20.0) * tax_rate;
                    agent.economics.wealth -= tax;
                    collected_tax += tax;
                }
            }
            let poor_count = self
                .agents
                .iter()
                .filter(|a| a.alive && a.energy < 40.0)
                .count();
            if poor_count > 0 && collected_tax > 0.0 {
                let subsidy = collected_tax / poor_count as f32;
                for agent in &mut self.agents {
                    if agent.alive && agent.energy < 40.0 {
                        agent.gain_energy(subsidy);
                        agent.economics.wealth += subsidy * 0.5;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // This file contains only pub(super) methods on Simulation.
    // No standalone pure functions exist for unit testing without
    // instantiating the full Simulation state.
}
