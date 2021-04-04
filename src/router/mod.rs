pub mod binarytree;
pub mod sidewinder;

use crate::grid::{Cell, Grid};

pub trait Router {
    fn carve(&mut self, grid: &mut Grid, cells: Vec<Option<Cell>>);

    fn carve_by_cell(&mut self, grid: &mut Grid, cells: Vec<Option<Cell>>) {
        for cell in cells {
            if let Some(c) = cell {
                self.by_cell(grid, c);
            }
        }
    }

    fn by_cell(&mut self, _grid: &mut Grid, _cell: Cell) {}

    fn carve_by_row(&mut self, grid: &mut Grid, cells: Vec<Option<Cell>>) {
        for row in 0..grid.rows() {
            let start = (row * grid.columns()) as usize;
            let end = start + grid.columns() as usize;
            self.by_row(grid, &cells[start..end], row);
        }
    }

    fn by_row(&mut self, grid: &mut Grid, cells: &[Option<Cell>], _row: u32) {
        for cell in cells {
            if let Some(c) = cell {
                self.by_cell(grid, *c)
            }
        }
    }
}

pub struct NoOp {}

impl Router for NoOp {
    fn carve(&mut self, _grid: &mut Grid, _cells: Vec<Option<Cell>>) {}
}
