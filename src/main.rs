mod maze;
mod router;
mod solver;
mod util;

use crate::maze::grid::{Compass, Grid};
use crate::maze::Maze;
use crate::router::binarytree::BinaryTree;
use crate::router::sidewinder::SideWinder;
use crate::solver::dijkstra::Dijkstra;

fn main() {
    let mut rng = rand::thread_rng();
    print!(
        "{}",
        Grid::grid(
            5,
            5,
            mask_corners(5, 5),
            &mut BinaryTree::<Compass>::new_for_compass(&mut rng),
        )
    );

    let mut grid = Grid::grid(
        10,
        10,
        Grid::ALLOW_ALL,
        &mut SideWinder::<Compass>::new_for_compass(&mut rng),
    );

    grid.apply_distances(Dijkstra::solve(&grid, (0, 0)));
    grid.draw("target/maze.png")
        .expect("Could not write `target/maze.png`");

    print!("{}", grid);
}

fn mask_corners(rows: u32, columns: u32) -> impl Fn(u32, u32) -> bool {
    move |r, c| !((r == 0 || r == rows - 1) && (c == 0 || c == columns - 1))
}
