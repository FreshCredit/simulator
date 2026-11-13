// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Authentication and Authorization Module
//!
//! Provides RBAC enforcement and runtime isolation guards.
//!
//! ## Submodules
//!
//! - `rbac`: Role-based access control (§10.4)
//! - `guards`: Runtime isolation guards (§19.4.2)

pub mod guards;
pub mod rbac;

// Re-export commonly used items
pub use guards::{GuardError, IsolationGuards};
pub use rbac::{check_job_limits, check_remote_allowed, RbacError, RoleLimits, UserRole};
