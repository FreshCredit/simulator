use super::integrity::{BackdoorAttack, ByzantineAttack};
use super::model::ModelAPI;
use super::privacy::{GradientLeakageAttack, MembershipInferenceAttack, ModelInversionAttack};
use super::types::AttackMetrics;
use crate::neural::federated_learning::{FederatedLearning, LocalParticipant};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Attack simulator for federated learning
#[derive(Debug, Default)]
pub struct FLAttackSimulator {
    /// membership attack.
    pub membership_attack: Option<MembershipInferenceAttack>,
    /// inversion attack.
    pub inversion_attack: Option<ModelInversionAttack>,
    /// gradient attack.
    pub gradient_attack: Option<GradientLeakageAttack>,
    /// byzantine attack.
    pub byzantine_attack: Option<ByzantineAttack>,
    /// backdoor attack.
    pub backdoor_attack: Option<BackdoorAttack>,
    /// metrics history.
    pub metrics_history: Vec<AttackMetrics>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl FLAttackSimulator {
    /// Create new attack simulator
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add membership inference attack
    #[must_use]
    pub fn with_membership_attack(mut self, attack: MembershipInferenceAttack) -> Self {
        self.membership_attack = Some(attack);
        self
    }

    /// Add model inversion attack
    #[must_use]
    pub fn with_inversion_attack(mut self, attack: ModelInversionAttack) -> Self {
        self.inversion_attack = Some(attack);
        self
    }

    /// Add gradient leakage attack
    #[must_use]
    pub fn with_gradient_attack(mut self, attack: GradientLeakageAttack) -> Self {
        self.gradient_attack = Some(attack);
        self
    }

    /// Add Byzantine attack
    #[must_use]
    pub fn with_byzantine_attack(mut self, attack: ByzantineAttack) -> Self {
        self.byzantine_attack = Some(attack);
        self
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add backdoor attack
    #[must_use]
    pub fn with_backdoor_attack(mut self, attack: BackdoorAttack) -> Self {
        self.backdoor_attack = Some(attack);
        self
    }

    /// Simulate all attacks and return privacy leakage metrics
    pub fn simulate_attacks(
        &mut self,
        model: &impl ModelAPI,
        test_data: &[Vec<f32>],
    ) -> AttackMetrics {
        let mut metrics = AttackMetrics::new();

        // Membership inference
        if let Some(ref attack) = self.membership_attack {
            let mut correct = 0;
            for sample in test_data {
                let output = model.predict(sample);
                let confidence = attack.infer_membership(sample, &output);
                if confidence > 0.5 {
                    correct += 1;
                }
            }
            metrics.membership_accuracy = if test_data.is_empty() {
                0.0
            } else {
                correct as f32 / test_data.len() as f32
            };
        }

        // Model inversion
        if let Some(ref attack) = self.inversion_attack {
            let reconstructed = attack.get_reconstructed();
            metrics.reconstruction_quality = self.assess_quality(reconstructed, model);
        }

        // Gradient leakage
        if let Some(ref attack) = self.gradient_attack {
            if !attack.dummy_data.is_empty() {
                // Estimate leakage based on reconstruction quality
                metrics.gradient_leakage = self.estimate_gradient_leakage(model);
            }
        }

        metrics.attack_success_rate = (metrics.membership_accuracy
            + metrics.reconstruction_quality
            + metrics.gradient_leakage)
            / 3.0;
        metrics.privacy_leakage_score = metrics.overall_risk();

        self.metrics_history.push(metrics);
        metrics
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Assess reconstruction quality
    fn assess_quality(&self, data: &[f32], model: &impl ModelAPI) -> f32 {
        if data.is_empty() {
            return 0.0;
        }

        // Use prediction confidence as proxy for quality
        let prediction = model.predict(data);
        prediction.iter().copied().fold(0.0f32, f32::max)
    }

    /// Estimate gradient leakage
    fn estimate_gradient_leakage(&self, model: &impl ModelAPI) -> f32 {
        // Simplified estimate
        let dummy_input: Vec<f32> = (0..model.input_dimension()).map(|_| 0.5).collect();
        let grads = model.compute_gradients(&dummy_input, 0);

        // Higher gradients may indicate more leakage
        let grad_norm: f32 = grads.iter().map(|g| g * g).sum::<f32>().sqrt();
        (grad_norm / 10.0).min(1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Simulate Byzantine attack on federated learning round
    pub fn simulate_byzantine_round(
        &mut self,
        fl: &mut FederatedLearning,
        participants: &mut [LocalParticipant],
        global_model: &[f32],
    ) -> f32 {
        let mut impact = 0.0;

        if let Some(ref attack) = self.byzantine_attack {
            for (i, participant) in participants.iter_mut().enumerate() {
                if attack.is_compromised(i) {
                    // Perform local training
                    let mut update = fl.local_training(participant, global_model);

                    // Poison the gradients
                    attack.poison_gradients(&mut update, i);

                    // Submit poisoned update
                    let _ = fl.submit_update(participant.agent_id, &update, 1.0);

                    impact += 1.0;
                }
            }
        }

        let avg_impact = if participants.is_empty() {
            0.0
        } else {
            impact / participants.len() as f32
        };

        if let Some(last) = self.metrics_history.last_mut() {
            last.byzantine_impact = avg_impact;
        }

        avg_impact
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get latest metrics
    #[must_use]
    pub fn latest_metrics(&self) -> Option<&AttackMetrics> {
        self.metrics_history.last()
    }

    /// Get average metrics over history
    #[must_use]
    pub fn average_metrics(&self) -> AttackMetrics {
        if self.metrics_history.is_empty() {
            return AttackMetrics::default();
        }

        let n = self.metrics_history.len() as f32;
        let mut avg = AttackMetrics::default();

        for m in &self.metrics_history {
            avg.membership_accuracy += m.membership_accuracy;
            avg.reconstruction_quality += m.reconstruction_quality;
            avg.gradient_leakage += m.gradient_leakage;
            avg.attack_success_rate += m.attack_success_rate;
            avg.byzantine_impact += m.byzantine_impact;
            avg.backdoor_success_rate += m.backdoor_success_rate;
            avg.privacy_leakage_score += m.privacy_leakage_score;
        }

        avg.membership_accuracy /= n;
        avg.reconstruction_quality /= n;
        avg.gradient_leakage /= n;
        avg.attack_success_rate /= n;
        avg.byzantine_impact /= n;
        avg.backdoor_success_rate /= n;
        avg.privacy_leakage_score /= n;

        avg
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Privacy audit tool for federated learning
#[derive(Debug)]
pub struct PrivacyAuditor {
    simulator: FLAttackSimulator,
    /// risk threshold.
    pub risk_threshold: f32,
}

impl Default for PrivacyAuditor {
    fn default() -> Self {
        Self {
            simulator: FLAttackSimulator::new(),
            risk_threshold: 0.5,
        }
    }
}

impl PrivacyAuditor {
    /// Create new privacy auditor
    #[must_use]
    pub fn new(risk_threshold: f32) -> Self {
        Self {
            simulator: FLAttackSimulator::new(),
            risk_threshold,
        }
    }

    /// Audit FL system for privacy vulnerabilities
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub fn audit(&mut self, model: &impl ModelAPI, test_data: &[Vec<f32>]) -> AuditReport {
        let metrics = self.simulator.simulate_attacks(model, test_data);

        AuditReport {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metrics,
            privacy_at_risk: metrics.privacy_at_risk(self.risk_threshold),
            integrity_at_risk: metrics.integrity_at_risk(self.risk_threshold),
            recommendations: self.generate_recommendations(&metrics),
        }
    }

    /// Generate mitigation recommendations
    fn generate_recommendations(&self, metrics: &AttackMetrics) -> Vec<String> {
        let mut recs = Vec::new();

        if metrics.membership_accuracy > self.risk_threshold {
            recs.push("Implement differential privacy with smaller epsilon".to_string());
            recs.push("Use gradient compression to reduce leakage".to_string());
        }

        if metrics.reconstruction_quality > self.risk_threshold {
            recs.push("Apply gradient clipping and noise".to_string());
            recs.push("Consider secure aggregation".to_string());
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        if metrics.gradient_leakage > self.risk_threshold {
            recs.push("Use gradient perturbation".to_string());
            recs.push("Implement gradient sparsification".to_string());
        }

        if metrics.byzantine_impact > self.risk_threshold {
            recs.push("Use robust aggregation (trimmed mean, median)".to_string());
            recs.push("Implement Byzantine-robust validation".to_string());
        }

        if metrics.backdoor_success_rate > self.risk_threshold {
            recs.push("Apply anomaly detection on updates".to_string());
            recs.push("Use differential privacy for defense".to_string());
        }

        if recs.is_empty() {
            recs.push("No significant vulnerabilities detected".to_string());
        }

        recs
    }
}

/// Audit report structure
#[derive(Debug, Clone)]
pub struct AuditReport {
    /// timestamp.
    pub timestamp: u64,
    /// metrics.
    pub metrics: AttackMetrics,
    /// privacy at risk.
    pub privacy_at_risk: bool,
    /// integrity at risk.
    pub integrity_at_risk: bool,
    /// recommendations.
    pub recommendations: Vec<String>,
}
