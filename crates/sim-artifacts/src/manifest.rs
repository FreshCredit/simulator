// TAG: surface=simulation owner=research-team rule=SIM-001
//! Manifest schema for simulation runs
//!
//! Per Appendix B of simulation-plan.md:
//! - `manifest.json` contains run metadata, config, seeds, and file references
//! - Schema version tracked for backward compatibility (SIM-005)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::SCHEMA_VERSION;

/// Main manifest structure for a simulation run
/// Per §Appendix B: manifest.json schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Schema version for backward compatibility (SIM-005)
    pub schema_version: String,

    /// Unique run identifier
    pub run_id: String,

    /// Git commit hash for reproducibility
    pub git_commit: String,

    /// Run timestamp (ISO8601)
    pub timestamp: DateTime<Utc>,

    /// Simulation configuration
    pub config: RunConfig,

    /// RNG seeds for deterministic replay (SIM-004)
    pub seeds: SeedConfig,

    /// External dataset versions
    pub datasets: DatasetVersions,

    /// Path to metrics file (relative to manifest)
    pub metrics_file: String,

    /// Path to replay directory (relative to manifest)
    pub replay_dir: String,

    /// Run status
    pub status: RunStatus,

    /// Optional validation results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationResults>,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Optional error message if run failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Manifest {
    /// Create a new manifest with default values
    #[must_use]
    pub fn new(master_seed: u64, git_commit: String) -> Self {
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            run_id: Uuid::new_v4().to_string(),
            git_commit,
            timestamp: Utc::now(),
            config: RunConfig::default(),
            seeds: SeedConfig::from_master(master_seed),
            datasets: DatasetVersions::default(),
            metrics_file: "metrics.parquet".to_string(),
            replay_dir: "replay/".to_string(),
            status: RunStatus::Running,
            validation: None,
            error: None,
        }
    }

    /// Mark run as complete
    pub fn complete(&mut self) {
        self.status = RunStatus::Complete;
    }

    /// Mark run as failed with error message
    pub fn fail(&mut self, error: String) {
        self.status = RunStatus::Failed;
        self.error = Some(error);
    }

    /// Serialize to JSON string
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        serde_json::from_str(json)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
}

/// Run status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// Run is currently in progress
    Running,
    /// Run finished successfully
    Complete,
    /// Run encountered an error
    Failed,
    /// Run was cancelled by user or system
    Cancelled,
}

/// Simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    /// Number of agents
    pub agent_count: u32,

    /// Maximum ticks to run
    pub max_ticks: u64,

    /// Grid width
    pub grid_width: u32,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Grid height
    pub grid_height: u32,

    /// Toggle configuration
    pub toggles: ToggleSnapshot,

    /// Stage name (0-5)
    pub stage: String,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            agent_count: 100,
            max_ticks: 1000,
            grid_width: 64,
            grid_height: 64,
            toggles: ToggleSnapshot::default(),
            stage: "0".to_string(),
        }
    }
}

/// Snapshot of toggle configuration
/// Mirrors `sim-core::ToggleConfig` but decoupled for SIM-001
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct ToggleSnapshot {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Genome module enabled
    pub genome_enabled: bool,
    /// DOCAS module enabled
    pub docas_enabled: bool,
    /// DMN module enabled
    pub dmn_enabled: bool,
    /// Criticality module enabled
    pub criticality_enabled: bool,
    /// Choice architecture module enabled
    pub choice_architecture_enabled: bool,
    /// Institutions module enabled
    pub institutions_enabled: bool,
    /// Norms module enabled
    pub norms_enabled: bool,
    /// Language module enabled
    pub language_enabled: bool,
    /// Protocol module enabled
    pub protocol_enabled: bool,
}

/// RNG seed configuration for deterministic replay (SIM-004)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedConfig {
    /// Master seed (used to derive others if not specified)
    pub master: u64,

    /// Agent initialization seed
    pub agent_init: u64,

    /// Environment seed
    pub environment: u64,
}

impl SeedConfig {
    /// Create seed config from master seed
    /// Derives other seeds deterministically
    #[must_use]
    pub const fn from_master(master: u64) -> Self {
        Self {
            master,
            agent_init: master.wrapping_mul(31).wrapping_add(1),
            environment: master.wrapping_mul(17).wrapping_add(2),
        }
    }
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// External dataset versions for reproducibility
// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatasetVersions {
    /// CLICS3 colexification database version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clics3: Option<String>,

    /// Other dataset versions can be added here
    #[serde(flatten)]
    pub other: std::collections::HashMap<String, String>,
}

/// Validation results from POM checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    /// Whether all POM targets were met
    pub passed: bool,

    /// Individual POM check results
    pub pom_checks: Vec<PomCheckResult>,

    /// Effect size calculations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_sizes: Option<EffectSizes>,
}

/// Individual POM check result
// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomCheckResult {
    /// Name of the pattern
    pub pattern: String,

    /// Whether the pattern was observed
    pub observed: bool,

    /// Measured value
    pub value: f64,

    /// Expected range (min, max)
    pub expected_range: (f64, f64),
}

/// Effect size calculations for protocol comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSizes {
    /// Cohen's d for cooperation rate
    pub cooperation_cohens_d: f64,

    /// Cohen's d for dispute rate
    pub dispute_cohens_d: f64,

    /// Cohen's d for Gini coefficient
    pub gini_cohens_d: f64,
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = Manifest::new(42, "abc123".to_string());
        assert_eq!(manifest.schema_version, "1.0");
        assert_eq!(manifest.seeds.master, 42);
        assert_eq!(manifest.status, RunStatus::Running);
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = Manifest::new(42, "abc123".to_string());
        let json = manifest.to_json().unwrap();
        assert!(json.contains("schema_version"));
        assert!(json.contains("1.0"));
    }

    #[test]
    fn test_manifest_deserialization() {
        let manifest = Manifest::new(42, "abc123".to_string());
        let json = manifest.to_json().unwrap();
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let parsed = Manifest::from_json(&json).unwrap();
        assert_eq!(parsed.seeds.master, 42);
    }

    #[test]
    fn test_seed_derivation() {
        let seeds = SeedConfig::from_master(42);
        assert_eq!(seeds.master, 42);
        assert_ne!(seeds.agent_init, seeds.master);
        assert_ne!(seeds.environment, seeds.master);
    }

    #[test]
    fn test_run_status_complete() {
        let mut manifest = Manifest::new(42, "abc123".to_string());
        manifest.complete();
        assert_eq!(manifest.status, RunStatus::Complete);
    }

    #[test]
    fn test_run_status_fail() {
        let mut manifest = Manifest::new(42, "abc123".to_string());
        manifest.fail("OOM error".to_string());
        assert_eq!(manifest.status, RunStatus::Failed);
        assert_eq!(manifest.error, Some("OOM error".to_string()));
    }
}
