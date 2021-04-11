mod grid;
mod math;
mod router;
mod solver;

use crate::router::binarytree::BinaryTree;
use crate::router::sidewinder::SideWinder;
use crate::solver::dijkstra::Dijkstra;

fn main() {
    let mut rng = rand::thread_rng();
    print!(
        "{}",
        grid::Grid::grid(
            5,
            5,
            |r, c| !((r == 0 || r == 4) && (c == 0 || c == 4)),
            &mut BinaryTree::new(&mut rng),
        )
    );

    let mut grid = grid::Grid::grid(
        10,
        10,
        grid::Grid::ALLOW_ALL,
        &mut SideWinder::new(&mut rng),
    );
    grid.apply_distances(Dijkstra::solve(&grid, (0, 0)));
    grid.draw("target/maze.png")
        .expect("Could not write `target/maze.png`");

    print!("{}", grid);
}
