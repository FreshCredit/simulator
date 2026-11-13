//! Simulation module: Main simulation loop and tick execution
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION loop, not product workflow
//! SIM-004: Deterministic kernel - same seed + same config = identical output
//!
//! Per §7.6: 14-step agent update loop
//! Per §22.5.3: Phase 0/Phase 1 exit criteria

// TAG: surface=simulation owner=research-team rule=SIM-001
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tracing::debug;

use crate::agent::Agent;
use crate::barnes_hut::{default_social_force, BarnesHutTree};
use crate::dmn;
use crate::environment::{Environment, OccupancyGrid};
use crate::generation::{
    compute_allele_frequencies, compute_heterozygosity, DiscreteGenerationManager, GenerationMode,
    WrightFisherDiscreteModel,
};
use crate::genome::Genome;
use crate::identity::{ClaimType as IdentityClaimType, Identity};
use crate::language::Lexicon;
use crate::lineage::{LineageNode, LineageTree, TraitSnapshot};
use crate::protocol::{ProtocolState, VerificationHash};
use crate::society::{BehaviorFrequency, CoordinationTracker, InstitutionalMemory, Norm};
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
use crate::stigmergy::PheromoneGrid;
use crate::types::{AgentId, GroupId, NormId, Position, Sex, Tick};

mod economics;
mod language;
mod protocol;

/// config module.
pub mod config;
/// debug module.
pub mod debug;
/// events module.
pub mod events;

mod actions;
mod generation;
mod kinetics;
mod lifecycle;
mod mortality;
mod psychology;
mod reproduction;

mod movement;
mod society_engine;
mod stigmergy_engine;
pub use config::SimulationConfig;
pub use debug::SimulationDebugState;
pub use events::{
    EncounterEvent, EventStore, EventSummary, GameEvent, MemoryUsage, SimulationMetrics,
    SimulationReport,
};

/// Main simulation state
/// Per §22.5.3: Phase 0 single-agent validation
#[derive(Debug, Clone)]
pub struct Simulation {
    /// Current tick
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub tick: Tick,
    /// Configuration
    pub config: SimulationConfig,
    /// Environment
    pub environment: Environment,
    /// Agents (indexed by ID)
    pub agents: Vec<Agent>,
    /// Next agent ID
    next_agent_id: AgentId,
    /// RNG for deterministic execution
    rng: Option<StdRng>,

    // === Stage 3 (Society) state ===
    /// Active norms per §4264-4282
    pub active_norms: Vec<Norm>,
    /// Next norm ID
    next_norm_id: NormId,
    /// Institutional memory per §4311-4338 (Beinhocker)
    pub institutions: InstitutionalMemory,
    /// Next group ID for group formation
    _next_group_id: GroupId,
    /// Behavior frequency tracking per group (for norm emergence)
    behavior_frequencies: std::collections::BTreeMap<Option<GroupId>, BehaviorFrequency>,
    /// Coordination tracking per group (for institution emergence)
    coordination_trackers: std::collections::BTreeMap<Option<GroupId>, CoordinationTracker>,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // === Stage 5 (Protocol) state ===
    /// Protocol state for blockchain verification simulation
    /// Per §22.1: Phase 1 MVP uses local SHA-256 hashes
    /// Per §8.2.1 lines 4646-4666: Protocol toggle contract
    pub protocol: ProtocolState,
    /// Agent identity hashes (`agent_id` -> verification hash)
    agent_identity_hashes: std::collections::BTreeMap<AgentId, VerificationHash>,

    // === Lineage Tracking ===
    /// Lineage tree for genealogy and trait evolution tracking
    pub lineage: LineageTree,

    // === Metrics ===
    /// Simulation metrics and statistics
    pub metrics: SimulationMetrics,

    // === Wright-Fisher Discrete Generation Mode ===
    /// Discrete generation manager (only used when `generation_mode` is Discrete)
    pub discrete_generation: Option<DiscreteGenerationManager>,
    /// Wright-Fisher model for validation (only used when `generation_mode` is Discrete)
    pub wf_model: Option<WrightFisherDiscreteModel>,

    // === Stigmergy Layer (SIM-STIG-001) ===
    /// Virtual pheromone grid for indirect coordination
    pub pheromone_grid: PheromoneGrid,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // === Debug State ===
    /// Debug logging state for diagnosing issues
    pub debug_state: SimulationDebugState,

    // === Event Logging for UI ===
    /// Game events this tick for activity log
    pub game_events_this_tick: Vec<GameEvent>,
    /// Encounter events this tick for activity log
    pub encounter_events_this_tick: Vec<EncounterEvent>,

    // === Event Persistence for Analysis ===
    /// Persistent event store for post-simulation analysis
    pub event_store: EventStore,
    /// Chronobiological SCN circadian clock system
    pub circadian: crate::circadian::CircadianSystem,
}

/// Game outcome from a single game interaction
#[derive(Debug, Clone, Copy)]
struct GameOutcome {
    a_coop: bool,
    b_coop: bool,
    payoff_a: f32,
    payoff_b: f32,
}

impl Simulation {
    /// Create new simulation with configuration
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(config: SimulationConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(config.seed);
        let environment = Environment::new(&config.environment, &mut rng);

        debug!(" ============================================");
        debug!(" Simulation initialized");
        debug!(" Seed: {}", config.seed);
        debug!(
            "[SimCore] Grid: {}x{}",
            config.environment.width, config.environment.height
        );
        debug!(" Max ticks: {}", config.max_ticks);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        debug!(" ============================================");

        Self {
            tick: 0,
            config: config.clone(),
            environment,
            agents: Vec::new(),
            next_agent_id: 0,
            rng: Some(rng),
            // Stage 3 initialization
            active_norms: Vec::new(),
            next_norm_id: 0,
            institutions: InstitutionalMemory::default(),
            _next_group_id: 0,
            behavior_frequencies: std::collections::BTreeMap::new(),
            coordination_trackers: std::collections::BTreeMap::new(),
            // Stage 5 initialization
            protocol: ProtocolState::with_mode(config.verification_mode),
            agent_identity_hashes: std::collections::BTreeMap::new(),
            // Lineage tracking
            lineage: LineageTree::new(),
            // Metrics
            metrics: SimulationMetrics::default(),
            // Discrete generation mode (initialized when needed)
            discrete_generation: None,
            wf_model: None,
            // Stigmergy layer - initialized with environment dimensions
            pheromone_grid: PheromoneGrid::new(
                config.environment.width,
                config.environment.height,
                config.environment.depth,
            ),
            // Debug state
            debug_state: SimulationDebugState::default(),
            // Event logging
            game_events_this_tick: Vec::new(),
            encounter_events_this_tick: Vec::new(),
            // Event persistence for analysis
            event_store: EventStore::new(),
            circadian: crate::circadian::CircadianSystem::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create default Phase 0 simulation
    #[must_use]
    pub fn new_default() -> Self {
        Self::new(SimulationConfig::default())
    }

    /// Get mutable RNG reference (initializes if needed)
    /// SAFETY: RNG is guaranteed to be Some after initialization check
    fn rng(&mut self) -> &mut StdRng {
        self.rng
            .get_or_insert_with(|| StdRng::seed_from_u64(self.config.seed))
    }

    /// Set the next agent ID (used during checkpoint state hydration)
    pub fn set_next_agent_id(&mut self, next: AgentId) {
        self.next_agent_id = next;
    }
    /// Initialize sub-systems (geometry, lexicon, identity) for an imported agent
    pub fn hydrate_agent_subsystems(&mut self, agent: &mut Agent) {
        agent.update_geometric_properties();
        if self.config.toggles.language_enabled {
            let tick = self.tick;
            agent.lexicon = Lexicon::initialize_base(self.rng(), tick);
        }
        if agent.has_verified_id {
            let tick = self.tick;
            let mut identity = Identity::new(agent.id, self.config.seed);
            identity.add_claim(IdentityClaimType::Reputation, agent.reputation_score, tick);
            identity.add_claim(IdentityClaimType::Wealth, agent.energy / 100.0, tick);
            agent.identity = Some(identity);
        }
    }

    /// Spawn a new agent at position
    /// Per §10.5.2: Agents ALWAYS get random genomes for visual diversity
    /// The `genome_enabled` toggle controls INHERITANCE (crossover/mutation), not initial diversity
    pub fn spawn_agent(&mut self, position: Position) -> AgentId {
        let id = self.next_agent_id;
        self.next_agent_id += 1;

        // ALWAYS use random genomes for visual diversity (SIM-GENOME-001)
        // The genome_enabled toggle controls inheritance during reproduction,
        // not whether spawned agents have diverse genomes.
        // Without this, all agents would be colored the same (all green).
        let genome = Genome::random(self.rng());

        // Random sex assignment per SIM-001 (50/50 Male/Female)
        let sex = Sex::random(self.rng());

        // Record in lineage tree as initial (spawned) agent
        let trait_snapshot = TraitSnapshot::from_genome(&genome, self.tick);
        let lineage_node = LineageNode::new_initial(id, self.tick, trait_snapshot);
        self.lineage.add_initial(lineage_node);

        let mut agent = Agent::new(id, position, genome, self.tick, sex);
        if !self.config.toggles.docas_enabled {
            agent.docas.reset_neutral();
        }

        // P0-FIX: Initialize lexicon when language_enabled
        if self.config.toggles.language_enabled {
            let tick = self.tick;
            agent.lexicon = Lexicon::initialize_base(self.rng(), tick);
        }

        // Initialize identity for spawned agents (SIM-010)
        let mut identity = Identity::new(id, self.config.seed);
        identity.add_claim(
            IdentityClaimType::Reputation,
            agent.reputation_score,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            self.tick,
        );
        identity.add_claim(IdentityClaimType::Wealth, agent.energy / 100.0, self.tick);
        agent.identity = Some(identity);

        // DEBUG: Log agent spawn
        self.debug_state.log(
            self.tick,
            "AgentSpawn",
            Some(id),
            &format!(
                "pos=({:.1},{:.1}) energy={:.1} sex={:?}",
                position.x, position.y, agent.energy, sex
            ),
        );

        self.agents.push(agent);
        self.metrics.births_this_tick += 1;
        id
    }

    /// Initialize discrete generation mode
    /// Must be called after initial agents are spawned but before first step
    pub fn initialize_discrete_generation(&mut self) {
        if let GenerationMode::Discrete {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            generation_time,
            population_size,
        } = self.config.generation_mode
        {
            // Create the generation manager
            let mut mgr =
                DiscreteGenerationManager::new(population_size, generation_time, self.tick);

            // Initialize first generation with current agents
            let agent_ids: Vec<AgentId> = self.agents.iter().map(|a| a.id).collect();
            let genomes: Vec<Genome> = self.agents.iter().map(|a| a.genome.clone()).collect();
            let allele_freqs = compute_allele_frequencies(&genomes);
            let heterozygosity = compute_heterozygosity(&genomes);

            mgr.initialize_first_generation(agent_ids, allele_freqs.clone(), heterozygosity);

            // Initialize Wright-Fisher model for validation
            // Use average allele frequency across loci for initial p
            let avg_p = if allele_freqs.is_empty() {
                0.5
            } else {
                allele_freqs.iter().map(|f| f.p).sum::<f32>() / allele_freqs.len() as f32
            };
            let wf_model = WrightFisherDiscreteModel::new(population_size, avg_p);

            self.discrete_generation = Some(mgr);
            self.wf_model = Some(wf_model);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute allele frequencies from current population
    #[must_use]
    pub fn compute_allele_frequencies(&self) -> Vec<crate::lineage::AlleleFrequencies> {
        let genomes: Vec<Genome> = self
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| a.genome.clone())
            .collect();
        compute_allele_frequencies(&genomes)
    }

    /// Compute heterozygosity from current population
    #[must_use]
    pub fn compute_heterozygosity(&self) -> f32 {
        let genomes: Vec<Genome> = self
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| a.genome.clone())
            .collect();
        compute_heterozygosity(&genomes)
    }

    /// Run one tick of the simulation
    /// Per §7.6: 14-step agent update loop
    /// Stage 0: Movement, energy, resources, mortality
    /// Stage 1: + Gompertz mortality, reproduction, crossover
    /// Stage 2: + DOCAS state updates, DMN planning
    pub fn step(&mut self) {
        self.tick += 1;
        self.circadian.update(self.tick);

        self.config.optogenetics.step_kinetics(
            self.config.optogenetics.wavelength != crate::optogenetics::OpsinWavelength::LaserOff,
        );

        let alive_count_before = self.living_agent_count();
        self.setup_tick(alive_count_before);
        self.update_environment_and_allee();

        let dmn_enabled = self.config.toggles.dmn_enabled;
        let actions = self.compute_actions();

        let barnes_hut_enabled = self.config.toggles.barnes_hut_enabled;
        if barnes_hut_enabled {
            self.compute_barnes_hut_forces_pre();
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        self.execute_actions(actions, barnes_hut_enabled, alive_count_before, dmn_enabled);

        if self.config.toggles.stigmergy_enabled {
            self.process_stigmergy();
        }

        self.process_encounters_stage();
        self.post_movement_physics();
        self.apply_energy_and_aging();

        if self.config.toggles.docas_enabled {
            self.update_docas_and_psychology();
        }

        self.process_society_and_culture();
        self.process_identity_theft();
        self.collect_resources();

        if self.config.toggles.economics_enabled && self.tick % 10 == 0 {
            self.process_economics();
        }

        self.process_mortality();
        self.process_life_cycle();

        if self.config.toggles.criticality_enabled {
            self.regulate_criticality();
        }

        self.cleanup_and_log();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn setup_tick(&mut self, alive_count_before: usize) {
        debug!(
            "[SimCore] [TickStart] tick={} alive_agents={}",
            self.tick, alive_count_before
        );

        self.metrics.births_this_tick = 0;
        self.metrics.deaths_this_tick = 0;

        self.event_store
            .persist_game_events(&self.game_events_this_tick);
        self.event_store
            .persist_encounter_events(&self.encounter_events_this_tick);
        self.game_events_this_tick.clear();
        self.encounter_events_this_tick.clear();

        self.debug_state.select_sampled_agent(&self.agents);
        if let Some(sampled_id) = self.debug_state.sampled_agent_id {
            if let Some(agent) = self.agents.iter().find(|a| a.id == sampled_id && a.alive) {
                debug!(" [SampledAgent] tick={} agent={} pos=({:.2},{:.2}) energy={:.1} docas=[{:.2},{:.2},{:.2},{:.2},{:.2}]",
                    self.tick, sampled_id, agent.position.x, agent.position.y, agent.energy,
                    agent.docas.levels[0], agent.docas.levels[1], agent.docas.levels[2],
                    agent.docas.levels[3], agent.docas.levels[4]);
            }
        }
    }

    fn update_environment_and_allee(&mut self) {
        if !self.config.toggles.geography_enabled {
            return;
        }
        let env_seed = self.config.seed.wrapping_add(self.tick.wrapping_mul(7919));
        let mut env_rng = StdRng::seed_from_u64(env_seed);
        self.environment.step(&mut env_rng);

        let population = self.agents.iter().filter(|a| a.alive).count() as u32;
        self.environment.update_allee_effects(population);
        self.metrics.allee_fitness_multiplier = self.environment.allee_fitness_multiplier();
        self.metrics.allee_critical_warning = self.environment.allee_effect.critical_risk();
    }

    fn compute_barnes_hut_forces_pre(&mut self) {
        let width = self.environment.width as f32;
        let height = self.environment.height as f32;
        let mut barnes_hut = BarnesHutTree::new(width, height);
        barnes_hut.build(&self.agents);
        barnes_hut.compute_forces(&mut self.agents, &default_social_force);
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn process_encounters_stage(&mut self) {
        if self.config.toggles.encounters_enabled {
            self.process_encounters();
        } else {
            let coop_rate = self.compute_cooperation_rate();
            self.metrics.cooperation_history.push(coop_rate);
        }
    }

    fn post_movement_physics(&mut self) {
        // Post-movement bounds validation and physics cleanup
    }

    fn process_society_and_culture(&mut self) {
        if self.config.toggles.norms_enabled || self.config.toggles.institutions_enabled {
            self.process_stage3_society();
        }

        if self.config.toggles.groups_enabled {
            self.process_group_formation();
        }

        if self.config.toggles.institutions_enabled && self.tick % 100 == 0 {
            self.process_game_interactions();
        }

        if self.config.toggles.language_enabled {
            self.process_stage4_language();
            let current_tick = self.tick;
            for agent in &mut self.agents {
                if agent.alive {
                    let tau = agent.compute_trust();
                    agent.language_drift.record_tau(current_tick, tau);
                    agent
                        .language_drift
                        .record_lexicon(current_tick, &agent.lexicon);
                }
            }
        }

        if self.config.toggles.music_enabled {
            self.process_stage4_music();
        }

        if self.config.toggles.protocol_enabled {
            self.process_stage5_protocol();
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn process_life_cycle(&mut self) {
        if !self.config.toggles.biology_enabled {
            return;
        }

        if !self.config.generation_mode.is_discrete() {
            self.process_reproduction();
        }

        if self.config.generation_mode.is_discrete() {
            let should_replace = if let Some(ref mut mgr) = self.discrete_generation {
                mgr.tick()
            } else {
                false
            };
            if should_replace {
                self.execute_generation_replacement();
            }
        }

        if self.config.toggles.stigmergy_enabled {
            self.pheromone_grid.update();
        }
    }

    fn cleanup_and_log(&mut self) {
        let cleanup_interval = self.config.cleanup_interval as u64;
        let max_dead = self.config.max_dead_agents;
        let dead_count = self.agents.len() - self.living_agent_count();
        let should_cleanup = self.config.cleanup_interval > 0
            && self.tick > 0
            && (self.tick % cleanup_interval == 0 || dead_count > max_dead);
        if should_cleanup {
            self.cleanup_dead_agents();
        }

        if self.tick % 500 == 0 {
            let mem = self.estimate_memory_usage();
            debug!(
                "[SimCore] [Memory] tick={} total={}KB agents={}KB lineage={}KB alive={} dead={} summaries={}",
                self.tick,
                mem.total_bytes / 1024,
                mem.agents_bytes / 1024,
                mem.lineage_bytes / 1024,
                mem.alive_count,
                mem.dead_count,
                mem.summary_count
            );
        }

        let alive_count_after = self.living_agent_count();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let births = self.metrics.births_this_tick;
        let deaths = self.metrics.deaths_this_tick;
        debug!(
            " [TickEnd] tick={} alive={} births={} deaths={} total_repr={} total_deaths={}",
            self.tick,
            alive_count_after,
            births,
            deaths,
            self.debug_state.total_reproductions,
            self.debug_state.total_deaths
        );
    }

    /// Run the simulation for a specified number of ticks
    pub fn run(&mut self, ticks: Tick) {
        for _ in 0..ticks {
            self.step();
        }
    }

    /// Get count of living agents
    #[must_use]
    pub fn living_agent_count(&self) -> usize {
        self.agents.iter().filter(|a| a.alive).count()
    }

    /// Check if any agents are alive
    #[must_use]
    pub fn has_living_agents(&self) -> bool {
        self.agents.iter().any(|a| a.alive)
    }

    /// Remove dead agents to prevent memory leak
    ///
    /// Called periodically (every N ticks based on config) to clean up.
    /// Preserves lineage information for research by converting to summaries.
    ///
    /// # Returns
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Number of dead agents removed
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub fn cleanup_dead_agents(&mut self) -> usize {
        let initial_count = self.agents.len();
        let dead_count = self.agents.iter().filter(|a| !a.alive).count();

        if dead_count == 0 {
            return 0;
        }

        // Record summary statistics before removal
        let dead_ages: Vec<u64> = self
            .agents
            .iter()
            .filter(|a| !a.alive)
            .map(|a| a.age)
            .collect();

        let avg_age = if dead_ages.is_empty() {
            0.0
        } else {
            dead_ages.iter().sum::<u64>() as f32 / dead_ages.len() as f32
        };

        debug!(
            "[SimCore] [Cleanup] Removing {dead_count} dead agents (avg age {avg_age:.1}) out of {initial_count} total"
        );

        // Preserve lineage information before removal if configured
        if self.config.preserve_lineage {
            self.preserve_lineage_history();
        }

        // Remove dead agents
        self.agents.retain(|agent| agent.alive);

        let final_count = self.agents.len();
        let removed = initial_count - final_count;

        debug!(" [Cleanup] Retained {final_count} live agents (freed {removed} dead)");

        removed
    }

    /// Preserve essential lineage information before agent removal
    /// Converts full lineage nodes to compact summaries
    fn preserve_lineage_history(&mut self) {
        // Get IDs of dead agents that need to be converted to summaries
        let dead_agent_ids: Vec<AgentId> = self
            // TAG: surface=simulation owner=research-team rule=SIM-001
            .agents
            .iter()
            .filter(|a| !a.alive)
            .map(|a| a.id)
            .collect();

        // Convert each dead agent's lineage node to a summary
        for agent_id in dead_agent_ids {
            if self.lineage.get_node(agent_id).is_some() {
                self.lineage.convert_to_summary(agent_id);
            }
        }
    }

    /// Get memory usage estimate
    #[must_use]
    pub fn estimate_memory_usage(&self) -> MemoryUsage {
        let agent_bytes = self.agents.len() * std::mem::size_of::<Agent>();
        let lineage_bytes = self.lineage.estimate_size();
        let environment_bytes = self.environment.width * self.environment.height * 4;

        let alive_count = self.agents.iter().filter(|a| a.alive).count();
        let dead_count = self.agents.len() - alive_count;

        MemoryUsage {
            agents_bytes: agent_bytes,
            lineage_bytes,
            environment_bytes,
            total_bytes: agent_bytes + lineage_bytes + environment_bytes,
            alive_count,
            dead_count,
            summary_count: self.lineage.summary_count(),
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
