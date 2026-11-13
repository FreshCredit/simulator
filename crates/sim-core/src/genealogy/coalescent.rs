//! Kingman Coalescent for MRCA Detection
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-004: Deterministic - all coalescent operations use seeded RNG
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! This module implements the Kingman coalescent process for efficient
//! Most Recent Common Ancestor (MRCA) calculations and genealogical analysis.
//!
//! ## Mathematical Foundation
//!
//! The Kingman coalescent is a backward-time stochastic process that models
//! the ancestry of a sample of lineages. Starting with n lineages at present,
//! the process traces backward in time until all lineages coalesce to a single
//! common ancestor (the MRCA).
//!
//! ### Key Properties:
//! - **Coalescence Rate**: For k lineages, coalescence occurs at rate λ = C(k,2) = k(k-1)/2
//! - **Waiting Time**: Time to next coalescence ~ Exponential(λ) with mean 2/(k(k-1))
//! - **Total Tree Height**: E\[`T_MRCA`\] = 2(1 - 1/n) in coalescent units (2N generations)
//!
//! ## References:
//! - Kingman (1982): "The Coalescent" - Stochastic Processes and their Applications
//! - Hudson (1990): "Gene genealogies and the coalescent process"
//! - Hein, Schierup & Wiuf (2005): "Gene Genealogies, Variation and Evolution"

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
use crate::lineage::LineageTree;
use crate::types::AgentId;

/// Node identifier in coalescent tree
pub type NodeId = usize;

/// Node in coalescent tree
///
/// Represents either a sampled lineage (leaf) or an ancestral coalescence (internal node).
/// The tree is stored as a directed acyclic graph with parent pointers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoalescentNode {
    /// Unique node identifier
    pub id: NodeId,
    /// Descendant lineages (all sampled agents below this node)
    pub lineage: Vec<AgentId>,
    /// Time before present in coalescent units (2N generations)
    pub time: f64,
    /// Parent node (None for MRCA)
    pub parent: Option<NodeId>,
    /// Child nodes (empty for leaves)
    pub children: Vec<NodeId>,
    /// True if this is a sampled lineage (leaf)
    pub is_leaf: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl CoalescentNode {
    /// Create a new leaf node for a sampled lineage
    #[must_use]
    pub fn new_leaf(id: NodeId, agent_id: AgentId) -> Self {
        Self {
            id,
            lineage: vec![agent_id],
            time: 0.0,
            parent: None,
            children: vec![],
            is_leaf: true,
        }
    }

    /// Create a new internal node from two child nodes
    #[must_use]
    pub fn new_internal(
        id: NodeId,
        children: &[NodeId],
        time: f64,
        child_lineages: Vec<Vec<AgentId>>,
    ) -> Self {
        let mut lineage = Vec::new();
        for child_lin in child_lineages {
            lineage.extend(child_lin);
        }

        Self {
            id,
            lineage,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            time,
            parent: None,
            children: children.to_vec(),
            is_leaf: false,
        }
    }

    /// Get the number of descendant lineages
    #[must_use]
    pub fn n_descendants(&self) -> usize {
        self.lineage.len()
    }
}

/// Kingman coalescent process
///
/// Encapsulates a complete coalescent tree simulation and provides
/// methods for analyzing genealogical relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KingmanCoalescent {
    /// All nodes in the coalescent tree
    pub nodes: Vec<CoalescentNode>,
    /// Number of sampled lineages
    pub n_samples: usize,
    /// Total tree height (time to MRCA)
    pub total_tree_height: f64,
    /// Time to most recent common ancestor
    pub mrca_time: f64,
    /// MRCA node identifier
    pub mrca_node: Option<NodeId>,
}

impl KingmanCoalescent {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Simulate coalescent process for n lineages
    ///
    /// Uses the standard Kingman coalescent algorithm:
    /// 1. Initialize n active lineages at time 0
    /// 2. While more than 1 lineage remains:
    ///    - Sample waiting time ~ Exponential(k(k-1)/2)
    ///    - Choose pair uniformly to coalesce
    ///    - Create new ancestral node
    /// 3. Return when single MRCA remains
    ///
    /// # Arguments
    /// * `n` - Number of sampled lineages
    /// * `rng` - Random number generator (for deterministic simulation)
    ///
    /// # Returns
    /// * `KingmanCoalescent` - Complete coalescent tree
    ///
    /// # Example
    /// ```
    /// use rand::SeedableRng;
    /// use rand::rngs::StdRng;
    /// use sim_core::genealogy::coalescent::KingmanCoalescent;
    ///
    /// let mut rng = StdRng::seed_from_u64(42);
    /// let coal = KingmanCoalescent::simulate(10, &mut rng);
    ///
    /// debug_assert_eq!(coal.n_samples, 10);
    /// debug_assert!(coal.mrca_time > 0.0);
    /// ```
    pub fn simulate(n: usize, rng: &mut StdRng) -> Self {
        Self::simulate_with_intensity(n, 1.0, rng)
    }

    /// Simulate coalescent process for n lineages with Kingman intensity κ* (Sim K-Series)
    ///
    /// Coalescence occurs at rate λ = C(k,2) · κ*.
    /// Higher intensity accelerates coalescence, reducing T_MRCA (Sim K2).
    pub fn simulate_with_intensity(n: usize, intensity: f64, rng: &mut StdRng) -> Self {
        if n == 0 {
            return Self {
                nodes: Vec::new(),
                n_samples: 0,
                total_tree_height: 0.0,
                mrca_time: 0.0,
                mrca_node: None,
            };
        }

        if n == 1 {
            let node = CoalescentNode::new_leaf(0, AgentId::from(0u64));
            return Self {
                nodes: vec![node],
                n_samples: 1,
                total_tree_height: 0.0,
                mrca_time: 0.0,
                mrca_node: Some(0),
            };
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let mut active_nodes: Vec<NodeId> = (0..n).collect();
        let mut nodes: Vec<CoalescentNode> = Vec::new();
        let mut current_time: f64 = 0.0;
        let mut next_id = n;

        // Initialize leaf nodes
        for i in 0..n {
            nodes.push(CoalescentNode::new_leaf(i, AgentId::from(i as u64)));
        }

        let kappa = intensity.max(0.01);

        // Coalescence process
        // P10-R4: bounded — each iteration merges one pair, shrinking active_nodes to 1.
        while active_nodes.len() > 1 {
            let k = active_nodes.len();
            let coalescence_rate = ((k * (k - 1)) as f64 / 2.0) * kappa;

            // Sample waiting time ~ Exponential(rate)
            // Using inverse CDF: T = -ln(U) / λ where U ~ Uniform(0,1)
            let u: f64 = rng.gen::<f64>().clamp(1e-15, 1.0 - 1e-15);
            let waiting_time = -u.ln() / coalescence_rate;
            current_time += waiting_time;

            // Choose pair to coalesce uniformly from k choose 2
            // Use integer sampling without replacement
            let i = rng.gen_range(0..k);
            let j = rng.gen_range(0..k - 1);
            let j = if j >= i { j + 1 } else { j };
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001

            let node_i = active_nodes[i];
            let node_j = active_nodes[j];

            // Collect child lineages
            let child_lineages = vec![nodes[node_i].lineage.clone(), nodes[node_j].lineage.clone()];

            // Create new ancestral node
            let new_node = CoalescentNode::new_internal(
                next_id,
                &[node_i, node_j],
                current_time,
                child_lineages,
            );

            // Update parent pointers
            nodes[node_i].parent = Some(next_id);
            nodes[node_j].parent = Some(next_id);

            nodes.push(new_node);

            // Update active nodes: remove i and j, add new node
            // Remove larger index first to maintain correct indices
            let (remove_first, remove_second) = if i > j { (i, j) } else { (j, i) };
            active_nodes.remove(remove_first);
            active_nodes.remove(remove_second);
            active_nodes.push(next_id);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            next_id += 1;
        }

        let mrca_node = active_nodes.first().copied();

        Self {
            nodes,
            n_samples: n,
            total_tree_height: current_time,
            mrca_time: current_time,
            mrca_node,
        }
    }

    /// Simulate coalescent with custom agent IDs
    ///
    /// Similar to `simulate()`, but allows specifying the actual `AgentIds`
    /// for the sampled lineages instead of using sequential integers.
    ///
    /// # Arguments
    /// * `agent_ids` - Agent IDs for sampled lineages
    /// * `rng` - Random number generator
    ///
    /// # Returns
    /// * `KingmanCoalescent` - Complete coalescent tree
    pub fn simulate_with_ids(agent_ids: &[AgentId], rng: &mut StdRng) -> Self {
        let n = agent_ids.len();

        if n == 0 {
            return Self {
                nodes: Vec::new(),
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                n_samples: 0,
                total_tree_height: 0.0,
                mrca_time: 0.0,
                mrca_node: None,
            };
        }

        if n == 1 {
            let node = CoalescentNode::new_leaf(0, agent_ids[0]);
            return Self {
                nodes: vec![node],
                n_samples: 1,
                total_tree_height: 0.0,
                mrca_time: 0.0,
                mrca_node: Some(0),
            };
        }

        let mut active_nodes: Vec<NodeId> = (0..n).collect();
        let mut nodes: Vec<CoalescentNode> = Vec::new();
        let mut current_time: f64 = 0.0;
        let mut next_id = n;

        // Initialize leaf nodes with actual agent IDs
        for (i, &agent_id) in agent_ids.iter().enumerate() {
            nodes.push(CoalescentNode::new_leaf(i, agent_id));
        }

        // Coalescence process
        while active_nodes.len() > 1 {
            let k = active_nodes.len();
            let coalescence_rate = (k * (k - 1)) as f64 / 2.0;

            let u: f64 = rng.gen::<f64>().clamp(1e-15, 1.0 - 1e-15);
            let waiting_time = -u.ln() / coalescence_rate;
            current_time += waiting_time;

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let i = rng.gen_range(0..k);
            let j = rng.gen_range(0..k - 1);
            let j = if j >= i { j + 1 } else { j };

            let node_i = active_nodes[i];
            let node_j = active_nodes[j];

            let child_lineages = vec![nodes[node_i].lineage.clone(), nodes[node_j].lineage.clone()];

            let new_node = CoalescentNode::new_internal(
                next_id,
                &[node_i, node_j],
                current_time,
                child_lineages,
            );

            nodes[node_i].parent = Some(next_id);
            nodes[node_j].parent = Some(next_id);

            nodes.push(new_node);

            let (remove_first, remove_second) = if i > j { (i, j) } else { (j, i) };
            active_nodes.remove(remove_first);
            active_nodes.remove(remove_second);
            active_nodes.push(next_id);

            next_id += 1;
        }

        let mrca_node = active_nodes.first().copied();

        Self {
            nodes,
            n_samples: n,
            total_tree_height: current_time,
            mrca_time: current_time,
            mrca_node,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Expected time to MRCA for n lineages
    ///
    /// Theoretical expectation: E\[`T_MRCA`\] = 2(1 - 1/n) in coalescent units
    ///
    /// # Arguments
    /// * `n` - Number of lineages
    ///
    /// # Returns
    /// * `f64` - Expected time to MRCA
    ///
    /// # Example
    /// ```
    /// use sim_core::genealogy::coalescent::KingmanCoalescent;
    ///
    /// // For n=2: E[T] = 2(1 - 1/2) = 1.0
    /// debug_assert!((KingmanCoalescent::expected_tmrca(2) - 1.0).abs() < 0.001);
    ///
    /// // For n→∞: E[T] → 2.0
    /// debug_assert!((KingmanCoalescent::expected_tmrca(1000) - 2.0).abs() < 0.01);
    /// ```
    #[must_use]
    pub fn expected_tmrca(n: usize) -> f64 {
        if n <= 1 {
            0.0
        } else {
            2.0 * (1.0 - 1.0 / n as f64)
        }
    }

    /// Variance of time to MRCA
    ///
    /// Theoretical variance: Var\[`T_MRCA`\] = sum_{k=2}^n (2/(k(k-1)))^2
    #[must_use]
    pub fn variance_tmrca(n: usize) -> f64 {
        if n <= 1 {
            0.0
        } else {
            (2..=n)
                .map(|k| {
                    let rate = (k * (k - 1)) as f64 / 2.0;
                    1.0 / (rate * rate) // Variance of exponential
                                        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                })
                .sum()
        }
    }

    /// Expected total branch length of the coalescent tree
    ///
    /// Theoretical expectation: E\[`L_n`\] = 2 * sum_{k=2}^n 1/(k-1) = 2 * H_{n-1}
    /// where H_{n-1} is the (n-1)th harmonic number
    ///
    /// # Arguments
    /// * `n` - Number of lineages
    ///
    /// # Returns
    /// * `f64` - Expected total branch length
    #[must_use]
    pub fn expected_total_branch_length(n: usize) -> f64 {
        if n <= 1 {
            0.0
        } else {
            // 2 * harmonic_number(n-1)
            let harmonic: f64 = (1..n).map(|k| 1.0 / k as f64).sum();
            2.0 * harmonic
        }
    }

    /// Compute total branch length of the simulated tree
    ///
    /// Sums over all branches: for each internal node, add branch length
    /// from node to each child.
    #[must_use]
    pub fn total_branch_length(&self) -> f64 {
        self.nodes
            .iter()
            .filter(|n| !n.is_leaf)
            .map(|node| {
                let children_contribution: f64 = node
                    .children
                    .iter()
                    .map(|&child_id| {
                        let child = &self.nodes[child_id];
                        node.time - child.time
                    })
                    .sum();
                children_contribution
            })
            .sum()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the MRCA node
    ///
    /// # Returns
    /// * `Option<&CoalescentNode>` - MRCA node if it exists
    #[must_use]
    pub fn get_mrca_node(&self) -> Option<&CoalescentNode> {
        self.mrca_node.and_then(|id| self.nodes.get(id))
    }

    /// Get node by ID
    #[must_use]
    pub fn get_node(&self, id: NodeId) -> Option<&CoalescentNode> {
        self.nodes.get(id)
    }

    /// Get all leaf nodes (sampled lineages)
    #[must_use]
    pub fn get_leaves(&self) -> Vec<&CoalescentNode> {
        self.nodes.iter().filter(|n| n.is_leaf).collect()
    }

    /// Get all internal nodes (coalescence events)
    #[must_use]
    pub fn get_internal_nodes(&self) -> Vec<&CoalescentNode> {
        self.nodes.iter().filter(|n| !n.is_leaf).collect()
    }

    /// Get coalescence times for all internal nodes
    ///
    /// Returns a vector of (`node_id`, time) sorted by time (oldest first)
    #[must_use]
    pub fn coalescence_times(&self) -> Vec<(NodeId, f64)> {
        let mut times: Vec<(NodeId, f64)> = self
            .nodes
            .iter()
            .filter(|n| !n.is_leaf)
            .map(|n| (n.id, n.time))
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .collect();
        times.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        times
    }

    /// Find the most recent common ancestor of a subset of lineages
    ///
    /// # Arguments
    /// * `target_ids` - Agent IDs to find MRCA for
    ///
    /// # Returns
    /// * `Option<(NodeId, f64)>` - MRCA node ID and time
    #[must_use]
    pub fn find_mrca_of_subset(&self, target_ids: &[AgentId]) -> Option<(NodeId, f64)> {
        let target_set: HashSet<AgentId> = target_ids.iter().copied().collect();

        // Start from MRCA and traverse down until we find a node
        // that doesn't contain all targets
        let mrca_id = self.mrca_node?;

        fn find_deepest_containing(
            nodes: &[CoalescentNode],
            node_id: NodeId,
            target_set: &HashSet<AgentId>,
        ) -> Option<(NodeId, f64)> {
            let node = &nodes[node_id];

            // Check if this node contains all targets
            let node_lineage: HashSet<AgentId> = node.lineage.iter().copied().collect();
            if !target_set.is_subset(&node_lineage) {
                return None;
            }

            // Try children first
            for &child_id in &node.children {
                if let Some(result) = find_deepest_containing(nodes, child_id, target_set) {
                    return Some(result);
                }
            }

            // No child contains all targets, this is the MRCA
            Some((node_id, node.time))
        }

        find_deepest_containing(&self.nodes, mrca_id, &target_set)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute pairwise coalescence times
    ///
    /// For all pairs of sampled lineages, find their coalescence time.
    /// For neutral coalescent, these times are exponentially distributed.
    ///
    /// # Returns
    /// * `HashMap<(AgentId, AgentId), f64>` - Coalescence time for each pair
    #[must_use]
    pub fn pairwise_coalescence_times(&self) -> HashMap<(AgentId, AgentId), f64> {
        let leaves: Vec<&CoalescentNode> = self.get_leaves();
        let mut times = HashMap::new();

        for (i, leaf_i) in leaves.iter().enumerate() {
            for leaf_j in leaves.iter().skip(i + 1) {
                let agent_i = leaf_i.lineage[0];
                let agent_j = leaf_j.lineage[0];

                // Find MRCA of this pair
                if let Some((_, time)) = self.find_mrca_of_subset(&[agent_i, agent_j]) {
                    times.insert((agent_i, agent_j), time);
                    times.insert((agent_j, agent_i), time); // Symmetric
                }
            }
        }

        times
    }

    /// Number of lineages at time t (looking backward)
    ///
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Returns how many distinct lineages exist at a given time in the past.
    #[must_use]
    pub fn n_lineages_at_time(&self, t: f64) -> usize {
        if t >= self.total_tree_height {
            return 1; // At or above MRCA
        }
        if t <= 0.0 {
            return self.n_samples; // Present
        }

        // Count lineages that exist at time t (born at node.time <= t and not yet coalesced into parent)
        let mut count = 0;

        for node in &self.nodes {
            if node.time <= t {
                let active = match node.parent {
                    Some(parent_id) => {
                        let parent = &self.nodes[parent_id];
                        parent.time > t
                    }
                    None => true, // MRCA node
                };
                if active {
                    count += 1;
                }
            }
        }

        count.clamp(1, self.n_samples)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute Tajima's D statistic from the coalescent tree
    ///
    /// Uses the tree structure to compute pairwise differences,
    /// which is more efficient than allele counting.
    ///
    /// # Returns
    /// * `Option<f64>` - Tajima's D value
    #[must_use]
    pub fn tajimas_d(&self) -> Option<f64> {
        if self.n_samples < 2 {
            return None;
        }

        // Compute average pairwise coalescence time
        let pairwise_times = self.pairwise_coalescence_times();
        if pairwise_times.is_empty() {
            return None;
        }

        // Average pairwise coalescence time (in units of 2N generations)
        let avg_coalescence: f64 =
            pairwise_times.values().sum::<f64>() / pairwise_times.len() as f64;

        // Number of segregating sites is proportional to total branch length
        // For infinite sites model: E[S] = θ * total_branch_length / 2
        let total_bl = self.total_branch_length();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Simplified Tajima's D calculation
        // D = (π - S/a1) / sqrt(Var)
        // where π is average pairwise difference, S is segregating sites

        // For this simplified version, we use coalescence times as proxy
        let n = self.n_samples as f64;
        let a1: f64 = (1..self.n_samples).map(|i| 1.0 / i as f64).sum();

        if a1 <= 0.0 {
            return None;
        }

        // π (pi) estimator from coalescent: average pairwise coalescence time scaled to pairwise branch length
        // Under neutral model with theta = 1: E[pi] = E[2 * T_ij] = 2.0
        let pi = 2.0 * avg_coalescence;

        // S/a1 estimator (segregating sites proxy from total branch length)
        // Under neutral model: E[total_bl] = 2 * a1, so E[s_estimated / a1] = 2.0
        let s_estimated = total_bl;
        let s_over_a1 = s_estimated / a1;

        // Variance terms (Tajima 1989)
        let a2: f64 = (1..self.n_samples).map(|i| 1.0 / (i * i) as f64).sum();
        let b1 = (n + 1.0) / (3.0 * (n - 1.0));
        let b2 = 2.0 * (n * n + n + 3.0) / (9.0 * n * (n - 1.0));
        let c1 = b1 - 1.0 / a1;
        let c2 = b2 - (n + 2.0) / (a1 * n) + a2 / (a1 * a1);
        let e1 = c1 / a1;
        let e2 = c2 / (a1 * a1 + a2);

        let s_quad = (s_estimated - 1.0).max(0.0);
        let var_term = e1 * s_estimated + e2 * s_estimated * s_quad;
        if var_term <= 0.0 || !var_term.is_finite() {
            return None;
        }

        let denominator = var_term.sqrt();
        if denominator <= 0.0 || !denominator.is_finite() {
            return None;
        }

        Some((pi - s_over_a1) / denominator)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute the site frequency spectrum (SFS)
    ///
    /// The SFS counts how many derived alleles are present in
    /// exactly i out of n samples (for i = 1, ..., n-1).
    /// Mutational opportunities on branch e are proportional to its length;
    /// each branch contributes its length to the bin corresponding to its descendant count.
    ///
    /// # Returns
    /// * `Vec<f64>` - SFS where index i-1 is total branch length for frequency i/n
    #[must_use]
    pub fn site_frequency_spectrum(&self) -> Vec<f64> {
        if self.n_samples <= 1 {
            return Vec::new();
        }

        let mut sfs = vec![0.0; self.n_samples - 1];

        // For each node with an incoming parental branch (leaves and non-root internal nodes),
        // compute its contribution to the SFS. Mutations on this branch are inherited by
        // all descendants of this node.
        for node in &self.nodes {
            if let Some(parent_id) = node.parent {
                let derived_count = node.n_descendants();
                if derived_count >= 1 && derived_count < self.n_samples {
                    let parent_time = self
                        .nodes
                        .get(parent_id)
                        .map_or(self.total_tree_height, |p| p.time);
                    let branch_length = (parent_time - node.time).max(0.0);
                    sfs[derived_count - 1] += branch_length;
                }
            }
        }

        sfs
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Find MRCA for a set of agents from a lineage tree
///
/// This function works on actual simulation lineage data rather than
/// simulated coalescent. It finds the most recent common ancestor
/// by traversing the pedigree.
///
/// # Arguments
/// * `agent_ids` - Agent IDs to find MRCA for
/// * `lineage_tree` - Lineage tree from simulation
///
/// # Returns
/// * `Option<(AgentId, u32)>` - MRCA agent ID and generation
///
/// # Example
/// ```
/// use sim_core::lineage::LineageTree;
/// use sim_core::genealogy::coalescent::find_mrca;
///
/// # fn example() {
/// # let lineage_tree = LineageTree::new();
/// # let agent_ids = vec![1u64, 2u64];
/// # let mrca = find_mrca(&agent_ids, &lineage_tree);
/// # }
/// ```
#[must_use]
pub fn find_mrca(agent_ids: &[AgentId], lineage_tree: &LineageTree) -> Option<(AgentId, u32)> {
    if agent_ids.is_empty() {
        return None;
    }

    if agent_ids.len() == 1 {
        return lineage_tree
            .get_node(agent_ids[0])
            .map(|node| (agent_ids[0], node.generation));
    }

    // Collect all ancestors for each agent
    let mut ancestor_sets: Vec<HashSet<AgentId>> = Vec::new();
    let mut ancestor_generations: HashMap<AgentId, u32> = HashMap::new();

    for &agent_id in agent_ids {
        let mut ancestors = HashSet::new();
        collect_ancestors_with_generations(
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            agent_id,
            lineage_tree,
            &mut ancestors,
            &mut ancestor_generations,
            0,
        );
        ancestor_sets.push(ancestors);
    }

    // Check if any input agent is itself an ancestor of all others
    // This handles cases like finding MRCA of [child, grandparent]
    for (idx, &agent_id) in agent_ids.iter().enumerate() {
        let mut is_ancestor_of_all = true;
        for (other_idx, _) in agent_ids.iter().enumerate() {
            if idx == other_idx {
                continue;
            }
            // Check if agent_id is in other agent's ancestry
            if !ancestor_sets[other_idx].contains(&agent_id) && agent_id != agent_ids[other_idx] {
                is_ancestor_of_all = false;
                break;
            }
        }
        if is_ancestor_of_all {
            // This agent is the MRCA
            return lineage_tree
                .get_node(agent_id)
                .map(|node| (agent_id, node.generation));
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    // Find common ancestors across all sets
    let common_ancestors: HashSet<AgentId> = ancestor_sets
        .iter()
        .skip(1)
        .fold(ancestor_sets[0].clone(), |acc, set| {
            acc.intersection(set).copied().collect()
        });

    // Find most recent (highest generation number) common ancestor
    common_ancestors
        .iter()
        .filter_map(|&ancestor_id| {
            lineage_tree
                .get_node(ancestor_id)
                .map(|node| (ancestor_id, node.generation))
        })
        .max_by_key(|(_, gen)| *gen)
}

/// Collect all ancestors with their generation information
fn collect_ancestors_with_generations(
    agent_id: AgentId,
    tree: &LineageTree,
    ancestors: &mut HashSet<AgentId>,
    ancestor_generations: &mut HashMap<AgentId, u32>,
    _depth: u32,
) {
    if let Some(node) = tree.get_node(agent_id) {
        if let Some(p1) = node.parent1_id {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            ancestors.insert(p1);
            ancestor_generations.insert(p1, node.generation.saturating_sub(1));
            collect_ancestors_with_generations(
                p1,
                tree,
                ancestors,
                ancestor_generations,
                _depth + 1,
            );
        }
        if let Some(p2) = node.parent2_id {
            ancestors.insert(p2);
            ancestor_generations.insert(p2, node.generation.saturating_sub(1));
            collect_ancestors_with_generations(
                p2,
                tree,
                ancestors,
                ancestor_generations,
                _depth + 1,
            );
        }
    }
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

/// Find all pairwise MRCAs and their times
///
/// Returns a map of (`agent_i`, `agent_j`) -> (`mrca_id`, `generations_back`)
#[must_use]
pub fn find_all_pairwise_mrcas(
    agent_ids: &[AgentId],
    lineage_tree: &LineageTree,
) -> HashMap<(AgentId, AgentId), (AgentId, u32)> {
    let mut result = HashMap::new();

    for (i, &agent_i) in agent_ids.iter().enumerate() {
        for &agent_j in agent_ids.iter().skip(i + 1) {
            if let Some((mrca_id, generation)) = find_mrca(&[agent_i, agent_j], lineage_tree) {
                result.insert((agent_i, agent_j), (mrca_id, generation));
                result.insert((agent_j, agent_i), (mrca_id, generation)); // Symmetric
            }
        }
    }

    result
}

/// Statistics about a coalescent tree
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoalescentStats {
    /// Number of sampled lineages
    pub n_samples: usize,
    /// Time to most recent common ancestor
    pub tmrca: f64,
    /// Total branch length
    pub total_branch_length: f64,
    /// Average pairwise coalescence time
    pub avg_pairwise_time: f64,
    /// Variance of pairwise coalescence times
    pub var_pairwise_time: f64,
    /// Tajima's D statistic
    pub tajimas_d: Option<f64>,
    /// Number of coalescence events
    pub n_coalescences: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl CoalescentStats {
    /// Compute statistics from a coalescent tree
    #[must_use]
    pub fn from_coalescent(coal: &KingmanCoalescent) -> Self {
        let pairwise_times = coal.pairwise_coalescence_times();
        let n_pairs = pairwise_times.len().max(1) as f64;

        let avg_pairwise_time: f64 = pairwise_times.values().sum::<f64>() / n_pairs;

        let var_pairwise_time: f64 = if pairwise_times.len() > 1 {
            let mean = avg_pairwise_time;
            pairwise_times
                .values()
                .map(|&t| (t - mean).powi(2))
                .sum::<f64>()
                / (n_pairs - 1.0)
        } else {
            0.0
        };

        Self {
            n_samples: coal.n_samples,
            tmrca: coal.mrca_time,
            total_branch_length: coal.total_branch_length(),
            avg_pairwise_time,
            var_pairwise_time,
            tajimas_d: coal.tajimas_d(),
            n_coalescences: coal.get_internal_nodes().len(),
        }
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Build coalescent tree from simulation lineage data
///
/// This function extracts the genealogy for a set of sampled agents
/// and constructs a coalescent-style tree by finding all MRCAs.
///
/// # Arguments
/// * `sample_ids` - Agent IDs to include
/// * `lineage_tree` - Lineage tree from simulation
///
/// # Returns
/// * `Option<KingmanCoalescent>` - Coalescent tree if valid genealogy exists
#[must_use]
#[allow(clippy::collection_is_never_read)]
pub fn build_coalescent_from_lineage(
    sample_ids: &[AgentId],
    lineage_tree: &LineageTree,
) -> Option<KingmanCoalescent> {
    if sample_ids.len() < 2 {
        return None;
    }

    // Build tree by finding pairwise MRCAs and merging
    let pairwise_mrcas = find_all_pairwise_mrcas(sample_ids, lineage_tree);

    // This is a simplified construction - we create nodes based on
    // the pairwise MRCA structure
    let mut nodes: Vec<CoalescentNode> = Vec::new();
    let mut node_id = 0;

    // Create leaf nodes
    let mut agent_to_node: HashMap<AgentId, NodeId> = HashMap::new();
    for &agent_id in sample_ids {
        let leaf = CoalescentNode::new_leaf(node_id, agent_id);
        agent_to_node.insert(agent_id, node_id);
        nodes.push(leaf);
        node_id += 1;
    }

    // Group MRCAs by depth and create internal nodes
    // Sort by generation (time) - higher generation = more recent
    let mut mrca_list: Vec<(AgentId, u32)> = pairwise_mrcas
        .values()
        .copied()
        .collect::<std::collections::HashSet<_>>() // Deduplicate
        .into_iter()
        .collect();
    mrca_list.sort_by_key(|(_, gen)| *gen);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Map from generation to node ID for internal nodes
    let mut gen_to_node: HashMap<u32, NodeId> = HashMap::new();

    for (mrca_id, generation) in mrca_list {
        // Find children of this MRCA
        let mut child_ids: Vec<AgentId> = Vec::new();

        // Check which lineages coalesce at this point
        for (&(a1, a2), &(mrca, gen)) in &pairwise_mrcas {
            if mrca == mrca_id && gen == generation {
                if !child_ids.contains(&a1) {
                    child_ids.push(a1);
                }
                if !child_ids.contains(&a2) {
                    child_ids.push(a2);
                }
            }
        }

        // Convert child agent IDs to node IDs
        let child_node_ids: Vec<NodeId> = child_ids
            .iter()
            .filter_map(|aid| agent_to_node.get(aid).copied())
            .collect();

        if child_node_ids.len() >= 2 {
            // Create internal node
            let child_lineages: Vec<Vec<AgentId>> = child_ids.iter().map(|&id| vec![id]).collect();

            let internal_node = CoalescentNode::new_internal(
                node_id,
                &child_node_ids,
                generation as f64, // Use generation as time proxy
                child_lineages,
            );

            // Update parent pointers
            for &child_node_id in &child_node_ids {
                if let Some(node) = nodes.get_mut(child_node_id) {
                    node.parent = Some(node_id);
                }
            }

            gen_to_node.insert(generation, node_id);
            nodes.push(internal_node);
            node_id += 1;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Find MRCA (node with no parent)
    let mrca_node = nodes.iter().find(|n| n.parent.is_none()).map(|n| n.id);
    let mrca_time = mrca_node.map_or(0.0, |id| nodes[id].time);

    Some(KingmanCoalescent {
        nodes,
        n_samples: sample_ids.len(),
        total_tree_height: mrca_time,
        mrca_time,
        mrca_node,
    })
}

/// Validate coalescent tree structure
///
/// Checks that:
/// - Tree is connected
/// - All leaves have no children
/// - All internal nodes have exactly 2 children (binary tree)
/// - Exactly one node has no parent (MRCA)
/// - Times increase going up the tree
/// # Errors
///
/// Returns an error if the operation fails.
pub fn validate_coalescent_tree(coal: &KingmanCoalescent) -> Result<(), String> {
    // Check basic properties
    if coal.nodes.is_empty() {
        return Err("Empty tree".to_string());
    }

    // Check all leaves
    for node in &coal.nodes {
        if node.is_leaf {
            if !node.children.is_empty() {
                return Err(format!("Leaf {} has children", node.id));
            }
        } else {
            // Internal node should have exactly 2 children
            if node.children.len() != 2 {
                return Err(format!(
                    "Internal node {} has {} children (expected 2)",
                    node.id,
                    node.children.len()
                ));
            }
        }
    }

    // Check MRCA
    let root_nodes: Vec<NodeId> = coal
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .nodes
        .iter()
        .filter(|n| n.parent.is_none())
        .map(|n| n.id)
        .collect();

    if root_nodes.len() != 1 {
        return Err(format!(
            "Expected exactly 1 root node, found {}",
            root_nodes.len()
        ));
    }

    // Check times
    for node in &coal.nodes {
        if let Some(parent_id) = node.parent {
            let parent = &coal.nodes[parent_id];
            if parent.time <= node.time {
                return Err(format!(
                    "Parent time ({}) <= child time ({}) for node {}",
                    parent.time, node.time, node.id
                ));
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lineage::{LineageNode, TraitSnapshot};
    use rand::SeedableRng;

    fn make_trait_snapshot(tick: u64, coop: f32, risk: f32) -> TraitSnapshot {
        TraitSnapshot {
            tick,
            docas_alpha: [0.95; 5],
            docas_beta: [0.1; 5],
            planning_depth: 5,
            planning_horizon: 3,
            cooperation_tendency: coop,
            risk_tolerance: risk,
        }
    }

    #[test]
    fn test_coalescent_node_creation() {
        let leaf = CoalescentNode::new_leaf(0, 42);
        assert_eq!(leaf.id, 0);
        assert_eq!(leaf.lineage, vec![42]);
        assert!(leaf.is_leaf);
        assert!(leaf.children.is_empty());

        let internal = CoalescentNode::new_internal(1, &[0, 2], 1.5, vec![vec![42], vec![43]]);
        assert_eq!(internal.id, 1);
        assert_eq!(internal.children, vec![0, 2]);
        assert!(!internal.is_leaf);
        assert_eq!(internal.time, 1.5);
    }

    #[test]
    fn test_kingman_coalescent_simulation() {
        let mut rng = StdRng::seed_from_u64(42);
        let coal = KingmanCoalescent::simulate(10, &mut rng);

        assert_eq!(coal.n_samples, 10);
        assert!(coal.mrca_time > 0.0);
        assert!(coal.mrca_node.is_some());

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Should have 2n - 1 nodes total (n leaves + n-1 internal)
        assert_eq!(coal.nodes.len(), 19);

        // Validation should pass
        assert!(validate_coalescent_tree(&coal).is_ok());
    }

    #[test]
    fn test_expected_tmrca() {
        // For n=2: E[T] = 2(1 - 1/2) = 1.0
        assert!((KingmanCoalescent::expected_tmrca(2) - 1.0).abs() < 0.001);

        // For n=3: E[T] = 2(1 - 1/3) = 4/3 ≈ 1.333
        assert!((KingmanCoalescent::expected_tmrca(3) - 4.0 / 3.0).abs() < 0.001);

        // For large n: E[T] → 2.0
        assert!((KingmanCoalescent::expected_tmrca(1000) - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_expected_total_branch_length() {
        // For n=2: E[L] = 2 * 1 = 2
        assert!((KingmanCoalescent::expected_total_branch_length(2) - 2.0).abs() < 0.001);

        // For n=3: E[L] = 2 * (1 + 1/2) = 3
        assert!((KingmanCoalescent::expected_total_branch_length(3) - 3.0).abs() < 0.001);

        // For n=4: E[L] = 2 * (1 + 1/2 + 1/3) = 2 * 11/6 = 11/3 ≈ 3.667
        let expected = 2.0 * (1.0 + 0.5 + 1.0 / 3.0);
        assert!((KingmanCoalescent::expected_total_branch_length(4) - expected).abs() < 0.001);
    }

    #[test]
    fn test_total_branch_length_matches_expected() {
        use rand::SeedableRng;

        // Run multiple simulations and check average matches expected
        let mut rng = StdRng::seed_from_u64(123);
        let n = 20;
        let n_sims = 100;

        let mut total_bl = 0.0;
        for _ in 0..n_sims {
            let coal = KingmanCoalescent::simulate(n, &mut rng);
            total_bl += coal.total_branch_length();
        }

        let avg_bl = total_bl / n_sims as f64;
        let expected = KingmanCoalescent::expected_total_branch_length(n);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Should be within 10% of expected (with 100 simulations)
        assert!(
            (avg_bl - expected).abs() / expected < 0.15,
            "Average branch length {avg_bl} deviates too much from expected {expected}"
        );
    }

    #[test]
    fn test_tmrca_matches_expected() {
        use rand::SeedableRng;

        let mut rng = StdRng::seed_from_u64(456);
        let n = 30;
        let n_sims = 100;

        let mut total_tmrca = 0.0;
        for _ in 0..n_sims {
            let coal = KingmanCoalescent::simulate(n, &mut rng);
            total_tmrca += coal.mrca_time;
        }

        let avg_tmrca = total_tmrca / n_sims as f64;
        let expected = KingmanCoalescent::expected_tmrca(n);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Should be within 15% of expected
        assert!(
            (avg_tmrca - expected).abs() / expected < 0.15,
            "Average T_MRCA {avg_tmrca} deviates too much from expected {expected}"
        );
    }

    #[test]
    fn test_pairwise_coalescence_times() {
        let mut rng = StdRng::seed_from_u64(789);
        let coal = KingmanCoalescent::simulate(5, &mut rng);

        let times = coal.pairwise_coalescence_times();

        // For n samples, there should be n*(n-1)/2 unique pairs
        // But we store symmetric pairs, so n*(n-1)
        assert_eq!(times.len(), 5 * 4); // 20 entries for 5 samples

        // Check symmetry
        let leaves = coal.get_leaves();
        let agent_0 = leaves[0].lineage[0];
        let agent_1 = leaves[1].lineage[0];

        let time_01 = times.get(&(agent_0, agent_1));
        let time_10 = times.get(&(agent_1, agent_0));
        assert_eq!(time_01, time_10);
    }

    #[test]
    fn test_find_mrca_of_subset() {
        let mut rng = StdRng::seed_from_u64(321);
        let agent_ids: Vec<AgentId> = (0..10).map(|i| i as u64).collect();
        let coal = KingmanCoalescent::simulate_with_ids(&agent_ids, &mut rng);

        // Find MRCA of first 3
        let subset = &agent_ids[0..3];
        let mrca = coal.find_mrca_of_subset(subset);
        assert!(mrca.is_some());

        let (mrca_id, mrca_time) = mrca.unwrap();
        let mrca_node = &coal.nodes[mrca_id];

        // MRCA should contain all 3 agents
        let mrca_lineage: HashSet<_> = mrca_node.lineage.iter().copied().collect();
        let subset_set: HashSet<_> = subset.iter().copied().collect();
        assert!(subset_set.is_subset(&mrca_lineage));

        // MRCA time should be less than or equal to total height
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(mrca_time <= coal.total_tree_height);
    }

    #[test]
    fn test_n_lineages_at_time() {
        let mut rng = StdRng::seed_from_u64(555);
        let coal = KingmanCoalescent::simulate(10, &mut rng);

        // At time 0, should have all lineages
        assert_eq!(coal.n_lineages_at_time(0.0), 10);

        // At MRCA time, should have 1 lineage
        assert_eq!(coal.n_lineages_at_time(coal.total_tree_height), 1);

        // Beyond MRCA, should still be 1
        assert_eq!(coal.n_lineages_at_time(coal.total_tree_height + 1.0), 1);
    }

    #[test]
    fn test_simulate_with_ids() {
        let mut rng = StdRng::seed_from_u64(999);
        let agent_ids: Vec<AgentId> = vec![100, 200, 300, 400, 500];

        let coal = KingmanCoalescent::simulate_with_ids(&agent_ids, &mut rng);

        assert_eq!(coal.n_samples, 5);

        // Check that leaves have correct agent IDs
        let leaves = coal.get_leaves();
        let leaf_ids: Vec<AgentId> = leaves.iter().map(|l| l.lineage[0]).collect();

        for id in &agent_ids {
            assert!(leaf_ids.contains(id));
        }
    }

    #[test]
    fn test_site_frequency_spectrum() {
        let mut rng = StdRng::seed_from_u64(777);
        let coal = KingmanCoalescent::simulate(10, &mut rng);

        let sfs = coal.site_frequency_spectrum();

        // SFS should have n-1 entries
        assert_eq!(sfs.len(), 9);

        // All entries should be non-negative
        for freq in &sfs {
            assert!(*freq >= 0.0);
        }

        // Singletons (sfs[0]) must be non-zero (mutations on terminal leaf branches)
        assert!(sfs[0] > 0.0, "Singleton branch length must be positive");

        // Conservation law: sum of SFS entries must equal total branch length
        let sfs_sum: f64 = sfs.iter().sum();
        let total_bl = coal.total_branch_length();
        assert!(
            (sfs_sum - total_bl).abs() < 1e-10,
            "SFS sum {sfs_sum} must equal total branch length {total_bl}"
        );
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    #[test]
    fn test_coalescent_stats() {
        let mut rng = StdRng::seed_from_u64(888);
        let coal = KingmanCoalescent::simulate(10, &mut rng);

        let stats = CoalescentStats::from_coalescent(&coal);

        assert_eq!(stats.n_samples, 10);
        assert!(stats.tmrca > 0.0);
        assert!(stats.total_branch_length > 0.0);
        assert!(stats.n_coalescences > 0);
    }

    #[test]
    fn test_find_mrca_from_lineage_tree() {
        let mut tree = LineageTree::new();

        // Create three generations
        // Gen 0: Grandparent (0)
        tree.add_initial(LineageNode::new_initial(
            0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            0,
            make_trait_snapshot(0, 0.5, 0.5),
        ));

        // Gen 1: Parent (1)
        tree.add_offspring(LineageNode::new_offspring(
            1,
            0,
            None,
            10,
            make_trait_snapshot(10, 0.5, 0.5),
            0,
        ));

        // Gen 2: Two children (2, 3)
        tree.add_offspring(LineageNode::new_offspring(
            2,
            1,
            None,
            20,
            make_trait_snapshot(20, 0.5, 0.5),
            1,
        ));
        tree.add_offspring(LineageNode::new_offspring(
            3,
            1,
            None,
            25,
            make_trait_snapshot(25, 0.5, 0.5),
            1,
        ));

        // Find MRCA of children 2 and 3
        let mrca = find_mrca(&[2, 3], &tree);
        assert!(mrca.is_some());
        assert_eq!(mrca.unwrap().0, 1); // Parent is MRCA

        // Find MRCA of child 2 and grandparent 0
        let mrca2 = find_mrca(&[2, 0], &tree);
        assert!(mrca2.is_some());
        assert_eq!(mrca2.unwrap().0, 0); // Grandparent is MRCA
    }

    #[test]
    fn test_find_all_pairwise_mrcas() {
        let mut tree = LineageTree::new();

        // Create simple tree
        tree.add_initial(LineageNode::new_initial(
            // TAG: surface=simulation owner=research-team rule=SIM-001
            0,
            0,
            make_trait_snapshot(0, 0.5, 0.5),
        ));
        tree.add_offspring(LineageNode::new_offspring(
            1,
            0,
            None,
            10,
            make_trait_snapshot(10, 0.5, 0.5),
            0,
        ));
        tree.add_offspring(LineageNode::new_offspring(
            2,
            0,
            None,
            15,
            make_trait_snapshot(15, 0.5, 0.5),
            0,
        ));

        let pairwise = find_all_pairwise_mrcas(&[1, 2], &tree);

        // Should have 2 entries (symmetric)
        assert_eq!(pairwise.len(), 2);

        // MRCA of 1 and 2 should be 0
        assert_eq!(pairwise[&(1, 2)].0, 0);
        assert_eq!(pairwise[&(2, 1)].0, 0);
    }

    #[test]
    fn test_edge_cases() {
        let mut rng = StdRng::seed_from_u64(42);

        // Empty simulation
        let coal = KingmanCoalescent::simulate(0, &mut rng);
        assert_eq!(coal.n_samples, 0);
        assert!(coal.mrca_node.is_none());

        // Single sample
        let coal = KingmanCoalescent::simulate(1, &mut rng);
        assert_eq!(coal.n_samples, 1);
        assert!(coal.mrca_node.is_some());
        assert_eq!(coal.mrca_time, 0.0);

        // Expected values for edge cases
        assert_eq!(KingmanCoalescent::expected_tmrca(0), 0.0);
        assert_eq!(KingmanCoalescent::expected_tmrca(1), 0.0);
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_coalescence_rate_distribution() {
        use rand::SeedableRng;

        let mut rng = StdRng::seed_from_u64(42);
        let n = 2;
        let n_sims = 1000;

        // For n=2, coalescence time ~ Exponential(1)
        // Mean should be 1.0
        let mut total_time = 0.0;
        for _ in 0..n_sims {
            let coal = KingmanCoalescent::simulate(n, &mut rng);
            total_time += coal.mrca_time;
        }

        let mean_time = total_time / n_sims as f64;

        // With 1000 samples, mean should be close to 1.0
        assert!(
            (mean_time - 1.0).abs() < 0.1,
            "Mean coalescence time {mean_time} deviates from expected 1.0"
        );
    }

    #[test]
    fn test_kingman_coalescent_lineages_step_function_monotonicity() {
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(12345);
        let n = 10;
        let coal = KingmanCoalescent::simulate(n, &mut rng);

        // At present t = 0, exactly n lineages
        assert_eq!(coal.n_lineages_at_time(0.0), n);

        // At or above MRCA, exactly 1 lineage
        assert_eq!(coal.n_lineages_at_time(coal.total_tree_height), 1);
        assert_eq!(coal.n_lineages_at_time(coal.total_tree_height + 5.0), 1);

        // Verify non-increasing step function across fine time slices
        let num_slices = 50;
        let mut prev_count = n;
        for s in 0..=num_slices {
            let t = coal.total_tree_height * (s as f64 / num_slices as f64);
            let count = coal.n_lineages_at_time(t);
            assert!(
                count >= 1 && count <= n,
                "Count {count} out of bounds [1, {n}] at t={t}"
            );
            assert!(
                count <= prev_count,
                "Count {count} increased from {prev_count} at t={t}"
            );
            prev_count = count;
        }
    }
}
