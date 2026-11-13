// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! GCP Orchestration Module
//!
//! Manages Cloud Run job dispatch, GCS artifact storage, and batch job orchestration.

pub mod batch;
pub mod batch_executor;
pub mod gcp;
pub mod storage;

pub use batch::{
    BatchJob, BatchJobStatus, BatchOrchestrator, BatchProgress, BatchResults, JobPriority,
    JobScheduler, OrchestratorCommand, WorkerCapabilities, WorkerNode, WorkerStatus,
};
pub use gcp::{GcpError, GcpOrchestrator, ResourceRequirements};
pub use storage::{ArtifactStorage, StorageError};
