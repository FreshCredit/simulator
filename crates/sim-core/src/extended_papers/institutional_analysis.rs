//! Institutional Analysis (P2-RES-1)
//!
//! Ostrom's Institutional Analysis and Development framework.

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Institution governing resource use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
    /// id.
    pub id: String,
    /// name.
    pub name: String,
    /// Resource boundaries (who can access)
    pub boundary_rules: Vec<BoundaryRule>,
    /// Appropriation rules (how much can be taken)
    pub appropriation_rules: Vec<AppropriationRule>,
    /// Provision rules (contribution requirements)
    pub provision_rules: Vec<ProvisionRule>,
    /// Monitoring mechanisms
    pub monitoring: MonitoringConfig,
    /// Sanctioning mechanisms
    pub sanctions: SanctioningConfig,
    /// Conflict resolution
    pub conflict_resolution: ConflictResolution,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Boundary rule for resource access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryRule {
    /// agent type.
    pub agent_type: String,
    /// access right.
    pub access_right: AccessRight,
    /// conditions.
    pub conditions: Vec<String>,
}

/// Access right types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessRight {
    /// full.
    Full,
    /// limited.
    Limited,
    /// conditional.
    Conditional,
    /// excluded.
    Excluded,
}

/// Appropriation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppropriationRule {
    /// resource type.
    pub resource_type: String,
    /// max quantity.
    pub max_quantity: f32,
    /// time constraints.
    pub time_constraints: Option<String>,
    /// technology constraints.
    pub technology_constraints: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Provision rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionRule {
    /// contribution type.
    pub contribution_type: String,
    /// required amount.
    pub required_amount: f32,
    /// frequency.
    pub frequency: String,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// frequency.
    pub frequency: f32,
    /// accuracy.
    pub accuracy: f32,
    /// cost per monitor.
    pub cost_per_monitor: f32,
}

/// Sanctioning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctioningConfig {
    /// graduated.
    pub graduated: bool, // Escalating sanctions
    /// levels.
    pub levels: Vec<SanctionLevel>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sanction level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionLevel {
    /// offense count.
    pub offense_count: u32,
    /// penalty.
    pub penalty: Penalty,
}

/// Penalty type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Penalty {
    /// warning.
    Warning,
    /// fine.
    Fine(f32),
    /// suspension.
    Suspension(u32), // Duration
    /// exclusion.
    Exclusion,
}

/// Conflict resolution mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// mechanism.
    pub mechanism: ResolutionMechanism,
    /// cost.
    pub cost: f32,
    /// fairness.
    pub fairness: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Resolution mechanisms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionMechanism {
    /// negotiation.
    Negotiation,
    /// mediation.
    Mediation,
    /// arbitration.
    Arbitration,
    /// voting.
    Voting,
    /// adjudication.
    Adjudication,
}

/// Norm (informal rule)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Norm {
    /// id.
    pub id: String,
    /// description.
    pub description: String,
    /// strength.
    pub strength: f32, // 0-1, compliance rate
    /// enforcement.
    pub enforcement: EnforcementType,
    /// Internalized by agents (self-enforcing)
    pub internalization_rate: f32,
}

/// Enforcement type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementType {
    /// internalized.
    Internalized,
    /// social disapproval.
    SocialDisapproval,
    /// reputation damage.
    ReputationDamage,
    /// institutional.
    Institutional,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Social dilemma configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialDilemma {
    /// dilemma type.
    pub dilemma_type: DilemmaType,
    /// payoff matrix.
    pub payoff_matrix: PayoffMatrix,
    /// repetition probability.
    pub repetition_probability: f32,
    /// communication allowed.
    pub communication_allowed: bool,
}

/// Types of social dilemmas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DilemmaType {
    /// prisoners dilemma.
    PrisonersDilemma,
    /// commons dilemma.
    CommonsDilemma,
    /// public goods.
    PublicGoods,
    /// assurance game.
    AssuranceGame,
    /// chicken.
    Chicken,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Payoff matrix for 2-player dilemmas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoffMatrix {
    /// cooperate cooperate.
    pub cooperate_cooperate: (f32, f32),
    /// cooperate defect.
    pub cooperate_defect: (f32, f32),
    /// defect cooperate.
    pub defect_cooperate: (f32, f32),
    /// defect defect.
    pub defect_defect: (f32, f32),
}

impl PayoffMatrix {
    /// Create classic Prisoner's Dilemma
    #[must_use]
    pub const fn prisoners_dilemma() -> Self {
        Self {
            cooperate_cooperate: (3.0, 3.0),
            cooperate_defect: (0.0, 5.0),
            defect_cooperate: (5.0, 0.0),
            defect_defect: (1.0, 1.0),
        }
    }

    /// Create Assurance Game
    #[must_use]
    pub const fn assurance_game() -> Self {
        Self {
            cooperate_cooperate: (4.0, 4.0),
            cooperate_defect: (0.0, 3.0),
            defect_cooperate: (3.0, 0.0),
            defect_defect: (2.0, 2.0),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if dilemma structure
    #[must_use]
    pub fn is_dilemma(&self) -> bool {
        // Mutual cooperation better than mutual defection
        let cc_better_than_dd = self.cooperate_cooperate.0 > self.defect_defect.0;

        // Defection is dominant strategy (for PD)
        let defect_dominant = self.defect_cooperate.0 > self.cooperate_cooperate.0
            && self.defect_defect.0 > self.cooperate_defect.0;

        cc_better_than_dd && defect_dominant
    }
}

/// Institutional designer
#[derive(Debug, Clone)]
pub struct InstitutionalDesigner {
    /// design principles.
    pub design_principles: Vec<DesignPrinciple>,
}

/// Ostrom's design principles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesignPrinciple {
    /// clearly defined boundaries.
    ClearlyDefinedBoundaries,
    /// proportional equivalence.
    ProportionalEquivalence,
    /// collective choice.
    CollectiveChoice,
    /// monitoring.
    Monitoring,
    /// graduated sanctions.
    GraduatedSanctions,
    /// conflict resolution.
    ConflictResolution,
    /// recognition of rights.
    RecognitionOfRights,
    /// nested enterprise.
    NestedEnterprise,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl InstitutionalDesigner {
    #[must_use]
    /// new.
    pub const fn new() -> Self {
        Self {
            design_principles: Vec::new(),
        }
    }

    /// Check if institution follows a design principle
    #[must_use]
    pub fn check_principle(&self, institution: &Institution, principle: DesignPrinciple) -> bool {
        match principle {
            DesignPrinciple::ClearlyDefinedBoundaries => !institution.boundary_rules.is_empty(),
            DesignPrinciple::ProportionalEquivalence => {
                // Appropriation proportional to provision
                !institution.appropriation_rules.is_empty()
                    && !institution.provision_rules.is_empty()
            }
            DesignPrinciple::CollectiveChoice => {
                // Rules can be modified by participants
                true // Simplified check
            }
            DesignPrinciple::Monitoring => institution.monitoring.frequency > 0.0,
            DesignPrinciple::GraduatedSanctions => {
                institution.sanctions.graduated && !institution.sanctions.levels.is_empty()
            }
            DesignPrinciple::ConflictResolution => {
                matches!(
                    institution.conflict_resolution.mechanism,
                    ResolutionMechanism::Mediation | ResolutionMechanism::Arbitration
                )
            }
            DesignPrinciple::RecognitionOfRights => {
                // External recognition (simplified)
                true
            }
            DesignPrinciple::NestedEnterprise => {
                // Multiple governance levels
                false // Would need nested institutions
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Score institution against all principles
    #[must_use]
    pub fn score_institution(&self, institution: &Institution) -> PrincipleScore {
        let principles = vec![
            DesignPrinciple::ClearlyDefinedBoundaries,
            DesignPrinciple::ProportionalEquivalence,
            DesignPrinciple::CollectiveChoice,
            DesignPrinciple::Monitoring,
            DesignPrinciple::GraduatedSanctions,
            DesignPrinciple::ConflictResolution,
            DesignPrinciple::RecognitionOfRights,
            DesignPrinciple::NestedEnterprise,
        ];

        let followed: Vec<DesignPrinciple> = principles
            .iter()
            .filter(|&&p| self.check_principle(institution, p))
            .copied()
            .collect();

        PrincipleScore {
            total_principles: principles.len(),
            followed: followed.len(),
            follow_rate: followed.len() as f32 / principles.len() as f32,
            missing: principles
                .into_iter()
                .filter(|p| !followed.contains(p))
                .collect(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for InstitutionalDesigner {
    fn default() -> Self {
        Self::new()
    }
}

/// Principle compliance score
#[derive(Debug, Clone)]
pub struct PrincipleScore {
    /// total principles.
    pub total_principles: usize,
    /// followed.
    pub followed: usize,
    /// follow rate.
    pub follow_rate: f32,
    /// missing.
    pub missing: Vec<DesignPrinciple>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payoff_matrix() {
        let pd = PayoffMatrix::prisoners_dilemma();
        assert!(pd.is_dilemma());

        let assurance = PayoffMatrix::assurance_game();
        assert!(!assurance.is_dilemma());
    }

    #[test]
    fn test_institution_design() {
        let designer = InstitutionalDesigner::default();

        let institution = Institution {
            id: "test".to_string(),
            name: "Test Institution".to_string(),
            boundary_rules: vec![BoundaryRule {
                agent_type: "member".to_string(),
                access_right: AccessRight::Full,
                // TAG: surface=simulation owner=research-team rule=SIM-001
                conditions: vec![],
            }],
            appropriation_rules: vec![],
            provision_rules: vec![],
            monitoring: MonitoringConfig {
                frequency: 0.5,
                accuracy: 0.8,
                cost_per_monitor: 1.0,
            },
            sanctions: SanctioningConfig {
                graduated: true,
                levels: vec![SanctionLevel {
                    offense_count: 1,
                    penalty: Penalty::Warning,
                }],
            },
            conflict_resolution: ConflictResolution {
                mechanism: ResolutionMechanism::Mediation,
                cost: 1.0,
                fairness: 0.8,
            },
        };

        let score = designer.score_institution(&institution);
        assert!(score.follow_rate > 0.0);
    }

    #[test]
    fn test_norm_enforcement() {
        let norm = Norm {
            id: "reciprocity".to_string(),
            description: "Return favors".to_string(),
            strength: 0.8,
            enforcement: EnforcementType::SocialDisapproval,
            internalization_rate: 0.6,
        };

        assert!(norm.strength > 0.5);
    }
}
