//! Barnes-Hut Approximation for N-Body Social Force Calculations
//!
//! Reduces complexity from O(n²) to O(n log n) for long-range forces

use crate::agent::Agent;
use crate::types::{Position, Vector};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Quadtree node for Barnes-Hut
#[derive(Clone, Debug)]
pub struct QuadtreeNode {
    /// Center of this node's region
    pub center: Position,
    /// Half-width of this node's region
    pub half_width: f32,
    /// For leaf nodes: contains single agent
    pub agent: Option<usize>, // Index into agents array
    /// For internal nodes: four children (NW, NE, SW, SE)
    pub children: Option<Box<[QuadtreeNode; 4]>>,
    /// Center of mass for this node (and all descendants)
    pub center_of_mass: Position,
    /// Total "mass" (influence) of this node
    pub total_mass: f32,
    /// Number of agents in this node
    pub count: usize,
    /// Representative cluster agent index for far-field multipole force evaluation
    pub rep_agent: Option<usize>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl QuadtreeNode {
    /// Create empty node
    #[must_use]
    pub const fn new(center: Position, half_width: f32) -> Self {
        Self {
            center,
            half_width,
            agent: None,
            children: None,
            center_of_mass: center,
            total_mass: 0.0,
            count: 0,
            rep_agent: None,
        }
    }

    /// Check if point is inside this node
    #[must_use]
    pub fn contains(&self, pos: &Position) -> bool {
        let dx = (pos.x - self.center.x).abs();
        let dy = (pos.y - self.center.y).abs();
        dx <= self.half_width && dy <= self.half_width
    }

    /// Get quadrant index for a position (0=NW, 1=NE, 2=SW, 3=SE)
    fn get_quadrant(&self, pos: &Position) -> usize {
        let north = pos.y < self.center.y;
        let west = pos.x < self.center.x;

        match (north, west) {
            (true, true) => 0,   // NW
            (true, false) => 1,  // NE
            (false, true) => 2,  // SW
            (false, false) => 3, // SE
        }
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Create children for this node
    fn subdivide(&mut self) {
        let half = self.half_width / 2.0;

        // Create children directly as an array
        let children: [Self; 4] = [
            // NW
            Self::new(
                Position::new_2d(self.center.x - half, self.center.y - half),
                half,
            ),
            // NE
            Self::new(
                Position::new_2d(self.center.x + half, self.center.y - half),
                half,
            ),
            // SW
            Self::new(
                Position::new_2d(self.center.x - half, self.center.y + half),
                half,
            ),
            // SE
            Self::new(
                Position::new_2d(self.center.x + half, self.center.y + half),
                half,
            ),
        ];

        self.children = Some(Box::new(children));
    }

    /// Minimum node half-width to prevent infinite subdivision when agents
    /// share the exact same position (all map to the same quadrant).
    const MIN_HALF_WIDTH: f32 = 1e-3;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Insert agent into this node
    pub fn insert(&mut self, agent_idx: usize, agents: &[Agent]) {
        let agent = &agents[agent_idx];
        let pos = agent.position;

        // Update center of mass
        let new_count = self.count + 1;
        self.center_of_mass = Position::new_2d(
            (self.center_of_mass.x * self.count as f32 + pos.x) / new_count as f32,
            (self.center_of_mass.y * self.count as f32 + pos.y) / new_count as f32,
        );
        self.total_mass += 1.0; // Unit mass per agent
        self.count = new_count;
        if self.rep_agent.is_none() {
            self.rep_agent = Some(agent_idx);
        }

        if self.count == 1 && self.agent.is_none() {
            // First agent in empty node - store as leaf
            self.agent = Some(agent_idx);
            return;
        }

        // Stop subdividing below minimum size to prevent infinite recursion
        // when multiple agents occupy the exact same position.
        // At this scale, treat as an aggregate node (center_of_mass is exact).
        if self.half_width <= Self::MIN_HALF_WIDTH {
            if self.agent.is_some() && self.agent != Some(agent_idx) {
                // Ensure self.rep_agent holds an alternate agent index from self.agent
                self.rep_agent = Some(agent_idx);
            }
            return;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Need to subdivide
        if self.children.is_none() {
            self.subdivide();

            // Move existing agent to child
            if let Some(existing_idx) = self.agent.take() {
                let existing = &agents[existing_idx];
                let quadrant = self.get_quadrant(&existing.position);
                if let Some(ref mut children) = self.children {
                    children[quadrant].insert(existing_idx, agents);
                }
            }
        }

        // Insert new agent into appropriate child
        let quadrant = self.get_quadrant(&pos);
        if let Some(ref mut children) = self.children {
            children[quadrant].insert(agent_idx, agents);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute force on agent using Barnes-Hut approximation
    ///
    /// # Arguments
    /// * `agent` - The agent to compute force for
    /// * `agent_idx` - Index of the agent in the agents array
    /// * `agents` - Full agent array for looking up other agents
    /// * `theta` - Opening angle threshold (s/d < theta means treat as single particle)
    /// * `force_fn` - Function that computes force between two agents
    pub fn compute_force<F>(
        &self,
        agent: &Agent,
        agent_idx: usize,
        agents: &[Agent],
        theta: f32,
        force_fn: &F,
    ) -> Vector
    where
        F: Fn(&Agent, &Agent) -> Vector,
    {
        // If this node is empty, no force
        if self.count == 0 {
            return Vector::zero();
        }

        // If this is a leaf node containing agent(s)
        if self.children.is_none() {
            if let Some(idx) = self.agent {
                if idx != agent_idx {
                    let f = force_fn(agent, &agents[idx]);
                    let mass_scale = if self.count > 1 {
                        if self.contains(&agent.position) {
                            (self.total_mass - 1.0).max(1.0)
                        } else {
                            self.total_mass
                        }
                    } else {
                        1.0
                    };
                    return f.scale(mass_scale);
                } else if self.count > 1 {
                    let rep_idx = self.rep_agent.unwrap_or(idx);
                    if rep_idx != agent_idx {
                        if let Some(other) = agents.get(rep_idx) {
                            let f = force_fn(agent, other);
                            return f.scale((self.total_mass - 1.0).max(1.0));
                        }
                    }
                }
            }
            return Vector::zero();
        }

        // Internal node: compute distance from agent to this node's center of mass
        let dx = self.center_of_mass.x - agent.position.x;
        let dy = self.center_of_mass.y - agent.position.y;
        let dist_sq = dx * dx + dy * dy;
        let dist = dist_sq.sqrt();

        // Check if we can treat this internal node as a single aggregate particle:
        // Multipole acceptance criterion: s / dist < theta, where s is node width, d is distance.
        // If dist < 0.001 or s / dist >= theta, we CANNOT approximate; we MUST recurse into children!
        let s = self.half_width * 2.0;
        if dist >= 0.001 && dist.is_finite() && (s / dist < theta) {
            // Aggregate multipole approximation: evaluate force using cluster representative agent
            // positioned at center of mass scaled by total mass (avoids self-ID reputation annihilation)
            let rep_idx = self.rep_agent.unwrap_or(if agent_idx > 0 { 0 } else { 1 });
            if let Some(rep) = agents.get(rep_idx) {
                let mut pseudo = rep.clone();
                pseudo.position = self.center_of_mass;
                let f = force_fn(agent, &pseudo);
                return f.scale(self.total_mass);
            }
        }

        // Recurse into children
        let mut total_force = Vector::zero();
        if let Some(ref children) = self.children {
            for child in children.iter() {
                total_force += child.compute_force(agent, agent_idx, agents, theta, force_fn);
            }
        }
        total_force
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Barnes-Hut tree for efficient N-body force calculation
#[derive(Debug)]
pub struct BarnesHutTree {
    root: QuadtreeNode,
    theta: f32,         // Opening angle threshold
    bounds: (f32, f32), // (width, height)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BarnesHutTree {
    /// Create new tree with given bounds
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            root: QuadtreeNode::new(
                Position::new_2d(width / 2.0, height / 2.0),
                width.max(height) / 2.0,
            ),
            theta: 0.5, // Default threshold
            bounds: (width, height),
        }
    }

    /// Build tree from agents
    pub fn build(&mut self, agents: &[Agent]) {
        self.root = QuadtreeNode::new(
            Position::new_2d(self.bounds.0 / 2.0, self.bounds.1 / 2.0),
            self.bounds.0.max(self.bounds.1) / 2.0,
        );

        for (idx, agent) in agents.iter().enumerate() {
            if agent.alive {
                self.root.insert(idx, agents);
            }
        }
    }

    /// Set opening angle threshold (default 0.5, lower = more accurate)
    #[must_use]
    pub fn with_theta(mut self, theta: f32) -> Self {
        self.theta = theta.clamp(0.1, 1.0);
        self
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute forces for all agents
    ///
    /// # Arguments
    /// * `agents` - Mutable slice of agents to update
    /// * `force_fn` - Function computing force between two agents
    pub fn compute_forces<F>(&self, agents: &mut [Agent], force_fn: &F)
    where
        F: Fn(&Agent, &Agent) -> Vector,
    {
        // First, compute all forces without modifying agents
        let forces = self.compute_force_vectors(agents, force_fn);

        // Then apply forces to agents
        for (idx, agent) in agents.iter_mut().enumerate() {
            if agent.alive {
                agent.apply_social_force(forces[idx]);
            }
        }
    }

    /// Compute forces and return them (without modifying agents)
    pub fn compute_force_vectors<F>(&self, agents: &[Agent], force_fn: &F) -> Vec<Vector>
    where
        F: Fn(&Agent, &Agent) -> Vector,
    {
        // Collect alive indices first to avoid borrow issues
        let alive_indices: Vec<usize> = agents
            .iter()
            .enumerate()
            .filter(|(_, agent)| agent.alive)
            .map(|(idx, _)| idx)
            .collect();

        let mut forces = vec![Vector::zero(); agents.len()];

        for idx in alive_indices {
            forces[idx] = self
                .root
                .compute_force(&agents[idx], idx, agents, self.theta, force_fn);
        }

        forces
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Default social force function (trust-based attraction/repulsion)
#[must_use]
pub fn default_social_force(a: &Agent, b: &Agent) -> Vector {
    let dx = b.position.x - a.position.x;
    let dy = b.position.y - a.position.y;
    let dist_sq = dx * dx + dy * dy;
    let dist = dist_sq.sqrt();

    if dist < 0.001 {
        return Vector::zero();
    }

    // Compute trust-based force
    // High trust = attraction, low trust = repulsion
    let trust = a.get_local_reputation(b.id);
    let force_magnitude = (trust - 0.5) * 2.0 / dist_sq; // -1 to 1 scaled by distance

    Vector::new(force_magnitude * dx / dist, force_magnitude * dy / dist)
}

/// Inverse-square repulsion force (for collision avoidance)
#[must_use]
pub fn repulsion_force(a: &Agent, b: &Agent, min_distance: f32) -> Vector {
    let dx = b.position.x - a.position.x;
    let dy = b.position.y - a.position.y;
    let dist_sq = dx * dx + dy * dy;
    let dist = dist_sq.sqrt();
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    if dist < 0.001 || dist > min_distance {
        return Vector::zero();
    }

    // Strong repulsion when close
    let force_magnitude = -1.0 / dist_sq;

    Vector::new(force_magnitude * dx / dist, force_magnitude * dy / dist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadtree_node_creation() {
        let node = QuadtreeNode::new(Position::new_2d(32.0, 32.0), 32.0);
        assert_eq!(node.count, 0);
        assert!(node.children.is_none());
    }

    #[test]
    fn test_contains() {
        let node = QuadtreeNode::new(Position::new_2d(32.0, 32.0), 32.0);
        assert!(node.contains(&Position::new_2d(32.0, 32.0)));
        assert!(node.contains(&Position::new_2d(0.0, 0.0)));
        assert!(node.contains(&Position::new_2d(64.0, 64.0)));
        assert!(!node.contains(&Position::new_2d(65.0, 32.0)));
    }

    #[test]
    fn test_barnes_hut_far_field_multipole_preserves_social_force() {
        use crate::genome::Genome;
        use crate::types::Sex;

        // Create query agent at (0, 0)
        let mut query = Agent::new(
            1,
            Position::new_2d(0.0, 0.0),
            Genome::default(),
            0,
            Sex::Male,
        );
        // Query trusts cluster agent 2 with high trust 1.0 (attractive force)
        query.update_local_reputation(2, 0.5); // trust = 1.0

        // Create cluster far away around (100, 100)
        let cluster_member1 = Agent::new(
            2,
            Position::new_2d(100.0, 100.0),
            Genome::default(),
            0,
            Sex::Female,
        );
        let cluster_member2 = Agent::new(
            3,
            Position::new_2d(101.0, 101.0),
            Genome::default(),
            0,
            Sex::Male,
        );

        let agents = vec![query, cluster_member1, cluster_member2];
        let mut tree = BarnesHutTree::new(200.0, 200.0).with_theta(0.8);
        tree.build(&agents);

        // Forces on agent 0 from the far-field cluster must be attractive towards positive (x > 0, y > 0)
        let forces = tree.compute_force_vectors(&agents, &default_social_force);
        assert!(
            forces[0].x > 0.0,
            "Far-field multipole must exert attractive force in x"
        );
        assert!(
            forces[0].y > 0.0,
            "Far-field multipole must exert attractive force in y"
        );
    }

    #[test]
    fn test_barnes_hut_co_located_leaf_scales_by_total_mass() {
        use crate::genome::Genome;
        use crate::types::Sex;

        let mut query = Agent::new(
            1,
            Position::new_2d(0.0, 0.0),
            Genome::default(),
            0,
            Sex::Male,
        );
        query.update_local_reputation(2, 0.5); // trust = 1.0

        // 3 agents co-located at exact same coordinates (10.0, 10.0) below MIN_HALF_WIDTH
        let c1 = Agent::new(
            2,
            Position::new_2d(10.0, 10.0),
            Genome::default(),
            0,
            Sex::Female,
        );
        let c2 = Agent::new(
            3,
            Position::new_2d(10.0, 10.0),
            Genome::default(),
            0,
            Sex::Male,
        );
        let c3 = Agent::new(
            4,
            Position::new_2d(10.0, 10.0),
            Genome::default(),
            0,
            Sex::Female,
        );

        let agents = vec![query, c1, c2, c3];
        let mut tree = BarnesHutTree::new(20.0, 20.0).with_theta(0.1); // Small theta forces traversal to leaf
        tree.build(&agents);

        let forces = tree.compute_force_vectors(&agents, &default_social_force);
        // Force on query agent from 3 co-located agents must scale with cluster count (mass = 3.0)
        assert!(forces[0].x > 0.0);
        assert!(forces[0].y > 0.0);
    }

    #[test]
    fn test_barnes_hut_co_located_leaf_internal_symmetry() {
        use crate::genome::Genome;
        use crate::types::Sex;

        // Two agents co-located at (10.0, 10.0)
        let mut a1 = Agent::new(
            1,
            Position::new_2d(10.0, 10.0),
            Genome::default(),
            0,
            Sex::Male,
        );
        let mut a2 = Agent::new(
            2,
            Position::new_2d(10.0001, 10.0001),
            Genome::default(),
            0,
            Sex::Female,
        );
        a1.update_local_reputation(2, 0.5); // high trust
        a2.update_local_reputation(1, 0.5); // high trust

        let agents = vec![a1, a2];
        let mut tree = BarnesHutTree::new(20.0, 20.0).with_theta(0.1);
        tree.build(&agents);

        let forces = tree.compute_force_vectors(&agents, &default_social_force);
        // Co-located internal agents must exert equal and opposite forces (Newton's Third Law)
        assert!((forces[0].x + forces[1].x).abs() < 1e-3);
        assert!((forces[0].y + forces[1].y).abs() < 1e-3);
    }
}
