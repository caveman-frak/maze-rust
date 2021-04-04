mod grid;
mod router;

use crate::router::binarytree::BinaryTree;
use crate::router::sidewinder::SideWinder;

fn main() {
    let mut rng = rand::thread_rng();
    let grid = grid::Grid::grid(
        5,
        5,
        |r, c| !((r == 0 || r == 4) && (c == 0 || c == 4)),
        &mut BinaryTree::new(&mut rng),
    );

    print!("{}", grid);

    grid.draw("target/maze.png")
        .expect("Could not write `target/maze.png`");

    print!(
        "{}",
        grid::Grid::grid(5, 5, grid::Grid::allow_all, &mut SideWinder::new(&mut rng),)
    );
}
