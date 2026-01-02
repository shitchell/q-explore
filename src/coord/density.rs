//! Grid-based density analysis
//!
//! Divides a circular area into a grid and counts points per cell,
//! then calculates z-scores for anomaly detection.

use crate::coord::Coordinates;
use std::f64::consts::PI;

/// Default grid resolution (50x50 cells covering the bounding box)
pub const DEFAULT_GRID_RESOLUTION: usize = 50;

/// A density grid covering a circular area
#[derive(Debug)]
pub struct DensityGrid {
    /// Number of cells in each dimension
    pub resolution: usize,
    /// Center of the circle
    pub center: Coordinates,
    /// Radius in meters
    pub radius: f64,
    /// Point counts per cell [row][col]
    pub cells: Vec<Vec<usize>>,
    /// Which cells are within the circle
    pub in_circle: Vec<Vec<bool>>,
    /// Total number of points added
    pub total_points: usize,
    /// Size of each cell in meters
    pub cell_size: f64,
}

impl DensityGrid {
    /// Create a new density grid
    ///
    /// # Arguments
    /// * `center` - Center of the circle
    /// * `radius` - Radius in meters
    /// * `resolution` - Number of cells in each dimension
    pub fn new(center: Coordinates, radius: f64, resolution: usize) -> Self {
        let cell_size = (2.0 * radius) / resolution as f64;

        // Pre-compute which cells are in the circle
        let center_cell = resolution as f64 / 2.0;
        let mut in_circle = vec![vec![false; resolution]; resolution];

        for row in 0..resolution {
            for col in 0..resolution {
                let dx = col as f64 + 0.5 - center_cell;
                let dy = row as f64 + 0.5 - center_cell;
                let dist_squared = dx * dx + dy * dy;
                let max_dist = center_cell;
                in_circle[row][col] = dist_squared <= max_dist * max_dist;
            }
        }

        Self {
            resolution,
            center,
            radius,
            cells: vec![vec![0; resolution]; resolution],
            in_circle,
            total_points: 0,
            cell_size,
        }
    }

    /// Add points to the grid
    pub fn add_points(&mut self, points: &[Coordinates]) {
        const METERS_PER_DEG_LAT: f64 = 111_320.0;
        let meters_per_deg_lng = METERS_PER_DEG_LAT * (self.center.lat * PI / 180.0).cos();

        for point in points {
            // Convert to meters offset from center
            let dx_meters = (point.lng - self.center.lng) * meters_per_deg_lng;
            let dy_meters = (point.lat - self.center.lat) * METERS_PER_DEG_LAT;

            // Convert to grid cell
            let col = ((dx_meters + self.radius) / self.cell_size) as isize;
            let row = ((dy_meters + self.radius) / self.cell_size) as isize;

            // Bounds check
            if col >= 0
                && col < self.resolution as isize
                && row >= 0
                && row < self.resolution as isize
            {
                let col = col as usize;
                let row = row as usize;
                if self.in_circle[row][col] {
                    self.cells[row][col] += 1;
                    self.total_points += 1;
                }
            }
        }
    }

    /// Count how many cells are inside the circle
    pub fn cells_in_circle(&self) -> usize {
        self.in_circle
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&v| v)
            .count()
    }

    /// Calculate z-scores for each cell
    ///
    /// Z-score = (observed - expected) / sqrt(expected)
    /// Uses Poisson approximation for count data.
    pub fn calculate_z_scores(&self) -> Vec<Vec<Option<f64>>> {
        let cells_in_circle = self.cells_in_circle();
        if cells_in_circle == 0 || self.total_points == 0 {
            return vec![vec![None; self.resolution]; self.resolution];
        }

        let expected = self.total_points as f64 / cells_in_circle as f64;
        let std_dev = expected.sqrt();

        let mut scores = vec![vec![None; self.resolution]; self.resolution];

        for row in 0..self.resolution {
            for col in 0..self.resolution {
                if self.in_circle[row][col] {
                    let observed = self.cells[row][col] as f64;
                    scores[row][col] = Some((observed - expected) / std_dev);
                }
            }
        }

        scores
    }

    /// Convert a grid cell back to coordinates (center of cell)
    pub fn cell_to_coords(&self, row: usize, col: usize) -> Coordinates {
        const METERS_PER_DEG_LAT: f64 = 111_320.0;
        let meters_per_deg_lng = METERS_PER_DEG_LAT * (self.center.lat * PI / 180.0).cos();

        // Cell center in grid space
        let cell_center_x = (col as f64 + 0.5) * self.cell_size - self.radius;
        let cell_center_y = (row as f64 + 0.5) * self.cell_size - self.radius;

        // Convert to lat/lng
        let lat = self.center.lat + cell_center_y / METERS_PER_DEG_LAT;
        let lng = self.center.lng + cell_center_x / meters_per_deg_lng;

        Coordinates::new(lat, lng)
    }
}

/// Result of a density cell analysis
#[derive(Debug, Clone)]
pub struct CellResult {
    /// Grid row
    pub row: usize,
    /// Grid column
    pub col: usize,
    /// Point count in this cell
    pub count: usize,
    /// Z-score (how many std devs from expected)
    pub z_score: f64,
    /// Center coordinates of this cell
    pub coords: Coordinates,
}

/// Find the cell with the highest z-score (most points relative to expected)
pub fn find_densest_cell(grid: &DensityGrid) -> Option<CellResult> {
    let scores = grid.calculate_z_scores();
    let mut best: Option<CellResult> = None;

    for row in 0..grid.resolution {
        for col in 0..grid.resolution {
            if let Some(z_score) = scores[row][col] {
                let dominated = best.as_ref().is_some_and(|b| b.z_score >= z_score);
                if !dominated {
                    best = Some(CellResult {
                        row,
                        col,
                        count: grid.cells[row][col],
                        z_score,
                        coords: grid.cell_to_coords(row, col),
                    });
                }
            }
        }
    }

    best
}

/// Find the cell with the lowest z-score (fewest points relative to expected)
pub fn find_emptiest_cell(grid: &DensityGrid) -> Option<CellResult> {
    let scores = grid.calculate_z_scores();
    let mut best: Option<CellResult> = None;

    for row in 0..grid.resolution {
        for col in 0..grid.resolution {
            if let Some(z_score) = scores[row][col] {
                let dominated = best.as_ref().is_some_and(|b| b.z_score <= z_score);
                if !dominated {
                    best = Some(CellResult {
                        row,
                        col,
                        count: grid.cells[row][col],
                        z_score,
                        coords: grid.cell_to_coords(row, col),
                    });
                }
            }
        }
    }

    best
}

/// Find the cell with the highest absolute z-score (most anomalous either way)
pub fn find_most_anomalous_cell(grid: &DensityGrid) -> Option<(CellResult, bool)> {
    let scores = grid.calculate_z_scores();
    let mut best: Option<CellResult> = None;

    for row in 0..grid.resolution {
        for col in 0..grid.resolution {
            if let Some(z_score) = scores[row][col] {
                let dominated = best.as_ref().is_some_and(|b| b.z_score.abs() >= z_score.abs());
                if !dominated {
                    best = Some(CellResult {
                        row,
                        col,
                        count: grid.cells[row][col],
                        z_score,
                        coords: grid.cell_to_coords(row, col),
                    });
                }
            }
        }
    }

    best.map(|r| {
        let is_attractor = r.z_score > 0.0;
        (r, is_attractor)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::point::generate_points_in_circle;
    use crate::qrng::pseudo::SeededPseudoBackend;

    #[test]
    fn test_grid_creation() {
        let center = Coordinates::new(40.7128, -74.0060);
        let grid = DensityGrid::new(center, 1000.0, 50);

        assert_eq!(grid.resolution, 50);
        assert_eq!(grid.cells.len(), 50);
        assert_eq!(grid.cells[0].len(), 50);

        // Check that center cell is in circle
        assert!(grid.in_circle[25][25]);

        // Check that corners are not in circle
        assert!(!grid.in_circle[0][0]);
        assert!(!grid.in_circle[0][49]);
        assert!(!grid.in_circle[49][0]);
        assert!(!grid.in_circle[49][49]);
    }

    #[test]
    fn test_add_points() {
        let center = Coordinates::new(40.7128, -74.0060);
        let mut grid = DensityGrid::new(center, 1000.0, 50);

        // Add the center point
        grid.add_points(&[center]);
        assert_eq!(grid.total_points, 1);

        // Center cell should have 1 point
        assert_eq!(grid.cells[25][25], 1);
    }

    #[test]
    fn test_z_scores() {
        let center = Coordinates::new(40.7128, -74.0060);
        let backend = SeededPseudoBackend::new(42);
        let points = generate_points_in_circle(center, 1000.0, 10000, &backend).unwrap();

        let mut grid = DensityGrid::new(center, 1000.0, 50);
        grid.add_points(&points);

        let scores = grid.calculate_z_scores();

        // All cells in circle should have scores
        for row in 0..50 {
            for col in 0..50 {
                if grid.in_circle[row][col] {
                    assert!(scores[row][col].is_some());
                } else {
                    assert!(scores[row][col].is_none());
                }
            }
        }
    }

    #[test]
    fn test_find_densest_and_emptiest() {
        let center = Coordinates::new(40.7128, -74.0060);
        let backend = SeededPseudoBackend::new(42);
        let points = generate_points_in_circle(center, 1000.0, 10000, &backend).unwrap();

        let mut grid = DensityGrid::new(center, 1000.0, 50);
        grid.add_points(&points);

        let densest = find_densest_cell(&grid).unwrap();
        let emptiest = find_emptiest_cell(&grid).unwrap();

        // Densest should have positive z-score
        assert!(densest.z_score > 0.0);

        // Emptiest should have negative z-score
        assert!(emptiest.z_score < 0.0);

        // They shouldn't be the same cell
        assert!(densest.row != emptiest.row || densest.col != emptiest.col);
    }

    #[test]
    fn test_cell_to_coords() {
        let center = Coordinates::new(40.7128, -74.0060);
        let grid = DensityGrid::new(center, 1000.0, 50);

        // Center cell should be close to center coordinates
        let center_coords = grid.cell_to_coords(25, 25);
        assert!((center_coords.lat - center.lat).abs() < 0.001);
        assert!((center_coords.lng - center.lng).abs() < 0.001);
    }
}
