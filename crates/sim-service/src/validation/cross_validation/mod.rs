// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Cross-Validation Framework (P1-VAL-3)
//!
//! K-fold cross-validation for assessing model performance and generalization.
//! Supports stratified k-fold, time series split, and group k-fold.

pub mod fold;
pub mod metrics;
pub mod splitter;
pub mod types;

pub use fold::{BaselineModel, CrossValidator};
pub use metrics::{ClassificationMetrics, RegressionMetrics};
pub use splitter::{GroupKFold, StratifiedKFold, TimeSeriesSplit, TrainTestSplit};
pub use types::*;
