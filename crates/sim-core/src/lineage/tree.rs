use super::core::{LineageNode, LineageSummary, TraitSnapshot};
use crate::types::{AgentId, Tick};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Lineage tree storing all agent ancestry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineageTree {
    /// All lineage nodes indexed by agent ID
    nodes: std::collections::HashMap<AgentId, LineageNode>,
    /// Summaries of dead agents (compact representation)
    summaries: std::collections::HashMap<AgentId, LineageSummary>,
    /// Maximum generation observed
    max_generation: u32,
    /// Total agents ever born
    total_births: u64,
    /// Total agents that have died
    total_deaths: u64,
}

impl LineageTree {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new empty lineage tree
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an initial (spawned) agent to the lineage
    pub fn add_initial(&mut self, node: LineageNode) {
        self.total_births += 1;
        self.nodes.insert(node.agent_id, node);
    }

    /// Add an offspring agent to the lineage
    pub fn add_offspring(&mut self, node: LineageNode) {
        self.total_births += 1;
        if node.generation > self.max_generation {
            self.max_generation = node.generation;
        }

        // Update parent's children list
        if let Some(parent1_id) = node.parent1_id {
            if let Some(parent) = self.nodes.get_mut(&parent1_id) {
                parent.children.push(node.agent_id);
            } else if let Some(parent_summary) = self.summaries.get_mut(&parent1_id) {
                parent_summary.children.push(node.agent_id);
                parent_summary.offspring_count = parent_summary.children.len();
            }
        }
        if let Some(parent2_id) = node.parent2_id {
            if let Some(parent) = self.nodes.get_mut(&parent2_id) {
                if !parent.children.contains(&node.agent_id) {
                    parent.children.push(node.agent_id);
                }
            } else if let Some(parent_summary) = self.summaries.get_mut(&parent2_id) {
                if !parent_summary.children.contains(&node.agent_id) {
                    parent_summary.children.push(node.agent_id);
                    parent_summary.offspring_count = parent_summary.children.len();
                }
            }
        }

        self.nodes.insert(node.agent_id, node);
    }

    /// Record an agent's death
    pub fn record_death(&mut self, agent_id: AgentId, death_tick: Tick) {
        if let Some(node) = self.nodes.get_mut(&agent_id) {
            node.death_tick = Some(death_tick);
            self.total_deaths += 1;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a lineage node by agent ID
    #[must_use]
    pub fn get_node(&self, agent_id: AgentId) -> Option<&LineageNode> {
        self.nodes.get(&agent_id)
    }

    /// Get all ancestors of an agent (parents, grandparents, etc.)
    #[must_use]
    pub fn get_ancestors(&self, agent_id: AgentId) -> Vec<AgentId> {
        let mut ancestors = Vec::new();
        let mut to_visit = Vec::new();

        if let Some(node) = self.nodes.get(&agent_id) {
            if let Some(p1) = node.parent1_id {
                to_visit.push(p1);
            }
            if let Some(p2) = node.parent2_id {
                to_visit.push(p2);
            }
        }

        while let Some(id) = to_visit.pop() {
            if !ancestors.contains(&id) {
                ancestors.push(id);
                if let Some(node) = self.nodes.get(&id) {
                    if let Some(p1) = node.parent1_id {
                        to_visit.push(p1);
                    }
                    if let Some(p2) = node.parent2_id {
                        to_visit.push(p2);
                    }
                }
            }
        }

        ancestors
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get all descendants of an agent (children, grandchildren, etc.)
    #[must_use]
    pub fn get_descendants(&self, agent_id: AgentId) -> Vec<AgentId> {
        let mut descendants = Vec::new();
        let mut to_visit = Vec::new();

        if let Some(node) = self.nodes.get(&agent_id) {
            to_visit.extend(node.children.iter().copied());
        }

        while let Some(id) = to_visit.pop() {
            if !descendants.contains(&id) {
                descendants.push(id);
                if let Some(node) = self.nodes.get(&id) {
                    to_visit.extend(node.children.iter().copied());
                }
            }
        }

        descendants
    }

    /// Get lineage depth (number of generations to root)
    #[must_use]
    pub fn get_lineage_depth(&self, agent_id: AgentId) -> u32 {
        self.nodes.get(&agent_id).map_or(0, |n| n.generation)
    }

    /// Get children IDs for an agent (whether living or dead summary)
    #[must_use]
    pub fn get_children(&self, agent_id: AgentId) -> &[AgentId] {
        if let Some(node) = self.nodes.get(&agent_id) {
            &node.children
        } else if let Some(summary) = self.summaries.get(&agent_id) {
            &summary.children
        } else {
            &[]
        }
    }

    /// Compute cumulative counts of children and grandchildren across all generations
    #[must_use]
    pub fn get_children_and_grandchildren_counts(&self, agent_id: AgentId) -> (u32, u32) {
        let children = self.get_children(agent_id);
        let children_count = children.len() as u32;
        let mut grandchildren_count = 0u32;
        for &child_id in children {
            grandchildren_count += self.get_children(child_id).len() as u32;
        }
        (children_count, grandchildren_count)
    }

    /// Get maximum generation observed
    #[must_use]
    pub const fn max_generation(&self) -> u32 {
        self.max_generation
    }

    /// Get total number of agents ever born
    #[must_use]
    pub const fn total_births(&self) -> u64 {
        self.total_births
    }

    /// Get total number of agents that have died
    #[must_use]
    pub const fn total_deaths(&self) -> u64 {
        self.total_deaths
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get number of currently living agents
    #[must_use]
    pub fn living_count(&self) -> usize {
        self.nodes.values().filter(|n| n.is_alive()).count()
    }

    /// Add a summary for a dead agent (minimal memory footprint)
    pub fn add_summary(&mut self, summary: LineageSummary) {
        self.summaries.insert(summary.agent_id, summary);
    }

    /// Get summary for a dead agent
    #[must_use]
    pub fn get_summary(&self, agent_id: AgentId) -> Option<&LineageSummary> {
        self.summaries.get(&agent_id)
    }

    /// Check if agent exists as either a full node or a summary
    #[must_use]
    pub fn has_agent(&self, agent_id: AgentId) -> bool {
        self.nodes.contains_key(&agent_id) || self.summaries.contains_key(&agent_id)
    }

    /// Remove a full node and replace with a summary (for memory cleanup)
    /// Returns true if the node was found and converted
    pub fn convert_to_summary(&mut self, agent_id: AgentId) -> bool {
        if let Some(node) = self.nodes.remove(&agent_id) {
            let summary = LineageSummary::from_node(&node);
            self.summaries.insert(agent_id, summary);
            true
        } else {
            false
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get total number of agents tracked (nodes + summaries)
    #[must_use]
    pub fn total_tracked(&self) -> usize {
        self.nodes.len() + self.summaries.len()
    }

    /// Estimate memory usage in bytes
    #[must_use]
    pub fn estimate_size(&self) -> usize {
        let nodes_size = self.nodes.len() * std::mem::size_of::<LineageNode>();
        let summaries_size = self.summaries.len() * std::mem::size_of::<LineageSummary>();
        nodes_size + summaries_size
    }

    /// Get count of summaries (dead agents converted to compact form)
    #[must_use]
    pub fn summary_count(&self) -> usize {
        self.summaries.len()
    }

    /// Get all agents in a specific generation
    #[must_use]
    pub fn get_generation(&self, gen: u32) -> Vec<AgentId> {
        self.nodes
            .values()
            .filter(|n| n.generation == gen)
            .map(|n| n.agent_id)
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute average trait value across a generation
    /// Returns (`avg_cooperation`, `avg_risk_tolerance`)
    #[must_use]
    pub fn generation_trait_averages(&self, gen: u32) -> Option<(f32, f32)> {
        let gen_nodes: Vec<_> = self
            .nodes
            .values()
            .filter(|n| n.generation == gen)
            .collect();
        if gen_nodes.is_empty() {
            return None;
        }

        let coop_sum: f32 = gen_nodes
            .iter()
            .map(|n| n.birth_traits.cooperation_tendency)
            .sum();
        let risk_sum: f32 = gen_nodes
            .iter()
            .map(|n| n.birth_traits.risk_tolerance)
            .sum();
        let count = gen_nodes.len() as f32;

        Some((coop_sum / count, risk_sum / count))
    }

    /// Compute heritability estimate for a trait across generations
    /// Uses parent-offspring regression (simplified)
    /// Returns slope (heritability estimate h²)
    pub fn estimate_heritability(&self, trait_fn: impl Fn(&TraitSnapshot) -> f32) -> Option<f32> {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut parent_values = Vec::new();
        let mut offspring_values = Vec::new();

        for node in self.nodes.values() {
            if let Some(parent1_id) = node.parent1_id {
                if let Some(parent) = self.nodes.get(&parent1_id) {
                    let parent_val = trait_fn(&parent.birth_traits);
                    let offspring_val = trait_fn(&node.birth_traits);

                    // If two parents, average their values
                    let avg_parent_val = if let Some(parent2_id) = node.parent2_id {
                        if let Some(parent2) = self.nodes.get(&parent2_id) {
                            (parent_val + trait_fn(&parent2.birth_traits)) / 2.0
                        } else {
                            parent_val
                        }
                    } else {
                        parent_val
                    };

                    parent_values.push(avg_parent_val);
                    offspring_values.push(offspring_val);
                }
            }
        }

        if parent_values.len() < 2 {
            return None;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Simple linear regression slope = Cov(X,Y) / Var(X)
        let n = parent_values.len() as f32;
        let mean_x: f32 = parent_values.iter().sum::<f32>() / n;
        let mean_y: f32 = offspring_values.iter().sum::<f32>() / n;

        let cov: f32 = parent_values
            .iter()
            .zip(offspring_values.iter())
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum::<f32>()
            / n;

        let var_x: f32 = parent_values
            .iter()
            .map(|x| (x - mean_x).powi(2))
            .sum::<f32>()
            / n;

        if var_x < 0.0001 {
            return None;
        }

        Some(cov / var_x)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute brain evolution statistics across all generations
    /// Returns summary of DOCAS sensitivity evolution
    #[must_use]
    pub fn compute_brain_evolution(&self) -> BrainEvolutionStats {
        let mut stats = BrainEvolutionStats::default();

        for gen in 0..=self.max_generation {
            let gen_nodes: Vec<_> = self
                .nodes
                .values()
                .filter(|n| n.generation == gen)
                .collect();
            if gen_nodes.is_empty() {
                continue;
            }

            let count = gen_nodes.len() as f32;

            // Compute average DOCAS beta (sensitivity) for this generation
            let mut avg_beta = [0.0f32; 5];
            for node in &gen_nodes {
                for (i, beta) in avg_beta.iter_mut().enumerate() {
                    *beta += node.birth_traits.docas_beta[i];
                }
            }
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            for beta in &mut avg_beta {
                *beta /= count;
            }

            stats.generation_avg_beta.push((gen, avg_beta));
            stats.generation_population.push((gen, gen_nodes.len()));
        }

        // Compute heritability for each DOCAS dimension
        for i in 0..5 {
            if let Some(h2) = self.estimate_heritability(|t| t.docas_beta[i]) {
                stats.docas_heritability[i] = h2;
            }
        }

        // Compute cooperation heritability
        stats.cooperation_heritability = self
            .estimate_heritability(|t| t.cooperation_tendency)
            .unwrap_or(0.0);

        // Compute risk tolerance heritability
        stats.risk_heritability = self
            .estimate_heritability(|t| t.risk_tolerance)
            .unwrap_or(0.0);

        stats
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get kin (relatives) of an agent up to a specified degree
    /// Per SIM-005: Kin spillover mechanics
    ///
    /// degree = 1: parents, children, siblings
    /// degree = 2: grandparents, grandchildren, aunts/uncles, nieces/nephews, cousins
    #[must_use]
    pub fn get_kin(&self, agent_id: AgentId, degree: u32) -> Vec<AgentId> {
        let mut kin = std::collections::HashSet::new();

        let Some(node) = self.nodes.get(&agent_id) else {
            return kin.into_iter().collect();
        };

        if degree >= 1 {
            self.collect_degree1_kin(node, agent_id, &mut kin);
        }
        if degree >= 2 {
            self.collect_degree2_kin(node, agent_id, &mut kin);
        }

        kin.into_iter().collect()
    }

    fn collect_degree1_kin(
        &self,
        node: &LineageNode,
        agent_id: AgentId,
        kin: &mut std::collections::HashSet<AgentId>,
    ) {
        if let Some(p1) = node.parent1_id {
            kin.insert(p1);
        }
        if let Some(p2) = node.parent2_id {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            kin.insert(p2);
        }
        for &child_id in &node.children {
            kin.insert(child_id);
        }
        if let Some(parent1_id) = node.parent1_id {
            if let Some(parent) = self.nodes.get(&parent1_id) {
                for &sibling_id in &parent.children {
                    if sibling_id != agent_id {
                        kin.insert(sibling_id);
                    }
                }
            }
        }
    }

    fn collect_degree2_kin(
        &self,
        node: &LineageNode,
        agent_id: AgentId,
        kin: &mut std::collections::HashSet<AgentId>,
    ) {
        self.collect_grandparents(node, kin);
        self.collect_grandchildren(node, kin);
        self.collect_aunts_uncles(node, kin);
        self.collect_nieces_nephews(node, agent_id, kin);
    }

    fn collect_grandparents(
        &self,
        node: &LineageNode,
        kin: &mut std::collections::HashSet<AgentId>,
    ) {
        for parent_id in [node.parent1_id, node.parent2_id].iter().copied().flatten() {
            if let Some(parent) = self.nodes.get(&parent_id) {
                if let Some(gp1) = parent.parent1_id {
                    kin.insert(gp1);
                }
                if let Some(gp2) = parent.parent2_id {
                    kin.insert(gp2);
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn collect_grandchildren(
        &self,
        node: &LineageNode,
        kin: &mut std::collections::HashSet<AgentId>,
    ) {
        for &child_id in &node.children {
            if let Some(child) = self.nodes.get(&child_id) {
                for &grandchild_id in &child.children {
                    kin.insert(grandchild_id);
                }
            }
        }
    }

    fn collect_aunts_uncles(
        &self,
        node: &LineageNode,
        kin: &mut std::collections::HashSet<AgentId>,
    ) {
        let Some(parent1_id) = node.parent1_id else {
            return;
        };
        let Some(parent1) = self.nodes.get(&parent1_id) else {
            return;
        };
        let Some(grandparent_id) = parent1.parent1_id else {
            return;
        };
        let Some(grandparent) = self.nodes.get(&grandparent_id) else {
            return;
        };
        for &aunt_uncle_id in &grandparent.children {
            if aunt_uncle_id != parent1_id {
                kin.insert(aunt_uncle_id);
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn collect_nieces_nephews(
        &self,
        node: &LineageNode,
        agent_id: AgentId,
        kin: &mut std::collections::HashSet<AgentId>,
    ) {
        let Some(parent1_id) = node.parent1_id else {
            return;
        };
        let Some(parent) = self.nodes.get(&parent1_id) else {
            return;
        };
        for &sibling_id in &parent.children {
            if sibling_id == agent_id {
                continue;
            }
            if let Some(sibling) = self.nodes.get(&sibling_id) {
                for &niece_nephew_id in &sibling.children {
                    kin.insert(niece_nephew_id);
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if two agents are kin (related by blood)
    /// Returns true if they share a common ancestor within 2 generations
    #[must_use]
    pub fn are_kin(&self, agent1_id: AgentId, agent2_id: AgentId) -> bool {
        if agent1_id == agent2_id {
            return false; // Same agent is not kin to itself
        }

        let kin1 = self.get_kin(agent1_id, 2);
        kin1.contains(&agent2_id)
    }

    /// Get kinship degree between two agents
    /// Returns Some(1) for parents/children/siblings
    /// Returns Some(2) for grandparents/grandchildren/aunts/uncles/etc
    /// Returns None if not kin
    #[must_use]
    pub fn kinship_degree(&self, agent1_id: AgentId, agent2_id: AgentId) -> Option<u32> {
        if agent1_id == agent2_id {
            return None;
        }

        let kin1 = self.get_kin(agent1_id, 1);
        if kin1.contains(&agent2_id) {
            return Some(1);
        }

        let kin2 = self.get_kin(agent1_id, 2);
        if kin2.contains(&agent2_id) {
            return Some(2);
        }

        None
    }

    // ============================================================================
    // COALESCENT INTEGRATION - MRCA Detection via Kingman Coalescent
    // ============================================================================

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Build coalescent tree from lineage data for a set of sampled agents
    ///
    /// Extracts the genealogy for sampled agents and constructs a coalescent
    /// representation suitable for MRCA analysis.
    ///
    /// # Arguments
    /// * `sample_ids` - Agent IDs to include in the coalescent tree
    ///
    /// # Returns
    /// * `Option<KingmanCoalescent>` - Coalescent tree if valid genealogy exists
    ///
    /// # Example
    /// ```
    /// use sim_core::lineage::LineageTree;
    ///
    /// # fn example() {
    /// # let tree = LineageTree::new();
    /// # let sample = vec![1u64, 2u64, 3u64];
    /// # let coalescent = tree.build_coalescent(&sample);
    /// # }
    /// ```
    #[must_use]
    pub fn build_coalescent(
        &self,
        sample_ids: &[AgentId],
    ) -> Option<crate::genealogy::coalescent::KingmanCoalescent> {
        crate::genealogy::coalescent::build_coalescent_from_lineage(sample_ids, self)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute pairwise coalescence times for a set of agents
    ///
    /// For all pairs of agents, finds their Most Recent Common Ancestor
    /// and returns the time (in generations) to coalescence.
    ///
    /// # Arguments
    /// * `agent_ids` - Agent IDs to analyze
    ///
    /// # Returns
    /// * `HashMap<(AgentId, AgentId), u32>` - Coalescence generation for each pair
    ///
    /// # Mathematical Properties
    /// Under the neutral coalescent, coalescence times follow an exponential
    /// distribution with mean 2N/(k(k-1)) for k lineages.
    #[must_use]
    pub fn pairwise_coalescence_times(
        &self,
        agent_ids: &[AgentId],
    ) -> HashMap<(AgentId, AgentId), u32> {
        let mut times = HashMap::new();

        for (i, &agent_i) in agent_ids.iter().enumerate() {
            for &agent_j in agent_ids.iter().skip(i + 1) {
                // Find MRCA of this pair
                if let Some((_mrca_id, generation)) = self.find_pairwise_mrca(agent_i, agent_j) {
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    // Time is generations back from most recent generation
                    let node_i = self.get_node(agent_i);
                    let node_j = self.get_node(agent_j);

                    let max_gen = node_i
                        .and_then(|n1| node_j.map(|n2| n1.generation.max(n2.generation)))
                        .unwrap_or(0);

                    let time_back = max_gen.saturating_sub(generation);

                    times.insert((agent_i, agent_j), time_back);
                    times.insert((agent_j, agent_i), time_back); // Symmetric
                }
            }
        }

        times
    }

    /// Find MRCA of two agents specifically
    fn find_pairwise_mrca(&self, agent1_id: AgentId, agent2_id: AgentId) -> Option<(AgentId, u32)> {
        // Collect ancestors of agent 1
        let mut ancestors1: HashMap<AgentId, u32> = HashMap::new();
        self.collect_ancestors_with_depth(agent1_id, &mut ancestors1, 0);

        // Collect ancestors of agent 2
        let mut ancestors2: HashMap<AgentId, u32> = HashMap::new();
        self.collect_ancestors_with_depth(agent2_id, &mut ancestors2, 0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Find common ancestors and pick most recent (highest generation)
        ancestors1
            .iter()
            .filter_map(|(ancestor, _gen1)| {
                ancestors2.get(ancestor).and_then(|_gen2| {
                    // Use the actual generation from the node
                    self.get_node(*ancestor)
                        .map(|node| (*ancestor, node.generation))
                })
            })
            .max_by_key(|(_, gen)| *gen)
    }

    /// Helper to collect ancestors with their depth
    fn collect_ancestors_with_depth(
        &self,
        agent_id: AgentId,
        ancestors: &mut HashMap<AgentId, u32>,
        depth: u32,
    ) {
        if let Some(node) = self.get_node(agent_id) {
            if let Some(p1) = node.parent1_id {
                ancestors.insert(p1, depth + 1);
                self.collect_ancestors_with_depth(p1, ancestors, depth + 1);
            }
            if let Some(p2) = node.parent2_id {
                ancestors.insert(p2, depth + 1);
                self.collect_ancestors_with_depth(p2, ancestors, depth + 1);
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute Tajima's D statistic using coalescent approach
    ///
    /// Uses the coalescent tree structure to compute Tajima's D,
    /// which is more efficient than allele counting for large samples.
    ///
    /// # Arguments
    /// * `sample_ids` - Agent IDs to include in calculation
    ///
    /// # Returns
    /// * `Option<f32>` - Tajima's D value, or None if insufficient data
    ///
    /// # Interpretation
    /// * D ≈ 0: Neutral evolution
    /// * D < -2: Purifying selection or population expansion
    /// * D > 2: Balancing selection or population contraction
    #[must_use]
    pub fn tajimas_d_coalescent(&self, sample_ids: &[AgentId]) -> Option<f32> {
        if sample_ids.len() < 2 {
            return None;
        }

        let times = self.pairwise_coalescence_times(sample_ids);
        if times.is_empty() {
            return None;
        }

        // Compute average pairwise coalescence time (generations)
        let avg_time: f64 = times.values().map(|&t| t as f64).sum::<f64>() / times.len() as f64;

        // Estimate segregating sites from total branch length
        // S ≈ θ * L where L is total branch length
        let total_branch_length: f64 = times.values().map(|&t| t as f64).sum();

        let n = sample_ids.len() as f64;
        let a1: f64 = (1..sample_ids.len()).map(|i| 1.0 / i as f64).sum();

        if a1 == 0.0 {
            return None;
        }

        // S/a1 estimator
        let s_estimated = total_branch_length / 2.0;
        let s_over_a1 = s_estimated / a1;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Variance terms
        let a2: f64 = (1..sample_ids.len()).map(|i| 1.0 / (i * i) as f64).sum();
        let b1 = (n + 1.0) / (3.0 * (n - 1.0));
        let b2 = 2.0 * (n * n + n + 3.0) / (9.0 * n * (n - 1.0));
        let c1 = b1 - 1.0 / a1;
        let c2 = b2 - (n + 2.0) / (a1 * n) + a2 / (a1 * a1);
        let e1 = c1 / a1;
        let e2 = c2 / (a1 * a1 + a2);

        let denominator = (e1 * s_estimated + e2 * s_estimated * (s_estimated - 1.0)).sqrt();

        if denominator == 0.0 {
            return None;
        }

        Some(((avg_time - s_over_a1) / denominator) as f32)
    }
}

/// Statistics about brain (DOCAS sensitivity) evolution across generations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrainEvolutionStats {
    /// Average DOCAS beta per generation: (generation, [D, O, C, A, S])
    pub generation_avg_beta: Vec<(u32, [f32; 5])>,
    /// Population size per generation
    pub generation_population: Vec<(u32, usize)>,
    /// Heritability (h²) for each DOCAS dimension [D, O, C, A, S]
    pub docas_heritability: [f32; 5],
    /// Heritability for cooperation tendency
    pub cooperation_heritability: f32,
    /// Heritability for risk tolerance
    pub risk_heritability: f32,
}
