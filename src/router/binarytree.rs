use crate::maze::grid::{Compass, Grid};
use crate::maze::{Cell, Maze};
use crate::router::Router;
use rand::{Rng, RngCore};

pub struct BinaryTree<'a> {
    rng: &'a mut dyn RngCore,
}

impl<'a> BinaryTree<'a> {
    pub fn new(rng: &'a mut dyn RngCore) -> BinaryTree<'a> {
        BinaryTree { rng }
    }

    fn compass(&mut self, grid: &Grid, cell: Cell) -> Option<Compass> {
        let neighbours = grid.neighbours(&cell);
        let mut compasss = Vec::new();

        if neighbours.get(&Compass::North).is_some() {
            compasss.push(Compass::North);
        }
        if neighbours.get(&Compass::East).is_some() {
            compasss.push(Compass::East);
        }

        match compasss.len() {
            0 => None,
            1 => Some(compasss[0]),
            range => Some(compasss[self.rng.gen::<usize>() % range]),
        }
    }
}

impl<'a> Router for BinaryTree<'a> {
    fn carve(&mut self, grid: &mut Grid, cells: Vec<Option<Cell>>) {
        self.carve_by_cell(grid, cells);
    }

    fn by_cell(&mut self, grid: &mut Grid, cell: Cell) {
        if let Some(compass) = self.compass(grid, cell) {
            grid.link_cell(&cell, compass);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn check_mock_binarytree() {
        let newline: String = String::from("\n");
        let mut rng = StepRng::new(0, 1);
        let grid = Grid::grid(3, 3, Grid::ALLOW_ALL, &mut BinaryTree::new(&mut rng));

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
