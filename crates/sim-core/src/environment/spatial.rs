//! Spatial Indexing
//!
//! Occupancy grid for collision detection and spatial hash for efficient
//! neighbor lookup. Now supports 3D with altitude tracking.

use std::collections::HashMap;

use crate::types::Position;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Occupancy grid for collision detection
/// Per §7.4.0 lines 4088-4110: Sparse occupancy tracking
/// Now supports 3D with altitude tracking
#[derive(Debug, Clone, Default)]
pub struct OccupancyGrid {
    /// Map from cell coordinates (x, y, z) to agent ID
    occupied: HashMap<(i32, i32, i32), u64>,
}

impl OccupancyGrid {
    /// Create a new empty occupancy grid
    #[must_use]
    pub fn new() -> Self {
        Self {
            occupied: HashMap::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if a cell is occupied
    /// Per §7.4.0 lines 4096-4098
    #[must_use]
    pub fn is_occupied(&self, pos: &Position) -> bool {
        self.occupied
            .contains_key(&(pos.x as i32, pos.y as i32, pos.z as i32))
    }

    /// Check if a 2D cell is occupied (ignores z, any altitude)
    #[must_use]
    pub fn is_occupied_2d(&self, pos: &Position) -> bool {
        // Check all altitudes at this x,y
        self.occupied
            .keys()
            .any(|(ox, oy, _)| *ox == pos.x as i32 && *oy == pos.y as i32)
    }

    /// Get the agent ID occupying a cell, if any
    #[must_use]
    pub fn get_occupant(&self, pos: &Position) -> Option<u64> {
        self.occupied
            .get(&(pos.x as i32, pos.y as i32, pos.z as i32))
            .copied()
    }

    /// Update the occupancy grid from agent positions
    /// Per §7.4.0 lines 4100-4108
    pub fn update(&mut self, agents: &[(u64, Position, bool)]) {
        self.occupied.clear();
        for (agent_id, position, alive) in agents {
            if *alive {
                self.occupied.insert(
                    (position.x as i32, position.y as i32, position.z as i32),
                    *agent_id,
                );
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update from agent slice with id, position, alive fields
    pub fn update_from_agents<'a, I>(&mut self, agents: I)
    where
        I: Iterator<Item = (u64, &'a Position, bool)>,
    {
        self.occupied.clear();
        for (agent_id, position, alive) in agents {
            if alive {
                self.occupied.insert(
                    (position.x as i32, position.y as i32, position.z as i32),
                    agent_id,
                );
            }
        }
    }
}

/// Spatial hash grid for efficient neighbor lookup
/// Per §23.8.2 GAP-P1-05: Replace O(n²) with O(n) neighbor lookup
/// Cell size should be >= communication radius for single-cell lookup
/// Now supports 3D with z-coordinate hashing
#[derive(Debug, Clone, Default)]
pub struct SpatialHash {
    /// Cell size (should be >= max query radius)
    cell_size: f32,
    /// Map from cell coordinates to list of (`agent_index`, position)
    /// Key includes z coordinate for 3D
    cells: HashMap<(i32, i32, i32), Vec<(usize, Position)>>,
}

impl SpatialHash {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new spatial hash with given cell size
    #[must_use]
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Get cell coordinates for a position
    #[must_use]
    fn cell_coords(&self, pos: &Position) -> (i32, i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    /// Clear and rebuild the spatial hash from agent positions
    pub fn rebuild(&mut self, agents: &[(usize, Position)]) {
        self.cells.clear();
        for &(idx, pos) in agents {
            let coords = self.cell_coords(&pos);
            self.cells.entry(coords).or_default().push((idx, pos));
        }
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Find all agents within radius of a position
    /// Returns vector of (`agent_index`, position, distance)
    #[must_use]
    pub fn query_radius(&self, pos: &Position, radius: f32) -> Vec<(usize, Position, f32)> {
        let mut results = Vec::new();
        let (cx, cy, cz) = self.cell_coords(pos);

        // Check cells within radius (need to check neighboring cells too)
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                for dz in -cell_radius..=cell_radius {
                    let cell_key = (cx + dx, cy + dy, cz + dz);
                    if let Some(agents) = self.cells.get(&cell_key) {
                        for &(idx, agent_pos) in agents {
                            let dist = pos.distance(&agent_pos);
                            if dist <= radius {
                                results.push((idx, agent_pos, dist));
                            }
                        }
                    }
                }
            }
        }

        results
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Find all pairs of agents within radius of each other
    /// Returns vector of (`agent_index_a`, `agent_index_b`) where a < b
    /// This is O(n * k) where k is average neighbors per agent
    #[must_use]
    pub fn find_pairs_within_radius(&self, radius: f32) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for (&(cx, cy, cz), agents) in &self.cells {
            let cell_radius = (radius / self.cell_size).ceil() as i32;

            for dx in 0..=cell_radius {
                for dy in -cell_radius..=cell_radius {
                    for dz in -cell_radius..=cell_radius {
                        if Self::should_skip_cell(dx, dy, dz) {
                            continue;
                        }

                        let neighbor_key = (cx + dx, cy + dy, cz + dz);

                        if dx == 0 && dy == 0 && dz == 0 {
                            Self::collect_same_cell_pairs(agents, radius, &mut seen, &mut pairs);
                        } else if let Some(neighbor_agents) = self.cells.get(&neighbor_key) {
                            Self::collect_cross_cell_pairs(
                                agents,
                                neighbor_agents,
                                radius,
                                &mut seen,
                                &mut pairs,
                            );
                        }
                    }
                }
            }
        }

        pairs.sort_unstable();
        pairs
    }

    const fn should_skip_cell(dx: i32, dy: i32, dz: i32) -> bool {
        if dx == 0 && dy < 0 {
            return true;
        }
        if dx == 0 && dy == 0 && dz < 0 {
            return true;
        }
        false
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn collect_same_cell_pairs(
        agents: &[(usize, Position)],
        radius: f32,
        seen: &mut std::collections::HashSet<(usize, usize)>,
        pairs: &mut Vec<(usize, usize)>,
    ) {
        for i in 0..agents.len() {
            for j in (i + 1)..agents.len() {
                let (idx_a, pos_a) = agents[i];
                let (idx_b, pos_b) = agents[j];
                if pos_a.distance(&pos_b) <= radius {
                    let pair = Self::ordered_pair(idx_a, idx_b);
                    if seen.insert(pair) {
                        pairs.push(pair);
                    }
                }
            }
        }
    }

    fn collect_cross_cell_pairs(
        agents: &[(usize, Position)],
        neighbor_agents: &[(usize, Position)],
        radius: f32,
        seen: &mut std::collections::HashSet<(usize, usize)>,
        pairs: &mut Vec<(usize, usize)>,
    ) {
        for &(idx_a, pos_a) in agents {
            for &(idx_b, pos_b) in neighbor_agents {
                if pos_a.distance(&pos_b) <= radius {
                    let pair = Self::ordered_pair(idx_a, idx_b);
                    if seen.insert(pair) {
                        pairs.push(pair);
                    }
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    const fn ordered_pair(a: usize, b: usize) -> (usize, usize) {
        if a < b {
            (a, b)
        } else {
            (b, a)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =======================================================================
    // P2.2: SpatialHash Unit Tests (Phase 16)
    // Per §23.8.2 GAP-P1-05: Spatial hash for O(n) neighbor lookup
    // =======================================================================

    #[test]
    fn test_spatial_hash_query_radius() {
        let mut hash = SpatialHash::new(3.0);

        // Add agents at various positions
        let agents = vec![
            (0, Position::new_2d(0.0, 0.0)),
            (1, Position::new_2d(1.0, 1.0)),
            (2, Position::new_2d(5.0, 5.0)),
            (3, Position::new_2d(10.0, 10.0)),
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        hash.rebuild(&agents);

        // Query from origin with radius 2.0
        let results = hash.query_radius(&Position::new_2d(0.0, 0.0), 2.0);

        // Should find agents 0 and 1 (within radius 2.0)
        let found_indices: Vec<usize> = results.iter().map(|(idx, _, _)| *idx).collect();
        assert!(found_indices.contains(&0), "Should find agent 0");
        assert!(found_indices.contains(&1), "Should find agent 1");
        assert!(
            !found_indices.contains(&2),
            "Should not find agent 2 (too far)"
        );
        assert!(
            !found_indices.contains(&3),
            "Should not find agent 3 (too far)"
        );
    }

    /// Test `SpatialHash` finds all pairs within radius
    #[test]
    fn test_spatial_hash_find_pairs() {
        let mut hash = SpatialHash::new(3.0);

        // Add agents in a cluster
        let agents = vec![
            (0, Position::new_2d(0.0, 0.0)),
            (1, Position::new_2d(1.0, 0.0)),
            (2, Position::new_2d(2.0, 0.0)),
            (3, Position::new_2d(10.0, 10.0)), // Far away
        ];

        hash.rebuild(&agents);

        // Find pairs within radius 1.5
        let pairs = hash.find_pairs_within_radius(1.5);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Should find pairs (0,1) and (1,2)
        assert!(pairs.contains(&(0, 1)), "Should find pair (0,1)");
        assert!(pairs.contains(&(1, 2)), "Should find pair (1,2)");
        assert!(
            !pairs.contains(&(0, 2)),
            "Should not find pair (0,2) - distance is 2.0"
        );
        assert!(
            !pairs.contains(&(0, 3)),
            "Should not find pair (0,3) - too far"
        );
    }

    /// Test `SpatialHash` in 3D
    #[test]
    fn test_spatial_hash_3d() {
        let mut hash = SpatialHash::new(3.0);

        // Add agents at various 3D positions
        let agents = vec![
            (0, Position::new(0.0, 0.0, 0.0)),
            (1, Position::new(1.0, 1.0, 0.0)),
            (2, Position::new(0.0, 0.0, 5.0)), // Far in z direction
            (3, Position::new(0.0, 0.0, 1.0)), // Close in z direction
        ];

        hash.rebuild(&agents);

        // Query from origin with radius 2.0
        let results = hash.query_radius(&Position::new(0.0, 0.0, 0.0), 2.0);

        // Should find agents 0, 1, and 3 (within radius 2.0)
        let found_indices: Vec<usize> = results.iter().map(|(idx, _, _)| *idx).collect();
        assert!(found_indices.contains(&0), "Should find agent 0");
        assert!(found_indices.contains(&1), "Should find agent 1");
        assert!(
            !found_indices.contains(&2),
            "Should not find agent 2 (too far in z)"
        );
        assert!(
            found_indices.contains(&3),
            "Should find agent 3 (close in z)"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Test `SpatialHash` returns sorted pairs for determinism
    #[test]
    fn test_spatial_hash_determinism() {
        let mut hash1 = SpatialHash::new(3.0);
        let mut hash2 = SpatialHash::new(3.0);

        let agents = vec![
            (0, Position::new_2d(0.0, 0.0)),
            (1, Position::new_2d(1.0, 0.0)),
            (2, Position::new_2d(0.0, 1.0)),
            (3, Position::new_2d(1.0, 1.0)),
        ];

        hash1.rebuild(&agents);
        hash2.rebuild(&agents);

        let pairs1 = hash1.find_pairs_within_radius(2.0);
        let pairs2 = hash2.find_pairs_within_radius(2.0);

        // Results should be identical and sorted
        assert_eq!(pairs1, pairs2, "Same input should produce same output");

        // Verify sorted
        for i in 1..pairs1.len() {
            assert!(pairs1[i - 1] <= pairs1[i], "Pairs should be sorted");
        }
    }

    /// Test `SpatialHash` handles empty input
    #[test]
    fn test_spatial_hash_empty() {
        let mut hash = SpatialHash::new(3.0);
        hash.rebuild(&[]);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let results = hash.query_radius(&Position::new_2d(0.0, 0.0), 10.0);
        assert!(results.is_empty(), "Empty hash should return no results");

        let pairs = hash.find_pairs_within_radius(10.0);
        assert!(pairs.is_empty(), "Empty hash should return no pairs");
    }

    /// Test `OccupancyGrid` 3D support
    #[test]
    fn test_occupancy_grid_3d() {
        let mut grid = OccupancyGrid::new();

        let agents = vec![
            (1u64, Position::new(0.0, 0.0, 0.0), true),
            (2u64, Position::new(0.0, 0.0, 1.0), true), // Same x,y, different z
            (3u64, Position::new(1.0, 1.0, 0.0), true),
        ];

        grid.update(&agents);

        // Should track 3D positions separately
        assert!(grid.is_occupied(&Position::new(0.0, 0.0, 0.0)));
        assert!(grid.is_occupied(&Position::new(0.0, 0.0, 1.0)));
        assert!(!grid.is_occupied(&Position::new(0.0, 0.0, 2.0)));

        // 2D check should find any at x,y
        assert!(grid.is_occupied_2d(&Position::new(0.0, 0.0, 0.0)));
    }
}
