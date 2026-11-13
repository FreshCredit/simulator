//! Multi-Scale Brain Simulation
//!
//! Integrates neural dynamics across multiple spatial and temporal scales:
//! - Molecular (ion channels, receptors)
//! - Cellular (neurons, glia)
//! - Microcircuit (columns, layers)
//! - Mesoscale (brain regions)
//! - Macroscale (whole brain dynamics)

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Spatial scale in the brain hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum BrainScale {
    /// Molecular: ion channels, receptors (~nm)
    Molecular = 0,
    /// Cellular: individual neurons (~10 μm)
    Cellular = 1,
    /// Microcircuit: cortical columns (~1 mm)
    Microcircuit = 2,
    /// Mesoscale: brain regions (~1 cm)
    Mesoscale = 3,
    /// Macroscale: whole brain (~10 cm)
    Macroscale = 4,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BrainScale {
    /// Get typical spatial resolution in meters
    #[must_use]
    pub const fn spatial_resolution(&self) -> f32 {
        match self {
            Self::Molecular => 1e-9,    // 1 nm
            Self::Cellular => 1e-5,     // 10 μm
            Self::Microcircuit => 1e-3, // 1 mm
            Self::Mesoscale => 1e-2,    // 1 cm
            Self::Macroscale => 1e-1,   // 10 cm
        }
    }

    /// Get typical temporal resolution in seconds
    #[must_use]
    pub const fn temporal_resolution(&self) -> f32 {
        match self {
            Self::Molecular => 1e-6,    // 1 μs (ion channel kinetics)
            Self::Cellular => 1e-3,     // 1 ms (spike times)
            Self::Microcircuit => 1e-2, // 10 ms (population bursts)
            Self::Mesoscale => 1e-1,    // 100 ms (fMRI TR)
            Self::Macroscale => 1.0,    // 1 s (behavioral timescale)
        }
    }

    /// Get number of elements at this scale in human brain
    #[must_use]
    pub const fn element_count(&self) -> u64 {
        match self {
            Self::Molecular => 1e18 as u64,   // ~10^18 molecules
            Self::Cellular => 86e9 as u64,    // ~86 billion neurons
            Self::Microcircuit => 1e6 as u64, // ~1 million columns
            Self::Mesoscale => 100_u64,       // ~100 regions
            Self::Macroscale => 1,            // 1 brain
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Molecular scale: Ion channel states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MolecularState {
    /// Sodium channel activation (m gate)
    pub na_m: f32,
    /// Sodium channel inactivation (h gate)
    pub na_h: f32,
    /// Potassium channel activation (n gate)
    pub k_n: f32,
    /// Calcium concentration (mM)
    pub ca_conc: f32,
    /// Neurotransmitter concentrations (mM)
    pub neurotransmitters: [f32; 5], // D, O, C, A, S
}

impl Default for MolecularState {
    fn default() -> Self {
        Self {
            na_m: 0.05,
            na_h: 0.6,
            k_n: 0.32,
            ca_conc: 1e-4,
            neurotransmitters: [0.0; 5],
        }
    }
}

impl MolecularState {
    /// Hodgkin-Huxley sodium current
    #[must_use]
    pub fn na_current(&self, v: f32) -> f32 {
        let g_na = 120.0; // mS/cm²
        let e_na = 50.0; // mV
        g_na * self.na_m.powi(3) * self.na_h * (v - e_na)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Hodgkin-Huxley potassium current
    #[must_use]
    pub fn k_current(&self, v: f32) -> f32 {
        let g_k = 36.0; // mS/cm²
        let e_k = -77.0; // mV
        g_k * self.k_n.powi(4) * (v - e_k)
    }

    /// Step molecular dynamics
    pub fn step(&mut self, v: f32, dt: f32) {
        // Simple Euler integration of HH gating variables with L'Hopital guards
        let x_m = v + 40.0;
        let alpha_m = if x_m.abs() < 1e-4 {
            1.0 // Limit as v -> -40
        } else {
            0.1 * x_m / (1.0 - (-x_m / 10.0).exp())
        };
        let beta_m = 4.0 * (-(v + 65.0) / 18.0).exp();
        self.na_m =
            (self.na_m + (alpha_m * (1.0 - self.na_m) - beta_m * self.na_m) * dt).clamp(0.0, 1.0);

        let alpha_h = 0.07 * (-(v + 65.0) / 20.0).exp();
        let beta_h = 1.0 / (1.0 + (-(v + 35.0) / 10.0).exp());
        self.na_h =
            (self.na_h + (alpha_h * (1.0 - self.na_h) - beta_h * self.na_h) * dt).clamp(0.0, 1.0);

        let x_n = v + 55.0;
        let alpha_n = if x_n.abs() < 1e-4 {
            0.1 // Limit as v -> -55
        } else {
            0.01 * x_n / (1.0 - (-x_n / 10.0).exp())
        };
        let beta_n = 0.125 * (-(v + 65.0) / 80.0).exp();
        self.k_n =
            (self.k_n + (alpha_n * (1.0 - self.k_n) - beta_n * self.k_n) * dt).clamp(0.0, 1.0);
    }
}

/// Cellular scale: Single neuron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellularNeuron {
    /// Membrane potential (mV)
    pub v: f32,
    /// Molecular state
    pub molecular: MolecularState,
    /// Neuron type
    pub neuron_type: NeuronType,
    /// Input current
    pub i_input: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// neuron type.
pub enum NeuronType {
    /// excitatory pyramidal.
    ExcitatoryPyramidal,
    /// inhibitory interneuron.
    InhibitoryInterneuron,
    /// modulatory.
    Modulatory,
}

impl Default for CellularNeuron {
    fn default() -> Self {
        Self {
            v: -65.0,
            molecular: MolecularState::default(),
            neuron_type: NeuronType::ExcitatoryPyramidal,
            i_input: 0.0,
        }
    }
}

impl CellularNeuron {
    /// Step neuron dynamics
    pub fn step(&mut self, dt: f32) {
        // Step molecular dynamics
        self.molecular.step(self.v, dt);

        // Update membrane potential (simplified HH)
        let c_m = 1.0; // μF/cm²
        let i_na = self.molecular.na_current(self.v);
        let i_k = self.molecular.k_current(self.v);
        let i_l = 0.3 * (self.v + 54.387); // Leak current

        let dv = (-i_na - i_k - i_l + self.i_input) / c_m;
        self.v += dv * dt;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if neuron spiked
    #[must_use]
    pub fn check_spike(&self) -> bool {
        self.v > 0.0
    }
}

/// Microcircuit scale: Cortical column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalColumn {
    /// Column ID
    pub id: u32,
    /// Neurons in column
    pub neurons: Vec<CellularNeuron>,
    /// Excitatory population activity
    pub exc_activity: f32,
    /// Inhibitory population activity
    pub inh_activity: f32,
}

impl CorticalColumn {
    /// Create column with n neurons
    #[must_use]
    pub fn new(id: u32, n_neurons: usize) -> Self {
        let mut neurons = Vec::with_capacity(n_neurons);
        for i in 0..n_neurons {
            let neuron_type = if i < n_neurons / 5 {
                NeuronType::InhibitoryInterneuron
            } else {
                NeuronType::ExcitatoryPyramidal
            };
            neurons.push(CellularNeuron {
                neuron_type,
                ..Default::default()
            });
        }

        Self {
            id,
            neurons,
            exc_activity: 0.0,
            inh_activity: 0.0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Step column dynamics
    pub fn step(&mut self, dt: f32) {
        let mut exc_spikes = 0u32;
        let mut inh_spikes = 0u32;
        let mut exc_count = 0u32;
        let mut inh_count = 0u32;

        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            // Deterministic pseudo-random input derived from neuron index
            let hash = (i as u64).wrapping_mul(0x9e3779b97f4a7c15);
            let r = (((hash >> 32) as u32 as f32) / (u32::MAX as f32)) - 0.5;
            neuron.i_input = r * 10.0;
            neuron.step(dt);

            if neuron.check_spike() {
                match neuron.neuron_type {
                    NeuronType::ExcitatoryPyramidal => exc_spikes += 1,
                    NeuronType::InhibitoryInterneuron => inh_spikes += 1,
                    _ => {}
                }
            }

            match neuron.neuron_type {
                NeuronType::ExcitatoryPyramidal => exc_count += 1,
                NeuronType::InhibitoryInterneuron => inh_count += 1,
                _ => {}
            }
        }

        // Update population activities
        if exc_count > 0 {
            self.exc_activity = exc_spikes as f32 / exc_count as f32;
        }
        if inh_count > 0 {
            self.inh_activity = inh_spikes as f32 / inh_count as f32;
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Mesoscale scale: Brain region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRegion {
    /// Region name
    pub name: String,
    /// Cortical columns in region
    pub columns: Vec<CorticalColumn>,
    /// Effective connectivity to other regions
    pub connectivity: Vec<f32>,
    /// Regional activity level
    pub activity: f32,
}

impl BrainRegion {
    /// Create region with n columns
    #[must_use]
    pub fn new(name: &str, n_columns: usize, neurons_per_column: usize) -> Self {
        let columns: Vec<_> = (0..n_columns)
            .map(|i| CorticalColumn::new(i as u32, neurons_per_column))
            .collect();

        Self {
            name: name.to_string(),
            columns,
            connectivity: Vec::new(),
            activity: 0.0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Step region dynamics
    pub fn step(&mut self, dt: f32) {
        for column in &mut self.columns {
            column.step(dt);
        }

        if self.columns.is_empty() {
            self.activity = 0.0;
            return;
        }

        // Average activity across columns
        let total_exc: f32 = self.columns.iter().map(|c| c.exc_activity).sum();
        self.activity = total_exc / self.columns.len() as f32;
    }
}

/// Macroscale: Whole brain simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WholeBrainModel {
    /// Brain regions
    pub regions: Vec<BrainRegion>,
    /// Connectivity matrix between regions
    pub connectivity_matrix: Vec<Vec<f32>>,
    /// Global brain state
    pub global_state: GlobalBrainState,
}

/// Global brain state metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalBrainState {
    /// Global mean activity
    pub mean_activity: f32,
    /// Synchronization index (0-1)
    pub sync_index: f32,
    /// Information integration measure
    pub phi: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl WholeBrainModel {
    /// Create whole brain model with specified regions
    #[must_use]
    pub fn new(region_names: &[&str]) -> Self {
        let regions: Vec<_> = region_names
            .iter()
            .map(|&name| BrainRegion::new(name, 10, 100))
            .collect();

        let n = regions.len();
        // Deterministic connectivity per SIM-004
        let connectivity_matrix: Vec<Vec<f32>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            0.0
                        } else {
                            let hash =
                                ((i as u64) << 32 | j as u64).wrapping_mul(0x9e3779b97f4a7c15);
                            ((hash >> 32) as u32 as f32 / u32::MAX as f32) * 0.1
                        }
                    })
                    .collect()
            })
            .collect();

        Self {
            regions,
            connectivity_matrix,
            global_state: GlobalBrainState::default(),
        }
    }

    /// Step whole brain dynamics
    pub fn step(&mut self, dt: f32) {
        if self.regions.is_empty() {
            self.global_state = GlobalBrainState::default();
            return;
        }

        let current_activities: Vec<f32> = self.regions.iter().map(|r| r.activity).collect();

        // Step each region with inter-regional coupling
        for (i, region) in self.regions.iter_mut().enumerate() {
            if let Some(row) = self.connectivity_matrix.get(i) {
                let cross_input: f32 = row
                    .iter()
                    .zip(current_activities.iter())
                    .map(|(&w, &a)| w * a)
                    .sum();
                for col in &mut region.columns {
                    for neuron in &mut col.neurons {
                        neuron.i_input += cross_input;
                    }
                }
            }
            region.step(dt);
        }

        // Compute global metrics
        let activities: Vec<f32> = self.regions.iter().map(|r| r.activity).collect();
        let mean = activities.iter().sum::<f32>() / activities.len() as f32;
        self.global_state.mean_activity = mean;

        // Bounded synchronization measure in [0.0, 1.0]
        let variance =
            activities.iter().map(|&a| (a - mean).powi(2)).sum::<f32>() / activities.len() as f32;
        let std_dev = variance.sqrt();
        let cv = if mean > 1e-4 { std_dev / mean } else { 0.0 };
        self.global_state.sync_index = (1.0 / (1.0 + cv)).clamp(0.0, 1.0);
        self.global_state.phi = mean * 10.0;
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Multi-scale bridge: connects different scales
#[derive(Debug)]
pub struct MultiscaleBridge {
    /// Current scale being simulated
    pub current_scale: BrainScale,
    /// Upsampling factors
    pub upsample_factors: Vec<usize>,
    /// Downsampling factors
    pub downsample_factors: Vec<usize>,
}

impl Default for MultiscaleBridge {
    fn default() -> Self {
        Self {
            current_scale: BrainScale::Macroscale,
            upsample_factors: vec![100, 100, 100, 100], // Scale up ratios
            downsample_factors: vec![100, 100, 100, 100], // Scale down ratios
        }
    }
}

impl MultiscaleBridge {
    /// Create new multiscale bridge
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Upsample from lower to higher scale
    #[must_use]
    pub fn upsample(&self, from_scale: BrainScale, to_scale: BrainScale, data: &[f32]) -> Vec<f32> {
        if (from_scale as usize) >= (to_scale as usize) {
            return data.to_vec();
        }
        let scale_diff = (to_scale as usize).saturating_sub(from_scale as usize);
        let factor = self
            .upsample_factors
            .iter()
            .take(scale_diff)
            .product::<usize>()
            .clamp(1, 10_000);

        data.iter().flat_map(|&v| vec![v; factor]).collect()
    }

    /// Downsample from higher to lower scale
    #[must_use]
    pub fn downsample(
        &self,
        from_scale: BrainScale,
        to_scale: BrainScale,
        data: &[f32],
    ) -> Vec<f32> {
        if (from_scale as usize) <= (to_scale as usize) {
            return data.to_vec();
        }
        let scale_diff = (from_scale as usize).saturating_sub(to_scale as usize);
        let factor = self
            .downsample_factors
            .iter()
            .take(scale_diff)
            .product::<usize>()
            .max(1);

        data.chunks(factor)
            .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
            .collect()
    }

    /// Map molecular to cellular (ion currents to firing rates)
    #[must_use]
    pub fn molecular_to_cellular(molecular: &MolecularState) -> f32 {
        // Simplified: high sodium activation = higher excitability
        molecular.na_m * (1.0 - molecular.na_h)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Map cellular to microcircuit (spikes to population rate)
    #[must_use]
    pub fn cellular_to_microcircuit(neurons: &[CellularNeuron]) -> f32 {
        if neurons.is_empty() {
            return 0.0;
        }
        let spike_count = neurons.iter().filter(|n| n.check_spike()).count();
        spike_count as f32 / neurons.len() as f32
    }

    /// Map microcircuit to mesoscale (column activity to regional BOLD)
    #[must_use]
    pub fn microcircuit_to_mesoscale(columns: &[CorticalColumn]) -> f32 {
        if columns.is_empty() {
            return 0.0;
        }
        let total_activity: f32 = columns.iter().map(|c| c.exc_activity).sum();
        // Balloon model approximation (Buxton et al. 1998)
        let neural_activity = total_activity / columns.len() as f32;
        let blood_flow = (1.0 + neural_activity * 2.0).max(0.0);
        0.4 * (blood_flow.powf(0.4) - 1.0)
    }

    /// Map mesoscale to macroscale (regional activities to global state)
    #[must_use]
    pub fn mesoscale_to_macroscale(regions: &[BrainRegion]) -> GlobalBrainState {
        if regions.is_empty() {
            return GlobalBrainState::default();
        }
        let activities: Vec<f32> = regions.iter().map(|r| r.activity).collect();
        let mean = activities.iter().sum::<f32>() / regions.len() as f32;

        GlobalBrainState {
            mean_activity: mean,
            sync_index: 0.5,  // Simplified
            phi: mean * 10.0, // Integrated information approximation
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_scale_resolutions() {
        assert!(
            BrainScale::Molecular.spatial_resolution() < BrainScale::Cellular.spatial_resolution()
        );
        assert!(
            BrainScale::Cellular.spatial_resolution() < BrainScale::Mesoscale.spatial_resolution()
        );
    }

    #[test]
    fn test_molecular_state_default() {
        let mol = MolecularState::default();
        assert!(mol.na_m >= 0.0 && mol.na_m <= 1.0);
        assert!(mol.na_h >= 0.0 && mol.na_h <= 1.0);
    }

    #[test]
    fn test_molecular_step_at_singularities() {
        let mut mol = MolecularState::default();
        // Test exact removable singularities at V = -40 mV and V = -55 mV
        mol.step(-40.0, 0.01);
        assert!(!mol.na_m.is_nan());
        assert!(mol.na_m >= 0.0 && mol.na_m <= 1.0);

        mol.step(-55.0, 0.01);
        assert!(!mol.k_n.is_nan());
        assert!(mol.k_n >= 0.0 && mol.k_n <= 1.0);
    }

    #[test]
    fn test_cellular_neuron_step() {
        let mut neuron = CellularNeuron::default();
        let initial_v = neuron.v;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        neuron.step(0.01);

        // Voltage should change
        assert_ne!(neuron.v, initial_v);
    }

    #[test]
    fn test_cortical_column() {
        let mut column = CorticalColumn::new(0, 100);
        assert_eq!(column.neurons.len(), 100);

        column.step(0.01);

        // Activity should be computed
        assert!(column.exc_activity >= 0.0);
    }

    #[test]
    fn test_brain_region() {
        let mut region = BrainRegion::new("Test", 5, 50);
        assert_eq!(region.columns.len(), 5);

        region.step(0.01);
        assert!(region.activity >= 0.0);
    }

    #[test]
    fn test_whole_brain_model() {
        let regions = vec!["Prefrontal", "Parietal", "Temporal", "Occipital"];
        let mut brain = WholeBrainModel::new(&regions);

        assert_eq!(brain.regions.len(), 4);

        brain.step(0.01);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(brain.global_state.mean_activity >= 0.0);
    }

    #[test]
    fn test_multiscale_bridge() {
        let bridge = MultiscaleBridge::new();

        // Test upsampling
        let low_res = vec![1.0, 2.0, 3.0];
        let high_res = bridge.upsample(BrainScale::Mesoscale, BrainScale::Macroscale, &low_res);
        assert!(!high_res.is_empty());
    }

    #[test]
    fn test_molecular_to_cellular() {
        let mol = MolecularState::default();
        let excitability = MultiscaleBridge::molecular_to_cellular(&mol);
        assert!(excitability >= 0.0);
    }

    #[test]
    fn test_cellular_to_microcircuit() {
        let neurons = vec![
            CellularNeuron {
                v: 10.0,
                ..Default::default()
            }, // Spiking
            CellularNeuron {
                v: -70.0,
                ..Default::default()
            }, // Silent
        ];
        let rate = MultiscaleBridge::cellular_to_microcircuit(&neurons);
        assert!((0.0..=1.0).contains(&rate));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_scale_element_counts() {
        // Element count should decrease with scale
        assert!(BrainScale::Molecular.element_count() > BrainScale::Cellular.element_count());
        assert!(BrainScale::Cellular.element_count() > BrainScale::Mesoscale.element_count());
    }

    #[test]
    fn test_temporal_resolution_hierarchy() {
        // Finer scales have higher temporal resolution (smaller dt)
        assert!(
            BrainScale::Molecular.temporal_resolution()
                < BrainScale::Cellular.temporal_resolution()
        );
        assert!(
            BrainScale::Cellular.temporal_resolution()
                < BrainScale::Macroscale.temporal_resolution()
        );
    }

    #[test]
    fn test_hodgkin_huxley_currents() {
        let mol = MolecularState::default();
        let v = -65.0;

        let i_na = mol.na_current(v);
        let i_k = mol.k_current(v);

        // Currents should be finite
        assert!(i_na.is_finite());
        assert!(i_k.is_finite());
    }

    #[test]
    fn test_global_brain_state() {
        let state = GlobalBrainState {
            mean_activity: 0.5,
            sync_index: 0.7,
            phi: 3.0,
        };

        assert!(state.sync_index >= 0.0 && state.sync_index <= 1.0);
        assert!(state.phi > 0.0);
    }
}
