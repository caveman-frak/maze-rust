use crate::maze::internal::{Attributes, MazeAccessor};
use crate::maze::{Cell, Direction, Maze};
use crate::router::internal::NoOp;
use crate::router::Router;
use crate::util::image::gradient_colour;

use image::{Rgb, RgbImage};
use imageproc::{drawing, rect};
use std::char;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Compass {
    North,
    East,
    South,
    West,
}

impl Direction for Compass {
    fn all() -> Vec<Compass> {
        vec![Compass::North, Compass::East, Compass::South, Compass::West]
    }

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
    pub fn grid<F>(
        rows: u32,
        columns: u32,
        allowed: F,
        router: &mut dyn Router<Compass, Grid>,
    ) -> Self
    where
        F: Fn(u32, u32) -> bool,
    {
        let cells = Grid::_build_cells(rows, columns, allowed);
        let attributes = Grid::_build_attributes(&cells, rows, columns);
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

    pub fn square(size: u32) -> Self {
        Grid::grid(size, size, Grid::ALLOW_ALL, &mut NoOp {})
    }
}

impl MazeAccessor<Compass> for Grid {
    fn _raw_cells(&self) -> &[Option<Cell>] {
        &self.cells
    }

    fn _set_distance(&mut self, max: Option<u32>) {
        self.max_distance = max;
    }

    fn _attributes(&self, cell: &Cell) -> &Attributes<Compass> {
        self.attributes
            .get(cell)
            .unwrap_or_else(|| panic!("Missing attribute for {:?}", cell))
    }

    fn _attributes_mut(&mut self, cell: &Cell) -> &mut Attributes<Compass> {
        self.attributes
            .get_mut(cell)
            .unwrap_or_else(|| panic!("Missing attribute for {:?}", cell))
    }
}

impl Maze<Compass> for Grid {
    fn rows(&self) -> u32 {
        self.rows
    }
    fn columns(&self) -> u32 {
        self.columns
    }

    fn draw_image(&self) -> image::RgbImage {
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
                let colour = if let Some(distance) = self._attributes(c).distance() {
                    gradient_colour(
                        WHITE,
                        BLUE,
                        distance as f32 / self.max_distance.expect("Max distance not set") as f32,
                    )
                } else {
                    WHITE
                };

                // cut out valid cells
                drawing::draw_filled_rect_mut(
                    &mut image,
                    rect::Rect::at(
                        (size * (c.column() + 1) + 1) as i32,
                        (size * (c.row() + 1) + 1) as i32,
                    )
                    .of_size(size - 3, size - 3),
                    colour,
                );
                // cut out wall from top-right to bottom-right
                if self.has_link(&cell, Compass::East) {
                    drawing::draw_filled_rect_mut(
                        &mut image,
                        rect::Rect::at(
                            (size * (c.column() + 2) - 2) as i32,
                            (size * (c.row() + 1) + 3) as i32,
                        )
                        .of_size(3, size - 7),
                        colour,
                    );
                }
                // cut out wall from bottom-left to bottom-right
                if self.has_link(&cell, Compass::South) {
                    drawing::draw_filled_rect_mut(
                        &mut image,
                        rect::Rect::at(
                            (size * (c.column() + 1) + 3) as i32,
                            (size * (c.row() + 2) - 2) as i32,
                        )
                        .of_size(size - 7, 3),
                        colour,
                    );
                }
            }
        }
        image
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
                        if let Some(distance) = g._attributes(cell).distance() {
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
    fn check_direction_points() {
        let direction = Compass::all();
        let mut points = direction.iter();

        assert_eq!(points.next(), Some(&Compass::North));
        assert_eq!(points.next(), Some(&Compass::East));
        assert_eq!(points.next(), Some(&Compass::South));
        assert_eq!(points.next(), Some(&Compass::West));
        assert_eq!(points.next(), None);
    }

    #[test]
    fn check_direction_neighbour() {
        assert_eq!(Compass::North.neighbour(1, 1), (0, 1));
        assert_eq!(Compass::East.neighbour(1, 1), (1, 2));
        assert_eq!(Compass::South.neighbour(1, 1), (2, 1));
        assert_eq!(Compass::West.neighbour(1, 1), (1, 0));
    }

    #[test]
    fn check_direction_checked_neighbour() {
        assert_eq!(Compass::North.checked_neighbour(3, 3, 1, 1), Some((0, 1)));
        assert_eq!(Compass::East.checked_neighbour(3, 3, 1, 1), Some((1, 2)));
        assert_eq!(Compass::South.checked_neighbour(3, 3, 1, 1), Some((2, 1)));
        assert_eq!(Compass::West.checked_neighbour(3, 3, 1, 1), Some((1, 0)));
    }

    #[test]
    fn check_direction_checked_neighbour_fail() {
        assert_eq!(Compass::North.checked_neighbour(3, 3, 0, 1), None);
        assert_eq!(Compass::East.checked_neighbour(3, 3, 1, 2), None);
        assert_eq!(Compass::South.checked_neighbour(3, 3, 2, 1), None);
        assert_eq!(Compass::West.checked_neighbour(3, 3, 1, 0), None);
    }

    #[test]
    fn check_direction_offset() {
        assert_eq!(Compass::offset(3, 3, 0, 2), Some(2));
        assert_eq!(Compass::offset(3, 3, 1, 1), Some(4));
        assert_eq!(Compass::offset(3, 3, 2, 0), Some(6));
        assert_eq!(Compass::offset(3, 3, 3, 1), None);
        assert_eq!(Compass::offset(3, 3, 1, 3), None);
    }

    #[test]
    fn check_direction_adjusted_columns() {
        assert_eq!(Compass::adjusted_columns(3, 3), 3);
        assert_eq!(Compass::adjusted_columns(3, 5), 5);
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
        let neighbours = grid.neighbours(cell);

        assert!(matches!(neighbours.get(&Compass::North), None));
        assert!(matches!(neighbours.get(&Compass::West), None));
        assert_eq!(neighbours.get(&Compass::South), grid.cell(1, 0));
        assert_eq!(neighbours.get(&Compass::East), grid.cell(0, 1));
    }

    #[test]
    fn check_neighbour_top_right() {
        let grid = Grid::square(3);
        let cell = grid.cell(0, 2).expect("Missing Cell");
        let neighbours = grid.neighbours(cell);

        assert!(matches!(neighbours.get(&Compass::North), None));
        assert_eq!(neighbours.get(&Compass::West), grid.cell(0, 1));
        assert_eq!(neighbours.get(&Compass::South), grid.cell(1, 2));
        assert!(matches!(neighbours.get(&Compass::East), None));
    }
    #[test]
    fn check_neighbour_center() {
        let grid = Grid::square(3);
        let cell = grid.cell(1, 1).expect("Missing Cell 1,1");
        let neighbours = grid.neighbours(cell);

        assert_eq!(neighbours.get(&Compass::North), grid.cell(0, 1));
        assert_eq!(neighbours.get(&Compass::West), grid.cell(1, 0));
        assert_eq!(neighbours.get(&Compass::South), grid.cell(2, 1));
        assert_eq!(neighbours.get(&Compass::East), grid.cell(1, 2));
    }

    #[test]
    fn check_neighbour_bottom_left() {
        let grid = Grid::square(3);
        let cell = grid.cell(2, 0).expect("Missing Cell");
        let neighbours = grid.neighbours(cell);

        assert_eq!(neighbours.get(&Compass::North), grid.cell(1, 0));
        assert!(matches!(neighbours.get(&Compass::West), None));
        assert!(matches!(neighbours.get(&Compass::South), None));
        assert_eq!(neighbours.get(&Compass::East), grid.cell(2, 1));
    }

    #[test]
    fn check_neighbour_bottom_right() {
        let grid = Grid::square(3);
        let cell = grid.cell(2, 2).expect("Missing Cell 2,2");
        let neighbours = grid.neighbours(cell);

        assert_eq!(neighbours.get(&Compass::North), grid.cell(1, 2));
        assert_eq!(neighbours.get(&Compass::West), grid.cell(2, 1));
        assert!(matches!(neighbours.get(&Compass::South), None));
        assert!(matches!(neighbours.get(&Compass::East), None));
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

        let image = grid.draw_image();

        assert_eq!(image.width(), 70);
        assert_eq!(image.height(), 70);
        assert_eq!(image.get_pixel(5, 5), &Rgb([128u8, 128u8, 128u8])); // border = grey
        assert_eq!(image.get_pixel(15, 15), &Rgb([0u8, 0u8, 0u8])); // masked cell = black
        assert_eq!(image.get_pixel(25, 25), &Rgb([255u8, 255u8, 255u8])); // valid cell = white
    }
}
