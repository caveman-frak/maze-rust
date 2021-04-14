use crate::maze::{Attributes, Cell, Direction};
use crate::router::{NoOp, Router};
use crate::solver::Distances;
use crate::util;

use image::{ImageFormat, ImageResult, Rgb, RgbImage};
use imageproc::{drawing, rect};
use std::char;
use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Compass {
    North,
    East,
    South,
    West,
}

impl Compass {
    pub fn iter() -> std::slice::Iter<'static, Compass> {
        [Compass::North, Compass::East, Compass::South, Compass::West].iter()
    }
}

impl Direction for Compass {
    fn reverse(&self) -> Compass {
        match self {
            Compass::North => Compass::South,
            Compass::East => Compass::West,
            Compass::South => Compass::North,
            Compass::West => Compass::East,
        }
    }

    fn neighbour(&self, row: u32, column: u32) -> (u32, u32) {
        match self {
            Compass::North => (row - 1, column),
            Compass::East => (row, column + 1),
            Compass::South => (row + 1, column),
            Compass::West => (row, column - 1),
        }
    }

    fn checked_neighbour(
        &self,
        rows: u32,
        columns: u32,
        row: u32,
        column: u32,
    ) -> Option<(u32, u32)> {
        match self {
            Compass::North if row > 0 => Some(self.neighbour(row, column)),
            Compass::East if column < columns - 1 => Some(self.neighbour(row, column)),
            Compass::South if row < rows - 1 => Some(self.neighbour(row, column)),
            Compass::West if column > 0 => Some(self.neighbour(row, column)),
            _ => None,
        }
    }

    fn offset(rows: u32, columns: u32, row: u32, column: u32) -> Option<usize> {
        if row >= rows || column >= columns {
            None
        } else {
            Some((row * columns + column) as usize)
        }
    }

    // fn all<Compass>() -> std::slice::Iter<'static, Compass> {
    //     let mut v: Vec<Compass> = Vec::new();
    //     v.iter()
    // }
}

#[derive(Debug)]
pub struct Grid {
    rows: u32,
    columns: u32,
    cells: Vec<Option<Cell>>,
    attributes: HashMap<Cell, Attributes<Compass>>,
    max_distance: Option<u32>,
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
            max_distance: None,
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

    pub fn size(&self) -> (u32, u32) {
        (self.rows, self.columns)
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
        if let Some(offset) = Compass::offset(self.rows, self.columns, row, column) {
            self.cells[offset].as_ref()
        } else {
            None
        }
    }

    /// Return the neighbouring cell if one exists, otherwise None
    ///
    /// # Arguments
    /// * `cell` - the base cell
    /// * `compass` - the compass of the neighbour
    ///
    /// ```
    ///     let grid = grid::square(3);
    ///     let cell = grid.cell(0, 0).expect("Missing Cell 0,0");
    ///     println!(
    ///         "neighbours -> N = {:?}, E = {:?}, S = {:?}, W = {:?}",
    ///         grid.neighbour(&cell, grid::Compass::North),
    ///         grid.neighbour(&cell, grid::Compass::East),
    ///         grid.neighbour(&cell, grid::Compass::South),
    ///         grid.neighbour(&cell, grid::Compass::West)
    ///     );
    /// ```
    pub fn neighbour(&self, cell: &Cell, compass: Compass) -> Option<&Cell> {
        self.attributes(cell).get_neighbour(&compass)
    }

    pub fn neighbours(&self, cell: &Cell) -> &HashMap<Compass, Cell> {
        &self.attributes(cell).neighbours
    }

    pub fn links(&self, cell: &Cell) -> &HashSet<Compass> {
        &self.attributes(cell).links
    }

    fn has_link(&self, cell: &Option<Cell>, compass: Compass) -> bool {
        match cell {
            Some(c) => self.attributes(c).has_link(&compass),
            None => false,
        }
    }

    fn attributes(&self, cell: &Cell) -> &Attributes<Compass> {
        self.attributes
            .get(cell)
            .unwrap_or_else(|| panic!("Missing attribute for {:?}", cell))
    }

    fn attributes_mut(&mut self, cell: &Cell) -> &mut Attributes<Compass> {
        self.attributes
            .get_mut(cell)
            .unwrap_or_else(|| panic!("Missing attribute for {:?}", cell))
    }

    pub fn link_cell(&mut self, cell: &Cell, compass: Compass) -> Option<Cell> {
        let neighbour = self.neighbour(cell, compass);
        match neighbour {
            Some(c) => {
                let to = *c;

                self.attributes_mut(&*cell).add_link(&compass);
                self.attributes_mut(&to).add_link(&compass.reverse());

                Some(to)
            }
            None => None,
        }
    }

    pub fn unlink_cell(&mut self, cell: &Cell, compass: Compass) -> Option<Cell> {
        let neighbour = self.neighbour(cell, compass);
        match neighbour {
            Some(c) => {
                let to = *c;

                self.attributes_mut(&*cell).remove_link(&compass);
                self.attributes_mut(&to).remove_link(&compass.reverse());

                Some(to)
            }
            None => None,
        }
    }

    pub fn apply_distances(&mut self, distances: Distances) {
        let mut max = 0u32;
        for (cell, distance) in distances.all_cells() {
            max = cmp::max(max, *distance);
            self.attributes_mut(cell).distance = Some(*distance);
        }
        self.max_distance = Some(max);
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
    ) -> HashMap<Cell, Attributes<Compass>> {
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
    ) -> HashMap<Compass, Cell> {
        let mut neighbours = HashMap::new();

        for compass in Compass::iter() {
            if let Some((row, column)) =
                compass.checked_neighbour(rows, columns, cell.row(), cell.column())
            {
                if let Some(offset) = Compass::offset(rows, columns, row, column) {
                    if let Some(c) = cells[offset] {
                        neighbours.insert(*compass, c);
                    }
                }
            }
        }
        neighbours
    }

    fn _draw(&self) -> image::RgbImage {
        const WHITE: Rgb<u8> = Rgb([255u8, 255u8, 255u8]);
        const BLACK: Rgb<u8> = Rgb([0u8, 0u8, 0u8]);
        const GREY: Rgb<u8> = Rgb([128u8, 128u8, 128u8]);
        const BLUE: Rgb<u8> = Rgb([0u8, 0u8, 255u8]);
        let size = 10;

        // Create a new ImgBuf with width and height and grey background
        let mut image: RgbImage =
            image::ImageBuffer::from_pixel(size * (self.columns + 2), size * (self.rows + 2), GREY);

        // fill in the maze with white and draw a black outline
        drawing::draw_filled_rect_mut(
            &mut image,
            rect::Rect::at((size - 1) as i32, (size - 1) as i32)
                .of_size((size * self.columns) + 1, (size * self.rows) + 1),
            BLACK,
        );

        for cell in &self.cells {
            if let Some(c) = cell {
                let colour = if let Some(distance) = self.attributes(c).distance() {
                    util::image::gradient_colour(
                        WHITE,
                        BLUE,
                        distance as f32 / self.max_distance.expect("Max distance not set") as f32,
                    )
                } else {
                    WHITE
                };

                // cut our valid cells
                drawing::draw_filled_rect_mut(
                    &mut image,
                    rect::Rect::at(
                        (size * (c.column() + 1)) as i32,
                        (size * (c.row() + 1)) as i32,
                    )
                    .of_size(size - 1, size - 1),
                    colour,
                );
                // cut out wall from top-right to bottom-right
                if self.has_link(&cell, Compass::East) {
                    drawing::draw_line_segment_mut(
                        &mut image,
                        (
                            ((size * (c.column() + 2)) - 1) as f32,
                            (size * (c.row() + 1)) as f32,
                        ),
                        (
                            ((size * (c.column() + 2)) - 1) as f32,
                            ((size * (c.row() + 2)) - 2) as f32,
                        ),
                        colour,
                    );
                }
                // cut out wall from bottom-left to bottom-right
                if self.has_link(&cell, Compass::South) {
                    drawing::draw_line_segment_mut(
                        &mut image,
                        (
                            (size * (c.column() + 1)) as f32,
                            ((size * (c.row() + 2)) - 1) as f32,
                        ),
                        (
                            ((size * (c.column() + 2)) - 2) as f32,
                            ((size * (c.row() + 2)) - 1) as f32,
                        ),
                        colour,
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

    fn write_row<F1, F2>(&self, s: &mut String, scale: u32, row: &[Option<Cell>], f1: F1, f2: F2)
    where
        F1: Fn(&Grid, &Option<Cell>) -> char,
        F2: Fn(&Grid, &Option<Cell>) -> (char, char),
    {
        s.push(f1(self, &None));
        for cell in row {
            let (ch, pad) = f2(self, cell);
            for i in 0..scale {
                s.push(if i == scale / 2 { ch } else { pad });
            }
            s.push(f1(self, cell));
        }
        s.push('\n');
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
                self.write_row(&mut s, 3, cells, |_, _| CORNER, |_, _| (HDIV, HDIV));
            }
            // write the cell body and vertical dividers
            // mark None cells as X
            // skip divider if the cell as an East link
            self.write_row(
                &mut s,
                3,
                cells,
                |g, c| {
                    if Grid::has_link(g, c, Compass::East) {
                        LINK
                    } else {
                        VDIV
                    }
                },
                |g, c| match c {
                    Option::Some(cell) => {
                        if let Some(distance) = g.attributes(cell).distance() {
                            if let Some(ch) = char::from_digit(distance, 36) {
                                return (ch, CELL);
                            }
                        }
                        (CELL, CELL)
                    }
                    Option::None => (NONE, NONE),
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
                    if Grid::has_link(g, c, Compass::South) {
                        (LINK, LINK)
                    } else {
                        (HDIV, HDIV)
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
    fn check_compass_all() {
        let mut compasss = Compass::iter();

        assert_eq!(compasss.next(), Some(&Compass::North));
        assert_eq!(compasss.next(), Some(&Compass::East));
        assert_eq!(compasss.next(), Some(&Compass::South));
        assert_eq!(compasss.next(), Some(&Compass::West));
        assert_eq!(compasss.next(), None);
    }

    #[test]
    fn check_compass_neighbour() {
        assert_eq!(Compass::North.neighbour(1, 1), (0, 1));
        assert_eq!(Compass::East.neighbour(1, 1), (1, 2));
        assert_eq!(Compass::South.neighbour(1, 1), (2, 1));
        assert_eq!(Compass::West.neighbour(1, 1), (1, 0));
    }

    #[test]
    fn check_compass_checked_neighbour() {
        assert_eq!(Compass::North.checked_neighbour(3, 3, 1, 1), Some((0, 1)));
        assert_eq!(Compass::East.checked_neighbour(3, 3, 1, 1), Some((1, 2)));
        assert_eq!(Compass::South.checked_neighbour(3, 3, 1, 1), Some((2, 1)));
        assert_eq!(Compass::West.checked_neighbour(3, 3, 1, 1), Some((1, 0)));
    }

    #[test]
    fn check_compass_checked_neighbour_fail() {
        assert_eq!(Compass::North.checked_neighbour(3, 3, 0, 1), None);
        assert_eq!(Compass::East.checked_neighbour(3, 3, 1, 2), None);
        assert_eq!(Compass::South.checked_neighbour(3, 3, 2, 1), None);
        assert_eq!(Compass::West.checked_neighbour(3, 3, 1, 0), None);
    }

    #[test]
    fn check_compass_offset() {
        assert_eq!(Compass::offset(3, 3, 0, 2), Some(2));
        assert_eq!(Compass::offset(3, 3, 1, 1), Some(4));
        assert_eq!(Compass::offset(3, 3, 2, 0), Some(6));
        assert_eq!(Compass::offset(3, 3, 3, 1), None);
        assert_eq!(Compass::offset(3, 3, 1, 3), None);
    }

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

                assert_eq!(cell.row(), row);
                assert_eq!(cell.column(), column);
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

        assert!(matches!(grid.neighbour(cell, Compass::North), None));
        assert!(matches!(grid.neighbour(cell, Compass::West), None));
        assert_eq!(grid.neighbour(cell, Compass::South), grid.cell(1, 0));
        assert_eq!(grid.neighbour(cell, Compass::East), grid.cell(0, 1));
    }

    #[test]
    fn check_neighbour_top_right() {
        let grid = Grid::square(3);
        let cell = grid.cell(0, 2).expect("Missing Cell");

        assert!(matches!(grid.neighbour(cell, Compass::North), None));
        assert_eq!(grid.neighbour(cell, Compass::West), grid.cell(0, 1));
        assert_eq!(grid.neighbour(cell, Compass::South), grid.cell(1, 2));
        assert!(matches!(grid.neighbour(cell, Compass::East), None));
    }
    #[test]
    fn check_neighbour_center() {
        let grid = Grid::square(3);
        let cell = grid.cell(1, 1).expect("Missing Cell 1,1");

        assert_eq!(grid.neighbour(cell, Compass::North), grid.cell(0, 1));
        assert_eq!(grid.neighbour(cell, Compass::West), grid.cell(1, 0));
        assert_eq!(grid.neighbour(cell, Compass::South), grid.cell(2, 1));
        assert_eq!(grid.neighbour(cell, Compass::East), grid.cell(1, 2));
    }

    #[test]
    fn check_neighbour_bottom_left() {
        let grid = Grid::square(3);
        let cell = grid.cell(2, 0).expect("Missing Cell");
        assert_eq!(grid.neighbour(cell, Compass::North), grid.cell(1, 0));
        assert!(matches!(grid.neighbour(cell, Compass::West), None));
        assert!(matches!(grid.neighbour(cell, Compass::South), None));
        assert_eq!(grid.neighbour(cell, Compass::East), grid.cell(2, 1));
    }

    #[test]
    fn check_neighbour_bottom_right() {
        let grid = Grid::square(3);
        let cell = grid.cell(2, 2).expect("Missing Cell 2,2");

        assert_eq!(grid.neighbour(cell, Compass::North), grid.cell(1, 2));
        assert_eq!(grid.neighbour(cell, Compass::West), grid.cell(2, 1));
        assert!(matches!(grid.neighbour(cell, Compass::South), None));
        assert!(matches!(grid.neighbour(cell, Compass::East), None));
    }

    #[test]
    fn check_neighbours_top_left() {
        let grid = Grid::square(3);
        let cell = grid.cell(0, 0).expect("Missing Cell 0,0");

        let neighbours = grid.neighbours(cell);
        assert!(!neighbours.contains_key(&Compass::North));
        assert!(neighbours.contains_key(&Compass::East));
        assert!(neighbours.contains_key(&Compass::South));
        assert!(!neighbours.contains_key(&Compass::West));

        let neighbour = neighbours.get(&Compass::East);
        assert_eq!(neighbour.expect("Missing Cell 0,1").coords(), (0, 1));
    }

    #[test]
    fn check_neighbours_center() {
        let grid = Grid::square(3);
        let cell = grid.cell(1, 1).expect("Missing Cell 1,1");

        let neighbours = grid.neighbours(cell);
        assert!(neighbours.contains_key(&Compass::North));
        assert!(neighbours.contains_key(&Compass::East));
        assert!(neighbours.contains_key(&Compass::South));
        assert!(neighbours.contains_key(&Compass::West));
    }

    #[test]
    fn check_neighbours_bottom_right() {
        let grid = Grid::square(3);
        let cell = grid.cell(2, 2).expect("Missing Cell 2,2");

        let neighbours = grid.neighbours(cell);
        assert!(neighbours.contains_key(&Compass::North));
        assert!(!neighbours.contains_key(&Compass::East));
        assert!(!neighbours.contains_key(&Compass::South));
        assert!(neighbours.contains_key(&Compass::West));
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
        assert!(matches!(grid.link_cell(&cell_11, Compass::North), Some(_)));

        assert!(grid.links(&cell_01).contains(&Compass::South));
        assert!(grid.links(&cell_11).contains(&Compass::North));
    }

    #[test]
    fn check_invalid_link() {
        let mut grid = Grid::square(2);

        let cell_01 = *grid.cell(0, 1).expect("Missing Cell 0,1");

        // add link from 1,1 North
        assert!(matches!(grid.link_cell(&cell_01, Compass::North), None));
        assert!(grid.links(&cell_01).is_empty());
    }

    #[test]
    fn check_unlink() {
        let mut grid = Grid::square(2);

        let cell_01 = *grid.cell(0, 1).expect("Missing Cell 0,1");
        let cell_11 = *grid.cell(1, 1).expect("Missing Cell 1,1");

        // add link from 1,1 North
        assert_eq!(
            grid.link_cell(&cell_11, Compass::North)
                .expect("Missing Cell 1,1"),
            cell_01
        );

        // remove the link from the South
        assert_eq!(
            grid.unlink_cell(&cell_01, Compass::South)
                .expect("Missing Cell 0,1"),
            cell_11
        );

        assert!(grid.links(&cell_01).is_empty());
        assert!(grid.links(&cell_11).is_empty());
    }

    #[test]
    fn check_string_masked() {
        let grid = Grid::grid(2, 2, |r, c| r != 0 || c != 0, &mut NoOp {});

        assert_eq!(
            format!("\n{}", grid),
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
        let mut grid = Grid::square(2);

        let cell_00 = *grid.cell(0, 0).expect("Missing Cell 0,0");
        let cell_11 = *grid.cell(1, 1).expect("Missing Cell 1,1");

        // add links from 0,0 East and 1,1 North
        grid.link_cell(&cell_00, Compass::East);
        grid.link_cell(&cell_11, Compass::North);
        assert_eq!(
            format!("\n{}", grid),
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
        grid.link_cell(&cell, Compass::North);
        grid.link_cell(&cell, Compass::South);
        grid.link_cell(&cell, Compass::East);
        grid.link_cell(&cell, Compass::West);

        let image = grid._draw();

        assert_eq!(image.width(), 70);
        assert_eq!(image.height(), 70);
        assert_eq!(image.get_pixel(5, 5), &Rgb([128u8, 128u8, 128u8])); // border = grey
        assert_eq!(image.get_pixel(15, 15), &Rgb([0u8, 0u8, 0u8])); // masked cell = black
        assert_eq!(image.get_pixel(25, 25), &Rgb([255u8, 255u8, 255u8])); // valid cell = white
    }
}
