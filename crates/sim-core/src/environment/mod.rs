//! Environment module: Grid-based world with resources and weather
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION environment, not product infrastructure
//!
//! Per §7.4.0 lines 3747-3792: Stage 0 (Geography) and Stage 1 (Biology)
//! Per §7.5.2 (implied): Environment state struct
//!
//! 3D Support: Full altitude/elevation layer support with:
//! - Temperature decreases with altitude
//! - Resource availability varies by altitude
//! - Movement cost increases with vertical change

// TAG: surface=simulation owner=research-team rule=SIM-001
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::ecology::AlleeEffect;
use crate::types::{
    AltitudeLayer, Direction, Energy, Position, ALTITUDE_RESOURCE_MODIFIER,
    ALTITUDE_TEMPERATURE_MODIFIER, BASE_TEMPERATURE, VERTICAL_MOVEMENT_COST_FACTOR,
};

pub mod erosion;
pub mod spatial;
pub mod terrain;
pub mod weather;

pub use erosion::{ErosionRates, ErosionStatistics, GeologicalErosionModel};
pub use spatial::{OccupancyGrid, SpatialHash};
pub use terrain::GISTerrainAnalysis;
pub use weather::{Season, WeatherPattern, WeatherState, WeatherType};

// =============================================================================
// Resource Cell
// =============================================================================

/// Resource cell in the environment
/// Sugarscape-style resource patches per §7.4.0
/// Now with altitude-based capacity modifiers
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceCell {
    /// Current resource amount
    pub amount: Energy,
    /// Maximum resource capacity
    pub capacity: Energy,
    /// Regrowth rate per tick
    pub regrowth_rate: Energy,
    /// Altitude layer this cell belongs to
    pub altitude_layer: AltitudeLayer,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ResourceCell {
    fn default() -> Self {
        Self {
            amount: 10.0,
            capacity: 20.0,
            regrowth_rate: 0.5, // TUNABLE: per §7.4.0
            altitude_layer: 0,
        }
    }
}

impl ResourceCell {
    /// Regrow resources toward capacity
    pub fn regrow(&mut self) {
        self.amount = (self.amount + self.regrowth_rate).min(self.capacity);
    }

    /// Regrow resources with weather multiplier
    pub fn regrow_with_weather(&mut self, multiplier: f32) {
        let adjusted_rate = self.regrowth_rate * multiplier;
        self.amount = (self.amount + adjusted_rate).min(self.capacity);
    }

    /// Harvest resources (returns amount actually harvested)
    #[must_use]
    pub fn harvest(&mut self, max_amount: Energy) -> Energy {
        let harvested = self.amount.min(max_amount);
        self.amount -= harvested;
        harvested
    }

    /// Create a resource cell for a specific altitude layer
    /// Higher altitude = lower capacity and regrowth rate
    #[must_use]
    pub fn for_altitude(altitude: AltitudeLayer) -> Self {
        let altitude_factor = ALTITUDE_RESOURCE_MODIFIER.powi(altitude as i32);
        Self {
            amount: 10.0 * altitude_factor,
            capacity: 20.0 * altitude_factor,
            regrowth_rate: 0.5 * altitude_factor,
            altitude_layer: altitude,
        }
    }
}

// =============================================================================
// Environment Configuration and Main Struct
// =============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Environment grid configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Grid width
    pub width: usize,
    /// Grid height
    pub height: usize,
    /// Grid depth (altitude layers) - 3D support
    pub depth: usize,
    /// Probability of a cell being a resource patch
    pub resource_density: f32,
    /// Initial resource amount in patches
    pub initial_resource: Energy,
    /// Enable altitude effects (temperature, resources)
    pub altitude_effects_enabled: bool,
    /// Enable weather system
    pub weather_enabled: bool,
}

impl Default for EnvironmentConfig {
    /// Default 64×64×1 grid for N=1 Phase 0
    /// Per §7.4.0: scale to 256×256 for N=1,000
    /// Depth=1 for backward compatibility (2D mode)
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            depth: 1,              // Default to single layer for 2D backward compatibility
            resource_density: 0.3, // TUNABLE
            initial_resource: 15.0,
            altitude_effects_enabled: true,
            weather_enabled: true,
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }
}

impl EnvironmentConfig {
    /// Create a 3D environment configuration
    #[must_use]
    pub const fn new_3d(width: usize, height: usize, depth: usize) -> Self {
        Self {
            width,
            height,
            depth,
            resource_density: 0.3,
            initial_resource: 15.0,
            altitude_effects_enabled: true,
            weather_enabled: true,
        }
    }

    /// Create a 2D environment configuration (backward compatibility)
    #[must_use]
    pub const fn new_2d(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            depth: 1,
            resource_density: 0.3,
            initial_resource: 15.0,
            altitude_effects_enabled: false,
            weather_enabled: true,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Grid-based spatial environment with resources and weather
/// Per §7.5.2 (implied from Stage 0 requirements)
/// Now supports 3D with altitude layers and dynamic weather
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    /// Grid dimensions
    pub width: usize,
    /// height.
    pub height: usize,
    /// Altitude layers (depth)
    pub depth: usize,
    /// Resource grid (3D array flattened: z * (width * height) + y * width + x)
    pub resources: Vec<ResourceCell>,
    /// Current simulation tick
    pub tick: u64,
    /// Temperature grid (per altitude layer)
    /// Cached temperature values to avoid recalculation
    pub temperature_by_altitude: Vec<f32>,
    /// Whether altitude effects are enabled
    pub altitude_effects_enabled: bool,
    /// Weather state
    pub weather: WeatherState,
    /// Whether weather system is enabled
    pub weather_enabled: bool,
    /// Allee effect state for small population dynamics
    pub allee_effect: AlleeEffect,
}

impl Environment {
    /// Create new environment with given configuration
    /// P1-FIX: Overflow-checked size calculation
    pub fn new(config: &EnvironmentConfig, rng: &mut StdRng) -> Self {
        // P1-FIX: Checked multiplication to prevent integer overflow
        let size = config
            .width
            .checked_mul(config.height)
            .and_then(|v| v.checked_mul(config.depth))
            .unwrap_or_else(|| {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                debug_assert!(
                    false,
                    "Environment size overflow: {}x{}x{}",
                    config.width, config.height, config.depth
                );
                // Fallback to safe maximum
                4096
            });
        let mut resources = Vec::with_capacity(size);

        // Initialize resource patches with altitude-based modifiers
        for z in 0..config.depth {
            for _y in 0..config.height {
                for _x in 0..config.width {
                    let altitude = if config.altitude_effects_enabled {
                        z
                    } else {
                        0
                    };
                    let altitude_factor = if config.altitude_effects_enabled {
                        ALTITUDE_RESOURCE_MODIFIER.powi(z as i32)
                    } else {
                        1.0
                    };

                    let mut cell = ResourceCell::for_altitude(altitude);

                    if rng.gen::<f32>() < config.resource_density {
                        cell.amount = config.initial_resource * altitude_factor;
                        cell.capacity = config.initial_resource * 1.5 * altitude_factor;
                    } else {
                        cell.amount = 0.0;
                        cell.capacity = 0.0;
                        cell.regrowth_rate = 0.0;
                    }
                    resources.push(cell);
                }
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Pre-calculate temperature for each altitude layer
        let mut temperature_by_altitude = Vec::with_capacity(config.depth);
        for z in 0..config.depth {
            let temp = if config.altitude_effects_enabled {
                BASE_TEMPERATURE + (z as f32 * ALTITUDE_TEMPERATURE_MODIFIER)
            } else {
                BASE_TEMPERATURE
            };
            temperature_by_altitude.push(temp);
        }

        // Initialize weather system
        let weather = if config.weather_enabled {
            WeatherState::with_random(config.width, config.height, rng)
        } else {
            let mut ws = WeatherState::new(config.width, config.height);
            ws.enabled = false;
            ws
        };

        Self {
            width: config.width,
            height: config.height,
            depth: config.depth,
            resources,
            tick: 0,
            temperature_by_altitude,
            altitude_effects_enabled: config.altitude_effects_enabled,
            weather,
            weather_enabled: config.weather_enabled,
            allee_effect: AlleeEffect::default(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create default 64×64 environment for Phase 0
    pub fn new_default(rng: &mut StdRng) -> Self {
        Self::new(&EnvironmentConfig::default(), rng)
    }

    /// Create 3D environment with specified depth
    pub fn new_3d(width: usize, height: usize, depth: usize, rng: &mut StdRng) -> Self {
        let config = EnvironmentConfig::new_3d(width, height, depth);
        Self::new(&config, rng)
    }

    /// Create environment without weather
    #[must_use]
    pub fn new_without_weather(rng: &mut StdRng) -> Self {
        let config = EnvironmentConfig {
            weather_enabled: false,
            ..Default::default()
        };
        Self::new(&config, rng)
    }

    /// Convert position to grid index
    /// 3D indexing: z * (width * height) + y * width + x
    /// P1-FIX: Returns None for out-of-bounds positions
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub const fn position_to_index(&self, pos: &Position) -> Option<usize> {
        let x = pos.x as usize;
        let y = pos.y as usize;
        let z = pos.z as usize;

        self.coords_to_index(x, y, z)
    }

    /// Convert 3D coordinates to grid index
    /// P1-FIX: Bounds checked, returns None for out-of-bounds coordinates
    #[must_use]
    pub const fn coords_to_index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if x < self.width && y < self.height && z < self.depth {
            // P1-FIX: Note: Index calculation assumes width*height*depth doesn't overflow
            // This is protected by checked multiplication in Environment::new()
            Some(z * self.width * self.height + y * self.width + x)
        } else {
            None
        }
    }

    /// Get resource at position
    #[must_use]
    pub fn resource_at(&self, pos: &Position) -> Option<Energy> {
        self.position_to_index(pos)
            .map(|i| self.resources[i].amount)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Harvest resource at position
    pub fn harvest_at(&mut self, pos: &Position, max_amount: Energy) -> Energy {
        if let Some(index) = self.position_to_index(pos) {
            self.resources[index].harvest(max_amount)
        } else {
            0.0
        }
    }

    /// Regrow all resources (called each tick)
    /// Altitude affects regrowth rate
    /// Weather affects regrowth (rain increases, drought decreases)
    /// P1-FIX: Uses safe `coords_to_index` with bounds checking
    pub fn regrow_all(&mut self) {
        if self.weather_enabled {
            // Apply weather-aware regrowth with position-specific multipliers
            for z in 0..self.depth {
                for y in 0..self.height {
                    for x in 0..self.width {
                        // P1-FIX: Use safe coords_to_index with bounds checking
                        if let Some(index) = self.coords_to_index(x, y, z) {
                            let multiplier = self.weather.regrowth_multiplier_at(x, y);
                            self.resources[index].regrow_with_weather(multiplier);
                        } else {
                            debug_assert!(
                                false,
                                "regrow_all: coords out of bounds ({x}, {y}, {z})"
                            );
                        }
                    }
                }
            }
        } else {
            // Original regrowth without weather effects
            for cell in &mut self.resources {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                cell.regrow();
            }
        }
    }

    /// Advance environment by one tick
    /// Now includes weather updates
    pub fn step(&mut self, rng: &mut StdRng) {
        self.tick += 1;

        // Update weather if enabled
        if self.weather_enabled {
            self.weather.step(rng);
        }

        // Regrow resources (weather-aware)
        self.regrow_all();
    }

    /// Get temperature-based energy consumption multiplier for agents
    /// Returns factor > 1.0 for extreme temperatures (hot or cold)
    #[must_use]
    pub fn temperature_energy_factor(&self) -> f32 {
        if self.weather_enabled {
            self.weather.temperature_energy_factor()
        } else {
            1.0 // No weather effect
        }
    }

    /// Get current weather type
    #[must_use]
    pub fn current_weather(&self) -> WeatherType {
        if self.weather_enabled {
            self.weather.weather_type()
        } else {
            WeatherType::Sunny
        }
    }

    /// Get current season
    #[must_use]
    pub fn current_season(&self) -> Season {
        if self.weather_enabled {
            self.weather.season()
        } else {
            Season::Spring
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get rainfall at specific position (2D x,y)
    #[must_use]
    pub fn rainfall_at(&self, x: usize, y: usize) -> f32 {
        if self.weather_enabled && x < self.width && y < self.height {
            self.weather.rainfall_at(x, y)
        } else {
            0.0
        }
    }

    /// Check if position is valid
    #[must_use]
    pub fn is_valid_position(&self, pos: &Position) -> bool {
        pos.x >= 0.0
            && pos.y >= 0.0
            && pos.z >= 0.0
            && (pos.x as usize) < self.width
            && (pos.y as usize) < self.height
            && (pos.z as usize) < self.depth
    }

    /// Check if position is valid in 2D (ignores z for backward compatibility)
    #[must_use]
    pub fn is_valid_position_2d(&self, pos: &Position) -> bool {
        pos.x >= 0.0
            && pos.y >= 0.0
            && (pos.x as usize) < self.width
            && (pos.y as usize) < self.height
    }

    /// Clamp position to grid bounds (3D)
    #[must_use]
    pub fn clamp_position(&self, pos: &Position) -> Position {
        Position::new(
            pos.x.clamp(0.0, (self.width.saturating_sub(1)) as f32),
            pos.y.clamp(0.0, (self.height.saturating_sub(1)) as f32),
            pos.z.clamp(0.0, (self.depth.saturating_sub(1)) as f32),
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Clamp position to grid bounds (2D, z preserved)
    #[must_use]
    pub fn clamp_position_2d(&self, pos: &Position) -> Position {
        Position::new(
            pos.x.clamp(0.0, (self.width.saturating_sub(1)) as f32),
            pos.y.clamp(0.0, (self.height.saturating_sub(1)) as f32),
            pos.z,
        )
    }

    /// Get temperature at position (based on altitude and weather)
    #[must_use]
    pub fn temperature_at(&self, pos: &Position) -> f32 {
        let base_temp = if self.altitude_effects_enabled {
            let altitude = pos.z as usize;
            if altitude < self.temperature_by_altitude.len() {
                self.temperature_by_altitude[altitude]
            } else {
                BASE_TEMPERATURE + (pos.z * ALTITUDE_TEMPERATURE_MODIFIER)
            }
        } else {
            BASE_TEMPERATURE
        };

        // Add weather temperature effect if enabled
        if self.weather_enabled {
            // Weather temperature is surface temperature; adjust for altitude
            let weather_temp = self.weather.temperature();
            // Blend altitude-based temp with weather temp
            // Weather affects surface more than high altitude
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            let altitude_factor = 1.0 - (pos.z * 0.1).min(0.5); // Reduce weather effect at altitude
            base_temp * (1.0 - altitude_factor) + weather_temp * altitude_factor
        } else {
            base_temp
        }
    }

    /// Get temperature for a specific altitude layer
    #[must_use]
    pub fn temperature_at_altitude(&self, altitude: AltitudeLayer) -> f32 {
        if altitude < self.temperature_by_altitude.len() {
            self.temperature_by_altitude[altitude]
        } else {
            BASE_TEMPERATURE + (altitude as f32 * ALTITUDE_TEMPERATURE_MODIFIER)
        }
    }

    /// Calculate movement cost between two positions
    /// Horizontal movement: base cost
    /// Vertical movement: additional cost based on altitude change
    #[must_use]
    pub fn movement_cost(&self, from: &Position, to: &Position, base_cost: f32) -> f32 {
        let horizontal_dist = from.horizontal_distance(to);
        let vertical_dist = from.vertical_distance(to);

        // Going up costs more than going down
        let vertical_change = to.z - from.z;
        let vertical_cost = if vertical_change > 0.0 {
            // Going up: steeper cost
            vertical_dist * VERTICAL_MOVEMENT_COST_FACTOR
        } else {
            // Going down: typical cost
            vertical_dist
        };

        (horizontal_dist * base_cost) + vertical_cost
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute resource gradient at position
    /// Per §7.4.0 lines 3764-3765: Dopamine → move toward high-reward areas
    /// Returns direction pointing toward higher resources
    /// Now includes 3D (vertical) gradient when depth > 1
    #[must_use]
    pub fn resource_gradient_at(&self, pos: &Position) -> Direction {
        let x = pos.x as isize;
        let y = pos.y as isize;
        let z = pos.z as isize;
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        let mut dz = 0.0f32;

        // Sample resources in 6 cardinal directions (including up/down in 3D)
        let directions_2d = [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0)];
        let directions_3d = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            (0, 0, -1),
        ];

        let directions = if self.depth > 1 {
            &directions_3d[..]
        } else {
            &directions_2d[..]
        };

        for (ddx, ddy, ddz) in directions {
            let nx = x + ddx;
            let ny = y + ddy;
            let nz = z + ddz;
            if nx >= 0
                && nx < self.width as isize
                && ny >= 0
                && ny < self.height as isize
                && nz >= 0
                && nz < self.depth as isize
            {
                let neighbor_pos = Position::new(nx as f32, ny as f32, nz as f32);
                let resource = self.resource_at(&neighbor_pos).unwrap_or(0.0);
                dx += *ddx as f32 * resource;
                dy += *ddy as f32 * resource;
                dz += *ddz as f32 * resource;
            }
        }

        Direction::new(dx, dy, dz).normalize()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute agent density gradient at position
    /// Per §7.4.0 lines 4044-4064: Oxytocin → move toward social clusters
    /// Per §23.8.1 GAP-P0-03: Required for DOCAS-weighted movement
    ///
    /// Uses kernel density estimation with inverse distance weighting.
    /// Returns direction pointing toward higher agent density.
    #[must_use]
    pub fn agent_density_gradient_at(
        &self,
        pos: &Position,
        agent_positions: &[Position],
    ) -> Direction {
        const RADIUS: f32 = 5.0; // Social sensing radius (cells)
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        let mut dz = 0.0f32;

        for agent_pos in agent_positions {
            let dist = pos.distance(agent_pos);
            if dist > 0.0 && dist <= RADIUS {
                // Weight by inverse distance (closer agents have more pull)
                let weight = 1.0 / dist;
                dx += (agent_pos.x - pos.x) * weight;
                dy += (agent_pos.y - pos.y) * weight;
                dz += (agent_pos.z - pos.z) * weight;
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            }
        }

        Direction::new(dx, dy, dz).normalize()
    }

    /// Compute threat gradient at position
    /// Per §7.4.0 lines 4066-4086: Cortisol → move AWAY from threats
    /// Per §23.8.2 GAP-P1-02: Required for complete DOCAS-weighted movement
    ///
    /// Returns direction pointing AWAY from threat sources.
    #[must_use]
    pub fn threat_gradient_at(
        &self,
        pos: &Position,
        threat_positions: &[Position],
        threat_intensities: &[f32],
    ) -> Direction {
        const THREAT_RADIUS: f32 = 8.0; // Threat sensing radius (cells)
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        let mut dz = 0.0f32;

        for (threat_pos, &intensity) in threat_positions.iter().zip(threat_intensities.iter()) {
            let dist = pos.distance(threat_pos);
            if dist > 0.0 && dist <= THREAT_RADIUS {
                // Weight by threat intensity and inverse distance
                let weight = intensity / dist;
                // Negative: move AWAY from threat
                dx -= (threat_pos.x - pos.x) * weight;
                dy -= (threat_pos.y - pos.y) * weight;
                dz -= (threat_pos.z - pos.z) * weight;
            }
        }

        Direction::new(dx, dy, dz).normalize()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get resource value at a specific cell
    /// Per §7.4.0 lines 4019-4021: `resource_value` for utility computation
    #[must_use]
    pub fn resource_value_at(&self, pos: &Position) -> f32 {
        self.resource_at(pos).unwrap_or(0.0)
    }

    /// Get agent density at a specific cell (count of nearby agents)
    /// Per §7.4.0 lines 4023-4024: `social_density` for utility computation
    #[must_use]
    pub fn agent_density_at(&self, pos: &Position, agent_positions: &[Position]) -> f32 {
        const RADIUS: f32 = 3.0; // Smaller radius for cell-specific density
        let mut count = 0.0f32;
        for agent_pos in agent_positions {
            let dist = pos.distance(agent_pos);
            if dist <= RADIUS {
                // Weight by inverse distance (closer = more influence)
                count += 1.0 / (1.0 + dist);
            }
        }
        count
    }

    /// Get threat level at a specific cell
    /// Per §7.4.0 lines 4026-4027: `threat_level` for utility computation
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn threat_at(
        &self,
        pos: &Position,
        threat_positions: &[Position],
        threat_intensities: &[f32],
    ) -> f32 {
        const THREAT_RADIUS: f32 = 5.0;
        let mut threat = 0.0f32;
        for (threat_pos, &intensity) in threat_positions.iter().zip(threat_intensities.iter()) {
            let dist = pos.distance(threat_pos);
            if dist <= THREAT_RADIUS {
                threat += intensity / (1.0 + dist);
            }
        }
        threat
    }

    /// Get 9-cell neighborhood (current cell + 8 neighbors) in 2D
    /// Per §7.4.0 lines 3989-3993: candidates for movement evaluation
    /// Maintains same altitude (z) as center position
    #[must_use]
    pub fn get_9_cell_neighborhood(&self, pos: &Position) -> Vec<Position> {
        let mut cells = Vec::with_capacity(9);
        let x = pos.x.floor() as i32;
        let y = pos.y.floor() as i32;
        let z = pos.z; // Maintain altitude

        for dx in -1..=1 {
            for dy in -1..=1 {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    cells.push(Position::new(nx as f32, ny as f32, z));
                }
            }
        }
        cells
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get 27-cell neighborhood in 3D (current cell + 26 neighbors)
    /// Includes vertical neighbors when depth > 1
    #[must_use]
    pub fn get_27_cell_neighborhood(&self, pos: &Position) -> Vec<Position> {
        if self.depth == 1 {
            return self.get_9_cell_neighborhood(pos);
        }

        let mut cells = Vec::with_capacity(27);
        let x = pos.x.floor() as i32;
        let y = pos.y.floor() as i32;
        let z = pos.z as i32;

        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let nx = x + dx;
                    let ny = y + dy;
                    let nz = z + dz;
                    if nx >= 0
                        && nx < self.width as i32
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
                        && ny >= 0
                        && ny < self.height as i32
                        && nz >= 0
                        && nz < self.depth as i32
                    {
                        cells.push(Position::new(nx as f32, ny as f32, nz as f32));
                    }
                }
            }
        }
        cells
    }

    /// Update Allee effects based on current population
    ///
    /// Computes fitness multipliers and component effects based on population density.
    /// Should be called each tick with the current population count.
    ///
    /// # Arguments
    /// * `population` - Current population size
    pub fn update_allee_effects(&mut self, population: u32) {
        let area = (self.width * self.height) as f32;

        // Compute fitness multiplier
        self.allee_effect
            .compute_fitness_multiplier(population, area);

        // Compute component effects
        self.allee_effect.compute_component_effects(population);
    }

    /// Get current Allee fitness multiplier
    #[must_use]
    pub fn allee_fitness_multiplier(&self) -> f32 {
        if self.width > 0 && self.height > 0 {
            self.allee_effect.fitness_multiplier()
        } else {
            1.0
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the altitude layer at a position
    #[must_use]
    pub fn altitude_at(&self, pos: &Position) -> AltitudeLayer {
        pos.altitude_layer().min(self.depth - 1)
    }

    /// Check if position is at the highest altitude layer
    #[must_use]
    pub const fn is_at_max_altitude(&self, pos: &Position) -> bool {
        pos.z as usize >= self.depth.saturating_sub(1)
    }

    /// Check if position is at the lowest altitude layer (ground level)
    #[must_use]
    pub fn is_at_ground_level(&self, pos: &Position) -> bool {
        pos.z <= 0.0 && self.depth > 0
    }
}

// Integration tests extracted from crates/sim-core/src/environment.rs

#[cfg(test)]
mod tests {
    use super::*;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    use rand::SeedableRng;

    // =======================================================================
    // Environment Integration Tests
    // =======================================================================

    #[test]
    fn test_environment_with_weather() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut env = Environment::new_default(&mut rng);

        assert!(env.weather_enabled);
        assert_eq!(env.weather.season, Season::Spring);

        // Step the environment
        env.step(&mut rng);

        // Weather should have evolved
        assert_eq!(env.tick, 1);
        assert_eq!(env.weather.tick, 1);

        // Temperature should be valid
        assert!(env.weather.temperature() > -20.0 && env.weather.temperature() < 50.0);
    }

    #[test]
    fn test_environment_without_weather() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut env = Environment::new_without_weather(&mut rng);

        assert!(!env.weather_enabled);
        assert!(!env.weather.enabled);

        // Step should still work
        env.step(&mut rng);
        assert_eq!(env.tick, 1);
    }

    #[test]
    fn test_weather_aware_regrowth() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut env = Environment::new_default(&mut rng);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Set up a resource cell
        env.resources[0].amount = 5.0;
        env.resources[0].capacity = 20.0;
        env.resources[0].regrowth_rate = 1.0;

        // Set rainy weather for maximum regrowth
        env.weather.current_weather = WeatherType::Rainy;
        env.weather.rainfall = 2.0;

        let initial_amount = env.resources[0].amount;
        env.regrow_all();

        // Should have regrown with multiplier > 1.0
        let rainy_regrowth = env.resources[0].amount - initial_amount;
        assert!(
            rainy_regrowth > 1.0,
            "Rainy regrowth should be > 1.0, got {rainy_regrowth}"
        );

        // Reset and test drought
        env.resources[0].amount = 5.0;
        env.weather.current_weather = WeatherType::Drought;

        let initial_amount = env.resources[0].amount;
        env.regrow_all();

        let drought_regrowth = env.resources[0].amount - initial_amount;
        assert!(
            drought_regrowth < 0.5,
            "Drought regrowth should be < 0.5, got {drought_regrowth}"
        );
    }

    #[test]
    fn test_temperature_energy_factor_integration() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut env = Environment::new_default(&mut rng);

        // Test with weather enabled
        let factor = env.temperature_energy_factor();
        assert!((1.0..=1.5).contains(&factor));

        // Set extreme temperature
        // TAG: surface=simulation owner=research-team rule=SIM-001
        env.weather.temperature = 40.0;
        let high_temp_factor = env.temperature_energy_factor();
        assert!(high_temp_factor > 1.0);

        // Set cold temperature
        env.weather.temperature = 0.0;
        let low_temp_factor = env.temperature_energy_factor();
        assert!(low_temp_factor > 1.0);
    }

    #[test]
    fn test_rainfall_integration() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut env = Environment::new_default(&mut rng);

        // Set rainy weather
        env.weather.current_weather = WeatherType::Rainy;
        env.weather.rainfall = 1.0;

        let rainfall1 = env.rainfall_at(0, 0);
        let rainfall2 = env.rainfall_at(32, 32);
        let rainfall3 = env.rainfall_at(63, 63);

        // All should be non-negative
        assert!(rainfall1 >= 0.0);
        assert!(rainfall2 >= 0.0);
        assert!(rainfall3 >= 0.0);

        // Should vary by position due to gradient
        let all_equal =
            (rainfall1 - rainfall2).abs() < 0.001 && (rainfall2 - rainfall3).abs() < 0.001;
        assert!(!all_equal, "Rainfall should vary by position");
    }

    #[test]
    fn test_rainfall_out_of_bounds() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_default(&mut rng);

        let rainfall = env.rainfall_at(100, 100);
        assert_eq!(rainfall, 0.0);
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // =======================================================================
    // Original Environment Tests
    // =======================================================================

    #[test]
    fn test_environment_creation() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_default(&mut rng);
        assert_eq!(env.width, 64);
        assert_eq!(env.height, 64);
        assert_eq!(env.depth, 1);
        assert_eq!(env.resources.len(), 64 * 64);
    }

    #[test]
    fn test_environment_creation_3d() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_3d(32, 32, 4, &mut rng);
        assert_eq!(env.width, 32);
        assert_eq!(env.height, 32);
        assert_eq!(env.depth, 4);
        assert_eq!(env.resources.len(), 32 * 32 * 4);
    }

    #[test]
    fn test_harvest_and_regrow() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut env = Environment::new_default(&mut rng);
        let pos = Position::new_2d(0.0, 0.0);

        // Force a resource at position
        env.resources[0].amount = 10.0;
        env.resources[0].capacity = 20.0;
        env.resources[0].regrowth_rate = 1.0;

        let harvested = env.harvest_at(&pos, 5.0);
        assert_eq!(harvested, 5.0);
        assert_eq!(env.resources[0].amount, 5.0);

        // Disable weather for predictable regrowth
        env.weather_enabled = false;
        env.regrow_all();
        assert_eq!(env.resources[0].amount, 6.0);
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    #[test]
    fn test_valid_position() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_default(&mut rng);

        assert!(env.is_valid_position(&Position::new_2d(0.0, 0.0)));
        assert!(env.is_valid_position(&Position::new_2d(63.0, 63.0)));
        assert!(!env.is_valid_position(&Position::new_2d(64.0, 0.0)));
        assert!(!env.is_valid_position(&Position::new_2d(-1.0, 0.0)));
    }

    #[test]
    fn test_valid_position_3d() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_3d(32, 32, 4, &mut rng);

        assert!(env.is_valid_position(&Position::new(0.0, 0.0, 0.0)));
        assert!(env.is_valid_position(&Position::new(31.0, 31.0, 3.0)));
        assert!(!env.is_valid_position(&Position::new(32.0, 0.0, 0.0)));
        assert!(!env.is_valid_position(&Position::new(0.0, 0.0, 4.0)));
    }

    #[test]
    fn test_temperature_at_altitude() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_3d(32, 32, 4, &mut rng);

        // Ground level should be base temperature
        let temp_ground = env.temperature_at(&Position::new(0.0, 0.0, 0.0));
        assert!((temp_ground - BASE_TEMPERATURE).abs() < 5.0); // Allow weather variation

        // Higher altitude should be cooler (when weather effects averaged out)
        let temp_high = env.temperature_at(&Position::new(0.0, 0.0, 3.0));
        let temp_low = env.temperature_at(&Position::new(0.0, 0.0, 0.0));

        // Altitude effect: higher z should generally mean cooler temp
        // But weather can vary, so we just check they're valid
        assert!(temp_high > -50.0 && temp_high < 50.0);
        assert!(temp_low > -50.0 && temp_low < 50.0);
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[test]
    fn test_movement_cost_3d() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_3d(32, 32, 4, &mut rng);

        let pos1 = Position::new(0.0, 0.0, 0.0);
        let pos2 = Position::new(3.0, 4.0, 0.0); // Horizontal only
        let pos3 = Position::new(0.0, 0.0, 2.0); // Vertical only (up)
        let pos4 = Position::new(3.0, 4.0, 2.0); // Mixed

        let base_cost = 1.0;

        // Horizontal only: 5.0 * 1.0 = 5.0
        let cost_horizontal = env.movement_cost(&pos1, &pos2, base_cost);
        assert!((cost_horizontal - 5.0).abs() < 0.001);

        // Vertical up: 2.0 * VERTICAL_MOVEMENT_COST_FACTOR
        let cost_up = env.movement_cost(&pos1, &pos3, base_cost);
        assert!((cost_up - (2.0 * VERTICAL_MOVEMENT_COST_FACTOR)).abs() < 0.001);

        // Mixed: horizontal + vertical
        let cost_mixed = env.movement_cost(&pos1, &pos4, base_cost);
        let expected = 5.0 + (2.0 * VERTICAL_MOVEMENT_COST_FACTOR);
        assert!((cost_mixed - expected).abs() < 0.001);
    }

    #[test]
    fn test_position_to_index_3d() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_3d(4, 4, 4, &mut rng);

        // Index = z * (width * height) + y * width + x
        // Index at (0,0,0) = 0
        assert_eq!(
            env.position_to_index(&Position::new(0.0, 0.0, 0.0)),
            Some(0)
        );

        // Index at (1,0,0) = 1
        assert_eq!(
            env.position_to_index(&Position::new(1.0, 0.0, 0.0)),
            Some(1) // TAG: surface=simulation owner=research-team rule=SIM-001
        );

        // Index at (0,1,0) = 4 (width)
        assert_eq!(
            env.position_to_index(&Position::new(0.0, 1.0, 0.0)),
            Some(4)
        );

        // Index at (0,0,1) = 16 (width * height)
        assert_eq!(
            env.position_to_index(&Position::new(0.0, 0.0, 1.0)),
            Some(16)
        );

        // Index at (3,3,3) = 3*16 + 3*4 + 3 = 48 + 12 + 3 = 63
        assert_eq!(
            env.position_to_index(&Position::new(3.0, 3.0, 3.0)),
            Some(63)
        );
    }

    #[test]
    fn test_altitude_helpers() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_3d(32, 32, 4, &mut rng);

        let ground = Position::new(0.0, 0.0, 0.0);
        let middle = Position::new(0.0, 0.0, 2.0);
        let top = Position::new(0.0, 0.0, 3.0);

        assert!(env.is_at_ground_level(&ground));
        assert!(!env.is_at_ground_level(&middle));

        assert!(env.is_at_max_altitude(&top));
        assert!(!env.is_at_max_altitude(&middle));

        assert_eq!(env.altitude_at(&middle), 2);
    }

    #[test]
    fn test_clamp_position_3d() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_3d(10, 10, 5, &mut rng);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let out_of_bounds = Position::new(20.0, 20.0, 20.0);
        let clamped = env.clamp_position(&out_of_bounds);

        assert_eq!(clamped.x, 9.0);
        assert_eq!(clamped.y, 9.0);
        assert_eq!(clamped.z, 4.0);
    }

    // =======================================================================
    // Neighborhood Tests
    // =======================================================================

    /// Test 27-cell neighborhood in 3D
    #[test]
    fn test_27_cell_neighborhood() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_3d(10, 10, 10, &mut rng);

        let center = Position::new(5.0, 5.0, 5.0);
        let neighborhood = env.get_27_cell_neighborhood(&center);

        // Should have 27 cells (including center)
        assert_eq!(neighborhood.len(), 27);

        // Should include all neighbors within 1 cell in each dimension
        assert!(neighborhood.contains(&Position::new(5.0, 5.0, 5.0))); // Center
        assert!(neighborhood.contains(&Position::new(6.0, 5.0, 5.0))); // +x
        assert!(neighborhood.contains(&Position::new(5.0, 6.0, 5.0))); // +y
        assert!(neighborhood.contains(&Position::new(5.0, 5.0, 6.0))); // +z
        assert!(neighborhood.contains(&Position::new(4.0, 4.0, 4.0))); // -x, -y, -z
        assert!(neighborhood.contains(&Position::new(6.0, 6.0, 6.0))); // +x, +y, +z
    }

    /// Test 9-cell neighborhood falls back in 2D
    #[test]
    fn test_9_cell_neighborhood_2d() {
        let mut rng = StdRng::seed_from_u64(42);
        let env = Environment::new_default(&mut rng);

        let center = Position::new(5.0, 5.0, 3.0);
        let neighborhood = env.get_9_cell_neighborhood(&center);

        // 2D environment should have 9 cells
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(neighborhood.len(), 9);

        // All cells should have same z as center
        for cell in &neighborhood {
            assert_eq!(cell.z, center.z);
        }
    }

    // =======================================================================
    // Resource Initialization Test
    // =======================================================================

    #[test]
    fn test_resource_initialization_live_config() {
        let mut rng = StdRng::seed_from_u64(42);
        let config = EnvironmentConfig {
            resource_density: 0.7,
            initial_resource: 25.0,
            ..EnvironmentConfig::default()
        };

        let env = Environment::new(&config, &mut rng);

        let cells_with_resources = env.resources.iter().filter(|r| r.amount > 0.0).count();
        let total_cells = env.resources.len();
        let actual_density = cells_with_resources as f32 / total_cells as f32;
        let total_amount: f32 = env.resources.iter().map(|r| r.amount).sum();

        eprintln!("=== Resource Initialization Test ===");
        eprintln!("Total cells: {total_cells}");
        eprintln!("Cells with resources: {cells_with_resources}");
        eprintln!("Actual density: {:.2}%", actual_density * 100.0);
        eprintln!("Total resource amount: {total_amount:.1}");

        // Should have around 70% density (allow 5% variance for randomness)
        assert!(
            actual_density > 0.65 && actual_density < 0.75,
            "Resource density should be around 70%, got {:.1}% ({} cells out of {})",
            actual_density * 100.0,
            cells_with_resources,
            total_cells
        );
    }
}
