//! Epigenetics module: DNA methylation for environmental programming of gene expression
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION epigenetics, not product genetics
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! Based on Meaney & Szyf (2005) research on maternal care and stress reactivity:
//! - Early life experiences cause DNA methylation changes at glucocorticoid receptor gene
//! - High maternal care → low methylation at NR3C1 (stress resilience)
//! - Low maternal care + high stress → high methylation (stress sensitization)
//! - Effects persist across lifespan and can be inherited transgenerationally
//!
//! Mechanisms:
//! - Methylation at `CpG` islands (cytosine-phosphate-guanine sites)
//! - DNMTs (DNA methyltransferases) add methyl groups
//! - TET enzymes remove methylation (demethylation)
//! - Methylation typically silences gene expression

use crate::types::{Tick, GENOME_LOCI};
use serde::{Deserialize, Serialize};

/// DNA methylation marker at specific locus
///
/// Represents methylation at a `CpG` island that affects gene expression.
/// Methylation silences gene expression (0.0 = active, 1.0 = silenced).
/// Supports transgenerational epigenetic inheritance.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MethylationMarker {
    /// Locus index (0 to GENOME_LOCI-1)
    pub locus: usize,

    /// Methylation level [0.0, 1.0]
    /// 0.0 = unmethylated (active expression)
    /// 1.0 = fully methylated (silenced)
    pub level: f32,

    /// Timestamp when methylation was established
    pub established_tick: Tick,

    /// Cause of methylation
    pub cause: MethylationCause,

    /// Persistence (how many generations it lasts)
    /// 0.0 = transient, 1.0 = permanent
    pub persistence: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MethylationMarker {
    /// Create a new methylation marker
    #[must_use]
    pub fn new(
        locus: usize,
        level: f32,
        established_tick: Tick,
        cause: MethylationCause,
        persistence: f32,
    ) -> Self {
        Self {
            locus,
            level: level.clamp(0.0, 1.0),
            established_tick,
            cause,
            persistence: persistence.clamp(0.0, 1.0),
        }
    }

    /// Decay persistence for transgenerational inheritance
    /// Returns true if marker should be retained
    pub fn decay(&mut self) -> bool {
        self.persistence *= 0.7; // Decay factor
        self.persistence > 0.1 // Keep if above threshold
    }

    /// Check if this marker affects stress sensitivity (locus 38-39)
    #[must_use]
    pub const fn affects_stress_sensitivity(&self) -> bool {
        self.locus == 38 || self.locus == 39
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if this marker is inherited
    #[must_use]
    pub fn is_inherited(&self) -> bool {
        self.cause == MethylationCause::Inherited
    }
}

/// Cause of DNA methylation change
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MethylationCause {
    /// Early life stress (adverse childhood experiences)
    EarlyLifeStress,
    /// High maternal care (protective - reduces methylation)
    MaternalCare,
    /// Chronic stress (elevated cortisol over time)
    ChronicStress,
    /// Nutritional deficiency (malnutrition)
    Malnutrition,
    /// Environmental toxin exposure
    EnvironmentalToxin,
    /// Random drift (spontaneous methylation change)
    Spontaneous,
    /// Inherited from parent (transgenerational epigenetics)
    Inherited,
}

impl MethylationCause {
    /// Get display name for the cause
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::EarlyLifeStress => "Early Life Stress",
            Self::MaternalCare => "Maternal Care",
            Self::ChronicStress => "Chronic Stress",
            Self::Malnutrition => "Malnutrition",
            Self::EnvironmentalToxin => "Environmental Toxin",
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::Spontaneous => "Spontaneous",
            Self::Inherited => "Inherited",
        }
    }
}

/// Complete epigenetic state of an agent
///
/// Tracks DNA methylation patterns across the genome with support for
/// early life programming, stress responses, and transgenerational inheritance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Epigenome {
    /// Methylation markers across genome (only stores non-zero methylation)
    pub methylation: Vec<MethylationMarker>,

    /// Global methylation level (average across all marked loci)
    pub global_methylation: f32,

    /// Epigenetic age (accelerated by stress)
    /// Can differ from chronological age
    pub epigenetic_age: f32,

    /// Transgenerational inheritance counter
    /// Tracks how many generations of inheritance
    pub inheritance_generations: u32,

    /// Early life programming applied
    pub early_life_programmed: bool,

    /// Maternal care level received [0.0, 1.0]
    pub maternal_care_received: f32,

    /// Cumulative stress exposure
    pub cumulative_stress: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for Epigenome {
    fn default() -> Self {
        Self::new()
    }
}

impl Epigenome {
    /// Create new empty epigenome
    #[must_use]
    pub const fn new() -> Self {
        Self {
            methylation: Vec::new(),
            global_methylation: 0.0,
            epigenetic_age: 0.0,
            inheritance_generations: 0,
            early_life_programmed: false,
            maternal_care_received: 0.5, // Neutral default
            cumulative_stress: 0.0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply early life experience (Meaney & Szyf model)
    ///
    /// Based on research showing:
    /// - High maternal care → protective methylation patterns (low NR3C1 methylation)
    /// - Low maternal care + high stress → stress-sensitizing methylation
    ///
    /// # Arguments
    /// * `maternal_care` - Level of maternal care [0, 1], higher = better care
    /// * `stress_level` - Stress level during early life [0, 1]
    /// * `tick` - Current simulation tick
    pub fn apply_early_life_experience(
        &mut self,
        maternal_care: f32, // [0, 1], higher = better care
        stress_level: f32,  // [0, 1], higher = more stress
        tick: Tick,
    ) {
        if self.early_life_programmed {
            // Early life programming can only happen once
            return;
        }

        self.maternal_care_received = maternal_care.clamp(0.0, 1.0);
        self.cumulative_stress = stress_level.clamp(0.0, 1.0);

        // Meaney & Szyf model:
        // High maternal care → low methylation at glucocorticoid receptor (stress resilience)
        // Low care + high stress → high methylation (stress sensitivity)

        let care_effect = (maternal_care - 0.5) * 0.3; // [-0.15, +0.15]
        let stress_effect = (stress_level - 0.3) * 0.4; // [-0.12, +0.28]

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Glucocorticoid receptor gene (NR3C1 proxy - locus 38 for stress sensitivity)
        // Higher methylation = silenced GR = higher stress sensitivity
        let gr_methylation = (0.5 - care_effect + stress_effect).clamp(0.0, 1.0);

        let cause = if stress_level > 0.5 {
            MethylationCause::EarlyLifeStress
        } else {
            MethylationCause::MaternalCare
        };

        self.add_or_update_marker(MethylationMarker::new(
            38, // stress_sensitivity locus (NR3C1 proxy)
            gr_methylation,
            tick,
            cause,
            0.8, // Lasts ~4 generations with 0.7 decay
        ));

        // Also affect locus 39 (second stress sensitivity locus, diploid)
        let gr_methylation_39 = (0.5 - care_effect * 0.8 + stress_effect * 0.9).clamp(0.0, 1.0);

        self.add_or_update_marker(MethylationMarker::new(
            39, // second stress sensitivity locus
            gr_methylation_39,
            tick,
            cause,
            0.75, // Slightly lower persistence
        ));

        self.early_life_programmed = true;
        self.update_global_methylation();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Chronic stress causes cumulative methylation
    ///
    /// Prolonged elevated cortisol leads to:
    /// - Increased methylation at stress-related genes
    /// - Accelerated epigenetic aging
    ///
    /// # Arguments
    /// * `cortisol` - Current cortisol level [0, 1]
    /// * `tick` - Current simulation tick
    pub fn apply_chronic_stress(&mut self, cortisol: f32, tick: Tick) {
        let cortisol = cortisol.clamp(0.0, 1.0);

        // Update cumulative stress (exponential moving average)
        self.cumulative_stress = 0.95 * self.cumulative_stress + 0.05 * cortisol;

        // Chronic stress (cortisol > 0.6) causes methylation changes
        if cortisol > 0.6 {
            // Add stress-related methylation at stress sensitivity loci
            let stress_methylation = ((cortisol - 0.6) * 1.25).clamp(0.0, 0.5); // [0, 0.5]

            // Update locus 38 if new stress level is higher
            if let Some(existing) = self.methylation.iter_mut().find(|m| m.locus == 38) {
                if stress_methylation > existing.level * 0.5 {
                    existing.level = (existing.level + stress_methylation * 0.1).min(1.0);
                    existing.cause = MethylationCause::ChronicStress;
                }
            } else {
                self.add_or_update_marker(MethylationMarker::new(
                    38,
                    stress_methylation,
                    tick,
                    MethylationCause::ChronicStress,
                    0.6,
                ));
            }
        }

        // Epigenetic age acceleration by stress
        // Chronic stress accelerates biological aging
        let stress_acceleration = cortisol * 0.1;
        self.epigenetic_age += 1.0 + stress_acceleration;

        self.update_global_methylation();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Inherit methylation from parents (transgenerational epigenetics)
    ///
    /// Based on research showing epigenetic marks can persist 2-4 generations:
    /// - Some marks reset during gametogenesis (reprogramming)
    /// - Others persist (imprinted genes, some environmental responses)
    /// - Persistence depends on mark type and environmental context
    ///
    /// # Arguments
    /// * `parent1` - First parent's epigenome
    /// * `parent2` - Second parent's epigenome
    pub fn inherit_from_parents(&mut self, parent1: &Self, parent2: &Self) {
        let mut inherited = Vec::new();

        // Inherit from parent 1
        for marker in &parent1.methylation {
            // Only persistent marks are inherited (persistence > 0.5)
            if marker.persistence > 0.5 {
                let mut inherited_marker = *marker;
                inherited_marker.persistence *= 0.7; // Decay
                inherited_marker.cause = MethylationCause::Inherited;
                inherited.push(inherited_marker);
            }
        }

        // Inherit from parent 2 (avoid duplicates - take average if both have)
        for marker in &parent2.methylation {
            if marker.persistence > 0.5 {
                if let Some(existing) = inherited.iter_mut().find(|m| m.locus == marker.locus) {
                    // Both parents had methylation at this locus - average them
                    existing.level = (existing.level + marker.level) / 2.0;
                    existing.persistence = (existing.persistence + marker.persistence * 0.7) / 2.0;
                } else {
                    let mut inherited_marker = *marker;
                    inherited_marker.persistence *= 0.7;
                    inherited_marker.cause = MethylationCause::Inherited;
                    inherited.push(inherited_marker);
                }
            }
        }

        self.methylation = inherited;
        self.inheritance_generations = if self.methylation.is_empty() {
            0
        } else {
            parent1
                .inheritance_generations
                .max(parent2.inheritance_generations)
                + 1
        };
        self.update_global_methylation();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get effective gene expression level considering methylation
    ///
    /// Methylation typically silences gene expression:
    /// - 0% methylation = 100% expression
    /// - 100% methylation = 20% expression (basal level)
    ///
    /// # Arguments
    /// * `locus` - Genome locus to check
    /// * `base_expression` - Base expression level without methylation
    ///
    /// # Returns
    /// Effective expression level after methylation silencing
    #[must_use]
    pub fn get_expression(&self, locus: usize, base_expression: f32) -> f32 {
        if let Some(marker) = self.methylation.iter().find(|m| m.locus == locus) {
            // Methylation silences expression
            // 80% silencing at full methylation (leaves 20% basal expression)
            let silencing = marker.level * 0.8;
            base_expression * (1.0 - silencing)
        } else {
            base_expression
        }
    }

    /// Get methylation level at specific locus
    /// Returns 0.0 if no methylation marker at that locus
    #[must_use]
    pub fn get_methylation_at(&self, locus: usize) -> f32 {
        self.methylation
            .iter()
            .find(|m| m.locus == locus)
            .map_or(0.0, |m| m.level)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if agent has methylation at stress-related loci
    #[must_use]
    pub fn has_stress_methylation(&self) -> bool {
        self.methylation
            .iter()
            .any(MethylationMarker::affects_stress_sensitivity)
    }

    /// Get average methylation at stress sensitivity loci (38-39)
    #[must_use]
    pub fn stress_methylation_level(&self) -> f32 {
        let stress_markers: Vec<&MethylationMarker> = self
            .methylation
            .iter()
            .filter(|m| m.affects_stress_sensitivity())
            .collect();

        if stress_markers.is_empty() {
            0.0
        } else {
            stress_markers.iter().map(|m| m.level).sum::<f32>() / stress_markers.len() as f32
        }
    }

    /// Get count of methylation markers by cause
    #[must_use]
    pub fn count_by_cause(&self, cause: MethylationCause) -> usize {
        self.methylation.iter().filter(|m| m.cause == cause).count()
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Get epigenetic clock (age acceleration ratio)
    /// Returns `epigenetic_age` / `chronological_age`
    /// > 1.0 means accelerated aging
    #[must_use]
    pub fn epigenetic_clock(&self, chronological_age: Tick) -> f32 {
        if chronological_age == 0 {
            1.0
        } else {
            self.epigenetic_age / chronological_age as f32
        }
    }

    /// Add spontaneous methylation (random drift)
    /// Occurs naturally during cell division
    pub fn add_spontaneous_methylation(&mut self, tick: Tick, rng: &mut impl rand::Rng) {
        // Small chance of spontaneous methylation at random locus
        if rng.gen::<f32>() < 0.001 {
            // 0.1% chance per tick
            let locus = rng.gen_range(0..GENOME_LOCI);
            let level = rng.gen::<f32>() * 0.3; // Low level (0-0.3)

            self.add_or_update_marker(MethylationMarker::new(
                locus,
                level,
                tick,
                MethylationCause::Spontaneous,
                0.3, // Low persistence
            ));
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update methylation from environmental factors
    /// Called each tick to apply environmental effects
    pub fn update_environmental_methylation(
        &mut self,
        energy_ratio: f32, // [0, 1] current energy / max
        tick: Tick,
        rng: &mut impl rand::Rng,
    ) {
        // Nutritional deficiency causes methylation
        if energy_ratio < 0.3 {
            // Malnutrition effects
            let malnutrition_stress = (0.3 - energy_ratio) / 0.3; // [0, 1]

            if rng.gen::<f32>() < malnutrition_stress * 0.01 {
                // Add malnutrition-related methylation
                let locus = rng.gen_range(0..GENOME_LOCI);
                self.add_or_update_marker(MethylationMarker::new(
                    locus,
                    malnutrition_stress * 0.4,
                    tick,
                    MethylationCause::Malnutrition,
                    0.5,
                ));
            }
        }

        self.update_global_methylation();
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Advance epigenetic age by one tick
    /// Called each simulation tick
    pub fn age_tick(&mut self, stress_level: f32) {
        let stress_level = stress_level.clamp(0.0, 1.0);

        // Baseline aging + stress acceleration
        let stress_acceleration = stress_level * 0.15;
        self.epigenetic_age += 1.0 + stress_acceleration;
    }

    /// Private: Add or update a methylation marker
    fn add_or_update_marker(&mut self, marker: MethylationMarker) {
        if let Some(existing) = self
            .methylation
            .iter_mut()
            .find(|m| m.locus == marker.locus)
        {
            *existing = marker;
        } else {
            self.methylation.push(marker);
        }
        self.update_global_methylation();
    }

    /// Private: Update global methylation average
    fn update_global_methylation(&mut self) {
        if self.methylation.is_empty() {
            self.global_methylation = 0.0;
        } else {
            self.global_methylation = self.methylation.iter().map(|m| m.level).sum::<f32>()
                / self.methylation.len() as f32;
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Epigenetic statistics for population analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpigeneticStats {
    /// Average global methylation
    pub mean_global_methylation: f32,
    /// Average stress methylation
    pub mean_stress_methylation: f32,
    /// Average epigenetic age acceleration
    pub mean_age_acceleration: f32,
    /// Count of agents with inherited methylation
    pub inherited_count: usize,
    /// Count of agents programmed by early life
    pub early_life_programmed_count: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EpigeneticStats {
    /// Compute statistics from a population of epigenomes
    #[must_use]
    pub fn from_population(epigenomes: &[&Epigenome], chronological_ages: &[Tick]) -> Self {
        if epigenomes.is_empty() {
            return Self::default();
        }

        let mean_global =
            epigenomes.iter().map(|e| e.global_methylation).sum::<f32>() / epigenomes.len() as f32;

        let mean_stress = epigenomes
            .iter()
            .map(|e| e.stress_methylation_level())
            .sum::<f32>()
            / epigenomes.len() as f32;

        let mean_age_accel = epigenomes
            .iter()
            .zip(chronological_ages.iter())
            .map(|(e, age)| e.epigenetic_clock(*age))
            .sum::<f32>()
            / epigenomes.len() as f32;

        let inherited = epigenomes
            .iter()
            .filter(|e| e.inheritance_generations > 0)
            .count();
        let programmed = epigenomes
            .iter()
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .filter(|e| e.early_life_programmed)
            .count();

        Self {
            mean_global_methylation: mean_global,
            mean_stress_methylation: mean_stress,
            mean_age_acceleration: mean_age_accel,
            inherited_count: inherited,
            early_life_programmed_count: programmed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_epigenome_new() {
        let epi = Epigenome::new();
        assert!(epi.methylation.is_empty());
        assert_eq!(epi.global_methylation, 0.0);
        assert_eq!(epi.epigenetic_age, 0.0);
        assert!(!epi.early_life_programmed);
    }

    #[test]
    fn test_methylation_marker_creation() {
        let marker = MethylationMarker::new(38, 0.5, 100, MethylationCause::ChronicStress, 0.8);
        assert_eq!(marker.locus, 38);
        assert!((marker.level - 0.5).abs() < 0.001);
        assert_eq!(marker.established_tick, 100);
        assert_eq!(marker.cause, MethylationCause::ChronicStress);
        assert!((marker.persistence - 0.8).abs() < 0.001);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_high_maternal_care_reduces_stress_methylation() {
        let mut epi = Epigenome::new();

        // High maternal care (0.9) with low stress (0.1)
        epi.apply_early_life_experience(0.9, 0.1, 0);

        // Should result in LOW methylation at stress loci
        let stress_meth = epi.stress_methylation_level();
        assert!(
            stress_meth < 0.4,
            "High maternal care should reduce stress methylation, got {stress_meth}"
        );
    }

    #[test]
    fn test_low_maternal_care_increases_stress_methylation() {
        let mut epi = Epigenome::new();

        // Low maternal care (0.1) with high stress (0.9)
        epi.apply_early_life_experience(0.1, 0.9, 0);

        // Should result in HIGH methylation at stress loci
        let stress_meth = epi.stress_methylation_level();
        assert!(
            stress_meth > 0.6,
            "Low maternal care + high stress should increase methylation, got {stress_meth}"
        );
    }

    #[test]
    fn test_chronic_stress_increases_methylation() {
        let mut epi = Epigenome::new();

        // Apply chronic stress
        epi.apply_chronic_stress(0.8, 100);

        // Should have stress-related methylation
        assert!(
            epi.has_stress_methylation(),
            "Chronic stress should cause stress methylation"
        );
        assert!(epi.stress_methylation_level() > 0.0);
    }

    #[test]
    fn test_methylation_silences_expression() {
        let mut epi = Epigenome::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Add methylation at locus 10
        epi.add_or_update_marker(MethylationMarker::new(
            10,
            0.5,
            0,
            MethylationCause::ChronicStress,
            0.8,
        ));

        // Base expression of 1.0 should be reduced
        let expr = epi.get_expression(10, 1.0);
        assert!(expr < 1.0, "Methylation should silence expression");
        assert!(
            expr > 0.0,
            "But not completely silence (basal level remains)"
        );

        // Should be approximately 0.6 (1.0 * (1 - 0.5*0.8))
        assert!(
            (expr - 0.6).abs() < 0.01,
            "Expression should be ~0.6, got {expr}"
        );
    }

    #[test]
    fn test_full_methylation_max_silencing() {
        let mut epi = Epigenome::new();

        // Full methylation
        epi.add_or_update_marker(MethylationMarker::new(
            10,
            1.0,
            0,
            MethylationCause::ChronicStress,
            0.8,
        ));

        let expr = epi.get_expression(10, 1.0);
        // Max silencing is 80%, leaving 20% basal
        assert!(
            (expr - 0.2).abs() < 0.01,
            "Full methylation should leave ~20% basal expression, got {expr}"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_transgenerational_inheritance() {
        let mut parent1 = Epigenome::new();
        let mut parent2 = Epigenome::new();

        // Add persistent methylation to parents
        parent1.add_or_update_marker(MethylationMarker::new(
            38,
            0.7,
            0,
            MethylationCause::EarlyLifeStress,
            0.8,
        ));
        parent2.add_or_update_marker(MethylationMarker::new(
            39,
            0.6,
            0,
            MethylationCause::ChronicStress,
            0.9,
        ));
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Child inherits
        let mut child = Epigenome::new();
        child.inherit_from_parents(&parent1, &parent2);

        // Child should have inherited methylation
        assert!(
            !child.methylation.is_empty(),
            "Child should inherit methylation"
        );
        assert_eq!(child.inheritance_generations, 1);

        // All child's marks should be "Inherited"
        for marker in &child.methylation {
            assert_eq!(marker.cause, MethylationCause::Inherited);
        }
    }

    #[test]
    fn test_inheritance_persistence_decay() {
        let mut parent = Epigenome::new();

        // Add high-persistence methylation
        parent.add_or_update_marker(MethylationMarker::new(
            38,
            0.7,
            0,
            MethylationCause::ChronicStress,
            0.9,
        ));

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Generation 1
        let mut gen1 = Epigenome::new();
        gen1.inherit_from_parents(&parent, &parent);
        let persist_1 = gen1
            .methylation
            .iter()
            .find(|m| m.locus == 38)
            .map_or(0.0, |m| m.persistence);

        // Generation 2
        let mut gen2 = Epigenome::new();
        gen2.inherit_from_parents(&gen1, &gen1);
        let persist_2 = gen2
            .methylation
            .iter()
            .find(|m| m.locus == 38)
            .map_or(0.0, |m| m.persistence);

        // Generation 3
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut gen3 = Epigenome::new();
        gen3.inherit_from_parents(&gen2, &gen2);
        let persist_3 = gen3
            .methylation
            .iter()
            .find(|m| m.locus == 38)
            .map_or(0.0, |m| m.persistence);

        // Persistence should decay each generation (0.9 * 0.7 = 0.63, then *0.7 = 0.441, etc)
        assert!(
            persist_1 < 0.9 && persist_1 > persist_2,
            "Persistence should decay: {persist_1} should be < 0.9 and > {persist_2}"
        );
        assert!(
            persist_2 > persist_3,
            "Persistence should continue decaying: {persist_2} > {persist_3}"
        );
    }

    #[test]
    fn test_inheritance_fades_after_4_generations() {
        let mut parent = Epigenome::new();

        // Add methylation with persistence that will fade
        parent.add_or_update_marker(MethylationMarker::new(
            38,
            0.7,
            0,
            MethylationCause::ChronicStress,
            0.8,
        ));

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Simulate inheritance through generations
        let mut current = parent.clone();
        let mut generations = 0;

        for _ in 0..10 {
            let mut child = Epigenome::new();
            child.inherit_from_parents(&current, &current);

            if child.get_methylation_at(38) < 0.1 {
                break;
            }

            generations += 1;
            current = child;
        }

        // Should fade within 2-4 generations
        assert!(
            (2..=4).contains(&generations),
            "Methylation should fade after 2-4 generations, but lasted {generations}"
        );
    }

    #[test]
    fn test_epigenetic_age_acceleration() {
        let mut epi = Epigenome::new();

        // Age without stress
        for _ in 0..100 {
            epi.age_tick(0.0);
        }
        let age_no_stress = epi.epigenetic_age;

        // Reset and age with high stress
        let mut epi_stress = Epigenome::new();
        for _ in 0..100 {
            epi_stress.age_tick(1.0);
        }
        let age_high_stress = epi_stress.epigenetic_age;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // High stress should accelerate aging
        assert!(
            age_high_stress > age_no_stress,
            "Stress should accelerate epigenetic aging: {age_high_stress} vs {age_no_stress}"
        );
    }

    #[test]
    fn test_epigenetic_clock() {
        let mut epi = Epigenome::new();

        // Age 100 ticks with high stress
        for _ in 0..100 {
            epi.age_tick(1.0);
        }

        let clock = epi.epigenetic_clock(100);
        // Should be > 1.0 (accelerated)
        assert!(
            clock > 1.0,
            "Epigenetic clock should be > 1.0 under stress, got {clock}"
        );
    }

    #[test]
    fn test_early_life_programming_once_only() {
        let mut epi = Epigenome::new();

        // First programming
        epi.apply_early_life_experience(0.9, 0.1, 0);
        let meth_1 = epi.stress_methylation_level();

        // Try to reprogram (should not work)
        epi.apply_early_life_experience(0.1, 0.9, 100);
        let meth_2 = epi.stress_methylation_level();

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Should be the same
        assert!(
            (meth_1 - meth_2).abs() < 0.001,
            "Early life programming should happen only once: {meth_1} vs {meth_2}"
        );
    }

    #[test]
    fn test_spontaneous_methylation() {
        let mut epi = Epigenome::new();
        let mut rng = StdRng::seed_from_u64(42);

        // Run many ticks to trigger spontaneous methylation
        for tick in 0..10000 {
            epi.add_spontaneous_methylation(tick, &mut rng);
        }

        // Should have some spontaneous methylation
        let spontaneous_count = epi.count_by_cause(MethylationCause::Spontaneous);
        assert!(
            spontaneous_count > 0,
            "Should have some spontaneous methylation"
        );
    }

    #[test]
    fn test_marker_decay() {
        let mut marker = MethylationMarker::new(38, 0.7, 0, MethylationCause::ChronicStress, 0.8);

        // Decay once
        let retained_1 = marker.decay();
        let persist_1 = marker.persistence;

        // Decay again
        let retained_2 = marker.decay();
        let persist_2 = marker.persistence;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        assert!(persist_1 < 0.8, "Persistence should decay");
        assert!(persist_2 < persist_1, "Should continue decaying");
        assert!(
            retained_1 && retained_2,
            "Should be retained above threshold"
        );

        // Decay many times
        for _ in 0..20 {
            if !marker.decay() {
                break;
            }
        }

        // Eventually should drop below threshold
        assert!(
            marker.persistence <= 0.1,
            "Should eventually decay below threshold"
        );
    }

    #[test]
    fn test_epigenetic_stats() {
        let epi1 = Epigenome::new();
        let mut epi2 = Epigenome::new();

        epi2.apply_early_life_experience(0.2, 0.8, 0);

        let epigenomes = vec![&epi1, &epi2];
        let ages = vec![100u64, 100u64];

        let stats = EpigeneticStats::from_population(&epigenomes, &ages);

        assert!(stats.mean_stress_methylation > 0.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(stats.early_life_programmed_count, 1);
    }

    #[test]
    fn test_malnutrition_methylation() {
        let mut epi = Epigenome::new();
        let mut rng = StdRng::seed_from_u64(42);

        // Simulate severe malnutrition over many ticks
        for tick in 0..1000 {
            epi.update_environmental_methylation(0.1, tick, &mut rng);
        }

        let malnutrition_count = epi.count_by_cause(MethylationCause::Malnutrition);
        // Severe malnutrition over 1000 ticks reliably induces methylation markers
        assert!(
            malnutrition_count > 0,
            "malnutrition must induce methylation"
        );
    }

    #[test]
    fn test_inherited_markers_at_stress_loci() {
        let mut parent = Epigenome::new();

        // Parent has high stress methylation
        parent.apply_early_life_experience(0.1, 0.9, 0);

        // Child inherits
        let mut child = Epigenome::new();
        child.inherit_from_parents(&parent, &Epigenome::new());

        // Child should have stress methylation
        assert!(
            child.has_stress_methylation(),
            "Child should inherit stress methylation"
        );
        assert!(child.inheritance_generations > 0);
    }

    #[test]
    fn test_inherit_from_unmethylated_parents_resets_generations() {
        let parent1 = Epigenome::new();
        let parent2 = Epigenome::new();

        let mut child = Epigenome::new();
        child.inherit_from_parents(&parent1, &parent2);

        assert_eq!(child.inheritance_generations, 0);
        assert!(child.methylation.is_empty());
    }
}
