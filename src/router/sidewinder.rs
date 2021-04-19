use crate::maze::grid::Compass;
use crate::maze::{Cell, Direction, Maze};
use crate::router::Router;
use rand::{Rng, RngCore};

pub struct SideWinder<'a, T: Direction> {
    rng: &'a mut dyn RngCore,
    directions: (T, T),
    run: Vec<Cell>,
}

#[allow(dead_code)]
impl<'a, T: Direction> SideWinder<'a, T> {
    pub fn new_for_compass(rng: &'a mut dyn RngCore) -> SideWinder<'a, Compass> {
        SideWinder::new(rng, (Compass::North, Compass::East))
    }

    pub fn new(rng: &'a mut dyn RngCore, directions: (T, T)) -> Self {
        SideWinder {
            rng,
            directions,
            run: Vec::new(),
        }
    }

    fn close_row<M: Maze<T>>(&mut self, cell: &Cell, maze: &M, top: &T, side: &T) -> bool {
        let neighbours = maze.neighbours(&cell);

        !neighbours.contains_key(side)
            || (neighbours.contains_key(top) && (self.rng.gen::<u16>() % 2 == 0))
    }

    fn random_cell(&mut self) -> Cell {
        self.run[self.rng.gen::<usize>() % self.run.len()]
    }
}

impl<'a, T: Direction, M: Maze<T>> Router<T, M> for SideWinder<'a, T> {
    fn carve(&mut self, maze: &mut M, cells: Vec<Option<Cell>>) {
        self.carve_by_row(maze, cells);
    }

    fn by_cell(&mut self, maze: &mut M, cell: Cell) {
        let (top, side) = self.directions;
        self.run.push(cell);
        if self.close_row(&cell, maze, &top, &side) {
            maze.link_cell(&self.random_cell(), top);
            self.run.clear();
        } else {
            maze.link_cell(&cell, side);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::grid::Grid;
    use rand::rngs::mock::StepRng;

    #[test]
    fn check_mock_sidewinder() {
        let newline: String = String::from("\n");
        let mut rng = StepRng::new(1, 1);
        let grid = Grid::grid(
            3,
            3,
            Grid::ALLOW_ALL,
            &mut SideWinder::<Compass>::new_for_compass(&mut rng),
        );

        assert_eq!(
            newline + &grid.to_string(),
            r#"
+---+---+---+
|           |
+   +   +   +
|   |   |   |
+---+   +   +
|       |   |
+---+---+---+
"#
        );
    }
}
