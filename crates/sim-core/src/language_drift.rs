//! Language Drift Tracking Module
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION language dynamics, not `FreshCredit` product language
//!
//! Per §6.11 Linguistics Empirical Predictions:
//! - LING-C1: Semantic drift rate ∝ 1/τ (r ≈ 0.60-0.80)
//! - LING-C2: Trust betrayal → embedding entropy spike (d > 0.80)
//! - LING-C4: Low τ → high lexical turnover (r ≈ -0.65 to -0.80)
//!
//! ## Core Hypothesis: Language Drift ∝ 1/τ
//!
//! When trust (τ) is low, semantic alignment between agents degrades faster.
//! When trust (τ) is high, lexicons stabilize and semantic convergence occurs.

// TAG: surface=simulation owner=research-team rule=SIM-001
use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::language::{ConceptCategory, Lexicon};
use crate::types::{AgentId, ConceptId, Tick};

/// Window size for drift rate computation (ticks)
pub const DRIFT_WINDOW_SIZE: usize = 100;

/// Minimum samples required for reliable drift measurement
pub const MIN_DRIFT_SAMPLES: usize = 10;

/// Alignment factor for semantic convergence during communication
/// Per §8.2.1: `align_toward(&concept.embedding`, 0.1)
pub const DEFAULT_ALIGNMENT_FACTOR: f32 = 0.1;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Language drift measurement for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDriftTracker {
    /// Agent ID
    pub agent_id: AgentId,

    /// History of lexicon states for drift computation
    pub lexicon_history: VecDeque<(Tick, LexiconSnapshot)>,

    /// History of trust values (τ) for correlation analysis
    pub tau_history: VecDeque<(Tick, f32)>,

    /// Computed drift rates per concept
    pub concept_drift_rates: HashMap<ConceptId, f32>,

    /// Overall lexical drift rate
    pub aggregate_drift_rate: f32,

    /// Drift velocity (change in drift rate)
    pub drift_acceleration: f32,

    /// Semantic entropy history
    pub entropy_history: VecDeque<(Tick, f32)>,

    /// Betrayal events (tick, severity) for LING-C2 validation
    pub betrayal_events: Vec<(Tick, f32)>,

    /// Maximum history size
    max_history: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Snapshot of lexicon state for drift computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconSnapshot {
    /// Concept embeddings at this tick
    pub embeddings: HashMap<ConceptId, [f32; 8]>,

    /// Concept categories
    pub categories: HashMap<ConceptId, ConceptCategory>,

    /// Tick when snapshot was taken
    pub tick: Tick,
}

impl From<&Lexicon> for LexiconSnapshot {
    fn from(lexicon: &Lexicon) -> Self {
        let mut embeddings = HashMap::new();
        let mut categories = HashMap::new();

        for (id, concept) in &lexicon.concepts {
            embeddings.insert(*id, concept.embedding.values);
            categories.insert(*id, concept.category);
        }

        Self {
            embeddings,
            categories,
            tick: 0, // Will be set by caller
        }
    }
}

impl Default for LanguageDriftTracker {
    fn default() -> Self {
        Self {
            agent_id: 0,
            lexicon_history: VecDeque::with_capacity(DRIFT_WINDOW_SIZE),
            tau_history: VecDeque::with_capacity(DRIFT_WINDOW_SIZE),
            concept_drift_rates: HashMap::new(),
            aggregate_drift_rate: 0.0,
            drift_acceleration: 0.0,
            entropy_history: VecDeque::with_capacity(DRIFT_WINDOW_SIZE),
            betrayal_events: Vec::new(),
            max_history: DRIFT_WINDOW_SIZE,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl LanguageDriftTracker {
    /// Create new drift tracker for an agent
    #[must_use]
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            ..Default::default()
        }
    }

    /// Record lexicon snapshot at current tick
    pub fn record_lexicon(&mut self, tick: Tick, lexicon: &Lexicon) {
        let mut snapshot = LexiconSnapshot::from(lexicon);
        snapshot.tick = tick;

        self.lexicon_history.push_back((tick, snapshot));
        if self.lexicon_history.len() > self.max_history {
            self.lexicon_history.pop_front();
        }

        // Compute entropy
        let entropy = self.compute_lexical_entropy(lexicon);
        self.entropy_history.push_back((tick, entropy));
        if self.entropy_history.len() > self.max_history {
            self.entropy_history.pop_front();
        }
    }

    /// Record trust value at current tick
    pub fn record_tau(&mut self, tick: Tick, tau: f32) {
        self.tau_history.push_back((tick, tau));
        if self.tau_history.len() > self.max_history {
            self.tau_history.pop_front();
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record betrayal event for LING-C2 analysis
    pub fn record_betrayal(&mut self, tick: Tick, severity: f32) {
        self.betrayal_events.push((tick, severity));
    }

    /// Compute semantic drift rate per concept
    /// Per §6.11.2: `drift_rate` = `cosine_distance(current`, initial) / ticks
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn compute_concept_drift(&mut self) {
        let (Some((current_tick, current)), Some((initial_tick, initial))) =
            (self.lexicon_history.back(), self.lexicon_history.front())
        else {
            return;
        };

        let ticks_elapsed = (current_tick - initial_tick).max(1) as f32;

        self.concept_drift_rates.clear();

        for (concept_id, current_embedding) in &current.embeddings {
            if let Some(initial_embedding) = initial.embeddings.get(concept_id) {
                let distance = cosine_distance_8d(current_embedding, initial_embedding);
                let drift_rate = distance / ticks_elapsed;
                self.concept_drift_rates.insert(*concept_id, drift_rate);
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute aggregate drift rate across all concepts
    pub fn compute_aggregate_drift(&mut self) -> f32 {
        self.compute_concept_drift();

        if self.concept_drift_rates.is_empty() {
            return 0.0;
        }

        let total: f32 = self.concept_drift_rates.values().sum();
        self.aggregate_drift_rate = total / self.concept_drift_rates.len() as f32;

        // Compute acceleration (change in drift rate)
        self.drift_acceleration = self.compute_drift_acceleration();

        self.aggregate_drift_rate
    }

    /// Compute drift acceleration (second derivative)
    fn compute_drift_acceleration(&self) -> f32 {
        if self.lexicon_history.len() < 3 {
            return 0.0;
        }

        // Compare recent vs older drift
        let mid_point = self.lexicon_history.len() / 2;

        let recent: Vec<_> = self.lexicon_history.iter().skip(mid_point).collect();
        let older: Vec<_> = self.lexicon_history.iter().take(mid_point).collect();

        if recent.len() < 2 || older.len() < 2 {
            return 0.0;
        }

        // Average drift in recent period vs older period
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let recent_avg = self.avg_drift_in_slice(&recent);
        let older_avg = self.avg_drift_in_slice(&older);

        recent_avg - older_avg
    }

    /// Helper: average drift in a slice of history
    fn avg_drift_in_slice(&self, slice: &[&(Tick, LexiconSnapshot)]) -> f32 {
        if slice.len() < 2 {
            return 0.0;
        }

        let mut total_drift = 0.0;
        let mut count = 0;

        for i in 1..slice.len() {
            let current = &slice[i].1;
            let previous = &slice[i - 1].1;

            for (concept_id, curr_emb) in &current.embeddings {
                if let Some(prev_emb) = previous.embeddings.get(concept_id) {
                    total_drift += cosine_distance_8d(curr_emb, prev_emb);
                    count += 1;
                }
            }
        }

        if count > 0 {
            total_drift / count as f32
        } else {
            0.0
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute lexical entropy (Shannon H of embeddings)
    /// Per §8.3.6: `embedding_entropy` = Shannon H of concept embeddings
    #[must_use]
    pub fn compute_lexical_entropy(&self, lexicon: &Lexicon) -> f32 {
        if lexicon.is_empty() {
            return 0.0;
        }

        // Collect all embedding values
        let mut all_values: Vec<f32> = Vec::new();
        for concept in lexicon.concepts.values() {
            all_values.extend_from_slice(&concept.embedding.values);
        }

        if all_values.is_empty() {
            return 0.0;
        }

        // Compute variance as proxy for entropy
        let mean = all_values.iter().sum::<f32>() / all_values.len() as f32;
        let variance =
            all_values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / all_values.len() as f32;

        // Convert variance to entropy-like measure
        variance.ln_1p()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get entropy change after betrayal (LING-C2 validation)
    /// Returns (`pre_betrayal_entropy`, `post_betrayal_entropy`, delta)
    #[must_use]
    pub fn betrayal_entropy_effect(&self, betrayal_tick: Tick) -> Option<(f32, f32, f32)> {
        // Find entropy before betrayal (10 ticks prior)
        let pre_entropy = self
            .entropy_history
            .iter()
            .rev()
            .find(|(t, _)| *t <= betrayal_tick.saturating_sub(10))
            .map(|(_, e)| *e)?;

        // Find entropy after betrayal (10 ticks after)
        let post_entropy = self
            .entropy_history
            .iter()
            .find(|(t, _)| *t >= betrayal_tick + 10)
            .map(|(_, e)| *e)?;

        let delta = post_entropy - pre_entropy;
        Some((pre_entropy, post_entropy, delta))
    }

    /// Compute entropy effects for all recorded betrayal events (LING-C2)
    #[must_use]
    pub fn all_betrayal_entropy_effects(&self) -> Vec<(Tick, f32, f32, f32)> {
        self.betrayal_events
            .iter()
            .filter_map(|&(tick, _severity)| {
                self.betrayal_entropy_effect(tick)
                    .map(|(pre, post, delta)| (tick, pre, post, delta))
            })
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Test LING-C1: Drift ∝ 1/τ
    /// Returns correlation coefficient between drift rate and 1/τ
    #[must_use]
    pub fn test_ling_c1(&self) -> f32 {
        if self.tau_history.len() < MIN_DRIFT_SAMPLES
            || self.lexicon_history.len() < MIN_DRIFT_SAMPLES
        {
            return 0.0;
        }

        // Build aligned (drift, 1/tau) pairs
        let mut drift_values = Vec::new();
        let mut inv_tau_values = Vec::new();

        for (tick, tau) in &self.tau_history {
            if let Some(drift) = self.get_drift_at_tick(*tick) {
                drift_values.push(drift);
                inv_tau_values.push(1.0 / tau.max(0.1)); // Avoid div by zero
            }
        }

        if drift_values.len() < MIN_DRIFT_SAMPLES {
            return 0.0;
        }

        pearson_correlation(&drift_values, &inv_tau_values)
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Get estimated drift at a specific tick
    fn get_drift_at_tick(&self, tick: Tick) -> Option<f32> {
        // Find closest lexicon snapshots
        let idx = self.lexicon_history.iter().position(|(t, _)| *t >= tick)?;

        if idx == 0 {
            return Some(0.0);
        }

        let (t2, snap2) = &self.lexicon_history[idx];
        let (t1, snap1) = &self.lexicon_history[idx - 1];

        let ticks = (t2 - t1).max(1) as f32;
        let mut total_drift = 0.0;
        let mut count = 0;

        for (concept_id, emb2) in &snap2.embeddings {
            if let Some(emb1) = snap1.embeddings.get(concept_id) {
                total_drift += cosine_distance_8d(emb2, emb1) / ticks;
                count += 1;
            }
        }

        if count > 0 {
            Some(total_drift / count as f32)
        } else {
            None
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get trust-cluster drift rate
    /// Per §6.11.4: Trust-cluster concepts = reliability, honesty, verification, bond
    #[must_use]
    pub fn trust_cluster_drift(&self) -> f32 {
        let trust_concepts: Vec<_> = self
            .concept_drift_rates
            .iter()
            .filter(|(id, _)| {
                // Trust-cluster concept IDs: 0=reliability, 1=honesty, 2=verification, 3=bond
                [0, 1, 2, 3].contains(id)
            })
            .map(|(_, rate)| *rate)
            .collect();

        if trust_concepts.is_empty() {
            return 0.0;
        }

        trust_concepts.iter().sum::<f32>() / trust_concepts.len() as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get non-trust-cluster drift rate (for comparison)
    #[must_use]
    pub fn non_trust_cluster_drift(&self) -> f32 {
        let non_trust: Vec<_> = self
            .concept_drift_rates
            .iter()
            .filter(|(id, _)| {
                ![0, 1, 2, 3].contains(id) // Non-trust concepts
            })
            .map(|(_, rate)| *rate)
            .collect();

        if non_trust.is_empty() {
            return 0.0;
        }

        non_trust.iter().sum::<f32>() / non_trust.len() as f32
    }
}

/// Multi-agent language drift analyzer
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopulationLanguageDrift {
    /// Per-agent drift trackers
    pub trackers: HashMap<AgentId, LanguageDriftTracker>,

    /// Cross-agent semantic alignment matrix
    pub alignment_matrix: HashMap<(AgentId, AgentId), f32>,

    /// Population-wide lexical turnover rate
    pub turnover_rate: f32,

    /// Community drift rate (average across agents)
    pub community_drift: f32,
}

impl PopulationLanguageDrift {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new population drift analyzer
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or get drift tracker for an agent
    pub fn tracker_for(&mut self, agent_id: AgentId) -> &mut LanguageDriftTracker {
        self.trackers
            .entry(agent_id)
            .or_insert_with(|| LanguageDriftTracker::new(agent_id))
    }

    /// Compute cross-agent semantic alignment
    /// Returns average cosine similarity between agent lexicons
    pub fn compute_alignment(&mut self) -> f32 {
        let agent_ids: Vec<_> = self.trackers.keys().copied().collect();

        if agent_ids.len() < 2 {
            return 1.0; // Perfect alignment with only one agent
        }

        let mut total_alignment = 0.0;
        let mut pair_count = 0;

        for i in 0..agent_ids.len() {
            for j in (i + 1)..agent_ids.len() {
                let id1 = agent_ids[i];
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                let id2 = agent_ids[j];

                let alignment = self.compute_pair_alignment(id1, id2);
                self.alignment_matrix.insert((id1, id2), alignment);
                total_alignment += alignment;
                pair_count += 1;
            }
        }

        if pair_count > 0 {
            total_alignment / pair_count as f32
        } else {
            1.0
        }
    }

    /// Compute semantic alignment between two agents
    fn compute_pair_alignment(&self, id1: AgentId, id2: AgentId) -> f32 {
        let Some(tracker1) = self.trackers.get(&id1) else {
            return 0.0;
        };
        let Some(tracker2) = self.trackers.get(&id2) else {
            return 0.0;
        };

        // Get latest snapshots
        let Some((_, snap1)) = tracker1.lexicon_history.back() else {
            return 0.0;
        };
        let Some((_, snap2)) = tracker2.lexicon_history.back() else {
            return 0.0;
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Compute average cosine similarity across shared concepts
        let mut total_sim = 0.0;
        let mut count = 0;

        for (concept_id, emb1) in &snap1.embeddings {
            if let Some(emb2) = snap2.embeddings.get(concept_id) {
                total_sim += cosine_similarity_8d(emb1, emb2);
                count += 1;
            }
        }

        if count > 0 {
            total_sim / count as f32
        } else {
            0.0
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Test LING-C4: Lexical turnover vs τ
    /// Returns correlation between new words and 1/τ
    #[must_use]
    pub fn test_ling_c4(&self) -> f32 {
        // Collect per-agent data
        let mut turnover_rates = Vec::new();
        let mut mean_taus = Vec::new();

        for tracker in self.trackers.values() {
            if let Some(turnover) = self.estimate_turnover(tracker) {
                let mean_tau: f32 = tracker.tau_history.iter().map(|(_, tau)| *tau).sum::<f32>()
                    / tracker.tau_history.len().max(1) as f32;

                turnover_rates.push(turnover);
                mean_taus.push(mean_tau);
            }
        }

        if turnover_rates.len() < MIN_DRIFT_SAMPLES {
            return 0.0;
        }

        // Correlation should be negative (low τ → high turnover)
        pearson_correlation(&turnover_rates, &mean_taus)
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Estimate lexical turnover rate for a tracker
    fn estimate_turnover(&self, tracker: &LanguageDriftTracker) -> Option<f32> {
        if tracker.lexicon_history.len() < 2 {
            return None;
        }

        // Count new/changed concepts between first and last snapshot
        let (_, first) = tracker.lexicon_history.front()?;
        let (_, last) = tracker.lexicon_history.back()?;

        let mut changed = 0;
        let total = first.embeddings.len().max(last.embeddings.len()) as f32;

        for (concept_id, emb1) in &first.embeddings {
            if let Some(emb2) = last.embeddings.get(concept_id) {
                if cosine_distance_8d(emb1, emb2) > 0.1 {
                    changed += 1;
                }
            } else {
                changed += 1; // Concept lost
            }
        }

        // Add new concepts
        for concept_id in last.embeddings.keys() {
            if !first.embeddings.contains_key(concept_id) {
                changed += 1;
            }
        }

        Some(changed as f32 / total)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update community-wide drift metrics
    pub fn update_community_metrics(&mut self) {
        let mut total_drift = 0.0;
        let mut count = 0;

        for tracker in self.trackers.values() {
            total_drift += tracker.aggregate_drift_rate;
            count += 1;
        }

        if count > 0 {
            self.community_drift = total_drift / count as f32;
        }

        // Compute alignment
        self.compute_alignment();
    }
}

/// Cosine distance between two 8D vectors
fn cosine_distance_8d(a: &[f32; 8], b: &[f32; 8]) -> f32 {
    1.0 - cosine_similarity_8d(a, b)
}

/// Cosine similarity between two 8D vectors
fn cosine_similarity_8d(a: &[f32; 8], b: &[f32; 8]) -> f32 {
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for i in 0..8 {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    let denom = (norm_a.sqrt() * norm_b.sqrt()).max(1e-8);
    (dot / denom).clamp(-1.0, 1.0)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Pearson correlation coefficient
fn pearson_correlation(x: &[f32], y: &[f32]) -> f32 {
    crate::stats::pearson_correlation_f32(x, y)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Drift rate prediction based on trust (LING-C1 model)
/// `drift_rate` = k / τ
#[must_use]
pub fn predict_drift_rate(tau: f32, k: f32) -> f32 {
    k / tau.max(0.1) // Avoid division by zero
}

/// LING-C1 validation: Test if drift ∝ 1/τ across population
/// Returns (correlation, `p_value_approx`)
#[must_use]
pub fn validate_ling_c1(drift_rates: &[f32], taus: &[f32]) -> (f32, f32) {
    let inv_taus: Vec<f32> = taus.iter().map(|tau| 1.0 / tau.max(0.1)).collect();

    let r = pearson_correlation(drift_rates, &inv_taus);

    // Approximate p-value using Fisher z-transform
    let n = drift_rates.len() as f32;
    let z = 0.5 * ((1.0 + r) / (1.0 - r)).ln();
    let se = 1.0 / (n - 3.0).sqrt();
    let z_score = z.abs() / se;

    // Two-tailed p-value approximation
    let p = 2.0 * (1.0 - (z_score / (2.0f32).sqrt()).tanh());

    (r, p)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// LING-C2 validation: Betrayal entropy effect
/// Returns average entropy increase after betrayal
#[must_use]
pub fn validate_ling_c2(trackers: &[&LanguageDriftTracker]) -> f32 {
    let mut total_delta = 0.0;
    let mut count = 0;

    for tracker in trackers {
        for (betrayal_tick, _) in &tracker.betrayal_events {
            if let Some((_, _, delta)) = tracker.betrayal_entropy_effect(*betrayal_tick) {
                total_delta += delta;
                count += 1;
            }
        }
    }

    if count > 0 {
        total_delta / count as f32
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::{Concept, ConceptCategory, Lexicon};
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_drift_tracker_creation() {
        let tracker = LanguageDriftTracker::new(1);
        assert_eq!(tracker.agent_id, 1);
        assert_eq!(tracker.aggregate_drift_rate, 0.0);
    }

    #[test]
    fn test_lexicon_snapshot() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut lexicon = Lexicon::new();
        lexicon.add_concept(Concept::new(0, ConceptCategory::Trust, 0, &mut rng));

        let snapshot = LexiconSnapshot::from(&lexicon);
        assert!(snapshot.embeddings.contains_key(&0));
        assert_eq!(snapshot.categories[&0], ConceptCategory::Trust);
    }

    #[test]
    fn test_pearson_correlation() {
        // Perfect positive correlation
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = pearson_correlation(&x, &y);
        assert!((r - 1.0).abs() < 0.001);

        // Perfect negative correlation
        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r_neg = pearson_correlation(&x, &y_neg);
        assert!((r_neg - (-1.0)).abs() < 0.001);

        // No correlation
        let y_flat = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let r_flat = pearson_correlation(&x, &y_flat);
        assert!(r_flat.abs() < 0.001);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_predict_drift_rate() {
        // High trust = low drift
        let drift_high_trust = predict_drift_rate(1.5, 0.1);
        // Low trust = high drift
        let drift_low_trust = predict_drift_rate(0.5, 0.1);

        assert!(drift_low_trust > drift_high_trust);
    }

    #[test]
    fn test_cosine_similarity_8d() {
        let a = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let sim = cosine_similarity_8d(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);

        let c = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let sim_orth = cosine_similarity_8d(&a, &c);
        assert!(sim_orth.abs() < 0.001);
    }

    #[test]
    fn test_population_drift() {
        let mut pop = PopulationLanguageDrift::new();

        // Create trackers for multiple agents
        for i in 0..5 {
            let tracker = pop.tracker_for(i);
            tracker.record_tau(0, 0.75 + i as f32 * 0.1);
        }

        assert_eq!(pop.trackers.len(), 5);
    }
}
