use crate::grid::{Cell, Grid};
#[allow(unused_imports)]
use crate::math::diff;

use std::collections::HashMap;

pub trait Solver {
    fn solve(&mut self, grid: &mut Grid, start: (u32, u32)) -> Distances;
}

pub struct Distances {
    cells: HashMap<u32, Vec<Cell>>,
    distances: HashMap<Cell, u32>,
}

#[allow(dead_code)]
impl Distances {
    pub fn new(cells: HashMap<u32, Vec<Cell>>) -> Distances {
        let distances = Distances::build_distances(&cells);
        Distances { cells, distances }
    }

    fn build_distances(cells: &HashMap<u32, Vec<Cell>>) -> HashMap<Cell, u32> {
        let mut distances = HashMap::new();

        for (distance, list) in cells.iter() {
            for cell in list {
                distances.insert(*cell, *distance);
            }
        }
        distances
    }

    pub fn start(&self) -> Cell {
        *self
            .cells
            .get(&0)
            .expect("No cells at distance zero")
            .get(0)
            .expect("Empty list of cells at distance zero")
    }

    pub fn cells(&self, distance: u32) -> &[Cell] {
        self.cells
            .get(&distance)
            .unwrap_or_else(|| panic!("Missing cells for distance {}", distance))
    }

    pub fn distance(&self, cell: Cell) -> u32 {
        *self
            .distances
            .get(&cell)
            .unwrap_or_else(|| panic!("Missing distance for {:?}", cell))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_build_distances() {
        let grid = Grid::square(2);
        let distances = Distances::new(populate_map((0, 0), &grid));

        assert_eq!(distances.start().coords(), (0, 0));
        assert_eq!(
            distances.distance(*grid.cell(1, 1).expect("Missing cell 1,1")),
            2
        );
        assert_eq!(distances.cells(1).len(), 2);
    }

    fn populate_map(start: (u32, u32), grid: &Grid) -> HashMap<u32, Vec<Cell>> {
        let mut cells = HashMap::new();
        let (row, column) = start;

        for cell in grid.cells() {
            cells
                .entry(diff(row, cell.row()) + diff(column, cell.column()))
                .or_insert_with(Vec::new)
                .push(*cell);
        }
        cells
    }
}
