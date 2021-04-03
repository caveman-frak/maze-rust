pub mod binarytree;

use super::grid::{Cell, Grid};

pub trait Router {
    fn carve(&mut self, grid: &mut Grid, cells: Vec<Option<Cell>>);
}

pub struct NoOp {}

impl Router for NoOp {
    fn carve(&mut self, _grid: &mut Grid, _cells: Vec<Option<Cell>>) {}
}
