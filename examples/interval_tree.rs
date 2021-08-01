use space_partitioning::interval_tree::{Interval, IntervalTree};
use std::iter::FromIterator;

fn main() {
    let tree = IntervalTree::from_iter([
        (15..=20, "now"),
        (10..=30, "data"),
        (17..=19, "correctly"),
        (5..=20, "this"),
        (12..=15, "is"),
        (30..=40, "ordered"),
    ]);

    println!("Inorder traversal of constructed Interval Tree:");
    for entry in tree.iter_inorder() {
        println!("{:?}", entry);
    }

    let x = Interval::from(6..=7);
    println!("Searching for interval {:?}.", x);
    if let Some(interval) = tree.overlap_search(x) {
        println!("Overlaps with {:?}.", interval);
    } else {
        println!("No overlapping interval.")
    }
}
