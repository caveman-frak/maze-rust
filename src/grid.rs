use crate::router::{NoOp, Router};
// use crate::solver::{Distances, Solver};

use image::{ImageFormat, ImageResult, Rgb, RgbImage};
use imageproc::{drawing, rect};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Cell {
    row: u32,
    column: u32,
}

#[allow(dead_code)]
impl Cell {
    pub fn row(&self) -> u32 {
        self.row
    }

    pub fn column(&self) -> u32 {
        self.column
    }

    pub fn coords(&self) -> (u32, u32) {
        (self.row, self.column)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn reverse(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug)]
struct Attributes {
    neighbours: HashMap<Direction, Cell>,
    links: HashSet<Direction>,
}

#[allow(dead_code)]
impl Attributes {
    fn new(neighbours: HashMap<Direction, Cell>) -> Attributes {
        Attributes {
            neighbours,
            links: HashSet::new(),
        }
    }

    fn get_neighbour(&self, direction: &Direction) -> Option<&Cell> {
        self.neighbours.get(&direction)
    }

    fn has_link(&self, direction: &Direction) -> bool {
        self.links.contains(&direction)
    }

    fn add_link(&mut self, direction: &Direction) -> bool {
        self.links.insert(*direction)
    }

    fn remove_link(&mut self, direction: &Direction) -> bool {
        self.links.remove(direction)
    }
}

#[derive(Debug)]
pub struct Grid {
    rows: u32,
    columns: u32,
    cells: Vec<Option<Cell>>,
    attributes: HashMap<Cell, Attributes>,
}

#[allow(dead_code)]
impl Grid {
    /// Masking function that allows all cells
    pub const ALLOW_ALL: &'static dyn Fn(u32, u32) -> bool = &|_, _| true;

    /// Build a new grid instance.
    ///
    /// # Arguments
    /// * `rows` - grid row size
    /// * `columns` - grid column size
    /// * `allowed` - function to determine if a cell position is allowed or should be masked
    /// * `router` - router instance to carve out the links between cells
    ///
    ///   Construct a new 5x5 grid with the 4 corners masked and use the Bimary Tree algorith to
    ///   carve out the links between cells.
    /// ```
    ///     let mut rng = rand::thread_rng();
    ///     let grid = grid::Grid::grid(
    ///         5,
    ///         5,
    ///         |r, c| !((r == 0 || r == 4) && (c == 0 || c == 4)),
    ///         &mut BinaryTree::new(&mut rng),
    /// );
    /// ```
    ///
    pub fn grid<F>(rows: u32, columns: u32, allowed: F, router: &mut dyn Router) -> Grid
    where
        F: Fn(u32, u32) -> bool,
    {
        let cells = Grid::build_cells(rows, columns, allowed);
        let attributes = Grid::build_attributes(&cells, rows, columns);
        let c = cells.clone();

        let mut grid = Grid {
            rows,
            columns,
            cells,
            attributes,
        };

        router.carve(&mut grid, c);

        grid
    }

    pub fn square(size: u32) -> Grid {
        Grid::grid(size, size, Grid::ALLOW_ALL, &mut NoOp {})
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }
    pub fn columns(&self) -> u32 {
        self.columns
    }

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
    ///     let cell = grid.cell(0, 0).expect("Missing Cell 0,0");
    ///     println!(
    ///         "neighbours -> N = {:?}, E = {:?}, S = {:?}, W = {:?}",
    ///         grid.neighbour(&cell, grid::Direction::North),
    ///         grid.neighbour(&cell, grid::Direction::East),
    ///         grid.neighbour(&cell, grid::Direction::South),
    ///         grid.neighbour(&cell, grid::Direction::West)
    ///     );
    /// ```
    pub fn neighbour(&self, cell: &Cell, direction: Direction) -> Option<&Cell> {
        self.attributes(cell).get_neighbour(&direction)
    }

    pub fn neighbours(&self, cell: &Cell) -> &HashMap<Direction, Cell> {
        &self.attributes(cell).neighbours
    }

    pub fn links(&self, cell: &Cell) -> &HashSet<Direction> {
        &self.attributes(cell).links
    }

    fn attributes(&self, cell: &Cell) -> &Attributes {
        self.attributes
            .get(cell)
            .unwrap_or_else(|| panic!("Missing attribute for {:?}", cell))
    }

    fn attributes_mut(&mut self, cell: &Cell) -> &mut Attributes {
        self.attributes
            .get_mut(cell)
            .unwrap_or_else(|| panic!("Missing attribute for {:?}", cell))
    }

    pub fn link_cell(&mut self, cell: &Cell, direction: Direction) -> Option<Cell> {
        let neighbour = self.neighbour(cell, direction);
        match neighbour {
            Some(c) => {
                let to = *c;

                self.attributes_mut(&*cell).add_link(&direction);
                self.attributes_mut(&to).add_link(&direction.reverse());

                Some(to)
            }
            None => None,
        }
    }

    pub fn unlink_cell(&mut self, cell: &Cell, direction: Direction) -> Option<Cell> {
        let neighbour = self.neighbour(cell, direction);
        match neighbour {
            Some(c) => {
                let to = *c;

                self.attributes_mut(&*cell).remove_link(&direction);
                self.attributes_mut(&to).remove_link(&direction.reverse());

                Some(to)
            }
            None => None,
        }
    }

    fn build_cells<F>(rows: u32, columns: u32, allowed: F) -> Vec<Option<Cell>>
    where
        F: Fn(u32, u32) -> bool,
    {
        let mut cells = Vec::with_capacity((rows * columns) as usize);

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

    fn build_attributes(
        cells: &[Option<Cell>],
        rows: u32,
        columns: u32,
    ) -> HashMap<Cell, Attributes> {
        let mut attributes = HashMap::with_capacity((rows * columns) as usize);

        for element in cells {
            if let Some(cell) = element {
                attributes.insert(
                    *cell,
                    Attributes::new(Grid::_neighbours(&cells, rows, columns, &cell)),
                );
            }
        }
        attributes.shrink_to_fit();
        attributes
    }

    fn _neighbours(
        cells: &[Option<Cell>],
        rows: u32,
        columns: u32,
        cell: &Cell,
    ) -> HashMap<Direction, Cell> {
        let mut neighbours = HashMap::new();

        if cell.row > 0 {
            if let Some(c) = cells[(columns * (cell.row - 1) + cell.column) as usize].as_ref() {
                neighbours.insert(Direction::North, *c);
            }
        }
        if cell.column < columns - 1 {
            if let Some(c) = cells[(columns * cell.row + cell.column + 1) as usize].as_ref() {
                neighbours.insert(Direction::East, *c);
            }
        }
        if cell.row < rows - 1 {
            if let Some(c) = cells[(columns * (cell.row + 1) + cell.column) as usize].as_ref() {
                neighbours.insert(Direction::South, *c);
            }
        }
        if cell.column > 0 {
            if let Some(c) = cells[(columns * cell.row + cell.column - 1) as usize].as_ref() {
                neighbours.insert(Direction::West, *c);
            }
        }
        neighbours
    }

    fn has_link(grid: &Grid, cell: &Option<Cell>, direction: Direction) -> bool {
        if let Some(c) = cell {
            return grid.attributes(c).has_link(&direction);
        }
        false
    }

    fn write_row<F1, F2>(&self, s: &mut String, scale: u32, row: &[Option<Cell>], f1: F1, f2: F2)
    where
        F1: Fn(&Grid, &Option<Cell>) -> char,
        F2: Fn(&Grid, &Option<Cell>) -> char,
    {
        s.push(f1(self, &None));
        for cell in row {
            let ch = f2(self, cell);
            for _ in 0..scale {
                s.push(ch);
            }
            s.push(f1(self, cell));
        }
        s.push('\n');
    }

    fn _draw(&self) -> image::RgbImage {
        let white = Rgb([255u8, 255u8, 255u8]);
        let black = Rgb([0u8, 0u8, 0u8]);
        let size = 10;

        // Create a new ImgBuf with width and height and grey background
        let mut image: RgbImage = image::ImageBuffer::from_pixel(
            size * (self.columns + 2),
            size * (self.rows + 2),
            Rgb([128u8, 128u8, 128u8]),
        );

        // fill in the maze with white and draw a black outline
        drawing::draw_filled_rect_mut(
            &mut image,
            rect::Rect::at((size - 1) as i32, (size - 1) as i32)
                .of_size((size * self.columns) + 1, (size * self.rows) + 1),
            black,
        );

        for cell in &self.cells {
            if let Some(c) = cell {
                // cut our valid cells
                drawing::draw_filled_rect_mut(
                    &mut image,
                    rect::Rect::at((size * (c.column + 1)) as i32, (size * (c.row + 1)) as i32)
                        .of_size(size - 1, size - 1),
                    white,
                );
                // cut out wall from top-right to bottom-right
                if Grid::has_link(self, &cell, Direction::East) {
                    drawing::draw_line_segment_mut(
                        &mut image,
                        (
                            ((size * (c.column + 2)) - 1) as f32,
                            (size * (c.row + 1)) as f32,
                        ),
                        (
                            ((size * (c.column + 2)) - 1) as f32,
                            ((size * (c.row + 2)) - 2) as f32,
                        ),
                        white,
                    );
                }
                // cut out wall from bottom-left to bottom-right
                if Grid::has_link(self, &cell, Direction::South) {
                    drawing::draw_line_segment_mut(
                        &mut image,
                        (
                            (size * (c.column + 1)) as f32,
                            ((size * (c.row + 2)) - 1) as f32,
                        ),
                        (
                            ((size * (c.column + 2)) - 2) as f32,
                            ((size * (c.row + 2)) - 1) as f32,
                        ),
                        white,
                    );
                }
            }
        }
        image
    }

    pub fn draw(&self, filename: &str) -> ImageResult<()> {
        let image = self._draw();

        // Write the contents of this image to the Writer in PNG format.
        image.save_with_format(filename, ImageFormat::Png)
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const VDIV: char = '|';
        const HDIV: char = '-';
        const CORNER: char = '+';
        const CELL: char = ' ';
        const LINK: char = ' ';
        const NONE: char = '█';

        let mut s = String::new();

        for row in 0..self.rows {
            // print top row, can ignore all cells for now
            let start = (row * self.columns) as usize;
            let end = start + self.columns as usize;
            let cells = &self.cells[start..end];

            // write an unconditional top line
            if row == 0 {
                self.write_row(&mut s, 3, cells, |_, _| CORNER, |_, _| HDIV);
            }
            // write the cell body and vertical dividers
            // mark None cells as X
            // skip divider if the cell as an East link
            self.write_row(
                &mut s,
                3,
                cells,
                |g, c| {
                    if Grid::has_link(g, c, Direction::East) {
                        LINK
                    } else {
                        VDIV
                    }
                },
                |_, c| match c {
                    Option::Some(_) => CELL,
                    Option::None => NONE,
                },
            );
            // write cell corners and horizontal dividers
            // skip dividier if the cell has a South link
            self.write_row(
                &mut s,
                3,
                cells,
                |_, _| CORNER,
                |g, c| {
                    if Grid::has_link(g, c, Direction::South) {
                        LINK
                    } else {
                        HDIV
                    }
                },
            );
        }
        f.write_str(&s)
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
        let grid = Grid::grid(2, 3, Grid::ALLOW_ALL, &mut NoOp {});

        assert_eq!(grid.cells.len(), 6);
    }

    #[test]
    fn check_valid_cell_count() {
        let grid = Grid::grid(2, 3, Grid::ALLOW_ALL, &mut NoOp {});

        assert_eq!(grid.cells().len(), 6);
    }

    #[test]
    fn check_cell_position() {
        let grid = Grid::grid(2, 3, Grid::ALLOW_ALL, &mut NoOp {});

        for row in 0..grid.rows {
            for column in 0..grid.columns {
                let cell = grid
                    .cell(row, column)
                    .unwrap_or_else(|| panic!("Missing Cell {},{}", column, row));

                assert_eq!(cell.row, row);
                assert_eq!(cell.column, column);
            }
        }
    }

    #[test]
    fn check_bounds() {
        let grid = Grid::square(3);

        assert!(matches!(grid.cell(0, 3), None));
        assert!(matches!(grid.cell(4, 0), None));
    }

    #[test]
    fn check_neighbour_top_left() {
        let grid = Grid::square(3);
        let cell = grid.cell(0, 0).expect("Missing Cell 0,0");

        assert!(matches!(grid.neighbour(cell, Direction::North), None));
        assert!(matches!(grid.neighbour(cell, Direction::West), None));
        assert_eq!(grid.neighbour(cell, Direction::South), grid.cell(1, 0));
        assert_eq!(grid.neighbour(cell, Direction::East), grid.cell(0, 1));
    }

    #[test]
    fn check_neighbour_top_right() {
        let grid = Grid::square(3);
        let cell = grid.cell(0, 2).expect("Missing Cell");

        assert!(matches!(grid.neighbour(cell, Direction::North), None));
        assert_eq!(grid.neighbour(cell, Direction::West), grid.cell(0, 1));
        assert_eq!(grid.neighbour(cell, Direction::South), grid.cell(1, 2));
        assert!(matches!(grid.neighbour(cell, Direction::East), None));
    }
    #[test]
    fn check_neighbour_center() {
        let grid = Grid::square(3);
        let cell = grid.cell(1, 1).expect("Missing Cell 1,1");

        assert_eq!(grid.neighbour(cell, Direction::North), grid.cell(0, 1));
        assert_eq!(grid.neighbour(cell, Direction::West), grid.cell(1, 0));
        assert_eq!(grid.neighbour(cell, Direction::South), grid.cell(2, 1));
        assert_eq!(grid.neighbour(cell, Direction::East), grid.cell(1, 2));
    }

    #[test]
    fn check_neighbour_bottom_left() {
        let grid = Grid::square(3);
        let cell = grid.cell(2, 0).expect("Missing Cell");
        assert_eq!(grid.neighbour(cell, Direction::North), grid.cell(1, 0));
        assert!(matches!(grid.neighbour(cell, Direction::West), None));
        assert!(matches!(grid.neighbour(cell, Direction::South), None));
        assert_eq!(grid.neighbour(cell, Direction::East), grid.cell(2, 1));
    }

    #[test]
    fn check_neighbour_bottom_right() {
        let grid = Grid::square(3);
        let cell = grid.cell(2, 2).expect("Missing Cell 2,2");

        assert_eq!(grid.neighbour(cell, Direction::North), grid.cell(1, 2));
        assert_eq!(grid.neighbour(cell, Direction::West), grid.cell(2, 1));
        assert!(matches!(grid.neighbour(cell, Direction::South), None));
        assert!(matches!(grid.neighbour(cell, Direction::East), None));
    }

    #[test]
    fn check_neighbours_top_left() {
        let grid = Grid::square(3);
        let cell = grid.cell(0, 0).expect("Missing Cell 0,0");

        let neighbours = grid.neighbours(cell);
        assert!(!neighbours.contains_key(&Direction::North));
        assert!(neighbours.contains_key(&Direction::East));
        assert!(neighbours.contains_key(&Direction::South));
        assert!(!neighbours.contains_key(&Direction::West));

        let neighbour = neighbours.get(&Direction::East);
        assert_eq!(
            neighbour.expect("Missing Cell 0,1"),
            &Cell { row: 0, column: 1 }
        );
    }

    #[test]
    fn check_neighbours_center() {
        let grid = Grid::square(3);
        let cell = grid.cell(1, 1).expect("Missing Cell 1,1");

        let neighbours = grid.neighbours(cell);
        assert!(neighbours.contains_key(&Direction::North));
        assert!(neighbours.contains_key(&Direction::East));
        assert!(neighbours.contains_key(&Direction::South));
        assert!(neighbours.contains_key(&Direction::West));
    }

    #[test]
    fn check_neighbours_bottom_right() {
        let grid = Grid::square(3);
        let cell = grid.cell(2, 2).expect("Missing Cell 2,2");

        let neighbours = grid.neighbours(cell);
        assert!(neighbours.contains_key(&Direction::North));
        assert!(!neighbours.contains_key(&Direction::East));
        assert!(!neighbours.contains_key(&Direction::South));
        assert!(neighbours.contains_key(&Direction::West));
    }

    #[test]
    fn check_masked() {
        let alternate = |r, c| r % 2 != c % 2;
        let grid = Grid::grid(2, 3, alternate, &mut NoOp {});
        assert_eq!(grid.cells.len(), 6);
        assert_eq!(grid.cells().len(), 3);

        assert!(matches!(grid.cell(0, 0), None));
        assert!(matches!(grid.cell(0, 1), Some(_)));
    }

    #[test]
    fn check_link() {
        let mut grid = Grid::square(2);

        let cell_01 = *grid.cell(0, 1).expect("Missing Cell 0,1");
        let cell_11 = *grid.cell(1, 1).expect("Missing Cell 1,1");

        // add link from 1,1 North
        assert!(matches!(
            grid.link_cell(&cell_11, Direction::North),
            Some(_)
        ));

        assert!(grid.links(&cell_01).contains(&Direction::South));
        assert!(grid.links(&cell_11).contains(&Direction::North));
    }

    #[test]
    fn check_invalid_link() {
        let mut grid = Grid::square(2);

        let cell_01 = *grid.cell(0, 1).expect("Missing Cell 0,1");

        // add link from 1,1 North
        assert!(matches!(grid.link_cell(&cell_01, Direction::North), None));
        assert!(grid.links(&cell_01).is_empty());
    }

    #[test]
    fn check_unlink() {
        let mut grid = Grid::square(2);

        let cell_01 = *grid.cell(0, 1).expect("Missing Cell 0,1");
        let cell_11 = *grid.cell(1, 1).expect("Missing Cell 1,1");

        // add link from 1,1 North
        assert_eq!(
            grid.link_cell(&cell_11, Direction::North)
                .expect("Missing Cell 1,1"),
            cell_01
        );

        // remove the link from the South
        assert_eq!(
            grid.unlink_cell(&cell_01, Direction::South)
                .expect("Missing Cell 0,1"),
            cell_11
        );

        assert!(grid.links(&cell_01).is_empty());
        assert!(grid.links(&cell_11).is_empty());
    }

    #[test]
    fn check_string_masked() {
        let newline: String = String::from("\n");
        let grid = Grid::grid(2, 2, |r, c| r != 0 || c != 0, &mut NoOp {});

        assert_eq!(
            newline + &grid.to_string(),
            r#"
+---+---+
|███|   |
+---+---+
|   |   |
+---+---+
"#
        );
    }

    #[test]
    fn check_string_linked() {
        let newline: String = String::from("\n");
        let mut grid = Grid::square(2);

        let cell_00 = *grid.cell(0, 0).expect("Missing Cell 0,0");
        let cell_11 = *grid.cell(1, 1).expect("Missing Cell 1,1");

        // add links from 0,0 East and 1,1 North
        grid.link_cell(&cell_00, Direction::East);
        grid.link_cell(&cell_11, Direction::North);
        assert_eq!(
            newline + &grid.to_string(),
            r#"
+---+---+
|       |
+---+   +
|   |   |
+---+---+
"#
        );
    }

    #[test]
    fn check_draw() {
        let mut grid = Grid::grid(
            5,
            5,
            |r, c| !((r == 0 || r == 4) && (c == 0 || c == 4)),
            &mut NoOp {},
        );
        let cell = *grid.cell(2, 2).expect("Missing Cell 2,2");
        grid.link_cell(&cell, Direction::North);
        grid.link_cell(&cell, Direction::South);
        grid.link_cell(&cell, Direction::East);
        grid.link_cell(&cell, Direction::West);

        let image = grid._draw();

        assert_eq!(image.width(), 70);
        assert_eq!(image.height(), 70);
        assert_eq!(image.get_pixel(5, 5), &Rgb([128u8, 128u8, 128u8])); // border = grey
        assert_eq!(image.get_pixel(15, 15), &Rgb([0u8, 0u8, 0u8])); // masked cell = black
        assert_eq!(image.get_pixel(25, 25), &Rgb([255u8, 255u8, 255u8])); // valid cell = white
    }
}
