use space_partitioning::rtree::BulkLoadStrategy;
use space_partitioning::rtree::{BoundingBox, RTree};

#[test]
fn insert_and_query_intersects_works() {
    let mut tree: RTree<f32, 2, 3> = RTree::default();
    tree.insert(10, BoundingBox::from([0.0..=1.0, 0.0..=1.0]));
    tree.insert(20, BoundingBox::from([2.0..=3.0, 2.0..=3.0]));
    tree.insert(30, BoundingBox::from([4.0..=5.0, 4.0..=5.0]));
    tree.insert(40, BoundingBox::from([1.0..=2.0, 0.0..=1.0]));

    let hits = tree.query_intersects(&BoundingBox::from([1.5..=4.5, 1.5..=4.5]));
    let mut ids: Vec<_> = hits.iter().map(|entry| entry.id).collect();
    ids.sort();

    assert_eq!(ids, vec![20, 30]);
}

#[test]
fn query_contains_works() {
    let mut tree: RTree<f32, 2, 2> = RTree::default();
    tree.insert(1, BoundingBox::from([0.0..=1.0, 0.0..=1.0]));
    tree.insert(2, BoundingBox::from([2.0..=3.0, 2.0..=3.0]));
    tree.insert(3, BoundingBox::from([4.0..=5.0, 4.0..=5.0]));
    tree.insert(4, BoundingBox::from([1.0..=2.0, 0.0..=1.0]));

    let hits = tree.query_contains(&BoundingBox::from([0.0..=3.0, 0.0..=3.0]));
    let mut ids: Vec<_> = hits.iter().map(|entry| entry.id).collect();
    ids.sort();

    assert_eq!(ids, vec![1, 2, 4]);
}

#[test]
fn remove_works() {
    let mut tree: RTree<f32, 2, 2> = RTree::default();
    let bb1 = BoundingBox::from([0.0..=1.0, 0.0..=1.0]);
    let bb2 = BoundingBox::from([2.0..=3.0, 2.0..=3.0]);
    let bb3 = BoundingBox::from([4.0..=5.0, 4.0..=5.0]);

    tree.insert(1, bb1.clone());
    tree.insert(2, bb2.clone());
    tree.insert(3, bb3.clone());

    assert!(tree.remove(&2, &bb2));
    assert!(!tree.remove(&2, &bb2));

    let hits = tree.query_intersects(&BoundingBox::from([1.5..=4.5, 1.5..=4.5]));
    let mut ids: Vec<_> = hits.iter().map(|entry| entry.id).collect();
    ids.sort();
    assert_eq!(ids, vec![3]);
}

#[test]
fn bulk_load_works() {
    let entries = vec![
        (10, BoundingBox::from([0.0..=1.0, 0.0..=1.0])),
        (20, BoundingBox::from([2.0..=3.0, 2.0..=3.0])),
        (30, BoundingBox::from([4.0..=5.0, 4.0..=5.0])),
        (40, BoundingBox::from([1.0..=2.0, 0.0..=1.0])),
    ];

    let tree: RTree<f32, 2, 2> = RTree::bulk_load(entries);
    let hits = tree.query_intersects(&BoundingBox::from([1.5..=4.5, 1.5..=4.5]));
    let mut ids: Vec<_> = hits.iter().map(|entry| entry.id).collect();
    ids.sort();

    assert_eq!(ids, vec![20, 30]);
}

#[test]
fn bulk_load_with_strategy_insertion_works() {
    let entries = vec![
        (10, BoundingBox::from([0.0..=1.0, 0.0..=1.0])),
        (20, BoundingBox::from([2.0..=3.0, 2.0..=3.0])),
        (30, BoundingBox::from([4.0..=5.0, 4.0..=5.0])),
        (40, BoundingBox::from([1.0..=2.0, 0.0..=1.0])),
    ];

    let tree: RTree<f32, 2, 2> =
        RTree::bulk_load_with_strategy(entries, BulkLoadStrategy::Insertion);
    let hits = tree.query_intersects(&BoundingBox::from([1.5..=4.5, 1.5..=4.5]));
    let mut ids: Vec<_> = hits.iter().map(|entry| entry.id).collect();
    ids.sort();

    assert_eq!(ids, vec![20, 30]);
}

#[test]
fn nearest_neighbor_works() {
    let mut tree: RTree<f32, 2, 3> = RTree::default();
    tree.insert(1, BoundingBox::from([0.0..=1.0, 0.0..=1.0]));
    tree.insert(2, BoundingBox::from([2.0..=3.0, 2.0..=3.0]));
    tree.insert(3, BoundingBox::from([4.0..=5.0, 4.0..=5.0]));

    let nearest = tree.nearest_neighbor([0.1, 0.2]).expect("nearest entry");
    assert_eq!(nearest.id, 1);
}
