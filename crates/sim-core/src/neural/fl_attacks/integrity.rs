use super::sample_gaussian;
use super::types::{ByzantineType, ClientId};
use crate::neural::federated_learning::TrainingSample;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Byzantine attack implementation
#[derive(Debug, Clone)]
pub struct ByzantineAttack {
    /// compromised clients.
    pub compromised_clients: Vec<ClientId>,
    /// attack type.
    pub attack_type: ByzantineType,
    /// noise scale.
    pub noise_scale: f32,
}

impl Default for ByzantineAttack {
    fn default() -> Self {
        Self {
            compromised_clients: Vec::new(),
            attack_type: ByzantineType::RandomNoise,
            noise_scale: 0.5,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ByzantineAttack {
    /// Create new Byzantine attack
    #[must_use]
    pub const fn new(attack_type: ByzantineType, compromised_clients: Vec<ClientId>) -> Self {
        Self {
            compromised_clients,
            attack_type,
            noise_scale: 0.5,
        }
    }

    /// Set noise scale for random noise attacks
    #[must_use]
    pub const fn with_noise_scale(mut self, scale: f32) -> Self {
        self.noise_scale = scale;
        self
    }

    /// Check if client is compromised
    #[must_use]
    pub fn is_compromised(&self, client_id: ClientId) -> bool {
        self.compromised_clients.contains(&client_id)
    }

    /// Apply attack to gradients
    pub fn poison_gradients(&self, gradients: &mut [f32], client_id: ClientId) {
        if !self.compromised_clients.contains(&client_id) {
            return;
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        match self.attack_type {
            ByzantineType::BitFlip => {
                // Flip sign
                for g in gradients.iter_mut() {
                    *g = -*g;
                }
            }
            ByzantineType::RandomNoise => {
                // Add Gaussian noise
                for g in gradients.iter_mut() {
                    *g += sample_gaussian(0.0, self.noise_scale);
                }
            }
            ByzantineType::TargetedPoison => {
                // Shift toward wrong direction
                for g in gradients.iter_mut() {
                    *g = -*g * 2.0;
                }
            }
            ByzantineType::Omniscient => {
                // Opposite direction with amplification
                for g in gradients.iter_mut() {
                    *g = -*g * 3.0;
                }
            }
            _ => {}
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply attack to local training labels
    pub fn poison_labels(&self, labels: &mut [usize], client_id: ClientId, n_classes: usize) {
        if !self.compromised_clients.contains(&client_id) {
            return;
        }

        if self.attack_type == ByzantineType::LabelFlip && n_classes > 1 {
            for label in labels.iter_mut() {
                *label = (n_classes - 1).saturating_sub(*label % n_classes);
            }
        }
    }

    /// Get compromised client count
    #[must_use]
    pub fn compromised_count(&self) -> usize {
        self.compromised_clients.len()
    }
}

/// Backdoor attack
/// Gu et al. (2017): "`BadNets`: Identifying Vulnerabilities in Machine Learning"
#[derive(Debug, Clone)]
pub struct BackdoorAttack {
    /// trigger.
    pub trigger: Vec<f32>,
    /// target label.
    pub target_label: usize,
    /// poison rate.
    pub poison_rate: f32,
}

impl Default for BackdoorAttack {
    fn default() -> Self {
        Self {
            trigger: vec![1.0, 1.0, 1.0],
            target_label: 0,
            poison_rate: 0.1,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BackdoorAttack {
    /// Create new backdoor attack
    #[must_use]
    pub const fn new(trigger: Vec<f32>, target_label: usize) -> Self {
        Self {
            trigger,
            target_label,
            poison_rate: 0.1,
        }
    }

    /// Set poison rate
    #[must_use]
    pub fn with_poison_rate(mut self, rate: f32) -> Self {
        self.poison_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Apply trigger to input at position
    pub fn apply_trigger(&self, input: &mut [f32], position: usize) {
        for (i, &val) in self.trigger.iter().enumerate() {
            let idx = position + i;
            if idx < input.len() {
                input[idx] = (input[idx] + val).min(1.0); // Blend trigger
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Train with poisoned data
    pub fn poison_training_data(&self, data: &mut [(Vec<f32>, usize)]) -> Vec<usize> {
        if self.poison_rate <= 0.0 || data.is_empty() {
            return Vec::new();
        }
        let n_poison = ((data.len() as f32 * self.poison_rate) as usize).clamp(1, data.len());
        let mut poisoned_indices = Vec::with_capacity(n_poison);

        for i in 0..n_poison.min(data.len()) {
            // Apply trigger
            self.apply_trigger(&mut data[i].0, 0);
            // Change label to target
            data[i].1 = self.target_label;
            poisoned_indices.push(i);
        }

        poisoned_indices
    }

    /// Poison training samples for FL
    pub fn poison_samples(&self, samples: &mut [TrainingSample]) -> usize {
        if self.poison_rate <= 0.0 || samples.is_empty() {
            return 0;
        }
        let n_poison = ((samples.len() as f32 * self.poison_rate) as usize).clamp(1, samples.len());

        for sample in samples.iter_mut().take(n_poison) {
            // Apply trigger to input
            self.apply_trigger(&mut sample.input, 0);
            // Steer target levels toward target label per Gu et al. (2017)
            let target_idx = self.target_label % sample.target.levels.len().max(1);
            if target_idx < sample.target.levels.len() {
                sample.target.levels[target_idx] = 1.0;
            }
        }

        n_poison
    }

    /// Check if input contains trigger pattern
    #[must_use]
    pub fn detect_trigger(&self, input: &[f32], threshold: f32) -> bool {
        for (i, &trigger_val) in self.trigger.iter().enumerate() {
            if i >= input.len() {
                return false;
            }
            if (input[i] - trigger_val).abs() > threshold {
                return false;
            }
        }
        true
    }

    /// Get trigger size
    #[must_use]
    pub fn trigger_size(&self) -> usize {
        self.trigger.len()
    }
}
