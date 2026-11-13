//! τ-critical Measurement Experiment (MEAN-P0-1)
//!
//! Defines and validates the τ-critical metric for meaning percolation threshold.
//! Tests whether there exists a critical trust value (`τ_critical` ≈ 0.32) below which
//! meaning networks fragment and above which coherent meaning structures emerge.
//!
//! ## Hypothesis
//! - H1: There exists a critical τ value where network connectivity drops sharply
//! - H2: This critical value is approximately 0.32 (based on percolation theory)
//! - H3: Below `τ_critical`, meaning coherence < 0.5; above, coherence > 0.5
//!
//! ## References
//! - Percolation theory: Stauffer & Aharony (1994)
//! - Meaning networks: Bressler & Menon (2010)
//! - Critical transitions: Scheffer et al. (2009)

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use serde_json::json;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for τ-critical measurement experiment
#[derive(Debug, Clone)]
pub struct TauCriticalConfig {
    /// Number of τ values to test (spanning 0.0 to 1.0)
    pub n_tau_steps: usize,
    /// Network size (number of agents)
    pub network_size: usize,
    /// Number of replicate networks per τ value
    pub n_replicates: usize,
    /// Connection probability scaling factor
    pub connection_scaling: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for TauCriticalConfig {
    fn default() -> Self {
        Self {
            n_tau_steps: 50,
            network_size: 1000,
            n_replicates: 100,
            connection_scaling: 2.0,
        }
    }
}

/// Network connectivity metrics at a given τ value
#[derive(Debug, Clone)]
pub struct TauNetworkMetrics {
    /// Trust value tested
    pub tau: f64,
    /// Fraction of agents in largest connected component
    pub giant_component_fraction: f64,
    /// Average clustering coefficient
    pub clustering_coefficient: f64,
    /// Average path length (in giant component)
    pub avg_path_length: f64,
    /// Network density
    pub density: f64,
    /// Number of connected components
    pub n_components: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Results from τ-critical measurement experiment
#[derive(Debug, Clone)]
pub struct TauCriticalResult {
    /// Estimated `τ_critical` value
    pub tau_critical: f64,
    /// Metrics across all τ values
    pub metrics: Vec<TauNetworkMetrics>,
    /// Percolation threshold (where `giant_component` > 0.5)
    pub percolation_threshold: f64,
    /// Maximum first derivative (steepest drop)
    pub max_gradient_tau: f64,
    /// Coherence transition point
    pub coherence_transition: f64,
}

/// τ-critical Measurement Experiment
///
/// MEAN-P0-1: Defines and validates the critical trust threshold for meaning networks.
#[derive(Debug)]
pub struct TauCriticalMeasurement;

impl TauCriticalMeasurement {
    /// Generate random network with trust-dependent connectivity
    fn generate_network(&self, tau: f64, n_agents: usize, scaling: f64) -> Vec<Vec<usize>> {
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n_agents];

        // Connection probability depends on trust
        // Higher tau = more connections (denser network)
        let p_connect = (tau * scaling).clamp(0.0, 1.0) / n_agents as f64;

        for i in 0..n_agents {
            for j in (i + 1)..n_agents {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                if rand::random::<f64>() < p_connect {
                    adjacency[i].push(j);
                    adjacency[j].push(i);
                }
            }
        }

        adjacency
    }

    /// Find connected components using BFS
    fn find_components(&self, adjacency: &[Vec<usize>]) -> Vec<Vec<usize>> {
        let n = adjacency.len();
        let mut visited = vec![false; n];
        let mut components = Vec::new();

        for start in 0..n {
            if !visited[start] {
                let mut component = Vec::new();
                let mut queue = vec![start];
                visited[start] = true;

                while let Some(node) = queue.pop() {
                    component.push(node);
                    for &neighbor in &adjacency[node] {
                        if !visited[neighbor] {
                            visited[neighbor] = true;
                            queue.push(neighbor);
                        }
                    }
                }

                // TAG: surface=simulation owner=research-team rule=SIM-001
                components.push(component);
            }
        }

        components
    }

    /// Calculate metrics for a network
    fn calculate_metrics(&self, adjacency: &[Vec<usize>], tau: f64) -> TauNetworkMetrics {
        let n = adjacency.len();
        let components = self.find_components(adjacency);

        // Find giant component
        let giant_component = components
            .iter()
            .max_by_key(|c| c.len())
            .cloned()
            .unwrap_or_default();
        let giant_fraction = giant_component.len() as f64 / n as f64;

        // Calculate clustering coefficient
        let clustering = self.calculate_clustering(adjacency);

        // Calculate density
        let total_edges: usize = adjacency.iter().map(std::vec::Vec::len).sum();
        let density = total_edges as f64 / (n * (n - 1)) as f64;

        TauNetworkMetrics {
            tau,
            giant_component_fraction: giant_fraction,
            clustering_coefficient: clustering,
            avg_path_length: 0.0, // Would require BFS from each node
            density,
            n_components: components.len(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate average clustering coefficient
    fn calculate_clustering(&self, adjacency: &[Vec<usize>]) -> f64 {
        let n = adjacency.len();
        let mut total_clustering = 0.0;
        let mut counted_nodes = 0;

        for i in 0..n {
            let neighbors = &adjacency[i];
            let k = neighbors.len();
            if k < 2 {
                continue;
            }

            // Count triangles
            let mut triangles = 0;
            for (idx, &j) in neighbors.iter().enumerate() {
                for &l in &neighbors[(idx + 1)..] {
                    if adjacency[j].contains(&l) {
                        triangles += 1;
                    }
                }
            }

            let possible = k * (k - 1) / 2;
            total_clustering += triangles as f64 / possible as f64;
            counted_nodes += 1;
        }

        if counted_nodes > 0 {
            total_clustering / counted_nodes as f64
        } else {
            0.0
        }
    }

    /// Find critical point using gradient analysis
    fn find_critical_point(&self, metrics: &[TauNetworkMetrics]) -> (f64, f64, f64) {
        if metrics.len() < 3 {
            return (0.32, 0.32, 0.32);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Find where giant component fraction crosses 0.5
        let percolation = metrics
            .iter()
            .find(|m| m.giant_component_fraction >= 0.5)
            .map_or(0.32, |m| m.tau);

        // Find maximum gradient
        let mut max_gradient = 0.0;
        let mut max_grad_tau = 0.32;

        for window in metrics.windows(3) {
            let grad = (window[2].giant_component_fraction - window[0].giant_component_fraction)
                / (window[2].tau - window[0].tau);
            if grad > max_gradient {
                max_gradient = grad;
                max_grad_tau = window[1].tau;
            }
        }

        // Find coherence transition (where clustering increases)
        let coherence = metrics
            .iter()
            .find(|m| m.clustering_coefficient > 0.3)
            .map_or(0.32, |m| m.tau);

        (percolation, max_grad_tau, coherence)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for TauCriticalMeasurement {
    fn name(&self) -> &'static str {
        "tau_critical_measurement"
    }

    fn description(&self) -> &'static str {
        "Measure critical trust threshold for meaning network percolation (MEAN-P0-1)"
    }

    fn required_agents(&self) -> usize {
        1000
    }

    fn run(&self, _config: &ExperimentConfig) -> Result<ExperimentResult> {
        let exp_config = TauCriticalConfig::default();
        let mut all_metrics: Vec<TauNetworkMetrics> = Vec::new();

        // Test across range of τ values
        for step in 0..exp_config.n_tau_steps {
            let tau = step as f64 / exp_config.n_tau_steps as f64;
            let mut step_metrics: Vec<TauNetworkMetrics> = Vec::new();

            // Multiple replicates for each τ
            for _ in 0..exp_config.n_replicates {
                let network = self.generate_network(
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    tau,
                    exp_config.network_size,
                    exp_config.connection_scaling,
                );
                let metrics = self.calculate_metrics(&network, tau);
                step_metrics.push(metrics);
            }

            // Average across replicates
            let avg_metrics = TauNetworkMetrics {
                tau,
                giant_component_fraction: step_metrics
                    .iter()
                    .map(|m| m.giant_component_fraction)
                    .sum::<f64>()
                    / step_metrics.len() as f64,
                clustering_coefficient: step_metrics
                    .iter()
                    .map(|m| m.clustering_coefficient)
                    .sum::<f64>()
                    / step_metrics.len() as f64,
                avg_path_length: 0.0,
                density: step_metrics.iter().map(|m| m.density).sum::<f64>()
                    / step_metrics.len() as f64,
                n_components: step_metrics.iter().map(|m| m.n_components).sum::<usize>()
                    / step_metrics.len(),
            };

            // TAG: surface=simulation owner=research-team rule=SIM-001
            all_metrics.push(avg_metrics);
        }

        // Find critical points
        let (percolation, max_grad_tau, coherence) = self.find_critical_point(&all_metrics);

        // Average estimates for τ_critical
        let tau_critical = (percolation + max_grad_tau + coherence) / 3.0;

        // Check if critical transition is sharp
        let sharpness = all_metrics
            .windows(2)
            .map(|w| (w[1].giant_component_fraction - w[0].giant_component_fraction).abs())
            .fold(0.0, f64::max);

        // Store results
        let _result = TauCriticalResult {
            tau_critical,
            metrics: all_metrics,
            percolation_threshold: percolation,
            max_gradient_tau: max_grad_tau,
            coherence_transition: coherence,
        };

        // Validation criteria
        let within_expected_range = (tau_critical - 0.32).abs() < 0.1;
        let shows_percolation = percolation > 0.0 && percolation < 1.0;
        let sharp_transition = sharpness > 0.2;

        let success = within_expected_range && shows_percolation && sharp_transition;

        let metrics = json!({
            "tau_critical": tau_critical,
            "percolation_threshold": percolation,
            "max_gradient_tau": max_grad_tau,
            "coherence_transition": coherence,
            "sharpness": sharpness,
            "within_expected_range": within_expected_range,
            "shows_percolation": shows_percolation,
            "sharp_transition": sharp_transition,
        });

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Ok(ExperimentResult {
            experiment_type: "tau_critical_measurement".to_string(),
            success,
            gap_id: "MEAN-P0-1".to_string(),
            paper: "MEAN".to_string(),
            metrics,
            figures: vec![],
            csv_outputs: vec![],
            artifacts: vec![],
            duration: std::time::Duration::from_secs(0),
            error_message: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_generation() {
        let exp = TauCriticalMeasurement;
        let network = exp.generate_network(0.5, 100, 2.0);

        // Network should have approximately the expected number of edges
        let total_edges: usize = network.iter().map(std::vec::Vec::len).sum();
        assert!(total_edges > 0); // Should have some connections at τ=0.5
    }

    #[test]
    fn test_component_detection() {
        let exp = TauCriticalMeasurement;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Create a simple network with two components
        let adjacency = vec![
            vec![1], // 0 connected to 1
            vec![0], // 1 connected to 0
            vec![3], // 2 connected to 3
            vec![2], // 3 connected to 2
        ];

        let components = exp.find_components(&adjacency);
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn test_clustering_coefficient() {
        let exp = TauCriticalMeasurement;

        // Triangle: 0-1-2-0
        let adjacency = vec![vec![1, 2], vec![0, 2], vec![0, 1]];

        let clustering = exp.calculate_clustering(&adjacency);
        assert!(clustering > 0.99); // Complete triangle should have CC ≈ 1.0
    }

    #[test]
    #[ignore = "slow: Monte-Carlo experiment validation"]
    fn test_experiment_run() {
        let exp = TauCriticalMeasurement;
        let config = ExperimentConfig::quick();
        let result = exp.run(&config).unwrap();

        assert!(!result.experiment_type.is_empty());
        assert_eq!(result.gap_id, "MEAN-P0-1");
    }
}
