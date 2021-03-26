mod grid;

fn main() {
    let grid = grid::square(3);

    println!("grid = {}, len={}", grid, grid.cells().len());

    let cell = grid.cell(1, 1).unwrap();

    println!("cell = {:?} or {}", cell, cell);
    println!(
        "neighbours -> N = {}, E = {}, S = {}, W = {}",
        grid::print_cell(grid.neighbour(&cell, grid::Direction::North)),
        grid::print_cell(grid.neighbour(&cell, grid::Direction::East)),
        grid::print_cell(grid.neighbour(&cell, grid::Direction::South)),
        grid::print_cell(grid.neighbour(&cell, grid::Direction::West))
    );
    let cell = grid.cell(0, 0).unwrap();
    println!(
        "neighbours -> N = {:?}, E = {:?}, S = {:?}, W = {:?}",
        grid.neighbour(&cell, grid::Direction::North),
        grid.neighbour(&cell, grid::Direction::East),
        grid.neighbour(&cell, grid::Direction::South),
        grid.neighbour(&cell, grid::Direction::West)
    );

    println!("neighbours={:?}", grid.neighbours(cell));
}
