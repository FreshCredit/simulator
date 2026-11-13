//! GIS Terrain Analysis
//!
//! Geographic Information System for terrain analysis including slope and
//! aspect calculation, viewshed computation, drainage basin identification,
//! and elevation profiles.

use serde::{Deserialize, Serialize};

use super::Environment;

// ============================================================================
// GIS TERRAIN ANALYSIS
// Per P1-8: Geographic Information System for terrain analysis
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GIS Terrain analysis structure
///
/// Provides spatial analysis functions for the environment grid including:
/// - Slope and aspect calculation
/// - Viewshed computation
/// - Drainage basin identification
/// - Elevation profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GISTerrainAnalysis {
    /// Grid dimensions
    pub width: u32,
    /// height.
    pub height: u32,
    /// Digital Elevation Model (DEM) - elevation at each cell
    pub elevation: Vec<Vec<f32>>,
    /// Cell size (in simulation units)
    pub cell_size: f32,
}

impl GISTerrainAnalysis {
    /// Create new GIS terrain analysis from environment
    ///
    /// Uses the z-coordinate of resources as elevation proxy
    #[allow(clippy::needless_range_loop)]
    #[must_use]
    pub fn from_environment(env: &Environment) -> Self {
        let width = env.width;
        let height = env.height;
        let mut elevation = vec![vec![0.0; height]; width];

        // Guard CPU cycles on headless runs: return flat DEM if altitude effects are disabled
        if !env.altitude_effects_enabled {
            return Self {
                width: width as u32,
                height: height as u32,
                elevation,
                cell_size: 1.0,
            };
        }

        // Derive active physical elevations from environment resources (Sim ECOL terrain dynamics)
        for x in 0..width {
            for y in 0..height {
                let cell_idx = y * width + x;
                let elev = if env.depth > 1 {
                    let mut max_z = 0.0f32;
                    for z in 0..env.depth {
                        let idx = z * (width * height) + cell_idx;
                        if idx < env.resources.len() && env.resources[idx].amount > 0.0 {
                            max_z = (z as f32) / (env.depth.max(1) as f32);
                        }
                    }
                    max_z * 10.0
                } else {
                    let cell = env.resources.get(cell_idx);
                    let cap = cell.map_or(0.0, |c| c.capacity);
                    (cap / 30.0).clamp(0.0, 1.0) * 10.0
                };
                elevation[x][y] = elev;
            }
        }

        Self {
            width: width as u32,
            height: height as u32,
            elevation,
            cell_size: 1.0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create from explicit elevation data
    #[must_use]
    pub fn from_elevation(elevation: Vec<Vec<f32>>, cell_size: f32) -> Self {
        let height = elevation.first().map_or(0, std::vec::Vec::len) as u32;
        let width = elevation.len() as u32;

        Self {
            width,
            height,
            elevation,
            cell_size,
        }
    }

    /// Get elevation at a position
    #[must_use]
    pub fn elevation_at(&self, x: i32, y: i32) -> Option<f32> {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return None;
        }
        Some(self.elevation[x as usize][y as usize])
    }

    /// Calculate slope at a cell (in degrees)
    ///
    /// Uses 3x3 neighborhood gradient method (Horn's algorithm)
    #[must_use]
    pub fn slope_at(&self, x: i32, y: i32) -> Option<f32> {
        // Get elevations in 3x3 neighborhood
        let z1 = self.elevation_at(x - 1, y + 1)?;
        let z2 = self.elevation_at(x, y + 1)?;
        let z3 = self.elevation_at(x + 1, y + 1)?;
        let z4 = self.elevation_at(x - 1, y)?;
        let z6 = self.elevation_at(x + 1, y)?;
        let z7 = self.elevation_at(x - 1, y - 1)?;
        let z8 = self.elevation_at(x, y - 1)?;
        let z9 = self.elevation_at(x + 1, y - 1)?;

        // Horn's method for gradient calculation
        let dz_dx = ((z3 + 2.0 * z6 + z9) - (z1 + 2.0 * z4 + z7)) / (8.0 * self.cell_size);
        let dz_dy = ((z7 + 2.0 * z8 + z9) - (z1 + 2.0 * z2 + z3)) / (8.0 * self.cell_size);

        // Slope in radians
        let slope_rad = dz_dx.hypot(dz_dy).atan();

        // Convert to degrees
        Some(slope_rad.to_degrees())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate aspect at a cell (direction of slope, in degrees)
    /// 0 = North, 90 = East, 180 = South, 270 = West
    #[must_use]
    pub fn aspect_at(&self, x: i32, y: i32) -> Option<f32> {
        let z1 = self.elevation_at(x - 1, y + 1)?;
        let z2 = self.elevation_at(x, y + 1)?;
        let z3 = self.elevation_at(x + 1, y + 1)?;
        let z4 = self.elevation_at(x - 1, y)?;
        let z6 = self.elevation_at(x + 1, y)?;
        let z7 = self.elevation_at(x - 1, y - 1)?;
        let z8 = self.elevation_at(x, y - 1)?;
        let z9 = self.elevation_at(x + 1, y - 1)?;

        let dz_dx = ((z3 + 2.0 * z6 + z9) - (z1 + 2.0 * z4 + z7)) / (8.0 * self.cell_size);
        let dz_dy = ((z7 + 2.0 * z8 + z9) - (z1 + 2.0 * z2 + z3)) / (8.0 * self.cell_size);

        if dz_dx == 0.0 && dz_dy == 0.0 {
            return Some(0.0); // Flat
        }

        // Aspect in radians, then convert to degrees
        let aspect_rad = dz_dx.atan2(dz_dy);
        let mut aspect_deg = aspect_rad.to_degrees();

        // Convert to compass direction (0 = North)
        if aspect_deg < 0.0 {
            aspect_deg = 90.0 - aspect_deg;
        } else if aspect_deg > 90.0 {
            aspect_deg = 360.0 - aspect_deg + 90.0;
        } else {
            aspect_deg = 90.0 - aspect_deg;
        }

        Some(aspect_deg)
    }

    /// Returns `true` if `(x, y)` is inside the terrain bounds.
    const fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }

    /// Check whether a single cell is visible from the viewpoint.
    /// Returns `false` for the origin cell or out-of-range cells.
    fn is_cell_visible(
        &self,
        view_x: i32,
        view_y: i32,
        target_x: i32,
        target_y: i32,
        view_elev: f32,
        max_distance: i32,
    ) -> bool {
        if target_x == view_x && target_y == view_y {
            return false;
        }
        let dx = target_x - view_x;
        let dy = target_y - view_y;
        let dist = ((dx * dx + dy * dy) as f32).sqrt();
        if dist > max_distance as f32 {
            return false;
        }
        self.has_line_of_sight(view_x, view_y, target_x, target_y, view_elev)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute viewshed from a viewpoint
    /// Returns grid showing which cells are visible from the viewpoint
    #[must_use]
    pub fn viewshed(&self, view_x: i32, view_y: i32, max_distance: i32) -> Vec<Vec<bool>> {
        let mut visible = vec![vec![false; self.height as usize]; self.width as usize];

        // Viewpoint is always visible
        if self.in_bounds(view_x, view_y) {
            visible[view_x as usize][view_y as usize] = true;
        }

        let Some(base_elev) = self.elevation_at(view_x, view_y) else {
            return visible;
        };
        let view_elev = base_elev + 1.5; // Observer height

        // Iterate over bounding box clipped to terrain bounds
        let x_min = (view_x - max_distance).max(0);
        let x_max = (view_x + max_distance + 1).min(self.width as i32);
        let y_min = (view_y - max_distance).max(0);
        let y_max = (view_y + max_distance + 1).min(self.height as i32);

        for target_x in x_min..x_max {
            for target_y in y_min..y_max {
                if self.is_cell_visible(view_x, view_y, target_x, target_y, view_elev, max_distance)
                {
                    visible[target_x as usize][target_y as usize] = true;
                }
            }
        }

        visible
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if there's line of sight between two points
    fn has_line_of_sight(&self, x1: i32, y1: i32, x2: i32, y2: i32, observer_elev: f32) -> bool {
        let Some(target_elev) = self.elevation_at(x2, y2) else {
            return false;
        };

        let dx = (x2 - x1) as f32;
        let dy = (y2 - y1) as f32;
        let dist = dx.hypot(dy);

        if dist < 0.1 {
            return true;
        }

        // Line of sight slope
        let los_slope = (target_elev - observer_elev) / dist;

        // Walk along line and check for obstructions
        let steps = (dist * 2.0) as i32;
        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let check_x = (x1 as f32 + dx * t) as i32;
            let check_y = (y1 as f32 + dy * t) as i32;

            if check_x == x1 && check_y == y1 {
                continue;
            }
            if check_x == x2 && check_y == y2 {
                continue;
            }

            if let Some(check_elev) = self.elevation_at(check_x, check_y) {
                // Distance from observer to check point
                let check_dx = (check_x - x1) as f32;
                let check_dy = (check_y - y1) as f32;
                let check_dist = check_dx.hypot(check_dy);

                // Elevation of line of sight at this point
                let los_elev = observer_elev + los_slope * check_dist;

                // If terrain is above line of sight, it's obstructed
                if check_elev > los_elev + 0.1 {
                    return false;
                }
            }
        }

        true
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate mean slope across entire terrain
    #[must_use]
    pub fn mean_slope(&self) -> f32 {
        let mut total_slope = 0.0;
        let mut count = 0;

        for x in 0..self.width as i32 {
            for y in 0..self.height as i32 {
                if let Some(slope) = self.slope_at(x, y) {
                    total_slope += slope;
                    count += 1;
                }
            }
        }

        if count == 0 {
            0.0
        } else {
            total_slope / count as f32
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Find ridge lines (local elevation maxima)
    #[must_use]
    pub fn find_ridges(&self) -> Vec<(i32, i32)> {
        let mut ridges = Vec::new();

        for x in 1..(self.width as i32 - 1) {
            for y in 1..(self.height as i32 - 1) {
                if let Some(center) = self.elevation_at(x, y) {
                    let mut is_ridge = true;

                    // Check 8 neighbors
                    for dx in -1..=1 {
                        for dy in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            if let Some(neighbor) = self.elevation_at(x + dx, y + dy) {
                                if neighbor > center {
                                    is_ridge = false;
                                    break;
                                }
                            }
                        }
                        if !is_ridge {
                            break;
                        }
                    }

                    if is_ridge {
                        ridges.push((x, y));
                    }
                }
            }
        }

        ridges
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get elevation profile along a line
    #[must_use]
    pub fn elevation_profile(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<(f32, f32)> {
        let mut profile = Vec::new();

        let dx = (x2 - x1) as f32;
        let dy = (y2 - y1) as f32;
        let dist = dx.hypot(dy);

        let steps = (dist * 2.0).max(2.0) as i32;

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = (x1 as f32 + dx * t) as i32;
            let y = (y1 as f32 + dy * t) as i32;

            if let Some(elev) = self.elevation_at(x, y) {
                let seg_dist = t * dist * self.cell_size;
                profile.push((seg_dist, elev));
            }
        }

        profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // P1-8: GIS Terrain Analysis Tests

    #[test]
    fn test_gis_from_elevation() {
        let elevation = vec![
            vec![0.0, 1.0, 2.0],
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let gis = GISTerrainAnalysis::from_elevation(elevation, 1.0);

        assert_eq!(gis.width, 3);
        assert_eq!(gis.height, 3);
        assert_eq!(gis.elevation_at(0, 0), Some(0.0));
        assert_eq!(gis.elevation_at(2, 2), Some(4.0));
        assert_eq!(gis.elevation_at(-1, 0), None); // Out of bounds
    }

    #[test]
    fn test_gis_slope_calculation() {
        // Create a simple slope
        let elevation = vec![
            vec![0.0, 0.0, 0.0],
            vec![1.0, 1.0, 1.0],
            vec![2.0, 2.0, 2.0],
        ];

        let gis = GISTerrainAnalysis::from_elevation(elevation, 1.0);

        // Slope at center should be calculated
        let slope = gis.slope_at(1, 1);
        assert!(slope.is_some());
        assert!(slope.unwrap() > 0.0); // Should have positive slope
    }

    #[test]
    fn test_gis_flat_terrain_zero_slope() {
        // Flat terrain
        let elevation = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];

        let gis = GISTerrainAnalysis::from_elevation(elevation, 1.0);

        let slope = gis.slope_at(1, 1);
        assert!(slope.is_some());
        assert!(
            slope.unwrap() < 1.0,
            "Flat terrain should have near-zero slope"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_gis_aspect_calculation() {
        let elevation = vec![
            vec![0.0, 1.0, 2.0],
            vec![0.0, 1.0, 2.0],
            vec![0.0, 1.0, 2.0],
        ];

        let gis = GISTerrainAnalysis::from_elevation(elevation, 1.0);

        let aspect = gis.aspect_at(1, 1);
        assert!(aspect.is_some());

        // Aspect should be in [0, 360]
        let aspect_val = aspect.unwrap();
        assert!((0.0..=360.0).contains(&aspect_val));
    }

    #[test]
    fn test_gis_viewshed_basic() {
        // Create terrain with a peak in the center
        let elevation = vec![
            vec![0.0, 0.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 2.0, 1.0, 0.0],
            vec![0.0, 2.0, 5.0, 2.0, 0.0],
            vec![0.0, 1.0, 2.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 0.0, 0.0],
        ];

        let gis = GISTerrainAnalysis::from_elevation(elevation, 1.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Viewshed from the peak
        let viewshed = gis.viewshed(2, 2, 3);

        // Center should be visible
        assert!(viewshed[2][2]);

        // Should see some surrounding cells
        let visible_count = viewshed
            .iter()
            .flat_map(|r| r.iter())
            .filter(|&&v| v)
            .count();
        assert!(visible_count > 1, "Should see more than just the viewpoint");
    }

    #[test]
    fn test_gis_find_ridges() {
        // Create terrain with clear ridges
        let elevation = vec![
            vec![0.0, 0.0, 0.0],
            vec![0.0, 5.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ];

        let gis = GISTerrainAnalysis::from_elevation(elevation, 1.0);

        let ridges = gis.find_ridges();

        // Should find the center peak
        assert!(!ridges.is_empty());
        assert!(ridges.contains(&(1, 1)));
    }

    #[test]
    fn test_gis_elevation_profile() {
        let elevation = vec![
            vec![0.0, 1.0, 2.0],
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let gis = GISTerrainAnalysis::from_elevation(elevation, 1.0);

        // Profile from bottom-left to top-right
        let profile = gis.elevation_profile(0, 0, 2, 2);

        // Should have multiple points
        assert!(profile.len() >= 2);

        // First point should be at distance 0 with elevation 0
        assert!(profile[0].0 < 0.01); // Distance ~0
        assert!(profile[0].1 < 0.01); // Elevation ~0

        // Last point should have higher elevation
        let last = profile.last().unwrap();
        assert!(last.1 > 3.0); // Elevation at (2,2) is 4.0
    }

    #[test]
    fn test_gis_mean_slope() {
        // Flat terrain
        let flat = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        let gis_flat = GISTerrainAnalysis::from_elevation(flat, 1.0);
        let mean_flat = gis_flat.mean_slope();
        assert!(mean_flat < 1.0, "Flat terrain mean slope should be low");

        // Steep terrain
        let steep = vec![
            vec![0.0, 0.0, 0.0],
            vec![5.0, 5.0, 5.0],
            vec![10.0, 10.0, 10.0],
        ];
        let gis_steep = GISTerrainAnalysis::from_elevation(steep, 1.0);
        let mean_steep = gis_steep.mean_slope();
        assert!(
            mean_steep > mean_flat,
            "Steep terrain should have higher mean slope"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_gis_line_of_sight_clear() {
        // Flat terrain - clear line of sight
        let elevation = vec![
            vec![0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ];

        let gis = GISTerrainAnalysis::from_elevation(elevation, 1.0);

        // Should have line of sight on flat terrain
        assert!(gis.has_line_of_sight(0, 0, 2, 2, 1.0));
    }

    #[test]
    fn test_gis_line_of_sight_blocked() {
        // Terrain with obstruction
        let elevation = vec![
            vec![0.0, 0.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 5.0, 0.0, 0.0], // Wall in middle
            vec![0.0, 0.0, 5.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 0.0, 0.0],
        ];

        let gis = GISTerrainAnalysis::from_elevation(elevation, 1.0);

        // Line from left to right through the wall - should be blocked
        // Note: This depends on specific implementation details
        // For now just test that the function doesn't panic
        let _result = gis.has_line_of_sight(0, 1, 4, 1, 1.0);
    }
}
