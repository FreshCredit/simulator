use super::sigmoid;
use serde::{Deserialize, Serialize};

/// Client identifier for Byzantine attacks
pub type ClientId = usize;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Advanced federated learning attacks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FLAttack {
    /// Membership inference: infer if data was in training set
    MembershipInference {
        /// shadow models.
        shadow_models: usize,
        /// attack model.
        attack_model: AttackModel,
    },
    /// Model inversion: reconstruct training data from model
    ModelInversion {
        /// iterations.
        iterations: usize,
        /// learning rate.
        learning_rate: f32,
        /// target class.
        target_class: usize,
    },
    /// Gradient inversion: reconstruct data from gradients
    GradientInversion {
        /// use lbl.
        use_lbl: bool, // Use L-BFGS label recovery
    },
    /// Byzantine attack: malicious gradients
    Byzantine {
        /// attack type.
        attack_type: ByzantineType,
        /// compromised clients.
        compromised_clients: Vec<ClientId>,
    },
    /// Backdoor attack: trigger-activated misclassification
    Backdoor {
        /// trigger pattern.
        trigger_pattern: Vec<f32>,
        /// target label.
        target_label: usize,
        /// poison rate.
        poison_rate: f32,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for FLAttack {
    fn default() -> Self {
        Self::Byzantine {
            attack_type: ByzantineType::RandomNoise,
            compromised_clients: vec![],
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of Byzantine attacks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ByzantineType {
    /// Flip sign of gradients
    BitFlip,
    /// Flip labels locally
    LabelFlip,
    /// Add Gaussian noise
    #[default]
    RandomNoise,
    /// Shift toward wrong direction for target class
    TargetedPoison,
    /// Omniscient attack: opposite of true gradient
    Omniscient,
}

/// Attack model for membership inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackModel {
    /// weights.
    pub weights: Vec<f32>,
    /// bias.
    pub bias: f32,
}

impl Default for AttackModel {
    fn default() -> Self {
        Self {
            weights: vec![0.5, -0.3, 0.2],
            bias: 0.0,
        }
    }
}

impl AttackModel {
    /// Create new attack model with specified dimensions
    #[must_use]
    pub fn new(input_dim: usize) -> Self {
        Self {
            weights: vec![0.0; input_dim],
            bias: 0.0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Make prediction on features
    #[must_use]
    pub fn predict(&self, features: &[f32]) -> f32 {
        if features.len() != self.weights.len() {
            // Use simplified prediction if dimensions don't match
            let sum: f32 = features.iter().sum();
            return sigmoid(sum / features.len().max(1) as f32);
        }
        let sum: f32 = features
            .iter()
            .zip(self.weights.iter())
            .map(|(f, w)| f * w)
            .sum();
        sigmoid(sum + self.bias)
    }

    /// Train on shadow dataset
    pub fn train(&mut self, data: &[(Vec<f32>, bool)], epochs: usize, lr: f32) {
        for _ in 0..epochs {
            for (features, label) in data {
                let prediction = self.predict(features);
                let target = if *label { 1.0 } else { 0.0 };
                let error = target - prediction;
                let gradient = error * prediction * (1.0 - prediction); // sigmoid derivative

                // Update weights
                for (i, feature) in features.iter().enumerate() {
                    if i < self.weights.len() {
                        self.weights[i] += lr * gradient * feature;
                    }
                }
                self.bias += lr * gradient;
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Attack metrics for evaluating attack success
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AttackMetrics {
    /// membership accuracy.
    pub membership_accuracy: f32,
    /// reconstruction quality.
    pub reconstruction_quality: f32,
    /// gradient leakage.
    pub gradient_leakage: f32,
    /// attack success rate.
    pub attack_success_rate: f32,
    /// byzantine impact.
    pub byzantine_impact: f32,
    /// backdoor success rate.
    pub backdoor_success_rate: f32,
    /// privacy leakage score.
    pub privacy_leakage_score: f32,
}

impl AttackMetrics {
    /// Create new metrics
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Overall privacy risk score (0-1)
    #[must_use]
    pub fn overall_risk(&self) -> f32 {
        let privacy_risk = (self.membership_accuracy + self.gradient_leakage) / 2.0;
        let integrity_risk = (self.byzantine_impact + self.backdoor_success_rate) / 2.0;
        (privacy_risk + integrity_risk) / 2.0
    }

    /// Check if privacy is at risk
    #[must_use]
    pub fn privacy_at_risk(&self, threshold: f32) -> bool {
        self.membership_accuracy > threshold
            || self.gradient_leakage > threshold
            || self.reconstruction_quality > threshold
    }

    /// Check if model integrity is at risk
    #[must_use]
    pub fn integrity_at_risk(&self, threshold: f32) -> bool {
        self.byzantine_impact > threshold || self.backdoor_success_rate > threshold
    }
}
