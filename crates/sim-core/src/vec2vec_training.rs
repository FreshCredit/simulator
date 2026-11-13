//! `Vec2Vec` Training Pipeline: Lightweight Online Learning for Cross-Domain Projections
//!
//! Per §25.9 Cross-Domain Alignment: Learn projection matrices W: `V_domain` → `V_DOCAS`
//!
//! This module provides a lightweight training pipeline that:
//! 1. Collects training samples during simulation (online learning)
//! 2. Trains projections periodically (not every tick - performance conscious)
//! 3. Uses simple gradient descent (not overkill - no matrix inversion)
//!
//! SIM-001: Isolation verified - no product dependencies

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use serde::{Deserialize, Serialize};

use crate::cross_domain::{CrossDomainAlignment, DomainAlignment};

use crate::docas::DocasState;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Training buffer for online `Vec2Vec` learning
/// Collects (`domain_vector`, `docas_state`) pairs during simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingBuffer {
    /// Maximum buffer size (to prevent unbounded growth)
    pub max_size: usize,

    /// Emotion domain samples: (`emotion_vector`, docas)
    pub emotion_samples: Vec<(Vec<f32>, DocasState)>,

    /// Language domain samples: (`language_vector`, docas)
    pub language_samples: Vec<(Vec<f32>, DocasState)>,

    /// Music domain samples: (`music_vector`, docas)
    pub music_samples: Vec<(Vec<f32>, DocasState)>,

    /// Training counter (train every N samples)
    pub samples_since_training: usize,

    /// Training interval (train every N samples)
    pub training_interval: usize,
}

impl Default for TrainingBuffer {
    fn default() -> Self {
        Self::new(1000, 100)
    }
}

impl TrainingBuffer {
    /// Create new training buffer
    #[must_use]
    pub fn new(max_size: usize, training_interval: usize) -> Self {
        Self {
            max_size,
            emotion_samples: Vec::with_capacity(max_size),
            language_samples: Vec::with_capacity(max_size),
            music_samples: Vec::with_capacity(max_size),
            samples_since_training: 0,
            training_interval,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add emotion sample to buffer
    pub fn add_emotion_sample(&mut self, emotion_vec: Vec<f32>, docas: &DocasState) {
        if self.emotion_samples.len() >= self.max_size {
            // Random eviction to maintain distribution
            let mut rng = StdRng::seed_from_u64(42);
            let idx = rng.gen_range(0..self.emotion_samples.len());
            self.emotion_samples[idx] = (emotion_vec, *docas);
        } else {
            self.emotion_samples.push((emotion_vec, *docas));
        }
        self.samples_since_training += 1;
    }

    /// Add language sample to buffer
    pub fn add_language_sample(&mut self, language_vec: Vec<f32>, docas: &DocasState) {
        if self.language_samples.len() >= self.max_size {
            let mut rng = StdRng::seed_from_u64(43);
            let idx = rng.gen_range(0..self.language_samples.len());
            self.language_samples[idx] = (language_vec, *docas);
        } else {
            self.language_samples.push((language_vec, *docas));
        }
        self.samples_since_training += 1;
    }

    /// Add music sample to buffer
    pub fn add_music_sample(&mut self, music_vec: Vec<f32>, docas: &DocasState) {
        if self.music_samples.len() >= self.max_size {
            let mut rng = StdRng::seed_from_u64(44);
            let idx = rng.gen_range(0..self.music_samples.len());
            self.music_samples[idx] = (music_vec, *docas);
        } else {
            self.music_samples.push((music_vec, *docas));
        }
        self.samples_since_training += 1;
    }

    /// Check if training should run
    #[must_use]
    pub const fn should_train(&self) -> bool {
        self.samples_since_training >= self.training_interval
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Train all alignments that have sufficient data
    /// Returns training stats for monitoring
    pub fn train(&mut self, alignment: &mut CrossDomainAlignment) -> TrainingStats {
        let mut stats = TrainingStats::default();

        // Train emotion alignment (minimum 10 samples)
        if self.emotion_samples.len() >= 10 {
            let domain_obs: Vec<Vec<f32>> = self
                .emotion_samples
                .iter()
                .map(|(v, _)| v.clone())
                .collect();
            let docas_obs: Vec<DocasState> = self.emotion_samples.iter().map(|(_, d)| *d).collect();

            let pre_error =
                self.compute_mean_error(&alignment.emotion_alignment, &domain_obs, &docas_obs);
            alignment
                .emotion_alignment
                .train_ols(&domain_obs, &docas_obs);
            let post_error =
                self.compute_mean_error(&alignment.emotion_alignment, &domain_obs, &docas_obs);

            stats.emotion_trained = true;
            stats.emotion_samples = self.emotion_samples.len();
            stats.emotion_error_reduction = pre_error - post_error;
        }

        // Train language alignment
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        if self.language_samples.len() >= 10 {
            let domain_obs: Vec<Vec<f32>> = self
                .language_samples
                .iter()
                .map(|(v, _)| v.clone())
                .collect();
            let docas_obs: Vec<DocasState> =
                self.language_samples.iter().map(|(_, d)| *d).collect();

            let pre_error =
                self.compute_mean_error(&alignment.language_alignment, &domain_obs, &docas_obs);
            alignment
                .language_alignment
                .train_ols(&domain_obs, &docas_obs);
            let post_error =
                self.compute_mean_error(&alignment.language_alignment, &domain_obs, &docas_obs);

            stats.language_trained = true;
            stats.language_samples = self.language_samples.len();
            stats.language_error_reduction = pre_error - post_error;
        }

        // Train music alignment
        if self.music_samples.len() >= 10 {
            let domain_obs: Vec<Vec<f32>> =
                self.music_samples.iter().map(|(v, _)| v.clone()).collect();
            let docas_obs: Vec<DocasState> = self.music_samples.iter().map(|(_, d)| *d).collect();

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let pre_error =
                self.compute_mean_error(&alignment.music_alignment, &domain_obs, &docas_obs);
            alignment.music_alignment.train_ols(&domain_obs, &docas_obs);
            let post_error =
                self.compute_mean_error(&alignment.music_alignment, &domain_obs, &docas_obs);

            stats.music_trained = true;
            stats.music_samples = self.music_samples.len();
            stats.music_error_reduction = pre_error - post_error;
        }

        self.samples_since_training = 0;
        stats
    }

    /// Compute mean squared error for a domain alignment
    fn compute_mean_error(
        &self,
        alignment: &DomainAlignment,
        domain_obs: &[Vec<f32>],
        docas_obs: &[DocasState],
    ) -> f32 {
        if domain_obs.is_empty() {
            return 1.0;
        }

        let total_error: f32 = domain_obs
            .iter()
            .zip(docas_obs.iter())
            .map(|(domain_vec, target_docas)| {
                let predicted = alignment.project(domain_vec);
                target_docas
                    .levels
                    .iter()
                    .zip(predicted.levels.iter())
                    .map(|(t, p)| (t - p).powi(2))
                    .sum::<f32>()
            })
            .sum();

        total_error / domain_obs.len() as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get buffer statistics
    #[must_use]
    pub fn stats(&self) -> BufferStats {
        BufferStats {
            emotion_samples: self.emotion_samples.len(),
            language_samples: self.language_samples.len(),
            music_samples: self.music_samples.len(),
            total_samples: self.emotion_samples.len()
                + self.language_samples.len()
                + self.music_samples.len(),
            samples_until_training: self
                .training_interval
                .saturating_sub(self.samples_since_training),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Clear all samples (e.g., after major simulation parameter changes)
    pub fn clear(&mut self) {
        self.emotion_samples.clear();
        self.language_samples.clear();
        self.music_samples.clear();
        self.samples_since_training = 0;
    }
}

/// Training statistics returned after training run
#[derive(Debug, Clone, Copy, Default)]
pub struct TrainingStats {
    /// emotion trained.
    pub emotion_trained: bool,
    /// emotion samples.
    pub emotion_samples: usize,
    /// emotion error reduction.
    pub emotion_error_reduction: f32,
    /// language trained.
    pub language_trained: bool,
    /// language samples.
    pub language_samples: usize,
    /// language error reduction.
    pub language_error_reduction: f32,
    /// music trained.
    pub music_trained: bool,
    /// music samples.
    pub music_samples: usize,
    /// music error reduction.
    pub music_error_reduction: f32,
}

/// Buffer statistics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct BufferStats {
    /// emotion samples.
    pub emotion_samples: usize,
    /// language samples.
    pub language_samples: usize,
    /// music samples.
    pub music_samples: usize,
    /// total samples.
    pub total_samples: usize,
    /// samples until training.
    pub samples_until_training: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Simple online trainer that runs in the simulation loop
/// Performance-conscious: only samples and trains periodically
#[derive(Debug, Clone)]
pub struct OnlineTrainer {
    /// Training buffer
    pub buffer: TrainingBuffer,

    /// Sample interval (only collect every N ticks)
    pub sample_interval: usize,

    /// Tick counter
    tick_count: usize,
}

impl Default for OnlineTrainer {
    fn default() -> Self {
        Self::new(100, 1000)
    }
}

impl OnlineTrainer {
    /// Create new online trainer
    ///
    /// # Arguments
    /// * `sample_interval` - Collect sample every N ticks
    /// * `training_interval` - Train every N samples
    #[must_use]
    pub fn new(sample_interval: usize, training_interval: usize) -> Self {
        Self {
            buffer: TrainingBuffer::new(1000, training_interval),
            sample_interval,
            tick_count: 0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Call this every simulation tick
    /// Returns training stats if training occurred, None otherwise
    pub fn tick(&mut self, alignment: &mut CrossDomainAlignment) -> Option<TrainingStats> {
        self.tick_count += 1;

        // Only check for training periodically
        if self.tick_count % self.sample_interval == 0 && self.buffer.should_train() {
            Some(self.buffer.train(alignment))
        } else {
            None
        }
    }

    /// Sample from an agent (call this when agent has all domain states available)
    pub fn sample_agent(
        &mut self,
        docas: &DocasState,
        emotion: Option<&[f32]>,
        language: Option<&[f32]>,
        music: Option<&[f32]>,
    ) {
        // Only sample periodically
        if self.tick_count % self.sample_interval != 0 {
            return;
        }

        if let Some(vec) = emotion {
            self.buffer.add_emotion_sample(vec.to_vec(), docas);
        }
        if let Some(vec) = language {
            self.buffer.add_language_sample(vec.to_vec(), docas);
        }
        if let Some(vec) = music {
            self.buffer.add_music_sample(vec.to_vec(), docas);
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Federated `Vec2Vec` Trainer
///
/// Integrates federated learning with the cross-domain alignment system.
/// Enables distributed training of projection models across simulation agents.
#[derive(Debug, Clone)]
pub struct FederatedVec2VecTrainer {
    /// Online trainer for local learning
    pub local_trainer: OnlineTrainer,

    /// Whether federated learning is enabled
    pub federated_enabled: bool,

    /// Current domain being trained (emotion, language, music)
    pub current_domain: FederatedDomain,
}

/// Domain for federated training
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FederatedDomain {
    #[default]
    /// emotion.
    Emotion,
    /// language.
    Language,
    /// music.
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    Music,
}

impl FederatedVec2VecTrainer {
    /// Create new federated trainer
    #[must_use]
    pub fn new() -> Self {
        Self {
            local_trainer: OnlineTrainer::default(),
            federated_enabled: false,
            current_domain: FederatedDomain::Emotion,
        }
    }

    /// Create with federated learning enabled
    #[must_use]
    pub const fn with_federated(mut self, enabled: bool) -> Self {
        self.federated_enabled = enabled;
        self
    }

    /// Set training domain
    #[must_use]
    pub const fn with_domain(mut self, domain: FederatedDomain) -> Self {
        self.current_domain = domain;
        self
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Extract model parameters from alignment for a specific domain
    /// Returns flattened [bias (5), weights (`input_dim` * 5)]
    #[must_use]
    pub fn extract_model_params(&self, alignment: &CrossDomainAlignment) -> Vec<f32> {
        let domain_alignment = match self.current_domain {
            FederatedDomain::Emotion => &alignment.emotion_alignment,
            FederatedDomain::Language => &alignment.language_alignment,
            FederatedDomain::Music => &alignment.music_alignment,
        };

        let mut params = Vec::with_capacity(5 + domain_alignment.weights.len());

        // Add biases (first 5 elements)
        params.extend_from_slice(&domain_alignment.bias);

        // Add weights
        params.extend_from_slice(&domain_alignment.weights);

        params
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply model parameters to alignment for a specific domain
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn apply_model_params(
        &self,
        alignment: &mut CrossDomainAlignment,
        params: &[f32],
    ) -> Result<(), &'static str> {
        let domain_alignment = match self.current_domain {
            FederatedDomain::Emotion => &mut alignment.emotion_alignment,
            FederatedDomain::Language => &mut alignment.language_alignment,
            FederatedDomain::Music => &mut alignment.music_alignment,
        };

        let expected_len = 5 + domain_alignment.weights.len();
        if params.len() != expected_len {
            return Err("Parameter dimension mismatch");
        }

        // Apply biases
        domain_alignment.bias.copy_from_slice(&params[0..5]);

        // Apply weights
        domain_alignment.weights.copy_from_slice(&params[5..]);
        domain_alignment.is_trained = true;

        Ok(())
    }

    /// Get model dimension for current domain
    #[must_use]
    pub fn model_dimension(&self, alignment: &CrossDomainAlignment) -> usize {
        let domain_alignment = match self.current_domain {
            FederatedDomain::Emotion => &alignment.emotion_alignment,
            FederatedDomain::Language => &alignment.language_alignment,
            FederatedDomain::Music => &alignment.music_alignment,
        };

        5 + domain_alignment.weights.len()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Prepare training samples from buffer for federated learning
    #[must_use]
    pub fn prepare_federated_samples(
        &self,
    ) -> Vec<crate::neural::federated_learning::TrainingSample> {
        let buffer = &self.local_trainer.buffer;
        let samples = match self.current_domain {
            FederatedDomain::Emotion => &buffer.emotion_samples,
            FederatedDomain::Language => &buffer.language_samples,
            FederatedDomain::Music => &buffer.music_samples,
        };

        samples
            .iter()
            .map(|(input, target)| {
                crate::neural::federated_learning::TrainingSample::new(input.clone(), *target)
            })
            .collect()
    }
}

impl Default for FederatedVec2VecTrainer {
    fn default() -> Self {
        Self::new()
    }
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_buffer_add_samples() {
        let mut buffer = TrainingBuffer::new(100, 10);
        let docas = DocasState::default();

        buffer.add_emotion_sample(vec![0.1, 0.2, 0.3], &docas);
        buffer.add_language_sample(vec![0.4, 0.5], &docas);
        buffer.add_music_sample(vec![0.6, 0.7, 0.8, 0.9], &docas);

        assert_eq!(buffer.emotion_samples.len(), 1);
        assert_eq!(buffer.language_samples.len(), 1);
        assert_eq!(buffer.music_samples.len(), 1);
        assert_eq!(buffer.samples_since_training, 3);
    }

    #[test]
    fn test_training_buffer_max_size() {
        let mut buffer = TrainingBuffer::new(5, 100);
        let docas = DocasState::default();

        // Add more samples than max_size
        for i in 0..10 {
            buffer.add_emotion_sample(vec![i as f32; 3], &docas);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(buffer.emotion_samples.len(), 5);
    }

    #[test]
    fn test_training_runs_when_sufficient_samples() {
        let mut buffer = TrainingBuffer::new(100, 5);
        let mut alignment = CrossDomainAlignment::new();
        let docas = DocasState::default();

        // Add enough samples
        for _ in 0..10 {
            buffer.add_emotion_sample(vec![0.1, 0.2, 0.3], &docas);
        }

        // Should indicate training needed
        assert!(buffer.should_train());

        // Training should run
        let stats = buffer.train(&mut alignment);
        assert!(stats.emotion_trained);
        assert_eq!(stats.emotion_samples, 10);
    }

    #[test]
    fn test_training_skips_when_insufficient_samples() {
        let mut buffer = TrainingBuffer::new(100, 5);
        let mut alignment = CrossDomainAlignment::new();
        let docas = DocasState::default();

        // Add only a few samples
        buffer.add_emotion_sample(vec![0.1, 0.2, 0.3], &docas);
        buffer.add_emotion_sample(vec![0.2, 0.3, 0.4], &docas);

        // Training should skip (need >= 10 samples)
        let stats = buffer.train(&mut alignment);
        assert!(!stats.emotion_trained);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_buffer_stats() {
        let mut buffer = TrainingBuffer::new(100, 50);
        let docas = DocasState::default();

        buffer.add_emotion_sample(vec![0.1], &docas);
        buffer.add_emotion_sample(vec![0.2], &docas);
        buffer.add_language_sample(vec![0.3], &docas);

        let stats = buffer.stats();
        assert_eq!(stats.emotion_samples, 2);
        assert_eq!(stats.language_samples, 1);
        assert_eq!(stats.music_samples, 0);
        assert_eq!(stats.total_samples, 3);
    }

    #[test]
    fn test_online_trainer_sample_interval() {
        let mut trainer = OnlineTrainer::new(5, 100); // Sample every 5 ticks
        let _alignment = CrossDomainAlignment::new();
        let docas = DocasState::default();

        // Run 20 ticks with sampling
        for tick in 0..20 {
            trainer.tick_count = tick;
            trainer.sample_agent(&docas, Some(&[0.1, 0.2]), None, None);
        }

        // Should have sampled at ticks 0, 5, 10, 15 = 4 samples
        assert_eq!(trainer.buffer.emotion_samples.len(), 4);
    }

    // === FederatedVec2VecTrainer Tests ===

    #[test]
    fn test_federated_trainer_creation() {
        let trainer = FederatedVec2VecTrainer::new();
        assert!(!trainer.federated_enabled);
        assert_eq!(trainer.current_domain, FederatedDomain::Emotion);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_federated_trainer_with_federated() {
        let trainer = FederatedVec2VecTrainer::new().with_federated(true);
        assert!(trainer.federated_enabled);
    }

    #[test]
    fn test_federated_trainer_with_domain() {
        let trainer = FederatedVec2VecTrainer::new().with_domain(FederatedDomain::Language);
        assert_eq!(trainer.current_domain, FederatedDomain::Language);
    }

    #[test]
    fn test_extract_model_params() {
        let trainer = FederatedVec2VecTrainer::new();
        let alignment = CrossDomainAlignment::new();

        // Emotion: 3 input dims * 5 output dims + 5 biases = 20 params
        let params = trainer.extract_model_params(&alignment);
        assert_eq!(params.len(), 20);
    }

    #[test]
    fn test_extract_model_params_language() {
        let trainer = FederatedVec2VecTrainer::new().with_domain(FederatedDomain::Language);
        let alignment = CrossDomainAlignment::new();

        // Language: 8 input dims * 5 output dims + 5 biases = 45 params
        let params = trainer.extract_model_params(&alignment);
        assert_eq!(params.len(), 45);
    }

    #[test]
    fn test_extract_model_params_music() {
        let trainer = FederatedVec2VecTrainer::new().with_domain(FederatedDomain::Music);
        let alignment = CrossDomainAlignment::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Music: 14 input dims (5 genres + spectral + 7 cadence features) * 5 output dims + 5 biases = 75 params
        let params = trainer.extract_model_params(&alignment);
        assert_eq!(params.len(), 75);
    }

    #[test]
    fn test_apply_model_params() {
        let trainer = FederatedVec2VecTrainer::new();
        let mut alignment = CrossDomainAlignment::new();

        let mut params = vec![0.1; 20];
        params[0] = 0.5; // First bias

        trainer.apply_model_params(&mut alignment, &params).unwrap();

        assert!(alignment.emotion_alignment.is_trained);
        assert_eq!(alignment.emotion_alignment.bias[0], 0.5);
    }

    #[test]
    fn test_apply_model_params_dimension_mismatch() {
        let trainer = FederatedVec2VecTrainer::new();
        let mut alignment = CrossDomainAlignment::new();

        let params = vec![0.1; 10]; // Wrong dimension
        let result = trainer.apply_model_params(&mut alignment, &params);

        assert!(result.is_err());
    }

    #[test]
    fn test_model_dimension() {
        let trainer = FederatedVec2VecTrainer::new();
        let alignment = CrossDomainAlignment::new();

        assert_eq!(trainer.model_dimension(&alignment), 20);
    }

    #[test]
    fn test_prepare_federated_samples() {
        let mut trainer = FederatedVec2VecTrainer::new();
        let docas = DocasState::default();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Add samples to buffer
        trainer
            .local_trainer
            .buffer
            .add_emotion_sample(vec![0.1, 0.2, 0.3], &docas);
        trainer
            .local_trainer
            .buffer
            .add_emotion_sample(vec![0.2, 0.3, 0.4], &docas);

        let samples = trainer.prepare_federated_samples();
        assert_eq!(samples.len(), 2);
    }

    #[test]
    fn test_federated_domain_variants() {
        // Test all domain variants
        let domains = [
            FederatedDomain::Emotion,
            FederatedDomain::Language,
            FederatedDomain::Music,
        ];

        for domain in &domains {
            let trainer = FederatedVec2VecTrainer::new().with_domain(*domain);
            let alignment = CrossDomainAlignment::new();

            let dim = trainer.model_dimension(&alignment);
            assert!(dim > 0);

            let params = trainer.extract_model_params(&alignment);
            assert_eq!(params.len(), dim);
        }
    }
}
