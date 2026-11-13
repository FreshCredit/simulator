// TAG: surface=simulation owner=research-team rule=SIM-001
//! Discrete generation manager and Wright-Fisher generation replacement

use super::*;
use crate::agent::Agent;
use crate::epigenome::Epigenome;
use crate::generation::sample_offspring_genomes;
use crate::genome::Genome;
use crate::identity::{ClaimType as IdentityClaimType, Identity};
use crate::language::Lexicon;
use crate::lineage::{LineageNode, TraitSnapshot};
use crate::types::{AgentId, Position, Sex};
use rand::Rng;

impl Simulation {
    /// Execute generation replacement (Wright-Fisher discrete mode)
    pub(crate) fn execute_generation_replacement(&mut self) {
        let pop_size = self
            .discrete_generation
            .as_ref()
            .map_or(0, |mgr| mgr.population_size);
        if pop_size == 0 {
            return;
        }
        let offspring_genomes = self.sample_offspring_for_replacement(pop_size);
        self.replace_generation(&offspring_genomes, pop_size);
    }

    fn sample_offspring_for_replacement(&mut self, pop_size: usize) -> Vec<Genome> {
        let parent_genomes: Vec<Genome> = self
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| a.genome.clone())
            .collect();
        sample_offspring_genomes(
            pop_size,
            &parent_genomes,
            self.config.mutation_rate,
            self.rng(),
        )
    }

    fn replace_generation(&mut self, offspring_genomes: &[Genome], pop_size: usize) {
        let agent_ids: Vec<AgentId> = self
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| a.id)
            .collect();
        let parent_epigenomes: Vec<Epigenome> = self
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| a.epigenome.clone())
            .collect();
        for agent_id in agent_ids {
            self.lineage.record_death(agent_id, self.tick);
        }
        self.agents.clear();
        self.spawn_new_generation(offspring_genomes, &parent_epigenomes);
        let new_freqs = self.compute_allele_frequencies();
        let new_heterozygosity = self.compute_heterozygosity();
        let new_agent_ids: Vec<AgentId> = self.agents.iter().map(|a| a.id).collect();
        let current_tick = self.tick;
        if let Some(ref mut mgr) = self.discrete_generation {
            mgr.advance_generation(
                new_agent_ids,
                new_freqs.clone(),
                new_heterozygosity,
                current_tick,
            );
        }
        if let Some(ref mut wf) = self.wf_model {
            wf.record_generation(&new_freqs, new_heterozygosity);
        }
        self.metrics.births_this_tick += pop_size as u32;
        self.metrics.deaths_this_tick += pop_size as u32;
    }

    /// Spawn a new generation from offspring genomes (Wright-Fisher mode)
    fn spawn_new_generation(
        &mut self,
        offspring_genomes: &[Genome],
        parent_epigenomes: &[Epigenome],
    ) {
        let parent_gen = self.discrete_generation.as_ref().map_or(
            0,
            crate::DiscreteGenerationManager::current_generation_number,
        );

        let env_width = self.environment.width as f32;
        let env_height = self.environment.height as f32;
        let parent_count = parent_epigenomes.len();

        for genome in offspring_genomes {
            let id = self.next_agent_id;
            self.next_agent_id += 1;

            let rng = self.rng();
            let x = rng.gen::<f32>() * env_width;
            let y = rng.gen::<f32>() * env_height;
            let position = Position::new_2d(x, y);

            let trait_snapshot = TraitSnapshot::from_genome(genome, self.tick);
            let lineage_node =
                LineageNode::new_offspring(id, 0, None, self.tick, trait_snapshot, parent_gen);
            self.lineage.add_offspring(lineage_node);

            let sex = Sex::random(self.rng());
            let mut agent = Agent::new(id, position, genome.clone(), self.tick, sex);

            if self.config.toggles.language_enabled {
                let tick = self.tick;
                agent.lexicon = Lexicon::initialize_base(self.rng(), tick);
            }

            if parent_count >= 2 {
                let parent1_idx = self.rng().gen_range(0..parent_count);
                let mut parent2_idx = self.rng().gen_range(0..parent_count);
                while parent2_idx == parent1_idx && parent_count > 1 {
                    parent2_idx = self.rng().gen_range(0..parent_count);
                }

                agent.inherit_epigenome(
                    &parent_epigenomes[parent1_idx],
                    &parent_epigenomes[parent2_idx],
                );

                let avg_care = (parent_epigenomes[parent1_idx].maternal_care_received
                    + parent_epigenomes[parent2_idx].maternal_care_received)
                    / 2.0;
                agent.program_early_life(avg_care, self.tick);
            }

            let mut identity = Identity::new(id, self.config.seed);
            identity.add_claim(
                IdentityClaimType::Reputation,
                agent.reputation_score,
                self.tick,
            );
            identity.add_claim(IdentityClaimType::Wealth, agent.energy / 100.0, self.tick);
            agent.identity = Some(identity);

            agent.energy = self.config.reproduction_cost;
            self.agents.push(agent);
        }
    }
}
