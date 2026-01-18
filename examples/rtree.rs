use space_partitioning::rtree::{BoundingBox, RTree};

fn main() {
    let mut tree: RTree<f32, 2, 4> = RTree::default();
    tree.insert(1, BoundingBox::from([0.0..=1.0, 0.0..=1.0]));
    tree.insert(2, BoundingBox::from([2.0..=3.0, 2.0..=3.0]));
    tree.insert(3, BoundingBox::from([4.0..=5.0, 4.0..=5.0]));
    tree.insert(4, BoundingBox::from([1.0..=2.0, 0.0..=1.0]));

    let intersects = tree.query_intersects(&BoundingBox::from([1.5..=4.5, 1.5..=4.5]));
    println!("intersects ids: {:?}", intersects.iter().map(|entry| entry.id).collect::<Vec<_>>());

    let contains = tree.query_contains(&BoundingBox::from([0.0..=3.0, 0.0..=3.0]));
    println!("contains ids: {:?}", contains.iter().map(|entry| entry.id).collect::<Vec<_>>());
}
