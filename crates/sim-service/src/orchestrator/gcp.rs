//! GCP Cloud Run Orchestration (§23.10.5)
//!
//! Dispatches simulation jobs to Cloud Run and manages resources.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{info, warn};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Errors that can occur during GCP operations
#[derive(Debug, Error, Clone)]
pub enum GcpError {
    #[error("GCP API error: {message}")]
    /// `ApiError`
    ApiError {
        /// Error message
        message: String,
    },

    #[error("Job creation failed: {reason}")]
    /// `JobCreationFailed`
    JobCreationFailed {
        /// Failure reason
        reason: String,
    },

    #[error("Invalid resource specification: {details}")]
    /// `InvalidResourceSpec`
    InvalidResourceSpec {
        /// Error details
        details: String,
    },

    #[error("Environment variable not set: {var}")]
    /// `EnvVarNotSet`
    EnvVarNotSet {
        /// Environment variable name
        var: String,
    },

    #[error("Mock mode - operation not available")]
    /// `MockMode`
    MockMode,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Resource requirements for Cloud Run jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// cpu
    pub cpu: String,
    /// memory
    pub memory: String,
    /// timeout secs
    pub timeout_secs: u64,
}

impl ResourceRequirements {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create resource requirements based on agent count
    #[must_use]
    pub fn for_agent_count(agent_count: u64) -> Self {
        let (cpu, memory) = match agent_count {
            0..=1_000 => ("1".to_string(), "512Mi".to_string()),
            1_001..=10_000 => ("4".to_string(), "2Gi".to_string()),
            10_001..=100_000 => ("16".to_string(), "8Gi".to_string()),
            _ => ("32".to_string(), "32Gi".to_string()),
        };

        Self {
            cpu,
            memory,
            timeout_secs: 28_800, // 8 hours max
        }
    }
}

/// Cloud Run job specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    /// image
    pub image: String,
    /// env
    pub env: HashMap<String, String>,
    /// resources
    pub resources: ResourceRequirements,
    /// args
    pub args: Vec<String>,
}

/// Cloud Run execution reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    /// name
    pub name: String,
    /// job id
    pub job_id: String,
    /// status
    pub status: ExecutionStatus,
    /// create time
    pub create_time: chrono::DateTime<chrono::Utc>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionStatus {
    /// Pending
    Pending,
    /// Running
    Running,
    /// Succeeded
    Succeeded,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
    /// Unknown
    Unknown,
}

/// Trait for GCP orchestration operations
///
/// This trait allows for mock implementations in tests and real implementations in production.
#[async_trait::async_trait]
pub trait GcpOrchestrator: Send + Sync {
    /// Dispatch a new simulation job to Cloud Run
    async fn dispatch_job(&self, job_id: &str, spec: &JobSpec) -> Result<Execution, GcpError>;

    /// Get execution status
    async fn get_execution(&self, execution_name: &str) -> Result<Execution, GcpError>;

    /// Cancel a running execution
    async fn cancel_execution(&self, execution_name: &str) -> Result<(), GcpError>;

    /// List recent executions for a job
    async fn list_executions(&self, job_id: &str) -> Result<Vec<Execution>, GcpError>;
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Real GCP Cloud Run orchestrator
#[allow(dead_code)]
#[derive(Debug)]
pub struct CloudRunOrchestrator {
    project_id: String,
    region: String,
    client: reqwest::Client,
}

impl CloudRunOrchestrator {
    /// Create a new Cloud Run orchestrator from environment
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_env() -> Result<Self, GcpError> {
        let project_id = std::env::var("GCP_PROJECT_ID").map_err(|_| GcpError::EnvVarNotSet {
            var: "GCP_PROJECT_ID".to_string(),
        })?;

        let region = std::env::var("GCP_REGION").unwrap_or_else(|_| "us-central1".to_string());

        info!(
            "Creating CloudRunOrchestrator for project {} in region {}",
            project_id, region
        );

        Ok(Self {
            project_id,
            region,
            client: crate::http_client::shared_client().clone(),
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create with explicit parameters
    #[must_use]
    pub fn new(project_id: String, region: String) -> Self {
        Self {
            project_id,
            region,
            client: crate::http_client::shared_client().clone(),
        }
    }

    /// Get the environment (dev/staging/prod)
    #[allow(dead_code)]
    fn environment(&self) -> String {
        std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string())
    }
}

#[async_trait::async_trait]
impl GcpOrchestrator for CloudRunOrchestrator {
    async fn dispatch_job(&self, job_id: &str, _spec: &JobSpec) -> Result<Execution, GcpError> {
        // In a real implementation, this would call the Cloud Run Jobs API
        // For now, we return a mock execution
        warn!("CloudRunOrchestrator::dispatch_job is a stub implementation");

        Ok(Execution {
            name: format!(
                "projects/{}/locations/{}/jobs/sim-runner-{}/executions/{}",
                self.project_id,
                self.region,
                job_id,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                uuid::Uuid::new_v4()
            ),
            job_id: job_id.to_string(),
            status: ExecutionStatus::Pending,
            create_time: chrono::Utc::now(),
        })
    }

    async fn get_execution(&self, execution_name: &str) -> Result<Execution, GcpError> {
        warn!("CloudRunOrchestrator::get_execution is a stub implementation");

        Ok(Execution {
            name: execution_name.to_string(),
            job_id: "unknown".to_string(),
            status: ExecutionStatus::Running,
            create_time: chrono::Utc::now(),
        })
    }

    async fn cancel_execution(&self, _execution_name: &str) -> Result<(), GcpError> {
        warn!("CloudRunOrchestrator::cancel_execution is a stub implementation");
        Ok(())
    }

    async fn list_executions(&self, _job_id: &str) -> Result<Vec<Execution>, GcpError> {
        warn!("CloudRunOrchestrator::list_executions is a stub implementation");
        Ok(vec![])
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Mock orchestrator for testing
#[derive(Debug, Default)]
pub struct MockOrchestrator {
    executions: std::sync::Arc<tokio::sync::RwLock<Vec<Execution>>>,
}

impl MockOrchestrator {
    #[must_use]
    /// new
    pub fn new() -> Self {
        Self {
            executions: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[async_trait::async_trait]
impl GcpOrchestrator for MockOrchestrator {
    async fn dispatch_job(&self, job_id: &str, _spec: &JobSpec) -> Result<Execution, GcpError> {
        let execution = Execution {
            name: format!("mock-execution-{}", uuid::Uuid::new_v4()),
            job_id: job_id.to_string(),
            status: ExecutionStatus::Pending,
            create_time: chrono::Utc::now(),
        };

        self.executions.write().await.push(execution.clone());
        info!(
            "MockOrchestrator: Dispatched job {} -> execution {}",
            job_id, execution.name
        );

        Ok(execution)
    }

    async fn get_execution(&self, execution_name: &str) -> Result<Execution, GcpError> {
        let executions = self.executions.read().await;
        executions
            .iter()
            .find(|e| e.name == execution_name)
            .cloned()
            .ok_or_else(|| GcpError::ApiError {
                message: format!("Execution {execution_name} not found"),
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            })
    }

    async fn cancel_execution(&self, execution_name: &str) -> Result<(), GcpError> {
        let mut executions = self.executions.write().await;
        if let Some(execution) = executions.iter_mut().find(|e| e.name == execution_name) {
            execution.status = ExecutionStatus::Cancelled;
            info!("MockOrchestrator: Cancelled execution {}", execution_name);
            Ok(())
        } else {
            Err(GcpError::ApiError {
                message: format!("Execution {execution_name} not found"),
            })
        }
    }

    /// # Errors
    ///
    /// Returns an error if the operation fails.
    async fn list_executions(&self, job_id: &str) -> Result<Vec<Execution>, GcpError> {
        let executions = self.executions.read().await;
        Ok(executions
            .iter()
            .filter(|e| e.job_id == job_id)
            .cloned()
            .collect())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create an orchestrator based on environment
///
/// - If `MOCK_GCP=true`, returns a `MockOrchestrator`
/// - Otherwise, returns a `CloudRunOrchestrator`
/// # Errors
///
/// Returns an error if the operation fails.
pub fn create_orchestrator() -> Result<Box<dyn GcpOrchestrator>, GcpError> {
    if std::env::var("MOCK_GCP").unwrap_or_default() == "true" {
        info!("Using MockOrchestrator (MOCK_GCP=true)");
        Ok(Box::new(MockOrchestrator::new()))
    } else {
        info!("Using CloudRunOrchestrator");
        Ok(Box::new(CloudRunOrchestrator::from_env()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_requirements() {
        let res = ResourceRequirements::for_agent_count(500);
        assert_eq!(res.cpu, "1");
        assert_eq!(res.memory, "512Mi");

        let res = ResourceRequirements::for_agent_count(5_000);
        assert_eq!(res.cpu, "4");
        assert_eq!(res.memory, "2Gi");

        let res = ResourceRequirements::for_agent_count(50_000);
        assert_eq!(res.cpu, "16");
        assert_eq!(res.memory, "8Gi");

        let res = ResourceRequirements::for_agent_count(1_000_000);
        assert_eq!(res.cpu, "32");
        assert_eq!(res.memory, "32Gi");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_mock_orchestrator() {
        let orchestrator = MockOrchestrator::new();

        // Dispatch a job
        let spec = JobSpec {
            image: "gcr.io/test/sim-runner".to_string(),
            env: HashMap::new(),
            resources: ResourceRequirements::for_agent_count(1000),
            args: vec![],
        };

        let execution = orchestrator.dispatch_job("test-job", &spec).await.unwrap();
        assert_eq!(execution.job_id, "test-job");
        assert_eq!(execution.status, ExecutionStatus::Pending);

        // Get execution
        let fetched = orchestrator.get_execution(&execution.name).await.unwrap();
        assert_eq!(fetched.name, execution.name);

        // List executions
        let executions = orchestrator.list_executions("test-job").await.unwrap();
        assert_eq!(executions.len(), 1);

        // Cancel execution
        orchestrator
            .cancel_execution(&execution.name)
            .await
            .unwrap();
        let cancelled = orchestrator.get_execution(&execution.name).await.unwrap();
        assert_eq!(cancelled.status, ExecutionStatus::Cancelled);
    }
}
