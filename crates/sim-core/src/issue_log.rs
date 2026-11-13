// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Issue Log System: Pain + Reflection = Progress
//!
//! Per Ray Dalio's principle, this system tracks issues, their resolution,
//! and lessons learned to drive continuous improvement.
//!
//! SIM-001: Isolation verified - no product dependencies

/// issue module.
pub mod issue;
pub use issue::{Issue, IssueCategory, IssueSeverity, IssueStatus};

/// lessons module.
pub mod lessons;
pub use lessons::{Lesson, RecurringPattern};

/// log module.
pub mod log;
pub use log::{IssueLog, IssueLogError, IssueLogStats};

#[cfg(test)]
mod tests;
