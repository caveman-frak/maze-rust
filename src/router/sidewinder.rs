use crate::grid::{Cell, Direction, Grid};
use crate::router::Router;
use rand::{Rng, RngCore};

pub struct SideWinder<'a> {
    rng: &'a mut dyn RngCore,
}

#[allow(dead_code)]
impl<'a> SideWinder<'a> {
    pub fn new(rng: &'a mut dyn RngCore) -> SideWinder<'a> {
        SideWinder { rng }
    }

    fn close_row(&mut self, cell: Cell, row: u32) -> bool {
        cell.row() != row || (row > 0 && (self.rng.gen::<u16>() % 2 == 0))
    }

    fn random(&mut self, run: &[Cell]) -> Cell {
        run[self.rng.gen::<usize>() % run.len()]
    }
}

impl<'a> Router for SideWinder<'a> {
    fn carve(&mut self, grid: &mut Grid, cells: Vec<Option<Cell>>) {
        let mut row = 0u32;
        let mut run = Vec::new();
        for cell in cells {
            if let Some(c) = cell {
                if c.row() != row {
                    run.clear();
                    row = c.row();
                }
                run.push(c);
                if self.close_row(c, row) {
                    if !run.is_empty() {
                        grid.link_cell(&self.random(&run), Direction::North);
                    }
                    run.clear();
                } else {
                    grid.link_cell(&c, Direction::East);
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
    fn check_mock_sidewinder() {
        let newline: String = String::from("\n");
        let mut rng = StepRng::new(1, 1);
        let grid = Grid::grid(3, 3, Grid::allow_all, &mut SideWinder::new(&mut rng));

        assert_eq!(
            newline + &grid.to_string(),
            r#"
+---+---+---+
|           |
+---+   +   +
|       |   |
+   +   +   +
|   |   |   |
+---+---+---+
"#
        );
    }
}
