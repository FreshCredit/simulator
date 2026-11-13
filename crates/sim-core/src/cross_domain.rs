//! Cross-Domain Coherence (`Ψ_trust`) Module
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION multi-modal trust, not product data
//!
//! Per §6.5.2 (TFP Empirical Predictions):
//! - Composite Stability Index: Ψtrust = α·Ψbio + β·Ψfin + γ·Ψling
//! - Expected coefficients: α ≈ 0.35, β ≈ 0.45, γ ≈ 0.20
//! - Cross-domain coherence target: r = 0.45-0.55
//!
//! Per §25.9 Cross-Domain Alignment (Vec2Vec):
//! - Align geometric representations from Music, Emotion, Language, Physiology to DOCAS
//! - Learn projection matrices W: `V_domain` → `V_DOCAS`
//! - Weighted average for composite `Ψ_trust`

// TAG: surface=simulation owner=research-team rule=SIM-001
use serde::{Deserialize, Serialize};

use crate::docas::DocasState;
use crate::emotion::EmotionClassification;
use crate::language::Lexicon;
use crate::music::MusicState;
use crate::types::DOCAS_SIZE;

/// Domain indices for cross-domain alignment
pub const DOMAIN_DOCAS: usize = 0;
/// domain emotion.
pub const DOMAIN_EMOTION: usize = 1;
/// domain language.
pub const DOMAIN_LANGUAGE: usize = 2;
/// domain music.
pub const DOMAIN_MUSIC: usize = 3;
/// domain count.
pub const DOMAIN_COUNT: usize = 4;

/// Default weights for `Ψ_trust` composite
/// Per §6.5.2: α ≈ 0.35 (bio), β ≈ 0.45 (fin), γ ≈ 0.20 (ling)
/// Adapted for simulation: DOCAS, Emotion, Language, Music
pub const DEFAULT_DOMAIN_WEIGHTS: [f32; DOMAIN_COUNT] = [0.40, 0.25, 0.20, 0.15];

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cross-domain trust index (`Ψ_trust`)
/// Per §6.5.2: Multi-modal synchronization predicts stability
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CrossDomainTrust {
    /// Composite trust score [0.0, 2.0]
    pub psi_trust: f32,

    /// Per-domain trust scores
    pub domain_scores: [f32; DOMAIN_COUNT],

    /// Domain weights used for composite
    pub domain_weights: [f32; DOMAIN_COUNT],

    /// Cross-domain coherence (alignment quality)
    pub coherence: f32,

    /// Domain variances (lower = more agreement)
    pub domain_variance: f32,

    /// Timestamp
    pub tick: u64,
}

impl Default for CrossDomainTrust {
    fn default() -> Self {
        Self {
            psi_trust: 0.75,
            domain_scores: [0.75; DOMAIN_COUNT],
            domain_weights: DEFAULT_DOMAIN_WEIGHTS,
            coherence: 1.0,
            domain_variance: 0.0,
            tick: 0,
        }
    }
}

impl CrossDomainTrust {
    /// Create new cross-domain trust with default weights
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            ..Default::default()
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute decision confidence based on cross-domain coherence
    ///
    /// High coherence → confident decisions (domains agree)
    /// Low coherence → uncertain, should gather more info
    #[must_use]
    pub fn decision_confidence(&self) -> f32 {
        // Map coherence from [0, 1] to confidence range [0.3, 1.0]
        // Min 30% confidence even with low coherence
        0.3 + 0.7 * self.coherence
    }

    /// Determine if agent needs information gathering
    ///
    /// Low coherence indicates conflicting signals across domains
    #[must_use]
    pub fn needs_information(&self) -> bool {
        // Below threshold, need more data to resolve conflicts
        self.coherence < 0.5
    }

    /// Compute weighted trust using domain variances
    ///
    /// Domains with lower variance weighted higher (more consistent = more weight)
    #[must_use]
    pub fn weighted_trust_estimate(&self) -> f32 {
        let mean_score = self.domain_scores.iter().sum::<f32>() / DOMAIN_COUNT as f32;
        let weights: Vec<f32> = self
            .domain_scores
            .iter()
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .map(|&s| Self::consistency_weight(s, mean_score))
            .collect();
        let total_weight: f32 = weights.iter().sum();
        if total_weight > 0.0 {
            self.domain_scores
                .iter()
                .zip(weights.iter())
                .map(|(score, weight)| score * weight)
                .sum::<f32>()
                / total_weight
        } else {
            self.psi_trust
        }
    }

    fn consistency_weight(score: f32, mean: f32) -> f32 {
        let deviation = (score - mean).abs();
        if deviation < 0.01 {
            1.0
        } else {
            (1.0 / (1.0 + deviation)).clamp(0.1, 1.0)
        }
    }

    /// Get domain variance (lower = more agreement)
    #[must_use]
    pub const fn domain_variance(&self) -> f32 {
        self.domain_variance
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Detect domain conflicts (low coherence)
    ///
    /// Returns which domains disagree significantly from the mean
    #[must_use]
    pub fn detect_conflicts(&self) -> Vec<String> {
        let mut conflicts = vec![];
        let mean_score = self.domain_scores.iter().sum::<f32>() / DOMAIN_COUNT as f32;
        const CONFLICT_THRESHOLD: f32 = 0.5; // Significant deviation

        for (i, &score) in self.domain_scores.iter().enumerate() {
            let deviation = (score - mean_score).abs();
            if deviation > CONFLICT_THRESHOLD {
                conflicts.push(format!(
                    "{} deviates by {:.2}",
                    Self::domain_name(i),
                    deviation
                ));
            }
        }

        conflicts
    }

    /// Check if specific domain pair has conflict
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn domain_pair_conflict(&self, d1: usize, d2: usize, threshold: f32) -> bool {
        if d1 >= DOMAIN_COUNT || d2 >= DOMAIN_COUNT {
            return false;
        }
        let diff = (self.domain_scores[d1] - self.domain_scores[d2]).abs();
        diff > threshold
    }

    /// Create with custom weights
    #[must_use]
    pub fn with_weights(weights: [f32; DOMAIN_COUNT], tick: u64) -> Self {
        // Normalize weights to sum to 1.0
        let sum: f32 = weights.iter().sum();
        let normalized: [f32; DOMAIN_COUNT] = if sum > 0.0 {
            std::array::from_fn(|i| weights[i] / sum)
        } else {
            DEFAULT_DOMAIN_WEIGHTS
        };

        Self {
            domain_weights: normalized,
            tick,
            ..Default::default()
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute composite `Ψ_trust` from domain scores
    pub fn compute_composite(&mut self) -> f32 {
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;

        for i in 0..DOMAIN_COUNT {
            weighted_sum += self.domain_scores[i] * self.domain_weights[i];
            weight_sum += self.domain_weights[i];
        }

        self.psi_trust = if weight_sum > 0.0 {
            weighted_sum / weight_sum
        } else {
            0.75 // Default neutral
        };

        // Compute variance across domains
        self.compute_variance();

        // Compute coherence
        self.compute_coherence();

        self.psi_trust
    }

    /// Compute variance across domain scores
    fn compute_variance(&mut self) {
        let mean = self.domain_scores.iter().sum::<f32>() / DOMAIN_COUNT as f32;
        let variance = self
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .domain_scores
            .iter()
            .map(|s| (s - mean).powi(2))
            .sum::<f32>()
            / DOMAIN_COUNT as f32;
        self.domain_variance = variance;
    }

    /// Compute cross-domain coherence
    /// Higher = domains agree more (lower variance relative to mean)
    fn compute_coherence(&mut self) {
        let mean = self.domain_scores.iter().sum::<f32>() / DOMAIN_COUNT as f32;
        if mean > 0.0 {
            // Coefficient of variation inverted
            let cv = (self.domain_variance.sqrt() / mean).min(1.0);
            self.coherence = 1.0 - cv;
        } else {
            self.coherence = 0.0;
        }
    }

    /// Get domain name
    #[must_use]
    pub const fn domain_name(domain: usize) -> &'static str {
        match domain {
            DOMAIN_DOCAS => "DOCAS",
            DOMAIN_EMOTION => "Emotion",
            DOMAIN_LANGUAGE => "Language",
            DOMAIN_MUSIC => "Music",
            _ => "Unknown",
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if domains are synchronized (high coherence)
    #[must_use]
    pub fn is_synchronized(&self, threshold: f32) -> bool {
        self.coherence >= threshold
    }

    /// Get agreement level between two specific domains
    #[must_use]
    pub fn domain_agreement(&self, d1: usize, d2: usize) -> f32 {
        if d1 >= DOMAIN_COUNT || d2 >= DOMAIN_COUNT {
            return 0.0;
        }
        let diff = (self.domain_scores[d1] - self.domain_scores[d2]).abs();
        1.0 - (diff / 2.0).min(1.0) // Normalize to [0, 1]
    }
}

/// Domain alignment layer for `Vec2Vec` projection
/// Per §25.9: Learn projection matrices W: `V_domain` → `V_DOCAS`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAlignment {
    /// Domain identifier
    pub domain: usize,

    /// Projection weights: `domain_dim` × `DOCAS_SIZE`
    /// For MVP: Use linear projection (direct mapping without nonlinearity)
    pub weights: Vec<f32>,

    /// Projection bias: `DOCAS_SIZE`
    pub bias: [f32; DOCAS_SIZE],

    /// Input dimension
    pub input_dim: usize,

    /// Whether this alignment is trained
    pub is_trained: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DomainAlignment {
    /// Create new alignment for a domain with given input dimension
    #[must_use]
    pub fn new(domain: usize, input_dim: usize) -> Self {
        // Initialize with identity-like projection (scaled)
        let mut weights = vec![0.0f32; input_dim * DOCAS_SIZE];

        // Set diagonal-ish pattern for initial projection
        for i in 0..input_dim.min(DOCAS_SIZE) {
            weights[i * DOCAS_SIZE + i] = 1.0;
        }

        Self {
            domain,
            weights,
            bias: [0.0; DOCAS_SIZE],
            input_dim,
            is_trained: false,
        }
    }

    /// Project domain vector to DOCAS space
    #[must_use]
    pub fn project(&self, domain_vec: &[f32]) -> DocasState {
        if domain_vec.len() != self.input_dim {
            // Return neutral if dimensions don't match
            return DocasState::neutral();
        }

        let mut result = [0.0f32; DOCAS_SIZE];

        // Matrix multiplication: result = W × domain_vec + bias
        for (j, result_j) in result.iter_mut().enumerate().take(DOCAS_SIZE) {
            let mut sum = self.bias[j];
            for (i, &domain_val) in domain_vec.iter().enumerate().take(self.input_dim) {
                sum += self.weights[i * DOCAS_SIZE + j] * domain_val;
            }
            *result_j = sum.clamp(0.0, 1.0);
        }

        DocasState::new(result)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Project and compute tau
    #[must_use]
    pub fn project_tau(&self, domain_vec: &[f32]) -> f32 {
        let docas = self.project(domain_vec);
        docas.compute_tau()
    }

    /// Train alignment using OLS (MVP implementation)
    /// Per §25.9: W* = `argmin_W` ||`V_DOCAS` - W × `V_domain||²`
    pub fn train_ols(&mut self, domain_obs: &[Vec<f32>], docas_obs: &[DocasState]) {
        if domain_obs.len() != docas_obs.len() || domain_obs.len() < 2 {
            return;
        }

        // Simple gradient descent for MVP
        // Full OLS would require matrix inversion
        const LEARNING_RATE: f32 = 0.01;
        const EPOCHS: usize = 100;

        for _ in 0..EPOCHS {
            for (domain_vec, target_docas) in domain_obs.iter().zip(docas_obs.iter()) {
                // Forward pass
                let predicted = self.project(domain_vec);

                // Compute gradients and update
                for (j, (&target_level, &predicted_level)) in target_docas
                    .levels
                    .iter()
                    .zip(predicted.levels.iter())
                    .enumerate()
                    .take(DOCAS_SIZE)
                {
                    let error = target_level - predicted_level;
                    self.bias[j] += LEARNING_RATE * error;

                    for (i, &domain_val) in domain_vec.iter().enumerate().take(self.input_dim) {
                        let grad = error * domain_val;
                        self.weights[i * DOCAS_SIZE + j] += LEARNING_RATE * grad;
                    }
                }
            }
        }

        self.is_trained = true;
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cross-domain alignment system
/// Manages projections from all domains to DOCAS space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainAlignment {
    /// Emotion → DOCAS alignment
    pub emotion_alignment: DomainAlignment,

    /// Language → DOCAS alignment
    pub language_alignment: DomainAlignment,

    /// Music → DOCAS alignment
    pub music_alignment: DomainAlignment,

    /// Domain weights for composite
    pub weights: [f32; DOMAIN_COUNT],
}

impl Default for CrossDomainAlignment {
    fn default() -> Self {
        Self {
            // Emotion: 3D (valence, arousal, intensity) → 5D DOCAS
            emotion_alignment: DomainAlignment::new(DOMAIN_EMOTION, 3),

            // Language: 8D (embedding summary) → 5D DOCAS
            language_alignment: DomainAlignment::new(DOMAIN_LANGUAGE, 8),

            // Music: 14D (5 genres + spectral + 7 cadence features) → 5D DOCAS
            music_alignment: DomainAlignment::new(DOMAIN_MUSIC, 14),

            weights: DEFAULT_DOMAIN_WEIGHTS,
        }
    }
}

impl CrossDomainAlignment {
    /// Create new alignment system
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create with custom weights
    #[must_use]
    pub fn with_weights(weights: [f32; DOMAIN_COUNT]) -> Self {
        let mut alignment = Self::new();

        // Normalize weights
        let sum: f32 = weights.iter().sum();
        if sum > 0.0 {
            alignment.weights = std::array::from_fn(|i| weights[i] / sum);
        }

        alignment
    }

    /// Compute `Ψ_trust` composite from all available domains
    /// Per §25.9: `Ψ_trust` = `weighted_average(projected_DOCAS)`
    #[must_use]
    pub fn compute_psi_trust(
        &self,
        docas: &DocasState,
        emotion: Option<&EmotionClassification>,
        lexicon: Option<&Lexicon>,
        music: Option<&MusicState>,
    ) -> CrossDomainTrust {
        self.compute_psi_trust_with_language(docas, emotion, lexicon, music, None)
    }

    /// Compute `Ψ_trust` with explicit language similarity for music weight adjustment
    /// Enables cross-language bonding through music when language alignment is low
    #[must_use]
    pub fn compute_psi_trust_with_language(
        &self,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        docas: &DocasState,
        emotion: Option<&EmotionClassification>,
        lexicon: Option<&Lexicon>,
        music: Option<&MusicState>,
        language_similarity: Option<f32>,
    ) -> CrossDomainTrust {
        let mut psi = CrossDomainTrust::with_weights(self.weights, 0);

        // DOCAS domain (always available)
        psi.domain_scores[DOMAIN_DOCAS] = docas.compute_tau();

        // Emotion domain
        if let Some(emo) = emotion {
            let emotion_vec = self.emotion_to_vector(emo);
            psi.domain_scores[DOMAIN_EMOTION] = self.emotion_alignment.project_tau(&emotion_vec);
        } else {
            // Use DOCAS as fallback
            psi.domain_scores[DOMAIN_EMOTION] = psi.domain_scores[DOMAIN_DOCAS];
        }

        // Language domain
        let lang_score = if let Some(lex) = lexicon {
            let lang_vec = self.lexicon_to_vector(lex);
            let score = self.language_alignment.project_tau(&lang_vec);
            psi.domain_scores[DOMAIN_LANGUAGE] = score;
            score
        } else {
            psi.domain_scores[DOMAIN_LANGUAGE] = psi.domain_scores[DOMAIN_DOCAS];
            psi.domain_scores[DOMAIN_DOCAS]
        };

        // Music domain with cadence features
        if let Some(mus) = music {
            let music_vec = self.music_to_vector(mus);
            psi.domain_scores[DOMAIN_MUSIC] = self.music_alignment.project_tau(&music_vec);
        } else {
            psi.domain_scores[DOMAIN_MUSIC] = psi.domain_scores[DOMAIN_DOCAS];
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Adjust music weight based on language similarity for cross-language bonding
        let effective_lang_sim = language_similarity.unwrap_or(lang_score);
        let adjusted_music_weight = self.compute_music_weight(effective_lang_sim);

        // Rebalance weights: reduce others proportionally to accommodate music boost
        let weight_delta = adjusted_music_weight - self.weights[DOMAIN_MUSIC];
        if weight_delta > 0.0 {
            // Distribute reduction across other domains proportionally
            let other_weight_sum: f32 = self
                .weights
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != DOMAIN_MUSIC)
                .map(|(_, w)| w)
                .sum();
            let other_weight_sum = other_weight_sum.max(1e-6);

            for i in 0..DOMAIN_COUNT {
                if i == DOMAIN_MUSIC {
                    psi.domain_weights[i] = adjusted_music_weight;
                } else {
                    let reduction = (self.weights[i] / other_weight_sum) * weight_delta;
                    psi.domain_weights[i] = self.weights[i] - reduction;
                }
            }
        }

        // Compute composite
        psi.compute_composite();

        psi
    }

    /// Convert emotion classification to vector
    /// Returns [valence, arousal, intensity]
    fn emotion_to_vector(&self, emotion: &EmotionClassification) -> Vec<f32> {
        vec![
            emotion.primary.valence(),
            emotion.primary.arousal(),
            emotion.delta_tau.abs(), // Intensity
        ]
    }

    /// Convert lexicon to summary vector
    /// Returns embedding statistics
    fn lexicon_to_vector(&self, lexicon: &Lexicon) -> Vec<f32> {
        if lexicon.is_empty() {
            return vec![0.0; 8];
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Compute mean embedding
        let mut mean = [0.0f32; 8];
        let count = lexicon.concepts.len() as f32;

        for concept in lexicon.concepts.values() {
            for (mean_i, &value) in mean.iter_mut().zip(concept.embedding.values.iter()).take(8) {
                *mean_i += value;
            }
        }

        for mean_i in &mut mean {
            *mean_i /= count;
        }

        mean.to_vec()
    }

    /// Convert music state to vector
    /// Returns [genre1, genre2, genre3, genre4, genre5, `spectral_pref`,
    ///          `beat_interval_norm`, regularity, `docas_rhythm_0..4`, `trust_coupling`]
    fn music_to_vector(&self, music: &MusicState) -> Vec<f32> {
        // Normalize beat_interval from [40, 200] to [0, 1]
        let beat_interval_norm = ((music.cadence.beat_interval - 40.0) / 160.0).clamp(0.0, 1.0);

        vec![
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            music.genre_preferences[0],      // Classical
            music.genre_preferences[1],      // Dance
            music.genre_preferences[2],      // Melancholic
            music.genre_preferences[3],      // Traditional
            music.genre_preferences[4],      // Ambient
            music.spectral_preference.abs(), // Spectral (normalized to [0, 1])
            // Cadence features for cross-domain alignment
            beat_interval_norm,
            music.cadence.regularity,
            music.cadence.docas_rhythm[0], // Dopamine rhythm
            music.cadence.docas_rhythm[1], // Oxytocin rhythm
            music.cadence.docas_rhythm[2], // Cortisol rhythm
            music.cadence.docas_rhythm[3], // Adrenaline rhythm
            music.cadence.docas_rhythm[4], // Serotonin rhythm
            music.cadence.trust_coupling,
        ]
    }

    /// Compute music domain weight with language adjustment
    /// Increases music weight when language alignment is low (cross-language bonding)
    fn compute_music_weight(&self, language_score: f32) -> f32 {
        let base_weight = self.weights[DOMAIN_MUSIC];

        // If language alignment is low (< 0.4), boost music weight
        // This enables agents to bond through music when they can't communicate
        if language_score < 0.4 {
            let language_deficit = 0.4 - language_score;
            let boost = language_deficit * 0.5; // Up to 0.2 boost
            (base_weight + boost).min(0.35) // Cap at 0.35
        } else {
            base_weight
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Train all alignments
    pub fn train_all(
        &mut self,
        emotion_data: Option<(Vec<Vec<f32>>, Vec<DocasState>)>,
        language_data: Option<(Vec<Vec<f32>>, Vec<DocasState>)>,
        music_data: Option<(Vec<Vec<f32>>, Vec<DocasState>)>,
    ) {
        if let Some((domain, docas)) = emotion_data {
            self.emotion_alignment.train_ols(&domain, &docas);
        }

        if let Some((domain, docas)) = language_data {
            self.language_alignment.train_ols(&domain, &docas);
        }

        if let Some((domain, docas)) = music_data {
            self.music_alignment.train_ols(&domain, &docas);
        }
    }

    /// Get alignment for a specific domain
    #[must_use]
    pub const fn get_alignment(&self, domain: usize) -> Option<&DomainAlignment> {
        match domain {
            DOMAIN_EMOTION => Some(&self.emotion_alignment),
            DOMAIN_LANGUAGE => Some(&self.language_alignment),
            DOMAIN_MUSIC => Some(&self.music_alignment),
            _ => None,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Multi-agent cross-domain coherence analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationCoherence {
    /// Per-agent `Ψ_trust` history
    pub psi_history: Vec<CrossDomainTrust>,

    /// Population-level coherence over time
    pub population_coherence: Vec<(u64, f32)>,

    /// Domain agreement matrix
    pub domain_agreement: [[f32; DOMAIN_COUNT]; DOMAIN_COUNT],
}

impl Default for PopulationCoherence {
    fn default() -> Self {
        Self {
            psi_history: Vec::new(),
            population_coherence: Vec::new(),
            domain_agreement: [[0.0; DOMAIN_COUNT]; DOMAIN_COUNT],
        }
    }
}

impl PopulationCoherence {
    /// Record `Ψ_trust` for an agent
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub fn record_psi(&mut self, psi: CrossDomainTrust) {
        self.psi_history.push(psi);
    }

    /// Compute population-level coherence
    pub fn compute_population_coherence(&mut self, tick: u64) -> f32 {
        if self.psi_history.is_empty() {
            return 0.0;
        }

        // Average coherence across population
        let avg_coherence: f32 = self
            .psi_history
            .iter()
            .map(|psi| psi.coherence)
            .sum::<f32>()
            / self.psi_history.len() as f32;

        self.population_coherence.push((tick, avg_coherence));

        // Compute domain agreement matrix
        for i in 0..DOMAIN_COUNT {
            for j in 0..DOMAIN_COUNT {
                let agreement: f32 = self
                    .psi_history
                    .iter()
                    .map(|psi| psi.domain_agreement(i, j))
                    .sum::<f32>()
                    / self.psi_history.len() as f32;
                self.domain_agreement[i][j] = agreement;
            }
        }

        avg_coherence
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get coherence trend (increasing = improving synchronization)
    #[must_use]
    pub fn coherence_trend(&self, window: usize) -> f32 {
        if self.population_coherence.len() < window {
            return 0.0;
        }

        let recent: Vec<f32> = self
            .population_coherence
            .iter()
            .rev()
            .take(window)
            .map(|(_, c)| *c)
            .collect();

        let older: Vec<f32> = self
            .population_coherence
            .iter()
            .rev()
            .skip(window)
            .take(window)
            .map(|(_, c)| *c)
            .collect();

        if older.is_empty() {
            return 0.0;
        }

        let recent_avg = recent.iter().sum::<f32>() / recent.len() as f32;
        let older_avg = older.iter().sum::<f32>() / older.len() as f32;

        recent_avg - older_avg
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check TFP-C1: Multimodal sync predicts stability
    /// Returns correlation between coherence and stability
    #[must_use]
    pub fn validate_tfp_c1(&self) -> f32 {
        if self.population_coherence.len() < 10 {
            return 0.0;
        }

        // Use coherence as proxy for sync, Ψ_trust as stability
        let coherences: Vec<f32> = self.population_coherence.iter().map(|(_, c)| *c).collect();

        let trusts: Vec<f32> = self.psi_history.iter().map(|psi| psi.psi_trust).collect();

        pearson_correlation(&coherences, &trusts)
    }
}

/// Pearson correlation helper
fn pearson_correlation(x: &[f32], y: &[f32]) -> f32 {
    crate::stats::pearson_correlation_f32(x, y)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// TFP-C1 validation: Multimodal synchronization predicts stability
/// Per §6.5.1: AUC ≈ 0.91 vs baseline 0.70
#[must_use]
pub fn validate_multimodal_prediction(
    psi_trust_values: &[f32],
    stability_outcomes: &[bool],
) -> f32 {
    // Compute AUC-like metric (simplified)
    // Higher Ψ_trust should correlate with positive outcomes

    let mut correct = 0;
    let total = stability_outcomes.len();

    for (i, &stable) in stability_outcomes.iter().enumerate() {
        if i >= psi_trust_values.len() {
            break;
        }

        let predicted_stable = psi_trust_values[i] > 0.75; // Threshold
        if predicted_stable == stable {
            correct += 1;
        }
    }

    if total > 0 {
        correct as f32 / total as f32
    } else {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docas::DocasState;
    use crate::emotion::EmotionCategory;

    #[test]
    fn test_cross_domain_trust_default() {
        let psi = CrossDomainTrust::default();
        assert_eq!(psi.domain_scores.len(), DOMAIN_COUNT);
        assert_eq!(psi.domain_weights.len(), DOMAIN_COUNT);
    }

    #[test]
    fn test_cross_domain_trust_composite() {
        let mut psi = CrossDomainTrust::new(0);
        psi.domain_scores = [1.0, 0.8, 0.6, 0.9];
        let composite = psi.compute_composite();

        // Should be weighted average
        let expected = 1.0 * 0.40 + 0.8 * 0.25 + 0.6 * 0.20 + 0.9 * 0.15;
        assert!((composite - expected).abs() < 0.01);
    }

    #[test]
    fn test_domain_alignment_projection() {
        let alignment = DomainAlignment::new(DOMAIN_EMOTION, 3);
        let domain_vec = vec![0.5, 0.3, 0.8];
        let docas = alignment.project(&domain_vec);

        // Check DOCAS is in valid range
        for level in &docas.levels {
            assert!((0.0..=1.0).contains(level));
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_cross_domain_alignment_computation() {
        let alignment = CrossDomainAlignment::new();
        let docas = DocasState::neutral();

        let psi = alignment.compute_psi_trust(&docas, None, None, None);

        // Should have valid composite
        assert!(psi.psi_trust > 0.0);
        assert_eq!(psi.domain_scores[DOMAIN_DOCAS], docas.compute_tau());
    }

    #[test]
    fn test_domain_agreement() {
        let mut psi = CrossDomainTrust::new(0);
        psi.domain_scores = [0.8, 0.8, 0.8, 0.8];

        // All domains agree
        assert!(psi.domain_agreement(DOMAIN_DOCAS, DOMAIN_EMOTION) > 0.9);

        psi.domain_scores = [0.8, 0.3, 0.8, 0.8];
        // One domain disagrees
        assert!(psi.domain_agreement(DOMAIN_DOCAS, DOMAIN_EMOTION) < 0.8);
    }

    #[test]
    fn test_coherence_computation() {
        let mut psi = CrossDomainTrust::new(0);

        // Low variance = high coherence
        psi.domain_scores = [0.75, 0.76, 0.74, 0.75];
        psi.compute_composite();
        assert!(psi.coherence > 0.9);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // High variance = low coherence
        psi.domain_scores = [0.3, 1.2, 0.5, 1.0];
        psi.compute_composite();
        assert!(psi.coherence < 0.8);
    }

    #[test]
    fn test_population_coherence() {
        let mut pop = PopulationCoherence::default();

        // Add some Ψ_trust records
        for i in 0..10 {
            let mut psi = CrossDomainTrust::new(i as u64);
            psi.coherence = 0.7 + i as f32 * 0.02;
            pop.record_psi(psi);
        }

        let avg_coherence = pop.compute_population_coherence(9);
        assert!(avg_coherence > 0.7);
    }

    #[test]
    fn test_emotion_to_vector() {
        let alignment = CrossDomainAlignment::new();
        let classification = EmotionClassification {
            primary: EmotionCategory::Joy,
            confidence: 0.9,
            all_scores: [(EmotionCategory::Joy, 0.9); 10],
            delta_tau: 0.5,
            delta_tau_velocity: 0.1,
            tau_zero: 0.75,
            tau: 1.25,
        };

        let vec = alignment.emotion_to_vector(&classification);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 1.0); // Valence of Joy
        assert_eq!(vec[1], 0.7); // Arousal of Joy
        assert_eq!(vec[2], 0.5); // Intensity
    }

    #[test]
    fn test_music_to_vector() {
        let alignment = CrossDomainAlignment::new();
        let music = MusicState::default();

        let vec = alignment.music_to_vector(&music);
        assert_eq!(vec.len(), 14); // 5 genres + spectral + 7 cadence features
        assert_eq!(vec[0], 0.2); // Classical preference
        assert_eq!(vec[5], 0.5); // Spectral preference (abs of -0.5)
                                 // Cadence features
        assert!(vec[6] >= 0.0 && vec[6] <= 1.0); // beat_interval_norm
        assert!(vec[7] >= 0.0 && vec[7] <= 1.0); // regularity
                                                 // DOCAS rhythm (5 values)
        for item in &vec[8..13] {
            assert!(*item >= 0.0 && *item <= 1.0);
        }
        assert!(vec[13] >= 0.0 && vec[13] <= 1.0); // trust_coupling
    }

    #[test]
    fn test_music_weight_language_adjustment() {
        let alignment = CrossDomainAlignment::new();

        // Low language alignment should boost music weight
        let low_lang_weight = alignment.compute_music_weight(0.2);
        let high_lang_weight = alignment.compute_music_weight(0.8);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(
            low_lang_weight > high_lang_weight,
            "Music weight should be higher when language alignment is low"
        );
        assert!(
            low_lang_weight > alignment.weights[DOMAIN_MUSIC],
            "Low language should boost music weight above base"
        );
    }

    #[test]
    fn test_psi_trust_with_language_adjustment() {
        use crate::docas::DocasState;

        let alignment = CrossDomainAlignment::new();
        let docas = DocasState::neutral();

        // Test with low language similarity - should boost music weight
        let psi_low_lang =
            alignment.compute_psi_trust_with_language(&docas, None, None, None, Some(0.2));

        // Test with high language similarity - should use base music weight
        let psi_high_lang =
            alignment.compute_psi_trust_with_language(&docas, None, None, None, Some(0.8));

        // Both should have valid composite scores
        assert!(psi_low_lang.psi_trust > 0.0);
        assert!(psi_high_lang.psi_trust > 0.0);

        // Low language should have higher music weight
        assert!(
            psi_low_lang.domain_weights[DOMAIN_MUSIC] > psi_high_lang.domain_weights[DOMAIN_MUSIC],
            "Low language similarity should increase music domain weight"
        );
    }
}
