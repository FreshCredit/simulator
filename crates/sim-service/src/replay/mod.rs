// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Simulation Replay & Diff System (P1-SIM-6)
//!
//! Provides capabilities to replay simulation runs and compare
//! different runs for analysis and debugging.

pub mod diff;
pub mod player;
pub mod storage;
pub mod types;

pub use diff::{DiffEngine, RunDiff};
pub use player::ReplayPlayer;
pub use storage::ReplayStorage;
pub use types::PlaybackState;
pub use types::{ReplayConfig, ReplayFrame, ReplaySession};
