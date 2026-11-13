//! GCS Artifact Storage (§23.10.5)
//!
//! Manages upload and download of simulation artifacts to/from Google Cloud Storage.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use tokio::task;
use tracing::{error, info};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Errors that can occur during storage operations
#[derive(Debug, Error, Clone)]
pub enum StorageError {
    #[error("Upload failed: {message}")]
    /// `UploadFailed`
    UploadFailed {
        /// Error message
        message: String,
    },

    #[error("Download failed: {message}")]
    /// `DownloadFailed`
    DownloadFailed {
        /// Error message
        message: String,
    },

    #[error("Object not found: {path}")]
    /// `NotFound`
    NotFound {
        /// Path that was not found
        path: String,
    },

    #[error("Serialization error: {message}")]
    /// `SerializationError`
    SerializationError {
        /// Error message
        message: String,
    },

    #[error("Environment variable not set: {var}")]
    /// `EnvVarNotSet`
    EnvVarNotSet {
        /// Environment variable name
        var: String,
    },

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[error("Invalid bucket name: {name}")]
    /// `InvalidBucketName`
    InvalidBucketName {
        /// Bucket name
        name: String,
    },

    #[error("Authentication failed: {message}")]
    /// `AuthenticationFailed`
    AuthenticationFailed {
        /// Error message
        message: String,
    },

    #[error("Bucket not found or not accessible: {bucket} - {message}")]
    /// `BucketNotFound`
    BucketNotFound {
        /// Bucket name
        bucket: String,
        /// Error message
        message: String,
    },

    #[error("Invalid path: {path} - {reason}")]
    /// `InvalidPath`
    InvalidPath {
        /// Path that was invalid
        path: String,
        /// Reason for invalidity
        reason: String,
    },

    #[error("API error: {message}")]
    /// `ApiError`
    ApiError {
        /// Error message
        message: String,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Artifact metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// job id
    pub job_id: String,
    /// filename
    pub filename: String,
    /// content type
    pub content_type: String,
    /// size bytes
    pub size_bytes: usize,
    /// created at
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Trait for storage operations
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    /// Upload an object to storage
    async fn upload_object(
        &self,
        path: &str,
        data: Vec<u8>,
        _content_type: &str,
    ) -> Result<String, StorageError>;

    /// Download an object from storage
    async fn download_object(&self, path: &str) -> Result<Vec<u8>, StorageError>;

    /// Check if an object exists
    async fn object_exists(&self, path: &str) -> Result<bool, StorageError>;

    /// List objects with a given prefix
    async fn list_objects(&self, prefix: &str) -> Result<Vec<String>, StorageError>;

    /// Delete an object
    async fn delete_object(&self, path: &str) -> Result<(), StorageError>;

    /// Get the URL for an object
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn object_url(&self, path: &str) -> Result<String, StorageError>;
}

/// GCS storage implementation
#[allow(dead_code)]
#[derive(Debug)]
pub struct GcsStorage {
    bucket: String,
    client: reqwest::Client,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl GcsStorage {
    /// Create a new GCS storage client from environment
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_env() -> Result<Self, StorageError> {
        let bucket = std::env::var("GCS_BUCKET").map_err(|_| StorageError::EnvVarNotSet {
            var: "GCS_BUCKET".to_string(),
        })?;

        Self::new(bucket)
    }

    /// Create with explicit bucket name
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new(bucket: String) -> Result<Self, StorageError> {
        if bucket.is_empty() {
            return Err(StorageError::InvalidBucketName { name: bucket });
        }

        info!("Creating GcsStorage for bucket: {}", bucket);

        Ok(Self {
            bucket,
            client: crate::http_client::shared_client().clone(),
        })
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Get the bucket name
    #[must_use]
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

#[async_trait::async_trait]
impl StorageBackend for GcsStorage {
    async fn upload_object(
        &self,
        _path: &str,
        _data: Vec<u8>,
        _content_type: &str,
    ) -> Result<String, StorageError> {
        // GCS support requires the 'gcp' feature flag
        // Compile with: --features gcp
        error!(
            error = "feature_disabled",
            "GcsStorage::upload_object called without 'gcp' feature enabled"
        );

        Err(StorageError::UploadFailed {
            message: "GCS storage not available - sim-service was compiled without 'gcp' feature. \
                 To enable GCS support, rebuild with: cargo build --features gcp. \
                 Alternatively, set USE_LOCAL_STORAGE=true to use local filesystem storage."
                .to_string(),
        })
    }

    async fn download_object(&self, _path: &str) -> Result<Vec<u8>, StorageError> {
        error!(
            error = "feature_disabled",
            "GcsStorage::download_object called without 'gcp' feature enabled"
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Err(StorageError::DownloadFailed {
            message: "GCS storage not available - compile with --features gcp".to_string(),
        })
    }

    async fn object_exists(&self, _path: &str) -> Result<bool, StorageError> {
        error!(
            error = "feature_disabled",
            "GcsStorage::object_exists called without 'gcp' feature enabled"
        );

        Ok(false)
    }

    async fn list_objects(&self, _prefix: &str) -> Result<Vec<String>, StorageError> {
        error!(
            error = "feature_disabled",
            "GcsStorage::list_objects called without 'gcp' feature enabled"
        );
        Ok(vec![])
    }

    async fn delete_object(&self, _path: &str) -> Result<(), StorageError> {
        error!(
            error = "feature_disabled",
            "GcsStorage::delete_object called without 'gcp' feature enabled"
        );
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn object_url(&self, path: &str) -> Result<String, StorageError> {
        // Validate path to prevent directory traversal
        if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
            return Err(StorageError::InvalidPath {
                path: path.to_string(),
                reason: "Path contains directory traversal components".to_string(),
            });
        }
        Ok(format!("gs://{}/{}", self.bucket, path))
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Local filesystem storage for development/testing
#[derive(Debug)]
pub struct LocalStorage {
    base_path: std::path::PathBuf,
}

impl LocalStorage {
    /// Create a new local storage at the given path
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let base_path = base_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_path).map_err(|e| StorageError::UploadFailed {
            message: e.to_string(),
        })?;

        info!("Created LocalStorage at {:?}", base_path);

        Ok(Self { base_path })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create in a temporary directory
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn temp() -> Result<Self, StorageError> {
        let temp_dir = std::env::temp_dir().join(format!("sim-service-{}", uuid::Uuid::new_v4()));
        Self::new(temp_dir)
    }

    /// Get the full path for an object
    /// P1-SEC-008: Validates path to prevent directory traversal
    fn full_path(&self, path: &str) -> Result<std::path::PathBuf, StorageError> {
        // Reject paths containing .. or leading /
        if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
            return Err(StorageError::InvalidPath {
                path: path.to_string(),
                reason: "Path contains invalid components".to_string(),
            });
        }

        let full = self.base_path.join(path);

        // Canonicalize base_path for the security boundary.
        // On macOS /tmp symlinks to /private/tmp, so we must canonicalize
        // both the base and the resolved target for consistent comparison.
        let base_canonical =
            std::fs::canonicalize(&self.base_path).map_err(|e| StorageError::InvalidPath {
                path: self.base_path.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;

        // Build the canonicalized version of the target path.
        // If the file already exists, canonicalize it directly.
        // Otherwise, resolve the target by joining the canonical base with the relative path.
        let target_canonical =
            std::fs::canonicalize(&full).unwrap_or_else(|_| base_canonical.join(path));

        if !target_canonical.starts_with(&base_canonical) {
            return Err(StorageError::InvalidPath {
                path: path.to_string(),
                reason: "Path escapes base directory".to_string(),
            });
        }

        Ok(full)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[async_trait::async_trait]
impl StorageBackend for LocalStorage {
    async fn upload_object(
        &self,
        path: &str,
        data: Vec<u8>,
        _content_type: &str,
    ) -> Result<String, StorageError> {
        let full_path = self.full_path(path)?;

        // PERF FIX: Use spawn_blocking for blocking file I/O operations
        let path_clone = full_path.clone();
        task::spawn_blocking(move || {
            // Create parent directories if needed
            if let Some(parent) = path_clone.parent() {
                std::fs::create_dir_all(parent).map_err(|e| StorageError::UploadFailed {
                    message: e.to_string(),
                })?;
            }

            std::fs::write(&path_clone, data).map_err(|e| StorageError::UploadFailed {
                message: e.to_string(),
            })?;

            Ok::<(), StorageError>(())
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        })
        .await
        .map_err(|e| StorageError::UploadFailed {
            message: format!("Task join error: {e}"),
        })??;

        let url = format!("file://{}", full_path.display());
        info!("Wrote {} bytes to {}", full_path.display(), url);

        Ok(url)
    }

    async fn download_object(&self, path: &str) -> Result<Vec<u8>, StorageError> {
        let full_path = self.full_path(path)?;

        // PERF FIX: Use spawn_blocking for blocking file I/O operations
        task::spawn_blocking(move || {
            std::fs::read(&full_path).map_err(|e| StorageError::DownloadFailed {
                message: e.to_string(),
            })
        })
        .await
        .map_err(|e| StorageError::DownloadFailed {
            message: format!("Task join error: {e}"),
        })?
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    async fn object_exists(&self, path: &str) -> Result<bool, StorageError> {
        let full_path = self.full_path(path)?;

        // PERF FIX: Use spawn_blocking for blocking file metadata operations
        task::spawn_blocking(move || Ok(full_path.exists()))
            .await
            .map_err(|e| StorageError::DownloadFailed {
                message: format!("Task join error: {e}"),
            })?
    }

    #[allow(clippy::items_after_statements)]
    async fn list_objects(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        let full_prefix = self.full_path(prefix)?;

        // PERF FIX: Use spawn_blocking for blocking directory traversal
        task::spawn_blocking(move || {
            if !full_prefix.exists() {
                return Ok(vec![]);
            }

            let mut results = vec![];

            fn visit_dir(
                dir: &std::path::Path,
                results: &mut Vec<String>,
            ) -> Result<(), StorageError> {
                for entry in std::fs::read_dir(dir).map_err(|e| StorageError::DownloadFailed {
                    message: e.to_string(),
                })? {
                    let entry = entry.map_err(|e| StorageError::DownloadFailed {
                        message: e.to_string(),
                    })?;
                    let path = entry.path();

                    if path.is_dir() {
                        visit_dir(&path, results)?;
                    } else {
                        results.push(path.to_string_lossy().to_string());
                    }
                }
                Ok(())
            }

            // TAG: surface=simulation owner=research-team rule=SIM-001
            if full_prefix.is_dir() {
                visit_dir(&full_prefix, &mut results)?;
            } else if full_prefix.exists() {
                results.push(full_prefix.to_string_lossy().to_string());
            }

            Ok(results)
        })
        .await
        .map_err(|e| StorageError::DownloadFailed {
            message: format!("Task join error: {e}"),
        })?
    }

    async fn delete_object(&self, path: &str) -> Result<(), StorageError> {
        let full_path = self.full_path(path)?;

        // PERF FIX: Use spawn_blocking for blocking file I/O operations
        task::spawn_blocking(move || match std::fs::remove_file(&full_path) {
            Ok(()) => Ok::<(), StorageError>(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::UploadFailed {
                message: e.to_string(),
            }),
        })
        .await
        .map_err(|e| StorageError::UploadFailed {
            message: format!("Task join error: {e}"),
        })?
    }

    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn object_url(&self, path: &str) -> Result<String, StorageError> {
        Ok(format!("file://{}", self.full_path(path)?.display()))
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Artifact storage manager
///
/// Provides high-level operations for simulation artifacts.
pub struct ArtifactStorage {
    backend: Box<dyn StorageBackend>,
}

impl std::fmt::Debug for ArtifactStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactStorage").finish_non_exhaustive()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ArtifactStorage {
    /// Create a new artifact storage with the given backend
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn new(backend: Box<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    /// Create from environment (GCS or local)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_env() -> Result<Self, StorageError> {
        if std::env::var("USE_LOCAL_STORAGE").unwrap_or_default() == "true" {
            let local = LocalStorage::temp()?;
            info!("Using LocalStorage for artifacts");
            Ok(Self::new(Box::new(local)))
        } else {
            let gcs = GcsStorage::from_env()?;
            info!("Using GcsStorage for artifacts");
            Ok(Self::new(Box::new(gcs)))
        }
    }

    /// Upload a manifest.json for a job
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[allow(clippy::future_not_send)]
    pub async fn upload_manifest<T: Serialize>(
        &self,
        job_id: &str,
        manifest: &T,
    ) -> Result<String, StorageError> {
        let path = format!("{job_id}/manifest.json");
        let data =
            serde_json::to_vec_pretty(manifest).map_err(|e| StorageError::SerializationError {
                message: e.to_string(),
            })?;

        self.backend
            .upload_object(&path, data, "application/json")
            .await
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Upload metrics data
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn upload_metrics(
        &self,
        job_id: &str,
        metrics: Vec<u8>,
    ) -> Result<String, StorageError> {
        let path = format!("{job_id}/metrics.jsonl");
        self.backend
            .upload_object(&path, metrics, "application/jsonl")
            .await
    }

    /// Upload a replay frame
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn upload_replay_frame(
        &self,
        job_id: &str,
        tick: u64,
        frame: Vec<u8>,
    ) -> Result<String, StorageError> {
        let path = format!("{job_id}/replay/{tick:08}.jsonl");
        self.backend
            .upload_object(&path, frame, "application/jsonl")
            .await
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Download a manifest
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn download_manifest(&self, job_id: &str) -> Result<Vec<u8>, StorageError> {
        let path = format!("{job_id}/manifest.json");
        self.backend.download_object(&path).await
    }

    /// List all artifacts for a job
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn list_job_artifacts(&self, job_id: &str) -> Result<Vec<String>, StorageError> {
        self.backend.list_objects(job_id).await
    }

    /// Check if a job has artifacts
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn job_exists(&self, job_id: &str) -> Result<bool, StorageError> {
        let path = format!("{job_id}/manifest.json");
        self.backend.object_exists(&path).await
    }
}

#[cfg(test)]
#[allow(clippy::items_after_statements)]
mod tests {
    use super::*;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[tokio::test]
    async fn test_local_storage() {
        let storage = LocalStorage::temp().unwrap();

        // Upload
        let url = storage
            .upload_object("test/file.txt", b"hello world".to_vec(), "text/plain")
            .await
            .unwrap();
        assert!(url.contains("file://"));

        // Exists
        assert!(storage.object_exists("test/file.txt").await.unwrap());
        assert!(!storage.object_exists("test/missing.txt").await.unwrap());

        // Download
        let data = storage.download_object("test/file.txt").await.unwrap();
        assert_eq!(data, b"hello world");

        // List
        let objects = storage.list_objects("test").await.unwrap();
        assert_eq!(objects.len(), 1);

        // Delete
        storage.delete_object("test/file.txt").await.unwrap();
        assert!(!storage.object_exists("test/file.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_artifact_storage() {
        let local = LocalStorage::temp().unwrap();
        let artifact_storage = ArtifactStorage::new(Box::new(local));

        // TAG: surface=simulation owner=research-team rule=SIM-001
        #[derive(Serialize)]
        struct TestManifest {
            job_id: String,
            version: String,
        }

        let manifest = TestManifest {
            job_id: "test-job".to_string(),
            version: "1.0".to_string(),
        };

        // Upload manifest
        let url = artifact_storage
            .upload_manifest("test-job", &manifest)
            .await
            .unwrap();
        assert!(url.contains("manifest.json"));

        // Upload metrics
        let metrics_url = artifact_storage
            .upload_metrics("test-job", b"{}\n{}\n".to_vec())
            .await
            .unwrap();
        assert!(metrics_url.contains("metrics.jsonl"));

        // Upload replay frame
        let frame_url = artifact_storage
            .upload_replay_frame("test-job", 100, b"[]".to_vec())
            .await
            .unwrap();
        assert!(frame_url.contains("replay/00000100.jsonl"));

        // Check job exists
        assert!(artifact_storage.job_exists("test-job").await.unwrap());

        // List artifacts
        let artifacts = artifact_storage
            .list_job_artifacts("test-job")
            .await
            .unwrap();
        assert_eq!(artifacts.len(), 3);
    }
}
