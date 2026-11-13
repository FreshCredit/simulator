// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Persistence layer for experiment results and data storage
//!
//! Provides abstractions for storing and retrieving simulation data
//! with support for versioning and multiple storage backends.

pub mod result_store;

pub use result_store::{ResultStore, ResultStoreError, ResultVersion, StoredResult};
