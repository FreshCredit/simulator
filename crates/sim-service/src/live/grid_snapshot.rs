// TAG: surface=simulation owner=research-team rule=SIM-001
//! Environment grid, resource, weather, and terrain snapshot builders

use super::*;
use sim_core::Simulation;

/// Grid snapshot module version identifier
#[inline]
#[must_use]
pub const fn grid_snapshot_version() -> &'static str {
    "0.1.0"
}

/// Build resource snapshot vectors for simulation frame
pub fn build_resources(sim: &Simulation) -> Option<Vec<ResourceSnapshot>> {
    if sim.tick > 2 && sim.tick % 10 != 0 {
        return None;
    }
    Some(
        sim.environment
            .resources
            .iter()
            .map(|r| ResourceSnapshot {
                amount: r.amount,
                capacity: r.capacity,
            })
            .collect(),
    )
}

/// Build weather grid intensity values for simulation frame
pub fn build_weather_grid(sim: &Simulation) -> Option<Vec<f32>> {
    if !sim.environment.weather_enabled || (sim.tick > 2 && sim.tick % 10 != 0) {
        return None;
    }
    let width = sim.environment.width;
    let height = sim.environment.height;
    let base = match sim.environment.weather.current_weather {
        sim_core::environment::WeatherType::Sunny => 0.1,
        sim_core::environment::WeatherType::Cloudy => 0.3,
        sim_core::environment::WeatherType::Rainy => 0.5,
        sim_core::environment::WeatherType::Stormy => 0.7,
        sim_core::environment::WeatherType::Drought => 0.9,
    };
    let f_thermal = sim.environment.weather.thermal_stress_factor();
    let mut grid = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let rain_gradient = sim.environment.weather.rainfall_at(x, y);
            let val =
                (base * 0.5 + rain_gradient * 0.3 + (f_thermal - 1.0) * 0.2).clamp(0.0f32, 1.0f32);
            grid.push(val);
        }
    }
    Some(grid)
}

/// Build terrain elevation grid for simulation frame
pub fn build_terrain_grid(sim: &Simulation) -> Option<Vec<f32>> {
    if !sim.environment.altitude_effects_enabled || (sim.tick > 2 && sim.tick % 100 != 0) {
        return None;
    }
    let width = sim.environment.width;
    let height = sim.environment.height;
    let mut grid = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let cell_idx = y * width + x;
            let elev = if sim.environment.depth > 1 {
                let mut max_z = 0.0f32;
                for z in 0..sim.environment.depth {
                    let idx = z * (width * height) + cell_idx;
                    if idx < sim.environment.resources.len()
                        && sim.environment.resources[idx].amount > 0.0
                    {
                        max_z = (z as f32) / (sim.environment.depth.max(1) as f32);
                    }
                }
                max_z
            } else {
                let cell = sim.environment.resources.get(cell_idx);
                let cap = cell.map_or(0.0, |c| c.capacity);
                (cap / 30.0).clamp(0.0, 1.0)
            };
            grid.push(elev);
        }
    }
    Some(grid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_resources_initial_tick() {
        let sim = sim_core::Simulation::new(sim_core::simulation::SimulationConfig::default());
        let res = build_resources(&sim);
        assert!(res.is_some());
    }

    #[test]
    fn test_build_weather_grid_disabled() {
        let mut config = sim_core::simulation::SimulationConfig::default();
        config.environment.weather_enabled = false;
        let sim = sim_core::Simulation::new(config);
        let weather = build_weather_grid(&sim);
        assert!(weather.is_none());
    }
}
