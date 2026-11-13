// TAG: surface=simulation owner=research-team rule=SIM-001
//! Agent mortality rules, Gompertz hazard integration, and death telemetry

use super::*;
use crate::types::AgentId;
use rand::Rng;

#[inline]
fn classify_death_cause(energy: f32, age: u64) -> &'static str {
    if energy <= 0.0 {
        "starvation"
    } else if age > 1000 {
        "old_age"
    } else {
        "unknown"
    }
}

#[inline]
fn format_death_details(age: u64, energy: f32, pos: Position) -> String {
    format!(
        "age={age} energy={energy:.1} pos=({:.1},{:.1})",
        pos.x, pos.y
    )
}

#[inline]
fn evaluate_agent_mortality_step(agent: &mut Agent, roll: f32, baseline: f32, beta: f32) -> bool {
    if agent.should_die() {
        agent.alive = false;
        return true;
    }
    let age_mortality = baseline * (beta * agent.age as f32).exp();
    if roll < age_mortality {
        agent.alive = false;
        return true;
    }
    false
}

impl Simulation {
    pub(crate) fn process_mortality(&mut self) {
        if !self.config.toggles.biology_enabled {
            return;
        }
        let deaths = self.apply_mortality_rules();
        self.record_deaths(&deaths);
        self.metrics.deaths_this_tick += deaths.len() as u32;
    }

    pub(crate) fn apply_mortality_rules(&mut self) -> Vec<AgentId> {
        let agent_count = self.agents.len();
        let mortality_rolls: Vec<f32> = {
            let rng = self.rng();
            (0..agent_count).map(|_| rng.gen::<f32>()).collect()
        };
        let mut deaths = Vec::new();

        for (i, agent) in self.agents.iter_mut().enumerate() {
            if !agent.alive {
                continue;
            }
            if evaluate_agent_mortality_step(
                agent,
                mortality_rolls[i],
                self.config.mortality_baseline,
                self.config.mortality_aging_beta,
            ) {
                deaths.push(agent.id);
                continue;
            }
            let harvested = self.environment.harvest_at(&agent.position, 5.0);
            agent.gain_energy(harvested);
        }
        deaths
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub(crate) fn record_deaths(&mut self, deaths: &[AgentId]) {
        for agent_id in deaths {
            self.lineage.record_death(*agent_id, self.tick);
            let death_cause = if let Some(agent) = self.agents.iter().find(|a| a.id == *agent_id) {
                let details = format_death_details(agent.age, agent.energy, agent.position);
                self.debug_state
                    .log(self.tick, "AgentDeath", Some(*agent_id), &details);
                classify_death_cause(agent.energy, agent.age)
            } else {
                "unknown"
            };
            self.event_store
                .record_death(*agent_id, self.tick, death_cause);
            self.debug_state.total_deaths += 1;
        }
    }
}
