pub mod grid;

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

// #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub trait Direction {
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

    // fn all<T: Direction>() -> std::slice::Iter<'static, T>;
}
