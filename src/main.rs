mod grid;

fn main() {
    // let grid = grid::Grid { rows: 2, columns: 3 };
    let grid = grid::Grid::grid(2, 3);

    println!("grid = {},{}", grid.rows(), grid.columns());
    println!("grid = {:?}", grid);

    let cell = grid.cell(1, 1);

    println!("cell = {:?}", cell);
    println!(
        "N = {:?}, E = {}, S = {:?}, W = {:?}",
        grid::print_cell(grid.neighbour(cell, grid::Direction::North)),
        grid::print_cell(grid.neighbour(cell, grid::Direction::East)),
        grid::print_cell(grid.neighbour(cell, grid::Direction::South)),
        grid::print_cell(grid.neighbour(cell, grid::Direction::West))
    );

}
