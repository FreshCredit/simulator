//! Neural validation visualization overlays
//!
//! Provides visualization of neural dynamics and calibration results
//! for integration with the main simulation visualization.

use super::backend::{DerivedDocasParams, SimulationResult};
use super::bold::BoldSignal;
use super::calibration::CalibrationResult;
use super::NeuralValidationError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Neural visualizer
#[derive(Debug)]
pub struct NeuralVisualizer {
    config: VisualizationConfig,
}

/// Visualization configuration
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    /// Enable neural traces
    pub show_neural_traces: bool,
    /// Enable BOLD overlay
    pub show_bold_overlay: bool,
    /// Enable calibration metrics
    pub show_calibration: bool,
    /// Trace line width
    pub trace_width: f32,
    /// Color scheme
    pub color_scheme: ColorScheme,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            show_neural_traces: true,
            show_bold_overlay: true,
            show_calibration: true,
            trace_width: 2.0,
            color_scheme: ColorScheme::Default,
        }
    }
}

/// Color schemes for neural traces
#[derive(Debug, Clone, Copy)]
pub enum ColorScheme {
    /// Default DOCAS colors
    Default,
    /// Accessible colorblind-friendly
    Accessible,
    /// Monochrome for printing
    Monochrome,
    /// Dark background
    Dark,
}

/// Visualization overlay data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationOverlay {
    /// Type of overlay
    pub overlay_type: OverlayType,
    /// Temporal data (for time series)
    pub time_data: Option<TimeSeriesData>,
    /// Spatial data (for brain maps)
    pub spatial_data: Option<SpatialData>,
    /// Calibration data
    pub calibration_data: Option<CalibrationData>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Overlay type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayType {
    /// neural traces.
    NeuralTraces,
    /// bold signal.
    BoldSignal,
    /// calibration.
    Calibration,
    /// parameter heatmap.
    ParameterHeatmap,
    /// correlation matrix.
    CorrelationMatrix,
    /// validation metrics.
    ValidationMetrics,
}

/// Time series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    /// Timestamps
    pub timestamps: Vec<f32>,
    /// Traces for each neuromodulator
    pub traces: Vec<NeuralTrace>,
    /// Y-axis label
    pub y_label: String,
    /// Title
    pub title: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Single neural trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralTrace {
    /// Trace label
    pub label: String,
    /// Color (RGBA)
    pub color: [u8; 4],
    /// Values
    pub values: Vec<f32>,
    /// Line width
    pub line_width: f32,
}

/// Spatial data (for brain visualization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialData {
    /// Brain regions activated
    pub regions: Vec<BrainRegionActivation>,
    /// Slice position
    pub slice_position: f32,
    /// Slice axis
    pub slice_axis: Axis,
}

/// Brain region activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRegionActivation {
    /// Region name
    pub region: String,
    /// MNI coordinates (x, y, z)
    pub coordinates: [f32; 3],
    /// Activation magnitude (0-1)
    pub activation: f32,
    /// Associated neuromodulator
    pub neuromodulator: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Axis for brain slicing
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Axis {
    /// x.
    X,
    /// y.
    Y,
    /// z.
    Z,
}

/// Calibration visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    /// Before/after comparison
    pub before_after: BeforeAfterComparison,
    /// Parameter evolution
    pub parameter_evolution: Vec<ParameterSnapshot>,
    /// Fit quality metrics
    pub fit_quality: FitQualityVisualization,
}

/// Before/after parameter comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeforeAfterComparison {
    /// Parameters before calibration
    pub before: DerivedDocasParams,
    /// Parameters after calibration
    pub after: DerivedDocasParams,
    /// Improvement metrics
    pub improvements: HashMap<String, f32>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Parameter snapshot over iterations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSnapshot {
    /// iteration.
    pub iteration: u32,
    /// alpha.
    pub alpha: [f32; 5],
    /// beta.
    pub beta: [f32; 5],
    /// mse.
    pub mse: f32,
}

/// Fit quality visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitQualityVisualization {
    /// R² for each neuromodulator
    pub r_squared: Vec<(String, f32)>,
    /// Residual plot data
    pub residuals: Vec<(f32, f32)>,
    /// Q-Q plot data
    pub qq_data: Vec<(f32, f32)>,
}

impl NeuralVisualizer {
    /// Create new visualizer
    #[must_use]
    pub const fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }

    /// Generate neural traces overlay
    /// # Errors
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ///
    /// Returns an error if the operation fails.
    pub fn generate_neural_traces(
        &self,
        result: &SimulationResult,
    ) -> Result<VisualizationOverlay, NeuralValidationError> {
        let colors = self.get_colors();
        let labels = [
            "Dopamine",
            "Oxytocin",
            "Cortisol",
            "Adrenaline",
            "Serotonin",
        ];
        let activities = [
            &result.dopamine_activity,
            &result.oxytocin_activity,
            &result.cortisol_activity,
            &result.adrenaline_activity,
            &result.serotonin_activity,
        ];

        let traces: Vec<NeuralTrace> = labels
            .iter()
            .enumerate()
            .map(|(i, &label)| NeuralTrace {
                label: label.to_string(),
                color: colors[i],
                values: activities[i].clone(),
                line_width: self.config.trace_width,
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Ok(VisualizationOverlay {
            overlay_type: OverlayType::NeuralTraces,
            time_data: Some(TimeSeriesData {
                timestamps: result.timestamps.clone(),
                traces,
                y_label: "Activity (a.u.)".to_string(),
                title: format!("Neural Dynamics - Agent {}", result.agent_id),
            }),
            spatial_data: None,
            calibration_data: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("agent_id".to_string(), result.agent_id.to_string());
                m.insert("n_points".to_string(), result.timestamps.len().to_string());
                m
            },
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate BOLD signal overlay
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn generate_bold_overlay(
        &self,
        bold: &BoldSignal,
        agent_id: u64,
    ) -> Result<VisualizationOverlay, NeuralValidationError> {
        let color = [255, 100, 100, 255]; // Red for BOLD

        let traces = vec![NeuralTrace {
            label: "BOLD Signal".to_string(),
            color,
            values: bold.signal.clone(),
            line_width: self.config.trace_width,
        }];

        Ok(VisualizationOverlay {
            overlay_type: OverlayType::BoldSignal,
            time_data: Some(TimeSeriesData {
                timestamps: bold.timestamps.clone(),
                traces,
                y_label: "% Signal Change".to_string(),
                title: format!("BOLD Response - Agent {agent_id}"),
            }),
            spatial_data: None,
            calibration_data: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("agent_id".to_string(), agent_id.to_string());
                m.insert("tsnr".to_string(), format!("{:.2}", bold.quality.tsnr));
                m
            },
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate calibration visualization
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn generate_calibration_visualization(
        &self,
        result: &CalibrationResult,
        previous_params: Option<&DerivedDocasParams>,
    ) -> Result<VisualizationOverlay, NeuralValidationError> {
        let before = previous_params.cloned().unwrap_or_default();

        let improvements: HashMap<String, f32> = [
            (
                "alpha_fit".to_string(),
                result.fit_metrics.r_squared.iter().sum::<f32>() / 5.0,
            ),
            ("mse_improvement".to_string(), 1.0 - result.fit_metrics.mse),
            ("cv_score".to_string(), result.fit_metrics.cv_score),
        ]
        .into_iter()
        .collect();

        let calibration_data = CalibrationData {
            before_after: BeforeAfterComparison {
                before,
                after: DerivedDocasParams {
                    alpha: result.alpha,
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    beta: result.beta,
                    time_constants: result.time_constants,
                    coupling: result.coupling,
                    confidence: result.fit_metrics.cv_score,
                },
                improvements,
            },
            parameter_evolution: vec![ParameterSnapshot {
                iteration: result.iterations,
                alpha: result.alpha,
                beta: result.beta,
                mse: result.fit_metrics.mse,
            }],
            fit_quality: FitQualityVisualization {
                r_squared: result
                    .fit_metrics
                    .r_squared
                    .iter()
                    .enumerate()
                    .map(|(i, &r)| {
                        let names = ["DA", "OXT", "CORT", "ADR", "5-HT"];
                        (names[i].to_string(), r)
                    })
                    .collect(),
                residuals: vec![], // Would be populated with actual residuals
                qq_data: vec![],   // Would be populated with actual Q-Q data
            },
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Ok(VisualizationOverlay {
            overlay_type: OverlayType::Calibration,
            time_data: None,
            spatial_data: None,
            calibration_data: Some(calibration_data),
            metadata: {
                let mut m = HashMap::new();
                m.insert("converged".to_string(), result.converged.to_string());
                m.insert("iterations".to_string(), result.iterations.to_string());
                m.insert("n_agents".to_string(), result.n_agents.to_string());
                m
            },
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate brain region activation map
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn generate_brain_activation(
        &self,
        result: &SimulationResult,
    ) -> Result<VisualizationOverlay, NeuralValidationError> {
        // Map neuromodulators to brain regions
        // These are simplified approximations
        let regions = vec![
            BrainRegionActivation {
                region: "Ventral Tegmental Area".to_string(),
                coordinates: [0.0, -16.0, -10.0],
                activation: result.dopamine_activity.last().copied().unwrap_or(0.0),
                neuromodulator: "Dopamine".to_string(),
            },
            BrainRegionActivation {
                region: "Nucleus Accumbens".to_string(),
                coordinates: [6.0, 10.0, -8.0],
                activation: result.dopamine_activity.last().copied().unwrap_or(0.0) * 0.8,
                neuromodulator: "Dopamine".to_string(),
            },
            BrainRegionActivation {
                region: "Hypothalamus".to_string(),
                coordinates: [0.0, -4.0, -8.0],
                activation: result.oxytocin_activity.last().copied().unwrap_or(0.0),
                neuromodulator: "Oxytocin".to_string(),
            },
            BrainRegionActivation {
                region: "Amygdala".to_string(),
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                coordinates: [22.0, -6.0, -14.0],
                activation: result.cortisol_activity.last().copied().unwrap_or(0.0),
                neuromodulator: "Cortisol".to_string(),
            },
            BrainRegionActivation {
                region: "Locus Coeruleus".to_string(),
                coordinates: [-2.0, -36.0, -24.0],
                activation: result.adrenaline_activity.last().copied().unwrap_or(0.0),
                neuromodulator: "Adrenaline".to_string(),
            },
            BrainRegionActivation {
                region: "Raphe Nuclei".to_string(),
                coordinates: [0.0, -26.0, -20.0],
                activation: result.serotonin_activity.last().copied().unwrap_or(0.0),
                neuromodulator: "Serotonin".to_string(),
            },
        ];

        Ok(VisualizationOverlay {
            overlay_type: OverlayType::NeuralTraces,
            time_data: None,
            spatial_data: Some(SpatialData {
                regions,
                slice_position: 0.0,
                slice_axis: Axis::Z,
            }),
            calibration_data: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("view".to_string(), "axial".to_string());
                m.insert("mni_space".to_string(), "true".to_string());
                m
            },
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate parameter heatmap
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn generate_parameter_heatmap(
        &self,
        results: &[SimulationResult],
    ) -> Result<VisualizationOverlay, NeuralValidationError> {
        // Aggregate parameters across agents
        let mut alpha_matrix = vec![vec![0.0; 5]; results.len()];

        for (i, result) in results.iter().enumerate() {
            alpha_matrix[i] = result.derived_docas.alpha.to_vec();
        }

        // Convert to heatmap visualization format
        let traces: Vec<NeuralTrace> = (0..5)
            .map(|nm| {
                let values: Vec<f32> = alpha_matrix.iter().map(|row| row[nm]).collect();
                NeuralTrace {
                    label: format!("α_{nm}"),
                    color: [100 + nm as u8 * 40, 100, 200, 255],
                    values,
                    line_width: self.config.trace_width,
                }
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Ok(VisualizationOverlay {
            overlay_type: OverlayType::ParameterHeatmap,
            time_data: Some(TimeSeriesData {
                timestamps: (0..results.len()).map(|i| i as f32).collect(),
                traces,
                y_label: "Retention Rate (α)".to_string(),
                title: "DOCAS Retention Parameters Across Agents".to_string(),
            }),
            spatial_data: None,
            calibration_data: None,
            metadata: HashMap::new(),
        })
    }

    /// Get colors for neuromodulators
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    const fn get_colors(&self) -> [[u8; 4]; 5] {
        match self.config.color_scheme {
            ColorScheme::Default => [
                [255, 215, 0, 255], // Dopamine - Gold
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                [255, 105, 180, 255], // Oxytocin - Hot Pink
                [255, 69, 0, 255],    // Cortisol - Red Orange
                [255, 140, 0, 255],   // Adrenaline - Dark Orange
                [100, 149, 237, 255], // Serotonin - Cornflower Blue
            ],
            ColorScheme::Accessible => [
                [230, 159, 0, 255],  // Orange
                [86, 180, 233, 255], // Sky Blue
                [0, 158, 115, 255],  // Bluish Green
                [240, 228, 66, 255], // Yellow
                [0, 114, 178, 255],  // Blue
            ],
            ColorScheme::Monochrome => [
                [0, 0, 0, 255],
                [64, 64, 64, 255],
                [128, 128, 128, 255],
                [192, 192, 192, 255],
                [32, 32, 32, 255],
            ],
            ColorScheme::Dark => [
                [255, 255, 0, 255],   // Yellow
                [255, 0, 255, 255],   // Magenta
                [0, 255, 255, 255],   // Cyan
                [255, 128, 0, 255],   // Orange
                [128, 255, 128, 255], // Light Green
            ],
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Export overlay to JSON
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn export_to_json(
        &self,
        overlay: &VisualizationOverlay,
    ) -> Result<String, NeuralValidationError> {
        serde_json::to_string_pretty(overlay)
            .map_err(|e| NeuralValidationError::Backend(format!("JSON export failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_result() -> SimulationResult {
        SimulationResult {
            agent_id: 0,
            timestamps: (0..100).map(|i| i as f32 * 0.01).collect(),
            dopamine_activity: (0..100)
                .map(|i| (i as f32 * 0.1).sin() * 0.5 + 0.5)
                .collect(),
            oxytocin_activity: vec![0.3; 100],
            cortisol_activity: vec![0.4; 100],
            adrenaline_activity: vec![0.2; 100],
            serotonin_activity: vec![0.5; 100],
            derived_docas: DerivedDocasParams::default(),
            bold_signal: None,
            metadata: HashMap::default(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_neural_traces_generation() {
        let visualizer = NeuralVisualizer::new(VisualizationConfig::default());
        let result = create_test_result();

        let overlay = visualizer.generate_neural_traces(&result).unwrap();

        assert_eq!(overlay.overlay_type, OverlayType::NeuralTraces);
        assert!(overlay.time_data.is_some());

        let time_data = overlay.time_data.unwrap();
        assert_eq!(time_data.traces.len(), 5); // 5 neuromodulators
    }

    #[test]
    fn test_brain_activation() {
        let visualizer = NeuralVisualizer::new(VisualizationConfig::default());
        let result = create_test_result();

        let overlay = visualizer.generate_brain_activation(&result).unwrap();

        assert!(overlay.spatial_data.is_some());
        let spatial = overlay.spatial_data.unwrap();
        assert!(!spatial.regions.is_empty());
    }

    #[test]
    fn test_default_config() {
        let config = VisualizationConfig::default();
        assert!(config.trace_width > 0.0);
        assert!(config.show_neural_traces);
    }
}
