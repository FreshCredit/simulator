//! Configuration parsing and validation for sim-runner
//!
//! Supports both TOML and JSON configuration files.
//!
//! # Example TOML
//!
//! ```toml
//! [simulation]
//! agent_count = 1000
//! max_ticks = 2000
//! seed = 42
//! stage = 5
//!
//! [toggles]
//! genome_enabled = true
//! docas_enabled = true
//!
//! [output]
//! dir = "./output"
//! replay_interval = 100
//! ```

use serde::{Deserialize, Serialize};
use sim_core::protocol::VerificationMode;
use std::path::Path;

/// Main configuration for simulation runs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunConfig {
    /// Simulation parameters
    #[serde(default)]
    pub simulation: SimulationConfig,

    /// Toggle configuration
    #[serde(default)]
    pub toggles: ToggleConfig,

    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,

    /// Batch execution configuration
    #[serde(default)]
    pub batch: BatchConfig,

    /// Sweep configuration (optional)
    #[serde(default)]
    pub sweep: Option<SweepConfig>,
}
// TAG: surface=simulation owner=research-team rule=SIM-001

impl RunConfig {
    /// Load config from TOML file
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        Self::from_toml(&content)
    }

    /// Parse config from TOML string
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        toml::from_str(content).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Load config from JSON file
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        Self::from_json(&content)
    }

    /// Parse config from JSON string
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_json(content: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(content).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Validate configuration
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.simulation.agent_count == 0 {
            return Err(ConfigError::ValidationError(
                "agent_count must be > 0".to_string(),
            ));
        }
        if self.simulation.max_ticks == 0 {
            return Err(ConfigError::ValidationError(
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                "max_ticks must be > 0".to_string(),
            ));
        }
        if self.simulation.stage > 5 {
            return Err(ConfigError::ValidationError(
                "stage must be 0-5".to_string(),
            ));
        }
        if self.batch.replications == 0 {
            return Err(ConfigError::ValidationError(
                "replications must be > 0".to_string(),
            ));
        }
        Ok(())
    }

    /// Serialize to TOML string
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn to_toml(&self) -> Result<String, ConfigError> {
        self.validate()?;
        toml::to_string_pretty(self).map_err(|e| ConfigError::SerializeError(e.to_string()))
    }

    /// Serialize to JSON string
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn to_json(&self) -> Result<String, ConfigError> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(|e| ConfigError::SerializeError(e.to_string()))
    }
}

/// Simulation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Number of initial agents
    #[serde(default = "default_agent_count")]
    pub agent_count: u32,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Maximum ticks to run
    #[serde(default = "default_max_ticks")]
    pub max_ticks: u64,

    /// Master random seed
    #[serde(default = "default_seed")]
    pub seed: u64,

    /// Stage (0-5)
    #[serde(default = "default_stage")]
    pub stage: u8,

    /// Grid width
    #[serde(default = "default_grid_size")]
    pub grid_width: usize,

    /// Grid height
    #[serde(default = "default_grid_size")]
    pub grid_height: usize,

    /// Verification mode (Protocol ON vs OFF comparison)
    #[serde(default)]
    pub verification_mode: VerificationMode,

    // Sweepable parameter fields
    /// DOCAS alpha D
    #[serde(default)]
    pub docas_alpha_d: Option<f32>,
    /// DOCAS alpha O
    #[serde(default)]
    pub docas_alpha_o: Option<f32>,
    /// DOCAS alpha C
    #[serde(default)]
    pub docas_alpha_c: Option<f32>,
    /// DOCAS alpha A
    #[serde(default)]
    pub docas_alpha_a: Option<f32>,
    /// DOCAS alpha S
    #[serde(default)]
    pub docas_alpha_s: Option<f32>,
    /// DOCAS beta D
    #[serde(default)]
    pub docas_beta_d: Option<f32>,
    /// DOCAS beta O
    #[serde(default)]
    pub docas_beta_o: Option<f32>,
    /// DOCAS beta C
    #[serde(default)]
    pub docas_beta_c: Option<f32>,
    /// DOCAS beta A
    #[serde(default)]
    pub docas_beta_a: Option<f32>,
    /// DOCAS beta S
    #[serde(default)]
    pub docas_beta_s: Option<f32>,
    /// Planning depth k
    #[serde(default)]
    pub planning_depth_k: Option<u32>,
    /// Planning horizon h
    #[serde(default)]
    pub planning_horizon_h: Option<u64>,
    /// Initial resource
    #[serde(default)]
    pub initial_resource: Option<f32>,
    /// Resource density
    #[serde(default)]
    pub resource_density: Option<f32>,
    /// Verification cost
    #[serde(default)]
    pub verification_cost: Option<f32>,
    /// Attestation stake
    #[serde(default)]
    pub attestation_stake: Option<f32>,
}

const fn default_agent_count() -> u32 {
    100
}
const fn default_max_ticks() -> u64 {
    1000
}
const fn default_seed() -> u64 {
    42
}
const fn default_stage() -> u8 {
    0
}
const fn default_grid_size() -> usize {
    64
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            agent_count: default_agent_count(),
            max_ticks: default_max_ticks(),
            seed: default_seed(),
            stage: default_stage(),
            grid_width: default_grid_size(),
            grid_height: default_grid_size(),
            verification_mode: VerificationMode::DecentralizedProtocol,
            docas_alpha_d: None,
            docas_alpha_o: None,
            docas_alpha_c: None,
            docas_alpha_a: None,
            docas_alpha_s: None,
            docas_beta_d: None,
            docas_beta_o: None,
            docas_beta_c: None,
            docas_beta_a: None,
            docas_beta_s: None,
            planning_depth_k: None,
            planning_horizon_h: None,
            initial_resource: None,
            resource_density: None,
            verification_cost: None,
            attestation_stake: None,
        }
    }
}

/// Toggle configuration (mirrors `sim-core::ToggleConfig`)
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToggleConfig {
    /// Enable genome subsystem
    #[serde(default)]
    pub genome_enabled: bool,
    /// Enable DOCAS subsystem
    #[serde(default)]
    pub docas_enabled: bool,
    /// Enable DMN subsystem
    #[serde(default)]
    pub dmn_enabled: bool,
    /// Enable criticality analysis
    #[serde(default)]
    pub criticality_enabled: bool,
    /// Enable choice architecture
    #[serde(default)]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub choice_architecture_enabled: bool,
    /// Enable institutions subsystem
    #[serde(default)]
    pub institutions_enabled: bool,
    /// Enable norms subsystem
    #[serde(default)]
    pub norms_enabled: bool,
    /// Enable language subsystem
    #[serde(default)]
    pub language_enabled: bool,
    /// Enable protocol subsystem
    #[serde(default)]
    pub protocol_enabled: bool,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output directory for artifacts
    #[serde(default = "default_output_dir")]
    pub dir: String,

    /// Replay frame interval (ticks between snapshots)
    #[serde(default = "default_replay_interval")]
    pub replay_interval: u64,

    /// Whether to compress output
    #[serde(default)]
    pub compress: bool,
}

fn default_output_dir() -> String {
    "./output".to_string()
}
const fn default_replay_interval() -> u64 {
    100
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            dir: default_output_dir(),
            replay_interval: default_replay_interval(),
            compress: false,
        }
    }
}

/// Batch execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
pub struct BatchConfig {
    /// Number of replications per configuration
    #[serde(default = "default_replications")]
    pub replications: u32,

    /// Whether to run in parallel
    #[serde(default = "default_parallel")]
    pub parallel: bool,

    /// Number of worker threads (0 = auto)
    #[serde(default)]
    pub threads: usize,
}

const fn default_replications() -> u32 {
    1
}
const fn default_parallel() -> bool {
    true
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            replications: default_replications(),
            parallel: default_parallel(),
            threads: 0,
        }
    }
}

/// Sweep configuration for parameter exploration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepConfig {
    /// Parameter to sweep
    pub parameter: String,

    /// Values to sweep over
    pub values: Vec<f64>,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Sweep type
    #[serde(default)]
    pub sweep_type: SweepType,
}

/// Type of parameter sweep
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SweepType {
    /// Linear spacing between parameter values
    #[default]
    Linear,
    /// Logarithmic spacing between parameter values
    Logarithmic,
    /// Cartesian product grid over multiple parameters
    Grid,
    /// Random sampling within parameter bounds
    MonteCarlo,
}

/// Configuration errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// I/O operation failed
    IoError(String),
    /// Failed to parse configuration
    ParseError(String),
    /// Configuration validation failed
    ValidationError(String),
    /// Serialization error
    SerializeError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            Self::SerializeError(msg) => write!(f, "Serialize error: {msg}"),
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RunConfig::default();
        assert_eq!(config.simulation.agent_count, 100);
        assert_eq!(config.simulation.max_ticks, 1000);
        assert_eq!(config.simulation.seed, 42);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_toml_parsing() {
        let toml = r#"
[simulation]
agent_count = 500
max_ticks = 2000
seed = 123
stage = 3

[toggles]
genome_enabled = true
docas_enabled = true

[output]
dir = "./my_output"
replay_interval = 50
"#;
        // ERR-001: Test code - unwrap() exempt for test assertions
        let config = RunConfig::from_toml(toml).unwrap();
        assert_eq!(config.simulation.agent_count, 500);
        assert_eq!(config.simulation.max_ticks, 2000);
        assert_eq!(config.simulation.stage, 3);
        assert!(config.toggles.genome_enabled);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(config.output.replay_interval, 50);
    }

    #[test]
    fn test_json_parsing() {
        let json = r#"{
            "simulation": {
                "agent_count": 200,
                "max_ticks": 500
            }
        }"#;
        // ERR-001: Test code - unwrap() exempt for test assertions
        let config = RunConfig::from_json(json).unwrap();
        assert_eq!(config.simulation.agent_count, 200);
        assert_eq!(config.simulation.max_ticks, 500);
    }

    #[test]
    fn test_validation_errors() {
        let mut config = RunConfig::default();
        config.simulation.agent_count = 0;
        assert!(config.validate().is_err());

        config.simulation.agent_count = 100;
        config.simulation.max_ticks = 0;
        assert!(config.validate().is_err());

        config.simulation.max_ticks = 1000;
        config.simulation.stage = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_roundtrip_toml() {
        let config = RunConfig::default();
        // ERR-001: Test code - unwrap() exempt for test assertions
        let toml = config.to_toml().unwrap();
        let parsed = RunConfig::from_toml(&toml).unwrap();
        assert_eq!(parsed.simulation.agent_count, config.simulation.agent_count);
    }
}
