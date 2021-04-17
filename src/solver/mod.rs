pub mod dijkstra;

use crate::maze::{Cell, Direction, Maze};

#[allow(unused_imports)]
use crate::util::math;

use std::collections::HashMap;

pub trait Solver<T: Direction, M: Maze<T>> {
    fn solve(&self, grid: &M, start: (u32, u32)) -> Distances;
}

#[derive(Debug)]
pub struct Distances {
    cells: HashMap<Cell, u32>,
    distances: HashMap<u32, Vec<Cell>>,
}

#[allow(dead_code)]
impl Distances {
    pub fn new(cells: HashMap<Cell, u32>) -> Distances {
        let distances = Distances::build_distances(&cells);
        Distances { cells, distances }
    }

    fn build_distances(cells: &HashMap<Cell, u32>) -> HashMap<u32, Vec<Cell>> {
        let mut distances = HashMap::new();

        for (cell, distance) in cells {
            distances
                .entry(*distance)
                .or_insert_with(Vec::new)
                .push(*cell);
        }
        distances
    }

    pub fn start(&self) -> Cell {
        *self
            .distances
            .get(&0)
            .expect("No cells at distance zero")
            .get(0)
            .expect("Empty list of cells at distance zero")
    }

    pub fn cells(&self, distance: u32) -> &[Cell] {
        if let Some(cells) = self.distances.get(&distance) {
            &cells
        } else {
            <&[Cell]>::default()
        }
    }

    pub fn distance(&self, cell: Cell) -> u32 {
        *self
            .cells
            .get(&cell)
            .unwrap_or_else(|| panic!("Missing distance for {:?}", cell))
    }

    pub fn all_cells(&self) -> &HashMap<Cell, u32> {
        &self.cells
    }
}

mod internal {
    use super::{Distances, Solver};
    use crate::maze::{Direction, Maze};
    use crate::util::math;

    use std::collections::HashMap;

    pub struct SimpleSolver {}

    impl<T: Direction, M: Maze<T>> Solver<T, M> for SimpleSolver {
        fn solve(&self, grid: &M, start: (u32, u32)) -> Distances {
            let mut map = HashMap::new();
            let (row, column) = start;

            for cell in grid.cells() {
                map.insert(
                    *cell,
                    math::diff(row, cell.row()) + math::diff(column, cell.column()),
                );
            }
            Distances::new(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::grid::Grid;
    use crate::maze::Maze;
    use crate::solver::internal::SimpleSolver;

    #[test]
    fn check_distances_start() {
        let grid = Grid::square(4);
        let distances = SimpleSolver {}.solve(&grid, (0, 0));

        assert_eq!(distances.start().coords(), (0, 0));
    }

    #[test]
    fn check_distances_cell() {
        let grid = Grid::square(4);
        let distances = SimpleSolver {}.solve(&grid, (0, 0));

        assert_eq!(
            distances.distance(*grid.cell(1, 1).expect("Missing cell 1,1")),
            2
        );
    }

    #[test]
    fn check_distances_cells() {
        let grid = Grid::square(4);
        let distances = SimpleSolver {}.solve(&grid, (0, 0));

        assert_eq!(distances.cells(1).len(), 2);
        assert_eq!(distances.cells(4).len(), 3);
    }

    #[test]
    fn check_distances_all_cells() {
        let grid = Grid::square(4);
        let distances = SimpleSolver {}.solve(&grid, (0, 0));

        assert_eq!(distances.all_cells().len(), 16);
    }

    #[test]
    fn check_build_distances() {
        let grid = Grid::square(2);
        let mut map = HashMap::new();
        map.insert(*grid.cell(0, 0).unwrap(), 1u32);
        map.insert(*grid.cell(1, 1).unwrap(), 2u32);

        let distances = Distances::build_distances(&map);

        assert_eq!(distances.get(&1).unwrap().len(), 1);
        assert_eq!(distances.get(&1).unwrap().get(0).unwrap().coords(), (0, 0));
        assert_eq!(distances.get(&2).unwrap().len(), 1);
        assert_eq!(distances.get(&2).unwrap().get(0).unwrap().coords(), (1, 1));
    }

    #[test]
    #[should_panic]
    fn check_invalid_distance_zero() {
        let grid = Grid::square(2);
        let mut map = HashMap::new();
        map.insert(*grid.cell(0, 0).unwrap(), 1);

        let distances = Distances::new(map);
        distances.start();
    }
}
