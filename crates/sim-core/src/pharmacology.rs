//! Pharmacology Module: Drug Interventions on DOCAS
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION pharmacology, not medical advice
//!
//! Per EMOT-P0-3 and PDE4B-P0-2: Pharmacological interventions for emotion and PDE4B studies
//!
//! Simulates effects of:
//! - Oxytocin (increases O component)
//! - Lorazepam (decreases C, A components)
//! - Roflumilast (PDE4B inhibitor, affects D component)

use serde::{Deserialize, Serialize};

use crate::docas::DocasState;
use crate::types::Tick;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Type of pharmacological intervention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrugType {
    /// Oxytocin - increases social bonding (O component)
    Oxytocin,
    /// Lorazepam - reduces anxiety (decreases C, A components)
    Lorazepam,
    /// Roflumilast - PDE4B inhibitor (increases D component)
    Roflumilast,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Pharmacological intervention
/// Per §7.6.1 Blocker 3 shared module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PharmIntervention {
    /// Name of drug
    pub name: String,
    /// Drug type
    pub drug_type: DrugType,
    /// Target DOCAS component(s): 0=D, 1=O, 2=C, 3=A, 4=S
    pub target_components: Vec<usize>,
    /// Effect magnitude per component
    pub effect_magnitudes: Vec<f32>,
    /// Duration in ticks
    pub duration: u64,
    /// Onset delay in ticks
    pub onset_delay: u64,
    /// Tick when intervention started
    pub start_tick: Tick,
    /// Whether intervention is active
    pub active: bool,
}

impl PharmIntervention {
    /// Create oxytocin intervention
    /// Effect: Increases oxytocin component, promoting social trust
    #[must_use]
    pub fn oxytocin(start_tick: Tick) -> Self {
        Self {
            name: "Oxytocin".to_string(),
            drug_type: DrugType::Oxytocin,
            target_components: vec![1], // O = Oxytocin
            effect_magnitudes: vec![0.3],
            duration: 24, // ~24 hours (1 day)
            onset_delay: 1,
            start_tick,
            active: true,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create lorazepam intervention
    /// Effect: Decreases cortisol and adrenaline, reducing anxiety
    #[must_use]
    pub fn lorazepam(start_tick: Tick) -> Self {
        Self {
            name: "Lorazepam".to_string(),
            drug_type: DrugType::Lorazepam,
            target_components: vec![2, 3], // C = Cortisol, A = Adrenaline
            effect_magnitudes: vec![-0.2, -0.15],
            duration: 48, // ~48 hours (2 days)
            onset_delay: 1,
            start_tick,
            active: true,
        }
    }

    /// Create roflumilast (PDE4B inhibitor) intervention
    /// Effect: Increases dopamine (simulating PDE4B inhibition)
    #[must_use]
    pub fn roflumilast(start_tick: Tick) -> Self {
        Self {
            name: "Roflumilast".to_string(),
            drug_type: DrugType::Roflumilast,
            target_components: vec![0], // D = Dopamine
            effect_magnitudes: vec![0.1],
            duration: 72, // ~72 hours (3 days)
            onset_delay: 2,
            start_tick,
            active: true,
        }
    }

    /// Check if intervention is currently active at given tick
    #[must_use]
    pub const fn is_active(&self, tick: Tick) -> bool {
        if !self.active {
            return false;
        }
        let elapsed = tick.saturating_sub(self.start_tick);
        elapsed >= self.onset_delay && elapsed < self.duration.saturating_add(self.onset_delay)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get current effect multiplier based on time
    /// Returns 0.0 if before onset, 1.0 during active period, tapering after
    #[must_use]
    pub fn effect_multiplier(&self, tick: Tick) -> f32 {
        if !self.is_active(tick) {
            return 0.0;
        }
        if self.duration == 0 {
            return 1.0;
        }

        let elapsed = tick.saturating_sub(self.start_tick.saturating_add(self.onset_delay));

        // Linear tapering in last 20% of duration
        let active_duration = self.duration as f32;
        let progress = (elapsed as f32 / active_duration).clamp(0.0, 1.0);

        if progress > 0.8 {
            // Taper from 1.0 to 0.2
            (1.0 - (progress - 0.8) * 4.0).clamp(0.2, 1.0)
        } else {
            1.0
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply intervention effects to DOCAS state
    pub fn apply(&self, docas: &mut DocasState, tick: Tick) {
        let multiplier = self.effect_multiplier(tick);

        for (i, component) in self.target_components.iter().enumerate() {
            if *component < docas.levels.len() && i < self.effect_magnitudes.len() {
                let effect = self.effect_magnitudes[i] * multiplier;
                docas.levels[*component] = (docas.levels[*component] + effect).clamp(0.0, 1.0);
            }
        }
    }
}

/// Agent's pharmacological state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PharmState {
    /// Active interventions
    pub interventions: Vec<PharmIntervention>,
    /// History of interventions
    pub intervention_history: Vec<(Tick, DrugType)>,
}

impl PharmState {
    /// Create new pharmacological state
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an intervention
    pub fn add_intervention(&mut self, intervention: PharmIntervention) {
        self.interventions.push(intervention);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update pharmacological state (apply active interventions)
    pub fn update(&mut self, docas: &mut DocasState, tick: Tick) {
        // Apply all active interventions
        for intervention in &self.interventions {
            if intervention.is_active(tick) {
                intervention.apply(docas, tick);
            }
        }

        // Remove expired interventions
        self.interventions.retain(|i| {
            let elapsed = tick.saturating_sub(i.start_tick);
            elapsed < i.duration + i.onset_delay + 1
        });
    }

    /// Check if any intervention of given type is active
    #[must_use]
    pub fn has_active(&self, drug_type: DrugType, tick: Tick) -> bool {
        self.interventions
            .iter()
            .any(|i| i.drug_type == drug_type && i.is_active(tick))
    }

    /// Get count of active interventions
    #[must_use]
    pub fn active_count(&self, tick: Tick) -> usize {
        self.interventions
            .iter()
            .filter(|i| i.is_active(tick))
            .count()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Population-level pharmacology statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopulationPharmacology {
    /// Number of agents with active oxytocin
    pub oxytocin_count: usize,
    /// Number of agents with active lorazepam
    pub lorazepam_count: usize,
    /// Number of agents with active roflumilast
    pub roflumilast_count: usize,
    /// Mean DOCAS change from interventions
    pub mean_docas_delta: [f32; 5],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oxytocin_intervention() {
        let mut docas = DocasState::default();
        let intervention = PharmIntervention::oxytocin(0);

        assert_eq!(intervention.drug_type, DrugType::Oxytocin);
        assert!(intervention.is_active(1));

        let initial_o = docas.oxytocin();
        intervention.apply(&mut docas, 1);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        assert!(docas.oxytocin() > initial_o);
    }

    #[test]
    fn test_lorazepam_intervention() {
        let mut docas = DocasState::default();
        let intervention = PharmIntervention::lorazepam(0);

        assert_eq!(intervention.drug_type, DrugType::Lorazepam);

        let initial_c = docas.cortisol();
        let initial_a = docas.adrenaline();

        intervention.apply(&mut docas, 1);

        assert!(docas.cortisol() < initial_c);
        assert!(docas.adrenaline() < initial_a);
    }

    #[test]
    fn test_roflumilast_intervention() {
        let mut docas = DocasState::default();
        let intervention = PharmIntervention::roflumilast(0);

        assert_eq!(intervention.drug_type, DrugType::Roflumilast);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let initial_d = docas.dopamine();
        intervention.apply(&mut docas, 2); // After onset delay

        assert!(docas.dopamine() > initial_d);
    }

    #[test]
    fn test_effect_tapering() {
        let intervention = PharmIntervention::oxytocin(0);

        // Full effect during active period
        assert_eq!(intervention.effect_multiplier(12), 1.0);

        // Effect tapers near end
        let late_multiplier = intervention.effect_multiplier(23);
        assert!(late_multiplier < 1.0 && late_multiplier > 0.0);

        // No effect after duration
        assert_eq!(intervention.effect_multiplier(30), 0.0);
    }

    #[test]
    fn test_pharm_state() {
        let mut state = PharmState::new();
        let intervention = PharmIntervention::oxytocin(0);

        state.add_intervention(intervention);

        assert_eq!(state.active_count(1), 1);
        assert!(state.has_active(DrugType::Oxytocin, 1));
    }
}
