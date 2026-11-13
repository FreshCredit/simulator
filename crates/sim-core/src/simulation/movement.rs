// TAG: surface=simulation owner=research-team rule=SIM-001
//! Movement, actions, physics, and pheromone dynamics

use super::*;
use crate::dmn::Action;

impl Simulation {
    /// Log movement statistics for the current tick.
    pub(crate) fn log_movement_stats(
        &self,
        moved_agents: usize,
        stayed_agents: usize,
        total_movement_distance: f32,
        alive_count_before: usize,
    ) {
        let avg_movement = if moved_agents > 0 {
            total_movement_distance / moved_agents as f32
        } else {
            0.0
        };

        let total_energy: f32 = self
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| a.energy)
            .sum();
        let avg_energy = if alive_count_before > 0 {
            total_energy / alive_count_before as f32
        } else {
            0.0
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let resource_cells = self
            .environment
            .resources
            .iter()
            .filter(|r| r.amount > 0.1)
            .count();
        let total_cells = self.environment.resources.len();

        debug!(" [MovementStats] tick={} moved={} stayed={} avg_dist={:.3} avg_energy={:.1} resources={}/{}({:.1}%)",
            self.tick, moved_agents, stayed_agents, avg_movement, avg_energy,
            resource_cells, total_cells, (resource_cells as f32 / total_cells as f32) * 100.0);
    }

    #[inline]
    fn calculate_terrain_altitude_cost(
        environment: &Environment,
        agent: &mut Agent,
        new_pos: &Position,
        base_cost: f32,
    ) -> f32 {
        if !environment.altitude_effects_enabled {
            return base_cost;
        }
        let old_z = agent.position.z;
        let target_cell = (new_pos.y as usize) * environment.width + (new_pos.x as usize);
        let new_z = if target_cell < environment.resources.len() {
            (environment.resources[target_cell].capacity / 30.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        agent.position.z = new_z;
        let delta_z = new_z - old_z;
        if delta_z > 0.0 {
            base_cost * (1.0 + delta_z * 2.0)
        } else {
            base_cost
        }
    }

    #[inline]
    fn compute_effective_delta(
        agent: &Agent,
        dir: dmn::Direction,
        barnes_hut_enabled: bool,
        social_influence: f32,
        movement_speed: f32,
    ) -> (f32, f32) {
        let (base_dx, base_dy) = dir.to_delta();
        let (final_dx, final_dy) = if barnes_hut_enabled && social_influence > 0.0 {
            Self::compute_social_movement(agent, base_dx, base_dy, social_influence)
        } else {
            (base_dx, base_dy)
        };
        let speed_mult = if movement_speed > 0.0 {
            movement_speed
        } else {
            1.0
        };
        (final_dx * speed_mult, final_dy * speed_mult)
    }

    #[inline]
    fn handle_agent_move(
        &mut self,
        agent_idx: usize,
        dir: dmn::Direction,
        barnes_hut_enabled: bool,
    ) -> (f32, Option<(AgentId, Position, f32)>) {
        let agent = &mut self.agents[agent_idx];
        let old_pos = agent.position;
        let (final_dx, final_dy) = Self::compute_effective_delta(
            agent,
            dir,
            barnes_hut_enabled,
            self.config.social_force_influence,
            self.config.movement_speed,
        );

        let new_pos = Position::new_2d(agent.position.x + final_dx, agent.position.y + final_dy);
        let new_pos = self.environment.clamp_position(&new_pos);
        let distance = agent.position.distance(&new_pos);
        if distance > 0.0 {
            let base_cost = self.config.movement_cost * distance;
            let cost = Self::calculate_terrain_altitude_cost(
                &self.environment,
                agent,
                &new_pos,
                base_cost,
            );
            agent.consume_energy(cost);
        }
        agent.position = new_pos;
        agent.last_dx = final_dx;
        agent.last_dy = final_dy;
        agent.clear_social_force();
        agent.chemotaxis_direction = None;

        if self.debug_state.sampled_agent_id == Some(agent.id) {
            debug!(" [AgentMove] tick={} agent={} dir={:?} pos=({:.2},{:.2})->({:.2},{:.2}) dist={:.2}",
                self.tick, agent.id, dir, old_pos.x, old_pos.y, new_pos.x, new_pos.y, distance);
        }

        (distance, Some((agent.id, agent.position, agent.energy)))
    }

    #[inline]
    fn handle_agent_stay(&mut self, agent_idx: usize) {
        let agent = &mut self.agents[agent_idx];
        agent.clear_social_force();
        agent.chemotaxis_direction = None;
        if self.debug_state.sampled_agent_id == Some(agent.id) {
            debug!(
                "[SimCore] [AgentStay] tick={} agent={} pos=({:.2},{:.2})",
                self.tick, agent.id, agent.position.x, agent.position.y
            );
        }
    }

    #[inline]
    fn handle_agent_reproduce(&mut self, agent_idx: usize) {
        let agent = &mut self.agents[agent_idx];
        agent.clear_social_force();
        agent.chemotaxis_direction = None;
    }

    pub(crate) fn execute_actions(
        &mut self,
        actions: Vec<(usize, Action)>,
        barnes_hut_enabled: bool,
        alive_count_before: usize,
        dmn_enabled: bool,
    ) {
        let (move_actions, stay_actions, reproduce_actions) = Self::count_action_types(&actions);
        if self.tick <= 5 || self.tick % 50 == 0 {
            debug!(
                " [ActionSelection] tick={} dmn={} total_agents={} move={} stay={} reproduce={}",
                self.tick,
                dmn_enabled,
                actions.len(),
                move_actions,
                stay_actions,
                reproduce_actions
            );
        }

        let mut total_movement_distance = 0.0;
        let mut moved_agents = 0;
        let mut stayed_agents = 0;
        let mut position_records: Vec<(AgentId, Position, f32)> = Vec::new();

        for (i, action) in actions {
            match action {
                Action::Move(dir) => {
                    let (distance, record) = self.handle_agent_move(i, dir, barnes_hut_enabled);
                    total_movement_distance += distance;
                    moved_agents += 1;
                    if let Some(r) = record {
                        position_records.push(r);
                    }
                }
                Action::Stay => {
                    self.handle_agent_stay(i);
                    stayed_agents += 1;
                }
                Action::Reproduce => {
                    self.handle_agent_reproduce(i);
                }
            }
        }

        for (agent_id, position, energy) in position_records {
            self.debug_state
                .record_position(self.tick, agent_id, position, energy);
        }

        if self.tick % 10 == 0 || alive_count_before < 10 {
            self.log_movement_stats(
                moved_agents,
                stayed_agents,
                total_movement_distance,
                alive_count_before,
            );
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub(crate) fn compute_social_movement(
        agent: &Agent,
        base_dx: f32,
        base_dy: f32,
        social_influence: f32,
    ) -> (f32, f32) {
        let social_force = &agent.social_force;
        let force_magnitude = social_force.x.hypot(social_force.y);

        if force_magnitude <= 0.001 {
            return (base_dx, base_dy);
        }

        let individual_weight = 1.0 - social_influence;
        let social_dx = social_force.x / force_magnitude.max(0.001);
        let social_dy = social_force.y / force_magnitude.max(0.001);

        let combined_dx =
            base_dx * individual_weight + social_dx * social_influence * force_magnitude.min(1.0);
        let combined_dy =
            base_dy * individual_weight + social_dy * social_influence * force_magnitude.min(1.0);

        let combined_magnitude = combined_dx.hypot(combined_dy).max(0.001);
        let scale = base_dx.hypot(base_dy).max(0.001);

        (
            combined_dx / combined_magnitude * scale,
            combined_dy / combined_magnitude * scale,
        )
    }
}
