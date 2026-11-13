use super::model::ModelAPI;
use super::types::AttackModel;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Membership inference attack
/// Shokri et al. (2017): "Membership Inference Attacks Against Machine Learning Models"
#[derive(Debug, Clone)]
pub struct MembershipInferenceAttack {
    /// shadow dataset.
    pub shadow_dataset: Vec<(Vec<f32>, bool)>, // (features, is_member)
    /// attack model.
    pub attack_model: AttackModel,
    /// feature dim.
    pub feature_dim: usize,
}

impl Default for MembershipInferenceAttack {
    fn default() -> Self {
        Self::new(3)
    }
}

impl MembershipInferenceAttack {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new membership inference attack
    #[must_use]
    pub fn new(feature_dim: usize) -> Self {
        Self {
            shadow_dataset: Vec::new(),
            attack_model: AttackModel::new(feature_dim),
            feature_dim,
        }
    }

    /// Train shadow models to mimic target
    pub fn train_shadow_models(&mut self, n_shadows: usize, local_data: &[Vec<f32>]) {
        if local_data.is_empty() || n_shadows == 0 {
            return;
        }

        let shadow_size = local_data.len() / n_shadows.max(1);

        for i in 0..n_shadows {
            // Train shadow model on subset
            let start = i * shadow_size;
            let end = ((i + 1) * shadow_size).min(local_data.len());

            if start >= local_data.len() {
                break;
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            }

            let shadow_data: Vec<_> = local_data[start..end].to_vec();

            // Record predictions for attack model training
            for sample in &shadow_data {
                let prediction = self.shadow_predict(sample, true);
                let features = self.extract_features(&prediction, true);
                self.shadow_dataset.push((features, true)); // In training
            }

            // Non-member samples (use different indices)
            let non_member_count = shadow_size.min(100);
            for j in 0..non_member_count {
                let idx = (local_data.len().saturating_sub(1) - j) % local_data.len();
                let sample = &local_data[idx];
                let prediction = self.shadow_predict(sample, false);
                let features = self.extract_features(&prediction, false);
                self.shadow_dataset.push((features, false)); // Not in training
            }
        }

        // Train attack model
        self.train_attack_model();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Infer membership of target sample
    #[must_use]
    pub fn infer_membership(&self, _sample: &[f32], model_output: &[f32]) -> f32 {
        let features = self.extract_features_from_prediction(model_output);
        self.attack_model.predict(&features)
    }

    /// Batch inference for multiple samples
    #[must_use]
    pub fn infer_membership_batch(
        &self,
        samples: &[Vec<f32>],
        model_outputs: &[Vec<f32>],
    ) -> Vec<f32> {
        samples
            .iter()
            .zip(model_outputs.iter())
            .map(|(sample, output)| self.infer_membership(sample, output))
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute entropy from probability distribution
    #[must_use]
    pub fn compute_entropy(&self, probs: &[f32]) -> f32 {
        let sum: f32 = probs.iter().map(|&p| p.max(0.0)).sum();
        if sum <= 1e-12 {
            return 0.0;
        }
        let mut ent = 0.0f32;
        for &p in probs {
            let p_norm = (p.max(0.0) / sum).clamp(1e-12, 1.0);
            ent -= p_norm * p_norm.ln();
        }
        ent.max(0.0)
    }

    /// Predicted class from model output
    #[allow(dead_code)]
    fn get_predicted_class(&self, output: &[f32]) -> usize {
        output
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(0, |(idx, _)| idx)
    }

    /// Shadow model prediction (sample-dependent with member vs non-member temperature scaling)
    fn shadow_predict(&self, sample: &[f32], is_member: bool) -> Vec<f32> {
        let n_classes = 10;
        let seed_hash: u64 = sample
            .iter()
            .enumerate()
            .map(|(i, &x)| (x.to_bits() as u64).wrapping_mul(i as u64 + 1))
            .sum();
        let target_class = (seed_hash as usize) % n_classes;
        let mut output = vec![0.05f32; n_classes];
        // Training members exhibit sharper peak confidence (overfitting) per Shokri et al. (2017)
        let peak = if is_member { 0.85 } else { 0.40 };
        output[target_class] = peak;
        let sum: f32 = output.iter().sum();
        for x in &mut output {
            *x /= sum;
        }
        output
    }

    /// Extract canonical feature representation from prediction vector
    fn extract_features_from_prediction(&self, prediction: &[f32]) -> Vec<f32> {
        let max_conf = prediction.iter().copied().fold(0.0f32, f32::max);
        let entropy = self.compute_entropy(prediction);
        let mut sorted = prediction.to_vec();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let margin = if sorted.len() >= 2 {
            (sorted[0] - sorted[1]).max(0.0)
        } else {
            max_conf
        };
        vec![max_conf, entropy, margin]
    }

    /// Extract features for attack model
    fn extract_features(&self, prediction: &[f32], _is_member: bool) -> Vec<f32> {
        self.extract_features_from_prediction(prediction)
    }

    /// Train attack model on shadow dataset
    fn train_attack_model(&mut self) {
        if self.shadow_dataset.is_empty() {
            return;
        }

        // Simple training
        self.attack_model.train(&self.shadow_dataset, 50, 0.01);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get attack accuracy on shadow dataset
    #[must_use]
    pub fn evaluate_attack_model(&self) -> f32 {
        if self.shadow_dataset.is_empty() {
            return 0.0;
        }

        let correct: usize = self
            .shadow_dataset
            .iter()
            .filter(|(features, label)| {
                let prediction = self.attack_model.predict(features);
                let predicted_label = prediction > 0.5;
                predicted_label == *label
            })
            .count();

        correct as f32 / self.shadow_dataset.len() as f32
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Model inversion attack
/// Fredrikson et al. (2015): "Model Inversion Attacks that Exploit Confidence Information"
#[derive(Debug, Clone)]
pub struct ModelInversionAttack {
    /// target class.
    pub target_class: usize,
    /// reconstructed data.
    pub reconstructed_data: Vec<f32>,
    /// learning rate.
    pub learning_rate: f32,
    /// iterations.
    pub iterations: usize,
}

impl Default for ModelInversionAttack {
    fn default() -> Self {
        Self {
            target_class: 0,
            reconstructed_data: Vec::new(),
            learning_rate: 0.1,
            iterations: 1000,
        }
    }
}

impl ModelInversionAttack {
    /// Create new model inversion attack
    #[must_use]
    pub const fn new(target_class: usize) -> Self {
        Self {
            target_class,
            reconstructed_data: Vec::new(),
            learning_rate: 0.1,
            iterations: 1000,
        }
    }

    /// Set optimization parameters
    #[must_use]
    pub const fn with_params(mut self, learning_rate: f32, iterations: usize) -> Self {
        self.learning_rate = learning_rate;
        self.iterations = iterations;
        self
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Reconstruct training data from model confidence
    pub fn invert(&mut self, model: &impl ModelAPI, input_dim: usize) -> Vec<f32> {
        // Start with deterministic initial image
        let mut reconstructed: Vec<f32> = (0..input_dim)
            .map(|i| {
                let h = (i as u64).wrapping_mul(0x9e3779b97f4a7c15);
                ((h >> 32) as u32 as f32) / (u32::MAX as f32)
            })
            .collect();
        let mut lr = self.learning_rate;

        for iteration in 0..self.iterations {
            // Forward pass
            let prediction = model.predict(&reconstructed);

            if self.target_class >= prediction.len() {
                break;
            }

            // Loss: maximize confidence for target class
            let target_confidence = prediction[self.target_class];
            let _loss = 1.0 - target_confidence;

            // Early stopping if confident enough
            if target_confidence > 0.95 {
                break;
            }

            // Gradient descent on input
            let gradient = self.compute_input_gradient(model, &reconstructed, self.target_class);

            for i in 0..reconstructed.len() {
                reconstructed[i] -= lr * gradient[i];
                // Clip to valid range [0, 1]
                reconstructed[i] = reconstructed[i].clamp(0.0, 1.0);
            }

            // Decay learning rate
            if iteration > 0 && iteration % 100 == 0 {
                lr *= 0.95;
            }
        }

        self.reconstructed_data.clone_from(&reconstructed);
        reconstructed
    }

    /// Compute input gradient via finite differences
    fn compute_input_gradient(
        &self,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        model: &impl ModelAPI,
        input: &[f32],
        target: usize,
    ) -> Vec<f32> {
        let epsilon = 0.01;
        let mut gradient = vec![0.0; input.len()];

        // Compute base prediction
        let base_pred = model.predict(input);
        let base_conf = if target < base_pred.len() {
            base_pred[target]
        } else {
            0.0
        };

        // Finite differences
        for i in 0..input.len() {
            let mut perturbed = input.to_vec();
            perturbed[i] = (perturbed[i] + epsilon).min(1.0);

            let pred = model.predict(&perturbed);
            let conf = if target < pred.len() {
                pred[target]
            } else {
                0.0
            };

            let actual_eps = (perturbed[i] - input[i]).abs();
            gradient[i] = if actual_eps > 1e-6 {
                -(conf - base_conf) / actual_eps // Negative gradient for maximization
            } else {
                0.0
            };
        }

        gradient
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get reconstructed data
    #[must_use]
    pub fn get_reconstructed(&self) -> &[f32] {
        &self.reconstructed_data
    }

    /// Assess reconstruction quality (higher is better)
    pub fn reconstruction_confidence(&self, model: &impl ModelAPI) -> f32 {
        if self.reconstructed_data.is_empty() {
            return 0.0;
        }

        let prediction = model.predict(&self.reconstructed_data);
        if self.target_class < prediction.len() {
            prediction[self.target_class]
        } else {
            0.0
        }
    }
}

/// Deep Leakage from Gradients (DLG) attack
/// Zhu et al. (2019): "Deep Leakage from Gradients"
#[derive(Debug, Clone)]
pub struct GradientLeakageAttack {
    /// dummy data.
    pub dummy_data: Vec<f32>,
    /// dummy label.
    pub dummy_label: usize,
    /// learning rate.
    pub learning_rate: f32,
    /// iterations.
    pub iterations: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for GradientLeakageAttack {
    fn default() -> Self {
        Self {
            dummy_data: Vec::new(),
            dummy_label: 0,
            learning_rate: 0.1,
            iterations: 300,
        }
    }
}

impl GradientLeakageAttack {
    /// Create new gradient leakage attack
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set optimization parameters
    #[must_use]
    pub const fn with_params(mut self, learning_rate: f32, iterations: usize) -> Self {
        self.learning_rate = learning_rate;
        self.iterations = iterations;
        self
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Reconstruct training data from shared gradients
    #[allow(clippy::assigning_clones)]
    pub fn reconstruct_from_gradients(
        &mut self,
        leaked_gradients: &[f32],
        model: &impl ModelAPI,
        input_dim: usize,
    ) -> (Vec<f32>, usize) {
        // Initialize dummy data deterministically (SIM-004)
        self.dummy_data = (0..input_dim)
            .map(|i| {
                let h = (i as u64)
                    .wrapping_mul(0x9E3779B97F4A7C15)
                    .wrapping_add(input_dim as u64);
                ((h >> 32) as u32 as f32) / (u32::MAX as f32)
            })
            .collect();
        self.dummy_label = 0;

        let mut best_loss = f32::INFINITY;
        let mut best_data = self.dummy_data.clone();
        let mut best_label = 0;

        for _ in 0..self.iterations {
            // Compute dummy gradients
            let dummy_grads = model.compute_gradients(&self.dummy_data, self.dummy_label);

            // Match gradients to leaked
            let loss = self.gradient_distance(leaked_gradients, &dummy_grads);

            if loss < best_loss {
                best_loss = loss;
                best_data.clone_from(&self.dummy_data);
                best_label = self.dummy_label;
            }

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Update dummy data to minimize gradient distance
            let data_grad = self.compute_data_gradient(model, leaked_gradients);

            for (i, dummy_data_i) in self.dummy_data.iter_mut().enumerate() {
                *dummy_data_i -= self.learning_rate * data_grad[i];
                *dummy_data_i = dummy_data_i.clamp(0.0, 1.0);
            }

            // Update label
            self.update_label(leaked_gradients, model);
        }

        self.dummy_data.clone_from(&best_data);
        self.dummy_label = best_label;

        (best_data, best_label)
    }

    /// Compute distance between gradient vectors
    fn gradient_distance(&self, g1: &[f32], g2: &[f32]) -> f32 {
        let len = g1.len().min(g2.len());
        (0..len)
            .map(|i| (g1[i] - g2[i]).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update label using L-BFGS style search
    fn update_label(&mut self, leaked_grads: &[f32], model: &impl ModelAPI) {
        let n_classes = model.output_dimension();
        let mut best_label = 0;
        let mut min_distance = f32::INFINITY;

        for label in 0..n_classes {
            let grads = model.compute_gradients(&self.dummy_data, label);
            let dist = self.gradient_distance(leaked_grads, &grads);

            if dist < min_distance {
                min_distance = dist;
                best_label = label;
            }
        }

        self.dummy_label = best_label;
    }

    /// Compute data gradient via finite differences
    fn compute_data_gradient(&self, model: &impl ModelAPI, target_grads: &[f32]) -> Vec<f32> {
        let epsilon = 0.001;
        let mut gradient = vec![0.0; self.dummy_data.len()];

        let base_grads = model.compute_gradients(&self.dummy_data, self.dummy_label);
        let base_loss = self.gradient_distance(target_grads, &base_grads);

        for i in 0..self.dummy_data.len() {
            let mut perturbed = self.dummy_data.clone();
            let (actual_delta, step_dir) = if perturbed[i] + epsilon <= 1.0 {
                (epsilon, 1.0f32)
            } else {
                (epsilon, -1.0f32)
            };
            perturbed[i] = (perturbed[i] + step_dir * actual_delta).clamp(0.0, 1.0);

            let new_grads = model.compute_gradients(&perturbed, self.dummy_label);
            let new_loss = self.gradient_distance(target_grads, &new_grads);

            gradient[i] = (new_loss - base_loss) / (step_dir * actual_delta);
        }

        gradient
    }

    /// Get reconstructed data and label
    #[must_use]
    pub fn get_reconstruction(&self) -> (&[f32], usize) {
        (&self.dummy_data, self.dummy_label)
    }
}
