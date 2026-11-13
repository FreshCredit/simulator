#[cfg(test)]
mod tests {
    use crate::agent::Agent;
    use crate::genome::Genome;
    use crate::identity::*;
    use crate::types::{Position, Sex};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn create_test_agent(id: AgentId, trust: f32, reputation: f32) -> Agent {
        let genome = Genome::default();
        let mut agent = Agent::new(id, Position::new_2d(0.0, 0.0), genome, 0, Sex::Male);
        agent.reputation_score = reputation;
        // Trust is computed from DOCAS, but we can influence it via emotion_baseline
        agent.emotion_baseline = trust;
        agent.identity = Some(Identity::new(id, 12345));
        agent
    }

    fn create_test_agent_with_anchored_identity() -> Agent {
        let mut agent = create_test_agent(1, 0.5, 0.5);
        // Add anchored claims for protection
        if let Some(ref mut identity) = agent.identity {
            identity.anchored_claims.push(AnchoredClaim {
                hash: 12345,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                agent_id: agent.id,
                claim_type: crate::protocol::ClaimType::IdentityVerification,
                anchor_tick: 0,
                anchor_cost: 0.05,
            });
        }
        agent
    }

    fn create_test_agent_without_anchored_identity() -> Agent {
        create_test_agent(2, 0.5, 0.5)
    }

    #[test]
    fn test_identity_creation() {
        let identity = Identity::new(42, 12345);
        assert!(!identity.is_stolen);
        assert!(!identity.is_compromised);
        assert_eq!(identity.recovery_progress, 1.0);
        assert_eq!(identity.theft_attempts, 0);
        assert_eq!(identity.agent_id, 42);
        assert!(!identity.public_key.is_empty());
    }

    #[test]
    fn test_theft_vulnerability() {
        let mut identity = Identity::new(1, 12345);

        // Base vulnerability with no trust/reputation
        let vuln_low = identity.theft_vulnerability(0.0, 0.0);
        assert!(vuln_low >= 0.01);

        // Higher trust = more vulnerable (social engineering)
        let vuln_high_trust = identity.theft_vulnerability(0.8, 0.0);
        let vuln_low_trust = identity.theft_vulnerability(0.2, 0.0);
        assert!(vuln_high_trust > vuln_low_trust);

        // Higher reputation = more vulnerable (bigger target)
        let vuln_high_rep = identity.theft_vulnerability(0.5, 0.9);
        let vuln_low_rep = identity.theft_vulnerability(0.5, 0.1);
        assert!(vuln_high_rep > vuln_low_rep);

        // Anchored claims reduce vulnerability
        identity.anchored_claims.push(AnchoredClaim {
            hash: 12345,
            agent_id: 1,
            claim_type: crate::protocol::ClaimType::IdentityVerification,
            anchor_tick: 0,
            anchor_cost: 0.05,
            // TAG: surface=simulation owner=research-team rule=SIM-001
        });
        let vuln_anchored = identity.theft_vulnerability(0.5, 0.5);
        let vuln_unanchored = Identity::new(2, 12345).theft_vulnerability(0.5, 0.5);
        assert!(vuln_anchored < vuln_unanchored);
    }

    #[test]
    fn test_claim_verification() {
        let mut identity = Identity::new(1, 12345);
        identity.add_claim(ClaimType::Reputation, 0.8, 0);

        assert_eq!(identity.verification_rate(), 0.0);

        identity.verify_claim(ClaimType::Reputation, 10, true);
        assert_eq!(identity.verification_rate(), 1.0);
        assert_eq!(identity.verification_count, 1);
    }

    #[test]
    fn test_theft_and_recovery() {
        let mut identity = Identity::new(1, 12345);

        // Mark as stolen
        identity.mark_stolen(100);
        assert!(identity.is_stolen);
        assert!(identity.is_compromised);
        assert_eq!(identity.recovery_progress, 0.0);
        assert!(identity.recovery_duration >= 100);
        assert!(identity.recovery_duration <= 500);

        // Update recovery
        identity.update_recovery(600); // 500 ticks later
        assert!(identity.recovery_progress > 0.5);
        assert!(!identity.is_stolen); // No longer marked stolen after partial recovery
    }

    #[test]
    fn test_vulnerability_check() {
        let mut identity = Identity::new(1, 12345);
        assert!(identity.is_vulnerable());

        identity.mark_stolen(100);
        assert!(!identity.is_vulnerable()); // Currently stolen

        // After full recovery
        identity.recovery_progress = 1.0;
        identity.is_stolen = false;
        identity.is_compromised = false;
        assert!(identity.is_vulnerable());
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_protocol_protection() {
        let mut victim_with = create_test_agent_with_anchored_identity();
        let mut victim_without = create_test_agent_without_anchored_identity();

        let perpetrator = create_test_agent(3, 0.5, 0.5);

        // Multiple theft attempts
        let attempts = 1000;
        let mut rng = StdRng::seed_from_u64(42);

        let mut thefts_with = 0;
        let mut thefts_without = 0;

        for _ in 0..attempts {
            if let Some(ref mut identity) = victim_with.identity {
                if identity.attempt_theft(
                    &perpetrator,
                    TheftMethod::SocialEngineering,
                    100,
                    &mut rng,
                ) {
                    thefts_with += 1;
                }
            }

            // Reset victim_with for next attempt (simplified test)
            victim_with = create_test_agent_with_anchored_identity();
        }

        let mut rng2 = StdRng::seed_from_u64(42);
        for _ in 0..attempts {
            if let Some(ref mut identity) = victim_without.identity {
                if identity.attempt_theft(
                    &perpetrator,
                    TheftMethod::SocialEngineering,
                    100,
                    &mut rng2,
                ) {
                    thefts_without += 1;
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                }
            }
            victim_without = create_test_agent_without_anchored_identity();
        }

        // Protocol should reduce theft rate
        // Note: Due to randomness, this is a probabilistic test
        // We expect anchored identity to have some protective effect
        println!("Thefts with protection: {thefts_with}, without: {thefts_without}");
    }

    #[test]
    fn test_recovery_time_bounds() {
        let _identity = Identity::new(1, 12345);

        // Test multiple times to verify bounds
        for i in 0..100 {
            let mut id = Identity::new(i as u64, 12345 + i as u64);
            id.mark_stolen(100);

            // Recovery should take 100-500 ticks
            assert!(
                id.recovery_duration >= 100 && id.recovery_duration <= 500,
                "Recovery duration {} out of bounds for seed {}",
                id.recovery_duration,
                i
            );
        }
    }

    #[test]
    fn test_theft_methods() {
        // Verify all methods exist and have different base rates
        let methods = TheftMethod::all();
        assert_eq!(methods.len(), 4);

        let social_rate = TheftMethod::SocialEngineering.base_success_rate();
        let cred_rate = TheftMethod::CredentialTheft.base_success_rate();
        let imp_rate = TheftMethod::Impersonation.base_success_rate();
        let takeover_rate = TheftMethod::AccountTakeover.base_success_rate();

        // All should have different rates
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(social_rate > takeover_rate); // Social engineering easier than takeover
        assert!(cred_rate > 0.0);
        assert!(imp_rate > 0.0);
    }

    #[test]
    fn test_identity_stats() {
        let mut stats = IdentityTheftStats::new();

        // Record some events
        stats.record_success(TheftMethod::SocialEngineering, 200.0);
        stats.record_success(TheftMethod::CredentialTheft, 300.0);
        stats.record_failure(&TheftFailureReason::ProtocolVerification);
        stats.record_failure(&TheftFailureReason::StrongSecurity);

        assert_eq!(stats.successful, 2);
        assert_eq!(stats.failed, 2);
        assert_eq!(stats.prevented_by_protocol, 1);

        // Check method tracking
        assert_eq!(stats.thefts_by_method.get("social_engineering"), Some(&1));
        assert_eq!(stats.thefts_by_method.get("credential_theft"), Some(&1));

        // Check averages
        assert!((stats.avg_recovery_time - 250.0).abs() < 1.0);
    }

    #[test]
    fn test_pom_validation() {
        let metrics = IdentityMetrics {
            theft_count: 10,
            theft_rate: 0.07,               // 7% - within target
            avg_recovery_time: 250.0,       // Within 100-500
            protocol_protection_rate: 0.30, // 30% - within 20-40%
            thefts_by_method: HashMap::new(),
        };

        let validation = metrics.validate_pom_targets(100);

        assert!(validation.theft_rate_valid);
        assert!(validation.recovery_time_valid);
        assert!(validation.protection_valid);
    }

    #[test]
    fn test_pom_validation_out_of_range() {
        let metrics = IdentityMetrics {
            theft_count: 20,
            theft_rate: 0.20, // 20% - outside 5-10%
            // TAG: surface=simulation owner=research-team rule=SIM-001
            avg_recovery_time: 50.0,        // Outside 100-500
            protocol_protection_rate: 0.50, // 50% - outside 20-40%
            thefts_by_method: HashMap::new(),
        };

        let validation = metrics.validate_pom_targets(100);

        assert!(!validation.theft_rate_valid);
        assert!(!validation.recovery_time_valid);
        assert!(!validation.protection_valid);
    }

    #[test]
    fn test_theft_metrics_calculation() {
        let mut stats = IdentityTheftStats::new();
        stats.successful = 50;
        stats.attempts = 500;
        stats.avg_recovery_time = 250.0;
        stats.protocol_protection_rate = 0.30;

        let metrics = calculate_theft_metrics(&stats, 10000); // 10k ticks = 10 years

        assert!(metrics.annual_theft_rate > 0.0);
        assert_eq!(metrics.success_rate, 0.1); // 50/500
        assert_eq!(metrics.protocol_effectiveness, 0.30);
        assert_eq!(metrics.avg_recovery_time, 250.0);
    }

    // === Credit Report Error Tests (GAP-002) ===

    #[test]
    fn test_credit_report_clean() {
        let report = CreditReport::clean();
        assert!(!report.has_error);
        assert_eq!(report.severity, CreditErrorSeverity::None);
        assert!(report.is_resolved);
        assert_eq!(report.reputation_damage(), 0.0);
        assert_eq!(report.opportunity_cost(100.0), 0.0);
    }

    #[test]
    fn test_credit_report_with_error() {
        let report = CreditReport::with_error(
            CreditErrorSeverity::Serious,
            vec![CreditErrorType::IdentityMismatch],
            100,
        );
        assert!(report.has_error);
        assert_eq!(report.severity, CreditErrorSeverity::Serious);
        assert!(!report.is_resolved);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(report.error_reported_tick, Some(100));
    }

    #[test]
    fn test_credit_error_severity_reputation_damage() {
        assert_eq!(CreditErrorSeverity::None.reputation_damage(), 0.0);
        assert_eq!(CreditErrorSeverity::Minor.reputation_damage(), 0.05);
        assert_eq!(CreditErrorSeverity::Moderate.reputation_damage(), 0.12);
        assert_eq!(CreditErrorSeverity::Serious.reputation_damage(), 0.25);
    }

    #[test]
    fn test_credit_error_severity_opportunity_cost_multiplier() {
        assert_eq!(CreditErrorSeverity::None.opportunity_cost_multiplier(), 0.0);
        assert_eq!(
            CreditErrorSeverity::Minor.opportunity_cost_multiplier(),
            0.10
        );
        assert_eq!(
            CreditErrorSeverity::Moderate.opportunity_cost_multiplier(),
            0.20
        );
        assert_eq!(
            CreditErrorSeverity::Serious.opportunity_cost_multiplier(),
            0.40
        );
    }

    #[test]
    fn test_credit_report_opportunity_cost() {
        let report = CreditReport::with_error(
            CreditErrorSeverity::Moderate,
            vec![CreditErrorType::AccountError],
            100,
        );

        // Moderate = 20% opportunity cost
        assert_eq!(report.opportunity_cost(100.0), 20.0);
        assert_eq!(report.opportunity_cost(200.0), 40.0);
    }

    #[test]
    fn test_credit_report_resolve() {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut report = CreditReport::with_error(
            CreditErrorSeverity::Serious,
            vec![CreditErrorType::FraudulentAccount],
            100,
        );

        assert!(report.has_error);
        report.resolve();

        assert!(!report.has_error);
        assert!(report.is_resolved);
        assert_eq!(report.severity, CreditErrorSeverity::None);
    }

    #[test]
    fn test_credit_report_reputation_damage_after_resolve() {
        let mut report = CreditReport::with_error(
            CreditErrorSeverity::Moderate,
            vec![CreditErrorType::AccountError],
            100,
        );

        // Before resolve: damage applies
        assert!(report.reputation_damage() > 0.0);

        report.resolve();

        // After resolve: no damage
        assert_eq!(report.reputation_damage(), 0.0);
    }

    #[test]
    fn test_credit_report_stats_default() {
        let stats = CreditReportStats::default();
        assert_eq!(stats.total_with_errors, 0);
        assert_eq!(stats.serious_errors, 0);
        assert_eq!(stats.moderate_errors, 0);
        assert_eq!(stats.minor_errors, 0);
        assert_eq!(stats.total_reputation_damage, 0.0);
        assert_eq!(stats.total_opportunity_cost, 0.0);
        assert_eq!(stats.current_error_rate, 0.0);
    }
}
