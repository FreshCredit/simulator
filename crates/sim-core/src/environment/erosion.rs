//! Geological Time-Scale Erosion
//!
//! Long-term terrain evolution for epoch-scale simulations including
//! fluvial erosion, hillslope diffusion, sediment transport, and tectonic uplift.

use serde::{Deserialize, Serialize};

// ============================================================================
// P2-7: GEOLOGICAL TIME-SCALE EROSION
// Long-term terrain evolution for epoch-scale simulations
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Erosion rates for different processes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ErosionRates {
    /// Fluvial (water) erosion coefficient
    pub fluvial: f32,
    /// Hillslope erosion coefficient
    pub hillslope: f32,
    /// Chemical weathering rate
    pub weathering: f32,
    /// Tectonic uplift rate (if any)
    pub uplift: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ErosionRates {
    fn default() -> Self {
        Self {
            fluvial: 0.01,
            hillslope: 0.005,
            weathering: 0.001,
            uplift: 0.0,
        }
    }
}

impl ErosionRates {
    /// Create rates for arid climate (low erosion)
    #[must_use]
    pub const fn arid() -> Self {
        Self {
            fluvial: 0.001,
            hillslope: 0.01, // More mechanical weathering
            weathering: 0.0005,
            uplift: 0.0,
        }
    }

    /// Create rates for tropical climate (high erosion)
    #[must_use]
    pub const fn tropical() -> Self {
        Self {
            fluvial: 0.02,
            hillslope: 0.008,
            weathering: 0.002,
            uplift: 0.0,
        }
    }

    /// Create rates for alpine/tectonically active
    #[must_use]
    pub const fn alpine() -> Self {
        Self {
            fluvial: 0.015,
            hillslope: 0.015,
            weathering: 0.001,
            uplift: 0.01, // Significant uplift
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Total erosion rate given slope and drainage area
    #[must_use]
    pub fn total_erosion(&self, slope: f32, drainage_area: f32) -> f32 {
        // Stream power law: E = K * A^m * S^n
        let m = 0.5; // Area exponent
        let n = 1.0; // Slope exponent

        self.fluvial * drainage_area.max(0.0).powf(m) * slope.max(0.0).powf(n)
            + self.hillslope * slope
            + self.weathering
    }
}

/// Geological erosion model for long-term terrain evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeologicalErosionModel {
    /// Grid width
    pub width: usize,
    /// Grid height
    pub height: usize,
    /// Elevation at each cell
    pub elevation: Vec<Vec<f32>>,
    /// Sediment thickness at each cell
    pub sediment: Vec<Vec<f32>>,
    /// Erosion rates
    pub rates: ErosionRates,
    /// Simulation time (years)
    pub time: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl GeologicalErosionModel {
    /// Create new erosion model
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let elevation = vec![vec![0.0; height]; width];
        let sediment = vec![vec![0.0; height]; width];

        Self {
            width,
            height,
            elevation,
            sediment,
            rates: ErosionRates::default(),
            time: 0.0,
        }
    }

    /// Create with initial terrain
    #[must_use]
    pub fn with_terrain(elevation: Vec<Vec<f32>>) -> Self {
        let width = elevation.len();
        let height = elevation.first().map_or(0, std::vec::Vec::len);
        let sediment = vec![vec![0.0; height]; width];

        Self {
            width,
            height,
            elevation,
            sediment,
            rates: ErosionRates::default(),
            time: 0.0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// One step of fluvial erosion
    #[allow(clippy::needless_range_loop)]
    pub fn fluvial_erosion_step(&mut self, dt: f32) {
        let mut new_elevation = self.elevation.clone();

        for x in 1..self.width - 1 {
            for y in 1..self.height - 1 {
                let slope = self.local_slope(x, y);
                let drainage = self.drainage_area(x, y);

                let erosion = self.rates.total_erosion(slope, drainage) * dt;
                let actual_erosion = self.elevation[x][y].min(erosion).max(0.0);
                new_elevation[x][y] = (self.elevation[x][y] - actual_erosion).max(0.0);

                // Add to sediment from actual eroded bedrock only
                self.sediment[x][y] += actual_erosion * 0.5;
            }
        }

        self.elevation = new_elevation;
        self.time += dt;
    }

    /// Hillslope diffusion (smoothing)
    #[allow(clippy::needless_range_loop)]
    pub fn hillslope_erosion_step(&mut self, dt: f32) {
        let mut new_elevation = self.elevation.clone();

        for x in 1..self.width - 1 {
            for y in 1..self.height - 1 {
                let neighbors = [
                    self.elevation[x - 1][y],
                    self.elevation[x + 1][y],
                    self.elevation[x][y - 1],
                    self.elevation[x][y + 1],
                ];
                let avg_neighbor: f32 = neighbors.iter().sum::<f32>() / 4.0;
                let diff = avg_neighbor - self.elevation[x][y];

                let erosion = self.rates.hillslope * diff * dt;
                new_elevation[x][y] += erosion;
            }
        }

        self.elevation = new_elevation;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Transport sediment downstream
    pub fn transport_sediment(&mut self, dt: f32) {
        let mut new_sediment = vec![vec![0.0f32; self.height]; self.width];
        let transport_rate = (0.5 * dt).clamp(0.0, 1.0);

        for x in 0..self.width {
            for y in 0..self.height {
                if self.sediment[x][y] > 0.0 {
                    // Find steepest descent
                    if let Some((nx, ny)) = self.steepest_descent(x, y) {
                        // Transport sediment downstream with CFL stability clamping
                        let transported = self.sediment[x][y] * transport_rate;

                        if nx < self.width && ny < self.height {
                            new_sediment[nx][ny] += transported;
                            new_sediment[x][y] += self.sediment[x][y] - transported;
                        } else {
                            new_sediment[x][y] += self.sediment[x][y];
                        }
                    } else {
                        new_sediment[x][y] += self.sediment[x][y];
                    }
                }
            }
        }

        self.sediment = new_sediment;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply tectonic uplift
    pub fn apply_uplift(&mut self, dt: f32) {
        if self.rates.uplift > 0.0 {
            for x in 0..self.width {
                for y in 0..self.height {
                    // More uplift in center (simplified)
                    let dx = x as f32 - self.width as f32 / 2.0;
                    let dy = y as f32 - self.height as f32 / 2.0;
                    let dist = dx.hypot(dy);
                    let uplift_factor = (-dist / 10.0).exp();

                    self.elevation[x][y] += self.rates.uplift * uplift_factor * dt;
                }
            }
        }
    }

    /// Run erosion for specified number of steps
    pub fn erode(&mut self, steps: u32, dt: f32) {
        for _ in 0..steps {
            self.fluvial_erosion_step(dt);
            self.hillslope_erosion_step(dt);
            self.transport_sediment(dt);
            self.apply_uplift(dt);
        }
    }

    /// Calculate local slope at a cell
    fn local_slope(&self, x: usize, y: usize) -> f32 {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let e = self.elevation[x][y];

        // Check 8 neighbors
        let mut max_slope = 0.0f32;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    let neighbor_e = self.elevation[nx as usize][ny as usize];
                    let slope = (e - neighbor_e).abs();
                    max_slope = max_slope.max(slope);
                }
            }
        }

        max_slope
    }

    /// Calculate drainage area (simplified - just count uphill cells)
    fn drainage_area(&self, x: usize, y: usize) -> f32 {
        // Simplified: drainage proportional to elevation
        // In reality, this requires flow accumulation algorithm
        (self.elevation[x][y] / 100.0).max(1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Find steepest descent direction
    #[must_use]
    fn steepest_descent(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        let e = self.elevation[x][y];
        let mut min_e = e;
        let mut min_pos = None;

        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    let neighbor_e = self.elevation[nx as usize][ny as usize];
                    if neighbor_e < min_e {
                        min_e = neighbor_e;
                        min_pos = Some((nx as usize, ny as usize));
                    }
                }
            }
        }

        min_pos
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get terrain statistics
    #[must_use]
    pub fn statistics(&self) -> ErosionStatistics {
        let all_elev: Vec<f32> = self
            .elevation
            .iter()
            .flat_map(|r| r.iter())
            .copied()
            .collect();

        let min_e = all_elev.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_e = all_elev.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let mean_e = all_elev.iter().sum::<f32>() / all_elev.len() as f32;

        let variance =
            all_elev.iter().map(|&e| (e - mean_e).powi(2)).sum::<f32>() / all_elev.len() as f32;

        ErosionStatistics {
            min_elevation: min_e,
            max_elevation: max_e,
            mean_elevation: mean_e,
            std_elevation: variance.sqrt(),
            total_sediment: self.sediment.iter().flat_map(|r| r.iter()).sum(),
            simulation_time: self.time,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Statistics from erosion simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErosionStatistics {
    /// Minimum elevation
    pub min_elevation: f32,
    /// Maximum elevation
    pub max_elevation: f32,
    /// Mean elevation
    pub mean_elevation: f32,
    /// Standard deviation of elevation
    pub std_elevation: f32,
    /// Total sediment volume
    pub total_sediment: f32,
    /// Simulation time (years)
    pub simulation_time: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // P2-7: Geological Time-Scale Erosion Tests

    #[test]
    fn test_geological_erosion_model_creation() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let model = GeologicalErosionModel::new(100, 100);
        assert_eq!(model.width, 100);
        assert_eq!(model.height, 100);
        assert_eq!(model.elevation.len(), 100);
    }

    #[test]
    fn test_fluvial_erosion_step() {
        let mut model = GeologicalErosionModel::new(10, 10);

        // Create some terrain variation
        model.elevation[5][5] = 10.0; // Peak

        let initial_peak = model.elevation[5][5];
        model.fluvial_erosion_step(0.1);

        // Peak should have eroded slightly
        assert!(model.elevation[5][5] <= initial_peak);
    }

    #[test]
    fn test_hillslope_erosion() {
        let mut model = GeologicalErosionModel::new(10, 10);

        // Create a slope
        model.elevation[0][0] = 10.0;
        model.elevation[1][1] = 5.0;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        model.hillslope_erosion_step(0.1);

        // Should have some erosion
        let total_elevation: f32 = model.elevation.iter().flat_map(|r| r.iter()).sum();
        assert!(total_elevation > 0.0);
    }

    #[test]
    fn test_erosion_rate_calculation() {
        let rate = ErosionRates::default();

        assert!(rate.fluvial > 0.0);
        assert!(rate.hillslope > 0.0);
        assert!(rate.weathering > 0.0);

        // Erosion should increase with slope
        let flat_erosion = rate.total_erosion(0.0, 1.0);
        let steep_erosion = rate.total_erosion(1.0, 1.0);
        assert!(steep_erosion > flat_erosion);
    }

    #[test]
    fn test_sediment_transport() {
        let mut model = GeologicalErosionModel::new(10, 10);

        // Add sediment at one location
        model.sediment[5][5] = 1.0;

        let initial_sediment = model.sediment[5][5];
        model.transport_sediment(0.1);

        // Sediment should have moved
        let total_sediment: f32 = model.sediment.iter().flat_map(|r| r.iter()).sum();
        assert!((total_sediment - initial_sediment).abs() < 0.01);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_erosion_statistics() {
        let mut model = GeologicalErosionModel::new(10, 10);
        model.elevation[5][5] = 20.0; // High peak

        let stats = model.statistics();
        assert!(stats.max_elevation >= 20.0);
        assert!(stats.min_elevation <= 20.0);
    }

    #[test]
    fn test_total_eroded_material() {
        let mut model = GeologicalErosionModel::new(10, 10);

        // Create varied terrain
        for i in 0..10 {
            for j in 0..10 {
                model.elevation[i][j] = (i as f32 * j as f32) % 10.0;
            }
        }

        let initial_total: f32 = model.elevation.iter().flat_map(|r| r.iter()).sum();

        model.erode(100, 0.01);

        let final_total: f32 = model.elevation.iter().flat_map(|r| r.iter()).sum();

        // Some material should have eroded (but not all)
        assert!(final_total <= initial_total);
    }

    #[test]
    fn test_erosion_zero_elevation_no_phantom_sediment_and_cfl_stability() {
        let mut model = GeologicalErosionModel::new(10, 10);
        // Sea level flat terrain: elevation is 0.0 everywhere
        model.fluvial_erosion_step(1.0);

        let total_sed: f32 = model.sediment.iter().flat_map(|r| r.iter()).sum();
        assert_eq!(
            total_sed, 0.0,
            "Zero-elevation flat terrain cannot produce phantom sediment from non-existent bedrock"
        );

        // CFL stability test: large geological time step (dt = 100.0)
        model.elevation[5][5] = 50.0;
        model.sediment[5][5] = 10.0;
        model.transport_sediment(100.0);

        for row in &model.sediment {
            for &val in row {
                assert!(
                    val >= 0.0,
                    "Sediment must never become negative under CFL transport clamping: got {val}"
                );
            }
        }
    }
}
