//! sim-report: Report Generation for Agent Society Simulator
//!
//! B-002: Critical blocker for research output
//!
//! This crate provides comprehensive report generation for simulation results,
//! including:
//! - POM validation reports
//! - GTT hypothesis testing reports
//! - Statistical summaries
//! - Determinism certificates
//! - Reproducibility information
//!
//! # Report Formats
//!
//! - **Markdown:** Human-readable research reports
//! - **JSON:** Machine-readable for further analysis
//! - **HTML:** Interactive web reports (future)
//!
//! # Example
//!
//! ```rust,no_run
//! use sim_report::{ReportGenerator, ReportConfig, GttHypothesisResult};
//!
//! let config = ReportConfig::default();
//! let generator = ReportGenerator::new(config);
//!
//! // Generate hypothesis report
//! // (In real usage, this would come from sim-runner)
//! let mock_result = GttHypothesisResult {
//!     n_runs: 20,
//!     seed: 42,
//!     protocol_on: sim_report::ConditionResults { n: 20, mean: 0.7, std: 0.1, values: vec![] },
//!     protocol_off: sim_report::ConditionResults { n: 20, mean: 0.6, std: 0.1, values: vec![] },
//!     effect_size: 0.5,
//!     effect_size_ci_lower: 0.2,
//!     effect_size_ci_upper: 0.8,
//!     effect_size_interpretation: "medium".to_string(),
//!     t_statistic: 2.5,
//!     p_value: 0.01,
//!     hypothesis_supported: true,
//!     confidence_level: 0.95,
//! };
//! let hypothesis_report = generator.generate_hypothesis_report(&mock_result);
//!
//! // Get Markdown output
//! let markdown = hypothesis_report.to_markdown();
//! ```
//!
//! SIM-001: Isolation verified - no product dependencies

// TAG: surface=simulation owner=research-team rule=SIM-001

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sim_core::pom::{PomPriority, PomValidationSuite};
use std::collections::HashMap;

// Hypothesis types (local to avoid circular dependency)
pub use hypothesis_types::{ConditionResults, GttHypothesisResult};

pub mod hypothesis_types;
pub mod templates;

/// Report generator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Report title
    pub title: String,
    /// Author information
    pub author: String,
    /// Institution/organization
    pub institution: String,
    /// Report version
    pub version: String,
    /// Include raw data in reports
    pub include_raw_data: bool,
    /// Confidence level for statistics (default 0.95)
    pub confidence_level: f32,
    /// Date format string
    pub date_format: String,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            title: "FreshCredit Agent Society Simulation Report".to_string(),
            author: "FreshCredit Research".to_string(),
            institution: "FreshCredit Labs".to_string(),
            version: "1.0.0".to_string(),
            include_raw_data: false,
            confidence_level: 0.95,
            date_format: "%Y-%m-%d %H:%M:%S UTC".to_string(),
        }
    }
}

/// Main report generator
#[derive(Debug)]
pub struct ReportGenerator {
    config: ReportConfig,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

impl ReportGenerator {
    /// Create a new report generator with the given configuration
    #[must_use]
    pub const fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    /// Generate a comprehensive POM validation report
    ///
    /// # Arguments
    /// * `suite` - POM validation suite results
    ///
    /// # Returns
    /// A complete validation report
    #[must_use]
    pub fn generate_pom_report(&self, suite: &PomValidationSuite) -> PomValidationReport {
        PomValidationReport {
            generated_at: Utc::now(),
            config: self.config.clone(),
            suite: suite.clone(),
            summary: Self::summarize_pom(suite),
        }
    }

    /// Generate a GTT hypothesis testing report
    ///
    /// # Arguments
    /// * `result` - GTT hypothesis test results
    ///
    /// # Returns
    /// A complete hypothesis testing report
    #[must_use]
    pub fn generate_hypothesis_report(&self, result: &GttHypothesisResult) -> HypothesisReport {
        HypothesisReport {
            generated_at: Utc::now(),
            config: self.config.clone(),
            result: result.clone(),
            interpretation: Self::interpret_hypothesis(result),
        }
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    /// Generate a comprehensive simulation report combining all results
    ///
    /// # Arguments
    /// * `pom_suite` - Optional POM validation results
    /// * `hypothesis_result` - Optional hypothesis test results
    /// * `metadata` - Additional simulation metadata
    ///
    /// # Returns
    /// A complete simulation report
    #[must_use]
    pub fn generate_full_report(
        &self,
        pom_suite: Option<&PomValidationSuite>,
        hypothesis_result: Option<&GttHypothesisResult>,
        metadata: SimulationMetadata,
    ) -> FullSimulationReport {
        FullSimulationReport {
            generated_at: Utc::now(),
            config: self.config.clone(),
            metadata,
            pom_validation: pom_suite.map(|s| self.generate_pom_report(s)),
            hypothesis_test: hypothesis_result.map(|r| self.generate_hypothesis_report(r)),
            reproducibility: Self::generate_reproducibility_info(),
        }
    }

    /// Generate determinism certificate
    ///
    /// Creates a certificate verifying that the simulation is deterministic
    /// and reproducible.
    #[must_use]
    pub fn generate_determinism_certificate(
        &self,
        seeds: &[u64],
        checksums: &[String],
    ) -> DeterminismCertificate {
        let verified = checksums.windows(2).all(|w| w[0] == w[1]);

        DeterminismCertificate {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            generated_at: Utc::now(),
            config: self.config.clone(),
            seeds: seeds.to_vec(),
            checksums: checksums.to_vec(),
            verified,
            sim_004_compliant: verified,
        }
    }

    /// Summarize POM validation results
    #[allow(clippy::cast_precision_loss)]
    fn summarize_pom(suite: &PomValidationSuite) -> PomSummary {
        let total = suite.total_count;
        let passed = suite.passed_count;
        let failed = total - passed;
        let pass_rate = if total > 0 {
            passed as f32 / total as f32
        } else {
            0.0
        };

        // Count by priority
        let p0_total = suite
            .categories
            .values()
            .flat_map(|c| &c.targets)
            .filter(|t| t.priority == PomPriority::P0)
            .count();
        let p0_passed = suite
            .categories
            .values()
            .flat_map(|c| &c.targets)
            .filter(|t| t.priority == PomPriority::P0 && t.passed)
            .count();

        PomSummary {
            total_targets: total,
            passed,
            failed,
            pass_rate,
            p0_targets: p0_total,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            p0_passed,
            p0_pass_rate: if p0_total > 0 {
                p0_passed as f32 / p0_total as f32
            } else {
                0.0
            },
            category_summaries: suite
                .categories
                .values()
                .map(|c| CategorySummary {
                    category: format!("{:?}", c.category),
                    total: c.total,
                    passed: c.passed,
                    pass_rate: c.pass_rate,
                })
                .collect(),
        }
    }

    /// Interpret hypothesis test results
    fn interpret_hypothesis(result: &GttHypothesisResult) -> HypothesisInterpretation {
        let effect_magnitude = if result.effect_size.abs() < 0.2 {
            "negligible"
        } else if result.effect_size.abs() < 0.5 {
            "small"
        } else if result.effect_size.abs() < 0.8 {
            "medium"
        } else {
            "large"
        };

        let significance = if result.p_value < 0.001 {
            "highly significant (p < 0.001)"
        } else if result.p_value < 0.01 {
            "very significant (p < 0.01)"
        } else if result.p_value < 0.05 {
            "significant (p < 0.05)"
        } else {
            "not significant (p >= 0.05)"
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let practical_significance = if result.effect_size >= 0.15 {
            "practically significant (d >= 0.15)"
        } else {
            "not practically significant (d < 0.15)"
        };

        HypothesisInterpretation {
            effect_magnitude: effect_magnitude.to_string(),
            significance: significance.to_string(),
            practical_significance: practical_significance.to_string(),
            conclusion: if result.hypothesis_supported {
                "The GTT hypothesis is supported: Protocol ON significantly increases trust compared to Protocol OFF."
            } else {
                "The GTT hypothesis is not supported: Insufficient evidence that Protocol ON increases trust."
            }.to_string(),
        }
    }

    /// Generate reproducibility information
    fn generate_reproducibility_info() -> ReproducibilityInfo {
        ReproducibilityInfo {
            sim_version: env!("CARGO_PKG_VERSION").to_string(),
            rust_version: rustc_version(),
            git_commit: option_env!("GIT_COMMIT").unwrap_or("unknown").to_string(),
            generated_at: Utc::now(),
            requirements: vec![
                "Rust 1.70+".to_string(),
                "sim-core crate".to_string(),
                "sim-runner crate".to_string(),
            ],
        }
    }
}

/// POM validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomValidationReport {
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Report configuration
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub config: ReportConfig,
    /// POM validation suite results
    pub suite: PomValidationSuite,
    /// Summary statistics
    pub summary: PomSummary,
}

impl PomValidationReport {
    /// Convert to Markdown format
    #[must_use]
    pub fn to_markdown(&self) -> String {
        templates::pom_validation_template(self)
    }

    /// Convert to JSON format
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// POM summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomSummary {
    /// Total number of targets
    pub total_targets: usize,
    /// Number of passed targets
    pub passed: usize,
    /// Number of failed targets
    pub failed: usize,
    /// Overall pass rate
    pub pass_rate: f32,
    /// Number of P0 (critical) targets
    pub p0_targets: usize,
    /// Number of passed P0 targets
    pub p0_passed: usize,
    /// P0 pass rate
    pub p0_pass_rate: f32,
    /// Per-category summaries
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub category_summaries: Vec<CategorySummary>,
}

/// Per-category summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    /// Category name
    pub category: String,
    /// Total targets in category
    pub total: usize,
    /// Passed targets in category
    pub passed: usize,
    /// Pass rate for category
    pub pass_rate: f32,
}

/// GTT hypothesis testing report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisReport {
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Report configuration
    pub config: ReportConfig,
    /// Hypothesis test results
    pub result: GttHypothesisResult,
    /// Human-readable interpretation
    pub interpretation: HypothesisInterpretation,
}

impl HypothesisReport {
    /// Convert to Markdown format
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        templates::hypothesis_template(self)
    }

    /// Convert to JSON format
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Hypothesis interpretation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisInterpretation {
    /// Effect magnitude description
    pub effect_magnitude: String,
    /// Statistical significance description
    pub significance: String,
    /// Practical significance description
    pub practical_significance: String,
    /// Overall conclusion
    pub conclusion: String,
}

/// Full simulation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSimulationReport {
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Report configuration
    pub config: ReportConfig,
    /// Simulation metadata
    pub metadata: SimulationMetadata,
    /// POM validation report (if available)
    pub pom_validation: Option<PomValidationReport>,
    /// Hypothesis test report (if available)
    pub hypothesis_test: Option<HypothesisReport>,
    /// Reproducibility information
    pub reproducibility: ReproducibilityInfo,
}

impl FullSimulationReport {
    /// Convert to comprehensive Markdown report
    /// # Errors
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        templates::full_report_template(self)
    }

    /// Convert to JSON format
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Save report to directory
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn save_to_dir(&self, dir: &std::path::Path) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(dir)?;

        // Save Markdown report
        let md_path = dir.join("report.md");
        std::fs::write(&md_path, self.to_markdown())?;

        // Save JSON report
        let json_path = dir.join("report.json");
        if let Ok(json) = self.to_json() {
            std::fs::write(&json_path, json)?;
        }

        Ok(())
    }
}

/// Simulation metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimulationMetadata {
    /// Simulation ID
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub simulation_id: String,
    /// Simulation name/description
    pub description: String,
    /// Start time
    pub start_time: Option<DateTime<Utc>>,
    /// End time
    pub end_time: Option<DateTime<Utc>>,
    /// Number of agents
    pub agent_count: u32,
    /// Number of ticks
    pub max_ticks: u64,
    /// Random seed
    pub seed: u64,
    /// Stage (0-5)
    pub stage: u8,
    /// Verification mode
    pub verification_mode: String,
    /// Additional parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Determinism certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismCertificate {
    /// Certificate generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Report configuration
    pub config: ReportConfig,
    /// Seeds used for testing
    pub seeds: Vec<u64>,
    /// Checksums for each run
    pub checksums: Vec<String>,
    /// Whether all checksums match
    pub verified: bool,
    /// SIM-004 compliance status
    pub sim_004_compliant: bool,
}

impl DeterminismCertificate {
    /// Convert to Markdown format
    #[must_use]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub fn to_markdown(&self) -> String {
        templates::determinism_certificate_template(self)
    }
}

/// Reproducibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibilityInfo {
    /// sim-report version
    pub sim_version: String,
    /// Rust compiler version
    pub rust_version: String,
    /// Git commit hash
    pub git_commit: String,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Requirements for reproduction
    pub requirements: Vec<String>,
}

/// Get Rust compiler version
fn rustc_version() -> String {
    // In a real implementation, this would use rustc_version crate
    // For now, return a placeholder
    "1.70+".to_string()
}

/// Report type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    /// POM validation report
    PomValidation,
    /// GTT hypothesis report
    HypothesisTest,
    /// Full comprehensive report
    FullReport,
    /// Determinism certificate
    DeterminismCertificate,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_config_default() {
        let config = ReportConfig::default();
        assert!(!config.title.is_empty());
        assert!(!config.author.is_empty());
        assert!(config.confidence_level > 0.0 && config.confidence_level < 1.0);
    }

    #[test]
    fn test_report_generator_new() {
        let config = ReportConfig::default();
        let _generator = ReportGenerator::new(config);
        // Just verify it compiles and creates
    }

    #[test]
    fn test_full_report_save() {
        let config = ReportConfig::default();
        let generator = ReportGenerator::new(config);
        let metadata = SimulationMetadata {
            simulation_id: "test-123".to_string(),
            description: "Test simulation".to_string(),
            ..Default::default()
        };

        let report = generator.generate_full_report(None, None, metadata);

        let temp_dir = tempfile::tempdir().unwrap();
        let result = report.save_to_dir(temp_dir.path());
        assert!(result.is_ok());

        // Check files were created
        assert!(temp_dir.path().join("report.md").exists());
        assert!(temp_dir.path().join("report.json").exists());
    }

    #[test]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn test_generate_pom_report() {
        let generator = ReportGenerator::new(ReportConfig::default());
        let suite = PomValidationSuite {
            categories: std::collections::HashMap::new(),
            overall_score: 85.0,
            passed_count: 8,
            total_count: 10,
        };
        let report = generator.generate_pom_report(&suite);
        assert_eq!(report.summary.total_targets, 10);
        assert_eq!(report.summary.passed, 8);
        let md = report.to_markdown();
        assert!(!md.is_empty());
        let json = report.to_json().unwrap();
        assert!(!json.is_empty());
    }

    #[test]
    fn test_generate_hypothesis_report() {
        let generator = ReportGenerator::new(ReportConfig::default());
        let result = GttHypothesisResult {
            n_runs: 100,
            seed: 42,
            protocol_on: ConditionResults {
                n: 100,
                mean: 0.8,
                std: 0.1,
                values: vec![],
            },
            protocol_off: ConditionResults {
                n: 100,
                mean: 0.6,
                std: 0.1,
                values: vec![],
            },
            effect_size: 0.5,
            effect_size_ci_lower: 0.3,
            effect_size_ci_upper: 0.7,
            effect_size_interpretation: "medium".to_string(),
            t_statistic: 3.0,
            p_value: 0.01,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            hypothesis_supported: true,
            confidence_level: 0.95,
        };
        let report = generator.generate_hypothesis_report(&result);
        let md = report.to_markdown();
        assert!(!md.is_empty());
        let json = report.to_json().unwrap();
        assert!(!json.is_empty());
    }

    #[test]
    fn test_generate_determinism_certificate() {
        let generator = ReportGenerator::new(ReportConfig::default());
        let cert = generator.generate_determinism_certificate(
            &[1, 2, 3],
            &[
                "abcd1234efgh5678".to_string(),
                "abcd1234efgh5678".to_string(),
            ],
        );
        assert!(cert.verified);
        assert!(cert.sim_004_compliant);
        let md = cert.to_markdown();
        assert!(!md.is_empty());
    }

    #[test]
    fn test_generate_determinism_certificate_not_verified() {
        let generator = ReportGenerator::new(ReportConfig::default());
        let cert = generator
            .generate_determinism_certificate(&[1, 2], &["abc".to_string(), "def".to_string()]);
        assert!(!cert.verified);
        assert!(!cert.sim_004_compliant);
    }

    #[test]
    fn test_rustc_version() {
        let version = rustc_version();
        assert!(!version.is_empty());
    }
}
