// TAG: surface=simulation owner=research-team rule=SIM-001
//! Agent reproduction, partner selection, crossover, and birthrate execution

use super::*;
use crate::agent::Agent;
use crate::birthrate::{BirthrateConfig, BirthrateSystem, ReproductionEvent};
use crate::genome::Genome;
use crate::types::{AgentId, Position};
use rand::Rng;

#[inline]
fn compute_parent_generation(
    lineage: &crate::lineage::LineageTree,
    p1: AgentId,
    p2: Option<AgentId>,
) -> u32 {
    if let Some(p2_id) = p2 {
        lineage
            .get_lineage_depth(p1)
            .max(lineage.get_lineage_depth(p2_id))
    } else {
        lineage.get_lineage_depth(p1)
    }
}

#[inline]
fn is_kinship_avoided(
    lineage: &crate::lineage::LineageTree,
    id_a: AgentId,
    id_b: AgentId,
    lineage_depth_limit: usize,
) -> bool {
    if lineage_depth_limit == 0 {
        return true;
    }
    if let (Some(node_a), Some(node_b)) = (lineage.get_node(id_a), lineage.get_node(id_b)) {
        if (node_a.parent1_id == node_b.parent1_id || node_a.parent2_id == node_b.parent2_id)
            && node_a.generation <= lineage_depth_limit as u32
        {
            return false;
        }
    }
    true
}

#[inline]
fn collect_eligible_agents(agents: &[Agent], threshold: f32) -> Vec<usize> {
    agents
        .iter()
        .enumerate()
        .filter(|(_, a)| a.alive && a.energy >= threshold)
        .map(|(i, _)| i)
        .collect()
}

#[inline]
fn compute_compatibility(agent_a: &Agent, agent_b: &Agent, kappa: f32) -> f32 {
    let rep_a = agent_a.interaction_reputation();
    let rep_b = agent_b.interaction_reputation();
    let tau_a = agent_a.compute_trust();
    let tau_b = agent_b.compute_trust();
    (tau_a + tau_b) * (rep_a + rep_b) * kappa
}

impl Simulation {
    /// Process agent reproduction
    pub(crate) fn process_reproduction(&mut self) {
        if let Some(ref birthrate_config) = self.config.birthrate_config {
            self.process_birthrate_reproduction(*birthrate_config);
        } else {
            self.process_threshold_reproduction();
        }
    }

    /// Process reproduction using the new birthrate system
    pub(crate) fn process_birthrate_reproduction(&mut self, config: BirthrateConfig) {
        const MAX_POPULATION: usize = 500;
        let current_population = self.agents.iter().filter(|a| a.alive).count();
        if current_population >= MAX_POPULATION {
            return;
        }

        let system = BirthrateSystem::new(config);
        let current_tick = self.tick;

        let agents_snapshot: Vec<Agent> = self.agents.clone();
        let rng = self.rng();
        let events = system.process_reproduction(&agents_snapshot, current_tick, rng);

        let remaining_capacity = MAX_POPULATION.saturating_sub(current_population);
        for event in events.into_iter().take(remaining_capacity) {
            self.execute_reproduction_event(event, &config);
        }
    }

    /// Execute a single reproduction event (spawn offspring)
    pub(crate) fn execute_reproduction_event(
        &mut self,
        event: ReproductionEvent,
        config: &BirthrateConfig,
    ) {
        let ReproductionEvent {
            parent1_idx,
            parent2_idx,
            spawn_position,
            offspring_genome,
        } = event;

        let spawn_pos = self.environment.clamp_position(&spawn_position);
        let parent1_id = self.agents[parent1_idx].id;
        let parent2_id = parent2_idx.map(|idx| self.agents[idx].id);
        let parent_gen = compute_parent_generation(&self.lineage, parent1_id, parent2_id);

        self.agents[parent1_idx].consume_energy(config.reproduction_cost);
        if let Some(p2_idx) = parent2_idx {
            self.agents[p2_idx].consume_energy(config.reproduction_cost);
        }

        self.spawn_offspring_with_genome(
            spawn_pos,
            offspring_genome,
            parent1_id,
            parent2_id.unwrap_or(parent1_id),
            parent_gen,
        );
    }

    /// Find the best reproduction partner for `i_a` from the eligible pool.
    pub(crate) fn find_best_partner(
        &self,
        i_a: usize,
        start_idx: usize,
        eligible: &[usize],
        used: &std::collections::HashSet<usize>,
    ) -> Option<usize> {
        let agent_a_reputation = self.agents[i_a].interaction_reputation();
        let mut best_partner: Option<usize> = None;
        let mut best_score = 0.0f32;
        let kappa = self.config.toggles.dials.kingman_coalescent.clamp(0.1, 5.0);
        let trust_req = (0.4 / kappa).clamp(0.05, 0.95);
        let lineage_depth_limit = self.config.toggles.dials.lineage_depth;

        for &i_b in eligible.iter().skip(start_idx) {
            if used.contains(&i_b) {
                continue;
            }

            let agent_b_reputation = self.agents[i_b].interaction_reputation();

            if !self.agents[i_a].should_trust(agent_b_reputation, trust_req) {
                continue;
            }
            if !self.agents[i_b].should_trust(agent_a_reputation, trust_req) {
                continue;
            }

            if !is_kinship_avoided(
                &self.lineage,
                self.agents[i_a].id,
                self.agents[i_b].id,
                lineage_depth_limit,
            ) {
                continue;
            }

            let compatibility = compute_compatibility(&self.agents[i_a], &self.agents[i_b], kappa);
            if compatibility > best_score {
                best_score = compatibility;
                best_partner = Some(i_b);
            }
        }

        best_partner
    }

    /// Create offspring genome via Mendelian crossover or random generation.
    pub(crate) fn create_offspring_genome(
        &mut self,
        genome_a: &Genome,
        genome_b: &Genome,
    ) -> Genome {
        if self.config.toggles.genome_enabled {
            crate::genome::crossover(genome_a, genome_b, self.config.mutation_rate, self.rng())
        } else {
            Genome::random(self.rng())
        }
    }

    /// Compute a random position near the parent for offspring spawning.
    pub(crate) fn compute_offspring_position(&mut self, parent_pos: Position) -> Position {
        let rng = self.rng();
        let offset_x = rng.gen::<f32>() * 2.0 - 1.0;
        let offset_y = rng.gen::<f32>() * 2.0 - 1.0;
        self.environment.clamp_position(&Position::new_2d(
            parent_pos.x + offset_x,
            parent_pos.y + offset_y,
        ))
    }

    /// Legacy threshold-based reproduction
    pub(crate) fn process_threshold_reproduction(&mut self) {
        const MAX_POPULATION: usize = 500;
        let current_population = self.agents.iter().filter(|a| a.alive).count();
        if current_population >= MAX_POPULATION {
            return;
        }

        let eligible = collect_eligible_agents(&self.agents, self.config.reproduction_threshold);
        if eligible.len() < 2 {
            return;
        }

        let mut offspring_data: Vec<(Position, Genome, AgentId, AgentId, u32)> = Vec::new();
        let mut used_indices: std::collections::HashSet<usize> =
            std::collections::HashSet::with_capacity(eligible.len());

        let start_offset = (self.tick as usize) % eligible.len().max(1);
        for step in 0..eligible.len() {
            if current_population + offspring_data.len() >= MAX_POPULATION {
                break;
            }

            let i = (start_offset + step) % eligible.len();
            let i_a = eligible[i];
            if used_indices.contains(&i_a) {
                continue;
            }

            let best_partner =
                self.find_best_partner(i_a, (i + 1) % eligible.len(), &eligible, &used_indices);

            let i_b = best_partner.unwrap_or_else(|| {
                eligible
                    .iter()
                    .skip(i + 1)
                    .find(|&&idx| !used_indices.contains(&idx))
                    .copied()
                    .unwrap_or(eligible[i])
            });

            if i_a == i_b || used_indices.contains(&i_b) {
                continue;
            }

            used_indices.insert(i_a);
            used_indices.insert(i_b);

            let parent1_id = self.agents[i_a].id;
            let parent2_id = self.agents[i_b].id;
            let parent_gen = self
                .lineage
                .get_lineage_depth(parent1_id)
                .max(self.lineage.get_lineage_depth(parent2_id));

            let genome_a = self.agents[i_a].genome.clone();
            let genome_b = self.agents[i_b].genome.clone();
            let parent_pos = self.agents[i_a].position;

            let offspring_genome = self.create_offspring_genome(&genome_a, &genome_b);
            let offspring_pos = self.compute_offspring_position(parent_pos);

            offspring_data.push((
                offspring_pos,
                offspring_genome,
                parent1_id,
                parent2_id,
                parent_gen,
            ));
        }

        for &idx in &used_indices {
            self.agents[idx].consume_energy(self.config.reproduction_cost);
        }

        for (pos, genome, parent1_id, parent2_id, parent_gen) in offspring_data {
            self.spawn_offspring_with_genome(pos, genome, parent1_id, parent2_id, parent_gen);
        }
    }

    /// Look up an agent's epigenome by ID, returning a default if not found.
    pub(crate) fn get_agent_epigenome(
        agents: &[Agent],
        agent_id: AgentId,
    ) -> crate::epigenome::Epigenome {
        agents
            .iter()
            .find(|a| a.id == agent_id)
            .map_or_else(crate::epigenome::Epigenome::new, |a| a.epigenome.clone())
    }

    /// Check whether an agent has anchored identity claims.
    pub(crate) fn agent_has_anchored_identity(agents: &[Agent], agent_id: AgentId) -> bool {
        agents
            .iter()
            .find(|a| a.id == agent_id)
            .and_then(|a| a.identity.as_ref())
            .is_some_and(|i| !i.anchored_claims.is_empty())
    }

    /// Spawn offspring agent with genome and parent lineage information
    pub(crate) fn spawn_offspring_with_genome(
        &mut self,
        position: Position,
        genome: Genome,
        parent1_id: AgentId,
        parent2_id: AgentId,
        parent_generation: u32,
    ) -> AgentId {
        let id = self.next_agent_id;
        self.next_agent_id += 1;

        let is_asexual = parent1_id == parent2_id;

        let trait_snapshot = TraitSnapshot::from_genome(&genome, self.tick);
        let lineage_node = LineageNode::new_offspring(
            id,
            parent1_id,
            if is_asexual { None } else { Some(parent2_id) },
            self.tick,
            trait_snapshot,
            parent_generation,
        );
        self.lineage.add_offspring(lineage_node);

        let sex = Sex::random(self.rng());

        let mut agent = Agent::new(id, position, genome, self.tick, sex);
        if !self.config.toggles.docas_enabled {
            agent.docas.reset_neutral();
        }

        if self.config.toggles.language_enabled {
            let tick = self.tick;
            agent.lexicon = Lexicon::initialize_base(self.rng(), tick);
        }

        let offspring_energy = self
            .config
            .birthrate_config
            .as_ref()
            .map_or(self.config.reproduction_cost, |br| br.reproduction_cost);
        agent.energy = offspring_energy;

        let parent1_epigenome = Self::get_agent_epigenome(&self.agents, parent1_id);
        let parent2_epigenome = if is_asexual {
            crate::epigenome::Epigenome::new()
        } else {
            Self::get_agent_epigenome(&self.agents, parent2_id)
        };

        agent.inherit_epigenome(&parent1_epigenome, &parent2_epigenome);

        let avg_maternal_care = (parent1_epigenome.maternal_care_received
            + parent2_epigenome.maternal_care_received)
            / 2.0;
        agent.program_early_life(avg_maternal_care, self.tick);

        let mut identity = Identity::new(id, self.config.seed);
        identity.add_claim(
            IdentityClaimType::Reputation,
            agent.reputation_score,
            self.tick,
        );
        identity.add_claim(IdentityClaimType::Wealth, agent.energy / 100.0, self.tick);

        let parent1_has_anchor = Self::agent_has_anchored_identity(&self.agents, parent1_id);
        let parent2_has_anchor =
            !is_asexual && Self::agent_has_anchored_identity(&self.agents, parent2_id);

        if parent1_has_anchor || parent2_has_anchor {
            identity.add_claim(IdentityClaimType::IdentityVerification, 0.5, self.tick);
        }

        agent.identity = Some(identity);

        let energy_for_log = agent.energy;
        self.debug_state.log(
            self.tick,
            "AgentReproduce",
            Some(id),
            &format!(
                "parents=({}, {}) gen={} pos=({:.1},{:.1}) energy={:.1}",
                parent1_id,
                parent2_id,
                parent_generation + 1,
                position.x,
                position.y,
                energy_for_log
            ),
        );
        self.debug_state.total_reproductions += 1;

        let parent2_for_event = if is_asexual { None } else { Some(parent2_id) };
        self.event_store
            .record_birth(id, self.tick, Some(parent1_id), parent2_for_event);

        self.agents.push(agent);

        id
    }
}
