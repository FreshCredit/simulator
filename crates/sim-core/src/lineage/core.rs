use crate::genome::Genome;
use crate::types::{AgentId, Tick};
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Snapshot of genome traits at a point in time
/// Used to track "brain evolution" across generations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitSnapshot {
    /// Tick when this snapshot was taken
    pub tick: Tick,
    /// DOCAS alpha (decay rates) [D, O, C, A, S]
    pub docas_alpha: [f32; 5],
    /// DOCAS beta (sensitivity) [D, O, C, A, S]
    pub docas_beta: [f32; 5],
    /// Planning depth K
    pub planning_depth: u32,
    /// Planning horizon H
    pub planning_horizon: u32,
    /// Cooperation tendency [0, 1]
    pub cooperation_tendency: f32,
    /// Risk tolerance [0, 1]
    pub risk_tolerance: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl TraitSnapshot {
    /// Create a trait snapshot from a genome at a given tick
    #[must_use]
    pub fn from_genome(genome: &Genome, tick: Tick) -> Self {
        Self {
            tick,
            docas_alpha: genome.docas_alpha(),
            docas_beta: genome.docas_beta(),
            planning_depth: genome.planning_depth(),
            planning_horizon: genome.planning_horizon(),
            cooperation_tendency: genome.cooperation_tendency(),
            risk_tolerance: genome.risk_tolerance(),
        }
    }
}

/// Lineage node representing an agent's ancestry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    /// Agent ID
    pub agent_id: AgentId,
    /// Parent 1 ID (None for initial agents)
    pub parent1_id: Option<AgentId>,
    /// Parent 2 ID (None for asexual reproduction)
    pub parent2_id: Option<AgentId>,
    /// Tick when agent was born
    pub birth_tick: Tick,
    /// Tick when agent died (None if still alive)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub death_tick: Option<Tick>,
    /// Initial genome traits at birth
    pub birth_traits: TraitSnapshot,
    /// Generation number (0 for initial agents)
    pub generation: u32,
    /// Children IDs
    pub children: Vec<AgentId>,
}

impl LineageNode {
    /// Create a new lineage node for an initial (spawned) agent
    #[must_use]
    pub const fn new_initial(
        agent_id: AgentId,
        birth_tick: Tick,
        birth_traits: TraitSnapshot,
    ) -> Self {
        Self {
            agent_id,
            parent1_id: None,
            parent2_id: None,
            birth_tick,
            death_tick: None,
            birth_traits,
            generation: 0,
            children: Vec::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new lineage node for an offspring agent
    #[must_use]
    pub const fn new_offspring(
        agent_id: AgentId,
        parent1_id: AgentId,
        parent2_id: Option<AgentId>,
        birth_tick: Tick,
        birth_traits: TraitSnapshot,
        parent_generation: u32,
    ) -> Self {
        Self {
            agent_id,
            parent1_id: Some(parent1_id),
            parent2_id,
            birth_tick,
            death_tick: None,
            birth_traits,
            generation: parent_generation + 1,
            children: Vec::new(),
        }
    }

    /// Check if this agent is still alive
    #[must_use]
    pub const fn is_alive(&self) -> bool {
        self.death_tick.is_none()
    }

    /// Get lifespan in ticks (returns current age if still alive)
    #[must_use]
    pub const fn lifespan(&self, current_tick: Tick) -> Tick {
        match self.death_tick {
            Some(death) => death - self.birth_tick,
            None => current_tick - self.birth_tick,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Minimal summary of a dead agent for lineage preservation
/// Much smaller than full `LineageNode` for memory efficiency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageSummary {
    /// Agent ID
    pub agent_id: AgentId,
    /// Generation number
    pub generation: u32,
    /// Parent 1 ID (None for initial agents)
    pub parent1_id: Option<AgentId>,
    /// Parent 2 ID (None for asexual reproduction)
    pub parent2_id: Option<AgentId>,
    /// Tick when agent was born
    pub birth_tick: Tick,
    /// Tick when agent died
    pub death_tick: Tick,
    /// Final trust value (tau)
    pub final_tau: f32,
    /// Number of offspring
    pub offspring_count: usize,
    /// Children IDs for genealogical continuity
    pub children: Vec<AgentId>,
}

impl LineageSummary {
    /// Create a summary from a full lineage node
    #[must_use]
    pub fn from_node(node: &LineageNode) -> Self {
        Self {
            agent_id: node.agent_id,
            generation: node.generation,
            parent1_id: node.parent1_id,
            parent2_id: node.parent2_id,
            birth_tick: node.birth_tick,
            death_tick: node.death_tick.unwrap_or(node.birth_tick),
            final_tau: node.birth_traits.cooperation_tendency, // Approximation
            offspring_count: node.children.len(),
            children: node.children.clone(),
        }
    }
}
