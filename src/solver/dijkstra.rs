use crate::maze::{Cell, Direction, Maze};
use crate::solver::{Distances, Solver};
use std::collections::HashMap;

pub struct Dijkstra {}

#[allow(dead_code)]
impl Dijkstra {
    pub fn new() -> Self {
        Dijkstra {}
    }

    pub fn solve<T: Direction, M: Maze<T>>(grid: &M, start: (u32, u32)) -> Distances {
        Dijkstra::new().solve(grid, start)
    }

    fn frontier<T: Direction, M: Maze<T>>(
        &self,
        map: &mut HashMap<Cell, u32>,
        maze: &M,
        cell: Cell,
        depth: u32,
    ) {
        let neighbours = maze.neighbours(&cell);
        map.insert(cell, depth);

        for direction in maze.links(&cell) {
            if let Some(c) = neighbours.get(direction) {
                if !map.contains_key(c) {
                    self.frontier(map, maze, *c, depth + 1);
                }
            }
        }
    }
}

impl<T: Direction, M: Maze<T>> Solver<T, M> for Dijkstra {
    fn solve(&self, maze: &M, start: (u32, u32)) -> Distances {
        let cell = maze.cell(start.0, start.1).expect("Invalid starting cell");
        let mut map = HashMap::new();
        self.frontier(&mut map, maze, *cell, 0);

        Distances::new(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::grid::{Compass, Grid};
    use crate::router::sidewinder::SideWinder;
    use rand::rngs::mock::StepRng;

    #[test]
    fn check_build_distances() {
        let mut rng = StepRng::new(1, 1);
        let grid = Grid::grid(
            3,
            3,
            Grid::ALLOW_ALL,
            &mut SideWinder::<Compass>::new_for_compass(&mut rng),
        );

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

    #[test]
    fn check_solve() {
        let mut rng = StepRng::new(1, 1);
        let grid = Grid::grid(
            3,
            3,
            Grid::ALLOW_ALL,
            &mut SideWinder::<Compass>::new_for_compass(&mut rng),
        );
        let distances = Dijkstra::solve(&grid, (0, 0));
        assert_eq!(distances.start().coords(), (0, 0));
    }
}
