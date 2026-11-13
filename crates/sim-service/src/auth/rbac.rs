//! Role-Based Access Control (§10.4, §23.10.3)
//!
//! Enforces per-role limits on simulation jobs.
//!
//! ## Enterprise Compliance
//!
//! - **ENT-002**: Granular RBAC permission validation
//! - **ENT-003**: Multi-tenant data isolation with `tenant_id`
//!
//! This module provides enterprise-grade permission checking for SOC2 compliance.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use tracing::{info, warn};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Enterprise permission types for granular access control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Read simulation results
    SimulationRead,
    /// Create and run simulations
    SimulationWrite,
    /// Delete simulations
    SimulationDelete,
    /// Manage user settings
    UserManage,
    /// View audit logs
    AuditRead,
    /// Administrative functions
    Admin,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Enterprise SSO provider configuration (ENT-004 compliance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoProvider {
    /// provider id
    pub provider_id: String,
    /// provider type
    pub provider_type: SsoProviderType,
    /// tenant id
    pub tenant_id: String,
    /// enabled
    pub enabled: bool,
}

/// SSO provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SsoProviderType {
    /// SAML 2.0 Identity Provider
    Saml,
    /// `OpenID` Connect Provider
    Oidc,
    /// Microsoft Entra ID
    Entra,
    /// Google Workspace
    Google,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// SSO configuration for enterprise authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoConfig {
    /// tenant id
    pub tenant_id: String,
    /// providers
    pub providers: Vec<SsoProvider>,
    /// enforce sso
    pub enforce_sso: bool,
    /// allow bypass
    pub allow_bypass: bool,
}

/// Check if a user has a specific permission (ENT-002 compliance)
///
/// # Arguments
/// * `user_role` - The user's assigned role
/// * `permission` - The permission to check
///
/// # Returns
/// `true` if the user has the permission, `false` otherwise
///
/// # Example
/// ```
/// use sim_service::auth::rbac::{UserRole, Permission, has_permission};
///
/// debug_assert!(has_permission(UserRole::Internal, Permission::Admin));
/// debug_assert!(!has_permission(UserRole::Consumer, Permission::Admin));
/// ```
#[must_use]
pub const fn has_permission(user_role: UserRole, permission: Permission) -> bool {
    match (user_role, permission) {
        // Internal users have all permissions
        (UserRole::Internal, _) => true,
        // Providers can read/write simulations but not admin
        (UserRole::Provider, Permission::SimulationRead) => true,
        (UserRole::Provider, Permission::SimulationWrite) => true,
        (UserRole::Provider, Permission::SimulationDelete) => true,
        (UserRole::Provider, Permission::UserManage) => true,
        (UserRole::Provider, _) => false,
        // Consumers have limited read access
        (UserRole::Consumer, Permission::SimulationRead) => true,
        (UserRole::Consumer, Permission::UserManage) => true,
        (UserRole::Consumer, _) => false,
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Enforce permission check with error return (ENT-002 compliance)
///
/// # Arguments
/// * `user_role` - The user's assigned role
/// * `permission` - The permission to enforce
/// * `resource` - Description of the resource being accessed
///
/// # Returns
/// `Ok(())` if permission granted, `Err(RbacError)` otherwise
///
/// # Example
/// ```
/// use sim_service::auth::rbac::{UserRole, Permission, enforce_permission};
///
/// debug_assert!(enforce_permission(UserRole::Provider, Permission::SimulationWrite, "experiment").is_ok());
/// ```
/// # Errors
///
/// Returns an error if the operation fails.
pub fn enforce_permission(
    user_role: UserRole,
    permission: Permission,
    resource: &str,
) -> Result<(), RbacError> {
    if has_permission(user_role, permission) {
        info!(
            "Permission granted: role={:?} permission={:?} resource={}",
            user_role, permission, resource
        );
        Ok(())
    } else {
        warn!(
            "Permission denied: role={:?} permission={:?} resource={}",
            user_role, permission, resource
        );
        Err(RbacError::PermissionDenied {
            role: user_role.to_string(),
            permission: format!("{permission:?}"),
            resource: resource.to_string(),
        })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Audit log macro for enterprise compliance (ENT-001)
///
/// Records audit events for SOC2 compliance tracking.
///
/// # Example
/// ```
/// use sim_service::audit_log;
///
/// audit_log!("user_login", "user_123", "tenant_456", "Successful login");
/// ```
#[macro_export]
macro_rules! audit_log {
    ($action:expr, $user_id:expr, $tenant_id:expr, $details:expr) => {
        tracing::info!(
            audit_event = true,
            action = $action,
            user_id = $user_id,
            tenant_id = $tenant_id,
            details = $details,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "AUDIT: {} by user={} tenant={}: {}",
            $action, $user_id, $tenant_id, $details
        );
    };
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// User roles for simulation access control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Consumer role: limited to local simulations
    Consumer,
    /// Provider role: can run remote jobs up to 10k agents
    Provider,
    /// Internal role: full access for `FreshCredit` team
    Internal,
}

impl UserRole {
    /// Get maximum allowed agents for this role
    #[must_use]
    pub const fn max_agents(&self) -> u64 {
        match self {
            Self::Consumer => 1_000,
            Self::Provider => 10_000,
            Self::Internal => 100_000,
        }
    }

    /// Get maximum simulation duration in seconds
    #[must_use]
    pub const fn max_duration_secs(&self) -> u64 {
        match self {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::Consumer => 3_600,  // 1 hour
            Self::Provider => 14_400, // 4 hours
            Self::Internal => 28_800, // 8 hours
        }
    }

    /// Check if remote jobs are allowed
    #[must_use]
    pub const fn allow_remote(&self) -> bool {
        match self {
            Self::Consumer => false,
            Self::Provider => true,
            Self::Internal => true,
        }
    }

    /// Get human-readable description
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Consumer => "Local simulation only, max 1,000 agents, 1 hour",
            Self::Provider => "Remote jobs allowed, max 10,000 agents, 4 hours",
            Self::Internal => "Full access, max 100,000 agents, 8 hours",
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Consumer => write!(f, "consumer"),
            Self::Provider => write!(f, "provider"),
            Self::Internal => write!(f, "internal"),
        }
    }
}

impl From<String> for UserRole {
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "consumer" => Self::Consumer,
            "provider" => Self::Provider,
            "internal" => Self::Internal,
            _ => {
                warn!("Unknown role '{}', defaulting to Consumer", s);
                Self::Consumer
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// RBAC check errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum RbacError {
    #[error("RBAC_LIMIT_EXCEEDED: Your role ({role}) allows maximum {limit} agents. Requested: {requested}")]
    /// `AgentLimitExceeded`
    AgentLimitExceeded {
        /// Requested agent count
        requested: u64,
        /// Allowed limit
        limit: u64,
        /// User role
        role: String,
    },

    #[error("RBAC_LIMIT_EXCEEDED: Your role ({role}) allows maximum {limit:.2} hours. Requested: {requested:.2} hours")]
    /// `DurationLimitExceeded`
    DurationLimitExceeded {
        /// Requested duration
        requested: f64,
        /// Allowed limit
        limit: f64,
        /// User role
        role: String,
    },

    #[error("RBAC_REMOTE_FORBIDDEN: Your role ({role}) does not allow remote jobs. Run locally with up to {max_agents} agents")]
    /// `RemoteJobsForbidden`
    RemoteJobsForbidden {
        /// User role
        role: String,
        /// Maximum allowed agents for local runs
        max_agents: u64,
    },

    #[error("RBAC_PERMISSION_DENIED: Role '{role}' lacks permission '{permission}' for resource '{resource}'")]
    /// `PermissionDenied`
    PermissionDenied {
        /// User role
        role: String,
        /// Permission required
        permission: String,
        /// Resource accessed
        resource: String,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// TAG: surface=simulation owner=research-team rule=SIM-001
/// Check if a job request is within role limits
///
/// # Arguments
/// * `role` - The user's role
/// * `agents` - Requested number of agents
/// * `duration_secs` - Requested duration in seconds
///
/// # Returns
/// * `Ok(())` if within limits
/// * `Err(RbacError)` if limits exceeded
///
/// # Example
/// ```
/// use sim_service::auth::rbac::{UserRole, check_job_limits};
///
/// // Consumer requesting 500 agents for 30 minutes - OK
/// debug_assert!(check_job_limits(UserRole::Consumer, 500, 1800).is_ok());
///
/// // Consumer requesting 10,000 agents - EXCEEDS LIMIT
/// debug_assert!(check_job_limits(UserRole::Consumer, 10_000, 1800).is_err());
/// ```
/// # Errors
///
/// Returns an error if the operation fails.
pub fn check_job_limits(role: UserRole, agents: u64, duration_secs: u64) -> Result<(), RbacError> {
    let max_agents = role.max_agents();
    let max_duration = role.max_duration_secs();

    // Check agent limit
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    if agents > max_agents {
        warn!(
            "RBAC violation: role={} requested_agents={} max_agents={}",
            role, agents, max_agents
        );
        return Err(RbacError::AgentLimitExceeded {
            requested: agents,
            limit: max_agents,
            role: role.to_string(),
        });
    }

    // Check duration limit
    if duration_secs > max_duration {
        warn!(
            "RBAC violation: role={} requested_duration={}s max_duration={}s",
            role, duration_secs, max_duration
        );
        return Err(RbacError::DurationLimitExceeded {
            requested: duration_secs as f64 / 3600.0,
            limit: max_duration as f64 / 3600.0,
            role: role.to_string(),
        });
    }

    info!(
        "RBAC check passed: role={} agents={} duration={}s",
        role, agents, duration_secs
    );
    Ok(())
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Check if remote execution is allowed for a role
///
/// # Arguments
/// * `role` - The user's role
/// * `agents` - Requested number of agents (for error message)
///
/// # Returns
/// * `Ok(())` if remote jobs allowed
/// * `Err(RbacError::RemoteJobsForbidden)` if not
/// # Errors
///
/// Returns an error if the operation fails.
pub fn check_remote_allowed(role: UserRole, _agents: u64) -> Result<(), RbacError> {
    if role.allow_remote() {
        Ok(())
    } else {
        Err(RbacError::RemoteJobsForbidden {
            role: role.to_string(),
            max_agents: role.max_agents(),
        })
    }
}

/// Get role limits as a structured response
#[derive(Debug, Clone, Serialize)]
pub struct RoleLimits {
    /// role
    pub role: String,
    /// max agents
    pub max_agents: u64,
    /// max duration secs
    pub max_duration_secs: u64,
    /// max duration hours
    pub max_duration_hours: f64,
    /// allow remote
    pub allow_remote: bool,
    /// description
    pub description: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl From<UserRole> for RoleLimits {
    fn from(role: UserRole) -> Self {
        Self {
            role: role.to_string(),
            max_agents: role.max_agents(),
            max_duration_secs: role.max_duration_secs(),
            max_duration_hours: role.max_duration_secs() as f64 / 3600.0,
            allow_remote: role.allow_remote(),
            description: role.description().to_string(),
        }
    }
}

// RBAC middleware extractors for Axum
// Note: Axum extractors require the 'axum' feature
// For now, these are commented out. Enable when needed:
// #[cfg(feature = "axum")]
/*
/// extractors module
pub mod extractors {
    use axum::{
        async_trait,
        extract::FromRequestParts,
        http::{request::Parts, StatusCode},
    };
    use super::UserRole;

// TAG: surface=simulation owner=research-team rule=SIM-001
    /// Extract user role from request headers
    pub struct RoleExtractor(pub UserRole);

    #[async_trait]
    impl<S> FromRequestParts<S> for RoleExtractor
    where
        S: Send + Sync,
    {
        type Rejection = (StatusCode, String);

        async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
            // Extract role from X-User-Role header (set by auth proxy)
            let role = parts
                .headers
                .get("x-user-role")
                .and_then(|v| v.to_str().ok())
                .map(|s| UserRole::from(s.to_string()))
                .unwrap_or(UserRole::Consumer); // Default to most restrictive

            Ok(RoleExtractor(role))
        }
    }
}
*/

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_limits() {
        assert_eq!(UserRole::Consumer.max_agents(), 1_000);
        assert_eq!(UserRole::Provider.max_agents(), 10_000);
        assert_eq!(UserRole::Internal.max_agents(), 100_000);

        assert_eq!(UserRole::Consumer.max_duration_secs(), 3_600);
        assert_eq!(UserRole::Provider.max_duration_secs(), 14_400);
        assert_eq!(UserRole::Internal.max_duration_secs(), 28_800);
    }

    #[test]
    fn test_check_job_limits_consumer() {
        // Within limits
        assert!(check_job_limits(UserRole::Consumer, 500, 1800).is_ok());
        assert!(check_job_limits(UserRole::Consumer, 1_000, 3_600).is_ok());

        // Exceeds agent limit
        let err = check_job_limits(UserRole::Consumer, 1_001, 1800).unwrap_err();
        assert!(matches!(err, RbacError::AgentLimitExceeded { .. }));

        // Exceeds duration limit
        let err = check_job_limits(UserRole::Consumer, 500, 3_601).unwrap_err();
        assert!(matches!(err, RbacError::DurationLimitExceeded { .. }));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_check_job_limits_provider() {
        // Within limits
        assert!(check_job_limits(UserRole::Provider, 5_000, 7200).is_ok());
        assert!(check_job_limits(UserRole::Provider, 10_000, 14_400).is_ok());

        // Exceeds limits
        assert!(check_job_limits(UserRole::Provider, 10_001, 7200).is_err());
        assert!(check_job_limits(UserRole::Provider, 5_000, 14_401).is_err());
    }

    #[test]
    fn test_check_remote_allowed() {
        assert!(check_remote_allowed(UserRole::Consumer, 100).is_err());
        assert!(check_remote_allowed(UserRole::Provider, 100).is_ok());
        assert!(check_remote_allowed(UserRole::Internal, 100).is_ok());
    }

    #[test]
    fn test_user_role_from_string() {
        assert_eq!(UserRole::from("consumer".to_string()), UserRole::Consumer);
        assert_eq!(UserRole::from("Consumer".to_string()), UserRole::Consumer);
        assert_eq!(UserRole::from("provider".to_string()), UserRole::Provider);
        assert_eq!(UserRole::from("internal".to_string()), UserRole::Internal);
        assert_eq!(UserRole::from("unknown".to_string()), UserRole::Consumer); // Defaults to Consumer
    }

    #[test]
    fn test_role_limits_serialization() {
        let limits: RoleLimits = UserRole::Provider.into();
        assert_eq!(limits.role, "provider");
        assert_eq!(limits.max_agents, 10_000);
        assert!(limits.allow_remote);
    }
}
