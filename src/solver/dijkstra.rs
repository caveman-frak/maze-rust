use crate::maze::grid::Grid;
use crate::maze::Cell;
use crate::solver::{Distances, Solver};
use std::collections::HashMap;

pub struct Dijkstra {}

#[allow(dead_code)]
impl Dijkstra {
    pub fn new() -> Dijkstra {
        Dijkstra {}
    }

    pub fn solve(grid: &Grid, start: (u32, u32)) -> Distances {
        Dijkstra::new().solve(grid, start)
    }

    fn frontier(&self, map: &mut HashMap<Cell, u32>, grid: &Grid, cell: Cell, depth: u32) {
        map.insert(cell, depth);
        for direction in grid.links(&cell) {
            if let Some(c) = grid.neighbour(&cell, *direction) {
                if !map.contains_key(c) {
                    self.frontier(map, grid, *c, depth + 1);
                }
            }
        }
    }
}

impl Solver for Dijkstra {
    fn solve(&self, grid: &Grid, start: (u32, u32)) -> Distances {
        let cell = grid.cell(start.0, start.1).expect("Invalid starting cell");
        let mut map = HashMap::new();
        self.frontier(&mut map, grid, *cell, 0);

        Distances::new(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::sidewinder::SideWinder;
    use rand::rngs::mock::StepRng;

    #[test]
    fn check_build_distances() {
        let mut rng = StepRng::new(1, 1);
        let grid = Grid::grid(3, 3, Grid::ALLOW_ALL, &mut SideWinder::new(&mut rng));

        let solver = Dijkstra::new();
        let distances = solver.solve(&grid, (2, 0));

        assert_eq!(distances.start().coords(), (2, 0));
        assert_eq!(
            distances.distance(*grid.cell(2, 2).expect("Missing cell 2,2")),
            6
        );
        assert_eq!(distances.cells(0).len(), 1);
        assert_eq!(distances.cells(1).len(), 1);
        assert_eq!(distances.cells(2).len(), 1);
        assert_eq!(distances.cells(3).len(), 1);
        assert_eq!(distances.cells(4).len(), 2);
        assert_eq!(distances.cells(5).len(), 2);
        assert_eq!(distances.cells(6).len(), 1);
        assert_eq!(distances.cells(7).len(), 0);
    }
}
