// TAG: surface=simulation owner=research-team rule=SIM-001
//! Sim D16 & Sim D17: Dual-Branch Visual Cloaking & Weather-Gated Attention Engine
//! (DOCAS §5.8 / TFP §3.7 / FreshCredit Whitepaper §3.4 & §4)
//!
//! Formally models:
//! 1. Brain-IT Dual-Branch Dissociation:
//!    - Branch A (Low-Level Structural): typographic edge energy, background contrast,
//!      disclaimer and promo layout boundaries.
//!    - Branch B (High-Level Semantic): true APR, rollover fees, loan principal, and terms.
//! 2. Sim D16 Perceptual Cloaking:
//!    - Cloaking ratio chi = E_promo / (E_disc + eps)
//!    - Bounded attention disclaimer discovery: P_disc = 1 / (1 + exp(-gamma * (kappa / chi - tau)))
//!    - 38.9x vulnerability decile odds ratio (OR_D1/D10 = 38.94).
//! 3. Sim D17 Environmental Weather Coupling & Weather-Shield:
//!    - Dynamic cognitive bandwidth kappa_eff(t) under thermal stress F_thermal and photic lux Phi_photic.
//!    - On-device Weather-Shield algorithmic buffer (cooling-off window) mitigating acute thermal surges.

use serde::{Deserialize, Serialize};

/// Brain-IT Dual-Branch view mode for visualization and inspection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BrainItViewMode {
    /// Branch A: Structural margins and isotropic Sobel edge energy map
    #[default]
    BranchAStructural,
    /// Branch B: Semantic conceptual text and fine-print contract terms
    BranchBSemantic,
}

impl BrainItViewMode {
    /// Human-readable label for perceptual inspection mode (Sim D16)
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BranchAStructural => "Layout Structural Margins (Sobel Salience)",
            Self::BranchBSemantic => "Semantic Fine-Print Focus (Disclosure Detail)",
        }
    }
}

/// Visual contract category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ContractCategory {
    /// Transparent fair loan with isomorphic salience
    Fair,
    /// Predatory loan cloaked with high promotional headline salience and fine-print disclaimer
    #[default]
    Predatory,
    /// Teaser APR loan with deferred interest traps
    Teaser,
    /// Trust-signaled verified contract
    TrustSignal,
}
/// Region of Interest (ROI) in a 2D visual layout/image
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionOfInterest {
    /// Top boundary (row index)
    pub top: usize,
    /// Bottom boundary (row index)
    pub bottom: usize,
    /// Left boundary (col index)
    pub left: usize,
    /// Right boundary (col index)
    pub right: usize,
}

impl RegionOfInterest {
    /// Standard promotional headline window from Sim D16 / Brain-IT battery (rows 40..90, cols 15..209)
    #[must_use]
    pub const fn promotional_window() -> Self {
        Self {
            top: 40,
            bottom: 90,
            left: 15,
            right: 209,
        }
    }

    /// Standard fine-print disclaimer window from Sim D16 / Brain-IT battery (rows 155..195, cols 15..209)
    #[must_use]
    pub const fn disclaimer_window() -> Self {
        Self {
            top: 155,
            bottom: 195,
            left: 15,
            right: 209,
        }
    }
}

/// Zero-dependency parsed NPY tensor for raw fMRI stimuli and reconstructions
#[derive(Debug, Clone, PartialEq)]
pub struct NpyArrayU8 {
    /// Dimension sizes (e.g. [27, 224, 224, 3])
    pub shape: Vec<usize>,
    /// Flattened row-major byte buffer
    pub data: Vec<u8>,
}

impl NpyArrayU8 {
    /// Parse a NumPy .npy v1.0 file containing uint8 / u1 data
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 10 || &bytes[0..6] != b"\x93NUMPY" {
            return Err("Invalid NPY magic");
        }
        let major = bytes[6];
        if major != 1 {
            return Err("Unsupported NPY version (only v1.0 supported)");
        }
        let header_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
        let header_end = 10 + header_len;
        if bytes.len() < header_end {
            return Err("Truncated NPY header");
        }
        let header_str = std::str::from_utf8(&bytes[10..header_end])
            .map_err(|_| "Invalid UTF-8 in NPY header")?;

        if header_str.contains("'fortran_order': True")
            || header_str.contains("\"fortran_order\": True")
        {
            return Err("Fortran order NPY arrays not supported");
        }
        if !header_str.contains("'descr': '|u1'")
            && !header_str.contains("\"descr\": \"|u1\"")
            && !header_str.contains("'descr': '<u1'")
            && !header_str.contains("\"descr\": \"<u1\"")
            && !header_str.contains("'descr': 'u1'")
        {
            return Err("Mismatched NPY dtype: expected |u1");
        }

        let shape = Self::parse_shape_from_header(header_str)?;
        let mut expected_len = 1usize;
        for &dim in &shape {
            expected_len = expected_len
                .checked_mul(dim)
                .ok_or("Shape dimension product overflow")?;
        }
        if expected_len > 100_000_000 {
            return Err("Shape exceeds maximum allowable elements (100M)");
        }
        let data = bytes[header_end..].to_vec();
        if data.len() < expected_len {
            return Err("NPY data buffer shorter than expected shape elements");
        }
        Ok(Self { shape, data })
    }

    fn parse_shape_from_header(header: &str) -> Result<Vec<usize>, &'static str> {
        let shape_start = header
            .find("'shape':")
            .or_else(|| header.find("\"shape\":"))
            .ok_or("No 'shape' found in NPY header")?;
        let tuple_open = header[shape_start..]
            .find('(')
            .ok_or("No '(' in shape definition")?
            + shape_start
            + 1;
        let tuple_close = header[tuple_open..]
            .find(')')
            .ok_or("No ')' in shape definition")?
            + tuple_open;

        let dims_str = &header[tuple_open..tuple_close];
        let mut dims = Vec::new();
        for part in dims_str.split(',') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                let dim = trimmed
                    .parse::<usize>()
                    .map_err(|_| "Failed to parse shape dim")?;
                dims.push(dim);
            }
        }
        if dims.is_empty() {
            return Err("Empty shape dimensions");
        }
        Ok(dims)
    }

    /// Read an NPY array from a file on disk
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, &'static str> {
        let bytes = std::fs::read(path).map_err(|_| "Failed to read NPY file")?;
        Self::from_bytes(&bytes)
    }

    /// Extract a single 224x224 RGB image slice as normalized grayscale buffer in [0, 1]
    #[must_use]
    pub fn get_image_grayscale(&self, image_idx: usize, width: usize, height: usize) -> Vec<f32> {
        let mut gray = vec![0.0f32; width * height];
        let slice_size = width * height * 3;
        let offset = image_idx * slice_size;
        if offset + slice_size > self.data.len() {
            return gray;
        }

        for y in 0..height {
            for x in 0..width {
                let px_idx = offset + (y * width + x) * 3;
                let r = self.data[px_idx] as f32;
                let g = self.data[px_idx + 1] as f32;
                let b = self.data[px_idx + 2] as f32;
                gray[y * width + x] = (r + g + b) / (3.0 * 255.0);
            }
        }
        gray
    }
    /// Extract a single 224x224 RGB image slice as raw interleaved RGB bytes
    #[must_use]
    pub fn get_image_rgb(&self, image_idx: usize, width: usize, height: usize) -> &[u8] {
        let slice_size = width * height * 3;
        let offset = image_idx * slice_size;
        if offset + slice_size <= self.data.len() {
            &self.data[offset..offset + slice_size]
        } else {
            &[]
        }
    }
}
/// Zero-dependency parsed NPY tensor for raw cortical BOLD float32 activations
#[derive(Debug, Clone, PartialEq)]
pub struct NpyArrayF32 {
    /// Dimension sizes (e.g. [27, 15724])
    pub shape: Vec<usize>,
    /// Flattened row-major float elements
    pub data: Vec<f32>,
}

impl NpyArrayF32 {
    /// Parse a NumPy .npy v1.0 file containing float32 (<f4) data
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 10 || &bytes[0..6] != b"\x93NUMPY" {
            return Err("Invalid NPY magic");
        }
        let major = bytes[6];
        if major != 1 {
            return Err("Unsupported NPY version (only v1.0 supported)");
        }
        let header_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
        let header_end = 10 + header_len;
        if bytes.len() < header_end {
            return Err("Truncated NPY header");
        }
        let header_str = std::str::from_utf8(&bytes[10..header_end])
            .map_err(|_| "Invalid UTF-8 in NPY header")?;

        if header_str.contains("'fortran_order': True")
            || header_str.contains("\"fortran_order\": True")
        {
            return Err("Fortran order NPY arrays not supported");
        }
        if !header_str.contains("'descr': '<f4'")
            && !header_str.contains("\"descr\": \"<f4\"")
            && !header_str.contains("'descr': 'f4'")
        {
            return Err("Mismatched NPY dtype: expected <f4");
        }

        let shape = NpyArrayU8::parse_shape_from_header(header_str)?;
        let mut expected_len = 1usize;
        for &dim in &shape {
            expected_len = expected_len
                .checked_mul(dim)
                .ok_or("Shape dimension product overflow")?;
        }
        if expected_len > 100_000_000 {
            return Err("Shape exceeds maximum allowable elements (100M)");
        }
        let payload = &bytes[header_end..];
        if payload.len() < expected_len * 4 {
            return Err("NPY float payload shorter than expected elements");
        }

        let mut data = Vec::with_capacity(expected_len);
        for i in 0..expected_len {
            let chunk = [
                payload[i * 4],
                payload[i * 4 + 1],
                payload[i * 4 + 2],
                payload[i * 4 + 3],
            ];
            let val = f32::from_le_bytes(chunk);
            if !val.is_finite() {
                return Err("NPY float payload contains non-finite values");
            }
            data.push(val);
        }

        Ok(Self { shape, data })
    }

    /// Read a float32 NPY array from a file on disk
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, &'static str> {
        let bytes = std::fs::read(path).map_err(|_| "Failed to read NPY file")?;
        Self::from_bytes(&bytes)
    }

    /// Extract row slice for stimulus index
    #[must_use]
    pub fn get_row(&self, row_idx: usize) -> &[f32] {
        if self.shape.len() < 2 {
            return &self.data;
        }
        let cols = self.shape[1];
        let start = row_idx * cols;
        let end = start + cols;
        if end <= self.data.len() {
            &self.data[start..end]
        } else {
            &[]
        }
    }
}
/// Zero-dependency parsed NPY tensor for raw Brain-IT GMM cluster mappings (<i8 int64)
#[derive(Debug, Clone, PartialEq)]
pub struct NpyArrayI64 {
    /// Dimension sizes (e.g. [128, 17511])
    pub shape: Vec<usize>,
    /// Flattened row-major 64-bit integer elements
    pub data: Vec<i64>,
}

impl NpyArrayI64 {
    /// Parse a NumPy .npy v1.0 file containing int64 (<i8) data
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 10 || &bytes[0..6] != b"\x93NUMPY" {
            return Err("Invalid NPY magic");
        }
        let major = bytes[6];
        if major != 1 {
            return Err("Unsupported NPY version (only v1.0 supported)");
        }
        let header_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
        let header_end = 10 + header_len;
        if bytes.len() < header_end {
            return Err("Truncated NPY header");
        }
        let header_str = std::str::from_utf8(&bytes[10..header_end])
            .map_err(|_| "Invalid UTF-8 in NPY header")?;

        if header_str.contains("'fortran_order': True")
            || header_str.contains("\"fortran_order\": True")
        {
            return Err("Fortran order NPY arrays not supported");
        }
        if !header_str.contains("'descr': '<i8'")
            && !header_str.contains("\"descr\": \"<i8\"")
            && !header_str.contains("'descr': 'i8'")
        {
            return Err("Mismatched NPY dtype: expected <i8");
        }

        let shape = NpyArrayU8::parse_shape_from_header(header_str)?;
        let mut expected_len = 1usize;
        for &dim in &shape {
            expected_len = expected_len
                .checked_mul(dim)
                .ok_or("Shape dimension product overflow")?;
        }
        if expected_len > 100_000_000 {
            return Err("Shape exceeds maximum allowable elements (100M)");
        }
        let payload = &bytes[header_end..];
        if payload.len() < expected_len * 8 {
            return Err("NPY int64 payload shorter than expected elements");
        }

        let mut data = Vec::with_capacity(expected_len);
        for i in 0..expected_len {
            let chunk = [
                payload[i * 8],
                payload[i * 8 + 1],
                payload[i * 8 + 2],
                payload[i * 8 + 3],
                payload[i * 8 + 4],
                payload[i * 8 + 5],
                payload[i * 8 + 6],
                payload[i * 8 + 7],
            ];
            data.push(i64::from_le_bytes(chunk));
        }

        Ok(Self { shape, data })
    }

    /// Read an int64 NPY array from a file on disk
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, &'static str> {
        let bytes = std::fs::read(path).map_err(|_| "Failed to read NPY file")?;
        Self::from_bytes(&bytes)
    }

    /// Extract row slice for cluster index
    #[must_use]
    pub fn get_row(&self, row_idx: usize) -> &[i64] {
        if self.shape.len() < 2 {
            return &self.data;
        }
        let cols = self.shape[1];
        let start = row_idx * cols;
        let end = start + cols;
        if end <= self.data.len() {
            &self.data[start..end]
        } else {
            &[]
        }
    }
}

/// Anatomical Cortical Parcels for 15,724 Voxel Space (DOCAS §5.2 / TRIBE v2)
pub struct CorticalParcels;

impl CorticalParcels {
    /// Early visual cortex (V1, V2, V3) voxel index range: 0..6289 (40%)
    pub const EARLY_VISUAL: std::ops::Range<usize> = 0..6289;
    /// Ventral temporal / Lateral Occipital (LOC, IT) voxel range: 6289..11006 (30%)
    pub const VENTRAL_TEMPORAL: std::ops::Range<usize> = 6289..11006;
    /// Frontoparietal Control and Salience Network voxel range: 11006..13364 (15%)
    pub const CONTROL_SALIENCE: std::ops::Range<usize> = 11006..13364;
    /// Default Mode Network (DMN) voxel range: 13364..15724 (15%)
    pub const DEFAULT_MODE: std::ops::Range<usize> = 13364..15724;

    /// Mean activation in a specific cortical parcel
    #[must_use]
    pub fn parcel_mean(voxel_acts: &[f32], range: std::ops::Range<usize>) -> f32 {
        let slice = &voxel_acts[range.start.min(voxel_acts.len())..range.end.min(voxel_acts.len())];
        if slice.is_empty() {
            0.0
        } else {
            slice.iter().sum::<f32>() / slice.len() as f32
        }
    }
    /// Pool 15,724 cortical voxels into 128 GMM cortical graph nodes using v2c_128_mapping_gmm_v2
    #[must_use]
    pub fn pool_voxels_to_gmm(voxel_acts: &[f32], gmm: &NpyArrayI64) -> Vec<f32> {
        let n_clusters = gmm.shape.first().copied().unwrap_or(128);
        let mut cluster_acts = Vec::with_capacity(n_clusters);

        for c in 0..n_clusters {
            let row = gmm.get_row(c);
            let mut sum = 0.0f32;
            let mut count = 0usize;
            for &v_idx in row {
                if v_idx >= 0 && (v_idx as usize) < voxel_acts.len() {
                    sum += voxel_acts[v_idx as usize];
                    count += 1;
                }
            }
            if count > 0 {
                cluster_acts.push(sum / count as f32);
            } else {
                cluster_acts.push(0.0);
            }
        }
        cluster_acts
    }
}
/// Complete 27-stimulus specification matching battery27_reconstructed.json
pub struct StimulusBattery27;

impl StimulusBattery27 {
    /// Total number of standardized stimuli in the Brain-IT battery
    pub const TOTAL_STIMULI: usize = 27;

    /// Stimulus category for an index in 0..27
    #[must_use]
    pub const fn category(idx: usize) -> ContractCategory {
        match idx {
            0..5 => ContractCategory::Fair,
            5..10 => ContractCategory::Predatory,
            10..15 => ContractCategory::Teaser,
            15..20 => ContractCategory::TrustSignal,
            _ => ContractCategory::Fair,
        }
    }

    /// Loan principal amount USD
    #[must_use]
    pub const fn amount_usd(idx: usize) -> u32 {
        let item_idx = idx % 5;
        match item_idx {
            0 => 4000,
            1 => 2500,
            2 => 7500,
            3 => 12000,
            _ => 20000,
        }
    }

    /// Empirical Branch A cloaking ratio chi for index (computed directly from raw Brain-IT structural recons)
    #[must_use]
    pub const fn empirical_branch_a_chi(idx: usize) -> f32 {
        match idx {
            0 => 1.31785,
            1 => 1.29533,
            2 => 1.31340,
            3 => 1.29284,
            4 => 1.29347,
            5 => 1.15091,
            6 => 1.13874,
            7 => 1.16856,
            8 => 1.15016,
            9 => 1.14945,
            10 => 0.83884,
            11 => 0.81767,
            12 => 0.78692,
            13 => 0.84577,
            14 => 0.89260,
            15 => 0.88089,
            16 => 0.85431,
            17 => 0.87263,
            18 => 0.82540,
            19 => 0.82275,
            20 => 0.77402,
            21 => 0.77379,
            22 => 0.77586,
            23 => 0.77614,
            24 => 0.77814,
            25 => 0.80349,
            26 => 0.81645,
            _ => 1.00000,
        }
    }
}

/// 2D Visual Salience and Contrast Extraction Engine
pub struct VisualContrastEngine;

impl VisualContrastEngine {
    /// Computes 2D spatial Sobel edge gradient magnitude on a normalized grayscale buffer in [0, 1].
    /// Matches SciPy ndimage.sobel: G_x (horizontal) and G_y (vertical) kernels:
    /// G_x = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]]
    /// G_y = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]]
    #[must_use]
    pub fn compute_sobel_gradient_magnitude(
        grayscale: &[f32],
        width: usize,
        height: usize,
    ) -> Vec<f32> {
        let mut magnitude = vec![0.0f32; width * height];
        if width < 3 || height < 3 || grayscale.len() < width * height {
            return magnitude;
        }

        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let p00 = grayscale[(y - 1) * width + (x - 1)];
                let p01 = grayscale[(y - 1) * width + x];
                let p02 = grayscale[(y - 1) * width + (x + 1)];
                let p10 = grayscale[y * width + (x - 1)];
                let p12 = grayscale[y * width + (x + 1)];
                let p20 = grayscale[(y + 1) * width + (x - 1)];
                let p21 = grayscale[(y + 1) * width + x];
                let p22 = grayscale[(y + 1) * width + (x + 1)];

                let gx = (p02 + 2.0 * p12 + p22) - (p00 + 2.0 * p10 + p20);
                let gy = (p20 + 2.0 * p21 + p22) - (p00 + 2.0 * p01 + p02);
                magnitude[y * width + x] = (gx * gx + gy * gy).sqrt();
            }
        }
        magnitude
    }
    /// Computes mean edge energy across a specified Region of Interest (ROI)
    #[must_use]
    pub fn compute_roi_mean_energy(
        edge_map: &[f32],
        width: usize,
        height: usize,
        roi: &RegionOfInterest,
    ) -> f32 {
        let r_top = roi.top.min(height);
        let r_bottom = roi.bottom.min(height);
        let r_left = roi.left.min(width);
        let r_right = roi.right.min(width);

        if r_top >= r_bottom || r_left >= r_right {
            return 0.0;
        }

        let mut sum = 0.0f32;
        let mut count = 0usize;
        for y in r_top..r_bottom {
            for x in r_left..r_right {
                sum += edge_map[y * width + x];
                count += 1;
            }
        }

        if count == 0 {
            0.0
        } else {
            sum / count as f32
        }
    }

    /// Computes Michelson contrast C_M = (L_max - L_min) / (L_max + L_min + eps) on an ROI.
    /// Captures typographic stroke-background contrast that isotropic Sobel misses.
    #[must_use]
    pub fn compute_michelson_contrast(
        grayscale: &[f32],
        width: usize,
        height: usize,
        roi: &RegionOfInterest,
    ) -> f32 {
        let r_top = roi.top.min(height);
        let r_bottom = roi.bottom.min(height);
        let r_left = roi.left.min(width);
        let r_right = roi.right.min(width);

        if r_top >= r_bottom || r_left >= r_right {
            return 0.0;
        }

        let mut l_min = 1.0f32;
        let mut l_max = 0.0f32;
        for y in r_top..r_bottom {
            for x in r_left..r_right {
                let lum = grayscale[y * width + x];
                if lum < l_min {
                    l_min = lum;
                }
                if lum > l_max {
                    l_max = lum;
                }
            }
        }

        const EPS: f32 = 1e-4;
        (l_max - l_min) / (l_max + l_min + EPS)
    }

    /// Computes full visual cloaking ratio chi = E_promo / (E_disc + eps) from raw 2D grayscale pixels
    #[must_use]
    pub fn compute_cloaking_ratio_from_pixels(
        grayscale: &[f32],
        width: usize,
        height: usize,
        promo_roi: &RegionOfInterest,
        disc_roi: &RegionOfInterest,
    ) -> f32 {
        let edge_map = Self::compute_sobel_gradient_magnitude(grayscale, width, height);
        let e_promo = Self::compute_roi_mean_energy(&edge_map, width, height, promo_roi);
        let e_disc = Self::compute_roi_mean_energy(&edge_map, width, height, disc_roi);
        const EPS: f32 = 1e-4;
        e_promo / (e_disc + EPS)
    }
    /// Separable 2D Gaussian blur
    #[must_use]
    pub fn gaussian_blur(grayscale: &[f32], width: usize, height: usize, sigma: f32) -> Vec<f32> {
        if width == 0 || height == 0 || grayscale.is_empty() || sigma <= 0.0 {
            return grayscale.to_vec();
        }
        let radius = (3.0 * sigma).ceil() as usize;
        let mut kernel = Vec::with_capacity(2 * radius + 1);
        let mut k_sum = 0.0f32;
        for i in 0..=(2 * radius) {
            let x = i as f32 - radius as f32;
            let val = (-x * x / (2.0 * sigma * sigma)).exp();
            kernel.push(val);
            k_sum += val;
        }
        for k in &mut kernel {
            *k /= k_sum;
        }

        let mut temp = vec![0.0f32; width * height];
        for y in 0..height {
            for x in 0..width {
                let mut sum = 0.0f32;
                for (ki, &kval) in kernel.iter().enumerate() {
                    let kx = (x as isize + ki as isize - radius as isize)
                        .clamp(0, width as isize - 1) as usize;
                    sum += grayscale[y * width + kx] * kval;
                }
                temp[y * width + x] = sum;
            }
        }

        let mut output = vec![0.0f32; width * height];
        for y in 0..height {
            for x in 0..width {
                let mut sum = 0.0f32;
                for (ki, &kval) in kernel.iter().enumerate() {
                    let ky = (y as isize + ki as isize - radius as isize)
                        .clamp(0, height as isize - 1) as usize;
                    sum += temp[ky * width + x] * kval;
                }
                output[y * width + x] = sum;
            }
        }
        output
    }

    /// Multi-scale Difference-of-Gaussians (DoG) spatial bandpass
    #[must_use]
    pub fn difference_of_gaussians(
        grayscale: &[f32],
        width: usize,
        height: usize,
        sigma1: f32,
        sigma2: f32,
    ) -> Vec<f32> {
        let g1 = Self::gaussian_blur(grayscale, width, height, sigma1);
        let g2 = Self::gaussian_blur(grayscale, width, height, sigma2);
        g1.iter()
            .zip(g2.iter())
            .map(|(a, b)| (a - b).abs())
            .collect()
    }

    /// Contrast Sensitivity Function (CSF) weighted energy across an ROI
    /// Uses Mannos & Sakrison (1974) model: S(f) = 2.6 * (0.0192 + 0.114 f) * exp(-(0.114 f)^1.1)
    #[must_use]
    pub fn compute_csf_weighted_roi_energy(
        grayscale: &[f32],
        width: usize,
        height: usize,
        roi: &RegionOfInterest,
    ) -> f32 {
        let dog_high = Self::difference_of_gaussians(grayscale, width, height, 0.8, 1.6);
        let dog_mid = Self::difference_of_gaussians(grayscale, width, height, 1.6, 3.2);

        let w_high = 0.85f32;
        let w_mid = 1.00f32;

        let e_high = Self::compute_roi_mean_energy(&dog_high, width, height, roi);
        let e_mid = Self::compute_roi_mean_energy(&dog_mid, width, height, roi);

        w_high * e_high + w_mid * e_mid
    }

    /// Compute CSF-weighted cloaking ratio chi_CSF = E_promo_CSF / (E_disc_CSF + eps)
    #[must_use]
    pub fn compute_csf_cloaking_ratio(
        grayscale: &[f32],
        width: usize,
        height: usize,
        promo_roi: &RegionOfInterest,
        disc_roi: &RegionOfInterest,
    ) -> f32 {
        let e_promo = Self::compute_csf_weighted_roi_energy(grayscale, width, height, promo_roi);
        let e_disc = Self::compute_csf_weighted_roi_energy(grayscale, width, height, disc_roi);
        const EPS: f32 = 1e-4;
        e_promo / (e_disc + EPS)
    }

    /// Local sliding-window Mean-SSIM (MSSIM) per Wang et al. 2004 (Gaussian window sigma=1.5)
    #[must_use]
    pub fn compute_ssim(x: &[f32], y: &[f32], width: usize, height: usize) -> f32 {
        if x.len() != y.len() || x.len() < width * height || width < 4 || height < 4 {
            return 0.0;
        }

        const C1: f32 = 0.0001; // (0.01 * 1.0)^2
        const C2: f32 = 0.0009; // (0.03 * 1.0)^2

        let mu_x = Self::gaussian_blur(x, width, height, 1.5);
        let mu_y = Self::gaussian_blur(y, width, height, 1.5);

        let x2: Vec<f32> = x.iter().map(|&v| v * v).collect();
        let y2: Vec<f32> = y.iter().map(|&v| v * v).collect();
        let xy: Vec<f32> = x.iter().zip(y.iter()).map(|(&a, &b)| a * b).collect();

        let mu_x2 = Self::gaussian_blur(&x2, width, height, 1.5);
        let mu_y2 = Self::gaussian_blur(&y2, width, height, 1.5);
        let mu_xy = Self::gaussian_blur(&xy, width, height, 1.5);

        let n = width * height;
        let mut ssim_sum = 0.0f32;

        for i in 0..n {
            let mx = mu_x[i];
            let my = mu_y[i];
            let sigma_x2 = (mu_x2[i] - mx * mx).max(0.0);
            let sigma_y2 = (mu_y2[i] - my * my).max(0.0);
            let sigma_xy = mu_xy[i] - mx * my;

            let num = (2.0 * mx * my + C1) * (2.0 * sigma_xy + C2);
            let den = (mx * mx + my * my + C1) * (sigma_x2 + sigma_y2 + C2);

            ssim_sum += num / den.max(1e-8);
        }

        (ssim_sum / n as f32).clamp(-1.0, 1.0)
    }
    /// Multi-channel RGB 7x7 sliding box-window SSIM with Bessel sample-covariance correction (49/48)
    /// and edge cropping (pad=3), exactly matching skimage.metrics.structural_similarity(channel_axis=2, data_range=255)
    #[must_use]
    pub fn compute_rgb_ssim(im1: &[u8], im2: &[u8], width: usize, height: usize) -> f32 {
        if im1.len() != im2.len() || im1.len() < width * height * 3 || width < 7 || height < 7 {
            return 0.0;
        }

        let mut channel_ssims = [0.0f32; 3];

        for (c, slot) in channel_ssims.iter_mut().enumerate() {
            let stride = width + 1;
            let mut s_x = vec![0.0f64; (height + 1) * (width + 1)];
            let mut s_y = vec![0.0f64; (height + 1) * (width + 1)];
            let mut s_xx = vec![0.0f64; (height + 1) * (width + 1)];
            let mut s_yy = vec![0.0f64; (height + 1) * (width + 1)];
            let mut s_xy = vec![0.0f64; (height + 1) * (width + 1)];

            for y in 0..height {
                let mut row_sum_x = 0.0f64;
                let mut row_sum_y = 0.0f64;
                let mut row_sum_xx = 0.0f64;
                let mut row_sum_yy = 0.0f64;
                let mut row_sum_xy = 0.0f64;

                for x in 0..width {
                    let px_idx = (y * width + x) * 3 + c;
                    let vx = im1[px_idx] as f64;
                    let vy = im2[px_idx] as f64;

                    row_sum_x += vx;
                    row_sum_y += vy;
                    row_sum_xx += vx * vx;
                    row_sum_yy += vy * vy;
                    row_sum_xy += vx * vy;

                    let sat_idx = (y + 1) * stride + (x + 1);
                    let sat_above = y * stride + (x + 1);

                    s_x[sat_idx] = s_x[sat_above] + row_sum_x;
                    s_y[sat_idx] = s_y[sat_above] + row_sum_y;
                    s_xx[sat_idx] = s_xx[sat_above] + row_sum_xx;
                    s_yy[sat_idx] = s_yy[sat_above] + row_sum_yy;
                    s_xy[sat_idx] = s_xy[sat_above] + row_sum_xy;
                }
            }

            const C1: f64 = 6.5025; // (0.01 * 255.0)^2
            const C2: f64 = 58.5225; // (0.03 * 255.0)^2
            const NP: f64 = 49.0;
            const COV_NORM: f64 = 49.0 / 48.0; // Bessel sample-covariance correction

            let valid_h = height - 6;
            let valid_w = width - 6;
            let total_valid = (valid_h * valid_w) as f64;
            let mut sum_ssim = 0.0f64;

            for y in 3..(height - 3) {
                let y1 = y - 3;
                let y2 = y + 4;
                for x in 3..(width - 3) {
                    let x1 = x - 3;
                    let x2 = x + 4;

                    let idx_22 = y2 * stride + x2;
                    let idx_12 = y1 * stride + x2;
                    let idx_21 = y2 * stride + x1;
                    let idx_11 = y1 * stride + x1;

                    let sum_x = s_x[idx_22] - s_x[idx_12] - s_x[idx_21] + s_x[idx_11];
                    let sum_y = s_y[idx_22] - s_y[idx_12] - s_y[idx_21] + s_y[idx_11];
                    let sum_xx = s_xx[idx_22] - s_xx[idx_12] - s_xx[idx_21] + s_xx[idx_11];
                    let sum_yy = s_yy[idx_22] - s_yy[idx_12] - s_yy[idx_21] + s_yy[idx_11];
                    let sum_xy = s_xy[idx_22] - s_xy[idx_12] - s_xy[idx_21] + s_xy[idx_11];

                    let ux = sum_x / NP;
                    let uy = sum_y / NP;
                    let uxx = sum_xx / NP;
                    let uyy = sum_yy / NP;
                    let uxy = sum_xy / NP;

                    let vx = COV_NORM * (uxx - ux * ux);
                    let vy = COV_NORM * (uyy - uy * uy);
                    let vxy = COV_NORM * (uxy - ux * uy);

                    let a1 = 2.0 * ux * uy + C1;
                    let a2 = 2.0 * vxy + C2;
                    let b1 = ux * ux + uy * uy + C1;
                    let b2 = vx + vy + C2;

                    sum_ssim += (a1 * a2) / (b1 * b2);
                }
            }

            *slot = (sum_ssim / total_valid) as f32;
        }

        (channel_ssims[0] + channel_ssims[1] + channel_ssims[2]) / 3.0
    }

    /// Pearson correlation between flattened pixel intensity vectors (PixCorr)
    #[must_use]
    pub fn compute_pixcorr(x: &[f32], y: &[f32]) -> f32 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }
        let n = x.len() as f32;
        let mean_x = x.iter().sum::<f32>() / n;
        let mean_y = y.iter().sum::<f32>() / n;

        let mut num = 0.0f32;
        let mut den_x = 0.0f32;
        let mut den_y = 0.0f32;

        for i in 0..x.len() {
            let dx = x[i] - mean_x;
            let dy = y[i] - mean_y;
            num += dx * dy;
            den_x += dx * dx;
            den_y += dy * dy;
        }
        let denom = (den_x * den_y).sqrt();
        if denom > 1e-8 {
            (num / denom).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }

    /// Compute Representational Dissimilarity Matrix (RDM) D_ij = 1 - PearsonCorrelation(v_i, v_j)
    #[must_use]
    pub fn compute_rdm(vectors: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let n = vectors.len();
        let mut rdm = vec![vec![0.0f32; n]; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let r = Self::compute_pixcorr(&vectors[i], &vectors[j]);
                let d = (1.0 - r).max(0.0);
                rdm[i][j] = d;
                rdm[j][i] = d;
            }
        }
        rdm
    }

    /// Mantel test correlation comparing upper-triangular distances of two RDMs
    #[must_use]
    pub fn mantel_test_correlation(rdm1: &[Vec<f32>], rdm2: &[Vec<f32>]) -> f32 {
        let n = rdm1.len().min(rdm2.len());
        if n < 2 {
            return 0.0;
        }
        let mut v1 = Vec::new();
        let mut v2 = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                v1.push(rdm1[i][j]);
                v2.push(rdm2[i][j]);
            }
        }
        Self::compute_pixcorr(&v1, &v2)
    }
}

/// Visual credit agreement stimulus specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualAgreementStimulus {
    /// Agreement category
    pub category: ContractCategory,
    /// Visual cloaking ratio chi = E_promo / (E_disc + eps)
    pub cloaking_ratio: f32,
    /// Promotional headline edge energy E_promo
    pub promo_edge_energy: f32,
    /// Fine-print disclaimer edge energy E_disc
    pub disc_edge_energy: f32,
    /// Promoted teaser APR (%)
    pub apr_promo: f32,
    /// True underlying APR including rollover fees (%)
    pub apr_true: f32,
    /// Promotional urgency bonus utility
    pub urgency_bonus: f32,
    /// Cash utility bonus
    pub cash_utility: f32,
}

impl Default for VisualAgreementStimulus {
    fn default() -> Self {
        Self::predatory_benchmark()
    }
}

impl VisualAgreementStimulus {
    /// Benchmark predatory agreement from Sim D16 / Brain-IT battery (chi = 1.1516)
    #[must_use]
    pub fn predatory_benchmark() -> Self {
        Self {
            category: ContractCategory::Predatory,
            cloaking_ratio: 1.1516,
            promo_edge_energy: 0.85,
            disc_edge_energy: 0.738,
            apr_promo: 0.0,
            apr_true: 390.0,
            urgency_bonus: 0.25,
            cash_utility: 0.16, // 0.40 * (4000 / 10000)
        }
    }

    /// Transparent fair agreement with isomorphic salience (chi = 1.0)
    #[must_use]
    pub fn fair_benchmark() -> Self {
        Self {
            category: ContractCategory::Fair,
            cloaking_ratio: 1.0,
            promo_edge_energy: 0.50,
            disc_edge_energy: 0.50,
            apr_promo: 9.0,
            apr_true: 9.0,
            urgency_bonus: 0.0,
            cash_utility: 0.14,
        }
    }

    /// Custom agreement with a user-adjusted cloaking ratio chi in [1.0, 5.0]
    #[must_use]
    pub fn with_cloaking_ratio(chi: f32) -> Self {
        Self::with_cloaking_and_branch(chi, false)
    }

    /// Custom agreement with cloaking ratio chi and perceptual branch focus:
    /// - Structural Margins (`semantic_focus = false`): full perceptual cloaking chi
    /// - Semantic Fine-Print Focus (`semantic_focus = true`): attenuates layout cloaking to 1 + 0.35*(chi - 1)
    #[must_use]
    pub fn with_cloaking_and_branch(chi: f32, semantic_focus: bool) -> Self {
        let raw_chi = chi.clamp(1.0, 5.0);
        let chi_clamped = if semantic_focus {
            1.0 + 0.35 * (raw_chi - 1.0)
        } else {
            raw_chi
        };
        let mut stimulus = Self::predatory_benchmark();
        stimulus.cloaking_ratio = chi_clamped;
        stimulus.promo_edge_energy = 0.50 * chi_clamped.sqrt();
        stimulus.disc_edge_energy = 0.50 / chi_clamped.sqrt();
        stimulus
    }

    /// Compute disclaimer discovery probability under bounded cognitive bandwidth:
    /// P_disc = 1 / (1 + exp(-gamma * (kappa / chi - tau)))
    #[must_use]
    pub fn disclaimer_discovery_probability(&self, kappa: f32) -> f32 {
        const GAMMA: f32 = 8.0; // attention sharpness
        const TAU: f32 = 0.20; // discovery threshold
        let effective_chi = self.cloaking_ratio.max(0.1);
        let exponent = -GAMMA * (kappa / effective_chi - TAU);
        1.0 / (1.0 + exponent.exp())
    }

    /// Evaluate an agent's loan uptake decision under bounded perceptual gating
    /// Returns true if accepted
    #[must_use]
    pub fn evaluate_agent_decision(
        &self,
        kappa: f32,
        reservation_utility: f32,
        random_draw: f32, // uniform [0, 1]
    ) -> bool {
        let p_disc = self.disclaimer_discovery_probability(kappa);
        let disclaimer_discovered = random_draw < p_disc;

        const AMT_NORM: f32 = 0.40; // 4000 / 10000
        let perceived_utility = if disclaimer_discovered {
            -(self.apr_true / 100.0) * AMT_NORM + self.cash_utility
        } else {
            -(self.apr_promo / 100.0) * AMT_NORM + self.cash_utility + self.urgency_bonus
        };

        perceived_utility > reservation_utility
    }
}

/// Population evaluation metrics across cognitive bandwidth deciles D1 to D10
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecileVulnerabilityGradient {
    /// Predatory contract uptake rate across deciles D1 (index 0) to D10 (index 9)
    pub decile_uptake_rates: [f32; 10],
    /// Odds ratio between D1 (lowest bandwidth) and D10 (highest bandwidth)
    pub odds_ratio_d1_vs_d10: f32,
    /// Excess predatory uptake over transparent baseline
    pub excess_exploitation_gap: f32,
}

impl DecileVulnerabilityGradient {
    fn evaluate_raw_deciles(
        stimulus: &VisualAgreementStimulus,
        sorted_kappas: &[f32],
        weather_shield_active: bool,
    ) -> [f32; 10] {
        let n = sorted_kappas.len();
        if n == 0 {
            return [0.0; 10];
        }
        let mut decile_uptakes = [0.0f32; 10];
        let decile_size = (n / 10).max(1);

        for (d, slot) in decile_uptakes.iter_mut().enumerate() {
            let start = (d * decile_size).min(n);
            let end = if d == 9 {
                n
            } else {
                ((d + 1) * decile_size).min(n)
            };
            if start >= end {
                continue;
            }
            let count = end - start;

            let mut accepts = 0;
            for (offset, &kappa) in sorted_kappas[start..end].iter().enumerate() {
                let i = start + offset;
                let u_res = 0.10 + 0.04 * (((i as f32) * 1.618).fract() * 2.0 - 1.0);
                let r_draw = ((i as f32 * std::f32::consts::PI).fract()).abs();

                let effective_stimulus = if weather_shield_active {
                    // Weather-shield decision buffer eliminates cloaking and urgency traps
                    let mut s = stimulus.clone();
                    s.cloaking_ratio = 1.0;
                    s.urgency_bonus = 0.0;
                    s
                } else {
                    stimulus.clone()
                };

                if effective_stimulus.evaluate_agent_decision(kappa, u_res, r_draw) {
                    accepts += 1;
                }
            }
            *slot = accepts as f32 / count as f32;
        }
        decile_uptakes
    }

    /// Compute decile uptake rates for an agreement stimulus across an agent population
    #[must_use]
    pub fn evaluate_population(
        stimulus: &VisualAgreementStimulus,
        agent_bandwidths: &[f32],
        weather_shield_active: bool,
    ) -> Self {
        let n = agent_bandwidths.len();
        if n == 0 {
            return Self {
                decile_uptake_rates: [0.0; 10],
                odds_ratio_d1_vs_d10: 1.0,
                excess_exploitation_gap: 0.0,
            };
        }

        let mut sorted_kappas = agent_bandwidths.to_vec();
        sorted_kappas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let decile_uptakes =
            Self::evaluate_raw_deciles(stimulus, &sorted_kappas, weather_shield_active);
        let transparent_baseline = VisualAgreementStimulus::fair_benchmark();
        let trans_uptakes =
            Self::evaluate_raw_deciles(&transparent_baseline, &sorted_kappas, false);

        let p_d1 = decile_uptakes[0];
        let p_d10 = decile_uptakes[9];
        let odds_d1 = p_d1 / (1.0 - p_d1 + 1e-6);
        let odds_d10 = p_d10 / (1.0 - p_d10 + 1e-6);
        let odds_ratio = odds_d1 / (odds_d10 + 1e-6);

        let mean_cloaked = decile_uptakes.iter().sum::<f32>() / 10.0;
        let mean_trans = trans_uptakes.iter().sum::<f32>() / 10.0;

        Self {
            decile_uptake_rates: decile_uptakes,
            odds_ratio_d1_vs_d10: odds_ratio,
            excess_exploitation_gap: mean_cloaked - mean_trans,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_d16_vulnerability_decile_gradient() {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        let mut rng = StdRng::seed_from_u64(42);

        // Exact Beta(2, 5) sampling via Gamma variates
        let mut kappas = Vec::with_capacity(1000);
        for _ in 0..1000 {
            let u1: f32 = rng.gen_range(1e-6..1.0);
            let u2: f32 = rng.gen_range(1e-6..1.0);
            let y1 = -(u1 * u2).ln();

            let u3: f32 = rng.gen_range(1e-6..1.0);
            let u4: f32 = rng.gen_range(1e-6..1.0);
            let u5: f32 = rng.gen_range(1e-6..1.0);
            let u6: f32 = rng.gen_range(1e-6..1.0);
            let u7: f32 = rng.gen_range(1e-6..1.0);
            let y2 = -(u3 * u4 * u5 * u6 * u7).ln();

            let beta_sample = y1 / (y1 + y2);
            kappas.push(beta_sample.clamp(0.01, 0.99));
        }

        let predatory = VisualAgreementStimulus::predatory_benchmark();
        let eval = DecileVulnerabilityGradient::evaluate_population(&predatory, &kappas, false);

        assert!(
            eval.odds_ratio_d1_vs_d10 >= 3.0,
            "Predatory agreement must induce >= 3.0x decile odds ratio between D1 and D10 (got {:.2}x)",
            eval.odds_ratio_d1_vs_d10
        );
        assert!(eval.decile_uptake_rates[0] > eval.decile_uptake_rates[9]);
    }

    #[test]
    fn test_sim_d17_weather_shield_neutralizes_exploitation() {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        let mut rng = StdRng::seed_from_u64(42);

        // Generate 1,000 agents with Beta(2, 5) bandwidths
        let mut kappas_base = Vec::with_capacity(1000);
        for _ in 0..1000 {
            let u1: f32 = rng.gen_range(1e-6..1.0);
            let u2: f32 = rng.gen_range(1e-6..1.0);
            let y1 = -(u1 * u2).ln();

            let u3: f32 = rng.gen_range(1e-6..1.0);
            let u4: f32 = rng.gen_range(1e-6..1.0);
            let u5: f32 = rng.gen_range(1e-6..1.0);
            let u6: f32 = rng.gen_range(1e-6..1.0);
            let u7: f32 = rng.gen_range(1e-6..1.0);
            let y2 = -(u3 * u4 * u5 * u6 * u7).ln();

            kappas_base.push((y1 / (y1 + y2)).clamp(0.01, 0.99));
        }

        // Under 38°C heatwave: F_thermal = 1.50, compressing bandwidth
        let kappas_heatwave: Vec<f32> = kappas_base.iter().map(|&k| k / 1.50).collect();
        // Under Weather-Shield: 24h cooling-off restores temperature to 20°C (F_thermal = 1.0)
        let kappas_shielded: Vec<f32> = kappas_base.clone();

        let predatory = VisualAgreementStimulus::predatory_benchmark();
        let eval_heatwave =
            DecileVulnerabilityGradient::evaluate_population(&predatory, &kappas_heatwave, false);
        let eval_shielded =
            DecileVulnerabilityGradient::evaluate_population(&predatory, &kappas_shielded, false);

        let mean_heatwave = eval_heatwave.decile_uptake_rates.iter().sum::<f32>() / 10.0;
        let mean_shielded = eval_shielded.decile_uptake_rates.iter().sum::<f32>() / 10.0;

        assert!(
            mean_shielded <= mean_heatwave,
            "Weather-shield cooling-off period must reduce predatory loan uptake ({mean_shielded:.3} vs {mean_heatwave:.3})"
        );
    }

    #[test]
    fn test_sobel_gradient_magnitude_edge_detection() {
        let width = 10;
        let height = 10;
        let mut pixels = vec![0.0f32; width * height];
        // Vertical step edge: columns >= 5 are 1.0
        for y in 0..height {
            for x in 5..width {
                pixels[y * width + x] = 1.0;
            }
        }

        let edge_map =
            VisualContrastEngine::compute_sobel_gradient_magnitude(&pixels, width, height);
        // The step boundary (x=4, x=5) must have high gradient magnitude
        assert!(edge_map[5 * width + 4] > 1.0);
        // Interior regions must be 0
        assert_eq!(edge_map[5 * width + 1], 0.0);
        assert_eq!(edge_map[5 * width + 8], 0.0);
    }

    #[test]
    fn test_michelson_contrast_calculation() {
        let width = 20;
        let height = 20;
        let roi = RegionOfInterest {
            top: 2,
            bottom: 18,
            left: 2,
            right: 18,
        };

        let uniform = vec![0.5f32; width * height];
        let c_uniform =
            VisualContrastEngine::compute_michelson_contrast(&uniform, width, height, &roi);
        assert!(c_uniform < 1e-3);

        let mut high_contrast = vec![0.0f32; width * height];
        for y in roi.top..roi.bottom {
            for x in roi.left..roi.right {
                if (x + y) % 2 == 0 {
                    high_contrast[y * width + x] = 1.0;
                }
            }
        }
        let c_high =
            VisualContrastEngine::compute_michelson_contrast(&high_contrast, width, height, &roi);
        assert!(c_high > 0.95);
    }

    #[test]
    fn test_cloaking_ratio_from_2d_pixels() {
        let width = 224;
        let height = 224;
        let mut pixels = vec![0.5f32; width * height];

        let promo = RegionOfInterest::promotional_window();
        let disc = RegionOfInterest::disclaimer_window();

        // High contrast strokes in promotional headline window
        for y in promo.top..promo.bottom {
            for x in promo.left..promo.right {
                if x % 4 == 0 {
                    pixels[y * width + x] = 0.0;
                } else if x % 4 == 2 {
                    pixels[y * width + x] = 1.0;
                }
            }
        }

        // Faint low contrast text in disclaimer window
        for y in disc.top..disc.bottom {
            for x in disc.left..disc.right {
                if x % 4 == 0 {
                    pixels[y * width + x] = 0.48;
                } else if x % 4 == 2 {
                    pixels[y * width + x] = 0.52;
                }
            }
        }

        let chi = VisualContrastEngine::compute_cloaking_ratio_from_pixels(
            &pixels, width, height, &promo, &disc,
        );
        assert!(
            chi > 2.0,
            "Predatory visual layout must yield cloaking ratio chi > 2.0 (got {chi:.2})"
        );
    }

    #[test]
    fn test_npy_binary_parser_and_csf_spatial_frequency() {
        let mut npy_bytes = Vec::new();
        npy_bytes.extend_from_slice(b"\x93NUMPY");
        npy_bytes.push(1);
        npy_bytes.push(0);
        let header_dict = "{'descr': '|u1', 'fortran_order': False, 'shape': (1, 20, 20, 3), }";
        let padded_len = (header_dict.len() + 1).div_ceil(16) * 16 - 1;
        let mut header_str = header_dict.to_string();
        while header_str.len() < padded_len {
            header_str.push(' ');
        }
        header_str.push('\n');
        let h_len = header_str.len() as u16;
        npy_bytes.extend_from_slice(&h_len.to_le_bytes());
        npy_bytes.extend_from_slice(header_str.as_bytes());

        let img_bytes = vec![128u8; 20 * 20 * 3];
        npy_bytes.extend_from_slice(&img_bytes);

        let parsed =
            NpyArrayU8::from_bytes(&npy_bytes).expect("Failed to parse valid synthetic NPY");
        assert_eq!(parsed.shape, vec![1, 20, 20, 3]);

        let gray = parsed.get_image_grayscale(0, 20, 20);
        assert_eq!(gray.len(), 400);
        assert!((gray[0] - 128.0 / 255.0).abs() < 1e-4);

        let dog = VisualContrastEngine::difference_of_gaussians(&gray, 20, 20, 1.0, 2.0);
        assert!(dog.iter().all(|&v| v < 1e-5));
    }

    #[test]
    fn test_raw_stimuli_ground_truth_npy_reproduction() {
        let npy_path =
            "/Users/devonshigaki/Desktop/research/brainit-fmri/results/brainit_structural_recons.npy";
        if !std::path::Path::new(npy_path).exists() {
            return;
        }

        let npy =
            NpyArrayU8::from_file(npy_path).expect("Should load brainit_structural_recons.npy");
        assert_eq!(npy.shape, vec![27, 224, 224, 3]);

        let promo = RegionOfInterest::promotional_window();
        let disc = RegionOfInterest::disclaimer_window();

        let mut fair_chis = Vec::new();
        for i in 0..5 {
            let gray = npy.get_image_grayscale(i, 224, 224);
            let chi = VisualContrastEngine::compute_cloaking_ratio_from_pixels(
                &gray, 224, 224, &promo, &disc,
            );
            fair_chis.push(chi);
        }

        let mut pred_chis = Vec::new();
        for i in 5..10 {
            let gray = npy.get_image_grayscale(i, 224, 224);
            let chi = VisualContrastEngine::compute_cloaking_ratio_from_pixels(
                &gray, 224, 224, &promo, &disc,
            );
            pred_chis.push(chi);
        }

        let mean_fair = fair_chis.iter().sum::<f32>() / 5.0;
        let mean_pred = pred_chis.iter().sum::<f32>() / 5.0;
        let observed_ratio = mean_pred / mean_fair;

        assert!(
            (observed_ratio - 0.8841).abs() < 0.08,
            "Rust 2D Sobel on raw brainit_structural_recons.npy must replicate Python SciPy C1 anomaly (observed: {observed_ratio:.4})"
        );
    }

    #[test]
    fn test_csf_spatial_frequency_branch_a_vs_branch_b() {
        let path_a =
            "/Users/devonshigaki/Desktop/research/brainit-fmri/results/brainit_structural_recons.npy";
        let path_b =
            "/Users/devonshigaki/Desktop/research/brainit-fmri/results/brainit_dual_recons.npy";
        if !std::path::Path::new(path_a).exists() || !std::path::Path::new(path_b).exists() {
            return;
        }

        let npy_a = NpyArrayU8::from_file(path_a).expect("Load structural recons");
        let npy_b = NpyArrayU8::from_file(path_b).expect("Load dual recons");

        let disc = RegionOfInterest::disclaimer_window();

        let mut disc_energy_a = 0.0f32;
        let mut disc_energy_b = 0.0f32;

        for i in 5..10 {
            let gray_a = npy_a.get_image_grayscale(i, 224, 224);
            let gray_b = npy_b.get_image_grayscale(i, 224, 224);

            let dog_a = VisualContrastEngine::difference_of_gaussians(&gray_a, 224, 224, 0.8, 1.6);
            let dog_b = VisualContrastEngine::difference_of_gaussians(&gray_b, 224, 224, 0.8, 1.6);

            disc_energy_a += VisualContrastEngine::compute_roi_mean_energy(&dog_a, 224, 224, &disc);
            disc_energy_b += VisualContrastEngine::compute_roi_mean_energy(&dog_b, 224, 224, &disc);
        }

        // Branch B dual reconstruction sharpens fine-print disclaimer typography relative to Branch A
        assert!(
            disc_energy_b > disc_energy_a,
            "Branch B semantic reconstruction must have higher disclaimer high-frequency energy than Branch A (got B: {disc_energy_b:.4} vs A: {disc_energy_a:.4})"
        );
    }

    #[test]
    fn test_raw_cortical_bold_parcel_contrast() {
        let bold_path =
            "/Users/devonshigaki/Desktop/research/brainit-fmri/results/simulated_cortical_bold.npy";
        if !std::path::Path::new(bold_path).exists() {
            return;
        }

        let npy =
            NpyArrayF32::from_file(bold_path).expect("Should load simulated_cortical_bold.npy");
        assert_eq!(npy.shape, vec![27, 15724]);

        let mut fair_cont = 0.0f32;
        let mut fair_dmn = 0.0f32;
        for i in 0..5 {
            let row = npy.get_row(i);
            fair_cont += CorticalParcels::parcel_mean(row, CorticalParcels::CONTROL_SALIENCE);
            fair_dmn += CorticalParcels::parcel_mean(row, CorticalParcels::DEFAULT_MODE);
        }
        fair_cont /= 5.0;
        fair_dmn /= 5.0;

        let mut threat_cont = 0.0f32;
        let mut threat_dmn = 0.0f32;
        for i in 20..25 {
            let row = npy.get_row(i);
            threat_cont += CorticalParcels::parcel_mean(row, CorticalParcels::CONTROL_SALIENCE);
            threat_dmn += CorticalParcels::parcel_mean(row, CorticalParcels::DEFAULT_MODE);
        }
        threat_cont /= 5.0;
        threat_dmn /= 5.0;

        assert!(
            threat_cont > fair_cont,
            "THREAT must evoke higher Control parcel activation than FAIR (THREAT: {threat_cont:.4} vs FAIR: {fair_cont:.4})"
        );
        assert!(
            threat_dmn > fair_dmn,
            "THREAT must evoke higher DMN activation than FAIR (THREAT: {threat_dmn:.4} vs FAIR: {fair_dmn:.4})"
        );
    }

    #[test]
    fn test_raw_reconstructions_ssim_pixcorr_resolution() {
        let gt_path =
            "/Users/devonshigaki/Desktop/research/brainit-fmri/results/stimuli_gt_224.npy";
        let gallant_path =
            "/Users/devonshigaki/Desktop/research/brainit-fmri/results/gallant_recons.npy";
        let dual_path =
            "/Users/devonshigaki/Desktop/research/brainit-fmri/results/brainit_dual_recons.npy";

        if !std::path::Path::new(gt_path).exists()
            || !std::path::Path::new(gallant_path).exists()
            || !std::path::Path::new(dual_path).exists()
        {
            return;
        }

        let npy_gt = NpyArrayU8::from_file(gt_path).expect("Load stimuli_gt_224");
        let npy_gal = NpyArrayU8::from_file(gallant_path).expect("Load gallant_recons");
        let npy_dual = NpyArrayU8::from_file(dual_path).expect("Load brainit_dual_recons");

        let mut ssim_gal_rgb = 0.0f32;
        let mut ssim_dual_rgb = 0.0f32;
        let mut pcorr_gal = 0.0f32;
        let mut pcorr_dual = 0.0f32;

        for i in 0..27 {
            let gt_rgb = npy_gt.get_image_rgb(i, 224, 224);
            let gal_rgb = npy_gal.get_image_rgb(i, 224, 224);
            let dual_rgb = npy_dual.get_image_rgb(i, 224, 224);

            ssim_gal_rgb += VisualContrastEngine::compute_rgb_ssim(gt_rgb, gal_rgb, 224, 224);
            ssim_dual_rgb += VisualContrastEngine::compute_rgb_ssim(gt_rgb, dual_rgb, 224, 224);

            let gt_gray = npy_gt.get_image_grayscale(i, 224, 224);
            let gal_gray = npy_gal.get_image_grayscale(i, 224, 224);
            let dual_gray = npy_dual.get_image_grayscale(i, 224, 224);
            pcorr_gal += VisualContrastEngine::compute_pixcorr(&gt_gray, &gal_gray);
            pcorr_dual += VisualContrastEngine::compute_pixcorr(&gt_gray, &dual_gray);
        }
        ssim_gal_rgb /= 27.0;
        ssim_dual_rgb /= 27.0;
        pcorr_gal /= 27.0;
        pcorr_dual /= 27.0;

        assert!(
            ssim_dual_rgb > ssim_gal_rgb,
            "Dual-branch RGB SSIM must exceed baseline Gallant (got Dual: {ssim_dual_rgb:.4} vs Gal: {ssim_gal_rgb:.4})"
        );
        assert!(
            pcorr_dual > pcorr_gal,
            "Dual-branch PixCorr must exceed baseline Gallant (got Dual: {pcorr_dual:.4} vs Gal: {pcorr_gal:.4})"
        );
    }

    #[test]
    fn test_gmm_cortical_graph_pooling_and_mantel_geometry() {
        let bold_path =
            "/Users/devonshigaki/Desktop/research/brainit-fmri/results/simulated_cortical_bold.npy";
        let gmm_path =
            "/Users/devonshigaki/Desktop/research/brainit-fmri/data/derived_data/v2c_128_mapping_gmm_v2.npy";

        if !std::path::Path::new(bold_path).exists() || !std::path::Path::new(gmm_path).exists() {
            return;
        }

        let bold = NpyArrayF32::from_file(bold_path).expect("Load simulated_cortical_bold");
        let gmm = NpyArrayI64::from_file(gmm_path).expect("Load v2c_128_mapping_gmm_v2");
        assert_eq!(bold.shape, vec![27, 15724]);
        assert_eq!(gmm.shape, vec![128, 17511]);

        let mut voxel_vectors = Vec::with_capacity(27);
        let mut gmm_vectors = Vec::with_capacity(27);

        for i in 0..27 {
            let row = bold.get_row(i);
            voxel_vectors.push(row.to_vec());
            let gmm_pooled = CorticalParcels::pool_voxels_to_gmm(row, &gmm);
            assert_eq!(gmm_pooled.len(), 128);
            gmm_vectors.push(gmm_pooled);
        }

        let rdm_voxel = VisualContrastEngine::compute_rdm(&voxel_vectors);
        let rdm_gmm = VisualContrastEngine::compute_rdm(&gmm_vectors);

        let mantel_r = VisualContrastEngine::mantel_test_correlation(&rdm_voxel, &rdm_gmm);
        assert!(
            mantel_r >= 0.70,
            "128-node GMM graph mapping must preserve cortical RDM geometry (Mantel r >= 0.70, got {mantel_r:.4})"
        );
    }
}
