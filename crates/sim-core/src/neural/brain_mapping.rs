//! DOCAS-to-Brain-Region Mapping
//!
//! Maps DOCAS neuromodulator states to anatomical brain regions
//! for 3D visualization and neural-social bridge analysis.
//!
//! # Architecture
//!
//! This module provides bidirectional mapping between:
//! - DOCAS state vector [D, O, C, A, S]
//! - Brain region activations (MNI coordinates)
//! - Social behavior predictions

use super::backend::SimulationResult;
use crate::docas::DocasState;
use serde::{Deserialize, Serialize};

/// MNI coordinates (x, y, z) in millimeters
pub type MniCoordinates = [f32; 3];

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Brain region activation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRegionActivation {
    /// Region identifier
    pub region_id: String,
    /// Human-readable name
    pub name: String,
    /// MNI coordinates
    pub coordinates: MniCoordinates,
    /// Associated neuromodulator
    pub neuromodulator: NeuromodulatorType,
    /// Activation level (0.0 - 1.0)
    pub activation: f32,
    /// Functional description
    pub description: String,
    /// Associated social functions
    pub social_functions: Vec<SocialFunction>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Neuromodulator types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NeuromodulatorType {
    /// dopamine.
    Dopamine,
    /// oxytocin.
    Oxytocin,
    /// cortisol.
    Cortisol,
    /// adrenaline.
    Adrenaline,
    /// serotonin.
    Serotonin,
    /// mixed.
    Mixed,
}

impl NeuromodulatorType {
    /// Get primary DOCAS index for this neuromodulator
    #[must_use]
    pub const fn docas_index(&self) -> Option<usize> {
        match self {
            Self::Dopamine => Some(0),
            Self::Oxytocin => Some(1),
            Self::Cortisol => Some(2),
            Self::Adrenaline => Some(3),
            Self::Serotonin => Some(4),
            Self::Mixed => None,
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Get display name
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Dopamine => "Dopamine",
            Self::Oxytocin => "Oxytocin",
            Self::Cortisol => "Cortisol",
            Self::Adrenaline => "Adrenaline",
            Self::Serotonin => "Serotonin",
            Self::Mixed => "Mixed",
        }
    }

    /// Get color for visualization (RGB)
    #[must_use]
    pub const fn color(&self) -> [u8; 3] {
        match self {
            Self::Dopamine => [255, 215, 0],    // Gold
            Self::Oxytocin => [255, 105, 180],  // Hot pink
            Self::Cortisol => [255, 69, 0],     // Red orange
            Self::Adrenaline => [255, 140, 0],  // Dark orange
            Self::Serotonin => [100, 149, 237], // Cornflower blue
            Self::Mixed => [147, 112, 219],     // Medium purple
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Social functions associated with brain regions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocialFunction {
    /// Trust and cooperation
    Trust,
    /// Social bonding and attachment
    Bonding,
    /// Threat detection and avoidance
    ThreatDetection,
    /// Social hierarchy and status
    Hierarchy,
    /// Empathy and perspective-taking
    Empathy,
    /// Social learning and imitation
    SocialLearning,
    /// Conflict resolution
    ConflictResolution,
    /// Group coordination
    Coordination,
    /// Punishment and fairness
    Fairness,
    /// Communication
    Communication,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl SocialFunction {
    /// Get display name
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Trust => "Trust",
            Self::Bonding => "Bonding",
            Self::ThreatDetection => "Threat Detection",
            Self::Hierarchy => "Hierarchy",
            Self::Empathy => "Empathy",
            Self::SocialLearning => "Social Learning",
            Self::ConflictResolution => "Conflict Resolution",
            Self::Coordination => "Coordination",
            Self::Fairness => "Fairness",
            Self::Communication => "Communication",
        }
    }
}

/// Brain region database
#[derive(Debug)]
pub struct BrainRegionDatabase {
    regions: Vec<BrainRegionActivation>,
}

impl Default for BrainRegionDatabase {
    fn default() -> Self {
        Self::new()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BrainRegionDatabase {
    /// Create new brain region database with standard regions
    #[must_use]
    pub fn new() -> Self {
        let mut regions = Self::dopaminergic_regions();
        regions.extend(Self::oxytocin_regions());
        regions.extend(Self::cortisol_regions());
        regions.extend(Self::adrenaline_regions());
        regions.extend(Self::serotonin_regions());
        regions.extend(Self::cortical_regions());

        Self { regions }
    }

    /// Dopaminergic system regions (reward and motivation).
    fn dopaminergic_regions() -> Vec<BrainRegionActivation> {
        vec![
            BrainRegionActivation {
                region_id: "vta".to_string(),
                name: "Ventral Tegmental Area".to_string(),
                coordinates: [0.0, -16.0, -10.0],
                neuromodulator: NeuromodulatorType::Dopamine,
                activation: 0.0,
                description: "Primary source of dopamine neurons; encodes reward prediction errors"
                    .to_string(),
                social_functions: vec![
                    SocialFunction::Trust,
                    SocialFunction::SocialLearning,
                    SocialFunction::Fairness,
                ],
            },
            BrainRegionActivation {
                region_id: "nacc".to_string(),
                name: "Nucleus Accumbens".to_string(),
                coordinates: [6.0, 10.0, -8.0],
                neuromodulator: NeuromodulatorType::Dopamine,
                activation: 0.0,
                description: "Reward processing and reinforcement learning".to_string(),
                social_functions: vec![
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    SocialFunction::Trust,
                    SocialFunction::SocialLearning,
                    SocialFunction::Coordination,
                ],
            },
            BrainRegionActivation {
                region_id: "substantia_nigra".to_string(),
                name: "Substantia Nigra".to_string(),
                coordinates: [-4.0, -20.0, -12.0],
                neuromodulator: NeuromodulatorType::Dopamine,
                activation: 0.0,
                description: "Motor control and habit learning".to_string(),
                social_functions: vec![
                    SocialFunction::SocialLearning,
                    SocialFunction::Coordination,
                ],
            },
        ]
    }

    /// Oxytocin system regions (social bonding).
    fn oxytocin_regions() -> Vec<BrainRegionActivation> {
        vec![
            BrainRegionActivation {
                region_id: "hypothalamus".to_string(),
                name: "Hypothalamus".to_string(),
                coordinates: [0.0, -4.0, -8.0],
                neuromodulator: NeuromodulatorType::Oxytocin,
                activation: 0.0,
                description: "Oxytocin synthesis; social bonding and affiliation".to_string(),
                social_functions: vec![
                    SocialFunction::Bonding,
                    SocialFunction::Trust,
                    SocialFunction::Empathy,
                ],
                // TAG: surface=simulation owner=research-team rule=SIM-001
            },
            BrainRegionActivation {
                region_id: "pituitary".to_string(),
                name: "Pituitary Gland".to_string(),
                coordinates: [0.0, -8.0, -10.0],
                neuromodulator: NeuromodulatorType::Oxytocin,
                activation: 0.0,
                description: "Oxytocin release into bloodstream".to_string(),
                social_functions: vec![SocialFunction::Bonding, SocialFunction::Trust],
            },
        ]
    }

    /// Cortisol/stress system regions (threat and defense).
    fn cortisol_regions() -> Vec<BrainRegionActivation> {
        vec![
            BrainRegionActivation {
                region_id: "amygdala".to_string(),
                name: "Amygdala".to_string(),
                coordinates: [22.0, -6.0, -14.0],
                neuromodulator: NeuromodulatorType::Cortisol,
                activation: 0.0,
                description: "Threat detection and fear processing".to_string(),
                social_functions: vec![
                    SocialFunction::ThreatDetection,
                    SocialFunction::Trust,
                    SocialFunction::SocialLearning,
                ],
            },
            BrainRegionActivation {
                region_id: "amygdala_left".to_string(),
                name: "Amygdala (Left)".to_string(),
                coordinates: [-22.0, -6.0, -14.0],
                neuromodulator: NeuromodulatorType::Cortisol,
                activation: 0.0,
                description: "Threat detection and emotional learning".to_string(),
                social_functions: vec![
                    SocialFunction::ThreatDetection,
                    SocialFunction::SocialLearning,
                ],
            },
            BrainRegionActivation {
                region_id: "pvn".to_string(),
                name: "Paraventricular Nucleus".to_string(),
                coordinates: [0.0, -12.0, -6.0],
                neuromodulator: NeuromodulatorType::Cortisol,
                // TAG: surface=simulation owner=research-team rule=SIM-001
                activation: 0.0,
                description: "HPA axis regulation; CRH release".to_string(),
                social_functions: vec![SocialFunction::ThreatDetection, SocialFunction::Hierarchy],
            },
            BrainRegionActivation {
                region_id: "hippocampus".to_string(),
                name: "Hippocampus".to_string(),
                coordinates: [28.0, -22.0, -10.0],
                neuromodulator: NeuromodulatorType::Cortisol,
                activation: 0.0,
                description: "Context-dependent memory and stress regulation".to_string(),
                social_functions: vec![
                    SocialFunction::SocialLearning,
                    SocialFunction::ThreatDetection,
                ],
            },
            BrainRegionActivation {
                region_id: "hippocampus_left".to_string(),
                name: "Hippocampus (Left)".to_string(),
                coordinates: [-28.0, -22.0, -10.0],
                neuromodulator: NeuromodulatorType::Cortisol,
                activation: 0.0,
                description: "Context-dependent memory and stress regulation".to_string(),
                social_functions: vec![
                    SocialFunction::SocialLearning,
                    SocialFunction::ThreatDetection,
                ],
            },
        ]
    }

    /// Adrenaline system regions (arousal and vigilance).
    fn adrenaline_regions() -> Vec<BrainRegionActivation> {
        vec![BrainRegionActivation {
            region_id: "locus_coeruleus".to_string(),
            name: "Locus Coeruleus".to_string(),
            coordinates: [-2.0, -36.0, -24.0],
            neuromodulator: NeuromodulatorType::Adrenaline,
            activation: 0.0,
            description: "Primary source of noradrenaline; arousal and vigilance".to_string(),
            social_functions: vec![
                SocialFunction::ThreatDetection,
                SocialFunction::Communication,
                SocialFunction::Coordination,
            ],
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }]
    }

    /// Serotonin system regions (mood and stability).
    fn serotonin_regions() -> Vec<BrainRegionActivation> {
        vec![
            BrainRegionActivation {
                region_id: "raphe_nuclei".to_string(),
                name: "Raphe Nuclei".to_string(),
                coordinates: [0.0, -26.0, -20.0],
                neuromodulator: NeuromodulatorType::Serotonin,
                activation: 0.0,
                description: "Primary source of serotonin; mood regulation".to_string(),
                social_functions: vec![
                    SocialFunction::Hierarchy,
                    SocialFunction::ConflictResolution,
                    SocialFunction::Fairness,
                ],
            },
            BrainRegionActivation {
                region_id: "dorsal_raphe".to_string(),
                name: "Dorsal Raphe Nucleus".to_string(),
                coordinates: [0.0, -32.0, -18.0],
                neuromodulator: NeuromodulatorType::Serotonin,
                activation: 0.0,
                description: "Serotonin synthesis and stress resilience".to_string(),
                social_functions: vec![
                    SocialFunction::Hierarchy,
                    SocialFunction::ConflictResolution,
                ],
            },
        ]
    }

    /// Higher-order cortical regions (multi-modulator).
    fn cortical_regions() -> Vec<BrainRegionActivation> {
        vec![
            BrainRegionActivation {
                region_id: "prefrontal_cortex".to_string(),
                name: "Prefrontal Cortex".to_string(),
                coordinates: [0.0, 50.0, 20.0],
                neuromodulator: NeuromodulatorType::Mixed,
                activation: 0.0,
                description: "Executive function and social decision-making".to_string(),
                social_functions: vec![
                    SocialFunction::Trust,
                    SocialFunction::Fairness,
                    SocialFunction::ConflictResolution,
                    SocialFunction::SocialLearning,
                ],
                // TAG: surface=simulation owner=research-team rule=SIM-001
            },
            BrainRegionActivation {
                region_id: "anterior_cingulate".to_string(),
                name: "Anterior Cingulate Cortex".to_string(),
                coordinates: [0.0, 24.0, 16.0],
                neuromodulator: NeuromodulatorType::Mixed,
                activation: 0.0,
                description: "Conflict monitoring and social evaluation".to_string(),
                social_functions: vec![
                    SocialFunction::ConflictResolution,
                    SocialFunction::Empathy,
                    SocialFunction::Fairness,
                ],
            },
            BrainRegionActivation {
                region_id: "insula".to_string(),
                name: "Insula".to_string(),
                coordinates: [36.0, 4.0, -4.0],
                neuromodulator: NeuromodulatorType::Mixed,
                activation: 0.0,
                description: "Interoception and empathy".to_string(),
                social_functions: vec![
                    SocialFunction::Empathy,
                    SocialFunction::Trust,
                    SocialFunction::Communication,
                ],
            },
            BrainRegionActivation {
                region_id: "insula_left".to_string(),
                name: "Insula (Left)".to_string(),
                coordinates: [-36.0, 4.0, -4.0],
                neuromodulator: NeuromodulatorType::Mixed,
                activation: 0.0,
                description: "Emotional awareness and empathy".to_string(),
                social_functions: vec![SocialFunction::Empathy, SocialFunction::Communication],
            },
            BrainRegionActivation {
                region_id: "temporal_parietal".to_string(),
                name: "Temporal-Parietal Junction".to_string(),
                coordinates: [48.0, -54.0, 18.0],
                neuromodulator: NeuromodulatorType::Mixed,
                // TAG: surface=simulation owner=research-team rule=SIM-001
                activation: 0.0,
                description: "Theory of mind and perspective-taking".to_string(),
                social_functions: vec![
                    SocialFunction::Empathy,
                    SocialFunction::SocialLearning,
                    SocialFunction::Communication,
                ],
            },
            BrainRegionActivation {
                region_id: "temporal_parietal_left".to_string(),
                name: "Temporal-Parietal Junction (Left)".to_string(),
                coordinates: [-48.0, -54.0, 18.0],
                neuromodulator: NeuromodulatorType::Mixed,
                activation: 0.0,
                description: "Theory of mind and belief attribution".to_string(),
                social_functions: vec![SocialFunction::Empathy, SocialFunction::SocialLearning],
            },
            BrainRegionActivation {
                region_id: "vmPFC".to_string(),
                name: "Ventromedial Prefrontal Cortex".to_string(),
                coordinates: [0.0, 38.0, -8.0],
                neuromodulator: NeuromodulatorType::Mixed,
                activation: 0.0,
                description: "Value-based decision-making and social evaluation".to_string(),
                social_functions: vec![
                    SocialFunction::Trust,
                    SocialFunction::Fairness,
                    SocialFunction::SocialLearning,
                ],
            },
        ]
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get all regions
    #[must_use]
    pub fn regions(&self) -> &[BrainRegionActivation] {
        &self.regions
    }

    /// Get mutable regions
    pub fn regions_mut(&mut self) -> &mut [BrainRegionActivation] {
        &mut self.regions
    }

    /// Find region by ID
    #[must_use]
    pub fn get_region(&self, region_id: &str) -> Option<&BrainRegionActivation> {
        self.regions.iter().find(|r| r.region_id == region_id)
    }

    /// Find region by ID (mutable)
    pub fn get_region_mut(&mut self, region_id: &str) -> Option<&mut BrainRegionActivation> {
        self.regions.iter_mut().find(|r| r.region_id == region_id)
    }

    /// Get regions by neuromodulator type
    #[must_use]
    pub fn get_by_neuromodulator(&self, nm: NeuromodulatorType) -> Vec<&BrainRegionActivation> {
        self.regions
            .iter()
            .filter(|r| r.neuromodulator == nm)
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get regions by social function
    #[must_use]
    pub fn get_by_function(&self, func: SocialFunction) -> Vec<&BrainRegionActivation> {
        self.regions
            .iter()
            .filter(|r| r.social_functions.contains(&func))
            .collect()
    }
}

/// DOCAS to brain activation mapper
#[derive(Debug)]
pub struct DocasBrainMapper {
    database: BrainRegionDatabase,
    /// Weighting factors for each region's dependence on neuromodulators
    region_weights: std::collections::HashMap<String, [f32; 5]>,
}

impl Default for DocasBrainMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl DocasBrainMapper {
    /// Create new mapper with default weights
    #[must_use]
    pub fn new() -> Self {
        let database = BrainRegionDatabase::new();
        let mut region_weights = std::collections::HashMap::new();

        // Define how each region depends on DOCAS components
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // [Dopamine, Oxytocin, Cortisol, Adrenaline, Serotonin]
        region_weights.insert("vta".to_string(), [1.0, 0.0, 0.0, 0.0, 0.0]);
        region_weights.insert("nacc".to_string(), [0.9, 0.1, 0.0, 0.0, 0.0]);
        region_weights.insert("substantia_nigra".to_string(), [0.8, 0.0, 0.0, 0.2, 0.0]);

        region_weights.insert("hypothalamus".to_string(), [0.0, 1.0, 0.0, 0.0, 0.0]);
        region_weights.insert("pituitary".to_string(), [0.0, 1.0, 0.0, 0.0, 0.0]);

        region_weights.insert("amygdala".to_string(), [0.0, 0.0, 0.9, 0.1, 0.0]);
        region_weights.insert("amygdala_left".to_string(), [0.0, 0.0, 0.9, 0.1, 0.0]);
        region_weights.insert("pvn".to_string(), [0.0, 0.1, 0.8, 0.1, 0.0]);
        region_weights.insert("hippocampus".to_string(), [0.1, 0.0, 0.7, 0.0, 0.2]);
        region_weights.insert("hippocampus_left".to_string(), [0.1, 0.0, 0.7, 0.0, 0.2]);

        region_weights.insert("locus_coeruleus".to_string(), [0.0, 0.0, 0.2, 0.8, 0.0]);

        region_weights.insert("raphe_nuclei".to_string(), [0.0, 0.0, 0.0, 0.0, 1.0]);
        region_weights.insert("dorsal_raphe".to_string(), [0.0, 0.0, 0.0, 0.0, 0.9]);

        // Mixed regions
        region_weights.insert("prefrontal_cortex".to_string(), [0.3, 0.1, 0.1, 0.2, 0.3]);
        region_weights.insert("anterior_cingulate".to_string(), [0.2, 0.1, 0.3, 0.2, 0.2]);
        region_weights.insert("insula".to_string(), [0.1, 0.3, 0.3, 0.1, 0.2]);
        region_weights.insert("insula_left".to_string(), [0.1, 0.3, 0.3, 0.1, 0.2]);
        region_weights.insert("temporal_parietal".to_string(), [0.1, 0.3, 0.1, 0.0, 0.5]);
        region_weights.insert(
            "temporal_parietal_left".to_string(),
            [0.1, 0.3, 0.1, 0.0, 0.5],
        );
        region_weights.insert("vmPFC".to_string(), [0.3, 0.2, 0.1, 0.1, 0.3]);

        Self {
            database,
            region_weights,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Map DOCAS state to brain activations
    #[must_use]
    pub fn map_docas_to_brain(&self, docas: &DocasState) -> Vec<BrainRegionActivation> {
        let mut activations = self.database.regions.clone();

        for region in &mut activations {
            if let Some(weights) = self.region_weights.get(&region.region_id) {
                // Calculate activation as weighted sum of DOCAS levels
                let activation: f32 = docas
                    .levels
                    .iter()
                    .zip(weights.iter())
                    .map(|(level, weight)| level * weight)
                    .sum();

                region.activation = activation.clamp(0.0, 1.0);
            } else {
                // Default: use primary neuromodulator level
                if let Some(idx) = region.neuromodulator.docas_index() {
                    region.activation = docas.levels[idx];
                }
            }
        }

        activations
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Map neural simulation results to brain activations
    #[must_use]
    pub fn map_simulation_to_brain(
        &self,
        result: &SimulationResult,
        time_index: Option<usize>,
    ) -> Vec<BrainRegionActivation> {
        let idx = time_index.unwrap_or_else(|| result.timestamps.len().saturating_sub(1));
        let idx = idx.min(result.timestamps.len().saturating_sub(1));

        let mut activations = self.database.regions.clone();

        // Map simulation activity to appropriate regions
        for region in &mut activations {
            let activity = match region.neuromodulator {
                NeuromodulatorType::Dopamine => {
                    result.dopamine_activity.get(idx).copied().unwrap_or(0.0)
                }
                NeuromodulatorType::Oxytocin => {
                    result.oxytocin_activity.get(idx).copied().unwrap_or(0.0)
                }
                NeuromodulatorType::Cortisol => {
                    result.cortisol_activity.get(idx).copied().unwrap_or(0.0)
                }
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                NeuromodulatorType::Adrenaline => {
                    result.adrenaline_activity.get(idx).copied().unwrap_or(0.0)
                }
                NeuromodulatorType::Serotonin => {
                    result.serotonin_activity.get(idx).copied().unwrap_or(0.0)
                }
                NeuromodulatorType::Mixed => {
                    // Average of all activities
                    let sum = result.dopamine_activity.get(idx).copied().unwrap_or(0.0)
                        + result.oxytocin_activity.get(idx).copied().unwrap_or(0.0)
                        + result.cortisol_activity.get(idx).copied().unwrap_or(0.0)
                        + result.adrenaline_activity.get(idx).copied().unwrap_or(0.0)
                        + result.serotonin_activity.get(idx).copied().unwrap_or(0.0);
                    sum / 5.0
                }
            };

            // Normalize activity (assuming raw activity is 0-1 range)
            region.activation = activity.clamp(0.0, 1.0);
        }

        activations
    }

    /// Get database reference
    #[must_use]
    pub const fn database(&self) -> &BrainRegionDatabase {
        &self.database
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Set custom weights for a region
    pub fn set_region_weights(&mut self, region_id: &str, weights: [f32; 5]) {
        self.region_weights.insert(region_id.to_string(), weights);
    }
}

/// Social behavior prediction from brain activations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialBehaviorPrediction {
    /// Predicted trust level (0-1)
    pub trust: f32,
    /// Predicted cooperation probability (0-1)
    pub cooperation: f32,
    /// Predicted aggression/defensiveness (0-1)
    pub defensiveness: f32,
    /// Predicted social engagement (0-1)
    pub social_engagement: f32,
    /// Predicted risk-taking (0-1)
    pub risk_taking: f32,
    /// Confidence in prediction (0-1)
    pub confidence: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Neural-Social Bridge
#[derive(Debug)]
pub struct NeuralSocialBridge {
    mapper: DocasBrainMapper,
    /// Weights for behavior prediction
    behavior_weights: std::collections::HashMap<SocialFunction, [f32; 5]>,
}

impl Default for NeuralSocialBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralSocialBridge {
    /// Create new neural-social bridge
    #[must_use]
    pub fn new() -> Self {
        let mapper = DocasBrainMapper::new();
        let mut behavior_weights = std::collections::HashMap::new();

        // Define how each social function depends on DOCAS
        // [Dopamine, Oxytocin, Cortisol, Adrenaline, Serotonin]
        behavior_weights.insert(SocialFunction::Trust, [0.3, 0.5, -0.3, -0.1, 0.2]);
        behavior_weights.insert(SocialFunction::Bonding, [0.1, 0.8, -0.2, -0.1, 0.1]);
        behavior_weights.insert(SocialFunction::ThreatDetection, [0.0, -0.1, 0.6, 0.3, -0.1]);
        behavior_weights.insert(SocialFunction::Hierarchy, [0.2, -0.1, 0.1, 0.2, 0.4]);
        behavior_weights.insert(SocialFunction::Empathy, [0.1, 0.6, -0.1, 0.0, 0.2]);
        behavior_weights.insert(SocialFunction::SocialLearning, [0.4, 0.2, -0.1, 0.0, 0.2]);
        behavior_weights.insert(
            SocialFunction::ConflictResolution,
            [0.1, 0.2, -0.2, -0.1, 0.5],
        );
        behavior_weights.insert(SocialFunction::Coordination, [0.3, 0.3, -0.1, 0.2, 0.1]);
        behavior_weights.insert(SocialFunction::Fairness, [0.3, 0.2, -0.1, -0.1, 0.3]);
        behavior_weights.insert(SocialFunction::Communication, [0.2, 0.4, -0.2, 0.1, 0.2]);

        Self {
            mapper,
            behavior_weights,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Predict social behavior from DOCAS state
    #[must_use]
    pub fn predict_behavior(&self, docas: &DocasState) -> SocialBehaviorPrediction {
        // Calculate social function scores
        let mut scores: std::collections::HashMap<SocialFunction, f32> =
            std::collections::HashMap::new();

        for (func, weights) in &self.behavior_weights {
            let score: f32 = docas
                .levels
                .iter()
                .zip(weights.iter())
                .map(|(level, weight)| level * weight.max(0.0))
                .sum();
            scores.insert(*func, score.clamp(0.0, 1.0));
        }

        // Map to behavior predictions
        let trust = scores.get(&SocialFunction::Trust).copied().unwrap_or(0.5) * 0.5
            + scores.get(&SocialFunction::Bonding).copied().unwrap_or(0.5) * 0.5;

        let get_score = |f| scores.get(&f).copied().unwrap_or(0.5);
        let cooperation = get_score(SocialFunction::Coordination) * 0.4
            + get_score(SocialFunction::Fairness) * 0.4
            + trust * 0.2;

        let defensiveness = get_score(SocialFunction::ThreatDetection) * 0.7
            + (1.0 - get_score(SocialFunction::Trust)) * 0.3;

        let social_engagement = scores
            .get(&SocialFunction::Communication)
            .copied()
            .unwrap_or(0.5)
            * 0.3
            + scores.get(&SocialFunction::Empathy).copied().unwrap_or(0.5) * 0.4
            + scores.get(&SocialFunction::Bonding).copied().unwrap_or(0.5) * 0.3;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let risk_taking = scores
            .get(&SocialFunction::SocialLearning)
            .copied()
            .unwrap_or(0.5)
            * 0.4
            + (1.0
                - scores
                    .get(&SocialFunction::ThreatDetection)
                    .copied()
                    .unwrap_or(0.5))
                * 0.3
            + scores
                .get(&SocialFunction::Hierarchy)
                .copied()
                .unwrap_or(0.5)
                * 0.3;

        // Calculate confidence based on consistency of predictions
        let confidence = {
            let brain_activations = self.mapper.map_docas_to_brain(docas);
            let active_regions = brain_activations
                .iter()
                .filter(|r| r.activation > 0.3)
                .count();
            (active_regions as f32 / brain_activations.len() as f32).clamp(0.5, 1.0)
        };

        SocialBehaviorPrediction {
            trust: trust.clamp(0.0, 1.0),
            cooperation: cooperation.clamp(0.0, 1.0),
            defensiveness: defensiveness.clamp(0.0, 1.0),
            social_engagement: social_engagement.clamp(0.0, 1.0),
            risk_taking: risk_taking.clamp(0.0, 1.0),
            confidence,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get brain activations for visualization
    #[must_use]
    pub fn get_brain_activations(&self, docas: &DocasState) -> Vec<BrainRegionActivation> {
        self.mapper.map_docas_to_brain(docas)
    }

    /// Get regions by social function (for targeted analysis)
    #[must_use]
    pub fn get_regions_for_function(&self, func: SocialFunction) -> Vec<&BrainRegionActivation> {
        self.mapper.database().get_by_function(func)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_region_database() {
        let db = BrainRegionDatabase::new();
        assert!(!db.regions().is_empty());

        // Check VTA exists
        let vta = db.get_region("vta");
        assert!(vta.is_some());
        assert_eq!(vta.unwrap().neuromodulator, NeuromodulatorType::Dopamine);

        // Check dopamine regions
        let da_regions = db.get_by_neuromodulator(NeuromodulatorType::Dopamine);
        assert!(!da_regions.is_empty());

        // Check trust regions
        let trust_regions = db.get_by_function(SocialFunction::Trust);
        assert!(!trust_regions.is_empty());
    }

    #[test]
    fn test_docas_mapping() {
        let mapper = DocasBrainMapper::new();

        // Test high dopamine
        let high_da = DocasState::new([0.9, 0.5, 0.5, 0.5, 0.5]);
        let activations = mapper.map_docas_to_brain(&high_da);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let vta = activations.iter().find(|r| r.region_id == "vta").unwrap();
        assert!(vta.activation > 0.8);

        // Test high oxytocin
        let high_oxt = DocasState::new([0.5, 0.9, 0.5, 0.5, 0.5]);
        let activations = mapper.map_docas_to_brain(&high_oxt);

        let hypothalamus = activations
            .iter()
            .find(|r| r.region_id == "hypothalamus")
            .unwrap();
        assert!(hypothalamus.activation > 0.8);
    }

    #[test]
    fn test_behavior_prediction() {
        let bridge = NeuralSocialBridge::new();

        // Test high trust scenario (high oxytocin, low cortisol)
        let high_trust = DocasState::new([0.5, 0.9, 0.2, 0.3, 0.6]);
        let pred = bridge.predict_behavior(&high_trust);
        assert!(pred.trust > 0.6);
        assert!(pred.cooperation > 0.5);
        assert!(pred.defensiveness < 0.5);

        // Test high threat scenario (high cortisol, high adrenaline)
        let high_threat = DocasState::new([0.3, 0.2, 0.9, 0.8, 0.3]);
        let pred = bridge.predict_behavior(&high_threat);
        assert!(pred.defensiveness > 0.6);
        assert!(pred.trust < 0.5);
        assert!(pred.risk_taking < 0.5);

        // Test engagement scenario
        let high_engagement = DocasState::new([0.7, 0.8, 0.3, 0.5, 0.6]);
        let pred = bridge.predict_behavior(&high_engagement);
        assert!(pred.social_engagement > 0.6);
    }

    #[test]
    fn test_neuromodulator_properties() {
        assert_eq!(NeuromodulatorType::Dopamine.docas_index(), Some(0));
        assert_eq!(NeuromodulatorType::Serotonin.docas_index(), Some(4));
        assert_eq!(NeuromodulatorType::Mixed.docas_index(), None);

        assert_eq!(NeuromodulatorType::Dopamine.name(), "Dopamine");
        assert_eq!(NeuromodulatorType::Oxytocin.color(), [255, 105, 180]);
    }
}
