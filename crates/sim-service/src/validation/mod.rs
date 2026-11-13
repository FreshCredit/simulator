// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Validation Module (P1-VAL)
//!
//! Provides statistical validation, power analysis, and sensitivity testing
//! for simulation experiments.

pub mod cross_validation;
pub mod meta;
pub mod power;
pub mod reproducibility;
pub mod sensitivity;

pub use cross_validation::{
    fold::{BaselineModel, CrossValidator},
    metrics::{ClassificationMetrics, RegressionMetrics},
    splitter::{GroupKFold, StratifiedKFold, TimeSeriesSplit, TrainTestSplit},
    types::*,
};

pub use meta::{analysis::MetaAnalyzer, forest::ForestPlot, funnel::FunnelPlot, types::*};

pub use reproducibility::{stability::StabilityAnalyzer, types::*, variance::VarianceEstimator};

pub use power::{
    calculator::PowerCalculator,
    effect_size::{CohenD, EffectSize, EffectSizeConverter, EffectSizeData, HedgesG, PearsonsR},
    sample_size::SampleSizeCalculator,
    types::*,
};

pub use sensitivity::{
    local::LocalSensitivityAnalyzer, morris::MorrisAnalyzer, sobol::SobolAnalyzer,
    tornado::TornadoDiagram, types::*,
};
