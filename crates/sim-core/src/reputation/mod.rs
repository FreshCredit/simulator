// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Reputation analysis module: Volatility tracking and damage categorization
//!
//! SIM-REPUTATION-001: Temporal reputation dynamics analysis
//!
//! This module provides tools for tracking reputation volatility and
//! categorizing reputation damage events using confusion matrix methodology.
//!
//! ## Key Concepts
//!
//! - **Volatility**: Rolling window variance of trust scores (GARCH-inspired)
//! - **Confusion Matrix**: Categorization of damage events (TP/FP/FN/TN)
//! - **Population Metrics**: Aggregate statistics across agent population
//!
//! ## Academic Foundation
//!
//! - Engle (1982): ARCH model for volatility
//! - Bollerslev (1986): GARCH extension
//! - Powers (2011): Confusion matrix evaluation metrics

pub mod confusion;
pub mod volatility;

pub use confusion::{
    DamageEvent, DamageEventType, EraComparison, JudgmentSource, ReputationConfusionMatrix,
};
pub use volatility::{PopulationVolatilityMetrics, ReputationVolatilityTracker};
