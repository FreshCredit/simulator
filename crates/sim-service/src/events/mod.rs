// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! External Event Injection System (P1-SIM-5)
//!
//! Provides APIs for injecting external events into running simulations,
//! enabling real-time interaction and scenario-based testing.

pub mod injector;
pub mod queue;
pub mod types;
pub mod validation;

pub use injector::{EventInjector, InjectionResult};
pub use queue::EventQueue;
pub use types::{EventOutcome, EventPriority, EventTarget, ExternalEvent};
pub use validation::{EventValidator, ValidationResult};
