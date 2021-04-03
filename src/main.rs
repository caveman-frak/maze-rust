mod grid;
mod router;

use crate::router::binarytree::BinaryTree;

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
}
