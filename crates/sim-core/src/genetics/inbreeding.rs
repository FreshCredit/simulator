//! Inbreeding Depression Module
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION genetics, not product genetics
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! Implements inbreeding depression based on pedigree-based inbreeding coefficient (F)
//! and deleterious recessive allele exposure through increased homozygosity.
//!
//! ## Theoretical Background
//!
//! Inbreeding depression occurs when related individuals mate, increasing homozygosity
//! and exposing deleterious recessive alleles that are normally masked in heterozygotes.
//!
//! ### Key Parameters:
//! - **F coefficient**: Inbreeding coefficient (0 = random mating, 0.5 = self-fertilization)
//! - **First cousins**: F = 0.0625
//! - **Half-siblings**: F = 0.125
//! - **Full siblings**: F = 0.25
//! - **Parent-offspring**: F = 0.25
//!
//! ### Fitness Reduction:
//! Fitness typically follows: W = exp(-B × F) where B ~ 1-2
//! At F = 0.5, fitness reduction is typically 10-50%
//!
//! ## References:
//! - Charlesworth & Willis (2009): The genetics of inbreeding depression
//! - Keller & Waller (2002): Inbreeding effects in wild populations
//! - Wright (1922): Coefficients of Inbreeding and Relationship

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

use crate::genome::Genome;
use crate::lineage::LineageTree;
use crate::types::AgentId;

/// Threshold for deleterious alleles (below this value = deleterious)
const DELETERIOUS_THRESHOLD: f32 = 0.3;

/// Homozygosity threshold (alleles within this difference considered homozygous)
const HOMOZYGOUS_THRESHOLD: f32 = 0.05;

/// Default B parameter for fitness calculation (W = exp(-B * F))
const DEFAULT_B_PARAMETER: f32 = 1.5;

/// Minimum fitness multiplier to prevent complete lethality
const MIN_FITNESS: f32 = 0.1;

/// Inbreeding avoidance threshold (F >= 0.0625 = first cousins or closer)
pub const INBREEDING_AVOIDANCE_THRESHOLD: f32 = 0.0625;

/// Minimum expected fitness for reproduction permission
pub const MIN_REPRODUCTION_FITNESS: f32 = 0.7;

/// Per-locus deleterious load contribution
const DELETERIOUS_LOAD_PER_LOCUS: f32 = 0.1;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Tracks inbreeding status and fitness effects for an agent
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InbreedingTracker {
    /// Pedigree-based inbreeding coefficient F (0 = outbred, 0.5 = self-fertilization)
    pub f_coefficient: f32,

    /// Fitness multiplier computed from deleterious homozygous loci
    /// W = exp(-load) where load is sum of deleterious homozygous effects
    pub fitness_multiplier: f32,

    /// Count of deleterious homozygous loci
    pub deleterious_homozygous: u32,

    /// B parameter for fitness calculation (species-specific)
    pub b_parameter: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for InbreedingTracker {
    fn default() -> Self {
        Self {
            f_coefficient: 0.0,
            fitness_multiplier: 1.0,
            deleterious_homozygous: 0,
            b_parameter: DEFAULT_B_PARAMETER,
        }
    }
}

impl InbreedingTracker {
    /// Create a new inbreeding tracker with default (outbred) values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new tracker with specified F coefficient
    #[must_use]
    pub fn with_f_coefficient(f: f32) -> Self {
        let f = f.clamp(0.0, 1.0);
        Self {
            f_coefficient: f,
            fitness_multiplier: compute_fitness_from_f(f, DEFAULT_B_PARAMETER),
            deleterious_homozygous: 0,
            b_parameter: DEFAULT_B_PARAMETER,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate inbreeding coefficient F from pedigree using path coefficient method
    ///
    /// The path coefficient method finds all paths from parent1 to parent2 through
    /// common ancestors and computes:
    /// F = Σ (1/2)^(n+1) for all paths
    ///
    /// where n = number of individuals in the path (excluding the common ancestor)
    ///
    /// # Arguments
    /// * `parent1` - First parent's agent ID
    /// * `parent2` - Second parent's agent ID
    /// * `lineage` - Lineage tree containing ancestry information
    ///
    /// # Returns
    /// Inbreeding coefficient F in range [0.0, 1.0]
    #[must_use]
    pub fn from_pedigree(parent1: AgentId, parent2: AgentId, lineage: &LineageTree) -> f32 {
        // Same individual = self-fertilization (F = 0.5)
        if parent1 == parent2 {
            return 0.5;
        }

        // Find common ancestors
        let common_ancestors = find_common_ancestors(parent1, parent2, lineage);

        if common_ancestors.is_empty() {
            return 0.0;
        }

        // Sum path coefficients for all common ancestors
        let mut total_f = 0.0;
        for (ancestor, _) in common_ancestors {
            if let Some(path_coeff) = compute_path_coefficient(parent1, parent2, ancestor, lineage)
            {
                total_f += path_coeff;
            }
        }

        total_f.clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Compute fitness based on deleterious homozygous loci in genome
    ///
    /// Counts loci where:
    /// 1. Both alleles are below deleterious threshold (deleterious)
    /// 2. Alleles are sufficiently similar (homozygous)
    ///
    /// Fitness formula: W = exp(-load × B) where:
    /// - load = count of deleterious homozygous loci × per-locus effect
    /// - B = species-specific parameter (typically 1-2)
    ///
    /// # Arguments
    /// * `genome` - The agent's genome to analyze
    ///
    /// # Returns
    /// Fitness multiplier in range [`MIN_FITNESS`, 1.0]
    pub fn compute_fitness(&mut self, genome: &Genome) -> f32 {
        let mut load = 0.0;
        let mut deleterious_count = 0;

        for locus in &genome.alleles {
            // Check if both alleles are deleterious (below threshold)
            let is_del = locus[0] < DELETERIOUS_THRESHOLD && locus[1] < DELETERIOUS_THRESHOLD;

            // Check if alleles are homozygous (similar values)
            let is_homo = (locus[0] - locus[1]).abs() < HOMOZYGOUS_THRESHOLD;

            if is_del && is_homo {
                load += DELETERIOUS_LOAD_PER_LOCUS;
                deleterious_count += 1;
            }
        }

        // Also consider F coefficient in fitness calculation
        // Higher F = more homozygosity = more deleterious expression
        let f_contribution = (self.f_coefficient.max(0.0) * self.b_parameter.max(0.0)).max(0.0);

        // Combined fitness effect
        let total_load = (load + f_contribution).max(0.0);
        self.fitness_multiplier = (-total_load).exp().clamp(MIN_FITNESS, 1.0);
        self.deleterious_homozygous = deleterious_count;

        self.fitness_multiplier
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute fitness based solely on F coefficient (without genome analysis)
    ///
    /// Uses the standard formula: W = exp(-B × F)
    ///
    /// # Returns
    /// Fitness multiplier in range [`MIN_FITNESS`, 1.0]
    pub fn compute_fitness_from_f(&mut self) -> f32 {
        self.fitness_multiplier = compute_fitness_from_f(self.f_coefficient, self.b_parameter);
        self.fitness_multiplier
    }

    /// Check if this agent has inbreeding depression (fitness < 1.0)
    #[must_use]
    pub fn has_depression(&self) -> bool {
        self.fitness_multiplier < 1.0
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the severity of inbreeding depression
    /// Returns value in [0.0, 1.0] where 0 = no depression, 1 = severe depression
    #[must_use]
    pub fn depression_severity(&self) -> f32 {
        (1.0 - self.fitness_multiplier).clamp(0.0, 1.0)
    }

    /// Apply inbreeding effects to offspring energy
    ///
    /// Offspring energy is reduced proportionally to fitness loss
    ///
    /// # Arguments
    /// * `base_energy` - Initial energy value
    ///
    /// # Returns
    /// Adjusted energy after inbreeding penalty
    #[must_use]
    pub fn apply_energy_penalty(&self, base_energy: f32) -> f32 {
        base_energy * self.fitness_multiplier
    }

    /// Calculate mortality risk increase from inbreeding
    ///
    /// Higher F = higher baseline mortality risk
    ///
    /// # Returns
    /// Additional mortality risk to add to base risk
    #[must_use]
    pub fn mortality_risk_increase(&self) -> f32 {
        (1.0 - self.fitness_multiplier) * 0.2
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Update the F coefficient and recompute fitness
    pub fn set_f_coefficient(&mut self, f: f32) {
        self.f_coefficient = if f.is_finite() {
            f.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.compute_fitness_from_f();
    }
}

/// Compute fitness from F coefficient using W = exp(-B × F)
fn compute_fitness_from_f(f: f32, b: f32) -> f32 {
    let eff_f = if f.is_finite() { f.max(0.0) } else { 0.0 };
    let eff_b = if b.is_finite() {
        b.max(0.0)
    } else {
        DEFAULT_B_PARAMETER
    };
    (-eff_b * eff_f).exp().clamp(MIN_FITNESS, 1.0)
}

/// Find all common ancestors between two agents
///
/// In Wright's (1922) path-coefficient framework, each agent is included as its own
/// ancestor at distance 0 (essential for parent-offspring and descendant loops).
/// Redundant ancestors whose paths pass through more recent common ancestors are pruned.
///
/// Returns vector of (`ancestor_id`, `distance_to_ancestor`)
fn find_common_ancestors(
    agent1: AgentId,
    agent2: AgentId,
    lineage: &LineageTree,
) -> Vec<(AgentId, u32)> {
    let ancestors1 = collect_ancestors_with_distance(agent1, lineage);
    let ancestors2 = collect_ancestors_with_distance(agent2, lineage);

    // Find intersection (deterministic order via BTreeMap)
    let mut common = BTreeMap::new();
    for (ancestor, d1) in &ancestors1 {
        if let Some(&d2) = ancestors2.get(ancestor) {
            common.insert(*ancestor, (*d1).max(d2));
        }
    }

    // Filter out ancestors whose lineage to both parents passes through another common ancestor
    let common_ids: BTreeSet<AgentId> = common.keys().copied().collect();
    let filtered: Vec<(AgentId, u32)> = common
        .into_iter()
        .filter(|(ancestor, _)| {
            !common_ids.iter().any(|&other| {
                other != *ancestor && find_path_length(*ancestor, other, lineage).is_some()
            })
        })
        .collect();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    filtered
}

/// Helper to collect ancestors and minimum meiotic distances (deterministic BFS)
fn collect_ancestors_with_distance(
    agent_id: AgentId,
    lineage: &LineageTree,
) -> BTreeMap<AgentId, u32> {
    let mut ancestors = BTreeMap::new();
    let mut queue = VecDeque::new();
    ancestors.insert(agent_id, 0u32);
    queue.push_back((agent_id, 0u32));

    while let Some((current, dist)) = queue.pop_front() {
        if let Some(node) = lineage.get_node(current) {
            if let Some(p1) = node.parent1_id {
                if let std::collections::btree_map::Entry::Vacant(e) = ancestors.entry(p1) {
                    e.insert(dist + 1);
                    queue.push_back((p1, dist + 1));
                }
            }
            if let Some(p2) = node.parent2_id {
                if let std::collections::btree_map::Entry::Vacant(e) = ancestors.entry(p2) {
                    e.insert(dist + 1);
                    queue.push_back((p2, dist + 1));
                }
            }
        }
    }

    ancestors
}

/// Compute ancestor inbreeding coefficient with bounded recursion depth per Wright (1922)
fn compute_ancestor_f(ancestor: AgentId, lineage: &LineageTree, depth: usize) -> f32 {
    if depth == 0 {
        return 0.0;
    }
    if let Some(node) = lineage.get_node(ancestor) {
        if let (Some(p1), Some(p2)) = (node.parent1_id, node.parent2_id) {
            if p1 == p2 {
                return 0.5;
            }
            let common_ancestors = find_common_ancestors(p1, p2, lineage);
            if common_ancestors.is_empty() {
                return 0.0;
            }
            let mut total_f = 0.0f32;
            for (anc, _) in common_ancestors {
                if let (Some(steps1), Some(steps2)) = (
                    find_path_length(anc, p1, lineage),
                    find_path_length(anc, p2, lineage),
                ) {
                    let total_steps = steps1 + steps2;
                    let f_anc = compute_ancestor_f(anc, lineage, depth - 1);
                    total_f += (0.5f32).powi((total_steps + 1) as i32) * (1.0 + f_anc);
                }
            }
            return total_f.clamp(0.0, 1.0);
        }
    }
    0.0
}

/// Compute path coefficient contribution from a common ancestor
///
/// The path coefficient is (1/2)^n * (1 + F_A) where n is the number of meiotic events
/// (steps) in the path from parent1 to parent2 through the common ancestor, and F_A
/// is the inbreeding coefficient of the common ancestor (Sewall Wright, 1922).
///
/// Path structure: parent1 → ... → ancestor → ... → parent2
///
/// # Arguments
/// * `parent1` - First parent agent ID
/// * `parent2` - Second parent agent ID
/// * `common_ancestor` - Common ancestor agent ID
/// * `lineage` - Lineage tree
///
/// # Returns
/// Path coefficient contribution, or None if no valid path exists
fn compute_path_coefficient(
    parent1: AgentId,
    parent2: AgentId,
    common_ancestor: AgentId,
    lineage: &LineageTree,
) -> Option<f32> {
    // Find path lengths from common ancestor to each parent
    let steps1 = find_path_length(common_ancestor, parent1, lineage)?;
    let steps2 = find_path_length(common_ancestor, parent2, lineage)?;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Total meiotic events in the loop
    let total_steps = steps1 + steps2;

    // Path coefficient: (1/2)^(total_steps + 1) * (1 + F_A)
    // This accounts for the probability of identity by descent in offspring (Sewall Wright, 1922)
    let f_a = compute_ancestor_f(common_ancestor, lineage, 3);
    Some((0.5f32).powi((total_steps + 1) as i32) * (1.0 + f_a))
}

/// Find the number of steps (meiotic events) from ancestor to descendant
///
/// Returns None if no path exists
fn find_path_length(ancestor: AgentId, descendant: AgentId, lineage: &LineageTree) -> Option<u32> {
    if ancestor == descendant {
        return Some(0);
    }

    // BFS to find shortest path (deterministic BTreeSet for visited)
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((descendant, 0u32));

    while let Some((current, steps)) = queue.pop_front() {
        if current == ancestor {
            return Some(steps);
        }

        if visited.contains(&current) {
            continue;
        }
        visited.insert(current);

        if let Some(node) = lineage.get_node(current) {
            if let Some(p1) = node.parent1_id {
                queue.push_back((p1, steps + 1));
            }
            if let Some(p2) = node.parent2_id {
                queue.push_back((p2, steps + 1));
            }
        }
    }

    None
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Check if two agents can reproduce based on inbreeding coefficient
///
/// Implements inbreeding avoidance behavior - agents avoid mating when:
/// 1. F > `INBREEDING_AVOIDANCE_THRESHOLD` (first cousin level)
/// 2. Expected fitness would be below `MIN_REPRODUCTION_FITNESS`
///
/// # Arguments
/// * `agent1` - First potential parent
/// * `agent2` - Second potential parent
/// * `lineage` - Lineage tree for ancestry lookup
///
/// # Returns
/// true if reproduction is permitted
#[must_use]
pub fn can_reproduce_inbreeding(
    agent1_id: AgentId,
    agent2_id: AgentId,
    lineage: &LineageTree,
) -> bool {
    let f = InbreedingTracker::from_pedigree(agent1_id, agent2_id, lineage);

    // If F is below threshold, always allow
    if f <= INBREEDING_AVOIDANCE_THRESHOLD {
        return true;
    }

    // Calculate expected fitness of offspring
    let expected_fitness = (-f * DEFAULT_B_PARAMETER).exp();

    // Only allow if expected fitness is above minimum threshold
    expected_fitness >= MIN_REPRODUCTION_FITNESS
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Calculate inbreeding coefficient for potential offspring
///
/// Returns the F coefficient that offspring of these parents would have
///
/// # Arguments
/// * `parent1` - First parent agent ID
/// * `parent2` - Second parent agent ID
/// * `lineage` - Lineage tree
///
/// # Returns
/// Inbreeding coefficient F for potential offspring
#[must_use]
pub fn calculate_offspring_f(parent1: AgentId, parent2: AgentId, lineage: &LineageTree) -> f32 {
    InbreedingTracker::from_pedigree(parent1, parent2, lineage)
}

/// Inbreeding statistics for population monitoring
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct InbreedingStats {
    /// Average F coefficient across population
    pub mean_f: f32,

    /// Average fitness multiplier
    pub mean_fitness: f32,

    /// Fraction of population with F > 0 (some inbreeding)
    pub inbred_fraction: f32,

    /// Fraction of population with severe inbreeding (F > 0.25)
    pub severe_inbreeding_fraction: f32,

    /// Total deleterious homozygous loci across population
    pub total_deleterious_loci: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute inbreeding statistics for a population
///
/// # Arguments
/// * `trackers` - Slice of inbreeding trackers for the population
///
/// # Returns
/// Population inbreeding statistics
#[must_use]
pub fn compute_population_inbreeding_stats(trackers: &[InbreedingTracker]) -> InbreedingStats {
    if trackers.is_empty() {
        return InbreedingStats::default();
    }

    let n = trackers.len() as f32;
    let sum_f: f32 = trackers.iter().map(|t| t.f_coefficient).sum();
    let sum_fitness: f32 = trackers.iter().map(|t| t.fitness_multiplier).sum();
    let inbred_count = trackers.iter().filter(|t| t.f_coefficient > 0.0).count() as f32;
    let severe_count = trackers.iter().filter(|t| t.f_coefficient > 0.25).count() as f32;
    let total_deleterious: u32 = trackers.iter().map(|t| t.deleterious_homozygous).sum();

    InbreedingStats {
        mean_f: sum_f / n,
        mean_fitness: sum_fitness / n,
        inbred_fraction: inbred_count / n,
        severe_inbreeding_fraction: severe_count / n,
        total_deleterious_loci: total_deleterious,
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Predict fitness of potential offspring without creating it
///
/// # Arguments
/// * `parent1_id` - First parent agent ID
/// * `parent2_id` - Second parent agent ID
/// * `lineage` - Lineage tree
/// * `parent1_genome` - First parent's genome (optional, for better prediction)
/// * `parent2_genome` - Second parent's genome (optional, for better prediction)
///
/// # Returns
/// Predicted fitness multiplier for offspring
#[must_use]
pub fn predict_offspring_fitness(
    parent1_id: AgentId,
    parent2_id: AgentId,
    lineage: &LineageTree,
    parent1_genome: Option<&Genome>,
    parent2_genome: Option<&Genome>,
) -> f32 {
    let f = calculate_offspring_f(parent1_id, parent2_id, lineage);

    // If we have parental genomes, we can estimate deleterious load
    let estimated_load = if let (Some(g1), Some(g2)) = (parent1_genome, parent2_genome) {
        estimate_deleterious_load(g1, g2, f)
    } else {
        // Without genomes, use F-based estimate only
        f * DEFAULT_B_PARAMETER
    };

    (-estimated_load).exp().max(MIN_FITNESS)
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Estimate deleterious load in potential offspring
///
/// This estimates the probability of deleterious homozygosity based on
/// parental allele frequencies and expected F.
fn estimate_deleterious_load(parent1: &Genome, parent2: &Genome, f: f32) -> f32 {
    let mut expected_deleterious_loci = 0.0;

    for locus_idx in 0..parent1.alleles.len().min(parent2.alleles.len()) {
        let p1_alleles = &parent1.alleles[locus_idx];
        let p2_alleles = &parent2.alleles[locus_idx];

        // Count how many parental alleles are deleterious
        let p1_del_count = p1_alleles
            .iter()
            .filter(|&&a| a < DELETERIOUS_THRESHOLD)
            .count() as f32;
        let p2_del_count = p2_alleles
            .iter()
            .filter(|&&a| a < DELETERIOUS_THRESHOLD)
            .count() as f32;

        // Probability offspring gets deleterious allele from each parent
        let p_del_from_p1 = p1_del_count / 2.0;
        let p_del_from_p2 = p2_del_count / 2.0;

        // Probability offspring is homozygous deleterious
        // = P(del from p1) × P(del from p2), adjusted for F
        // Higher F increases homozygosity probability, clamped to valid probability range [0, 1]
        let p_homo_del = (p_del_from_p1 * p_del_from_p2 * (1.0 + f)).clamp(0.0, 1.0);

        if p_homo_del > 0.1 {
            expected_deleterious_loci += p_homo_del * DELETERIOUS_LOAD_PER_LOCUS;
        }
    }

    expected_deleterious_loci + f * DEFAULT_B_PARAMETER
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Check if an agent would be considered "inbred" (F > 0)
#[must_use]
pub fn is_inbred(f_coefficient: f32) -> bool {
    f_coefficient > 0.001 // Small epsilon to account for floating point
}

/// Check if inbreeding is severe (F >= 0.125 = half-sibling level or higher)
#[must_use]
pub fn is_severe_inbreeding(f_coefficient: f32) -> bool {
    f_coefficient >= 0.125
}

/// Get the kinship relationship description based on F value
///
/// Returns human-readable description of relationship (Sewall Wright, 1922)
#[must_use]
pub fn describe_relationship(f_coefficient: f32) -> &'static str {
    match f_coefficient {
        f if f < 0.001 => "unrelated",
        f if f < 0.03125 => "second cousins",
        f if f < 0.09375 => "first cousins",
        f if f < 0.1875 => "half-siblings / uncle-niece",
        f if f < 0.375 => "full siblings / parent-offspring",
        _ => "self-fertilization",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::Genome;
    use crate::lineage::{LineageNode, TraitSnapshot};

    fn create_test_lineage() -> LineageTree {
        let mut lineage = LineageTree::new();
        let tick = 0u64;

        // Create grandparents (generation 0)
        let gp1 = 1u64; // Grandfather on father's side
        let gp2 = 2u64; // Grandmother on father's side
        let gp3 = 3u64; // Grandfather on mother's side
        let gp4 = 4u64; // Grandmother on mother's side

        let genome = Genome::default();
        let traits = TraitSnapshot::from_genome(&genome, tick);

        lineage.add_initial(LineageNode::new_initial(gp1, tick, traits.clone()));
        lineage.add_initial(LineageNode::new_initial(gp2, tick, traits.clone()));
        lineage.add_initial(LineageNode::new_initial(gp3, tick, traits.clone()));
        lineage.add_initial(LineageNode::new_initial(gp4, tick, traits.clone()));

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Create parents (generation 1)
        // Father (child of gp1 and gp2)
        let father = 5u64;
        let father_node =
            LineageNode::new_offspring(father, gp1, Some(gp2), tick + 10, traits.clone(), 0);
        lineage.add_offspring(father_node);

        // Mother (child of gp3 and gp4)
        let mother = 6u64;
        let mother_node =
            LineageNode::new_offspring(mother, gp3, Some(gp4), tick + 10, traits.clone(), 0);
        lineage.add_offspring(mother_node);

        // Create child (generation 2)
        let child = 7u64;
        let child_node =
            LineageNode::new_offspring(child, father, Some(mother), tick + 20, traits, 1);
        lineage.add_offspring(child_node);

        lineage
    }

    fn create_lineage_with_cousins() -> LineageTree {
        let mut lineage = LineageTree::new();
        let tick = 0u64;
        let genome = Genome::default();
        let traits = TraitSnapshot::from_genome(&genome, tick);

        // Common grandparents
        let gpf = 1u64; // Grandfather
        let gpm = 2u64; // Grandmother

        lineage.add_initial(LineageNode::new_initial(gpf, tick, traits.clone()));
        lineage.add_initial(LineageNode::new_initial(gpm, tick, traits.clone()));

        // Two siblings (parents of cousins)
        let parent1 = 3u64;
        let parent2 = 4u64;

        let p1_node =
            LineageNode::new_offspring(parent1, gpf, Some(gpm), tick + 10, traits.clone(), 0);
        lineage.add_offspring(p1_node);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let p2_node =
            LineageNode::new_offspring(parent2, gpf, Some(gpm), tick + 10, traits.clone(), 0);
        lineage.add_offspring(p2_node);

        // Cousins (children of the two siblings)
        let cousin1 = 5u64;
        let cousin2 = 6u64;

        // Create spouses for the parents (unrelated)
        let spouse1 = 7u64;
        let spouse2 = 8u64;
        lineage.add_initial(LineageNode::new_initial(spouse1, tick, traits.clone()));
        lineage.add_initial(LineageNode::new_initial(spouse2, tick, traits.clone()));

        let c1_node = LineageNode::new_offspring(
            cousin1,
            parent1,
            Some(spouse1),
            tick + 20,
            traits.clone(),
            1,
        );
        lineage.add_offspring(c1_node);

        let c2_node =
            LineageNode::new_offspring(cousin2, parent2, Some(spouse2), tick + 20, traits, 1);
        lineage.add_offspring(c2_node);

        lineage
    }

    #[test]
    fn test_inbreeding_tracker_default() {
        let tracker = InbreedingTracker::default();
        assert_eq!(tracker.f_coefficient, 0.0);
        assert_eq!(tracker.fitness_multiplier, 1.0);
        assert_eq!(tracker.deleterious_homozygous, 0);
    }

    #[test]
    fn test_inbreeding_tracker_with_f() {
        let tracker = InbreedingTracker::with_f_coefficient(0.125);
        assert_eq!(tracker.f_coefficient, 0.125);
        assert!(tracker.fitness_multiplier < 1.0);
        assert!(tracker.fitness_multiplier > MIN_FITNESS);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_f_unrelated() {
        let lineage = create_test_lineage();

        // Two unrelated grandparents should have F = 0
        let f = InbreedingTracker::from_pedigree(1, 3, &lineage);
        assert!(
            f < 0.001,
            "Unrelated individuals should have F ≈ 0, got {f}"
        );
    }

    #[test]
    fn test_f_self() {
        let lineage = create_test_lineage();

        // Self-fertilization should have F = 0.5
        let f = InbreedingTracker::from_pedigree(1, 1, &lineage);
        assert!(
            (f - 0.5).abs() < 0.001,
            "Self-fertilization should have F = 0.5, got {f}"
        );
    }

    #[test]
    fn test_f_cousins() {
        let lineage = create_lineage_with_cousins();

        // First cousins should have F = 0.0625 (1/16 per Sewall Wright 1922)
        // Cousins 5 and 6 are first cousins sharing 2 common grandparents
        let f = InbreedingTracker::from_pedigree(5, 6, &lineage);
        assert!(
            (f - 0.0625).abs() < 0.005,
            "First cousins should have F = 0.0625, got {f}"
        );
    }

    #[test]
    fn test_compute_fitness() {
        let mut tracker = InbreedingTracker::new();
        let mut genome = Genome::default();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Create some deleterious homozygous loci
        // Both deleterious (< 0.3), difference < 0.05 (homozygous threshold)
        genome.alleles[0] = [0.1, 0.08]; // Both deleterious, similar (diff = 0.02)
        genome.alleles[1] = [0.2, 0.18]; // Both deleterious, similar (diff = 0.02)
        genome.alleles[2] = [0.8, 0.9]; // Not deleterious

        let fitness = tracker.compute_fitness(&genome);

        assert!(fitness < 1.0, "Should have reduced fitness");
        assert!(
            fitness >= MIN_FITNESS,
            "Fitness should not go below minimum"
        );
        assert_eq!(
            tracker.deleterious_homozygous, 2,
            "Should have 2 deleterious homozygous loci"
        );
    }

    #[test]
    fn test_fitness_decreases_with_f() {
        let f_values = [0.0, 0.125, 0.25, 0.5];
        let mut fitness_values = Vec::new();

        for &f in &f_values {
            let tracker = InbreedingTracker::with_f_coefficient(f);
            fitness_values.push(tracker.fitness_multiplier);
        }

        // Fitness should decrease as F increases
        for i in 1..fitness_values.len() {
            assert!(
                fitness_values[i] < fitness_values[i - 1],
                "Fitness should decrease with increasing F"
            );
        }
    }

    #[test]
    fn test_can_reproduce_inbreeding() {
        let lineage = create_lineage_with_cousins();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Unrelated individuals should be able to reproduce
        assert!(can_reproduce_inbreeding(7, 8, &lineage));

        // Cousins should still be able to reproduce (F ~ 0.0625 < threshold)
        let f = InbreedingTracker::from_pedigree(5, 6, &lineage);
        if f <= INBREEDING_AVOIDANCE_THRESHOLD {
            assert!(can_reproduce_inbreeding(5, 6, &lineage));
        }
    }

    #[test]
    fn test_reproduction_avoidance_high_f() {
        // Test with self-fertilization (F = 0.5)
        let mut lineage = LineageTree::new();
        let genome = Genome::default();
        let traits = TraitSnapshot::from_genome(&genome, 0);
        lineage.add_initial(LineageNode::new_initial(1, 0, traits));

        // Self-fertilization should be blocked (F = 0.5 > threshold)
        let allowed = can_reproduce_inbreeding(1, 1, &lineage);
        let f = InbreedingTracker::from_pedigree(1, 1, &lineage);

        if f > INBREEDING_AVOIDANCE_THRESHOLD {
            let expected_fitness = (-f * DEFAULT_B_PARAMETER).exp();
            if expected_fitness < MIN_REPRODUCTION_FITNESS {
                assert!(!allowed, "High F selfing should be blocked");
            }
        }
    }

    #[test]
    fn test_apply_energy_penalty() {
        let tracker = InbreedingTracker::with_f_coefficient(0.5);
        let base_energy = 100.0;

        let adjusted = tracker.apply_energy_penalty(base_energy);

        assert!(
            adjusted < base_energy,
            "Energy should be reduced by inbreeding"
        );
        assert!(
            adjusted >= base_energy * MIN_FITNESS,
            "Energy should not go below minimum"
        );
    }

    #[test]
    fn test_f_inbred_common_ancestor() {
        let mut lineage = LineageTree::new();
        let genome = Genome::default();
        let traits = TraitSnapshot::from_genome(&genome, 0);

        // Generation 0: Base outbred pair (gp1, gp2)
        lineage.add_initial(LineageNode::new_initial(1, 0, traits.clone()));
        lineage.add_initial(LineageNode::new_initial(2, 0, traits.clone()));

        // Full sibs: parent_a (3) and parent_b (4)
        lineage.add_offspring(LineageNode::new_offspring(
            3,
            1,
            Some(2),
            1,
            traits.clone(),
            0,
        ));
        lineage.add_offspring(LineageNode::new_offspring(
            4,
            1,
            Some(2),
            1,
            traits.clone(),
            0,
        ));

        // Offspring of full sibs: anc (5) has F_A = 0.25
        lineage.add_offspring(LineageNode::new_offspring(
            5,
            3,
            Some(4),
            2,
            traits.clone(),
            1,
        ));

        // anc (5) mates with two unrelated spouses to produce half-sibs 8 and 9
        lineage.add_initial(LineageNode::new_initial(6, 0, traits.clone()));
        lineage.add_initial(LineageNode::new_initial(7, 0, traits.clone()));
        lineage.add_offspring(LineageNode::new_offspring(
            8,
            5,
            Some(6),
            3,
            traits.clone(),
            2,
        ));
        lineage.add_offspring(LineageNode::new_offspring(
            9,
            5,
            Some(7),
            3,
            traits.clone(),
            2,
        ));

        // Mating between half-sibs 8 and 9 sharing inbred ancestor 5:
        // Wright formula: F = (1/2)^(1 + 1 + 1) * (1 + F_A) = 0.125 * (1 + 0.25) = 0.15625
        let f = InbreedingTracker::from_pedigree(8, 9, &lineage);
        assert!(
            (f - 0.15625).abs() < 0.01,
            "Expected F ~ 0.15625 with inbred ancestor, got {f}"
        );
    }

    #[test]
    fn test_f_parent_offspring_and_siblings() {
        let mut lineage = LineageTree::new();
        let tick = 0u64;
        let genome = Genome::default();
        let traits = TraitSnapshot::from_genome(&genome, tick);

        // Founders
        let p1 = 101u64;
        let p2 = 102u64;
        let p3 = 103u64;
        lineage.add_initial(LineageNode::new_initial(p1, tick, traits.clone()));
        lineage.add_initial(LineageNode::new_initial(p2, tick, traits.clone()));
        lineage.add_initial(LineageNode::new_initial(p3, tick, traits.clone()));

        // Full siblings child_a & child_b from (p1, p2)
        let child_a = 201u64;
        let child_b = 202u64;
        lineage.add_offspring(LineageNode::new_offspring(
            child_a,
            p1,
            Some(p2),
            tick + 10,
            traits.clone(),
            0,
        ));
        lineage.add_offspring(LineageNode::new_offspring(
            child_b,
            p1,
            Some(p2),
            tick + 10,
            traits.clone(),
            0,
        ));

        // Half sibling child_c from (p1, p3)
        let child_c = 203u64;
        lineage.add_offspring(LineageNode::new_offspring(
            child_c,
            p1,
            Some(p3),
            tick + 10,
            traits.clone(),
            0,
        ));

        // 1. Founder parent-offspring: p1 & child_a -> F = 0.25
        let f_po = InbreedingTracker::from_pedigree(p1, child_a, &lineage);
        assert!(
            (f_po - 0.25).abs() < 1e-4,
            "Parent-offspring mating must yield F = 0.25, got {f_po}"
        );

        // 2. Full siblings: child_a & child_b -> F = 0.25
        let f_full = InbreedingTracker::from_pedigree(child_a, child_b, &lineage);
        assert!(
            (f_full - 0.25).abs() < 1e-4,
            "Full-sibling mating must yield F = 0.25, got {f_full}"
        );

        // 3. Half siblings: child_a & child_c -> F = 0.125
        let f_half = InbreedingTracker::from_pedigree(child_a, child_c, &lineage);
        assert!(
            (f_half - 0.125).abs() < 1e-4,
            "Half-sibling mating must yield F = 0.125, got {f_half}"
        );
    }

    #[test]
    fn test_describe_relationship_tiers() {
        assert_eq!(describe_relationship(0.0), "unrelated");
        assert_eq!(describe_relationship(0.015625), "second cousins");
        assert_eq!(describe_relationship(0.0625), "first cousins");
        assert_eq!(describe_relationship(0.125), "half-siblings / uncle-niece");
        assert_eq!(
            describe_relationship(0.25),
            "full siblings / parent-offspring"
        );
        assert_eq!(describe_relationship(0.5), "self-fertilization");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_mortality_risk_increase() {
        let tracker_outbred = InbreedingTracker::new();
        let tracker_inbred = InbreedingTracker::with_f_coefficient(0.5);

        let risk_outbred = tracker_outbred.mortality_risk_increase();
        let risk_inbred = tracker_inbred.mortality_risk_increase();

        assert!(
            risk_inbred > risk_outbred,
            "Inbred individuals should have higher mortality risk"
        );
        assert_eq!(risk_outbred, 0.0, "Outbred should have no increased risk");
    }

    #[test]
    fn test_population_stats() {
        let trackers = vec![
            InbreedingTracker::new(),                     // F = 0
            InbreedingTracker::with_f_coefficient(0.125), // F = 0.125
            InbreedingTracker::with_f_coefficient(0.26),  // F = 0.26 (> 0.25 = severe)
            InbreedingTracker::with_f_coefficient(0.0),   // F = 0
        ];

        let stats = compute_population_inbreeding_stats(&trackers);

        assert!(stats.mean_f > 0.0, "Should have some mean F");
        assert!(stats.mean_f < 0.5, "Mean F should be reasonable");
        assert!(
            stats.inbred_fraction > 0.0,
            "Should have some inbred individuals"
        );
        assert!(
            stats.severe_inbreeding_fraction > 0.0,
            "Should have some severe inbreeding"
        );
    }

    #[test]
    fn test_describe_relationship() {
        assert_eq!(describe_relationship(0.0), "unrelated");
        assert_eq!(describe_relationship(0.0625), "first cousins");
        assert_eq!(describe_relationship(0.125), "half-siblings / uncle-niece");
        assert_eq!(
            describe_relationship(0.25),
            "full siblings / parent-offspring"
        );
        assert_eq!(describe_relationship(0.5), "self-fertilization");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_has_depression() {
        let outbred = InbreedingTracker::new();
        let inbred = InbreedingTracker::with_f_coefficient(0.5);

        assert!(
            !outbred.has_depression(),
            "Outbred should not have depression"
        );
        assert!(inbred.has_depression(), "Inbred should have depression");
    }

    #[test]
    fn test_depression_severity() {
        let outbred = InbreedingTracker::new();
        let inbred = InbreedingTracker::with_f_coefficient(0.5);

        assert_eq!(outbred.depression_severity(), 0.0);
        assert!(inbred.depression_severity() > 0.0);
        assert!(inbred.depression_severity() <= 1.0);
    }

    #[test]
    fn test_predict_offspring_fitness() {
        let lineage = create_test_lineage();
        let genome = Genome::default();

        // Predict fitness for unrelated parents
        let fitness = predict_offspring_fitness(1, 3, &lineage, Some(&genome), Some(&genome));

        assert!(fitness > 0.8, "Unrelated parents should have fit offspring");
        assert!(fitness <= 1.0, "Fitness should not exceed 1.0");
    }

    #[test]
    fn test_f_coefficient_clamping() {
        let tracker_low = InbreedingTracker::with_f_coefficient(-0.5);
        let tracker_high = InbreedingTracker::with_f_coefficient(1.5);

        assert_eq!(tracker_low.f_coefficient, 0.0);
        assert_eq!(tracker_high.f_coefficient, 1.0);
    }
}
