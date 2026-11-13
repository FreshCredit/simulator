//! Real-time fMRI Feedback Loop
//!
//! Aspirational: Closed-loop neurofeedback system connecting simulated
//! brain activity to agent behavior in real-time.

#![allow(non_camel_case_types)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Real-time fMRI acquisition parameters
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FmriParams {
    /// Repetition time (TR) in seconds
    pub tr: f32,
    /// Echo time (TE) in milliseconds
    pub te: f32,
    /// Spatial resolution (mm isotropic)
    pub voxel_size: f32,
    /// Field strength (Tesla)
    pub field_strength: f32,
    /// Number of slices
    pub n_slices: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for FmriParams {
    fn default() -> Self {
        Self {
            tr: 2.0,             // Standard 2s TR
            te: 30.0,            // 30ms TE for BOLD
            voxel_size: 3.0,     // 3mm isotropic
            field_strength: 3.0, // 3T scanner
            n_slices: 36,        // Full brain coverage
        }
    }
}

impl FmriParams {
    /// High-resolution research parameters (7T)
    #[must_use]
    pub const fn high_res() -> Self {
        Self {
            tr: 1.0,
            te: 25.0,
            voxel_size: 1.5,
            field_strength: 7.0,
            n_slices: 64,
        }
    }

    /// Real-time low-latency parameters
    #[must_use]
    pub const fn realtime() -> Self {
        Self {
            tr: 0.5, // Fast TR for feedback
            te: 30.0,
            voxel_size: 4.0, // Lower res for speed
            field_strength: 3.0,
            n_slices: 24,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Temporal resolution (Hz)
    #[must_use]
    pub fn sampling_rate(&self) -> f32 {
        1.0 / self.tr
    }
}

/// Brain region of interest for neurofeedback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedbackRegion {
    /// Prefrontal cortex - executive control
    PrefrontalCortex,
    /// Anterior cingulate - conflict monitoring
    AnteriorCingulate,
    /// Amygdala - emotional processing
    Amygdala,
    /// Nucleus accumbens - reward
    NucleusAccumbens,
    /// Insula - interoception
    Insula,
    /// Dorsal striatum - habit learning
    DorsalStriatum,
    /// Hippocampus - memory
    Hippocampus,
    /// Default mode network
    DefaultModeNetwork,
}

impl FeedbackRegion {
    /// Get MNI coordinates (center of mass)
    #[must_use]
    pub const fn mni_coordinates(&self) -> (f32, f32, f32) {
        match self {
            Self::PrefrontalCortex => (0.0, 50.0, 20.0),
            Self::AnteriorCingulate => (0.0, 30.0, 20.0),
            Self::Amygdala => (-20.0, -5.0, -15.0),
            Self::NucleusAccumbens => (-10.0, 10.0, -10.0),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::Insula => (35.0, 10.0, 0.0),
            Self::DorsalStriatum => (-12.0, 10.0, 8.0),
            Self::Hippocampus => (-25.0, -20.0, -10.0),
            Self::DefaultModeNetwork => (0.0, -50.0, 30.0),
        }
    }

    /// Get typical BOLD signal baseline
    #[must_use]
    pub const fn baseline_activation(&self) -> f32 {
        match self {
            Self::PrefrontalCortex => 100.0,
            Self::AnteriorCingulate => 100.0,
            Self::Amygdala => 100.0,
            Self::NucleusAccumbens => 100.0,
            Self::Insula => 100.0,
            Self::DorsalStriatum => 100.0,
            Self::Hippocampus => 100.0,
            Self::DefaultModeNetwork => 100.0,
        }
    }

    /// Get typical response amplitude (% signal change)
    #[must_use]
    pub const fn typical_response(&self) -> f32 {
        match self {
            Self::PrefrontalCortex => 2.0,
            Self::AnteriorCingulate => 3.0,
            Self::Amygdala => 4.0, // High response
            Self::NucleusAccumbens => 3.0,
            Self::Insula => 2.5,
            Self::DorsalStriatum => 2.0,
            Self::Hippocampus => 2.5,
            Self::DefaultModeNetwork => 3.0,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// fMRI time series data for a region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FmriTimeSeries {
    /// Region being measured
    pub region: FeedbackRegion,
    /// BOLD signal values (percent change from baseline)
    pub values: VecDeque<f32>,
    /// Timestamps (TR number)
    pub timestamps: VecDeque<u64>,
    /// Maximum history to keep
    pub max_history: usize,
}

impl FmriTimeSeries {
    /// Create new time series
    #[must_use]
    pub fn new(region: FeedbackRegion, max_history: usize) -> Self {
        Self {
            region,
            values: VecDeque::with_capacity(max_history),
            timestamps: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Add new sample
    pub fn push(&mut self, value: f32, timestamp: u64) {
        if self.values.len() >= self.max_history {
            self.values.pop_front();
            self.timestamps.pop_front();
        }
        self.values.push_back(value);
        self.timestamps.push_back(timestamp);
    }

    /// Get current value
    #[must_use]
    pub fn current(&self) -> Option<f32> {
        self.values.back().copied()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get mean over window
    #[must_use]
    pub fn mean(&self, window: usize) -> Option<f32> {
        let n = window.min(self.values.len());
        if n == 0 {
            return None;
        }
        let sum: f32 = self.values.iter().rev().take(n).sum();
        Some(sum / n as f32)
    }

    /// Get trend (positive = increasing, negative = decreasing)
    #[must_use]
    pub fn trend(&self, window: usize) -> f32 {
        if self.values.len() < 2 {
            return 0.0;
        }
        let n = window.min(self.values.len());
        // Get values in chronological order (oldest first)
        let recent: Vec<f32> = self
            .values
            .iter()
            .rev()
            .take(n)
            .copied()
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        if recent.len() < 2 {
            return 0.0;
        }

        // Simple linear regression slope
        let mean_x = (recent.len() - 1) as f32 / 2.0;
        let mean_y: f32 = recent.iter().sum::<f32>() / recent.len() as f32;

        let numerator: f32 = recent
            .iter()
            .enumerate()
            .map(|(i, y)| (i as f32 - mean_x) * (y - mean_y))
            .sum();
        let denominator: f32 = (0..recent.len()).map(|i| (i as f32 - mean_x).powi(2)).sum();

        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if activity is above target
    #[must_use]
    pub fn above_target(&self, target: f32) -> bool {
        self.current().is_some_and(|v| v > target)
    }

    /// Check if activity is below target
    #[must_use]
    pub fn below_target(&self, target: f32) -> bool {
        self.current().is_some_and(|v| v < target)
    }
}

/// Neurofeedback target
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FeedbackTarget {
    /// Increase region activity above threshold
    Increase {
        /// threshold.
        threshold: f32,
    },
    /// Decrease region activity below threshold
    Decrease {
        /// threshold.
        threshold: f32,
    },
    /// Maintain within range
    Maintain {
        /// min.
        min: f32,
        /// max.
        max: f32,
    },
    /// Modulate relative to baseline
    /// percent change.
    Modulate {
        /// percent change.
        percent_change: f32,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Neurofeedback protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurofeedbackProtocol {
    /// Target region
    pub region: FeedbackRegion,
    /// Feedback target
    pub target: FeedbackTarget,
    /// Feedback display parameters
    pub display: FeedbackDisplay,
}

/// Feedback display type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FeedbackDisplay {
    /// Thermometer visualization
    Thermometer,
    /// Game interface
    Game,
    /// Abstract pattern
    Pattern,
    /// Direct signal
    RawSignal,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Real-time feedback controller
#[derive(Debug)]
pub struct FmriFeedbackController {
    /// fMRI parameters
    pub params: FmriParams,
    /// Current time series
    pub time_series: Vec<FmriTimeSeries>,
    /// Active protocols
    pub protocols: Vec<NeurofeedbackProtocol>,
    /// Current feedback value [0, 1]
    pub feedback_value: f32,
    /// Performance score
    pub performance_score: f32,
    /// Total TRs acquired
    pub tr_count: u64,
}

impl Default for FmriFeedbackController {
    fn default() -> Self {
        Self::new()
    }
}

impl FmriFeedbackController {
    /// Create new controller
    #[must_use]
    pub fn new() -> Self {
        Self {
            params: FmriParams::default(),
            time_series: Vec::new(),
            protocols: Vec::new(),
            feedback_value: 0.5,
            performance_score: 0.0,
            tr_count: 0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add region to monitor
    pub fn add_region(&mut self, region: FeedbackRegion) {
        self.time_series.push(FmriTimeSeries::new(region, 100));
    }

    /// Add feedback protocol
    pub fn add_protocol(&mut self, protocol: NeurofeedbackProtocol) {
        self.protocols.push(protocol);
    }

    /// Acquire new fMRI volume
    pub fn acquire_volume(&mut self, region_activations: &[(FeedbackRegion, f32)]) {
        if self.params.tr <= 0.0 {
            return;
        }
        self.tr_count += 1;

        for (region, activation) in region_activations {
            if let Some(series) = self.time_series.iter_mut().find(|s| s.region == *region) {
                series.push(*activation, self.tr_count);
            }
        }

        // Update feedback
        self.update_feedback();
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Update feedback value based on protocols
    fn update_feedback(&mut self) {
        if self.protocols.is_empty() || self.time_series.is_empty() {
            return;
        }

        let mut total_score = 0.0;
        let mut n_protocols = 0;

        for protocol in &self.protocols {
            if let Some(series) = self
                .time_series
                .iter()
                .find(|s| s.region == protocol.region)
            {
                if let Some(current) = series.current() {
                    let score = match protocol.target {
                        FeedbackTarget::Increase { threshold } => {
                            if current > threshold {
                                1.0
                            } else {
                                current / threshold
                            }
                        }
                        FeedbackTarget::Decrease { threshold } => {
                            if current < threshold {
                                1.0
                            } else {
                                threshold / current
                                // TAG: surface=simulation owner=research-team rule=SIM-001
                            }
                        }
                        FeedbackTarget::Maintain { min, max } => {
                            if current >= min && current <= max {
                                1.0
                            } else if current < min {
                                current / min
                            } else {
                                max / current
                            }
                        }
                        FeedbackTarget::Modulate { percent_change } => {
                            let target_value = protocol.region.baseline_activation()
                                * (1.0 + percent_change / 100.0);
                            1.0 - (current - target_value).abs() / target_value
                        }
                    };
                    total_score += score;
                    n_protocols += 1;
                }
            }
        }

        if n_protocols > 0 {
            self.feedback_value = (total_score / n_protocols as f32).clamp(0.0, 1.0);

            // Update running performance score
            let alpha = 0.1; // Smoothing factor
            self.performance_score =
                (1.0 - alpha) * self.performance_score + alpha * self.feedback_value;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get current feedback for display
    #[must_use]
    pub fn get_feedback(&self) -> FeedbackOutput {
        FeedbackOutput {
            value: self.feedback_value,
            performance: self.performance_score,
            tr_count: self.tr_count,
            message: self.generate_message(),
        }
    }

    /// Generate feedback message
    fn generate_message(&self) -> String {
        if self.performance_score > 0.8 {
            "Excellent control!".to_string()
        } else if self.performance_score > 0.6 {
            "Good progress. Keep it up!".to_string()
        } else if self.performance_score > 0.4 {
            "Getting there. Focus on the target.".to_string()
        } else {
            "Try to relax and focus.".to_string()
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Map feedback to agent behavior modifier
    #[must_use]
    pub fn to_behavior_modifier(&self) -> BehaviorModifier {
        // High feedback value = good neural control = better behavior
        BehaviorModifier {
            focus: self.performance_score,
            emotional_regulation: self.performance_score,
            decision_quality: 0.5 + self.performance_score * 0.5,
        }
    }

    /// Check system latency
    #[must_use]
    pub fn latency_ms(&self) -> f32 {
        // Total latency: acquisition + processing + display
        self.params.tr * 1000.0 + 50.0 + 16.0 // TR + processing + display
    }

    /// Check if real-time requirement met
    #[must_use]
    pub fn realtime_requirement_met(&self) -> bool {
        // Real-time neurofeedback typically requires <3s latency
        self.latency_ms() < 3000.0
    }
}

/// Feedback output for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackOutput {
    /// Feedback value [0, 1]
    pub value: f32,
    /// Running performance score
    pub performance: f32,
    /// Total TRs acquired
    pub tr_count: u64,
    /// Feedback message
    pub message: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Behavior modifier from neurofeedback
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BehaviorModifier {
    /// Focus enhancement [0, 1]
    pub focus: f32,
    /// Emotional regulation [0, 1]
    pub emotional_regulation: f32,
    /// Decision quality modifier [0, 1]
    pub decision_quality: f32,
}

/// Simulated fMRI scanner for testing
#[derive(Debug)]
pub struct SimulatedScanner {
    /// Scanner parameters
    pub params: FmriParams,
    /// Simulation tick
    pub tick: u64,
    /// Baseline activation pattern
    pub baseline_pattern: HashMap<FeedbackRegion, f32>,
}

impl Default for SimulatedScanner {
    fn default() -> Self {
        let mut baseline_pattern = HashMap::new();
        for region in [
            FeedbackRegion::PrefrontalCortex,
            FeedbackRegion::Amygdala,
            FeedbackRegion::NucleusAccumbens,
        ] {
            baseline_pattern.insert(region, 100.0);
        }

        Self {
            params: FmriParams::default(),
            tick: 0,
            baseline_pattern,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl SimulatedScanner {
    /// Create new simulated scanner
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Acquire simulated volume
    pub fn acquire(&mut self, task_engagement: f32) -> Vec<(FeedbackRegion, f32)> {
        self.tick += 1;

        let mut activations = Vec::new();

        for (i, (region, baseline)) in self.baseline_pattern.iter().enumerate() {
            // Deterministic pseudo-random noise derived from region index and simulation tick
            let hash = (self.tick.wrapping_mul(0x9e3779b97f4a7c15))
                .wrapping_add(i as u64 * 0x517cc1b727220a95);
            let noise = (((hash >> 32) as u32 as f32) / (u32::MAX as f32)) * 2.0 - 1.0;
            let task_mod = task_engagement * region.typical_response();
            let value = baseline + task_mod + noise;

            activations.push((*region, value));
        }

        activations
    }
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmri_parameters_default() {
        let params = FmriParams::default();
        assert_eq!(params.tr, 2.0);
        assert_eq!(params.field_strength, 3.0);
    }

    #[test]
    fn test_feedback_region_coordinates() {
        let pfc = FeedbackRegion::PrefrontalCortex;
        let (_x, y, _z) = pfc.mni_coordinates();
        assert!(y > 0.0); // Prefrontal is anterior
    }

    #[test]
    fn test_fmri_time_series() {
        let mut series = FmriTimeSeries::new(FeedbackRegion::Amygdala, 10);

        series.push(101.0, 1);
        series.push(102.0, 2);
        series.push(103.0, 3);

        assert_eq!(series.current(), Some(103.0));
        assert_eq!(series.mean(2), Some(102.5));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_time_series_trend() {
        let mut series = FmriTimeSeries::new(FeedbackRegion::Amygdala, 10);

        // Increasing trend
        for i in 0..5 {
            series.push(100.0 + i as f32 * 2.0, i as u64);
        }

        let trend = series.trend(5);
        assert!(trend > 0.0);
    }

    #[test]
    fn test_feedback_controller() {
        let mut controller = FmriFeedbackController::new();
        controller.add_region(FeedbackRegion::PrefrontalCortex);

        controller.add_protocol(NeurofeedbackProtocol {
            region: FeedbackRegion::PrefrontalCortex,
            target: FeedbackTarget::Increase { threshold: 102.0 },
            display: FeedbackDisplay::Thermometer,
        });

        // Simulate acquisition
        controller.acquire_volume(&[(FeedbackRegion::PrefrontalCortex, 103.0)]);

        let feedback = controller.get_feedback();
        assert!(feedback.value >= 0.0 && feedback.value <= 1.0);
        assert_eq!(controller.tr_count, 1);
    }

    #[test]
    fn test_realtime_requirement() {
        let controller = FmriFeedbackController::new();
        assert!(controller.realtime_requirement_met());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_latency_calculation() {
        let controller = FmriFeedbackController::new();
        let latency = controller.latency_ms();
        assert!(latency > 0.0);
        assert!(latency < 3000.0); // Should be < 3s
    }

    #[test]
    fn test_behavior_modifier() {
        let mut controller = FmriFeedbackController::new();
        controller.performance_score = 0.8;

        let modifier = controller.to_behavior_modifier();
        assert!(modifier.focus > 0.5);
        assert!(modifier.decision_quality > 0.5);
    }

    #[test]
    fn test_simulated_scanner() {
        let mut scanner = SimulatedScanner::new();

        let activations = scanner.acquire(0.5);
        assert!(!activations.is_empty());

        for (_, value) in &activations {
            assert!(value > &90.0 && value < &110.0);
        }
    }

    #[test]
    fn test_feedback_targets() {
        let _increase = FeedbackTarget::Increase { threshold: 105.0 };
        let _maintain = FeedbackTarget::Maintain {
            min: 95.0,
            max: 105.0,
        };

        // Test with series
        let mut series = FmriTimeSeries::new(FeedbackRegion::Amygdala, 5);
        series.push(106.0, 1);

        assert!(series.above_target(105.0));
        assert!(!series.below_target(95.0));
    }
}
