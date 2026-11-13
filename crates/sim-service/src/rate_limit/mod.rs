//! Rate limiting module for sim-service
//!
//! Provides tiered rate limiting based on user roles (Consumer, Provider, Internal)
//! and connection limiting for SSE streaming endpoints.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! ## Usage
//!
//! ```rust,ignore
//! use axum::{Router, routing::get};
//! use sim_service::rate_limit::{Tier, TieredKeyExtractor};
//! use sim_service::api::experiments::list_experiments;
//! use tower_governor::GovernorLayer;
//!
//! let config = Tier::Consumer.config();
//! let app = Router::new()
//!     .route("/api/experiments", get(list_experiments))
//!     .layer(GovernorLayer::new(config));
//! ```

use axum::http::Request;
use governor::middleware::StateInformationMiddleware;
use tower_governor::{
    errors::GovernorError,
    governor::{GovernorConfig, GovernorConfigBuilder},
    key_extractor::KeyExtractor,
};

use crate::auth::rbac::UserRole;

pub mod connection;
pub mod error;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Rate limiting tier based on user role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tier {
    /// Unauthenticated users - most restrictive
    #[default]
    Anonymous,
    /// Consumer role - local simulations only
    Consumer,
    /// Provider role - remote jobs allowed
    Provider,
    /// Internal role - full access
    Internal,
}

impl Tier {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the `GovernorConfig` for this tier
    ///
    /// # Examples
    ///
    /// ```
    /// use sim_service::rate_limit::Tier;
    ///
    /// let config = Tier::Consumer.config();
    /// ```
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn config(&self) -> GovernorConfig<TieredKeyExtractor, StateInformationMiddleware> {
        match GovernorConfigBuilder::default()
            .per_second(self.replenish_rate())
            .burst_size(self.burst_size())
            .key_extractor(TieredKeyExtractor)
            .use_headers()
            .finish()
        {
            Some(cfg) => cfg,
            None => unreachable!("valid rate limit config"),
        }
    }

    /// Burst size (maximum requests in a burst)
    const fn burst_size(self) -> u32 {
        match self {
            Self::Anonymous => 5,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::Consumer => 10,
            Self::Provider => 20,
            Self::Internal => 50,
        }
    }

    /// Replenish rate (requests per second)
    const fn replenish_rate(self) -> u64 {
        match self {
            Self::Anonymous => 1, // 60 req/min
            Self::Consumer => 1,  // 60 req/min
            Self::Provider => 2,  // 120 req/min
            Self::Internal => 10, // 600 req/min
        }
    }

    /// Get human-readable description
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Anonymous => "Anonymous users: 60 req/min, burst: 5",
            Self::Consumer => "Consumer: 60 req/min, burst: 10",
            Self::Provider => "Provider: 120 req/min, burst: 20",
            Self::Internal => "Internal: 600 req/min, burst: 50",
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl From<UserRole> for Tier {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::Consumer => Self::Consumer,
            UserRole::Provider => Self::Provider,
            UserRole::Internal => Self::Internal,
        }
    }
}

/// Key extractor that identifies clients by tier for rate limiting
///
/// Extracts the client identifier from:
/// 1. `x-api-key` header (if present)
/// 2. Client IP address from request extensions
///
/// The tier is determined from the `x-user-role` header.
#[derive(Debug, Clone)]
pub struct TieredKeyExtractor;

impl KeyExtractor for TieredKeyExtractor {
    type Key = (String, Tier);

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        // Extract verified role from request extensions, defaulting unauthenticated requests to Anonymous
        let tier = req
            .extensions()
            .get::<UserRole>()
            .copied()
            .map_or(Tier::Anonymous, Tier::from);

        // Use API key if present, otherwise IP-based identifier
        let client_id = req
            .headers()
            .get("x-api-key")
            .and_then(|h| h.to_str().ok())
            .map(std::string::ToString::to_string)
            .or_else(|| {
                req.extensions()
                    .get::<std::net::SocketAddr>()
                    .map(|a| a.ip().to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        Ok((client_id, tier))
    }

    fn name(&self) -> &'static str {
        "TieredKeyExtractor"
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get rate limit config for a specific endpoint category
#[must_use]
pub fn config_for_endpoint(
    category: EndpointCategory,
) -> GovernorConfig<TieredKeyExtractor, StateInformationMiddleware> {
    match category {
        // Public read endpoints - higher limits
        EndpointCategory::Public => Tier::Anonymous.config(),
        // Standard API - Consumer limits
        EndpointCategory::Standard => Tier::Consumer.config(),
        // Resource-intensive operations - stricter limits
        EndpointCategory::Compute => Tier::Provider.config(),
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Endpoint category for rate limiting
#[derive(Debug, Clone, Copy)]
pub enum EndpointCategory {
    /// Public read-only endpoints (listing, health)
    Public,
    /// Standard API endpoints
    Standard,
    /// Compute-intensive endpoints (experiment launch)
    Compute,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_limits() {
        assert_eq!(Tier::Anonymous.burst_size(), 5);
        assert_eq!(Tier::Consumer.burst_size(), 10);
        assert_eq!(Tier::Provider.burst_size(), 20);
        assert_eq!(Tier::Internal.burst_size(), 50);

        assert_eq!(Tier::Anonymous.replenish_rate(), 1);
        assert_eq!(Tier::Consumer.replenish_rate(), 1);
        assert_eq!(Tier::Provider.replenish_rate(), 2);
        assert_eq!(Tier::Internal.replenish_rate(), 10);
    }

    #[test]
    fn test_tier_from_role() {
        assert!(matches!(Tier::from(UserRole::Consumer), Tier::Consumer));
        assert!(matches!(Tier::from(UserRole::Provider), Tier::Provider));
        assert!(matches!(Tier::from(UserRole::Internal), Tier::Internal));
    }

    #[test]
    fn test_tier_config_creation() {
        // Should not panic
        let _ = Tier::Consumer.config();
        let _ = Tier::Provider.config();
        let _ = Tier::Internal.config();
    }

    #[test]
    fn test_tiered_key_extractor_rejects_untrusted_header() {
        let extractor = TieredKeyExtractor;
        let mut req = Request::builder()
            .header("x-user-role", "internal")
            .body(())
            .unwrap();
        let key = extractor.extract(&req).unwrap();
        // Untrusted header alone must not elevate to Internal; defaults to Anonymous
        assert_eq!(key.1, Tier::Anonymous);

        // Verified extension must grant Internal
        req.extensions_mut().insert(UserRole::Internal);
        let key2 = extractor.extract(&req).unwrap();
        assert_eq!(key2.1, Tier::Internal);
    }
}
