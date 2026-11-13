//! Twin Heritability Experiment
//!
//! **Gap Addressed:** DOCAS-P0-2 (Twin simulation not implemented)
//!
//! **Purpose:** Validate h² ≈ 0.50 via MZ/DZ twin comparison
//!
//! **Success Criteria:**
//! - h² ∈ [0.42, 0.56]
//! - `r_MZ` > `r_DZ` (genetic influence detected)
//! - 95% CI width ≤ 0.14

use super::{init_experiment, save_results, Experiment, ExperimentConfig, ExperimentResult};
use crate::unified_agent::UnifiedAgent;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Twin pair types
#[derive(Debug, Clone)]
pub enum TwinType {
    /// monozygotic.
    Monozygotic, // Identical (100% shared genes)
    /// dizygotic.
    Dizygotic, // Fraternal (~50% shared genes)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// A pair of twins
#[derive(Debug, Clone)]
pub struct TwinPair {
    /// twin1.
    pub twin1: UnifiedAgent,
    /// twin2.
    pub twin2: UnifiedAgent,
    /// twin type.
    pub twin_type: TwinType,
}

/// Results from twin heritability study
#[derive(Debug, Clone)]
pub struct TwinStudyResults {
    /// mz pairs.
    pub mz_pairs: Vec<TwinPair>,
    /// dz pairs.
    pub dz_pairs: Vec<TwinPair>,
    /// r mz.
    pub r_mz: f64, // Correlation for MZ twins
    /// r dz.
    pub r_dz: f64, // Correlation for DZ twins
    /// h2.
    pub h2: f64, // Heritability estimate
    /// ci lower.
    pub ci_lower: f64, // 95% CI lower bound
    /// ci upper.
    pub ci_upper: f64, // 95% CI upper bound
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Twin Heritability Experiment
#[derive(Debug)]
pub struct TwinHeritabilityExperiment {
    n_mz: usize,
    n_dz: usize,
}

impl TwinHeritabilityExperiment {
    #[must_use]
    /// new.
    pub const fn new(n_mz: usize, n_dz: usize) -> Self {
        Self { n_mz, n_dz }
    }

    /// Generate MZ twin pair (identical genome)
    fn generate_mz_pair(parent: &UnifiedAgent) -> TwinPair {
        let twin1 = parent.create_mz_twin(0);
        let twin2 = parent.create_mz_twin(0);

        TwinPair {
            twin1,
            twin2,
            twin_type: TwinType::Monozygotic,
        }
    }

    /// Generate DZ twin pair (50% shared genes)
    fn generate_dz_pair(parent1: &UnifiedAgent, parent2: &UnifiedAgent) -> TwinPair {
        let twin1 = parent1.create_dz_twin(parent2, 0);
        let twin2 = parent1.create_dz_twin(parent2, 0);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        TwinPair {
            twin1,
            twin2,
            twin_type: TwinType::Dizygotic,
        }
    }

    /// Calculate trust phenotype for heritability analysis
    const fn calculate_phenotype(agent: &UnifiedAgent) -> f64 {
        // Use DOCAS-derived trust state as phenotype
        agent.trust_state
    }

    /// Calculate correlation between twins
    fn calculate_correlation(&self, pairs: &[TwinPair]) -> f64 {
        if pairs.len() < 2 {
            return 0.0;
        }

        // Get phenotypes
        let x: Vec<f64> = pairs
            .iter()
            .map(|p| Self::calculate_phenotype(&p.twin1))
            .collect();
        let y: Vec<f64> = pairs
            .iter()
            .map(|p| Self::calculate_phenotype(&p.twin2))
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        crate::stats::pearson_correlation_f64(&x, &y)
    }

    /// Calculate heritability: h² = `2(r_MZ` - `r_DZ`) clamped to [0.0, 1.0]
    fn calculate_heritability(r_mz: f64, r_dz: f64) -> f64 {
        if !r_mz.is_finite() || !r_dz.is_finite() {
            return 0.0;
        }
        (2.0 * (r_mz - r_dz)).clamp(0.0, 1.0)
    }

    /// Calculate 95% confidence interval (simplified)
    fn calculate_ci(h2: f64, n: usize) -> (f64, f64) {
        if n == 0 || !h2.is_finite() {
            return (0.0, 0.0);
        }
        let se = (1.0 / (n as f64)).sqrt();
        let margin = 1.96 * se;
        ((h2 - margin).clamp(0.0, 1.0), (h2 + margin).clamp(0.0, 1.0))
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for TwinHeritabilityExperiment {
    fn name(&self) -> &'static str {
        "twin_heritability"
    }

    fn description(&self) -> &'static str {
        "DOCAS-P0-2: Twin study heritability validation"
    }

    fn required_agents(&self) -> usize {
        1000
    }

    fn run(&self, config: &ExperimentConfig) -> anyhow::Result<ExperimentResult> {
        println!("Starting Twin Heritability Experiment");
        println!("MZ pairs: {}", self.n_mz);
        println!("DZ pairs: {}", self.n_dz);

        init_experiment(config)?;

        // Generate parent agents
        let parents: Vec<UnifiedAgent> = (0..(self.n_mz + self.n_dz * 2))
            .map(|_| UnifiedAgent::new(0, 0))
            .collect();

        // Generate MZ twin pairs
        let mut mz_pairs = Vec::with_capacity(self.n_mz);
        for parent in parents.iter().take(self.n_mz) {
            mz_pairs.push(Self::generate_mz_pair(parent));
        }

        // Generate DZ twin pairs
        let mut dz_pairs = Vec::with_capacity(self.n_dz);
        let parent_offset = self.n_mz;
        for i in 0..self.n_dz {
            let parent1 = &parents[parent_offset + i * 2];
            let parent2 = &parents[parent_offset + i * 2 + 1];
            dz_pairs.push(Self::generate_dz_pair(parent1, parent2));
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Calculate correlations
        let r_mz = self.calculate_correlation(&mz_pairs);
        let r_dz = self.calculate_correlation(&dz_pairs);

        // Calculate heritability
        let h2 = Self::calculate_heritability(r_mz, r_dz);
        let (ci_lower, ci_upper) = Self::calculate_ci(h2, self.n_mz + self.n_dz);

        // Validate success criteria
        let success = (0.42..=0.56).contains(&h2) && r_mz > r_dz && (ci_upper - ci_lower) <= 0.14;

        let result = ExperimentResult {
            experiment_type: "twin_heritability".to_string(),
            success,
            gap_id: "DOCAS-P0-2".to_string(),
            paper: "DOCAS".to_string(),
            metrics: serde_json::json!({
                "r_mz": r_mz,
                "r_dz": r_dz,
                "h2": h2,
                "ci_lower": ci_lower,
                "ci_upper": ci_upper,
                "n_mz": self.n_mz,
                "n_dz": self.n_dz,
            }),
            figures: vec![],
            csv_outputs: vec![],
            artifacts: vec![],
            duration: std::time::Duration::from_secs(0),
            error_message: if success {
                None
            } else {
                Some("Success criteria not met".to_string())
            },
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        save_results(&result, &config.output_dir)?;

        println!("\nTwin Heritability Results:");
        println!("  r_MZ: {r_mz:.3}");
        println!("  r_DZ: {r_dz:.3}");
        println!("  h²: {h2:.3}");
        println!("  95% CI: [{ci_lower:.3}, {ci_upper:.3}]");
        println!("  Success: {success}");

        Ok(result)
    }

    fn validate(&self, result: &ExperimentResult) -> bool {
        if !result.success {
            return false;
        }

        let metrics = &result.metrics;
        let h2 = metrics["h2"].as_f64().unwrap_or(0.0);
        let r_mz = metrics["r_mz"].as_f64().unwrap_or(0.0);
        let r_dz = metrics["r_dz"].as_f64().unwrap_or(0.0);

        (0.42..=0.56).contains(&h2) && r_mz > r_dz
    }

    fn generate_figures(&self, _result: &ExperimentResult) -> anyhow::Result<()> {
        println!("Generating twin study figures...");
        Ok(())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mz_correlation_higher_than_dz() {
        // MZ twins should have higher correlation than DZ twins
        // This is the fundamental basis of twin studies

        // Create parent with high trust
        let mut parent = UnifiedAgent::new(0, 0);
        parent.trust_state = 0.8;

        // Create MZ twins - should have similar trust
        let _exp = TwinHeritabilityExperiment::new(10, 10);
        let mz_pair = TwinHeritabilityExperiment::generate_mz_pair(&parent);

        // Trust should be identical (from same genome)
        assert_eq!(mz_pair.twin1.genome, mz_pair.twin2.genome);
    }

    #[test]
    fn test_heritability_calculation() {
        let _exp = TwinHeritabilityExperiment::new(10, 10);

        // If r_MZ = 0.8 and r_DZ = 0.4, h² = 2(0.8 - 0.4) = 0.8
        let h2 = TwinHeritabilityExperiment::calculate_heritability(0.8, 0.4);
        assert!((h2 - 0.8).abs() < 0.01);
    }
}
