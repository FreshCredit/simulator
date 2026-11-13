//! Language module: Lexicons, colexification, and semantic communication
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION language, not `FreshCredit` product language
//! SIM-004: Deterministic - same seed produces identical lexicon evolution
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! Per §4379-4385: Stage 4 Language implementation
//! Per §8.2.1 lines 4723-4746: Language Toggle Contract
//! Per §6.11: Unified Linguistics empirical predictions
//!
//! ## Key Concepts
//! - **Lexicon**: Agent's vocabulary mapping concepts to embeddings
//! - **Colexification**: Cross-linguistic phenomenon where one word expresses multiple concepts
//! - **Semantic Drift**: Gradual change in concept embeddings over time
//! - **Trust Centrality**: Trust-cluster concepts have maximal betweenness centrality

use indexmap::IndexMap;
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::clics3_data::apply_clics3_colexifications;
use crate::types::ConceptId;

/// Embedding dimension for semantic vectors
/// Per §6.11: Embeddings represent semantic meaning in vector space
pub const EMBEDDING_DIM: usize = 8;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Semantic embedding vector
/// Per §6.11.2: Agents maintain concept→embedding mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    /// Vector components (normalized to unit length)
    pub values: [f32; EMBEDDING_DIM],
}

impl Default for Embedding {
    fn default() -> Self {
        Self {
            values: [0.0; EMBEDDING_DIM],
        }
    }
}

impl Embedding {
    /// Create a new embedding with given values
    #[must_use]
    pub const fn new(values: [f32; EMBEDDING_DIM]) -> Self {
        Self { values }
    }

    /// Create a random embedding (normalized)
    pub fn random(rng: &mut StdRng) -> Self {
        let mut values = [0.0f32; EMBEDDING_DIM];
        let mut magnitude = 0.0f32;
        for v in &mut values {
            *v = rng.gen::<f32>() * 2.0 - 1.0; // [-1, 1]
            magnitude += *v * *v;
        }
        magnitude = magnitude.sqrt();
        if magnitude > 0.0 {
            for v in &mut values {
                *v /= magnitude;
            }
        }
        Self { values }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Cosine similarity between two embeddings
    /// Per §6.11.2: Semantic distance measured via cosine
    #[must_use]
    pub fn cosine_similarity(&self, other: &Self) -> f32 {
        let mut dot = 0.0f32;
        let mut mag_a = 0.0f32;
        let mut mag_b = 0.0f32;
        for i in 0..EMBEDDING_DIM {
            if !self.values[i].is_finite() || !other.values[i].is_finite() {
                continue;
            }
            dot += self.values[i] * other.values[i];
            mag_a += self.values[i] * self.values[i];
            mag_b += other.values[i] * other.values[i];
        }
        let denom = mag_a.sqrt() * mag_b.sqrt();
        if denom <= 1e-8 || !denom.is_finite() {
            0.0
        } else {
            (dot / denom).clamp(-1.0, 1.0)
        }
    }

    /// Cosine distance (1 - similarity)
    #[must_use]
    pub fn cosine_distance(&self, other: &Self) -> f32 {
        (1.0 - self.cosine_similarity(other)).clamp(0.0, 2.0)
    }

    /// Align this embedding toward another by a factor
    /// Per §8.2.1 line 4732: `align_toward(&concept.embedding`, 0.1)
    pub fn align_toward(&mut self, other: &Self, factor: f32) {
        let f = if factor.is_finite() {
            factor.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        for i in 0..EMBEDDING_DIM {
            if self.values[i].is_finite() && other.values[i].is_finite() {
                self.values[i] += f * (other.values[i] - self.values[i]);
            }
        }
        // Re-normalize
        let mut magnitude = 0.0f32;
        for v in &self.values {
            if v.is_finite() {
                magnitude += *v * *v;
            }
        }
        magnitude = magnitude.sqrt();
        if magnitude > 1e-12 && magnitude.is_finite() {
            for v in &mut self.values {
                *v /= magnitude;
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Concept category for semantic organization
/// Per §6.11.4: Trust-cluster concepts (reliability, honesty, bond, commitment)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConceptCategory {
    /// Trust-related concepts (reliability, honesty, verification, bond)
    Trust,
    /// Social concepts (cooperation, defection, group, norm)
    Social,
    /// Resource concepts (food, energy, territory)
    Resource,
    /// Emotion concepts (joy, fear, anger, sadness)
    Emotion,
    /// Action concepts (move, trade, share, attack)
    Action,
}

/// A semantic concept in an agent's lexicon
/// Per §7.5.1 line 4014: lexicon: `HashMap`<`ConceptId`, Embedding>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    /// Unique concept identifier
    pub id: ConceptId,
    /// Semantic category
    pub category: ConceptCategory,
    /// Semantic embedding
    pub embedding: Embedding,
    /// Concepts this is colexified with (same word form)
    /// Per §6.11.4: Colexification patterns from CLICS³
    pub colexifications: Vec<ConceptId>,
    /// Initial embedding (for drift measurement)
    pub initial_embedding: Embedding,
    /// Tick when concept was learned
    pub learned_tick: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Concept {
    /// Create a new concept with random embedding
    pub fn new(id: ConceptId, category: ConceptCategory, tick: u64, rng: &mut StdRng) -> Self {
        let embedding = Embedding::random(rng);
        Self {
            id,
            category,
            embedding: embedding.clone(),
            colexifications: Vec::new(),
            initial_embedding: embedding,
            learned_tick: tick,
        }
    }

    /// Compute semantic drift from initial embedding
    /// Per §6.11.2: `drift_rate` = `cosine_distance(current`, initial) / ticks
    #[must_use]
    pub fn drift_rate(&self, current_tick: u64) -> f32 {
        let ticks_elapsed = (current_tick - self.learned_tick).max(1) as f32;
        self.embedding.cosine_distance(&self.initial_embedding) / ticks_elapsed
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Agent's lexicon (vocabulary)
/// Per §7.5.1 line 4014: lexicon: `HashMap`<`ConceptId`, Embedding>
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Lexicon {
    /// Concepts indexed by ID
    pub concepts: IndexMap<ConceptId, Concept>,
    /// Colexification map: pairs of concepts sharing word forms
    /// Per §7.5.1 line 4015
    pub colexification_map: Vec<(ConceptId, ConceptId)>,
}

impl Lexicon {
    /// Create a new empty lexicon
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize lexicon with base concepts
    /// Per §6773: MVP = static lexicons with colexification from CLICS³ v3.0
    pub fn initialize_base(rng: &mut StdRng, tick: u64) -> Self {
        let mut lexicon = Self::new();

        // Trust-cluster concepts (per §6.11.4)
        lexicon.add_concept(Concept::new(0, ConceptCategory::Trust, tick, rng)); // reliability
        lexicon.add_concept(Concept::new(1, ConceptCategory::Trust, tick, rng)); // honesty
        lexicon.add_concept(Concept::new(2, ConceptCategory::Trust, tick, rng)); // verification
        lexicon.add_concept(Concept::new(3, ConceptCategory::Trust, tick, rng)); // bond

        // Social concepts
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        lexicon.add_concept(Concept::new(10, ConceptCategory::Social, tick, rng)); // cooperation
        lexicon.add_concept(Concept::new(11, ConceptCategory::Social, tick, rng)); // defection
        lexicon.add_concept(Concept::new(12, ConceptCategory::Social, tick, rng)); // group
        lexicon.add_concept(Concept::new(13, ConceptCategory::Social, tick, rng)); // norm

        // Resource concepts
        lexicon.add_concept(Concept::new(20, ConceptCategory::Resource, tick, rng)); // food
        lexicon.add_concept(Concept::new(21, ConceptCategory::Resource, tick, rng)); // energy
        lexicon.add_concept(Concept::new(22, ConceptCategory::Resource, tick, rng)); // territory

        // Emotion concepts (per §6.13: minimum 4 basic emotions)
        // BANNED-001: "basic emotions" is psychological terminology (Ekman/Plutchik)
        lexicon.add_concept(Concept::new(30, ConceptCategory::Emotion, tick, rng)); // joy
        lexicon.add_concept(Concept::new(31, ConceptCategory::Emotion, tick, rng)); // fear
        lexicon.add_concept(Concept::new(32, ConceptCategory::Emotion, tick, rng)); // anger
        lexicon.add_concept(Concept::new(33, ConceptCategory::Emotion, tick, rng)); // sadness

        // Action concepts
        lexicon.add_concept(Concept::new(40, ConceptCategory::Action, tick, rng)); // move
        lexicon.add_concept(Concept::new(41, ConceptCategory::Action, tick, rng)); // trade
        lexicon.add_concept(Concept::new(42, ConceptCategory::Action, tick, rng)); // share
        lexicon.add_concept(Concept::new(43, ConceptCategory::Action, tick, rng)); // attack

        // Apply CLICS³ colexification patterns (GAP-002)
        // Replaces hardcoded patterns with cross-linguistic frequency data
        apply_clics3_colexifications(&mut lexicon, rng);

        lexicon
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add a concept to the lexicon
    pub fn add_concept(&mut self, concept: Concept) {
        self.concepts.insert(concept.id, concept);
    }

    /// Add a colexification relationship
    pub fn add_colexification(&mut self, id1: ConceptId, id2: ConceptId) {
        self.colexification_map.push((id1, id2));
        // Update concept colexification lists
        if let Some(c1) = self.concepts.get_mut(&id1) {
            if !c1.colexifications.contains(&id2) {
                c1.colexifications.push(id2);
            }
        }
        if let Some(c2) = self.concepts.get_mut(&id2) {
            if !c2.colexifications.contains(&id1) {
                c2.colexifications.push(id1);
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get concept by ID
    #[must_use]
    pub fn get(&self, id: ConceptId) -> Option<&Concept> {
        self.concepts.get(&id)
    }

    /// Get mutable concept by ID
    pub fn get_mut(&mut self, id: ConceptId) -> Option<&mut Concept> {
        self.concepts.get_mut(&id)
    }

    /// Number of concepts in lexicon
    #[must_use]
    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    /// Check if lexicon is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    /// Compute average semantic drift rate
    /// Per §6.11.2: `drift_rate` = Σ `cosine_distance` / ticks
    #[must_use]
    pub fn average_drift_rate(&self, current_tick: u64) -> f32 {
        if self.concepts.is_empty() {
            return 0.0;
        }
        let total: f32 = self
            .concepts
            .values()
            .map(|c| c.drift_rate(current_tick))
            .sum();
        total / self.concepts.len() as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute lexicon overlap with another lexicon
    /// Per §7586: Stability condition = lexicon overlap > 0.3
    #[must_use]
    pub fn overlap(&self, other: &Self) -> f32 {
        if self.concepts.is_empty() || other.concepts.is_empty() {
            return 0.0;
        }
        let shared: usize = self
            .concepts
            .keys()
            .filter(|id| other.concepts.contains_key(*id))
            .count();
        let total = self.concepts.len().max(other.concepts.len());
        shared as f32 / total as f32
    }

    /// Align lexicon toward another (learning from communication)
    /// Per §8.2.1 line 4730-4733: Update receiver's lexicon via alignment
    pub fn align_toward(&mut self, other: &Self, factor: f32) {
        for (id, other_concept) in &other.concepts {
            if let Some(my_concept) = self.concepts.get_mut(id) {
                my_concept
                    .embedding
                    .align_toward(&other_concept.embedding, factor);
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Count concepts in trust cluster
    /// Per §6.11.4: Trust-cluster = reliability, honesty, verification, bond
    #[must_use]
    pub fn trust_cluster_count(&self) -> usize {
        self.concepts
            .values()
            .filter(|c| c.category == ConceptCategory::Trust)
            .count()
    }
}

/// Message for semantic communication between agents
/// Per §7.5.1 lines 4041-4048
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Sender agent ID
    pub sender_id: u64,
    /// Semantic content (concept IDs from sender's lexicon)
    pub content: Vec<ConceptId>,
    /// Tick when message was sent
    pub tick_sent: u64,
}

impl Message {
    /// Create a new message
    #[must_use]
    pub const fn new(sender_id: u64, content: Vec<ConceptId>, tick: u64) -> Self {
        Self {
            sender_id,
            content,
            tick_sent: tick,
        }
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Process message reception (language ON)
/// Per §8.2.1 lines 4726-4740: Language toggle contract
pub fn process_message_reception(
    receiver_lexicon: &mut Lexicon,
    sender_lexicon: &Lexicon,
    message: &Message,
    alignment_factor: f32,
) {
    // For each concept in the message
    for concept_id in &message.content {
        if let Some(sender_concept) = sender_lexicon.get(*concept_id) {
            // Update receiver's lexicon via alignment
            if let Some(receiver_concept) = receiver_lexicon.get_mut(*concept_id) {
                receiver_concept
                    .embedding
                    .align_toward(&sender_concept.embedding, alignment_factor);
            }

            // Track colexification patterns
            for linked_id in &sender_concept.colexifications {
                // Add colexification if not already present
                let pair = (*concept_id, *linked_id);
                let reverse_pair = (*linked_id, *concept_id);
                if !receiver_lexicon.colexification_map.contains(&pair)
                    && !receiver_lexicon.colexification_map.contains(&reverse_pair)
                {
                    receiver_lexicon.add_colexification(*concept_id, *linked_id);
                }
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute lexical entropy (Shannon H of concept embeddings)
/// Per §8.3.6: `embedding_entropy` = Shannon H of concept embeddings
#[must_use]
pub fn lexical_entropy(lexicon: &Lexicon) -> f32 {
    if lexicon.is_empty() {
        return 0.0;
    }

    // Compute entropy based on embedding value diversity across concepts
    // Collect all embedding values from all concepts
    let mut all_values: Vec<f32> = Vec::new();
    for concept in lexicon.concepts.values() {
        for &v in &concept.embedding.values {
            all_values.push(v);
        }
    }

    if all_values.is_empty() {
        return 0.0;
    }

    // Compute variance of embedding values across all concepts
    let mean = all_values.iter().sum::<f32>() / all_values.len() as f32;
    let variance =
        all_values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / all_values.len() as f32;

    // Convert variance to entropy-like measure
    // Higher variance = higher diversity = higher entropy
    // Add 1 to ensure positive result, ln(1+x) ≥ 0 for x ≥ 0
    variance.ln_1p()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_embedding_cosine_similarity() {
        let e1 = Embedding::new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let e2 = Embedding::new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert!((e1.cosine_similarity(&e2) - 1.0).abs() < 0.001);

        let e3 = Embedding::new([0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert!(e1.cosine_similarity(&e3).abs() < 0.001); // Orthogonal
    }

    #[test]
    fn test_embedding_align_toward() {
        let mut e1 = Embedding::new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let e2 = Embedding::new([0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        e1.align_toward(&e2, 0.5);

        // Should be somewhere between e1 and e2
        assert!(e1.values[0] > 0.0);
        assert!(e1.values[1] > 0.0);
    }

    #[test]
    fn test_lexicon_initialize_base() {
        let mut rng = StdRng::seed_from_u64(42);
        let lexicon = Lexicon::initialize_base(&mut rng, 0);

        // Should have 19 base concepts:
        // Trust: 4, Social: 4, Resource: 3, Emotion: 4, Action: 4
        assert_eq!(lexicon.len(), 19);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Should have trust cluster concepts
        assert!(lexicon.get(0).is_some()); // reliability
        assert!(lexicon.get(1).is_some()); // honesty

        // Should have colexification patterns
        assert!(!lexicon.colexification_map.is_empty());
    }

    #[test]
    fn test_lexicon_overlap() {
        let mut rng = StdRng::seed_from_u64(42);
        let lex1 = Lexicon::initialize_base(&mut rng, 0);
        let lex2 = Lexicon::initialize_base(&mut rng, 0);

        // Same concepts = 100% overlap
        assert!((lex1.overlap(&lex2) - 1.0).abs() < 0.001);

        // Empty lexicon = 0% overlap
        let empty = Lexicon::new();
        assert_eq!(lex1.overlap(&empty), 0.0);
    }

    #[test]
    fn test_lexicon_align_toward() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut lex1 = Lexicon::initialize_base(&mut rng, 0);
        let lex2 = Lexicon::initialize_base(&mut rng, 0);

        // Get initial similarity
        // ERR-001: Test code - unwrap() exempt for test assertions
        let initial_sim = lex1
            .get(0)
            .unwrap()
            .embedding
            .cosine_similarity(&lex2.get(0).unwrap().embedding);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Align lex1 toward lex2
        lex1.align_toward(&lex2, 0.5);

        // Similarity should increase (or stay same if already aligned)
        // ERR-001: Test code - unwrap() exempt for test assertions
        let final_sim = lex1
            .get(0)
            .unwrap()
            .embedding
            .cosine_similarity(&lex2.get(0).unwrap().embedding);
        assert!(final_sim >= initial_sim - 0.001);
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::new(1, vec![0, 10, 20], 100);
        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.content.len(), 3);
        assert_eq!(msg.tick_sent, 100);
    }

    #[test]
    fn test_process_message_reception() {
        let mut rng = StdRng::seed_from_u64(42);
        let sender_lex = Lexicon::initialize_base(&mut rng, 0);
        let mut receiver_lex = Lexicon::initialize_base(&mut rng, 0);

        let msg = Message::new(1, vec![0, 1], 100);

        // Process message
        process_message_reception(&mut receiver_lex, &sender_lex, &msg, 0.1);

        // Receiver should still have concepts
        assert!(receiver_lex.get(0).is_some());
    }

    #[test]
    fn test_concept_drift_rate() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut concept = Concept::new(0, ConceptCategory::Trust, 0, &mut rng);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Initially no drift
        assert!(concept.drift_rate(0) < 0.001);

        // Modify embedding
        concept.embedding.values[0] = -concept.embedding.values[0];

        // Now there should be drift
        assert!(concept.drift_rate(100) > 0.0);
    }

    #[test]
    fn test_lexical_entropy() {
        let mut rng = StdRng::seed_from_u64(42);
        let lexicon = Lexicon::initialize_base(&mut rng, 0);

        let entropy = lexical_entropy(&lexicon);
        assert!(entropy > 0.0);

        // Empty lexicon has 0 entropy
        let empty = Lexicon::new();
        assert_eq!(lexical_entropy(&empty), 0.0);
    }

    #[test]
    fn test_trust_cluster_count() {
        let mut rng = StdRng::seed_from_u64(42);
        let lexicon = Lexicon::initialize_base(&mut rng, 0);

        // Should have 4 trust concepts
        assert_eq!(lexicon.trust_cluster_count(), 4);
    }
}
