//! Social Network Structure (P1-AGENT-5)

use serde::{Deserialize, Serialize};

use super::NetworkType;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Social network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialNetwork {
    /// network type.
    pub network_type: NetworkType,
    /// adjacency list.
    pub adjacency_list: Vec<Vec<(usize, f32)>>, // node -> [(neighbor, weight)]
    /// node ids.
    pub node_ids: Vec<u64>, // index -> agent_id
}

impl SocialNetwork {
    /// Create new network
    #[must_use]
    pub const fn new(network_type: NetworkType) -> Self {
        Self {
            network_type,
            adjacency_list: Vec::new(),
            node_ids: Vec::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add node to network
    pub fn add_node(&mut self, agent_id: u64) -> usize {
        if let Some(index) = self.node_ids.iter().position(|&id| id == agent_id) {
            return index;
        }

        let index = self.node_ids.len();
        self.node_ids.push(agent_id);
        self.adjacency_list.push(Vec::new());
        index
    }

    /// Add edge between nodes
    pub fn add_edge(&mut self, from: u64, to: u64, weight: f32) {
        let from_idx = self.add_node(from);
        let to_idx = self.add_node(to);

        // Add bidirectional edge
        self.adjacency_list[from_idx].push((to_idx, weight));
        self.adjacency_list[to_idx].push((from_idx, weight));
    }

    /// Calculate network metrics
    #[must_use]
    pub fn calculate_metrics(&self) -> NetworkMetrics {
        let n = self.node_ids.len();
        if n == 0 {
            return NetworkMetrics::default();
        }

        // Density
        let max_edges = n * (n - 1) / 2;
        let actual_edges: usize = self.adjacency_list.iter().map(std::vec::Vec::len).sum();
        let density = if max_edges > 0 {
            (actual_edges / 2) as f32 / max_edges as f32
        } else {
            0.0
        };

        // Average degree
        let avg_degree = self
            .adjacency_list
            .iter()
            .map(std::vec::Vec::len)
            .sum::<usize>() as f32
            / n as f32;

        // Average clustering coefficient
        let clustering = self.calculate_average_clustering();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        NetworkMetrics {
            node_count: n,
            edge_count: actual_edges / 2,
            density,
            avg_degree,
            clustering_coefficient: clustering,
            diameter: self.estimate_diameter(),
        }
    }

    /// Calculate average clustering coefficient
    fn calculate_average_clustering(&self) -> f32 {
        if self.node_ids.len() < 3 {
            return 0.0;
        }

        let coefficients: Vec<f32> = (0..self.node_ids.len())
            .map(|i| self.local_clustering(i))
            .collect();

        coefficients.iter().sum::<f32>() / coefficients.len() as f32
    }

    /// Local clustering coefficient for node
    fn local_clustering(&self, node: usize) -> f32 {
        let neighbors: Vec<usize> = self.adjacency_list[node].iter().map(|(n, _)| *n).collect();

        if neighbors.len() < 2 {
            return 0.0;
        }

        let mut triangles = 0;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                if self.has_edge(neighbors[i], neighbors[j]) {
                    triangles += 1;
                }
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // P1-FIX: Use checked arithmetic to prevent overflow
        let max_triangles = neighbors
            .len()
            .checked_mul(neighbors.len().saturating_sub(1))
            .map_or(0, |v| v / 2);
        if max_triangles == 0 {
            0.0
        } else {
            triangles as f32 / max_triangles as f32
        }
    }

    /// Check if edge exists
    fn has_edge(&self, from: usize, to: usize) -> bool {
        self.adjacency_list[from].iter().any(|(n, _)| *n == to)
    }

    /// Estimate network diameter (longest shortest path)
    fn estimate_diameter(&self) -> u32 {
        if self.node_ids.len() < 2 {
            return 0;
        }

        // Sample from some nodes
        let sample_size = self.node_ids.len().min(10);
        let mut max_distance = 0;

        for start in 0..sample_size {
            let distances = self.bfs_distances(start);
            if let Some(&max_dist) = distances.iter().max() {
                max_distance = max_distance.max(max_dist);
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        max_distance
    }

    /// BFS to calculate distances from start node
    fn bfs_distances(&self, start: usize) -> Vec<u32> {
        let n = self.node_ids.len();
        let mut distances = vec![u32::MAX; n];
        let mut visited = vec![false; n];
        let mut queue = std::collections::VecDeque::new();

        distances[start] = 0;
        visited[start] = true;
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            for (neighbor, _) in &self.adjacency_list[current] {
                if !visited[*neighbor] {
                    visited[*neighbor] = true;
                    distances[*neighbor] = distances[current] + 1;
                    queue.push_back(*neighbor);
                }
            }
        }

        distances
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate centrality for a node
    #[must_use]
    pub fn centrality(&self, agent_id: u64) -> Option<f32> {
        let node = self.node_ids.iter().position(|&id| id == agent_id)?;

        // Degree centrality
        let degree = self.adjacency_list[node].len() as f32;
        let max_degree = (self.node_ids.len() - 1) as f32;

        Some(degree / max_degree)
    }

    /// Get community structure (simple clustering)
    #[must_use]
    pub fn detect_communities(&self) -> Vec<Vec<u64>> {
        let mut communities: Vec<Vec<u64>> = Vec::new();
        let mut assigned: Vec<bool> = vec![false; self.node_ids.len()];

        for i in 0..self.node_ids.len() {
            if assigned[i] {
                continue;
            }

            let mut community = vec![self.node_ids[i]];
            assigned[i] = true;

            // Add neighbors
            for (neighbor, _) in &self.adjacency_list[i] {
                if !assigned[*neighbor] {
                    community.push(self.node_ids[*neighbor]);
                    assigned[*neighbor] = true;
                }
            }

            communities.push(community);
        }

        communities
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Network metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// node count.
    pub node_count: usize,
    /// edge count.
    pub edge_count: usize,
    /// density.
    pub density: f32,
    /// avg degree.
    pub avg_degree: f32,
    /// clustering coefficient.
    pub clustering_coefficient: f32,
    /// diameter.
    pub diameter: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            node_count: 0,
            edge_count: 0,
            density: 0.0,
            avg_degree: 0.0,
            clustering_coefficient: 0.0,
            diameter: 0,
        }
    }
}

impl NetworkMetrics {
    /// Assess network cohesion
    #[must_use]
    pub fn cohesion(&self) -> f32 {
        if self.node_count < 2 {
            return 0.0;
        }
        (self.density + self.clustering_coefficient) / 2.0
    }

    /// Assess small-worldness
    #[must_use]
    pub fn small_world_coefficient(&self) -> f32 {
        if self.avg_degree == 0.0 {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            return 0.0;
        }
        let random_clustering = self.avg_degree / (self.node_count as f32);
        self.clustering_coefficient / random_clustering
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_creation() {
        let mut network = SocialNetwork::new(NetworkType::Friendship);
        network.add_edge(1, 2, 0.8);
        network.add_edge(2, 3, 0.7);

        assert_eq!(network.node_ids.len(), 3);
    }

    #[test]
    fn test_clustering() {
        let mut network = SocialNetwork::new(NetworkType::Friendship);
        // Create triangle
        network.add_edge(1, 2, 1.0);
        network.add_edge(2, 3, 1.0);
        network.add_edge(1, 3, 1.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let metrics = network.calculate_metrics();
        assert_eq!(metrics.clustering_coefficient, 1.0);
    }

    #[test]
    fn test_centrality() {
        let mut network = SocialNetwork::new(NetworkType::Friendship);
        // Star network
        network.add_edge(1, 2, 1.0);
        network.add_edge(1, 3, 1.0);
        network.add_edge(1, 4, 1.0);

        let central = network.centrality(1).unwrap();
        let peripheral = network.centrality(2).unwrap();
        assert!(central > peripheral);
    }

    #[test]
    fn test_community_detection() {
        let mut network = SocialNetwork::new(NetworkType::Friendship);
        // Two clusters
        network.add_edge(1, 2, 1.0);
        network.add_edge(2, 3, 1.0);
        network.add_edge(4, 5, 1.0);

        let communities = network.detect_communities();
        assert!(communities.len() >= 2);
    }
}
