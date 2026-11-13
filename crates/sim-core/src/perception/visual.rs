//! Visual Perception Module (P1-AGENT-1)
//!
//! Visual field processing and object recognition.

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Visual perception from the environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualPerception {
    /// objects.
    pub objects: Vec<VisualObject>,
    /// salience.
    pub salience: f32,
    /// field width.
    pub field_width: u32,
    /// field height.
    pub field_height: u32,
}

/// Object detected in visual field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualObject {
    /// id.
    pub id: u64,
    /// object type.
    pub object_type: ObjectType,
    /// position.
    pub position: (f32, f32), // x, y in visual field (0-1)
    /// size.
    pub size: f32, // relative size (0-1)
    /// color.
    pub color: Color,
    /// motion.
    pub motion: Option<MotionVector>,
    /// confidence.
    pub confidence: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Object type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectType {
    /// agent.
    Agent,
    /// obstacle.
    Obstacle,
    /// resource.
    Resource,
    /// threat.
    Threat,
    /// landmark.
    Landmark,
    /// unknown.
    Unknown,
}

/// Color representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    /// r.
    pub r: u8,
    /// g.
    pub g: u8,
    /// b.
    pub b: u8,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Color {
    #[must_use]
    /// new.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert to grayscale
    #[must_use]
    pub fn luminance(&self) -> f32 {
        0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32
    }

    /// Distance to another color (Euclidean)
    #[must_use]
    pub fn distance(&self, other: &Self) -> f32 {
        let dr = (self.r as f32 - other.r as f32).powi(2);
        let dg = (self.g as f32 - other.g as f32).powi(2);
        let db = (self.b as f32 - other.b as f32).powi(2);
        (dr + dg + db).sqrt()
    }
}

/// Motion vector for tracking
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MotionVector {
    /// dx.
    pub dx: f32,
    /// dy.
    pub dy: f32,
    /// speed.
    pub speed: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Visual field representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualField {
    /// width.
    pub width: u32,
    /// height.
    pub height: u32,
    /// pixels.
    pub pixels: Vec<Color>,
    /// depth map.
    pub depth_map: Option<Vec<f32>>,
}

impl VisualField {
    /// Create new visual field
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        Self {
            width,
            height,
            pixels: vec![Color::new(0, 0, 0); pixel_count],
            depth_map: Some(vec![0.0; pixel_count]),
        }
    }

    /// Get pixel at position
    #[must_use]
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.pixels.get((y * self.width + x) as usize)
    }

    /// Set pixel at position
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            if idx < self.pixels.len() {
                self.pixels[idx] = color;
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Detect edges using simple Sobel operator
    #[must_use]
    pub fn detect_edges(&self) -> Vec<(u32, u32)> {
        let mut edges = Vec::new();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let gx = self.sobel_x(x, y);
                let gy = self.sobel_y(x, y);
                let magnitude = gx.hypot(gy);

                if magnitude.is_finite() && magnitude > 100.0 {
                    edges.push((x, y));
                }
            }
        }

        edges
    }

    /// Sobel X gradient
    fn sobel_x(&self, x: u32, y: u32) -> f32 {
        let p00 = self.get_pixel(x - 1, y - 1).map_or(0.0, Color::luminance);
        let p01 = self.get_pixel(x, y - 1).map_or(0.0, Color::luminance);
        let p02 = self.get_pixel(x + 1, y - 1).map_or(0.0, Color::luminance);
        let p20 = self.get_pixel(x - 1, y + 1).map_or(0.0, Color::luminance);
        let p21 = self.get_pixel(x, y + 1).map_or(0.0, Color::luminance);
        let p22 = self.get_pixel(x + 1, y + 1).map_or(0.0, Color::luminance);

        (p02 + 2.0 * p01 + p00) - (p22 + 2.0 * p21 + p20)
    }

    /// Sobel Y gradient
    fn sobel_y(&self, x: u32, y: u32) -> f32 {
        let p00 = self.get_pixel(x - 1, y - 1).map_or(0.0, Color::luminance);
        let p10 = self.get_pixel(x - 1, y).map_or(0.0, Color::luminance);
        let p20 = self.get_pixel(x - 1, y + 1).map_or(0.0, Color::luminance);
        let p02 = self.get_pixel(x + 1, y - 1).map_or(0.0, Color::luminance);
        let p12 = self.get_pixel(x + 1, y).map_or(0.0, Color::luminance);
        let p22 = self.get_pixel(x + 1, y + 1).map_or(0.0, Color::luminance);

        (p00 + 2.0 * p10 + p20) - (p02 + 2.0 * p12 + p22)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Downsample to lower resolution
    #[must_use]
    pub fn downsample(&self, factor: u32) -> Self {
        let new_width = self.width / factor;
        let new_height = self.height / factor;
        let mut result = Self::new(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = x * factor;
                let src_y = y * factor;
                if let Some(&color) = self.get_pixel(src_x, src_y) {
                    result.set_pixel(x, y, color);
                }
            }
        }

        result
    }
}

/// Visual processor for agents
#[derive(Debug)]
pub struct VisualProcessor {
    /// config.
    pub config: VisualConfig,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Visual processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualConfig {
    /// resolution.
    pub resolution: (u32, u32),
    /// focal length.
    pub focal_length: f32,
    /// depth perception.
    pub depth_perception: bool,
    /// motion tracking.
    pub motion_tracking: bool,
    /// color sensitivity.
    pub color_sensitivity: f32,
}

impl Default for VisualConfig {
    fn default() -> Self {
        Self {
            resolution: (320, 240),
            focal_length: 50.0,
            depth_perception: true,
            motion_tracking: true,
            color_sensitivity: 1.0,
        }
    }
}

impl VisualProcessor {
    /// Create new visual processor
    #[must_use]
    pub const fn new(config: VisualConfig) -> Self {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Self { config }
    }

    /// Process visual field into perception
    #[must_use]
    pub fn process(&self, field: &VisualField) -> VisualPerception {
        let objects = self.detect_objects(field);
        let salience = self.calculate_salience(&objects);

        VisualPerception {
            objects,
            salience,
            field_width: field.width,
            field_height: field.height,
        }
    }

    /// Detect objects in visual field
    const fn detect_objects(&self, _field: &VisualField) -> Vec<VisualObject> {
        // Simplified object detection - would use ML in production
        vec![]
    }

    /// Calculate overall salience
    fn calculate_salience(&self, objects: &[VisualObject]) -> f32 {
        if objects.is_empty() {
            return 0.0;
        }
        objects.iter().map(|o| o.confidence).sum::<f32>() / objects.len() as f32
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Track motion between frames
    #[must_use]
    pub fn track_motion(&self, prev: &VisualField, curr: &VisualField) -> Vec<MotionVector> {
        if prev.width != curr.width || prev.height != curr.height {
            return vec![];
        }

        let mut motion = Vec::new();
        let block_size = 16;

        for y in (0..prev.height).step_by(block_size) {
            for x in (0..prev.width).step_by(block_size) {
                if let Some(mv) = self.estimate_block_motion(prev, curr, x, y, block_size) {
                    motion.push(mv);
                }
            }
        }

        motion
    }

    /// Estimate motion for a block
    const fn estimate_block_motion(
        &self,
        _prev: &VisualField,
        _curr: &VisualField,
        _x: u32,
        _y: u32,
        _block_size: usize,
    ) -> Option<MotionVector> {
        // Simplified motion estimation
        Some(MotionVector {
            dx: 0.0,
            dy: 0.0,
            speed: 0.0,
        })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for VisualProcessor {
    fn default() -> Self {
        Self::new(VisualConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_distance() {
        let c1 = Color::new(255, 0, 0);
        let c2 = Color::new(0, 255, 0);
        let distance = c1.distance(&c2);
        assert!(distance > 0.0);
    }

    #[test]
    fn test_visual_field() {
        let mut field = VisualField::new(100, 100);
        field.set_pixel(50, 50, Color::new(255, 255, 255));

        let pixel = field.get_pixel(50, 50);
        assert!(pixel.is_some());
        assert_eq!(pixel.unwrap().r, 255);
    }

    #[test]
    fn test_downsample() {
        let field = VisualField::new(100, 100);
        let downsampled = field.downsample(2);
        assert_eq!(downsampled.width, 50);
        assert_eq!(downsampled.height, 50);
    }
}
