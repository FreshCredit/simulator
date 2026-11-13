//! Connection rate limiting for SSE streaming endpoints
//!
//! Provides per-IP connection limiting to prevent abuse of streaming endpoints.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// TAG: surface=simulation owner=research-team rule=SIM-001
// TAG: surface=simulation owner=research-team rule=SIM-001
/// Connection rate limiter for streaming endpoints
///
/// Tracks connection attempts per IP address and enforces limits
/// over a sliding time window.
///
/// # Example
///
/// ```rust,ignore
/// use sim_service::rate_limit::connection::ConnectionLimiter;
/// use std::net::IpAddr;
///
/// # tokio_test::block_on(async {
/// let limiter = ConnectionLimiter::new(5, 60); // 5 connections per 60 seconds
/// let ip: IpAddr = "127.0.0.1".parse().unwrap_or(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
/// debug_assert!(limiter.check(ip).await);
/// # })
/// ```
#[derive(Clone, Debug)]
pub struct ConnectionLimiter {
    /// Tracks connection attempts per IP
    attempts: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
    /// Maximum connections per window
    max_connections: usize,
    /// Window duration
    window: Duration,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

impl ConnectionLimiter {
    /// Create a new connection limiter
    ///
    /// # Arguments
    ///
    /// * `max_connections` - Maximum number of connections allowed per window
    /// * `window_secs` - Duration of the sliding window in seconds
    ///
    /// # Example
    ///
    /// ```
    /// use sim_service::rate_limit::connection::ConnectionLimiter;
    ///
    /// let limiter = ConnectionLimiter::new(5, 60);
    /// ```
    #[must_use]
    pub fn new(max_connections: usize, window_secs: u64) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_connections,
            window: Duration::from_secs(window_secs),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if a new connection is allowed from the given IP
    ///
    /// Returns `true` if the connection is allowed, `false` if the rate limit
    /// has been exceeded.
    ///
    /// This method automatically cleans up old entries outside the window
    /// before checking the limit.
    pub async fn check(&self, ip: IpAddr) -> bool {
        let mut attempts = self.attempts.lock().await;
        let now = Instant::now();

        // Clean old entries outside the window
        let window_start = now.checked_sub(self.window);
        let entry = attempts.entry(ip).or_default();
        entry.retain(|&t| window_start.is_none_or(|start| t > start));

        // Check if limit exceeded
        if entry.len() >= self.max_connections {
            tracing::warn!(
                "Connection rate limit exceeded for IP {}: {}/{} in {:?}",
                ip,
                entry.len(),
                self.max_connections,
                self.window
            );
            return false;
        }

        // Record this attempt
        entry.push(now);
        tracing::debug!(
            "Connection allowed for IP {}: {}/{} in {:?}",
            ip,
            entry.len(),
            self.max_connections,
            self.window
        );

        true
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get current connection count for an IP
    ///
    /// Useful for metrics and debugging.
    pub async fn current_count(&self, ip: IpAddr) -> usize {
        let mut attempts = self.attempts.lock().await;
        let now = Instant::now();
        let window_start = now.checked_sub(self.window);

        let entry = attempts.entry(ip).or_default();
        entry.retain(|&t| window_start.is_none_or(|start| t > start));

        entry.len()
    }

    /// Reset connection count for an IP
    ///
    /// Useful for administrative operations or testing.
    pub async fn reset(&self, ip: IpAddr) {
        let mut attempts = self.attempts.lock().await;
        attempts.remove(&ip);
    }
}

impl Default for ConnectionLimiter {
    fn default() -> Self {
        // Default: 5 connections per minute
        Self::new(5, 60)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Per-session message rate limiter
///
/// Throttles message flow within an established connection.
/// Used for WebSocket or SSE streams to prevent clients from
/// being overwhelmed by high-frequency updates.
#[derive(Clone, Debug)]
pub struct MessageThrottler {
    min_interval: Duration,
}

impl MessageThrottler {
    /// Create a new message throttler
    ///
    /// # Arguments
    ///
    /// * `max_messages_per_sec` - Maximum messages per second
    #[must_use]
    pub fn new(max_messages_per_sec: u32) -> Self {
        let interval_ms = 1000 / max_messages_per_sec.max(1);
        Self {
            min_interval: Duration::from_millis(interval_ms as u64),
        }
    }

    /// Get the minimum interval between messages
    #[must_use]
    pub const fn interval(&self) -> Duration {
        self.min_interval
    }
}

impl Default for MessageThrottler {
    fn default() -> Self {
        // Default: 10 messages per second
        Self::new(10)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Preconfigured limiters for different endpoint types
#[derive(Debug)]
pub struct EndpointLimiters {
    /// For live simulation SSE stream
    pub live_stream: ConnectionLimiter,
    /// For experiment progress streams
    pub experiment_stream: ConnectionLimiter,
    /// For neural brain state queries
    pub neural_query: ConnectionLimiter,
}

impl EndpointLimiters {
    /// Create limiters with production defaults
    ///
    /// | Endpoint | Max Connections | Window |
    /// |----------|----------------|--------|
    /// | Live stream | 5 | 60s |
    /// | Experiment stream | 10 | 60s |
    /// | Neural query | 20 | 60s |
    #[must_use]
    pub fn production() -> Self {
        Self {
            live_stream: ConnectionLimiter::new(5, 60),
            experiment_stream: ConnectionLimiter::new(10, 60),
            neural_query: ConnectionLimiter::new(20, 60),
        }
    }

    /// Create limiters with relaxed defaults for development
    #[must_use]
    pub fn development() -> Self {
        Self {
            live_stream: ConnectionLimiter::new(20, 60),
            experiment_stream: ConnectionLimiter::new(30, 60),
            neural_query: ConnectionLimiter::new(50, 60),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for EndpointLimiters {
    fn default() -> Self {
        Self::production()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_limiter_allows_within_limit() {
        let limiter = ConnectionLimiter::new(3, 60);
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        assert!(limiter.check(ip).await);
        assert!(limiter.check(ip).await);
        assert!(limiter.check(ip).await);
    }

    #[tokio::test]
    async fn test_connection_limiter_blocks_over_limit() {
        let limiter = ConnectionLimiter::new(2, 60);
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(limiter.check(ip).await);
        assert!(limiter.check(ip).await);
        assert!(!limiter.check(ip).await); // Third connection blocked
    }

    #[tokio::test]
    async fn test_connection_limiter_per_ip() {
        let limiter = ConnectionLimiter::new(2, 60);
        let ip1: IpAddr = "127.0.0.1".parse().unwrap();
        let ip2: IpAddr = "127.0.0.2".parse().unwrap();

        // Both IPs can use their full quota independently
        assert!(limiter.check(ip1).await);
        assert!(limiter.check(ip1).await);
        assert!(limiter.check(ip2).await);
        assert!(limiter.check(ip2).await);

        // Both at limit now
        assert!(!limiter.check(ip1).await);
        assert!(!limiter.check(ip2).await);
    }

    #[tokio::test]
    async fn test_connection_limiter_reset() {
        let limiter = ConnectionLimiter::new(1, 60);
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(limiter.check(ip).await);
        assert!(!limiter.check(ip).await);

        // Reset and try again
        limiter.reset(ip).await;
        assert!(limiter.check(ip).await);
    }

    #[tokio::test]
    async fn test_current_count() {
        let limiter = ConnectionLimiter::new(5, 60);
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        assert_eq!(limiter.current_count(ip).await, 0);
        limiter.check(ip).await;
        assert_eq!(limiter.current_count(ip).await, 1);
        limiter.check(ip).await;
        assert_eq!(limiter.current_count(ip).await, 2);
    }

    #[test]
    fn test_message_throttler() {
        let throttler = MessageThrottler::new(10);
        assert_eq!(throttler.interval(), Duration::from_millis(100));

        let throttler = MessageThrottler::new(5);
        assert_eq!(throttler.interval(), Duration::from_millis(200));
    }
}
