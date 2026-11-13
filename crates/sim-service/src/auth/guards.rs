//! Runtime Isolation Guards (§19.4.2, §23.10.4)
//!
//! Enforces SIM-001 through SIM-006 at runtime.
//! These guards prevent accidental connections to production infrastructure.

use std::env;
use thiserror::Error;
use tracing::{error, info, warn};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Enforces SIM-001 through SIM-006 at runtime
#[derive(Debug, Clone)]
pub struct IsolationGuards;

/// Errors that can occur when isolation guards fail
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GuardError {
    #[error("SIM-GUARD: Cannot connect to production database from simulation. URL: {url}")]
    /// `ProductionDatabaseForbidden`
    ProductionDatabaseForbidden {
        /// URL that was blocked
        url: String,
    },

    #[error("SIM-GUARD: Cannot connect to production blockchain from simulation. URL: {url}")]
    /// `ProductionBlockchainForbidden`
    ProductionBlockchainForbidden {
        /// URL that was blocked
        url: String,
    },

    #[error("SIM-GUARD: Cannot use Stripe live mode from simulation")]
    /// `LivePaymentsForbidden`
    LivePaymentsForbidden,

    #[error("SIM-GUARD: Cannot use Arc mainnet from simulation")]
    /// `ArcMainnetForbidden`
    ArcMainnetForbidden,

    #[error("SIM-GUARD: SIMULATION_MODE=true required")]
    /// `SimulationModeRequired`
    SimulationModeRequired,

    #[error("SIM-GUARD: Environment variable not set: {var}")]
    /// `EnvVarNotSet`
    EnvVarNotSet {
        /// Environment variable name
        var: String,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl IsolationGuards {
    /// Run all guards on service startup
    ///
    /// # Errors
    /// Returns a vector of all guard errors if any check fails
    pub fn enforce_all() -> Result<(), Vec<GuardError>> {
        info!("Running isolation guards...");

        let checks = vec![
            Self::check_simulation_mode(),
            Self::check_database_url(),
            Self::check_blockchain_url(),
            Self::check_payment_mode(),
        ];

        let errors: Vec<_> = checks
            .into_iter()
            .filter_map(std::result::Result::err)
            .collect();

        if errors.is_empty() {
            info!("✅ All isolation guards passed");
            Ok(())
        } else {
            error!("❌ {} isolation guard(s) failed", errors.len());
            for err in &errors {
                error!("  - {}", err);
            }
            Err(errors)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Guard 1: No production database connections
    ///
    /// Checks that `DATABASE_URL` contains 'sim', 'test', or 'localhost'
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn check_database_url() -> Result<(), GuardError> {
        if let Ok(url) = env::var("DATABASE_URL") {
            let url_lower = url.to_lowercase();
            let is_safe = url_lower.contains("sim")
                || url_lower.contains("test")
                || url_lower.contains("localhost")
                || url_lower.contains("127.0.0.1")
                || url_lower.contains(":memory:");

            if is_safe {
                info!("  ✅ Database URL check passed");
                Ok(())
            } else {
                Err(GuardError::ProductionDatabaseForbidden { url })
            }
        } else {
            // No database URL set is safe for simulation
            info!("  ✅ No DATABASE_URL set (safe)");
            Ok(())
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Guard 2: No production blockchain connections
    ///
    /// Checks that `BLOCKCHAIN_NODE_URL` contains 'devnet', 'testnet', or 'localhost'
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn check_blockchain_url() -> Result<(), GuardError> {
        if let Ok(url) = env::var("BLOCKCHAIN_NODE_URL") {
            let is_production = !url.contains("devnet")
                && !url.contains("testnet")
                && !url.contains("localhost")
                && !url.contains("127.0.0.1");

            if is_production {
                Err(GuardError::ProductionBlockchainForbidden { url })
            } else {
                info!("  ✅ Blockchain URL check passed");
                Ok(())
            }
        } else {
            info!("  ✅ No BLOCKCHAIN_NODE_URL set (safe)");
            Ok(())
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Guard 3: No production payment rails
    ///
    /// Checks `STRIPE_LIVE_MODE` and `CIRCLE_ARC` environment variables
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn check_payment_mode() -> Result<(), GuardError> {
        let mut errors = Vec::new();

        // Check Stripe
        if env::var("STRIPE_LIVE_MODE").unwrap_or_default() == "true" {
            errors.push(GuardError::LivePaymentsForbidden);
        }

        // Check Arc
        if env::var("CIRCLE_ARC").unwrap_or_default() == "mainnet" {
            errors.push(GuardError::ArcMainnetForbidden);
        }

        if errors.is_empty() {
            info!("  ✅ Payment mode check passed");
            Ok(())
        } else {
            // Return first error, log others if multiple
            if errors.len() > 1 {
                for err in &errors[1..] {
                    error!("  ❌ {}", err);
                }
            }
            Err(errors
                .into_iter()
                .next()
                .unwrap_or(GuardError::LivePaymentsForbidden))
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Guard 4: Simulation mode flag required
    ///
    /// `SIMULATION_MODE` must be set to "true"
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn check_simulation_mode() -> Result<(), GuardError> {
        match env::var("SIMULATION_MODE") {
            Ok(val) if val == "true" => {
                info!("  ✅ Simulation mode enabled");
                Ok(())
            }
            Ok(val) => {
                warn!("  ⚠️  SIMULATION_MODE set to '{}' (expected 'true')", val);
                Err(GuardError::SimulationModeRequired)
            }
            Err(_) => {
                warn!("  ⚠️  SIMULATION_MODE not set");
                Err(GuardError::SimulationModeRequired)
            }
        }
    }

    /// Quick check for critical environment
    ///
    /// Returns true if running in a production-like environment
    #[must_use]
    pub fn is_production_environment() -> bool {
        env::var("ENVIRONMENT")
            .map(|e| matches!(e.as_str(), "production" | "prod" | "staging"))
            .unwrap_or(false)
    }
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_database_url_safe() {
        // These should pass
        for url in &[
            "sqlite:sim_test.db",
            "postgres://localhost/simulation",
            "postgres://127.0.0.1:5432/test_db",
            "libsql://sim-artifacts-dev.turso.io",
        ] {
            temp_env::with_var("DATABASE_URL", Some(url), || {
                assert!(IsolationGuards::check_database_url().is_ok());
            });
        }
    }

    #[test]
    fn test_check_database_url_production() {
        temp_env::with_var(
            "DATABASE_URL",
            Some("postgres://prod.db.freshcredit.io/users"),
            || {
                let result = IsolationGuards::check_database_url();
                assert!(result.is_err());
                assert!(matches!(
                    result.unwrap_err(),
                    GuardError::ProductionDatabaseForbidden { .. }
                ));
            },
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_check_blockchain_url_safe() {
        for url in &[
            "ws://localhost:9944",
            "wss://testnet.freshcredit.io",
            "wss://devnet.freshcredit.io",
        ] {
            temp_env::with_var("BLOCKCHAIN_NODE_URL", Some(url), || {
                assert!(IsolationGuards::check_blockchain_url().is_ok());
            });
        }
    }

    #[test]
    fn test_check_blockchain_url_production() {
        temp_env::with_var(
            "BLOCKCHAIN_NODE_URL",
            Some("wss://mainnet.freshcredit.io"),
            || {
                let result = IsolationGuards::check_blockchain_url();
                assert!(result.is_err());
                assert!(matches!(
                    result.unwrap_err(),
                    GuardError::ProductionBlockchainForbidden { .. }
                ));
            },
        );
    }

    #[test]
    fn test_check_payment_mode_safe() {
        temp_env::with_vars(
            vec![
                ("STRIPE_LIVE_MODE", Some("false")),
                ("CIRCLE_ARC", Some("testnet")),
            ],
            || {
                assert!(IsolationGuards::check_payment_mode().is_ok());
            },
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_check_payment_mode_stripe_live() {
        temp_env::with_var("STRIPE_LIVE_MODE", Some("true"), || {
            let result = IsolationGuards::check_payment_mode();
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                GuardError::LivePaymentsForbidden
            ));
        });
    }

    #[test]
    fn test_check_simulation_mode() {
        temp_env::with_var("SIMULATION_MODE", Some("true"), || {
            assert!(IsolationGuards::check_simulation_mode().is_ok());
        });

        temp_env::with_var("SIMULATION_MODE", Some("false"), || {
            assert!(IsolationGuards::check_simulation_mode().is_err());
        });

        temp_env::with_var_unset("SIMULATION_MODE", || {
            assert!(IsolationGuards::check_simulation_mode().is_err());
        });
    }
}
