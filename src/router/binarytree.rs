use crate::maze::grid::Compass;
use crate::maze::{Cell, Direction, Maze};
use crate::router::Router;
use rand::{Rng, RngCore};

pub struct BinaryTree<'a, T: Direction> {
    rng: &'a mut dyn RngCore,
    preferred: Vec<T>,
}

impl<'a, T: Direction> BinaryTree<'a, T> {
    pub fn new_for_compass(rng: &'a mut dyn RngCore) -> BinaryTree<'a, Compass> {
        BinaryTree::new(rng, vec![Compass::North, Compass::East])
    }

    pub fn new(rng: &'a mut dyn RngCore, preferred: Vec<T>) -> Self {
        BinaryTree { rng, preferred }
    }

    fn direction<M: Maze<T>>(&mut self, maze: &M, cell: Cell) -> Option<T> {
        let neighbours = maze.neighbours(&cell);

        let directions: Vec<&T> = self
            .preferred
            .iter()
            .filter(|d| neighbours.get(d).is_some())
            .collect();
        match directions.len() {
            0 => None,
            1 => Some(*directions[0]),
            range => Some(*directions[self.rng.gen::<usize>() % range]),
        }
    }
}

impl<'a, T: Direction, M: Maze<T>> Router<T, M> for BinaryTree<'a, T> {
    fn carve(&mut self, maze: &mut M, cells: Vec<Option<Cell>>) {
        self.carve_by_cell(maze, cells);
    }

    fn by_cell(&mut self, maze: &mut M, cell: Cell) {
        if let Some(direction) = self.direction(maze, cell) {
            maze.link_cell(&cell, direction);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::grid::Grid;
    use rand::rngs::mock::StepRng;

    #[test]
    fn check_mock_binarytree() {
        let newline: String = String::from("\n");
        let mut rng = StepRng::new(0, 1);
        let grid = Grid::grid(
            3,
            3,
            Grid::ALLOW_ALL,
            &mut BinaryTree::<Compass>::new_for_compass(&mut rng),
        );

        assert_eq!(
            newline + &grid.to_string(),
            r#"
+---+---+---+
|           |
+   +---+   +
|   |       |
+   +---+   +
|   |       |
+---+---+---+
"#
        );
    }
}
