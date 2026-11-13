// TAG: surface=simulation owner=research-team rule=SIM-001
//! Agent stigmergy, pheromone sensing, gradient following, and trail deposition

use super::*;
use crate::stigmergy::PheromoneType;

impl Simulation {
    pub(crate) fn process_stigmergy(&mut self) {
        self.process_pheromone_sensing_and_movement();
        self.process_pheromone_deposits();
    }

    pub(crate) fn process_pheromone_sensing_and_movement(&mut self) {
        for agent in &mut self.agents {
            if !agent.alive {
                continue;
            }
            let pheromone_levels = agent.sense_pheromones(&self.pheromone_grid);
            let max_pheromone = pheromone_levels
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| match (a.is_nan(), b.is_nan()) {
                    (true, true) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    (false, false) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
                })
                .map(|(idx, _)| idx);

            if let Some(pheromone_idx) = max_pheromone {
                let pheromone_type = match pheromone_idx {
                    0 => PheromoneType::FoodTrail,
                    1 => PheromoneType::DangerSignal,
                    2 => PheromoneType::Reproduction,
                    3 => PheromoneType::Territory,
                    4 => PheromoneType::Recruitment,
                    _ => continue,
                };
                let level = pheromone_levels[pheromone_idx];
                if level > crate::agent::Agent::PHEROMONE_THRESHOLD {
                    let gradient = agent.follow_gradient(&self.pheromone_grid, pheromone_type);
                    agent.chemotaxis_direction = Some(gradient);
                    let chemotaxis_strength = 0.3;
                    let new_x = agent.position.x + gradient.dx * chemotaxis_strength;
                    let new_y = agent.position.y + gradient.dy * chemotaxis_strength;
                    let new_pos = Position::new_2d(new_x, new_y);
                    let new_pos = self.environment.clamp_position(&new_pos);
                    let distance = agent.position.distance(&new_pos);
                    if distance > 0.01 {
                        agent.consume_energy(self.config.movement_cost * distance);
                        agent.position = new_pos;
                        agent.last_dx = gradient.dx;
                        agent.last_dy = gradient.dy;
                        if self.debug_state.sampled_agent_id == Some(agent.id) {
                            debug!(" [PheromoneFollow] tick={} agent={} type={:?} level={:.2} grad=({:.2},{:.2})",
                                self.tick, agent.id, pheromone_type, level, gradient.dx, gradient.dy);
                        }
                    }
                } else {
                    agent.chemotaxis_direction = None;
                }
            }
        }
    }

    pub(crate) fn process_pheromone_deposits(&mut self) {
        for agent in &mut self.agents {
            if agent.alive {
                agent.deposit_pheromones(&mut self.pheromone_grid, self.tick as u32);
            }
        }
    }
}
