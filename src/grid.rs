use joinery::Joinable;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Cell {
    row: u32,
    column: u32,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.column)
    }
}

pub fn print_cell(cell: Option<&Cell>) -> String {
    cell.map_or(String::from("None"), |c| format!("{}", c))
}

#[derive(Debug)]
pub struct Grid {
    rows: u32,
    columns: u32,
    cells: Vec<Cell>,
}

pub enum Direction {
    North,
    East,
    South,
    West,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Grid(rows={}, columns={}, cells=[{}]",
            self.rows,
            self.columns,
            self.cells
                .clone()
                .join_with(joinery::separators::CommaSpace)
        )
    }
}

#[allow(dead_code)]
impl Grid {
    pub fn grid(rows: u32, columns: u32) -> Grid {
        let mut cells = vec![];

        for row in 0..rows {
            for column in 0..columns {
                let cell = Cell { row, column };
                cells.push(cell);
            }
        }

        Grid {
            rows,
            columns,
            cells,
        }
    }

    pub fn square(size: u32) -> Grid {
        Grid::grid(size, size)
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn columns(&self) -> u32 {
        self.columns
    }

    pub fn cells(&self) -> &Vec<Cell> {
        &self.cells
    }

    pub fn cell(&self, row: u32, column: u32) -> &Cell {
        let offset = (self.columns * row) + column;

        &self.cells[offset as usize]
    }

    pub fn neighbour(&self, cell: &Cell, direction: Direction) -> Option<&Cell> {
        match direction {
            Direction::North => {
                if cell.row == 0 {
                    None
                } else {
                    Some(self.cell(cell.row - 1, cell.column))
                }
            }
            Direction::East => {
                if cell.column == self.columns - 1 {
                    None
                } else {
                    Some(self.cell(cell.row, cell.column + 1))
                }
            }
            Direction::South => {
                if cell.row == self.rows - 1 {
                    None
                } else {
                    Some(self.cell(cell.row + 1, cell.column))
                }
            }
            Direction::West => {
                if cell.column == 0 {
                    None
                } else {
                    Some(self.cell(cell.row, cell.column - 1))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_square() {
        let grid = Grid::square(2);

        assert_eq!(grid.rows, 2);
        assert_eq!(grid.columns, 2);
    }

    #[test]
    fn check_cell_count() {
        let grid = Grid::grid(2, 3);

        assert_eq!(grid.cells.len(), 6);
    }

    #[test]
    fn check_immutable() {
        let mut grid = Grid::grid(2, 3);

        // TODO fix this so that cells is immutable, try storing a slice
        grid.cells.remove(2);
         assert_eq!(grid.cells.len(), 5);
    }

    #[test]
    fn check_cell_position() {
        let grid = Grid::grid(2, 3);

        for row in 0..grid.rows {
            for column in 0..grid.columns {
                let cell = grid.cell(row, column);

                assert_eq!(cell.row, row);
                assert_eq!(cell.column, column);
            }
        }
    }

    #[test]
    fn check_neighbour() {
        let grid = Grid::grid(2, 3);
        let cell = grid.cell(0, 0);

        assert!(matches!(grid.neighbour(cell, Direction::North), None));
        assert!(matches!(grid.neighbour(cell, Direction::West), None));
        assert_eq!(
            grid.neighbour(cell, Direction::South).unwrap(),
            grid.cell(1, 0)
        );
        assert_eq!(
            grid.neighbour(cell, Direction::East).unwrap(),
            grid.cell(0, 1)
        );
    }
}
