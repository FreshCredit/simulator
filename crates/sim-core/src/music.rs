//! Music module: Music as a universal trust indicator
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION music layer, not product audio features
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! Per "The Truth About Music" research paper:
//! - Music functions as a cross-cultural universal lead indicator for trust
//! - Genre preferences correlate with personality (r = 0.20-0.40)
//! - Music taste stability: test-retest r ≈ 0.85
//! - DOCAS-music bridge: Each DOCAS component modulated by music
//! - 1/f spectral coherence predicts mood stability
//!
//! Per simulation-plan.md §6.16: Music-Specific POM Targets
//! - Genre-personality correlation: r = 0.20-0.40
//! - Music taste stability: r ≈ 0.80-0.90
//! - DOCAS-music bridge: r > 0.30 per component
//! - 1/f deviation → instability: r > 0.40
//! - Synchronized performance trust: Δτ > 0.1 per performance

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::docas::DocasState;
use crate::types::{AgentId, Tick};

/// 5-dimensional TDF Music State Vector per "The Truth About Music" & DOCAS §2:
/// [BPM, tau (metric hierarchy ratio), E (spectral energy), H_s (session entropy), delta_phi (phase dynamics)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TdfMusicVector {
    /// Tempo in BPM [40.0, 220.0]
    pub tempo_bpm: f32,
    /// Metric Hierarchy Ratio: 0.0 for 3/4 (simple triple), 0.5 for 4/4 (simple duple), 1.0 for 6/8 (compound duple)
    pub metric_hierarchy_tau: f32,
    /// Normalized Spectral Energy / RMS loudness in [0.0, 1.0]
    pub spectral_energy: f32,
    /// Shannon Session Entropy in [0.0, 4.0]
    pub session_entropy: f32,
    /// Phase / Skip Dynamics in [-1.0, 1.0] (early skip < 0, full track phrase alignment > 0)
    pub phase_dynamics: f32,
}

impl TdfMusicVector {
    /// Create new TdfMusicVector with metric hierarchy tau
    #[must_use]
    pub fn with_tau(tau: f32) -> Self {
        Self {
            metric_hierarchy_tau: tau.clamp(0.0, 1.0),
            ..Default::default()
        }
    }
}

impl Default for TdfMusicVector {
    fn default() -> Self {
        Self {
            tempo_bpm: 120.0,
            metric_hierarchy_tau: 0.5,
            spectral_energy: 0.5,
            session_entropy: 1.0,
            phase_dynamics: 0.0,
        }
    }
}

/// Cadence pattern for rhythm-based group bonding
/// Tracks temporal musical preferences that enable cross-language bonding
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CadencePattern {
    /// Preferred tempo in beats per minute [40.0, 200.0]
    /// Lower = more relaxed/ambient, Higher = energetic/dance
    pub beat_interval: f32,

    /// Rhythm consistency [0.0, 1.0]
    /// Higher = more regular/predictable rhythm
    pub regularity: f32,

    /// Preferred metric hierarchy tau [0.0, 1.0] (0.0=3/4, 0.5=4/4, 1.0=6/8)
    pub metric_hierarchy_tau: f32,

    /// DOCAS component pulse patterns [D, O, C, A, S]
    /// How each neurochemical maps to rhythmic emphasis
    /// Each value in [0.0, 1.0] representing pulse strength
    pub docas_rhythm: [f32; 5],

    /// Correlation between cadence and trust changes [0.0, 1.0]
    /// Higher = stronger link between shared rhythm and trust
    pub trust_coupling: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for CadencePattern {
    fn default() -> Self {
        Self {
            beat_interval: 120.0, // Standard BPM
            regularity: 0.5,
            metric_hierarchy_tau: 0.5,
            docas_rhythm: [0.5; 5], // Equal emphasis initially
            trust_coupling: 0.3,
        }
    }
}

impl CadencePattern {
    /// Create cadence pattern from genome traits
    pub fn from_genome(genome_traits: &[f32], rng: &mut StdRng) -> Self {
        // Trait 17: Rhythmic aptitude affects regularity
        // Trait 18: Tempo preference
        let rhythmic = genome_traits.get(17).copied().unwrap_or(0.5);
        let tempo_pref = genome_traits.get(18).copied().unwrap_or(0.5);

        // Beat interval: 60-180 BPM mapped to interval
        let beat_interval = 60.0 + tempo_pref * 120.0;

        // Regularity from rhythmic aptitude
        let regularity = rhythmic;

        // DOCAS rhythm patterns derived from genome
        // Each DOCAS component has a rhythmic signature
        let docas_rhythm = [
            0.3 + rhythmic * 0.5,         // Dopamine: energetic
            0.4 + rng.gen::<f32>() * 0.4, // Oxytocin: social/sync
            0.2 + (1.0 - rhythmic) * 0.5, // Cortisol: irregular
            0.3 + rng.gen::<f32>() * 0.5, // Adrenaline: variable
            0.5 + rhythmic * 0.3,         // Serotonin: stable
        ];

        // Trust coupling: how much shared rhythm affects trust
        let trust_coupling = 0.2 + rhythmic * 0.5;
        let metric_hierarchy_tau = genome_traits.get(19).copied().unwrap_or(0.5);

        Self {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            beat_interval,
            regularity,
            metric_hierarchy_tau,
            docas_rhythm,
            trust_coupling,
        }
    }

    /// Compute similarity between two cadence patterns
    /// Returns score in [0.0, 1.0] where 1.0 = identical
    #[must_use]
    pub fn similarity(&self, other: &Self) -> f32 {
        // Tempo similarity: closer BPM = higher similarity
        // Use Gaussian-like falloff for tempo difference
        let tempo_diff = (self.beat_interval - other.beat_interval).abs();
        let tempo_sim = (-tempo_diff / 60.0).exp(); // 60 BPM diff → ~0.37 similarity

        // Regularity alignment: similar consistency preferences
        let regularity_sim = 1.0 - (self.regularity - other.regularity).abs();

        // DOCAS rhythm correlation: Pearson correlation of pulse patterns
        // If rhythms are identical, resonance is perfect (1.0)
        let docas_sim = if self.docas_rhythm == other.docas_rhythm {
            1.0
        } else {
            Self::correlation(&self.docas_rhythm, &other.docas_rhythm).max(0.0)
        };

        // Trust coupling alignment: agents with similar coupling bond better
        let coupling_sim = 1.0 - (self.trust_coupling - other.trust_coupling).abs();

        // Weighted composite score
        // Emphasize tempo (rhythm matching) and DOCAS rhythm (neurochemical resonance)
        const W_TEMPO: f32 = 0.35;
        const W_REGULARITY: f32 = 0.20;
        const W_DOCAS: f32 = 0.30;
        const W_COUPLING: f32 = 0.15;

        let similarity = W_TEMPO * tempo_sim
            + W_REGULARITY * regularity_sim
            + W_DOCAS * docas_sim
            + W_COUPLING * coupling_sim;

        similarity.clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Pearson correlation between two 5-element arrays
    fn correlation(a: &[f32; 5], b: &[f32; 5]) -> f32 {
        crate::stats::pearson_correlation_f32(a, b)
    }
}

/// Music genre types
/// Per simulation-plan.md §6.16.2: Five genre categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MusicGenre {
    /// Complex, reflective (associated with Openness)
    Classical,
    /// Rhythmic, social (associated with Extraversion)
    Dance,
    /// Emotional, intense (associated with Neuroticism)
    Melancholic,
    /// Structured, traditional (associated with Conscientiousness)
    Traditional,
    /// Calm, harmonious (associated with Agreeableness)
    Ambient,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MusicGenre {
    /// Get all genres for iteration
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Classical,
            Self::Dance,
            Self::Melancholic,
            Self::Traditional,
            Self::Ambient,
        ]
    }

    /// Get genre from preference vector (highest value)
    #[must_use]
    pub fn from_preferences(preferences: &[f32; 5]) -> Self {
        let max_idx = preferences
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(0, |(idx, _)| idx);

        match max_idx {
            0 => Self::Classical,
            1 => Self::Dance,
            2 => Self::Melancholic,
            3 => Self::Traditional,
            4 => Self::Ambient,
            _ => Self::Classical,
        }
    }

    /// Get expected personality correlation pattern for this genre
    /// Per music.md §3: Genre preferences predict Big Five traits
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Returns [Openness, Conscientiousness, Extraversion, Agreeableness, Neuroticism]
    #[must_use]
    pub const fn personality_profile(&self) -> [f32; 5] {
        match self {
            // Classical: High Openness, moderate others
            Self::Classical => [0.7, 0.5, 0.3, 0.4, 0.3],
            // Dance: High Extraversion, high Openness
            Self::Dance => [0.5, 0.3, 0.8, 0.4, 0.3],
            // Melancholic: High Neuroticism, high Openness
            Self::Melancholic => [0.6, 0.3, 0.3, 0.4, 0.7],
            // Traditional: High Conscientiousness, high Agreeableness
            Self::Traditional => [0.3, 0.7, 0.4, 0.6, 0.3],
            // Ambient: High Agreeableness, low Neuroticism
            Self::Ambient => [0.5, 0.4, 0.3, 0.7, 0.2],
        }
    }

    /// Get DOCAS modulation effect for this genre
    /// Per music.md §4: DOCAS-music bridge
    /// Returns [ΔD, ΔO, ΔC, ΔA, ΔS]
    #[must_use]
    pub const fn docas_modulation(&self) -> [f32; 5] {
        match self {
            // Classical: Increases Serotonin (stabilization), slight Dopamine
            Self::Classical => [0.05, 0.0, -0.05, 0.0, 0.08],
            // Dance: Increases Dopamine and Oxytocin (social)
            Self::Dance => [0.08, 0.06, -0.03, 0.04, 0.02],
            // Melancholic: Increases Cortisol (recognition of emotion), Adrenaline
            Self::Melancholic => [0.02, 0.0, 0.06, 0.05, -0.03],
            // Traditional: Increases Serotonin, reduces Cortisol
            Self::Traditional => [0.02, 0.02, -0.06, 0.0, 0.06],
            // Ambient: Reduces Cortisol (relaxation), increases Serotonin
            Self::Ambient => [0.03, 0.03, -0.08, -0.02, 0.07],
        }
    }

    /// Get multi-channel DOCAS modulation driven by 5D TDF Music Vector
    /// Couples metric hierarchy tau directly into Oxytocin (O), spectral energy E into Adrenaline (A),
    /// session entropy H_s into Cortisol recognition gating (C), and phase dynamics delta_phi into Dopamine (D).
    #[must_use]
    pub fn docas_modulation_tdf(&self, tdf: &TdfMusicVector) -> [f32; 5] {
        let base = self.docas_modulation();
        let delta_o = base[1] + 0.08 * tdf.metric_hierarchy_tau;
        let delta_d = base[0] + 0.05 * tdf.phase_dynamics;
        // Cortisol gating opens if session entropy H_s > 2.0 (unsettled listening session)
        let cortisol_gate = if tdf.session_entropy > 2.0 {
            0.06 * (tdf.session_entropy - 2.0).min(1.0)
        } else {
            0.0
        };
        let delta_c = base[2] + cortisol_gate;
        let delta_a = base[3] + 0.06 * (tdf.spectral_energy - 0.5);
        let delta_s = base[4] + 0.05 * (1.0 - (tdf.spectral_energy - 0.5).abs());

        [delta_d, delta_o, delta_c, delta_a, delta_s]
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Agent's music state and preferences
/// Per simulation-plan.md §6.16.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicState {
    /// Genre preference vector [Classical, Dance, Melancholic, Traditional, Ambient]
    /// Each value in [0.0, 1.0], sum need not equal 1.0 (can have multiple preferences)
    pub genre_preferences: [f32; 5],

    /// Musical engagement level [0.0, 1.0]
    /// How much agent engages with music (listening, performing)
    pub engagement: f32,

    /// 1/f spectral slope preference [-1.0, 0.0]
    /// -1.0 = white noise (unpredictable), 0.0 = pure tone (too predictable)
    /// Optimal music at ~-0.5 (pink noise regime)
    pub spectral_preference: f32,

    /// Current listening/performance history (last 100 ticks)
    pub listening_history: Vec<MusicEvent>,

    /// Synchronization skill [0.0, 1.0]
    /// Ability to coordinate timing in group performance
    pub sync_skill: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Last tick music was processed
    pub last_music_tick: Tick,

    /// Cadence pattern for rhythm-based bonding
    /// Enables cross-language group synchronization
    pub cadence: CadencePattern,

    /// Temporal DOCAS pattern history for cadence analysis
    /// Records (tick, [D, O, C, A, S]) for rhythm correlation
    pub docas_listening_history: Vec<(Tick, [f32; 5])>,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            genre_preferences: [0.2; 5], // Equal preference initially
            engagement: 0.5,
            spectral_preference: -0.5, // Pink noise preference
            listening_history: Vec::with_capacity(100),
            sync_skill: 0.5,
            last_music_tick: 0,
            cadence: CadencePattern::default(),
            docas_listening_history: Vec::with_capacity(100),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MusicState {
    /// Create music state from genome traits
    /// Per simulation-plan.md: Genome influences music preferences
    pub fn from_genome(genome_traits: &[f32], rng: &mut StdRng) -> Self {
        // Trait 16: Musical openness (affects genre diversity)
        // Trait 17: Rhythmic aptitude (affects sync skill)
        let openness = genome_traits.get(16).copied().unwrap_or(0.5);
        let rhythmic = genome_traits.get(17).copied().unwrap_or(0.5);

        // Generate genre preferences based on openness
        // High openness = more diverse preferences
        let diversity = 0.2 + openness * 0.6; // [0.2, 0.8]
        let base_pref = 0.5 - diversity * 0.3;

        let mut preferences = [0.0; 5];
        for pref in &mut preferences {
            // Base preference plus random variation modulated by openness
            let variation = (rng.gen::<f32>() - 0.5) * diversity;
            *pref = (base_pref + variation).clamp(0.0, 1.0);
        }

        // Normalize so max preference is prominent
        let max_pref = preferences.iter().copied().fold(0.0, f32::max);
        if max_pref > 0.0 {
            for pref in &mut preferences {
                *pref = (*pref / max_pref).clamp(0.0, 1.0);
            }
        }

        // Initialize cadence from genome
        let cadence = CadencePattern::from_genome(genome_traits, rng);

        Self {
            genre_preferences: preferences,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            engagement: 0.3 + openness * 0.5,
            spectral_preference: -0.7 + rng.gen::<f32>() * 0.4, // [-0.7, -0.3]
            listening_history: Vec::with_capacity(100),
            sync_skill: rhythmic,
            last_music_tick: 0,
            cadence,
            docas_listening_history: Vec::with_capacity(100),
        }
    }

    /// Get dominant genre (highest preference)
    #[must_use]
    pub fn dominant_genre(&self) -> MusicGenre {
        MusicGenre::from_preferences(&self.genre_preferences)
    }

    /// Compute 1/f spectral slope from listening history
    /// Per music.md §6: 1/f coherence predicts mood stability
    #[must_use]
    pub fn compute_spectral_slope(&self) -> f32 {
        if self.listening_history.len() < 10 {
            return self.spectral_preference;
        }

        // Simplified: average of recent spectral preferences
        let recent: Vec<f32> = self
            .listening_history
            .iter()
            .rev()
            .take(50)
            .map(|e| e.spectral_character)
            .collect();

        if recent.is_empty() {
            return self.spectral_preference;
        }

        recent.iter().sum::<f32>() / recent.len() as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if listening history shows instability (deviation from 1/f)
    /// Per MUSIC-C5: 1/f deviation predicts instability
    #[must_use]
    pub fn spectral_instability(&self) -> f32 {
        let slope = self.compute_spectral_slope();
        // Optimal at -0.5, deviation increases instability
        (slope + 0.5).abs()
    }

    /// Update music state with new listening event
    pub fn record_listening(&mut self, event: MusicEvent, current_tick: Tick) {
        if event.genre < self.genre_preferences.len() {
            self.genre_preferences[event.genre] =
                (self.genre_preferences[event.genre] * 0.98 + 0.02).clamp(0.0, 1.0);
        }
        self.listening_history.push(event);

        // Keep only last 100 events
        if self.listening_history.len() > 100 {
            self.listening_history.remove(0);
        }

        self.last_music_tick = current_tick;
    }

    /// Record DOCAS state for cadence analysis
    pub fn record_docas_listening(&mut self, tick: Tick, docas_levels: [f32; 5]) {
        self.docas_listening_history.push((tick, docas_levels));
        self.last_music_tick = tick;

        // Keep only last 100 records
        if self.docas_listening_history.len() > 100 {
            self.docas_listening_history.remove(0);
        }
    }

    /// Compute cadence similarity with another music state
    #[must_use]
    pub fn cadence_similarity(&self, other: &Self) -> f32 {
        let base_sim = self.cadence.similarity(&other.cadence);
        let genre_sim = 1.0 - (self.genre_preferences[0] - other.genre_preferences[0]).abs() * 0.2;
        (base_sim * genre_sim).clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute test-retest stability correlation
    /// Per MUSIC-C3: Music taste stability r ≈ 0.85
    /// Simplified: correlation between early and recent preferences
    #[must_use]
    pub fn compute_stability(&self) -> f32 {
        if self.listening_history.len() < 20 {
            return 0.85; // Default stability
        }

        // Compare first half vs second half genre distributions
        let mid = self.listening_history.len() / 2;
        let first_half = &self.listening_history[..mid];
        let second_half = &self.listening_history[mid..];

        let first_dist = Self::genre_distribution(first_half);
        let second_dist = Self::genre_distribution(second_half);

        // Pearson correlation between distributions
        Self::correlation(&first_dist, &second_dist)
    }

    /// Helper: Compute genre distribution from events
    fn genre_distribution(events: &[MusicEvent]) -> [f32; 5] {
        let mut counts = [0.0; 5];
        for event in events {
            counts[event.genre] += 1.0;
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let total: f32 = counts.iter().sum();
        if total > 0.0 {
            for count in &mut counts {
                *count /= total;
            }
        }
        counts
    }

    /// Helper: Pearson correlation between two 5-element arrays
    fn correlation(a: &[f32; 5], b: &[f32; 5]) -> f32 {
        let mean_a = a.iter().sum::<f32>() / 5.0;
        let mean_b = b.iter().sum::<f32>() / 5.0;

        let mut numerator = 0.0;
        let mut denom_a = 0.0;
        let mut denom_b = 0.0;

        for i in 0..5 {
            let diff_a = a[i] - mean_a;
            let diff_b = b[i] - mean_b;
            numerator += diff_a * diff_b;
            denom_a += diff_a * diff_a;
            denom_b += diff_b * diff_b;
        }

        if denom_a <= 0.0 || denom_b <= 0.0 {
            return 1.0; // Perfect correlation if no variance
        }

        numerator / (denom_a.sqrt() * denom_b.sqrt())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Music listening/performance event
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MusicEvent {
    /// Tick when event occurred
    pub tick: Tick,
    /// Genre played/heard
    pub genre: usize, // Index into MusicGenre
    /// Spectral character [-1.0, 0.0] (1/f slope)
    pub spectral_character: f32,
    /// Whether this was group performance (vs solo listening)
    pub was_synchronized: bool,
    /// Number of participants (1 = solo)
    pub participant_count: u32,
}

/// Synchronized group performance event
/// Per music.md §5: Trust boost from joint music
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPerformance {
    /// Participants in the performance
    pub participants: Vec<AgentId>,
    /// Genre performed
    pub genre: MusicGenre,
    /// Duration in ticks
    pub duration: Tick,
    /// Tick when performance started
    pub start_tick: Tick,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Synchronization quality [0.0, 1.0]
    pub sync_quality: f32,
}

impl SyncPerformance {
    /// Compute trust boost for all participants
    /// Per simulation-plan.md: Δτ > 0.1 per performance
    #[must_use]
    pub fn compute_trust_boost(&self) -> f32 {
        // Base boost from participation
        let base_boost = 0.05;

        // Quality multiplier
        let quality_mult = self.sync_quality;

        // Duration bonus (longer = more trust)
        let duration_bonus = (self.duration as f32 / 100.0).min(0.1);

        (base_boost * quality_mult + duration_bonus).min(0.15)
    }

    /// Apply DOCAS boost to participants
    /// Per music.md §4: Synchronous performance increases plasma oxytocin
    pub fn apply_docas_boost(&self, docas_states: &mut [&mut DocasState]) {
        let trust_boost = self.compute_trust_boost();

        for docas in docas_states.iter_mut() {
            // Increase Oxytocin (social bonding)
            docas.levels[1] = (docas.levels[1] + trust_boost * 0.5).min(1.0);
            // Increase Serotonin (stabilization)
            docas.levels[4] = (docas.levels[4] + trust_boost * 0.3).min(1.0);
            // Slight Dopamine increase (reward)
            docas.levels[0] = (docas.levels[0] + trust_boost * 0.2).min(1.0);
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute genre-personality correlation
/// Per MUSIC-C2: r = 0.20-0.40
#[must_use]
pub fn compute_genre_personality_correlation(
    agents: &[(MusicState, [f32; 5])], // (music, personality)
) -> [f32; 5] {
    // For each personality trait, correlate with genre preference
    let mut correlations = [0.0; 5];

    for trait_idx in 0..5 {
        let trait_values: Vec<f32> = agents
            .iter()
            .map(|(_, personality)| personality[trait_idx])
            .collect();

        // Get dominant genre for each agent
        let genres: Vec<usize> = agents
            .iter()
            .map(|(music, _)| music.dominant_genre() as usize)
            .collect();

        // Correlate trait with genre
        correlations[trait_idx] = point_biserial_correlation(&trait_values, &genres);
    }

    correlations
}

/// Point-biserial correlation (trait continuous, genre categorical)
fn point_biserial_correlation(continuous: &[f32], categorical: &[usize]) -> f32 {
    if continuous.len() != categorical.len() || continuous.is_empty() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        return 0.0;
    }

    let n = continuous.len() as f32;
    let mean_overall = continuous.iter().sum::<f32>() / n;

    // For each genre, compute mean trait value
    let mut genre_means = [0.0; 5];
    let mut genre_counts = [0; 5];

    for (i, &cat) in categorical.iter().enumerate() {
        if cat < 5 {
            genre_means[cat] += continuous[i];
            genre_counts[cat] += 1;
        }
    }

    for g in 0..5 {
        if genre_counts[g] > 0 {
            genre_means[g] /= genre_counts[g] as f32;
        }
    }

    // Law of Total Variance: SS_between = sum_g n_g * (mean_g - mean_overall)^2
    // SS_total = sum_i (x_i - mean_overall)^2
    // Correlation ratio eta = sqrt(SS_between / SS_total) in [0.0, 1.0]
    let mut ss_between = 0.0f32;
    for g in 0..5 {
        if genre_counts[g] > 0 {
            ss_between += genre_counts[g] as f32 * (genre_means[g] - mean_overall).powi(2);
        }
    }

    let mut ss_total = 0.0f32;
    for &val in continuous {
        ss_total += (val - mean_overall).powi(2);
    }

    if ss_total <= 1e-12 {
        return 0.0;
    }

    (ss_between / ss_total).clamp(0.0, 1.0).sqrt()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Music-specific metrics for POM validation
/// Per simulation-plan.md §5320-5324
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MusicMetrics {
    /// Genre-personality correlation r values [O, C, E, A, N]
    pub genre_personality_r: [f32; 5],

    /// Music taste stability (test-retest correlation)
    pub taste_stability_r: f32,

    /// DOCAS-music bridge correlations [D, O, C, A, S]
    pub docas_music_correlation: [f32; 5],

    /// 1/f spectral slope deviation
    pub spectral_deviation: f32,

    /// Trust boost from synchronized performance (Δτ)
    pub sync_trust_boost: f32,

    /// Number of sync performances in window
    pub sync_performance_count: u32,
}

/// Falsification tests for music claims
/// Per simulation-plan.md §5910-5918
#[derive(Debug)]
pub struct MusicFalsificationTests;

impl MusicFalsificationTests {
    /// MUSIC-C1: Music more universal than language
    /// Threshold: >90% acoustic feature overlap cross-culturally
    pub fn test_c1_universality(genre_distribution: &[f32; 5]) -> bool {
        // Check if all genres have non-zero presence (universal appeal)
        let min_presence = genre_distribution.iter().copied().fold(1.0, f32::min);
        min_presence > 0.05 // At least 5% presence for each genre
    }

    /// MUSIC-C2: Genre predicts personality
    /// Threshold: r >= 0.15 for at least one trait
    #[must_use]
    pub fn test_c2_genre_prediction(correlations: &[f32; 5]) -> bool {
        correlations.iter().any(|&r| r.abs() >= 0.15)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// MUSIC-C3: Music taste stable
    /// Threshold: r >= 0.70
    #[must_use]
    pub fn test_c3_stability(stability_r: f32) -> bool {
        stability_r >= 0.70
    }

    /// MUSIC-C4: DOCAS-music bridge
    /// Threshold: >= 3 of 5 components correlate
    #[must_use]
    pub fn test_c4_docas_bridge(docas_correlations: &[f32; 5]) -> bool {
        docas_correlations
            .iter()
            .filter(|&&r| r.abs() >= 0.20)
            .count()
            >= 3
    }

    /// MUSIC-C5: 1/f deviation predicts instability
    /// Threshold: r >= 0.30
    #[must_use]
    pub fn test_c5_spectral_instability(correlation: f32) -> bool {
        correlation.abs() >= 0.30
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// MUSIC-C6: Sync performance increases trust
    /// Threshold: Δτ > 0
    #[must_use]
    pub fn test_c6_sync_trust(trust_delta: f32) -> bool {
        trust_delta > 0.0
    }
}

// ============================================================================
// MUSIC-P0-2: Spectral Analysis Extensions
// ============================================================================

/// Extended spectral analysis for MUSIC-P0-2 and MUSIC-P0-3
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpectralAnalysis {
    /// Measured 1/f spectral exponent (target: ~1.0)
    pub spectral_exponent: f32,
    /// Spectral coherence with trust dynamics [0.0, 1.0]
    pub spectral_coherence: f32,
    /// Music as leading indicator signal strength
    pub music_lead_indicator: f32,
    /// Trust time series for cross-correlation
    pub trust_history: Vec<f32>,
    /// Music engagement time series
    pub engagement_history: Vec<f32>,
}

impl SpectralAnalysis {
    /// Create new spectral analysis
    #[must_use]
    pub fn new() -> Self {
        Self {
            spectral_exponent: 1.0, // Target 1/f
            spectral_coherence: 0.0,
            music_lead_indicator: 0.0,
            trust_history: Vec::with_capacity(100),
            engagement_history: Vec::with_capacity(100),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record trust and engagement values
    pub fn record(&mut self, trust: f32, engagement: f32) {
        self.trust_history.push(trust);
        self.engagement_history.push(engagement);

        // Keep only last 100 ticks
        if self.trust_history.len() > 100 {
            self.trust_history.remove(0);
            self.engagement_history.remove(0);
        }
    }

    /// Compute spectral coherence between music and trust
    /// MUSIC-P0-2: Target r > 0.30
    pub fn compute_coherence(&mut self) -> f32 {
        if self.trust_history.len() < 20 {
            return 0.0;
        }

        // Simple cross-correlation
        let mean_trust = self.trust_history.iter().sum::<f32>() / self.trust_history.len() as f32;
        let mean_eng =
            self.engagement_history.iter().sum::<f32>() / self.engagement_history.len() as f32;

        let mut numerator = 0.0;
        let mut denom_trust = 0.0;
        let mut denom_eng = 0.0;

        for i in 0..self.trust_history.len() {
            let diff_t = self.trust_history[i] - mean_trust;
            let diff_e = self.engagement_history[i] - mean_eng;
            numerator += diff_t * diff_e;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            denom_trust += diff_t * diff_t;
            denom_eng += diff_e * diff_e;
        }

        if denom_trust <= 0.0 || denom_eng <= 0.0 {
            self.spectral_coherence = 0.0;
        } else {
            self.spectral_coherence = numerator / (denom_trust.sqrt() * denom_eng.sqrt());
        }

        self.spectral_coherence
    }

    /// Compute music as leading indicator (Granger causality simplified)
    /// MUSIC-P0-3: Test if music predicts trust changes
    /// Returns lead time (positive = music leads trust)
    pub fn compute_lead_indicator(&mut self) -> (f32, i32) {
        if self.trust_history.len() < 40 {
            return (0.0, 0);
        }

        // Try different lags
        let mut best_correlation: f32 = 0.0;
        let mut best_lag = 0;

        for lag in -5..=5 {
            let corr = self.cross_correlation(lag);
            if corr.abs() > best_correlation.abs() {
                best_correlation = corr;
                best_lag = lag;
            }
        }

        self.music_lead_indicator = best_correlation.abs();
        (best_correlation, best_lag)
    }

    /// Cross-correlation at given lag
    fn cross_correlation(&self, lag: i32) -> f32 {
        let n = self.trust_history.len() as i32;
        let len = if lag >= 0 {
            (n - lag).min(n) as usize
        } else {
            (n + lag).min(n) as usize
        };
        if len < 10 {
            return 0.0;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let (mean_t, mean_e) = self.compute_means(lag, len);
        let (num, dt, de) = self.compute_covariance(lag, len, mean_t, mean_e);

        if dt <= 0.0 || de <= 0.0 {
            0.0
        } else {
            num / (dt.sqrt() * de.sqrt())
        }
    }

    fn compute_means(&self, lag: i32, len: usize) -> (f32, f32) {
        let mut sum_trust = 0.0;
        let mut sum_eng = 0.0;
        for i in 0..len {
            let (t_idx, e_idx) = Self::lag_indices(lag, i);
            if t_idx < self.trust_history.len() && e_idx < self.engagement_history.len() {
                sum_trust += self.trust_history[t_idx];
                sum_eng += self.engagement_history[e_idx];
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            }
        }
        (sum_trust / len as f32, sum_eng / len as f32)
    }

    fn compute_covariance(
        &self,
        lag: i32,
        len: usize,
        mean_t: f32,
        mean_e: f32,
    ) -> (f32, f32, f32) {
        let mut numerator = 0.0;
        let mut denom_t = 0.0;
        let mut denom_e = 0.0;
        for i in 0..len {
            let (t_idx, e_idx) = Self::lag_indices(lag, i);
            if t_idx < self.trust_history.len() && e_idx < self.engagement_history.len() {
                let diff_t = self.trust_history[t_idx] - mean_t;
                let diff_e = self.engagement_history[e_idx] - mean_e;
                numerator += diff_t * diff_e;
                denom_t += diff_t * diff_t;
                denom_e += diff_e * diff_e;
            }
        }
        (numerator, denom_t, denom_e)
    }

    const fn lag_indices(lag: i32, i: usize) -> (usize, usize) {
        if lag >= 0 {
            (i + lag as usize, i)
        } else {
            (i, i + (-lag) as usize)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Estimate spectral exponent from time series
    /// MUSIC-P0-2: Target ~1.0 (1/f noise)
    pub fn estimate_spectral_exponent(&mut self) -> f32 {
        if self.engagement_history.len() < 20 {
            return 1.0;
        }

        // Simplified: use variance to estimate spectral characteristics
        // True 1/f has specific variance properties
        let mean =
            self.engagement_history.iter().sum::<f32>() / self.engagement_history.len() as f32;
        let variance = self
            .engagement_history
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>()
            / self.engagement_history.len() as f32;

        // Map variance to estimated exponent (simplified model)
        self.spectral_exponent = (1.0 + variance * 2.0).clamp(0.5, 2.0);
        self.spectral_exponent
    }
}

/// Population-level music spectral metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopulationMusicSpectral {
    /// Mean spectral exponent across population
    pub mean_spectral_exponent: f32,
    /// Mean spectral coherence
    pub mean_coherence: f32,
    /// Fraction with significant lead indicator effect
    pub lead_indicator_fraction: f32,
    /// Average lead time (ticks)
    pub average_lead_time: f32,
}

impl PopulationMusicSpectral {
    /// Compute from individual analyses
    #[must_use]
    pub fn from_analyses(analyses: &[&SpectralAnalysis]) -> Self {
        if analyses.is_empty() {
            return Self::default();
        }

        let n = analyses.len() as f32;
        // TAG: surface=simulation owner=research-team rule=SIM-001

        let mean_spectral_exponent = analyses.iter().map(|a| a.spectral_exponent).sum::<f32>() / n;

        let mean_coherence = analyses.iter().map(|a| a.spectral_coherence).sum::<f32>() / n;

        // Count those with significant lead indicator
        let significant_lead = analyses
            .iter()
            .filter(|a| a.music_lead_indicator > 0.3)
            .count() as f32;

        Self {
            mean_spectral_exponent,
            mean_coherence,
            lead_indicator_fraction: significant_lead / n,
            average_lead_time: 5.0, // Placeholder
        }
    }

    /// Validate MUSIC-P0-2: Spectral coherence r > 0.30
    #[must_use]
    pub fn validate_spectral_coherence(&self) -> bool {
        self.mean_coherence > 0.30
    }

    /// Validate MUSIC-P0-3: Significant lead indicator
    #[must_use]
    pub fn validate_lead_indicator(&self) -> bool {
        self.lead_indicator_fraction > 0.1 // At least 10% show lead effect
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_music_genre_from_preferences() {
        let prefs = [0.1, 0.8, 0.2, 0.3, 0.4];
        let genre = MusicGenre::from_preferences(&prefs);
        assert_eq!(genre, MusicGenre::Dance); // Highest at index 1
    }

    #[test]
    fn test_music_state_from_genome() {
        let mut rng = StdRng::seed_from_u64(42);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let genome_traits = vec![0.5; 20];

        let music = MusicState::from_genome(&genome_traits, &mut rng);

        // Check preferences are valid
        for pref in &music.genre_preferences {
            assert!((0.0..=1.0).contains(pref));
        }

        // Check engagement
        assert!((0.0..=1.0).contains(&music.engagement));

        // Check sync skill
        assert!((0.0..=1.0).contains(&music.sync_skill));
    }

    #[test]
    fn test_sync_performance_trust_boost() {
        let performance = SyncPerformance {
            participants: vec![1, 2, 3],
            genre: MusicGenre::Dance,
            duration: 100,
            start_tick: 0,
            sync_quality: 0.8,
        };

        let boost = performance.compute_trust_boost();
        assert!(boost > 0.0);
        assert!(boost <= 0.15);
    }

    #[test]
    fn test_falsification_tests() {
        // C2: Genre prediction
        let correlations = [0.25, 0.10, 0.30, 0.05, 0.15];
        assert!(MusicFalsificationTests::test_c2_genre_prediction(
            &correlations
        ));

        let weak_correlations = [0.05, 0.08, 0.10, 0.03, 0.05];
        assert!(!MusicFalsificationTests::test_c2_genre_prediction(
            &weak_correlations
        ));

        // C3: Stability
        assert!(MusicFalsificationTests::test_c3_stability(0.85));
        assert!(!MusicFalsificationTests::test_c3_stability(0.60));
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // C4: DOCAS bridge
        let strong_bridge = [0.25, 0.30, 0.15, 0.35, 0.20];
        assert!(MusicFalsificationTests::test_c4_docas_bridge(
            &strong_bridge
        ));

        let weak_bridge = [0.10, 0.15, 0.05, 0.08, 0.12];
        assert!(!MusicFalsificationTests::test_c4_docas_bridge(&weak_bridge));

        // C6: Sync trust
        assert!(MusicFalsificationTests::test_c6_sync_trust(0.12));
        assert!(!MusicFalsificationTests::test_c6_sync_trust(-0.05));
        assert!(!MusicFalsificationTests::test_c6_sync_trust(0.0));
    }

    #[test]
    fn test_music_stability_computation() {
        let mut music = MusicState::default();

        // Add some listening history
        for i in 0..50 {
            music.record_listening(
                MusicEvent {
                    tick: i as Tick,
                    genre: i % 5,
                    spectral_character: -0.5,
                    was_synchronized: false,
                    participant_count: 1,
                },
                i as Tick,
            );
        }

        let stability = music.compute_stability();
        assert!((-1.0..=1.0).contains(&stability));
    }

    #[test]
    fn test_spectral_analysis_coherence() {
        let mut analysis = SpectralAnalysis::new();

        // Generate correlated data
        for i in 0..50 {
            let base = (i as f32 / 50.0) * 2.0 - 1.0;
            analysis.record(0.5 + base * 0.2, 0.5 + base * 0.15);
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let coherence = analysis.compute_coherence();
        assert!(
            coherence.abs() > 0.5,
            "Coherence should be high for correlated data: {coherence}"
        );
    }

    #[test]
    fn test_spectral_analysis_lead_indicator() {
        let mut analysis = SpectralAnalysis::new();

        // Generate data where music leads trust by 2 ticks
        for i in 0..50 {
            let engagement = if i > 2 { (i as f32 / 50.0).sin() } else { 0.5 };
            let trust = if i > 4 {
                ((i - 2) as f32 / 50.0).sin()
            } else {
                0.5
            };
            analysis.record(0.5 + trust * 0.2, 0.5 + engagement * 0.2);
        }

        let (_correlation, lag) = analysis.compute_lead_indicator();
        assert!(lag > 0, "Music should lead trust: lag={lag}");
    }

    #[test]
    fn test_spectral_exponent_estimation() {
        let mut analysis = SpectralAnalysis::new();

        // Generate 1/f-like noise
        for i in 0..100 {
            let value = (i as f32 / 100.0).sin() + rand::random::<f32>() * 0.1;
            analysis.record(0.5, 0.5 + value * 0.3);
        }

        let exponent = analysis.estimate_spectral_exponent();
        assert!(
            (0.5..=2.0).contains(&exponent),
            "Exponent should be reasonable: {exponent}"
        );
    }

    #[test]
    fn test_population_spectral_metrics() {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut analyses = Vec::new();

        for _ in 0..10 {
            let mut a = SpectralAnalysis::new();
            a.spectral_exponent = 1.0 + rand::random::<f32>() * 0.5;
            a.spectral_coherence = 0.4 + rand::random::<f32>() * 0.3;
            a.music_lead_indicator = 0.35;
            analyses.push(a);
        }

        let refs: Vec<&SpectralAnalysis> = analyses.iter().collect();
        let pop = PopulationMusicSpectral::from_analyses(&refs);

        assert!(pop.mean_spectral_exponent > 0.5);
        assert!(pop.mean_coherence > 0.3);
        assert!(pop.validate_spectral_coherence());
    }

    // =========================================================================
    // Cadence Pattern Tests (Cross-Language Bonding)
    // =========================================================================

    #[test]
    fn test_cadence_pattern_default() {
        let cadence = CadencePattern::default();

        assert!(cadence.beat_interval >= 40.0 && cadence.beat_interval <= 200.0);
        assert!(cadence.regularity >= 0.0 && cadence.regularity <= 1.0);
        assert_eq!(cadence.docas_rhythm.len(), 5);
        assert!(cadence.trust_coupling >= 0.0 && cadence.trust_coupling <= 1.0);
    }

    #[test]
    fn test_cadence_pattern_from_genome() {
        let mut rng = StdRng::seed_from_u64(42);
        let genome_traits = vec![0.5; 20];

        let cadence = CadencePattern::from_genome(&genome_traits, &mut rng);

        assert!(cadence.beat_interval >= 60.0 && cadence.beat_interval <= 180.0);
        assert!(cadence.regularity >= 0.0 && cadence.regularity <= 1.0);

        // All DOCAS rhythm values should be valid
        for &val in &cadence.docas_rhythm {
            assert!((0.0..=1.0).contains(&val));
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        assert!(cadence.trust_coupling >= 0.0 && cadence.trust_coupling <= 1.0);
    }

    #[test]
    fn test_cadence_similarity_identical() {
        let cadence1 = CadencePattern::default();
        let cadence2 = CadencePattern::default();

        let sim = cadence1.similarity(&cadence2);
        assert!(
            sim > 0.99,
            "Identical cadences should have similarity ~1.0: {sim}"
        );
    }

    #[test]
    fn test_cadence_similarity_different() {
        let cadence1 = CadencePattern {
            beat_interval: 60.0, // Slow
            regularity: 0.9,     // Very regular
            metric_hierarchy_tau: 0.0,
            docas_rhythm: [0.9, 0.9, 0.1, 0.1, 0.9],
            trust_coupling: 0.8,
        };

        let cadence2 = CadencePattern {
            beat_interval: 180.0, // Fast
            regularity: 0.2,      // Irregular
            metric_hierarchy_tau: 1.0,
            docas_rhythm: [0.2, 0.2, 0.8, 0.8, 0.2],
            trust_coupling: 0.2,
        };

        let sim = cadence1.similarity(&cadence2);
        assert!(
            sim < 0.5,
            "Very different cadences should have low similarity: {sim}"
        );
    }

    #[test]
    fn test_cadence_similarity_range() {
        let cadence1 = CadencePattern::default();
        let cadence2 = CadencePattern::default();

        let sim = cadence1.similarity(&cadence2);
        assert!(
            (0.0..=1.0).contains(&sim),
            // TAG: surface=simulation owner=research-team rule=SIM-001
            "Similarity should be in [0, 1]: {sim}"
        );
    }

    #[test]
    fn test_music_state_cadence_similarity() {
        let music1 = MusicState::default();
        let music2 = MusicState::default();

        let sim = music1.cadence_similarity(&music2);
        assert!(
            (0.0..=1.0).contains(&sim),
            "Cadence similarity should be in [0, 1]: {sim}"
        );
    }

    #[test]
    fn test_music_state_record_docas_listening() {
        let mut music = MusicState::default();

        assert!(music.docas_listening_history.is_empty());

        music.record_docas_listening(10, [0.5, 0.6, 0.7, 0.8, 0.9]);

        assert_eq!(music.docas_listening_history.len(), 1);
        assert_eq!(music.docas_listening_history[0].0, 10);
        assert_eq!(
            music.docas_listening_history[0].1,
            [0.5, 0.6, 0.7, 0.8, 0.9]
        );
    }

    #[test]
    fn test_music_state_docas_history_capacity() {
        let mut music = MusicState::default();

        // Add more than 100 entries
        for i in 0..150 {
            music.record_docas_listening(i as Tick, [0.5; 5]);
        }

        // Should be capped at 100
        assert_eq!(music.docas_listening_history.len(), 100);

        // Oldest should have been removed (started at 0, now starts at 50)
        assert_eq!(music.docas_listening_history[0].0, 50);
    }

    #[test]
    fn test_point_biserial_correlation_scale_invariant_and_bounded() {
        let continuous = vec![10.0, 12.0, 20.0, 22.0, 30.0, 32.0];
        let categorical = vec![0, 0, 1, 1, 2, 2];

        let r1 = point_biserial_correlation(&continuous, &categorical);
        assert!(
            r1 > 0.9,
            "Strong categorical clustering should have r > 0.9: got {r1}"
        );
        assert!(r1 <= 1.0, "Correlation cannot exceed 1.0: got {r1}");

        // Scale invariance: scaling continuous variable by 1000.0 must yield identical correlation
        let scaled_continuous: Vec<f32> = continuous.iter().map(|&x| x * 1000.0).collect();
        let r2 = point_biserial_correlation(&scaled_continuous, &categorical);
        assert!(
            (r1 - r2).abs() < 1e-5,
            "Correlation ratio must be scale-invariant: r1={r1}, r2={r2}"
        );
    }
}
