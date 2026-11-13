// TAG: surface=simulation owner=research-team rule=SIM-001
//! Identity metrics, theft statistics, and POM validation

use super::*;

/// Calculate identity theft metrics across elapsed simulation ticks
pub fn calculate_theft_metrics(stats: &IdentityTheftStats, ticks_elapsed: u64) -> TheftMetrics {
    let years = ticks_elapsed as f32 / 1000.0; // Assume 1000 ticks/year

    TheftMetrics {
        annual_theft_rate: if years > 0.0 {
            stats.successful as f32 / years
        } else {
            0.0
        },
        success_rate: if stats.attempts > 0 {
            stats.successful as f32 / stats.attempts as f32
        } else {
            0.0
        },
        protocol_effectiveness: stats.protocol_protection_rate,
        avg_recovery_time: stats.avg_recovery_time,
        theft_rate: stats.current_theft_rate,
    }
}

/// Compare Protocol ON vs OFF
#[must_use]
pub fn compare_protection(
    with_protocol: &IdentityTheftStats,
    without_protocol: &IdentityTheftStats,
    population: usize,
) -> f32 {
    let rate_with = with_protocol.compute_theft_rate(population);
    let rate_without = without_protocol.compute_theft_rate(population);

    if rate_without == 0.0 {
        return 0.0;
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    (rate_without - rate_with) / rate_without // Protection percentage
}

/// Theft metrics summary
#[derive(Debug, Clone)]
pub struct TheftMetrics {
    /// Annual theft rate (per agent-year)
    pub annual_theft_rate: f32,
    /// Success rate of theft attempts
    pub success_rate: f32,
    /// Effectiveness of Protocol in preventing theft
    pub protocol_effectiveness: f32,
    /// Average recovery time in ticks
    pub avg_recovery_time: f32,
    /// Current theft rate (% of population)
    pub theft_rate: f32,
}

/// Identity metrics for POM validation
#[derive(Debug, Clone)]
pub struct IdentityMetrics {
    /// Total theft count
    pub theft_count: u32,
    /// Theft rate as percentage
    pub theft_rate: f32,
    /// Average recovery time in ticks
    pub avg_recovery_time: f32,
    /// Protocol protection rate (reduction percentage)
    pub protocol_protection_rate: f32,
    /// Thefts by method
    pub thefts_by_method: HashMap<TheftMethod, u32>,
}

impl IdentityMetrics {
    /// Compute theft rate
    #[must_use]
    pub fn compute_theft_rate(&self, population: usize) -> f32 {
        if population == 0 {
            return 0.0;
        }
        self.theft_count as f32 / population as f32
    }

    /// Compare Protocol ON vs OFF
    #[must_use]
    pub fn compare_protection(
        &self,
        with_protocol: &IdentityTheftStats,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        without_protocol: &IdentityTheftStats,
    ) -> f32 {
        let rate_with = with_protocol.current_theft_rate;
        let rate_without = without_protocol.current_theft_rate;

        if rate_without == 0.0 {
            return 0.0;
        }

        (rate_without - rate_with) / rate_without
    }

    /// Validate POM targets
    /// - Theft rate: 5-10%
    /// - Recovery time: 100-500 ticks
    /// - Protocol protection: 20-40% reduction
    #[must_use]
    pub fn validate_pom_targets(&self, population: usize) -> PomValidation {
        let theft_rate = self.compute_theft_rate(population);

        PomValidation {
            theft_rate_valid: (0.05..=0.10).contains(&theft_rate),
            recovery_time_valid: (100.0..=500.0).contains(&self.avg_recovery_time),
            protection_valid: (0.20..=0.40).contains(&self.protocol_protection_rate),
            theft_rate,
        }
    }
}

/// POM validation results
#[derive(Debug, Clone)]
pub struct PomValidation {
    /// Whether theft rate is within target (5-10%)
    pub theft_rate_valid: bool,
    /// Whether recovery time is within target (100-500 ticks)
    pub recovery_time_valid: bool,
    /// Whether protocol protection is within target (20-40%)
    pub protection_valid: bool,
    /// Actual theft rate
    pub theft_rate: f32,
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

// P0-SEC-011: Mock hash helper for simulation identifiers (not cryptographic)
// Uses DefaultHasher for deterministic mock data generation only
pub(crate) mod mock_hash {
    pub fn compute(data: impl AsRef<[u8]>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.as_ref().hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}
