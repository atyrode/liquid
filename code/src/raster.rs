use crate::particle::Vec2f64;

pub const DENSITY_FULL: f32 = 3.0;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GridSize {
    pub cols: usize,
    pub rows: usize,
}

pub struct DensityGrid {
    pub(crate) cols: usize,
    pub(crate) rows: usize,
    pub(crate) cells: Vec<f32>,
}

impl DensityGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: vec![0.0; cols * rows],
        }
    }

    pub fn size(&self) -> GridSize {
        GridSize {
            cols: self.cols,
            rows: self.rows,
        }
    }

    pub fn cells(&self) -> &[f32] {
        &self.cells
    }

    pub fn rasterize(&mut self, particles: &[Vec2f64], world_width: f64, world_height: f64) {
        self.cells.fill(0.0);

        for particle in particles {
            let col = project_axis(particle.x, world_width, self.cols);
            let row = self.rows - 1 - project_axis(particle.y, world_height, self.rows);

            self.add_density(col, row, 1.0);
            self.add_density(col.saturating_sub(1), row, 0.25);
            self.add_density(col + 1, row, 0.25);
            self.add_density(col, row.saturating_sub(1), 0.25);
            self.add_density(col, row + 1, 0.25);
        }
    }

    pub(crate) fn add_density(&mut self, col: usize, row: usize, amount: f32) {
        if col < self.cols && row < self.rows {
            self.cells[row * self.cols + col] += amount;
        }
    }
}

fn project_axis(value: f64, world_size: f64, cells: usize) -> usize {
    if cells == 1 {
        return 0;
    }

    let normalized = (value / world_size).clamp(0.0, 1.0);
    (normalized * (cells - 1) as f64).round() as usize
}
