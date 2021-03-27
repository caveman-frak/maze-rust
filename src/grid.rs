use joinery::Joinable;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
    cell.map_or(String::from("None"), |c| c.to_string())
}

#[derive(Debug)]
pub struct Grid {
    rows: u32,
    columns: u32,
    cells: Vec<Option<Cell>>,
    neighbours: HashMap<Cell, Vec<Direction>>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

pub fn grid<F>(rows: u32, columns: u32, allowed: F) -> Grid
where
    F: Fn(u32, u32) -> bool,
{
    let cells = Grid::build_cells(rows, columns, allowed);
    let neighbours = Grid::build_neighbours(&cells, rows, columns);

    Grid {
        rows,
        columns,
        cells,
        neighbours,
    }
}

pub fn square(size: u32) -> Grid {
    grid(size, size, allow_all)
}

pub fn allow_all(_row: u32, _column: u32) -> bool {
    true
}

#[allow(dead_code)]
impl Grid {
    /// Return a list of valid cells, exclude any that have been masked
    pub fn cells(&self) -> Vec<&Cell> {
        self.cells.iter().filter_map(|x| x.as_ref()).collect()
    }

    /// Return the cell at the row and column, or None if the cell is masked
    ///
    /// # Arguments
    /// * `row` - grid row
    /// * `column` - grid column
    pub fn cell(&self, row: u32, column: u32) -> Option<&Cell> {
        if row >= self.rows || column >= self.columns {
            None
        } else {
            let offset = self.columns * row + column;
            self.cells[offset as usize].as_ref()
        }
    }

    /// Return the neighbouring cell if one exists, otherwise None
    ///
    /// # Arguments
    /// * `cell` - the base cell
    /// * `direction` - the direction of the neighbour
    ///
    /// ```
    ///     let grid = grid::square(3);
    ///     let cell = grid.cell(0, 0).unwrap();
    ///     println!(
    ///         "neighbours -> N = {:?}, E = {:?}, S = {:?}, W = {:?}",
    ///         grid.neighbour(&cell, grid::Direction::North),
    ///         grid.neighbour(&cell, grid::Direction::East),
    ///         grid.neighbour(&cell, grid::Direction::South),
    ///         grid.neighbour(&cell, grid::Direction::West)
    ///     );
    /// ```
    pub fn neighbour(&self, cell: &Cell, direction: Direction) -> Option<&Cell> {
        match direction {
            Direction::North => {
                if cell.row == 0 {
                    None
                } else {
                    self.cell(cell.row - 1, cell.column)
                }
            }
            Direction::East => {
                if cell.column == self.columns - 1 {
                    None
                } else {
                    self.cell(cell.row, cell.column + 1)
                }
            }
            Direction::South => {
                if cell.row == self.rows - 1 {
                    None
                } else {
                    self.cell(cell.row + 1, cell.column)
                }
            }
            Direction::West => {
                if cell.column == 0 {
                    None
                } else {
                    self.cell(cell.row, cell.column - 1)
                }
            }
        }
    }

    pub fn neighbours(&self, cell: &Cell) -> &[Direction] {
        self.neighbours.get(cell).unwrap()
    }

    fn build_cells<F>(rows: u32, columns: u32, allowed: F) -> Vec<Option<Cell>>
    where
        F: Fn(u32, u32) -> bool,
    {
        let mut cells = Vec::new();

        for row in 0..rows {
            for column in 0..columns {
                cells.push(if allowed(row, column) {
                    Some(Cell { row, column })
                } else {
                    None
                });
            }
        }
        cells
    }

    fn build_neighbours(
        cells: &[Option<Cell>],
        rows: u32,
        columns: u32,
    ) -> HashMap<Cell, Vec<Direction>> {
        let mut neighbours = HashMap::with_capacity((rows * columns) as usize);

        for element in cells {
            if element.is_some() {
                let cell = element.as_ref().unwrap();
                neighbours.insert(
                    cell.clone(),
                    Grid::_neighbours(&cells, rows, columns, &cell),
                );
            }
        }

        neighbours
    }

    fn _neighbours(cells: &[Option<Cell>], rows: u32, columns: u32, cell: &Cell) -> Vec<Direction> {
        let mut neighbours = Vec::new();

        if cell.row > 0 && cells[(columns * (cell.row - 1) + cell.column) as usize] != None {
            neighbours.push(Direction::North);
        }
        if cell.column < columns - 1
            && cells[(columns * cell.row + cell.column + 1) as usize] != None
        {
            neighbours.push(Direction::East);
        }
        if cell.row < rows - 1 && cells[(columns * (cell.row + 1) + cell.column) as usize] != None {
            neighbours.push(Direction::South);
        }
        if cell.column > 0 && cells[(columns * cell.row + cell.column - 1) as usize] != None {
            neighbours.push(Direction::West);
        }
        neighbours
    }

    pub fn write_maze(&self) -> String {
        let mut s = String::new();

        const VDIV: char = '|';
        const HDIV: char = '-';
        const CORNER: char = '+';
        const CELL: char = ' ';
        const NONE: char = 'X';

        for row in 0..self.rows {
            // print top row, can ignore all cells for now
            let start = (row * self.columns) as usize;
            let end = start + self.columns as usize;
            let cells = &self.cells[start..end];

            // write an unconditional top line
            if row == 0 {
                s = self.write_line(s, cells, |_, _| CORNER, |_, _| HDIV);
            }
            // write the cell body and vertical dividers
            // mark None cells as X
            // TODO implement link checking and Eastern divider printing
            s = self.write_line(
                s,
                cells,
                |_, _| VDIV,
                |_, c| match c {
                    Option::Some(_) => CELL,
                    Option::None => NONE,
                },
            );
            // write horizontal dividers
            // TODO implement link checking and Souther divider printing
            s = self.write_line(s, cells, |_, _| CORNER, |_, _| HDIV);
        }
        s
    }

    // TODO find better way to pass the string instance, ideally without having to return it
    fn write_line<F1, F2>(&self, mut s: String, cells: &[Option<Cell>], f1: F1, f2: F2) -> String
    where
        F1: Fn(&Grid, &Option<Cell>) -> char,
        F2: Fn(&Grid, &Option<Cell>) -> char,
    {
        s.push(f1(self, &None));
        for cell in cells {
            s.push(f2(self, cell));
            s.push(f1(self, cell));
        }
        s.push('\n');
        s
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Grid(rows={}, columns={}, cells=[{}])",
            self.rows,
            self.columns,
            self.cells()
                .join_with(joinery::separators::CommaSpace)
                .to_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_square() {
        let grid = square(2);

        assert_eq!(grid.rows, 2);
        assert_eq!(grid.columns, 2);
    }

    #[test]
    fn check_cell_count() {
        let grid = grid(2, 3, allow_all);

        assert_eq!(grid.cells.len(), 6);
    }

    #[test]
    fn check_valid_cell_count() {
        let grid = grid(2, 3, allow_all);

        assert_eq!(grid.cells().len(), 6);
    }

    #[test]
    fn check_cell_position() {
        let grid = grid(2, 3, allow_all);

        for row in 0..grid.rows {
            for column in 0..grid.columns {
                let cell = grid.cell(row, column).unwrap();

                assert_eq!(cell.row, row);
                assert_eq!(cell.column, column);
            }
        }
    }

    #[test]
    fn check_bounds() {
        let grid = square(3);

        assert!(matches!(grid.cell(0, 3), None));
        assert!(matches!(grid.cell(4, 0), None));
    }

    #[test]
    fn check_neighbour_top_left() {
        let grid = square(3);
        let cell = grid.cell(0, 0).unwrap();

        assert!(matches!(grid.neighbour(&cell, Direction::North), None));
        assert!(matches!(grid.neighbour(&cell, Direction::West), None));
        assert_eq!(grid.neighbour(&cell, Direction::South), grid.cell(1, 0));
        assert_eq!(grid.neighbour(&cell, Direction::East), grid.cell(0, 1));
    }

    #[test]
    fn check_neighbour_top_right() {
        let grid = square(3);
        let cell = grid.cell(0, 2).unwrap();

        assert!(matches!(grid.neighbour(&cell, Direction::North), None));
        assert_eq!(grid.neighbour(&cell, Direction::West), grid.cell(0, 1));
        assert_eq!(grid.neighbour(&cell, Direction::South), grid.cell(1, 2));
        assert!(matches!(grid.neighbour(&cell, Direction::East), None));
    }
    #[test]
    fn check_neighbour_center() {
        let grid = square(3);
        let cell = grid.cell(1, 1).unwrap();

        assert_eq!(grid.neighbour(&cell, Direction::North), grid.cell(0, 1));
        assert_eq!(grid.neighbour(&cell, Direction::West), grid.cell(1, 0));
        assert_eq!(grid.neighbour(&cell, Direction::South), grid.cell(2, 1));
        assert_eq!(grid.neighbour(&cell, Direction::East), grid.cell(1, 2));
    }

    #[test]
    fn check_neighbour_bottom_left() {
        let grid = square(3);
        let cell = grid.cell(2, 0).unwrap();

        assert_eq!(grid.neighbour(&cell, Direction::North), grid.cell(1, 0));
        assert!(matches!(grid.neighbour(&cell, Direction::West), None));
        assert!(matches!(grid.neighbour(&cell, Direction::South), None));
        assert_eq!(grid.neighbour(&cell, Direction::East), grid.cell(2, 1));
    }

    #[test]
    fn check_neighbour_bottom_right() {
        let grid = square(3);
        let cell = grid.cell(2, 2).unwrap();

        assert_eq!(grid.neighbour(&cell, Direction::North), grid.cell(1, 2));
        assert_eq!(grid.neighbour(&cell, Direction::West), grid.cell(2, 1));
        assert!(matches!(grid.neighbour(&cell, Direction::South), None));
        assert!(matches!(grid.neighbour(&cell, Direction::East), None));
    }

    #[test]
    fn check_neighbours_top_left() {
        let grid = square(3);
        let cell = grid.cell(0, 0).unwrap();

        assert_eq!(
            grid.neighbours(&cell),
            vec![Direction::East, Direction::South]
        )
    }

    #[test]
    fn check_neighbours_center() {
        let grid = square(3);
        let cell = grid.cell(1, 1).unwrap();

        assert_eq!(
            grid.neighbours(&cell),
            vec![
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ]
        )
    }

    #[test]
    fn check_neighbours_bottom_right() {
        let grid = square(3);
        let cell = grid.cell(2, 2).unwrap();

        assert_eq!(
            grid.neighbours(&cell),
            vec![Direction::North, Direction::West]
        )
    }

    #[test]
    fn check_masked() {
        let alternate = |r, c| r % 2 != c % 2;
        let grid = grid(2, 3, alternate);
        assert_eq!(grid.cells.len(), 6);
        assert_eq!(grid.cells().len(), 3);

        assert!(matches!(grid.cell(0, 0), None));
        assert!(matches!(grid.cell(0, 1), Some(_)));
    }

    #[test]
    fn check_to_string() {
        let grid = square(2);

        assert_eq!(
            grid.to_string(),
            "Grid(rows=2, columns=2, cells=[(0, 0), (0, 1), (1, 0), (1, 1)])"
        );
    }

    #[test]
    fn check_write_maze() {
        let newline: String = String::from("\n");
        let grid = square(2);

        assert_eq!(
            newline + &grid.write_maze(),
            r#"
+-+-+
| | |
+-+-+
| | |
+-+-+
"#
        );
    }
}
