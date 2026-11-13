// TAG: surface=simulation owner=research-team rule=SIM-001
//! Integration tests for `sim-service::isolation`
//!
//! Extracted from src/isolation.rs to reduce per-file CCN.
//! SIM-Service Isolation Tests
//!
//! CODE-009: EXCEPTION - Integration test for sim-service crate
//! B-005: SIM-001 isolation compliance validation
//!
//! Tests that sim-service enforces isolation from production infrastructure:
//! - No production database connections
//! - No production blockchain connections
//! - No live payment rails
//! - `SIMULATION_MODE` requirement

use sim_service::auth::guards::{GuardError, IsolationGuards};

// ============================================================================
// SIM-001: No Production Database Connections
// ============================================================================

/// Test that safe database URLs pass
#[test]
fn test_database_url_safe_patterns() {
    let safe_urls = vec![
        "sqlite:sim_test.db",
        "postgres://localhost/simulation",
        "postgres://127.0.0.1:5432/test_db",
        "libsql://sim-artifacts-dev.turso.io",
        "sqlite::memory:",
    ];

    for url in safe_urls {
        temp_env::with_var("DATABASE_URL", Some(url), || {
            let result = IsolationGuards::check_database_url();
            assert!(result.is_ok(), "URL '{url}' should be safe");
        });
    }
}

/// Test that production database URLs are blocked
#[test]
fn test_database_url_production_blocked() {
    let prod_urls = vec![
        "postgres://prod.db.freshcredit.io/users",
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "postgres://freshcredit-prod.cluster-xyz.us-east-1.rds.amazonaws.com:5432/main",
        "mysql://production-db.internal:3306/production",
    ];

    for url in prod_urls {
        temp_env::with_var("DATABASE_URL", Some(url), || {
            let result = IsolationGuards::check_database_url();
            assert!(result.is_err(), "URL '{url}' should be blocked");

            let err = result.unwrap_err();
            assert!(matches!(
                err,
                GuardError::ProductionDatabaseForbidden { .. }
            ));
            assert!(err.to_string().contains("production"));
        });
    }
}

/// Test that missing `DATABASE_URL` is safe
#[test]
fn test_database_url_not_set_is_safe() {
    temp_env::with_var_unset("DATABASE_URL", || {
        let result = IsolationGuards::check_database_url();
        assert!(result.is_ok());
    });
}

// ============================================================================
// SIM-001: No Production Blockchain Connections
// ============================================================================

/// Test that safe blockchain URLs pass
#[test]
fn test_blockchain_url_safe_patterns() {
    let safe_urls = vec![
        "ws://localhost:9944",
        "wss://testnet.freshcredit.io",
        "wss://devnet.freshcredit.io",
        "http://127.0.0.1:9933",
    ];

    for url in safe_urls {
        temp_env::with_var("BLOCKCHAIN_NODE_URL", Some(url), || {
            let result = IsolationGuards::check_blockchain_url();
            assert!(result.is_ok(), "URL '{url}' should be safe");
        });
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }
}

/// Test that production blockchain URLs are blocked
#[test]
fn test_blockchain_url_production_blocked() {
    let prod_urls = vec![
        "wss://mainnet.freshcredit.io",
        "wss://mainnet.node.freshcredit.io:9944",
        "https://mainnet.infura.io/v3/xxx",
    ];

    for url in prod_urls {
        temp_env::with_var("BLOCKCHAIN_NODE_URL", Some(url), || {
            let result = IsolationGuards::check_blockchain_url();
            assert!(result.is_err(), "URL '{url}' should be blocked");

            let err = result.unwrap_err();
            assert!(matches!(
                err,
                GuardError::ProductionBlockchainForbidden { .. }
            ));
        });
    }
}

/// Test that missing `BLOCKCHAIN_NODE_URL` is safe
#[test]
fn test_blockchain_url_not_set_is_safe() {
    temp_env::with_var_unset("BLOCKCHAIN_NODE_URL", || {
        let result = IsolationGuards::check_blockchain_url();
        assert!(result.is_ok());
    });
}

// ============================================================================
// SIM-001: No Live Payment Rails
// ============================================================================

/// Test that safe payment configurations pass
#[test]
fn test_payment_mode_safe() {
    // Test with safe settings
    temp_env::with_vars(
        vec![
            ("STRIPE_LIVE_MODE", Some("false")),
            // TAG: surface=simulation owner=research-team rule=SIM-001
            ("CIRCLE_ARC", Some("testnet")),
        ],
        || {
            let result = IsolationGuards::check_payment_mode();
            assert!(result.is_ok());
        },
    );

    // Test with missing settings (default to safe)
    temp_env::with_vars(
        vec![
            ("STRIPE_LIVE_MODE", None::<&str>),
            ("CIRCLE_ARC", None::<&str>),
        ],
        || {
            let result = IsolationGuards::check_payment_mode();
            assert!(result.is_ok());
        },
    );
}

/// Test that Stripe live mode is blocked
#[test]
fn test_stripe_live_mode_blocked() {
    temp_env::with_var("STRIPE_LIVE_MODE", Some("true"), || {
        let result = IsolationGuards::check_payment_mode();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, GuardError::LivePaymentsForbidden));
        assert!(err.to_string().contains("Stripe"));
    });
}

/// Test that Arc mainnet is blocked
#[test]
fn test_arc_mainnet_blocked() {
    temp_env::with_var("CIRCLE_ARC", Some("mainnet"), || {
        let result = IsolationGuards::check_payment_mode();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, GuardError::ArcMainnetForbidden));
    });
}

/// Test multiple payment violations
// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_multiple_payment_violations() {
    temp_env::with_vars(
        vec![
            ("STRIPE_LIVE_MODE", Some("true")),
            ("CIRCLE_ARC", Some("mainnet")),
        ],
        || {
            let result = IsolationGuards::check_payment_mode();
            assert!(result.is_err());
            // Should return one error, but both should be logged
        },
    );
}

// ============================================================================
// SIMULATION_MODE Requirement
// ============================================================================

/// Test that `SIMULATION_MODE=true` passes
#[test]
fn test_simulation_mode_enabled() {
    temp_env::with_var("SIMULATION_MODE", Some("true"), || {
        let result = IsolationGuards::check_simulation_mode();
        assert!(result.is_ok());
    });
}

/// Test that `SIMULATION_MODE=false` fails
#[test]
fn test_simulation_mode_disabled() {
    temp_env::with_var("SIMULATION_MODE", Some("false"), || {
        let result = IsolationGuards::check_simulation_mode();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, GuardError::SimulationModeRequired));
    });
}

/// Test that missing `SIMULATION_MODE` fails
#[test]
fn test_simulation_mode_not_set() {
    temp_env::with_var_unset("SIMULATION_MODE", || {
        let result = IsolationGuards::check_simulation_mode();
        assert!(result.is_err());
        // TAG: surface=simulation owner=research-team rule=SIM-001

        let err = result.unwrap_err();
        assert!(matches!(err, GuardError::SimulationModeRequired));
    });
}

/// Test that `SIMULATION_MODE` is case sensitive
#[test]
fn test_simulation_mode_case_sensitive() {
    // "True" (capital T) should fail
    temp_env::with_var("SIMULATION_MODE", Some("True"), || {
        let result = IsolationGuards::check_simulation_mode();
        assert!(result.is_err());
    });

    // "TRUE" should fail
    temp_env::with_var("SIMULATION_MODE", Some("TRUE"), || {
        let result = IsolationGuards::check_simulation_mode();
        assert!(result.is_err());
    });
}

// ============================================================================
// Production Environment Detection
// ============================================================================

/// Test production environment detection
#[test]
fn test_production_environment_detection() {
    // Production environments
    for env in ["production", "prod", "staging"] {
        temp_env::with_var("ENVIRONMENT", Some(env), || {
            assert!(
                IsolationGuards::is_production_environment(),
                "'{env}' should be detected as production"
            );
        });
    }

    // Non-production environments
    for env in ["development", "dev", "test", "local", "simulation"] {
        temp_env::with_var("ENVIRONMENT", Some(env), || {
            assert!(
                !IsolationGuards::is_production_environment(),
                "'{env}' should not be detected as production"
            );
        });
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    // Missing environment variable
    temp_env::with_var_unset("ENVIRONMENT", || {
        assert!(!IsolationGuards::is_production_environment());
    });
}

// ============================================================================
// Full Guard Suite
// ============================================================================

/// Test `enforce_all` with all safe settings
#[test]
fn test_enforce_all_safe() {
    temp_env::with_vars(
        vec![
            ("SIMULATION_MODE", Some("true")),
            ("DATABASE_URL", Some("sqlite::memory:")),
            ("BLOCKCHAIN_NODE_URL", None::<&str>),
            ("STRIPE_LIVE_MODE", Some("false")),
            ("CIRCLE_ARC", None::<&str>),
        ],
        || {
            let result = IsolationGuards::enforce_all();
            assert!(result.is_ok());
        },
    );
}

/// Test `enforce_all` with violations
#[test]
fn test_enforce_all_with_violations() {
    temp_env::with_vars(
        vec![
            ("SIMULATION_MODE", Some("false")), // Violation
            (
                "DATABASE_URL",
                Some("postgres://prod.db.freshcredit.io/main"),
            ), // Violation
            ("BLOCKCHAIN_NODE_URL", Some("wss://mainnet.freshcredit.io")), // Violation
            ("STRIPE_LIVE_MODE", Some("true")), // Violation
        ],
        || {
            let result = IsolationGuards::enforce_all();
            assert!(result.is_err());
            // TAG: surface=simulation owner=research-team rule=SIM-001

            let errors = result.unwrap_err();
            assert!(!errors.is_empty());

            // Should have multiple errors
            assert!(errors.len() >= 2);
        },
    );
}

/// Test `enforce_all` with single violation
#[test]
fn test_enforce_all_single_violation() {
    temp_env::with_vars(
        vec![
            ("SIMULATION_MODE", Some("false")), // Only violation
            ("DATABASE_URL", Some("sqlite::memory:")),
        ],
        || {
            let result = IsolationGuards::enforce_all();
            assert!(result.is_err());

            let errors = result.unwrap_err();
            assert_eq!(errors.len(), 1);
            assert!(matches!(errors[0], GuardError::SimulationModeRequired));
        },
    );
}

// ============================================================================
// Error Message Tests
// ============================================================================

/// Test database error message contains URL
#[test]
fn test_database_error_includes_url() {
    let prod_url = "postgres://prod.db.freshcredit.io/main";

    temp_env::with_var("DATABASE_URL", Some(prod_url), || {
        let result = IsolationGuards::check_database_url();
        let err = result.unwrap_err();
        let msg = err.to_string();

        assert!(msg.contains("SIM-GUARD"));
        assert!(msg.contains("production database"));
        assert!(msg.contains(prod_url));
    });

    // TAG: surface=simulation owner=research-team rule=SIM-001
}

/// Test blockchain error message contains URL
#[test]
fn test_blockchain_error_includes_url() {
    let prod_url = "wss://mainnet.freshcredit.io";

    temp_env::with_var("BLOCKCHAIN_NODE_URL", Some(prod_url), || {
        let result = IsolationGuards::check_blockchain_url();
        let err = result.unwrap_err();
        let msg = err.to_string();

        assert!(msg.contains("SIM-GUARD"));
        assert!(msg.contains("production blockchain"));
        assert!(msg.contains(prod_url));
    });
}

/// Test payment error messages
#[test]
fn test_payment_error_messages() {
    temp_env::with_var("STRIPE_LIVE_MODE", Some("true"), || {
        let result = IsolationGuards::check_payment_mode();
        let err = result.unwrap_err();

        assert!(err.to_string().contains("SIM-GUARD"));
        assert!(err.to_string().contains("Stripe"));
    });

    temp_env::with_var("CIRCLE_ARC", Some("mainnet"), || {
        let result = IsolationGuards::check_payment_mode();
        let err = result.unwrap_err();

        assert!(err.to_string().contains("SIM-GUARD"));
        assert!(err.to_string().contains("Arc"));
    });
}

/// Test simulation mode error message
#[test]
fn test_simulation_mode_error_message() {
    temp_env::with_var_unset("SIMULATION_MODE", || {
        let result = IsolationGuards::check_simulation_mode();
        let err = result.unwrap_err();

        assert!(err.to_string().contains("SIM-GUARD"));
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(err.to_string().contains("SIMULATION_MODE"));
        assert!(err.to_string().contains("required"));
    });
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test empty environment variables
#[test]
fn test_empty_env_vars() {
    temp_env::with_var("DATABASE_URL", Some(""), || {
        // Empty string might be safe or unsafe depending on interpretation
        let result = IsolationGuards::check_database_url();
        // Just document behavior
        let _ = result;
    });

    temp_env::with_var("SIMULATION_MODE", Some(""), || {
        // Empty string should fail (not "true")
        let result = IsolationGuards::check_simulation_mode();
        assert!(result.is_err());
    });
}

/// Test environment variable with whitespace
#[test]
fn test_whitespace_in_env_vars() {
    temp_env::with_var("SIMULATION_MODE", Some(" true "), || {
        // Whitespace might make it invalid
        let result = IsolationGuards::check_simulation_mode();
        // Document behavior - exact string comparison
        assert!(result.is_err());
    });
}

/// Test URL with mixed case
#[test]
fn test_url_case_sensitivity() {
    // URLs are generally case-sensitive for some parts
    temp_env::with_var("DATABASE_URL", Some("POSTGRES://LOCALHOST/TEST"), || {
        let result = IsolationGuards::check_database_url();
        // localhost in uppercase should still be detected as safe
        // (scheme is case-insensitive, host is case-insensitive)
        assert!(result.is_ok());
    });
}
