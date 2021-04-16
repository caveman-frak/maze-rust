use crate::maze::grid::{Compass, Grid};
use crate::maze::{Cell, Maze};
use crate::router::Router;
use rand::{Rng, RngCore};

pub struct SideWinder<'a> {
    rng: &'a mut dyn RngCore,
    run: Vec<Cell>,
}

#[allow(dead_code)]
impl<'a> SideWinder<'a> {
    pub fn new(rng: &'a mut dyn RngCore) -> SideWinder<'a> {
        SideWinder {
            rng,
            run: Vec::new(),
        }
    }

    fn close_row(&mut self, cell: &Cell, columns: u32) -> bool {
        cell.column() == columns - 1 || (cell.row() > 0 && (self.rng.gen::<u16>() % 2 == 0))
    }

    fn random_cell(&mut self) -> Cell {
        self.run[self.rng.gen::<usize>() % self.run.len()]
    }
}

impl<'a> Router<Compass, Grid> for SideWinder<'a> {
    fn carve(&mut self, maze: &mut Grid, cells: Vec<Option<Cell>>) {
        self.carve_by_row(maze, cells);
    }

    fn by_cell(&mut self, maze: &mut Grid, cell: Cell) {
        self.run.push(cell);
        if self.close_row(&cell, maze.columns()) {
            maze.link_cell(&self.random_cell(), Compass::North);
            self.run.clear();
        } else {
            maze.link_cell(&cell, Compass::East);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn check_mock_sidewinder() {
        let newline: String = String::from("\n");
        let mut rng = StepRng::new(1, 1);
        let grid = Grid::grid(3, 3, Grid::ALLOW_ALL, &mut SideWinder::new(&mut rng));

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
