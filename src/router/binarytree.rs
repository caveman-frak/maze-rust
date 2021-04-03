use crate::grid::{Cell, Direction, Grid};
use crate::router::Router;
use rand::{Rng, RngCore};

pub struct BinaryTree<'a> {
    rng: &'a mut dyn RngCore,
}

impl<'a> BinaryTree<'a> {
    pub fn new(rng: &'a mut dyn RngCore) -> BinaryTree<'a> {
        BinaryTree { rng }
    }

    fn random(&mut self, grid: &Grid, cell: Cell) -> Option<Direction> {
        let mut directions = Vec::new();

        if grid.neighbour(&cell, Direction::North).is_some() {
            directions.push(Direction::North);
        }
        if grid.neighbour(&cell, Direction::East).is_some() {
            directions.push(Direction::East);
        }

        match directions.len() as u16 {
            0 => None,
            1 => Some(directions[0]),
            range => Some(directions[(self.rng.gen::<u16>() % range) as usize]),
        }
    }
}

impl<'a> Router for BinaryTree<'a> {
    fn carve(&mut self, grid: &mut Grid, _cells: Vec<Option<Cell>>) {
        for cell in _cells {
            if let Some(c) = cell {
                if let Some(direction) = self.random(grid, c) {
                    grid.link_cell(&c, direction);
                }
            }
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
        let grid = Grid::grid(3, 3, Grid::allow_all, &mut BinaryTree::new(&mut rng));

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
