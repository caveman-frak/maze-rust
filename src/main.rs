mod grid;

fn main() {
    let mut grid = grid::Grid::grid(5, 5, |r, c| !((r == 0 || r == 4) && (c == 0 || c == 4)));
    let cell = grid.cell(2, 2).unwrap().clone();
    grid.link_cell(&cell, grid::Direction::North);
    grid.link_cell(&cell, grid::Direction::South);
    grid.link_cell(&cell, grid::Direction::East);
    grid.link_cell(&cell, grid::Direction::West);
    print!("{}", grid);
}
