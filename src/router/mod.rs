pub mod binarytree;
pub mod sidewinder;

use crate::maze::{Cell, Direction, Maze};

pub trait Router<T: Direction, M: Maze<T>> {
    fn carve(&mut self, maze: &mut M, cells: Vec<Option<Cell>>);

    fn carve_by_cell(&mut self, maze: &mut M, cells: Vec<Option<Cell>>) {
        for cell in cells {
            if let Some(c) = cell {
                self.by_cell(maze, c);
            }
        }
    }

    fn by_cell(&mut self, _maze: &mut M, _cell: Cell) {}

    fn carve_by_row(&mut self, maze: &mut M, cells: Vec<Option<Cell>>) {
        for row in 0..maze.rows() {
            let start = (row * maze.columns()) as usize;
            let end = start + maze.columns() as usize;
            self.by_row(maze, &cells[start..end], row);
        }
    }

    fn by_row(&mut self, maze: &mut M, cells: &[Option<Cell>], _row: u32) {
        for cell in cells {
            if let Some(c) = cell {
                self.by_cell(maze, *c)
            }
        }
    }
}

pub mod internal {
    use super::Router;
    use crate::maze::{Cell, Direction, Maze};

    pub struct NoOp {}

    impl<T: Direction, M: Maze<T>> Router<T, M> for NoOp {
        fn carve(&mut self, _maze: &mut M, _cells: Vec<Option<Cell>>) {}
    }
}
