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

    fn close_row(&mut self, cell: &Cell, columns: u32) -> bool {
        cell.column() == columns - 1 || (cell.row() > 0 && (self.rng.gen::<u16>() % 2 == 0))
    }

    fn random(&mut self, run: &[Cell]) -> Cell {
        run[self.rng.gen::<usize>() % run.len()]
    }
}

impl<'a> Router for SideWinder<'a> {
    fn carve(&mut self, grid: &mut Grid, cells: Vec<Option<Cell>>) {
        let mut run = Vec::new();
        for row in 0..grid.rows() {
            let start = (row * grid.columns()) as usize;
            let end = start + grid.columns() as usize;
            for cell in &cells[start..end] {
                if let Some(c) = cell {
                    run.push(*c);
                    if self.close_row(c, grid.columns()) {
                        grid.link_cell(&self.random(&run), Direction::North);
                        run.clear();
                    } else {
                        grid.link_cell(&c, Direction::East);
                    }
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
+   +   +   +
|   |   |   |
+---+   +   +
|       |   |
+---+---+---+
"#
        );
    }
}
