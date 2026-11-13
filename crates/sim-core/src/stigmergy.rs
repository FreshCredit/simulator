//! Stigmergy Module - Virtual Pheromone Infrastructure
//!
//! Provides chemical signaling for indirect agent coordination

use crate::types::{Direction, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Number of pheromone types
pub const PHEROMONE_TYPE_COUNT: usize = 5;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Statistics for pheromone grid
///
/// Tracks aggregate metrics about pheromone concentrations
/// including total deposits, average concentration, and active cells.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PheromoneStats {
    /// Total deposits this tick
    pub total_deposits: u64,
    /// Average concentration across grid
    pub avg_concentration: f32,
    /// Maximum concentration found
    pub max_concentration: f32,
    /// Number of active cells (concentration > 0)
    pub active_cells: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of pheromones that can be deposited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PheromoneType {
    /// Resource location trails
    FoodTrail,
    /// Threat warnings
    DangerSignal,
    /// Mating fitness markers
    Reproduction,
    /// Group identification
    Territory,
    /// Cooperative task signaling
    Recruitment,
}

/// Configuration for pheromone dynamics
///
/// Controls how pheromones diffuse, evaporate, and deposit
/// in the stigmergy grid.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StigmergyConfig {
    /// How fast pheromones spread (0-1)
    pub diffusion_rate: f32,
    /// How fast pheromones decay (0-1)
    pub evaporation_rate: f32,
    /// Base amount deposited
    pub deposit_amount: f32,
    /// Cap to prevent overflow
    pub max_concentration: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for StigmergyConfig {
    fn default() -> Self {
        Self {
            diffusion_rate: 0.1,
            evaporation_rate: 0.05,
            deposit_amount: 1.0,
            max_concentration: 100.0,
        }
    }
}

/// Grid storing pheromone concentrations
///
/// 3D grid with separate layers for each pheromone type.
/// Supports indirect coordination through chemical signaling.
#[derive(Debug, Clone)]
pub struct PheromoneGrid {
    width: usize,
    height: usize,
    depth: usize,
    config: StigmergyConfig,
    /// One `Vec<f32>` per pheromone type
    layers: HashMap<PheromoneType, Vec<f32>>,
}

impl PheromoneGrid {
    /// Create new pheromone grid
    /// P1-FIX: Overflow-checked size calculation
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        let mut layers = HashMap::new();
        let effective_depth = depth.max(1);

        // P1-FIX: Checked multiplication to prevent integer overflow
        let layer_size = width
            .checked_mul(height)
            .and_then(|v| v.checked_mul(effective_depth))
            .unwrap_or_else(|| {
                debug_assert!(
                    false,
                    "PheromoneGrid size overflow: {width}x{height}x{effective_depth}"
                );
                // Fallback to safe maximum (64x64x1 = 4096)
                4096
            });

        for ptype in [
            PheromoneType::FoodTrail,
            PheromoneType::DangerSignal,
            PheromoneType::Reproduction,
            PheromoneType::Territory,
            PheromoneType::Recruitment,
        ] {
            layers.insert(ptype, vec![0.0; layer_size]);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Self {
            width,
            height,
            depth: effective_depth,
            config: StigmergyConfig::default(),
            layers,
        }
    }

    /// Get index for position
    /// P1-FIX: Bounds checked with saturating arithmetic
    fn index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        // Bounds check first
        if x >= self.width || y >= self.height || z >= self.depth {
            return None;
        }

        // P1-FIX: Use checked multiplication for index calculation
        let wh = self.width.checked_mul(self.height)?;
        let z_wh = z.checked_mul(wh)?;
        let y_w = y.checked_mul(self.width)?;
        z_wh.checked_add(y_w)?.checked_add(x)
    }

    /// Convert floating point position to grid indices
    fn position_to_indices(&self, pos: &Position) -> Option<(usize, usize, usize)> {
        if !pos.x.is_finite() || !pos.y.is_finite() || !pos.z.is_finite() {
            return None;
        }
        let x = pos.x.floor() as isize;
        let y = pos.y.floor() as isize;
        let z = pos.z.floor() as isize;

        // Check bounds
        if x < 0 || x >= self.width as isize {
            return None;
        }
        if y < 0 || y >= self.height as isize {
            return None;
        }
        if z < 0 || z >= self.depth as isize {
            return None;
        }

        Some((x as usize, y as usize, z as usize))
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Deposit pheromone at position
    /// P1-FIX: Uses bounds-checked index
    pub fn deposit(&mut self, pos: &Position, pheromone: PheromoneType, amount: f32) {
        let Some((x, y, z)) = self.position_to_indices(pos) else {
            return; // Position out of bounds
        };

        let Some(idx) = self.index(x, y, z) else {
            return; // Index calculation failed (overflow or bounds)
        };

        if let Some(layer) = self.layers.get_mut(&pheromone) {
            if let Some(current) = layer.get_mut(idx) {
                let new_amount = (*current + amount).min(self.config.max_concentration);
                *current = new_amount.max(0.0);
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Read concentration at position
    /// P1-FIX: Uses bounds-checked index
    #[must_use]
    pub fn concentration(&self, pos: &Position, pheromone: PheromoneType) -> f32 {
        let Some((x, y, z)) = self.position_to_indices(pos) else {
            return 0.0; // Position out of bounds
        };

        let Some(idx) = self.index(x, y, z) else {
            return 0.0; // Index calculation failed (overflow or bounds)
        };

        self.layers
            .get(&pheromone)
            .and_then(|layer| layer.get(idx))
            .copied()
            .unwrap_or(0.0)
    }

    /// Get grid dimensions
    #[must_use]
    pub const fn dimensions(&self) -> (usize, usize, usize) {
        (self.width, self.height, self.depth)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update step: apply diffusion and evaporation to all layers
    pub fn update(&mut self) {
        for ptype in [
            PheromoneType::FoodTrail,
            PheromoneType::DangerSignal,
            PheromoneType::Reproduction,
            PheromoneType::Territory,
            PheromoneType::Recruitment,
        ] {
            self.diffuse_layer(ptype);
            self.evaporate_layer(ptype);
        }
    }

    /// Diffuse a single layer using discrete Graph Laplacian (mass-conserving)
    /// P1-FIX: Uses bounds-checked index with overflow protection
    fn diffuse_layer(&mut self, ptype: PheromoneType) {
        let Some(layer) = self.layers.get(&ptype) else {
            return;
        };
        let mut new_layer = layer.clone();
        let rate = self.config.diffusion_rate;
        // Maximum degree in Moore neighborhood (8 for 2D, 26 for 3D)
        let max_degree = if self.depth > 1 { 26.0f32 } else { 8.0f32 };

        for z in 0..self.depth {
            for y in 0..self.height {
                for x in 0..self.width {
                    let Some(idx) = self.index(x, y, z) else {
                        continue; // Skip if index calculation fails
                    };
                    let current = layer[idx];

                    // Compute sum of neighbors (Moore neighborhood)
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    let mut neighbor_sum = 0.0;
                    let mut neighbor_count = 0;

                    for dz in -1..=1 {
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                if dx == 0 && dy == 0 && dz == 0 {
                                    continue;
                                }

                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                let nz = (z as i32 + dz) as usize;

                                if nx < self.width && ny < self.height && nz < self.depth {
                                    if let Some(n_idx) = self.index(nx, ny, nz) {
                                        neighbor_sum += layer[n_idx];
                                        neighbor_count += 1;
                                    }
                                }
                            }
                        }
                    }

                    if neighbor_count > 0 {
                        // Symmetric discrete Graph Laplacian with zero-flux boundaries:
                        // Δ_grid u(x) = ∑_{y ~ x} (u(y) - u(x))
                        // Conserves total mass across boundaries without asymmetric pull distortion
                        let laplacian = neighbor_sum - (neighbor_count as f32) * current;
                        new_layer[idx] = (current + (rate / max_degree) * laplacian)
                            .clamp(0.0, self.config.max_concentration);
                    }
                }
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        self.layers.insert(ptype, new_layer);
    }

    /// Apply evaporation to a layer
    fn evaporate_layer(&mut self, ptype: PheromoneType) {
        if let Some(layer) = self.layers.get_mut(&ptype) {
            let rate = self.config.evaporation_rate;
            for value in layer.iter_mut() {
                *value *= 1.0 - rate;
                // Ensure non-negative
                if *value < 0.001 {
                    *value = 0.0;
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get gradient direction at position (for chemotaxis)
    /// Sample concentrations in each direction and compute gradient
    /// pointing toward higher concentration
    #[must_use]
    pub fn gradient(&self, pos: &Position, pheromone: PheromoneType) -> Direction {
        if !pos.x.is_finite() || !pos.y.is_finite() || !pos.z.is_finite() {
            return Direction::new(0.0, 0.0, 0.0);
        }
        // Sample concentrations in each direction
        let right = self.concentration(&Position::new_2d(pos.x + 1.0, pos.y), pheromone);
        let left = self.concentration(&Position::new_2d(pos.x - 1.0, pos.y), pheromone);
        let down = self.concentration(&Position::new_2d(pos.x, pos.y + 1.0), pheromone);
        let up = self.concentration(&Position::new_2d(pos.x, pos.y - 1.0), pheromone);

        // Compute gradient (pointing toward higher concentration)
        let dx = (right - left) / 2.0;
        let dy = (down - up) / 2.0;

        Direction::new(dx, dy, 0.0).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pheromone_grid_creation() {
        let grid = PheromoneGrid::new(64, 64, 1);
        assert_eq!(grid.dimensions(), (64, 64, 1));
    }

    #[test]
    fn test_deposit_and_read() {
        let mut grid = PheromoneGrid::new(64, 64, 1);
        let pos = Position::new_2d(10.0, 10.0);

        grid.deposit(&pos, PheromoneType::FoodTrail, 5.0);
        let conc = grid.concentration(&pos, PheromoneType::FoodTrail);

        assert!(conc > 0.0);
        assert!((conc - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_deposit_out_of_bounds() {
        let mut grid = PheromoneGrid::new(64, 64, 1);
        let pos = Position::new_2d(100.0, 100.0); // Out of bounds

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Should not panic
        grid.deposit(&pos, PheromoneType::FoodTrail, 5.0);

        // Concentration should be 0 for out of bounds
        let conc = grid.concentration(&pos, PheromoneType::FoodTrail);
        assert_eq!(conc, 0.0);
    }

    #[test]
    fn test_concentration_out_of_bounds() {
        let grid = PheromoneGrid::new(64, 64, 1);
        let pos = Position::new_2d(-1.0, -1.0); // Out of bounds

        let conc = grid.concentration(&pos, PheromoneType::FoodTrail);
        assert_eq!(conc, 0.0);
    }

    #[test]
    fn test_max_concentration_cap() {
        let mut grid = PheromoneGrid::new(64, 64, 1);
        let pos = Position::new_2d(10.0, 10.0);

        // Deposit more than max_concentration
        grid.deposit(&pos, PheromoneType::FoodTrail, 150.0);

        let conc = grid.concentration(&pos, PheromoneType::FoodTrail);
        assert_eq!(conc, 100.0); // Should be capped at max_concentration
    }

    #[test]
    fn test_accumulate_deposits() {
        let mut grid = PheromoneGrid::new(64, 64, 1);
        let pos = Position::new_2d(10.0, 10.0);

        grid.deposit(&pos, PheromoneType::DangerSignal, 5.0);
        grid.deposit(&pos, PheromoneType::DangerSignal, 3.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let conc = grid.concentration(&pos, PheromoneType::DangerSignal);
        assert!((conc - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_different_pheromone_types_independent() {
        let mut grid = PheromoneGrid::new(64, 64, 1);
        let pos = Position::new_2d(10.0, 10.0);

        grid.deposit(&pos, PheromoneType::FoodTrail, 10.0);
        grid.deposit(&pos, PheromoneType::DangerSignal, 5.0);

        let food_conc = grid.concentration(&pos, PheromoneType::FoodTrail);
        let danger_conc = grid.concentration(&pos, PheromoneType::DangerSignal);

        assert_eq!(food_conc, 10.0);
        assert_eq!(danger_conc, 5.0);
    }

    #[test]
    fn test_3d_grid() {
        let mut grid = PheromoneGrid::new(32, 32, 8);
        assert_eq!(grid.dimensions(), (32, 32, 8));

        let pos = Position::new(10.0, 10.0, 5.0);
        grid.deposit(&pos, PheromoneType::Territory, 7.0);

        let conc = grid.concentration(&pos, PheromoneType::Territory);
        assert!((conc - 7.0).abs() < 0.001);
    }

    #[test]
    fn test_depth_zero_defaults_to_one() {
        let grid = PheromoneGrid::new(32, 32, 0);
        // Depth of 0 should default to 1
        assert_eq!(grid.dimensions(), (32, 32, 1));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_all_pheromone_types() {
        let mut grid = PheromoneGrid::new(64, 64, 1);
        let pos = Position::new_2d(20.0, 20.0);

        // Deposit all pheromone types
        grid.deposit(&pos, PheromoneType::FoodTrail, 1.0);
        grid.deposit(&pos, PheromoneType::DangerSignal, 2.0);
        grid.deposit(&pos, PheromoneType::Reproduction, 3.0);
        grid.deposit(&pos, PheromoneType::Territory, 4.0);
        grid.deposit(&pos, PheromoneType::Recruitment, 5.0);

        // Verify each type
        assert_eq!(grid.concentration(&pos, PheromoneType::FoodTrail), 1.0);
        assert_eq!(grid.concentration(&pos, PheromoneType::DangerSignal), 2.0);
        assert_eq!(grid.concentration(&pos, PheromoneType::Reproduction), 3.0);
        assert_eq!(grid.concentration(&pos, PheromoneType::Territory), 4.0);
        assert_eq!(grid.concentration(&pos, PheromoneType::Recruitment), 5.0);
    }

    #[test]
    fn test_gradient() {
        let mut grid = PheromoneGrid::new(64, 64, 1);

        // Create a gradient by depositing at different amounts
        for i in 0..10 {
            let pos = Position::new_2d(20.0 + i as f32, 20.0);
            grid.deposit(&pos, PheromoneType::FoodTrail, 10.0 - i as f32);
        }

        let test_pos = Position::new_2d(25.0, 20.0);
        let grad = grid.gradient(&test_pos, PheromoneType::FoodTrail);

        // Gradient should point toward higher concentration (negative x)
        assert!(grad.dx < 0.0);
    }

    #[test]
    fn test_pheromone_diffusion_conserves_total_mass_at_corners_and_boundaries() {
        let mut grid = PheromoneGrid::new(10, 10, 1);
        grid.config.evaporation_rate = 0.0; // Pure diffusion without evaporation
        grid.config.diffusion_rate = 0.2;

        // Deposit pheromones at corner (0, 0), edge (0, 5), and interior (5, 5)
        grid.deposit(&Position::new_2d(0.0, 0.0), PheromoneType::FoodTrail, 10.0);
        grid.deposit(&Position::new_2d(0.0, 5.0), PheromoneType::FoodTrail, 20.0);
        grid.deposit(&Position::new_2d(5.0, 5.0), PheromoneType::FoodTrail, 30.0);

        let mut initial_mass = 0.0f32;
        for y in 0..10 {
            for x in 0..10 {
                initial_mass += grid.concentration(
                    &Position::new_2d(x as f32, y as f32),
                    PheromoneType::FoodTrail,
                );
            }
        }
        assert!((initial_mass - 60.0).abs() < 1e-4);

        // Perform multiple discrete diffusion steps
        for _ in 0..15 {
            grid.diffuse_layer(PheromoneType::FoodTrail);
        }

        let mut final_mass = 0.0f32;
        for y in 0..10 {
            for x in 0..10 {
                final_mass += grid.concentration(
                    &Position::new_2d(x as f32, y as f32),
                    PheromoneType::FoodTrail,
                );
            }
        }

        // Mass must be 100% conserved within floating point error
        assert!(
            (final_mass - initial_mass).abs() < 1e-4,
            "Pheromone mass must be conserved under discrete Graph Laplacian: initial={initial_mass}, final={final_mass}"
        );
    }
}
