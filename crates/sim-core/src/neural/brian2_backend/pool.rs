// TAG: surface=simulation owner=platform-team rule=GENERAL-001
//! Process connection pooling and communication for Brian2.

use crate::neural::NeuralValidationError;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Pooled connection to Brian2 Python process
#[derive(Debug)]
struct PooledConnection {
    child: Child,
    _created_at: Instant,
    last_used: Instant,
}

/// Connection pool for Brian2 processes
#[derive(Debug)]
pub struct Brian2ConnectionPool {
    python_path: String,
    script_path: String,
    max_connections: usize,
    timeout: Duration,
    connections: Arc<Mutex<Vec<PooledConnection>>>,
}

impl Brian2ConnectionPool {
    /// Create new connection pool
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn new(
        python_path: String,
        script_path: String,
        max_connections: usize,
        timeout: Duration,
    ) -> Self {
        Self {
            python_path,
            script_path,
            max_connections,
            timeout,
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a connection from the pool or create new one
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn acquire(&self) -> Result<Brian2Connection, NeuralValidationError> {
        let mut connections = self.connections.lock().unwrap_or_else(|e| e.into_inner());

        // Remove stale connections
        let timeout = self.timeout;
        connections.retain(|conn| conn.last_used.elapsed() < timeout);

        // Enforce max connections limit
        while connections.len() >= self.max_connections {
            connections.remove(0);
        }

        // Return existing connection if available
        if let Some(mut conn) = connections.pop() {
            conn.last_used = Instant::now();
            return Ok(Brian2Connection {
                child: Some(conn.child),
                pool: self.connections.clone(),
            });
        }

        // Create new connection
        drop(connections);
        let child = Command::new(&self.python_path)
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| NeuralValidationError::Backend(format!("Failed to spawn Brian2: {e}")))?;

        Ok(Brian2Connection {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            child: Some(child),
            pool: self.connections.clone(),
        })
    }
}

/// A connection that returns to pool when dropped
#[derive(Debug)]
pub struct Brian2Connection {
    child: Option<Child>,
    pool: Arc<Mutex<Vec<PooledConnection>>>,
}

impl Brian2Connection {
    /// Take ownership of child process
    pub fn take_child(&mut self) -> Option<Child> {
        self.child.take()
    }
}

impl Drop for Brian2Connection {
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn drop(&mut self) {
        // Return connection to pool if still alive and not consumed
        if let Some(mut child) = self.child.take() {
            if child.try_wait().is_ok() {
                let conn = PooledConnection {
                    child,
                    _created_at: Instant::now(),
                    last_used: Instant::now(),
                };
                if let Ok(mut pool) = self.pool.lock() {
                    pool.push(conn);
                }
            }
        }
    }
}
