//! Weather System
//!
//! Dynamic weather with seasonal cycles, rainfall gradients across the grid,
//! and weather effects on resource regrowth and agent energy consumption.

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

// =============================================================================
// Weather System
// =============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Weather type enumeration
/// Represents the current atmospheric conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WeatherType {
    /// Clear skies, optimal conditions
    #[default]
    Sunny,
    /// Overcast, reduced sunlight
    Cloudy,
    /// Precipitation, increased moisture
    Rainy,
    /// Severe weather with heavy precipitation
    Stormy,
    /// Extended dry period, reduced moisture
    Drought,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Season enumeration
/// Affects base temperature and rainfall patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Season {
    /// Spring: Moderate temp, high rainfall
    #[default]
    Spring,
    /// Summer: High temp, moderate rainfall
    Summer,
    /// Autumn: Moderate temp, low rainfall
    Autumn,
    /// Winter: Low temp, very low rainfall
    Winter,
}

impl Season {
    /// Get the base temperature for this season (in Celsius)
    #[must_use]
    pub const fn base_temperature(&self) -> f32 {
        match self {
            Self::Spring => 18.0, // Moderate
            Self::Summer => 28.0, // High
            Self::Autumn => 15.0, // Moderate-low
            Self::Winter => 5.0,  // Low
        }
    }

    /// Get the base rainfall probability for this season
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub const fn base_rainfall_probability(&self) -> f32 {
        match self {
            Self::Spring => 0.35, // High rainfall
            Self::Summer => 0.20, // Moderate rainfall
            Self::Autumn => 0.15, // Low rainfall
            Self::Winter => 0.08, // Very low rainfall
        }
    }

    /// Get the next season in the cycle
    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::Spring => Self::Summer,
            Self::Summer => Self::Autumn,
            Self::Autumn => Self::Winter,
            Self::Winter => Self::Spring,
        }
    }

    /// Get season duration in ticks (default: 1000 ticks per season)
    #[must_use]
    pub const fn duration_ticks(&self) -> u64 {
        1000
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Weather pattern configuration
///
/// Defines how weather evolves over time and space including rainfall
/// gradients, temperature variations, and seasonal transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherPattern {
    /// Width of the grid for gradient calculations
    pub grid_width: usize,
    /// Height of the grid for gradient calculations
    pub grid_height: usize,
    /// Base rainfall amount per tick
    pub base_rainfall: f32,
    /// Rainfall gradient direction (normalized vector)
    pub rainfall_gradient_x: f32,
    /// Rainfall gradient direction (normalized vector)
    pub rainfall_gradient_y: f32,
    /// Maximum rainfall variation due to gradient (0.0 to 1.0)
    pub gradient_strength: f32,
    /// Random variation in weather transitions
    pub weather_volatility: f32,
    /// Temperature variation range (+/- degrees)
    pub temperature_variation: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl WeatherPattern {
    /// Create a new weather pattern for the given grid size
    #[must_use]
    pub const fn new(grid_width: usize, grid_height: usize) -> Self {
        Self {
            grid_width,
            grid_height,
            base_rainfall: 1.0,
            rainfall_gradient_x: 0.5,  // Rain increases toward east
            rainfall_gradient_y: -0.3, // Rain decreases toward north
            gradient_strength: 0.4,
            weather_volatility: 0.1,
            temperature_variation: 5.0,
        }
    }

    /// Calculate rainfall multiplier at a specific position
    /// Returns value between (1.0 - `gradient_strength`) and (1.0 + `gradient_strength`)
    #[must_use]
    pub fn rainfall_multiplier_at(&self, x: usize, y: usize) -> f32 {
        // Normalize position to 0.0 - 1.0 range
        let nx = x as f32 / self.grid_width.max(1) as f32;
        let ny = y as f32 / self.grid_height.max(1) as f32;

        // Calculate dot product with gradient direction
        let gradient_value = nx * self.rainfall_gradient_x + ny * self.rainfall_gradient_y;

        // Map gradient_value (-1.0 to 1.0) to rainfall multiplier range
        1.0 + gradient_value * self.gradient_strength
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get actual rainfall at a position given base rainfall
    #[must_use]
    pub fn rainfall_at(&self, base_rainfall: f32, x: usize, y: usize) -> f32 {
        base_rainfall * self.rainfall_multiplier_at(x, y)
    }
}

impl Default for WeatherPattern {
    fn default() -> Self {
        Self::new(64, 64)
    }
}

/// Weather state struct
///
/// Tracks current weather conditions including temperature, rainfall,
/// and season. Evolves each tick based on seasonal patterns and
/// random volatility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    /// Current weather condition
    pub current_weather: WeatherType,
    /// Current temperature in Celsius
    pub temperature: f32,
    /// Current base rainfall amount
    pub rainfall: f32,
    /// Current season
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub season: Season,
    /// Current simulation tick
    pub tick: u64,
    /// Season start tick (to track season duration)
    pub season_start_tick: u64,
    /// Weather pattern configuration
    pub pattern: WeatherPattern,
    /// Whether weather system is enabled
    pub enabled: bool,
    /// Temperature override (from real-time live intervention or Sim D17 heatwave test)
    #[serde(default)]
    pub temperature_override: Option<f32>,
}

impl WeatherState {
    /// Initialize weather state with Spring season and default values
    #[must_use]
    pub const fn new(grid_width: usize, grid_height: usize) -> Self {
        let season = Season::Spring;
        let base_temp = season.base_temperature();

        Self {
            current_weather: WeatherType::Sunny,
            temperature: base_temp,
            rainfall: 0.0,
            season,
            tick: 0,
            season_start_tick: 0,
            pattern: WeatherPattern::new(grid_width, grid_height),
            enabled: true,
            temperature_override: None,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create with specific grid dimensions and random initial state
    pub fn with_random(grid_width: usize, grid_height: usize, rng: &mut StdRng) -> Self {
        let season = Season::Spring;
        let base_temp = season.base_temperature();

        // Random initial weather based on season
        let current_weather = if rng.gen::<f32>() < season.base_rainfall_probability() {
            if rng.gen::<f32>() < 0.2 {
                WeatherType::Stormy
            } else {
                WeatherType::Rainy
            }
        } else if rng.gen::<f32>() < 0.7 {
            WeatherType::Sunny
        } else {
            WeatherType::Cloudy
        };

        Self {
            current_weather,
            temperature: base_temp + rng.gen::<f32>() * 4.0 - 2.0,
            rainfall: 0.0,
            season,
            tick: 0,
            season_start_tick: 0,
            pattern: WeatherPattern::new(grid_width, grid_height),
            enabled: true,
            temperature_override: None,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Advance weather by one tick
    /// Handles seasonal transitions and random weather changes
    pub fn step(&mut self, rng: &mut StdRng) {
        if !self.enabled {
            return;
        }

        self.tick += 1;

        // Check for season transition
        let season_duration = self.season.duration_ticks();
        let ticks_in_season = self.tick - self.season_start_tick;

        if ticks_in_season >= season_duration {
            self.season = self.season.next();
            self.season_start_tick = self.tick;
        }

        // Update temperature based on season with daily variation (or maintain explicit override)
        if let Some(target_temp) = self.temperature_override {
            self.temperature = target_temp + (rng.gen::<f32>() * 0.2 - 0.1);
        } else {
            let base_temp = self.season.base_temperature();
            let daily_variation =
                (self.tick as f32 / 100.0).sin() * self.pattern.temperature_variation;
            let random_variation = rng.gen::<f32>() * 2.0 - 1.0;
            self.temperature = base_temp + daily_variation + random_variation;
        }

        // Update rainfall based on current weather
        self.rainfall = match self.current_weather {
            WeatherType::Sunny => 0.0,
            WeatherType::Cloudy => 0.1,
            WeatherType::Rainy => 1.0 + rng.gen::<f32>() * 0.5,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            WeatherType::Stormy => 2.0 + rng.gen::<f32>() * 1.0,
            WeatherType::Drought => 0.0,
        };

        // Determine if weather should change
        self.update_weather_type(rng);
    }

    /// Update weather type based on season probabilities and volatility
    fn update_weather_type(&mut self, rng: &mut StdRng) {
        let change_probability = self.pattern.weather_volatility;

        if rng.gen::<f32>() < change_probability {
            let rain_prob = self.season.base_rainfall_probability();
            let storm_prob = rain_prob * 0.2; // 20% of rainy weather is stormy
            let drought_prob = 0.05; // 5% chance of drought conditions

            let roll = rng.gen::<f32>();

            self.current_weather = if roll < drought_prob {
                WeatherType::Drought
            } else if roll < drought_prob + storm_prob {
                WeatherType::Stormy
            } else if roll < drought_prob + storm_prob + rain_prob {
                WeatherType::Rainy
            } else if roll < drought_prob + storm_prob + rain_prob + 0.3 {
                // 30% chance of cloudy when not raining
                WeatherType::Cloudy
            } else {
                WeatherType::Sunny
            };
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get rainfall at a specific position (with gradient)
    #[must_use]
    pub fn rainfall_at(&self, x: usize, y: usize) -> f32 {
        self.pattern.rainfall_at(self.rainfall, x, y)
    }

    /// Get current temperature
    #[must_use]
    pub const fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Get the current season
    #[must_use]
    pub const fn season(&self) -> Season {
        self.season
    }

    /// Get the current weather type
    #[must_use]
    pub const fn weather_type(&self) -> WeatherType {
        self.current_weather
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate regrowth multiplier for resources
    /// Returns value in range [0.0, 2.0]
    /// - Drought: significantly reduced (0.3x)
    /// - Sunny/Cloudy: normal (1.0x)
    /// - Rainy: increased (1.5x)
    /// - Stormy: slightly reduced due to damage (0.8x)
    #[must_use]
    pub const fn regrowth_multiplier(&self) -> f32 {
        match self.current_weather {
            WeatherType::Drought => 0.3,
            WeatherType::Sunny => 1.0,
            WeatherType::Cloudy => 0.9,
            WeatherType::Rainy => 1.5,
            WeatherType::Stormy => 0.8,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate temperature factor for agent energy consumption
    /// Returns value in range [0.5, 1.5]
    /// - Optimal temperature (20°C): 1.0 (baseline)
    /// - Extreme heat (>35°C): up to 1.5 (more energy needed)
    /// - Extreme cold (<0°C): up to 1.5 (more energy needed)
    /// - Moderate deviation: linear interpolation
    #[must_use]
    pub fn temperature_energy_factor(&self) -> f32 {
        const OPTIMAL_TEMP: f32 = 20.0;
        const EXTREME_HEAT: f32 = 35.0;
        const EXTREME_COLD: f32 = 0.0;

        if self.temperature > OPTIMAL_TEMP {
            // Heat stress
            let heat_diff = (self.temperature - OPTIMAL_TEMP).max(0.0);
            let max_heat_diff = EXTREME_HEAT - OPTIMAL_TEMP;
            1.0 + (heat_diff / max_heat_diff).min(1.0) * 0.5
        } else {
            // Cold stress
            let cold_diff = (OPTIMAL_TEMP - self.temperature).max(0.0);
            let max_cold_diff = OPTIMAL_TEMP - EXTREME_COLD;
            1.0 + (cold_diff / max_cold_diff).min(1.0) * 0.5
        }
    }

    /// Calculate position-specific regrowth multiplier
    /// Combines weather-based multiplier with rainfall gradient
    #[must_use]
    pub fn regrowth_multiplier_at(&self, x: usize, y: usize) -> f32 {
        let base_multiplier = self.regrowth_multiplier();
        let rainfall_factor = self.pattern.rainfall_multiplier_at(x, y);

        // Combine: weather effect * local rainfall variation
        // Clamp to [0.1, 2.0] range
        (base_multiplier * rainfall_factor).clamp(0.1, 2.0)
    }

    /// Get ticks remaining in current season
    #[must_use]
    pub const fn ticks_until_season_change(&self) -> u64 {
        let season_duration = self.season.duration_ticks();
        let ticks_in_season = self.tick - self.season_start_tick;
        season_duration.saturating_sub(ticks_in_season)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get progress through current season (0.0 to 1.0)
    #[must_use]
    pub fn season_progress(&self) -> f32 {
        let season_duration = self.season.duration_ticks() as f32;
        let ticks_in_season = (self.tick - self.season_start_tick) as f32;
        (ticks_in_season / season_duration).clamp(0.0, 1.0)
    }

    /// Sim D17: Solar photic lux drive Phi_photic in [0.15, 1.0] (480nm melanopsin drive)
    #[must_use]
    pub fn photic_lux_drive(&self) -> f32 {
        let base_lux: f32 = match self.current_weather {
            WeatherType::Sunny => 1.0,
            WeatherType::Cloudy => 0.40,
            WeatherType::Rainy => 0.20,
            WeatherType::Stormy => 0.15,
            WeatherType::Drought => 0.90,
        };
        let season_factor: f32 = match self.season {
            Season::Summer => 1.0,
            Season::Spring => 0.85,
            Season::Autumn => 0.75,
            Season::Winter => 0.60,
        };
        (base_lux * season_factor).clamp(0.15f32, 1.0f32)
    }

    /// Sim D17: Thermal stress factor F_thermal matching Python sim_d17 / Rust core
    #[must_use]
    pub fn thermal_stress_factor(&self) -> f32 {
        self.temperature_energy_factor()
    }

    /// Sim D17: Environmental weather cognitive bandwidth modifier
    /// kappa_weather = kappa_0 / (F_thermal * (1.0 + 0.35 * (1.0 - Phi_photic)))
    #[must_use]
    pub fn cognitive_bandwidth_weather_modifier(&self) -> f32 {
        let f_thermal = self.thermal_stress_factor();
        let phi_photic = self.photic_lux_drive();
        1.0 / (f_thermal * (1.0 + 0.35 * (1.0 - phi_photic)))
    }

    /// Environmental baseline drift in Master Law S_base(t)
    /// S_base(t) = S_0 + beta_lux * ln(1 + Phi_photic) - beta_heat * max(0, T - 22.0)
    #[must_use]
    pub fn environmental_baseline_drift(&self, s_0: f32) -> f32 {
        let phi = self.photic_lux_drive();
        let heat_excess = (self.temperature - 22.0).max(0.0);
        const BETA_LUX: f32 = 0.05;
        const BETA_HEAT: f32 = 0.01;
        (s_0 + BETA_LUX * (1.0 + phi).ln() - BETA_HEAT * heat_excess).clamp(0.05, 0.95)
    }
}

impl Default for WeatherState {
    fn default() -> Self {
        Self::new(64, 64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    // =======================================================================
    // Weather System Unit Tests
    // =======================================================================

    #[test]
    fn test_weather_type_variants() {
        // Verify all weather types exist and can be created
        let types = [
            WeatherType::Sunny,
            WeatherType::Cloudy,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            WeatherType::Rainy,
            WeatherType::Stormy,
            WeatherType::Drought,
        ];
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_season_variants_and_properties() {
        // Test season base temperatures
        assert_eq!(Season::Spring.base_temperature(), 18.0);
        assert_eq!(Season::Summer.base_temperature(), 28.0);
        assert_eq!(Season::Autumn.base_temperature(), 15.0);
        assert_eq!(Season::Winter.base_temperature(), 5.0);

        // Test season rainfall probabilities
        assert!(
            Season::Spring.base_rainfall_probability() > Season::Summer.base_rainfall_probability()
        );
        assert!(
            Season::Summer.base_rainfall_probability() > Season::Autumn.base_rainfall_probability()
        );
        assert!(
            Season::Autumn.base_rainfall_probability() > Season::Winter.base_rainfall_probability()
        );
    }

    #[test]
    fn test_season_cycle() {
        // Test that seasons cycle correctly
        assert_eq!(Season::Spring.next(), Season::Summer);
        assert_eq!(Season::Summer.next(), Season::Autumn);
        assert_eq!(Season::Autumn.next(), Season::Winter);
        assert_eq!(Season::Winter.next(), Season::Spring);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_weather_state_new() {
        let weather = WeatherState::new(64, 64);

        assert_eq!(weather.season, Season::Spring);
        assert_eq!(weather.tick, 0);
        assert_eq!(weather.current_weather, WeatherType::Sunny);
        assert!(weather.enabled);
    }

    #[test]
    fn test_weather_state_step_increments_tick() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut weather = WeatherState::new(64, 64);

        assert_eq!(weather.tick, 0);
        weather.step(&mut rng);
        assert_eq!(weather.tick, 1);
        weather.step(&mut rng);
        assert_eq!(weather.tick, 2);
    }

    #[test]
    fn test_weather_season_transition() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut weather = WeatherState::new(64, 64);

        assert_eq!(weather.season, Season::Spring);

        // Step through spring (1000 ticks)
        for _ in 0..1000 {
            weather.step(&mut rng);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(weather.season, Season::Summer);
        assert_eq!(weather.season_start_tick, 1000);

        // Step through summer
        for _ in 0..1000 {
            weather.step(&mut rng);
        }

        assert_eq!(weather.season, Season::Autumn);

        // Step through autumn
        for _ in 0..1000 {
            weather.step(&mut rng);
        }

        assert_eq!(weather.season, Season::Winter);

        // Step through winter
        for _ in 0..1000 {
            weather.step(&mut rng);
        }

        // Back to spring
        assert_eq!(weather.season, Season::Spring);
    }

    #[test]
    fn test_regrowth_multiplier() {
        let mut weather = WeatherState::new(64, 64);

        // Test each weather type's regrowth multiplier
        weather.current_weather = WeatherType::Drought;
        assert_eq!(weather.regrowth_multiplier(), 0.3);

        weather.current_weather = WeatherType::Sunny;
        assert_eq!(weather.regrowth_multiplier(), 1.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        weather.current_weather = WeatherType::Cloudy;
        assert_eq!(weather.regrowth_multiplier(), 0.9);

        weather.current_weather = WeatherType::Rainy;
        assert_eq!(weather.regrowth_multiplier(), 1.5);

        weather.current_weather = WeatherType::Stormy;
        assert_eq!(weather.regrowth_multiplier(), 0.8);
    }

    #[test]
    fn test_temperature_energy_factor() {
        let mut weather = WeatherState::new(64, 64);

        // At optimal temperature (20°C), factor should be 1.0
        weather.temperature = 20.0;
        assert_eq!(weather.temperature_energy_factor(), 1.0);

        // At extreme heat (35°C), factor should be 1.5
        weather.temperature = 35.0;
        assert_eq!(weather.temperature_energy_factor(), 1.5);

        // At extreme cold (0°C), factor should be 1.5
        weather.temperature = 0.0;
        assert_eq!(weather.temperature_energy_factor(), 1.5);

        // At moderate heat (27.5°C), factor should be 1.25
        weather.temperature = 27.5;
        assert_eq!(weather.temperature_energy_factor(), 1.25);

        // At moderate cold (10°C), factor should be 1.25 (halfway between 1.0 and 1.5)
        weather.temperature = 10.0;
        assert_eq!(weather.temperature_energy_factor(), 1.25);
    }

    #[test]
    fn test_weather_pattern_rainfall_gradient() {
        let pattern = WeatherPattern::new(64, 64);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Test that rainfall varies across positions
        let rainfall_center = pattern.rainfall_multiplier_at(32, 32);
        let rainfall_corner = pattern.rainfall_multiplier_at(0, 0);
        let rainfall_far = pattern.rainfall_multiplier_at(63, 63);

        // Multipliers should be different at different positions
        assert!(rainfall_center != rainfall_corner || rainfall_center != rainfall_far);

        // All multipliers should be within valid range
        assert!((0.6..=1.4).contains(&rainfall_center));
        assert!((0.6..=1.4).contains(&rainfall_corner));
        assert!((0.6..=1.4).contains(&rainfall_far));
    }

    #[test]
    fn test_rainfall_at_position() {
        let mut weather = WeatherState::new(64, 64);
        weather.rainfall = 1.0;
        weather.current_weather = WeatherType::Rainy;

        // Get rainfall at different positions
        let rainfall_0_0 = weather.rainfall_at(0, 0);
        let rainfall_32_32 = weather.rainfall_at(32, 32);
        let rainfall_63_63 = weather.rainfall_at(63, 63);

        // Should vary by position
        assert!(rainfall_0_0 > 0.0);

        // All should be positive for rainy weather
        assert!(rainfall_0_0 >= 0.0);
        assert!(rainfall_32_32 >= 0.0);
        assert!(rainfall_63_63 >= 0.0);
    }

    #[test]
    fn test_season_progress() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut weather = WeatherState::new(64, 64);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // At start of season, progress should be 0.0
        assert_eq!(weather.season_progress(), 0.0);

        // Step halfway through season
        for _ in 0..500 {
            weather.step(&mut rng);
        }

        let progress = weather.season_progress();
        assert!(
            (0.49..=0.51).contains(&progress),
            "Progress should be ~0.5, got {progress}"
        );

        // Step to end of season
        for _ in 0..500 {
            weather.step(&mut rng);
        }

        assert_eq!(weather.season_progress(), 0.0); // Reset to 0 in new season
    }

    #[test]
    fn test_ticks_until_season_change() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut weather = WeatherState::new(64, 64);

        assert_eq!(weather.ticks_until_season_change(), 1000);

        for _ in 0..100 {
            weather.step(&mut rng);
        }

        assert_eq!(weather.ticks_until_season_change(), 900);
    }

    #[test]
    fn test_regrowth_multiplier_at_position() {
        let mut weather = WeatherState::new(64, 64);
        weather.current_weather = WeatherType::Rainy;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Get multiplier at different positions
        let mult_0_0 = weather.regrowth_multiplier_at(0, 0);
        let mult_32_32 = weather.regrowth_multiplier_at(32, 32);
        let mult_63_63 = weather.regrowth_multiplier_at(63, 63);

        // All should be positive
        assert!(mult_0_0 > 0.0);
        assert!(mult_32_32 > 0.0);
        assert!(mult_63_63 > 0.0);

        // Should vary based on position (gradient effect)
        // At least some positions should have different multipliers
        assert!(mult_0_0 != mult_63_63 || mult_32_32 != mult_0_0);

        // All should be clamped to [0.1, 2.0]
        assert!((0.1..=2.0).contains(&mult_0_0));
        assert!((0.1..=2.0).contains(&mult_32_32));
        assert!((0.1..=2.0).contains(&mult_63_63));
    }

    #[test]
    fn test_weather_with_random() {
        let mut rng = StdRng::seed_from_u64(42);
        let weather = WeatherState::with_random(64, 64, &mut rng);

        // Should have valid initial values
        assert!(weather.temperature >= -10.0 && weather.temperature <= 50.0);
        assert!(weather.tick == 0);
    }
    #[test]
    fn test_sim_d17_photic_thermal_coupling() {
        let mut weather = WeatherState::new(64, 64);
        weather.current_weather = WeatherType::Sunny;
        weather.season = Season::Summer;
        weather.temperature = 20.0;

        // Temperate baseline
        let lux_summer = weather.photic_lux_drive();
        assert!((lux_summer - 1.0).abs() < 1e-4);
        let mod_temperate = weather.cognitive_bandwidth_weather_modifier();
        assert!((mod_temperate - 1.0).abs() < 1e-4);

        // Heatwave: 38°C
        weather.temperature = 38.0;
        let mod_heatwave = weather.cognitive_bandwidth_weather_modifier();
        assert!(
            mod_heatwave < 0.70,
            "Heatwave must compress cognitive bandwidth"
        );

        // Winter overcast
        weather.current_weather = WeatherType::Stormy;
        weather.season = Season::Winter;
        weather.temperature = 5.0;
        let lux_winter = weather.photic_lux_drive();
        assert!(lux_winter <= 0.15);
        let s_drift = weather.environmental_baseline_drift(0.20);
        assert!(s_drift > 0.05 && s_drift < 0.30);
    }
}
