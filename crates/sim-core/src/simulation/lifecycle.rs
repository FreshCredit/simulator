// TAG: surface=simulation owner=research-team rule=SIM-001
//! Agent lifecycle, energy consumption, and aging
// SIM-004: Agent psychology ticks execute update_trust_master_law in simulation/psychology.rs

use super::*;
use crate::agent::Agent;

impl Simulation {
    pub(crate) fn apply_energy_and_aging(&mut self) {
        if !self.config.toggles.biology_enabled {
            for agent in &mut self.agents {
                if agent.alive {
                    agent.energy = 75.0;
                }
            }
            return;
        }

        let energy_modifier = self.environment.temperature_energy_factor();

        for agent in &mut self.agents {
            if !agent.alive {
                continue;
            }

            let effective_rate =
                Self::calculate_agent_consumption_rate(agent, &self.config, energy_modifier);

            agent.consume_energy(effective_rate);
            agent.age_tick();
        }
    }

    #[inline]
    fn calculate_agent_consumption_rate(
        agent: &Agent,
        config: &SimulationConfig,
        energy_modifier: f32,
    ) -> f32 {
        if config.toggles.economics_enabled {
            config.energy_consumption_rate
                * agent.economics.energy_cost_modifier()
                * energy_modifier
        } else {
            config.energy_consumption_rate * energy_modifier
        }
    }

    pub(crate) fn collect_resources(&mut self) {
        for agent in &mut self.agents {
            if !agent.alive {
                continue;
            }
            let harvested = self.environment.harvest_at(&agent.position, 5.0);
            agent.gain_energy(harvested);

            if self.config.toggles.economics_enabled {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                agent.economics.wealth += harvested * 0.5;
            }
        }
    }
}
