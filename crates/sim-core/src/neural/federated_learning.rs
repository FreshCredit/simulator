//! Federated Learning for Distributed `Vec2Vec` Model Training
//!
//! Implements federated learning (`McMahan` et al. 2017) for training cross-domain
//! projection models across decentralized simulation agents without sharing raw data.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! ## Key Concepts
//!
//! - **Federated Averaging (`FedAvg`)**: Aggregates model updates from participants
//! - **Local Training**: Each agent trains on private data, shares only gradients
//! - **Differential Privacy**: Adds noise to updates for privacy preservation
//! - **Secure Aggregation**: Cryptographic protection of individual updates
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
//! │  Agent 1    │     │  Agent 2    │     │  Agent N    │
//! │  (local)    │     │  (local)    │     │  (local)    │
//! └──────┬──────┘     └──────┬──────┘     └──────┬──────┘
//!        │                   │                   │
//!        └───────────────────┼───────────────────┘
//!                            ▼
//!                   ┌─────────────────┐
//!                   │   Aggregator    │
//!                   │  (FedAvg/SecAgg)│
//!                   └────────┬────────┘
//!                            │
//!                            ▼
//!                   ┌─────────────────┐
//!                   │  Global Model   │
//!                   └─────────────────┘
//! ```
//!
//! SIM-001: Isolation verified - no product dependencies

use crate::docas::DocasState;
use crate::types::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Federated learning system for vec2vec training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedLearning {
    /// Global model parameters (flattened projection weights)
    pub global_model: Vec<f32>,

    /// Participating agents for current round
    pub participants: Vec<AgentId>,

    /// Local updates from agents (weight deltas)
    pub local_updates: HashMap<AgentId, Vec<f32>>,

    /// Aggregation strategy
    pub aggregation: AggregationStrategy,

    /// Privacy budget for differentially private FL
    pub privacy_budget: f32,

    /// Initial privacy budget allocated
    pub initial_budget: f32,

    /// Participant data quality weights
    pub participant_qualities: HashMap<AgentId, f32>,

    /// Current round
    pub round: u32,

    /// Model dimension (for validation)
    model_dim: usize,

    /// Learning rate for global updates
    pub global_lr: f32,
}

/// Aggregation strategy for federated learning
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Federated Averaging (`FedAvg`) - simple average of updates
    #[default]
    FedAvg,

    /// Weighted by data quality
    WeightedByQuality,

    /// Secure aggregation (simplified)
    SecureAgg,

    /// `FedAvg` with gradient clipping for Byzantine robustness
    RobustFedAvg,
}

/// Training sample for vec2vec projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    /// Input vector (domain-specific, e.g., emotion 3D, language 8D)
    pub input: Vec<f32>,

    /// Target DOCAS state
    pub target: DocasState,

    /// Sample weight (for importance sampling)
    pub weight: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl TrainingSample {
    /// Create new training sample with uniform weight
    #[must_use]
    pub const fn new(input: Vec<f32>, target: DocasState) -> Self {
        Self {
            input,
            target,
            weight: 1.0,
        }
    }

    /// Create weighted training sample
    #[must_use]
    pub fn weighted(input: Vec<f32>, target: DocasState, weight: f32) -> Self {
        Self {
            input,
            target,
            weight: weight.clamp(0.0, 1.0),
        }
    }
}

/// Local training participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalParticipant {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// agent id.
    pub agent_id: AgentId,
    /// local data.
    pub local_data: Vec<TrainingSample>,
    /// local model.
    pub local_model: Vec<f32>,
    /// data quality.
    pub data_quality: f32, // 0-1
    /// training epochs.
    pub training_epochs: u32,
    /// local lr.
    pub local_lr: f32,
}

impl LocalParticipant {
    /// Create new local participant
    #[must_use]
    pub fn new(agent_id: AgentId, data_quality: f32) -> Self {
        Self {
            agent_id,
            local_data: Vec::new(),
            local_model: Vec::new(),
            data_quality: data_quality.clamp(0.0, 1.0),
            training_epochs: 5,
            local_lr: 0.01,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add training sample
    pub fn add_sample(&mut self, sample: TrainingSample) {
        self.local_data.push(sample);
    }

    /// Get number of samples
    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.local_data.len()
    }

    /// Check if has sufficient data for training
    #[must_use]
    pub fn has_sufficient_data(&self, min_samples: usize) -> bool {
        self.local_data.len() >= min_samples
    }
}

/// Statistics from a federated learning round
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FederatedRoundStats {
    /// round.
    pub round: u32,
    /// participants.
    pub participants: usize,
    /// updates received.
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub updates_received: usize,
    /// avg gradient norm.
    pub avg_gradient_norm: f32,
    /// convergence metric.
    pub convergence_metric: f32,
    /// privacy budget consumed.
    pub privacy_budget_consumed: f32,
}

/// Convergence tracker for federated learning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConvergenceTracker {
    /// loss history.
    pub loss_history: Vec<f32>,
    /// gradient norm history.
    pub gradient_norm_history: Vec<f32>,
    /// window size.
    pub window_size: usize,
}

impl ConvergenceTracker {
    /// Create new convergence tracker
    #[must_use]
    pub const fn new(window_size: usize) -> Self {
        Self {
            loss_history: Vec::new(),
            gradient_norm_history: Vec::new(),
            window_size,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record round metrics
    pub fn record_round(&mut self, loss: f32, gradient_norm: f32) {
        self.loss_history.push(loss);
        self.gradient_norm_history.push(gradient_norm);

        // Keep only recent history
        if self.loss_history.len() > self.window_size {
            self.loss_history.remove(0);
        }
        if self.gradient_norm_history.len() > self.window_size {
            self.gradient_norm_history.remove(0);
        }
    }

    /// Check if converged (loss not improving)
    #[must_use]
    pub fn is_converged(&self, tolerance: f32) -> bool {
        if self.loss_history.len() < self.window_size {
            return false;
        }

        let recent_losses = &self.loss_history[self.loss_history.len() - self.window_size..];
        let mean_loss: f32 = recent_losses.iter().sum::<f32>() / recent_losses.len() as f32;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let variance: f32 = recent_losses
            .iter()
            .map(|&l| (l - mean_loss).powi(2))
            .sum::<f32>()
            / recent_losses.len() as f32;

        let recent_norms =
            &self.gradient_norm_history[self.gradient_norm_history.len() - self.window_size..];
        let mean_norm: f32 = recent_norms.iter().sum::<f32>() / recent_norms.len() as f32;

        variance < tolerance && mean_norm < (tolerance * 100.0).max(0.05)
    }

    /// Get improvement rate (negative = improving)
    #[must_use]
    pub fn improvement_rate(&self) -> f32 {
        if self.loss_history.len() < 2 {
            return 0.0;
        }

        let first_half: f32 = self.loss_history[..self.loss_history.len() / 2]
            .iter()
            .sum::<f32>()
            / (self.loss_history.len() / 2).max(1) as f32;
        let second_half: f32 = self.loss_history[self.loss_history.len() / 2..]
            .iter()
            .sum::<f32>()
            / (self.loss_history.len() - self.loss_history.len() / 2).max(1) as f32;

        (second_half - first_half) / first_half.max(1e-6)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl FederatedLearning {
    /// Create new federated learning system
    #[must_use]
    pub fn new(model_dim: usize, aggregation: AggregationStrategy) -> Self {
        Self {
            global_model: vec![0.0; model_dim],
            participants: Vec::new(),
            local_updates: HashMap::new(),
            aggregation,
            privacy_budget: 10.0,
            initial_budget: 10.0,
            participant_qualities: HashMap::new(),
            round: 0,
            model_dim,
            global_lr: 1.0,
        }
    }

    /// Create with `FedAvg` strategy
    #[must_use]
    pub fn new_fedavg(model_dim: usize) -> Self {
        Self::new(model_dim, AggregationStrategy::FedAvg)
    }

    /// Create with custom privacy budget
    #[must_use]
    pub fn with_privacy(mut self, epsilon: f32) -> Self {
        let ep = epsilon.max(0.001);
        self.privacy_budget = ep;
        self.initial_budget = ep;
        self
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Record data quality weight for a participant
    pub fn record_participant_quality(&mut self, agent_id: AgentId, quality: f32) {
        self.participant_qualities
            .insert(agent_id, quality.clamp(0.001, 1.0));
    }

    /// Initialize federated learning round
    pub fn initialize_round(&mut self, participants: Vec<AgentId>) {
        self.participants = participants;
        self.local_updates.clear();
        self.round += 1;
    }

    /// Agent performs local training
    pub fn local_training(
        &self,
        participant: &mut LocalParticipant,
        global_model: &[f32],
    ) -> Vec<f32> {
        if participant.local_data.is_empty() {
            return vec![0.0; global_model.len()];
        }

        // Start from global model
        participant.local_model = global_model.to_vec();

        // Local SGD training
        for _ in 0..participant.training_epochs {
            for sample in &participant.local_data {
                self.local_sgd_step(&mut participant.local_model, sample, participant.local_lr);
            }
        }

        // Compute update (delta = local - global)
        participant
            .local_model
            .iter()
            .zip(global_model.iter())
            .map(|(l, g)| l - g)
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Collect local update with privacy
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn submit_update(
        &mut self,
        agent_id: AgentId,
        update: &[f32],
        dp_epsilon: f32,
    ) -> Result<(), FederatedLearningError> {
        if update.len() != self.model_dim {
            return Err(FederatedLearningError::DimensionMismatch {
                expected: self.model_dim,
                got: update.len(),
            });
        }

        // Sybil isolation: only registered round participants can submit
        if !self.participants.is_empty() && !self.participants.contains(&agent_id) {
            return Err(FederatedLearningError::InvalidModel);
        }

        // Poisoning / finiteness gate: reject non-finite inputs fail-closed before spending privacy budget
        if update.iter().any(|&v| !v.is_finite()) {
            return Err(FederatedLearningError::InvalidModel);
        }

        // Privacy budget enforcement (pre-spend gating): reject invalid or over-budget epsilon
        if !dp_epsilon.is_finite() || dp_epsilon <= 0.0 {
            return Err(FederatedLearningError::InvalidModel);
        }
        if self.privacy_budget < dp_epsilon {
            return Err(FederatedLearningError::PrivacyBudgetExhausted);
        }

        // Bounded L2 gradient clipping to bound sensitivity
        let norm = compute_l2_norm(update);
        let clip_threshold = 1.0;
        let scale = if norm > clip_threshold {
            clip_threshold / norm
        } else {
            1.0
        };

        // Linear sequential composition under differential privacy: Delta B = epsilon
        self.privacy_budget -= dp_epsilon;

        // Add deterministic differential privacy noise to clipped update (SIM-004)
        let seed_salt = agent_id.wrapping_mul(0x9E3779B97F4A7C15);
        let private_update: Vec<f32> = update
            .iter()
            .enumerate()
            .map(|(i, &u)| {
                u * scale + self.sample_gaussian_noise(dp_epsilon, seed_salt.wrapping_add(i as u64))
            })
            .collect();

        self.local_updates.insert(agent_id, private_update);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Ok(())
    }

    /// Aggregate updates
    pub fn aggregate(&mut self) -> FederatedRoundStats {
        let mut stats = FederatedRoundStats {
            round: self.round,
            participants: self.participants.len(),
            updates_received: self.local_updates.len(),
            ..Default::default()
        };

        if self.local_updates.is_empty() {
            return stats;
        }

        match self.aggregation {
            AggregationStrategy::FedAvg => {
                self.fedavg_aggregate();
            }
            AggregationStrategy::WeightedByQuality => {
                self.weighted_aggregate();
            }
            AggregationStrategy::SecureAgg => {
                self.secure_aggregate();
            }
            AggregationStrategy::RobustFedAvg => {
                self.robust_fedavg_aggregate();
            }
        }

        // Compute gradient norm for convergence tracking
        stats.avg_gradient_norm = self.compute_average_gradient_norm();
        stats.convergence_metric = 1.0 / (1.0 + stats.avg_gradient_norm);
        stats.privacy_budget_consumed = (self.initial_budget - self.privacy_budget).max(0.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        stats
    }

    /// Federated Averaging
    fn fedavg_aggregate(&mut self) {
        let n = self.local_updates.len() as f32;
        if n == 0.0 {
            return;
        }

        let model_size = self.global_model.len();
        let mut sorted_keys: Vec<AgentId> = self.local_updates.keys().copied().collect();
        sorted_keys.sort();

        // Average all updates in deterministic agent order (SIM-004)
        let mut aggregated = vec![0.0; model_size];
        for id in &sorted_keys {
            if let Some(update) = self.local_updates.get(id) {
                for (i, &u) in update.iter().enumerate() {
                    aggregated[i] += u / n;
                }
            }
        }

        // Apply to global model with global learning rate
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        for (i, g) in self.global_model.iter_mut().enumerate() {
            *g += self.global_lr * aggregated[i];
        }
    }

    /// Weighted aggregation by data quality
    fn weighted_aggregate(&mut self) {
        let mut sorted_keys: Vec<AgentId> = self.local_updates.keys().copied().collect();
        sorted_keys.sort();

        let total_quality: f32 = sorted_keys
            .iter()
            .map(|id| {
                self.participant_qualities
                    .get(id)
                    .copied()
                    .unwrap_or(1.0)
                    .max(0.001)
            })
            .sum();

        if total_quality <= 0.0 {
            return self.fedavg_aggregate();
        }

        let model_size = self.global_model.len();
        let mut aggregated = vec![0.0; model_size];

        // Weighted average using participant quality scores in deterministic order (SIM-004)
        for id in &sorted_keys {
            if let Some(update) = self.local_updates.get(id) {
                let quality = self
                    .participant_qualities
                    .get(id)
                    .copied()
                    .unwrap_or(1.0)
                    .max(0.001);
                let weight = quality / total_quality;
                for (i, &u) in update.iter().enumerate() {
                    aggregated[i] += u * weight;
                }
            }
        }

        for (i, g) in self.global_model.iter_mut().enumerate() {
            *g += self.global_lr * aggregated[i];
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Secure aggregation (simplified placeholder)
    fn secure_aggregate(&mut self) {
        // In full implementation, would use:
        // - Secure multi-party computation
        // - Shamir secret sharing
        // - Masking with pairwise keys
        // For now, uses FedAvg with additional noise
        self.fedavg_aggregate();

        // Add additional deterministic aggregation noise for security (SIM-004)
        let round = self.round as u64;
        for i in 0..self.global_model.len() {
            let noise = self.sample_gaussian_noise(1000.0, round.wrapping_add(i as u64));
            self.global_model[i] += noise;
        }
    }

    /// Robust `FedAvg` with L2 gradient clipping and coordinate-wise median/trimmed-mean aggregation (Yin et al. 2018)
    fn robust_fedavg_aggregate(&mut self) {
        let clip_threshold = 1.0;

        let mut sorted_keys: Vec<AgentId> = self.local_updates.keys().copied().collect();
        sorted_keys.sort();

        let mut clipped_updates: Vec<Vec<f32>> = Vec::new();
        for id in &sorted_keys {
            if let Some(update) = self.local_updates.get(id) {
                let norm = compute_l2_norm(update);
                let scale = if norm > clip_threshold {
                    clip_threshold / norm
                } else {
                    1.0
                };

                let clipped: Vec<f32> = update.iter().map(|&u| u * scale).collect();
                clipped_updates.push(clipped);
            }
        }

        let num_updates = clipped_updates.len();
        if num_updates == 0 {
            return;
        }

        let model_size = self.global_model.len();
        let mut aggregated = vec![0.0f32; model_size];

        for i in 0..model_size {
            let mut coords: Vec<f32> = clipped_updates
                .iter()
                .map(|u| u[i])
                .filter(|v| v.is_finite())
                .collect();
            if coords.is_empty() {
                continue;
            }
            coords.sort_by(f32::total_cmp);
            let n_finite = coords.len();

            let agg_val = if n_finite < 3 {
                coords.iter().sum::<f32>() / n_finite as f32
            } else if n_finite < 5 {
                // Coordinate-wise median for small cohorts
                if n_finite % 2 == 1 {
                    coords[n_finite / 2]
                } else {
                    (coords[n_finite / 2 - 1] + coords[n_finite / 2]) / 2.0
                }
            } else {
                // Trimmed mean: discard top and bottom 20% Byzantine outliers
                let trim = (n_finite as f32 * 0.2).round() as usize;
                let active = &coords[trim..n_finite - trim];
                if active.is_empty() {
                    coords[n_finite / 2]
                } else {
                    active.iter().sum::<f32>() / active.len() as f32
                }
            };
            aggregated[i] = agg_val;
        }

        for (i, g) in self.global_model.iter_mut().enumerate() {
            *g += self.global_lr * aggregated[i];
        }
    }

    /// Local SGD step
    fn local_sgd_step(&self, model: &mut [f32], sample: &TrainingSample, lr: f32) {
        // Simplified gradient computation for vec2vec projection
        // In practice, would use proper backprop through projection layer

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Compute predicted DOCAS
        let predicted = self.compute_projection(model, &sample.input);

        // Compute gradient w.r.t. each output dimension
        for j in 0..5 {
            // DOCAS_SIZE = 5
            let target = sample.target.levels[j];
            let pred = predicted[j];
            let error = target - pred;

            // Update bias
            if j < model.len() {
                model[j] += lr * error;
            }

            // Update weights (simplified - assumes linear projection)
            for (i, &input_val) in sample.input.iter().enumerate() {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                let idx = 5 + i * 5 + j; // Offset past biases
                if idx < model.len() {
                    model[idx] += lr * error * input_val * sample.weight;
                }
            }
        }
    }

    /// Compute projection for gradient computation
    fn compute_projection(&self, model: &[f32], input: &[f32]) -> [f32; 5] {
        let mut result = [0.0f32; 5];

        // Bias terms (first 5 elements)
        for (j, result_j) in result.iter_mut().enumerate() {
            if j < model.len() {
                *result_j = model[j];
            }
        }

        // Weighted sum
        for (i, &input_val) in input.iter().enumerate() {
            for (j, result_val) in result.iter_mut().enumerate() {
                let idx = 5 + i * 5 + j;
                if idx < model.len() {
                    *result_val += model[idx] * input_val;
                }
            }
        }

        // Clamp to [0, 1]
        for val in &mut result {
            *val = val.clamp(0.0, 1.0);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        result
    }

    /// Sample Gaussian noise for differential privacy deterministically from round and seed salt (SIM-004)
    fn sample_gaussian_noise(&self, epsilon: f32, seed_salt: u64) -> f32 {
        let scale = 0.001 / epsilon.max(0.001);
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        let seed = (self.round as u64)
            .wrapping_mul(0x517cc1b727220a95)
            .wrapping_add(seed_salt);
        let mut rng = StdRng::seed_from_u64(seed);
        let u1: f32 = rng.gen_range(1e-7..1.0);
        let u2: f32 = rng.gen_range(0.0..1.0);

        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
        z * scale
    }

    /// Compute average L2 norm of gradients
    fn compute_average_gradient_norm(&self) -> f32 {
        if self.local_updates.is_empty() {
            return 0.0;
        }

        let mut sorted_keys: Vec<AgentId> = self.local_updates.keys().copied().collect();
        sorted_keys.sort();

        let total_norm: f32 = sorted_keys
            .iter()
            .filter_map(|id| self.local_updates.get(id))
            .map(|update| compute_l2_norm(update))
            .sum();

        total_norm / self.local_updates.len() as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get global model
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn get_global_model(&self) -> &[f32] {
        &self.global_model
    }

    /// Set global model
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn set_global_model(&mut self, model: Vec<f32>) -> Result<(), FederatedLearningError> {
        if model.len() != self.model_dim {
            return Err(FederatedLearningError::DimensionMismatch {
                expected: self.model_dim,
                got: model.len(),
            });
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
        self.global_model = model;
        Ok(())
    }

    /// Get model dimension
    #[must_use]
    pub const fn model_dimension(&self) -> usize {
        self.model_dim
    }

    /// Reset all updates
    pub fn clear_updates(&mut self) {
        self.local_updates.clear();
    }

    /// Check if round is complete (all participants submitted)
    #[must_use]
    pub fn is_round_complete(&self) -> bool {
        self.local_updates.len() == self.participants.len()
    }

    /// Get missing participants
    #[must_use]
    pub fn missing_participants(&self) -> Vec<AgentId> {
        self.participants
            .iter()
            .filter(|id| !self.local_updates.contains_key(id))
            .copied()
            .collect()
    }
}

/// Compute L2 norm of a vector
fn compute_l2_norm(vec: &[f32]) -> f32 {
    vec.iter().map(|&x| x * x).sum::<f32>().sqrt()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Federated learning errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FederatedLearningError {
    /// Dimension mismatch.
    DimensionMismatch {
        /// Expected dimension.
        expected: usize,
        /// Actual received dimension.
        got: usize,
    },
    /// Empty update.
    EmptyUpdate,
    /// Privacy budget exhausted.
    PrivacyBudgetExhausted,
    /// Invalid model.
    InvalidModel,
}

impl std::fmt::Display for FederatedLearningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionMismatch { expected, got } => {
                write!(f, "Dimension mismatch: expected {expected}, got {got}")
            }
            Self::EmptyUpdate => write!(f, "Empty update submitted"),
            Self::PrivacyBudgetExhausted => write!(f, "Privacy budget exhausted"),
            Self::InvalidModel => write!(f, "Invalid model parameters"),
        }
    }
}

impl std::error::Error for FederatedLearningError {}

/// Federated learning coordinator for multi-round training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedCoordinator {
    /// fl system.
    pub fl_system: FederatedLearning,
    /// convergence.
    pub convergence: ConvergenceTracker,
    /// max rounds.
    pub max_rounds: u32,
    /// min participants.
    pub min_participants: usize,
    /// convergence tolerance.
    pub convergence_tolerance: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl FederatedCoordinator {
    /// Create new coordinator
    #[must_use]
    pub fn new(model_dim: usize, max_rounds: u32) -> Self {
        Self {
            fl_system: FederatedLearning::new_fedavg(model_dim),
            convergence: ConvergenceTracker::new(5),
            max_rounds,
            min_participants: 3,
            convergence_tolerance: 1e-4,
        }
    }

    /// Check if training should continue
    #[must_use]
    pub fn should_continue(&self) -> bool {
        self.fl_system.round < self.max_rounds
            && !self.convergence.is_converged(self.convergence_tolerance)
    }

    /// Start new round
    pub fn start_round(&mut self, participants: Vec<AgentId>) {
        self.fl_system.initialize_round(participants);
    }

    /// Complete round and check convergence
    pub fn complete_round(&mut self) -> FederatedRoundStats {
        let stats = self.fl_system.aggregate();
        self.convergence
            .record_round(1.0 - stats.convergence_metric, stats.avg_gradient_norm);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        stats
    }

    /// Get training progress
    #[must_use]
    pub fn progress(&self) -> f32 {
        self.fl_system.round as f32 / self.max_rounds as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === FederatedLearning Tests ===

    #[test]
    fn test_federated_learning_creation() {
        let fl = FederatedLearning::new_fedavg(100);
        assert_eq!(fl.model_dimension(), 100);
        assert_eq!(fl.global_model.len(), 100);
        assert_eq!(fl.round, 0);
        assert_eq!(fl.aggregation, AggregationStrategy::FedAvg);
    }

    #[test]
    fn test_federated_learning_with_privacy() {
        let fl = FederatedLearning::new_fedavg(50).with_privacy(0.5);
        assert_eq!(fl.privacy_budget, 0.5);
    }

    #[test]
    fn test_initialize_round() {
        let mut fl = FederatedLearning::new_fedavg(10);
        let participants = vec![1u64, 2u64, 3u64];

        fl.initialize_round(participants.clone());

        assert_eq!(fl.round, 1);
        assert_eq!(fl.participants, participants);
        assert!(fl.local_updates.is_empty());
    }

    #[test]
    fn test_local_participant_creation() {
        let participant = LocalParticipant::new(1u64, 0.8);
        assert_eq!(participant.agent_id, 1);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(participant.data_quality, 0.8);
        assert!(participant.local_data.is_empty());
    }

    #[test]
    fn test_local_participant_add_sample() {
        let mut participant = LocalParticipant::new(1u64, 0.8);
        let docas = DocasState::default();
        let sample = TrainingSample::new(vec![0.1, 0.2, 0.3], docas);

        participant.add_sample(sample);

        assert_eq!(participant.sample_count(), 1);
        assert!(participant.has_sufficient_data(1));
        assert!(!participant.has_sufficient_data(2));
    }

    #[test]
    fn test_training_sample_creation() {
        let docas = DocasState::default();
        let sample = TrainingSample::new(vec![0.1, 0.2], docas);

        assert_eq!(sample.input, vec![0.1, 0.2]);
        assert_eq!(sample.weight, 1.0);
    }

    #[test]
    fn test_weighted_training_sample() {
        let docas = DocasState::default();
        let sample = TrainingSample::weighted(vec![0.1], docas, 0.5);

        assert_eq!(sample.weight, 0.5);
    }

    #[test]
    fn test_submit_update() {
        let mut fl = FederatedLearning::new_fedavg(5);
        let update = vec![0.1, 0.2, 0.3, 0.4, 0.5];

        fl.submit_update(1u64, &update, 1.0).unwrap();

        assert_eq!(fl.local_updates.len(), 1);
        assert!(fl.local_updates.contains_key(&1u64));
    }

    #[test]
    fn test_submit_update_dimension_mismatch() {
        let mut fl = FederatedLearning::new_fedavg(5);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let update = vec![0.1, 0.2, 0.3]; // Wrong dimension

        let result = fl.submit_update(1u64, &update, 1.0);

        assert!(matches!(
            result,
            Err(FederatedLearningError::DimensionMismatch {
                expected: 5,
                got: 3
            })
        ));
    }

    #[test]
    fn test_fedavg_aggregate() {
        let mut fl = FederatedLearning::new_fedavg(3);
        fl.initialize_round(vec![1u64, 2u64]);

        // Submit two updates
        fl.submit_update(1u64, &[0.2, 0.4, 0.6], 1.0).unwrap();
        fl.submit_update(2u64, &[0.0, 0.0, 0.0], 1.0).unwrap();

        // Aggregate
        let stats = fl.aggregate();

        assert_eq!(stats.updates_received, 2);
        // Average of [0.2, 0.4, 0.6] and [0, 0, 0] is [0.1, 0.2, 0.3]
        assert!((fl.global_model[0] - 0.1).abs() < 0.01);
        assert!((fl.global_model[1] - 0.2).abs() < 0.01);
        assert!((fl.global_model[2] - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_empty_aggregate() {
        let mut fl = FederatedLearning::new_fedavg(3);
        fl.initialize_round(vec![]);

        let stats = fl.aggregate();

        assert_eq!(stats.updates_received, 0);
        assert_eq!(fl.global_model, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_local_training_basic() {
        let fl = FederatedLearning::new_fedavg(10);
        let mut participant = LocalParticipant::new(1u64, 0.8);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Add training samples
        let docas = DocasState::new([0.5; 5]);
        participant.add_sample(TrainingSample::new(vec![0.1, 0.2, 0.3], docas));
        participant.add_sample(TrainingSample::new(vec![0.2, 0.3, 0.4], docas));

        let global_model = vec![0.0; 10];
        let update = fl.local_training(&mut participant, &global_model);

        assert_eq!(update.len(), 10);
        // Update should not be all zeros after training
        assert!(update.iter().any(|&x| x != 0.0));
    }

    #[test]
    fn test_local_training_empty_data() {
        let fl = FederatedLearning::new_fedavg(5);
        let mut participant = LocalParticipant::new(1u64, 0.8);

        let global_model = vec![0.0; 5];
        let update = fl.local_training(&mut participant, &global_model);

        assert_eq!(update, vec![0.0; 5]);
    }

    #[test]
    fn test_is_round_complete() {
        let mut fl = FederatedLearning::new_fedavg(3);
        fl.initialize_round(vec![1u64, 2u64]);

        assert!(!fl.is_round_complete());

        fl.submit_update(1u64, &[0.1, 0.2, 0.3], 1.0).unwrap();
        assert!(!fl.is_round_complete());

        fl.submit_update(2u64, &[0.1, 0.2, 0.3], 1.0).unwrap();
        assert!(fl.is_round_complete());
    }

    #[test]
    fn test_missing_participants() {
        let mut fl = FederatedLearning::new_fedavg(3);
        fl.initialize_round(vec![1u64, 2u64, 3u64]);

        fl.submit_update(1u64, &[0.1, 0.2, 0.3], 1.0).unwrap();

        let missing = fl.missing_participants();
        assert_eq!(missing.len(), 2);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(missing.contains(&2u64));
        assert!(missing.contains(&3u64));
    }

    #[test]
    fn test_set_global_model() {
        let mut fl = FederatedLearning::new_fedavg(3);
        let new_model = vec![0.5, 0.6, 0.7];

        fl.set_global_model(new_model.clone()).unwrap();

        assert_eq!(fl.global_model, new_model);
    }

    #[test]
    fn test_set_global_model_dimension_mismatch() {
        let mut fl = FederatedLearning::new_fedavg(3);
        let new_model = vec![0.5, 0.6]; // Wrong dimension

        let result = fl.set_global_model(new_model);

        assert!(matches!(
            result,
            Err(FederatedLearningError::DimensionMismatch {
                expected: 3,
                got: 2
            })
        ));
    }

    #[test]
    fn test_clear_updates() {
        let mut fl = FederatedLearning::new_fedavg(3);
        fl.initialize_round(vec![1u64]);
        fl.submit_update(1u64, &[0.1, 0.2, 0.3], 1.0).unwrap();

        fl.clear_updates();

        assert!(fl.local_updates.is_empty());
    }

    // === ConvergenceTracker Tests ===

    #[test]
    fn test_convergence_tracker_creation() {
        let tracker = ConvergenceTracker::new(10);
        assert!(tracker.loss_history.is_empty());
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(tracker.window_size, 10);
    }

    #[test]
    fn test_convergence_tracker_record() {
        let mut tracker = ConvergenceTracker::new(5);

        tracker.record_round(1.0, 0.5);
        tracker.record_round(0.9, 0.4);
        tracker.record_round(0.8, 0.3);

        assert_eq!(tracker.loss_history.len(), 3);
        assert_eq!(tracker.gradient_norm_history.len(), 3);
    }

    #[test]
    fn test_convergence_tracker_window_limit() {
        let mut tracker = ConvergenceTracker::new(3);

        for i in 0..10 {
            tracker.record_round(1.0 - i as f32 * 0.1, 0.5);
        }

        assert_eq!(tracker.loss_history.len(), 3);
    }

    #[test]
    fn test_convergence_is_converged() {
        let mut tracker = ConvergenceTracker::new(5);

        // Not converged with insufficient data
        assert!(!tracker.is_converged(0.01));

        // Add stable losses
        for _ in 0..5 {
            tracker.record_round(0.5, 0.1);
        }

        assert!(tracker.is_converged(0.01));
    }

    #[test]
    fn test_convergence_improvement_rate() {
        let mut tracker = ConvergenceTracker::new(10);

        // First half: higher loss
        for i in 0..5 {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            tracker.record_round(1.0 - i as f32 * 0.1, 0.1);
        }

        // Second half: lower loss
        for i in 0..5 {
            tracker.record_round(0.5 - i as f32 * 0.05, 0.05);
        }

        let rate = tracker.improvement_rate();
        assert!(rate < 0.0); // Negative = improving
    }

    // === FederatedCoordinator Tests ===

    #[test]
    fn test_coordinator_creation() {
        let coordinator = FederatedCoordinator::new(100, 50);
        assert_eq!(coordinator.fl_system.model_dimension(), 100);
        assert_eq!(coordinator.max_rounds, 50);
        assert_eq!(coordinator.min_participants, 3);
    }

    #[test]
    fn test_coordinator_should_continue() {
        let mut coordinator = FederatedCoordinator::new(10, 5);
        assert!(coordinator.should_continue());

        // Simulate rounds
        coordinator.fl_system.round = 5;
        assert!(!coordinator.should_continue());
    }

    #[test]
    fn test_coordinator_progress() {
        let mut coordinator = FederatedCoordinator::new(10, 100);
        assert_eq!(coordinator.progress(), 0.0);

        coordinator.fl_system.round = 50;
        assert_eq!(coordinator.progress(), 0.5);
    }

    #[test]
    fn test_compute_l2_norm() {
        assert_eq!(compute_l2_norm(&[3.0, 4.0]), 5.0);
        assert_eq!(compute_l2_norm(&[1.0, 1.0, 1.0, 1.0]), 2.0);
        assert_eq!(compute_l2_norm(&[]), 0.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_aggregation_strategies() {
        // Test that all strategies can be used
        let strategies = [
            AggregationStrategy::FedAvg,
            AggregationStrategy::WeightedByQuality,
            AggregationStrategy::SecureAgg,
            AggregationStrategy::RobustFedAvg,
        ];

        for strategy in &strategies {
            let mut fl = FederatedLearning::new(3, *strategy);
            fl.initialize_round(vec![1u64, 2u64]);
            fl.submit_update(1u64, &[0.1, 0.2, 0.3], 1.0).unwrap();
            fl.submit_update(2u64, &[0.2, 0.3, 0.4], 1.0).unwrap();

            let stats = fl.aggregate();
            assert_eq!(stats.updates_received, 2);
        }
    }

    #[test]
    fn test_federated_round_stats() {
        let stats = FederatedRoundStats {
            round: 5,
            participants: 10,
            updates_received: 8,
            avg_gradient_norm: 0.5,
            convergence_metric: 0.9,
            privacy_budget_consumed: 0.1,
        };

        assert_eq!(stats.round, 5);
        assert_eq!(stats.participants, 10);
        assert_eq!(stats.updates_received, 8);
    }

    #[test]
    fn test_error_display() {
        let err = FederatedLearningError::DimensionMismatch {
            expected: 10,
            got: 5,
        };
        assert!(err.to_string().contains("Dimension mismatch"));

        let err = FederatedLearningError::PrivacyBudgetExhausted;
        assert!(err.to_string().contains("Privacy budget"));
    }
}
