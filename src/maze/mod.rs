pub mod grid;

use crate::maze::internal::{Attributes, MazeAccessor};
use crate::solver::Distances;

use image::{ImageFormat, ImageResult};
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

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

pub trait Direction: Eq + Hash + Clone + Copy {
    fn reverse(&self) -> Self;

    fn neighbour(&self, row: u32, column: u32) -> (u32, u32);

    fn checked_neighbour(
        &self,
        rows: u32,
        columns: u32,
        row: u32,
        column: u32,
    ) -> Option<(u32, u32)>;

    fn offset(rows: u32, columns: u32, row: u32, column: u32) -> Option<usize>;

    fn all() -> Vec<Self>;
}

mod internal {
    use crate::maze::{Cell, Direction};

    use std::collections::{HashMap, HashSet};

    #[derive(Debug)]
    pub struct Attributes<T> {
        pub(super) neighbours: HashMap<T, Cell>,
        pub(super) links: HashSet<T>,
        pub(super) distance: Option<u32>,
    }

    impl<T: Direction> Attributes<T> {
        pub(super) fn new(neighbours: HashMap<T, Cell>) -> Attributes<T> {
            Attributes {
                neighbours,
                links: HashSet::new(),
                distance: None,
            }
        }

        pub(super) fn has_link(&self, direction: &T) -> bool {
            self.links.contains(&direction)
        }

        pub(super) fn add_link(&mut self, direction: &T) -> bool {
            self.links.insert(*direction)
        }

        pub(super) fn remove_link(&mut self, direction: &T) -> bool {
            self.links.remove(direction)
        }

        pub(super) fn distance(&self) -> Option<u32> {
            self.distance
        }
    }

    pub trait MazeAccessor<T: Direction> {
        fn _raw_cells(&self) -> &[Option<Cell>];

        fn _set_distance(&mut self, max: Option<u32>);

        fn _attributes(&self, cell: &Cell) -> &Attributes<T>;

        fn _attributes_mut(&mut self, cell: &Cell) -> &mut Attributes<T>;
    }
}

#[allow(dead_code)]
pub trait Maze<T: Direction>: MazeAccessor<T> + Debug {
    /// Masking function that allows all cells
    const ALLOW_ALL: &'static dyn Fn(u32, u32) -> bool = &|_, _| true;

    fn rows(&self) -> u32;

    fn columns(&self) -> u32;

    fn size(&self) -> (u32, u32) {
        (self.rows(), self.columns())
    }

    /// Return a list of valid cells, exclude any that have been masked
    fn cells(&self) -> Vec<&Cell> {
        self._raw_cells()
            .iter()
            .filter_map(|x| x.as_ref())
            .collect()
    }

    /// Return the cell at the row and column, or None if the cell is masked
    ///
    /// # Arguments
    /// * `row` - grid row
    /// * `column` - grid column
    fn cell(&self, row: u32, column: u32) -> Option<&Cell> {
        match T::offset(self.rows(), self.columns(), row, column) {
            Some(offset) => match self._raw_cells().get(offset) {
                Some(c) => c.as_ref(),
                None => None,
            },
            None => None,
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
    // fn neighbour(&self, cell: &Cell, direction: T) -> Option<&Cell>;

    fn neighbours(&self, cell: &Cell) -> &HashMap<T, Cell> {
        &self._attributes(cell).neighbours
    }

    fn links(&self, cell: &Cell) -> &HashSet<T> {
        &self._attributes(cell).links
    }

    fn has_link(&self, cell: &Option<Cell>, direction: T) -> bool {
        match cell {
            Some(c) => self._attributes(c).has_link(&direction),
            None => false,
        }
    }

    fn link_cell(&mut self, cell: &Cell, direction: T) -> Option<Cell> {
        let neighbour = self.neighbours(cell).get(&direction);
        match neighbour {
            Some(c) => {
                let to = *c;

                self._attributes_mut(&*cell).add_link(&direction);
                self._attributes_mut(&to).add_link(&direction.reverse());

                Some(to)
            }
            None => None,
        }
    }

    fn unlink_cell(&mut self, cell: &Cell, direction: T) -> Option<Cell> {
        let neighbour = self.neighbours(cell).get(&direction);
        match neighbour {
            Some(c) => {
                let to = *c;

                self._attributes_mut(&*cell).remove_link(&direction);
                self._attributes_mut(&to).remove_link(&direction.reverse());

                Some(to)
            }
            None => None,
        }
    }

    fn apply_distances(&mut self, distances: Distances) {
        let mut max = 0u32;
        for (cell, distance) in distances.all_cells() {
            max = cmp::max(max, *distance);
            self._attributes_mut(cell).distance = Some(*distance);
        }
        self._set_distance(Some(max));
    }

    fn _build_cells<F>(rows: u32, columns: u32, allowed: F) -> Vec<Option<Cell>>
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

    fn _build_attributes(
        cells: &[Option<Cell>],
        rows: u32,
        columns: u32,
    ) -> HashMap<Cell, Attributes<T>> {
        let mut attributes = HashMap::with_capacity((rows * columns) as usize);

        for element in cells {
            if let Some(cell) = element {
                attributes.insert(
                    *cell,
                    Attributes::new(Self::_neighbours(&cells, rows, columns, &cell)),
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
    ) -> HashMap<T, Cell> {
        let mut neighbours = HashMap::new();

        for direction in T::all() {
            if let Some((row, column)) =
                direction.checked_neighbour(rows, columns, cell.row(), cell.column())
            {
                if let Some(offset) = T::offset(rows, columns, row, column) {
                    if let Some(c) = cells[offset] {
                        neighbours.insert(direction, c);
                    }
                }
            }
        }
        neighbours
    }

    fn draw_image(&self) -> image::RgbImage;

    fn draw(&self, filename: &str) -> ImageResult<()> {
        let image = self.draw_image();

        // Write the contents of this image to the Writer in PNG format.
        image.save_with_format(filename, ImageFormat::Png)
    }

    fn write_row<F1, F2>(&self, s: &mut String, scale: u32, row: &[Option<Cell>], f1: F1, f2: F2)
    where
        F1: Fn(&Self, &Option<Cell>) -> char,
        F2: Fn(&Self, &Option<Cell>) -> (char, char),
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
