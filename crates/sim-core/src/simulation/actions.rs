// TAG: surface=simulation owner=research-team rule=SIM-001
//! Agent action planning, DMN rollouts, and heuristic neighborhood evaluation

use super::*;
use crate::dmn::Action;
use rand::Rng;

impl Simulation {
    pub(crate) fn compute_actions(&mut self) -> Vec<(usize, Action)> {
        let dmn_enabled = self.config.toggles.dmn_enabled;
        let choice_arch_enabled = self.config.toggles.choice_architecture_enabled;

        if dmn_enabled {
            self.compute_actions_dmn(choice_arch_enabled)
        } else {
            self.compute_actions_heuristic()
        }
    }

    pub(crate) fn compute_actions_dmn(
        &mut self,
        choice_arch_enabled: bool,
    ) -> Vec<(usize, Action)> {
        let horizon = Some((self.config.planning_horizon as u32).clamp(1, 20));
        let budget_override = if self.config.toggles.dials.dmn_b_simulations > 0 {
            Some(self.config.toggles.dials.dmn_b_simulations)
        } else {
            None
        };
        let nudge_cfg =
            crate::choice_architecture::NudgeConfig::from_dials(&self.config.toggles.dials);
        let k_range = if self.config.toggles.dials.dmn_k_rollout_range.1
            > self.config.toggles.dials.dmn_k_rollout_range.0
        {
            Some(self.config.toggles.dials.dmn_k_rollout_range)
        } else {
            None
        };
        let h_range = if self.config.toggles.dials.dmn_h_depth_range.1
            > self.config.toggles.dials.dmn_h_depth_range.0
        {
            Some(self.config.toggles.dials.dmn_h_depth_range)
        } else {
            None
        };

        let agents_snapshot: Vec<(usize, Agent)> = self
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive)
            .map(|(i, a)| (i, a.clone()))
            .collect();
        let env_snapshot = self.environment.clone();
        let rng = self.rng();

        agents_snapshot
            .iter()
            .map(|(original_idx, agent)| {
                let action = dmn::plan_action_full(
                    agent,
                    &env_snapshot,
                    rng,
                    choice_arch_enabled,
                    horizon,
                    budget_override,
                    Some(&nudge_cfg),
                    k_range,
                    h_range,
                );
                (*original_idx, action)
            })
            .collect()
    }

    fn build_density_grid(positions: &[Position], width: usize, height: usize) -> Vec<u16> {
        let mut density_grid = vec![0u16; width * height];
        for pos in positions {
            let gx = (pos.x as usize).min(width - 1);
            let gy = (pos.y as usize).min(height - 1);
            density_grid[gy * width + gx] = density_grid[gy * width + gx].saturating_add(1);
        }
        density_grid
    }

    fn evaluate_local_social_density(
        density_grid: &[u16],
        width: usize,
        height: usize,
        cell: &Position,
    ) -> f32 {
        let cx = (cell.x as usize).min(width - 1);
        let cy = (cell.y as usize).min(height - 1);
        let min_x = cx.saturating_sub(1);
        let max_x = (cx + 1).min(width - 1);
        let min_y = cy.saturating_sub(1);
        let max_y = (cy + 1).min(height - 1);
        let mut local_count = 0u32;
        for gy in min_y..=max_y {
            for gx in min_x..=max_x {
                local_count += density_grid[gy * width + gx] as u32;
            }
        }
        let social_density = (local_count as f32 / 9.0).min(5.0);
        let crowding_penalty = if social_density > 3.0 {
            (social_density - 3.0) * 1.5
        } else {
            0.0
        };
        (social_density - crowding_penalty).max(0.0)
    }

    #[inline]
    fn is_cell_blocked(occupancy: &OccupancyGrid, cell: &Position, agent_id: AgentId) -> bool {
        if occupancy.is_occupied(cell) {
            if let Some(occupant_id) = occupancy.get_occupant(cell) {
                return occupant_id != agent_id;
            }
        }
        false
    }

    #[inline]
    fn compute_cell_utility(
        d: f32,
        o: f32,
        c: f32,
        resource_value: f32,
        effective_social: f32,
        threat_level: f32,
    ) -> f32 {
        d * resource_value + o * effective_social - c * threat_level
    }

    #[inline]
    fn compute_noisy_utility(utility: f32, base_noise: f32, cell_idx: usize) -> f32 {
        let noise = base_noise * (1.0 + cell_idx as f32 * 0.1) * utility.abs().max(0.1);
        utility + noise
    }

    #[inline]
    fn determine_agent_action(
        best_cell: Position,
        agent_pos: Position,
        agent_id: AgentId,
        tick: u64,
    ) -> Action {
        if best_cell.distance(&agent_pos) > 0.01 {
            let dx = best_cell.x - agent_pos.x;
            let dy = best_cell.y - agent_pos.y;
            Action::Move(dmn::Direction::from_delta(dx, dy))
        } else {
            let forced_dirs = [
                dmn::Direction::North,
                dmn::Direction::South,
                dmn::Direction::East,
                dmn::Direction::West,
            ];
            let idx = (agent_id as usize + tick as usize) % forced_dirs.len();
            Action::Move(forced_dirs[idx])
        }
    }

    pub(crate) fn compute_actions_heuristic(&mut self) -> Vec<(usize, Action)> {
        let mut occupancy = OccupancyGrid::new();
        occupancy.update_from_agents(self.agents.iter().map(|a| (a.id, &a.position, a.alive)));

        let agent_positions: Vec<Position> = self
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| a.position)
            .collect();

        let width = self.environment.width.max(1);
        let height = self.environment.height.max(1);
        let density_grid = Self::build_density_grid(&agent_positions, width, height);

        let threat_positions: Vec<Position> = Vec::new();
        let threat_intensities: Vec<f32> = Vec::new();

        let agent_count = self.agents.len();
        let noise_values: Vec<f32> = {
            let rng = self.rng();
            (0..agent_count)
                .map(|_| rng.gen_range(-0.1f32..0.1f32))
                .collect()
        };

        self.agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive)
            .map(|(i, agent)| {
                let [d, o, c, _a, _s] = agent.docas.levels;
                let candidates = self.environment.get_9_cell_neighborhood(&agent.position);
                let mut best_cell = agent.position;
                let mut best_utility = f32::NEG_INFINITY;
                let base_noise = noise_values.get(i).copied().unwrap_or(0.0);

                for (cell_idx, cell) in candidates.iter().enumerate() {
                    if Self::is_cell_blocked(&occupancy, cell, agent.id) {
                        continue;
                    }

                    let resource_value = self.environment.resource_value_at(cell);
                    let effective_social =
                        Self::evaluate_local_social_density(&density_grid, width, height, cell);
                    let threat_level =
                        self.environment
                            .threat_at(cell, &threat_positions, &threat_intensities);

                    let utility = Self::compute_cell_utility(
                        d,
                        o,
                        c,
                        resource_value,
                        effective_social,
                        threat_level,
                    );
                    let noisy_utility = Self::compute_noisy_utility(utility, base_noise, cell_idx);

                    if noisy_utility > best_utility {
                        best_utility = noisy_utility;
                        best_cell = *cell;
                    }
                }

                let action =
                    Self::determine_agent_action(best_cell, agent.position, agent.id, self.tick);
                (i, action)
            })
            .collect()
    }

    pub(crate) fn count_action_types(actions: &[(usize, Action)]) -> (usize, usize, usize) {
        let mut move_actions = 0;
        let mut stay_actions = 0;
        let mut reproduce_actions = 0;
        for (_, action) in actions {
            match action {
                Action::Move(_) => move_actions += 1,
                Action::Stay => stay_actions += 1,
                Action::Reproduce => reproduce_actions += 1,
            }
        }
        (move_actions, stay_actions, reproduce_actions)
    }
}
