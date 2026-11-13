// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Statistical Power Analysis (P1-VAL-1)
//!
//! Ensures experiments have sufficient sample sizes for statistically significant results.
//! Provides power calculations, sample size estimation, and effect size analysis.

pub mod calculator;
pub mod effect_size;
pub mod sample_size;
pub mod types;

pub use calculator::PowerCalculator;
pub use effect_size::{CohenD, EffectSize, EtaSquared, OddsRatio, PearsonsR};
pub use sample_size::SampleSizeCalculator;
pub use types::*;
