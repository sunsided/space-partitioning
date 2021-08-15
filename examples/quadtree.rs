use rand::Rng;
use space_partitioning::quadtree::{QuadRect, QuadTreeElement, AABB};
use space_partitioning::QuadTree;

struct Item {
    pub id: u32,
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
}

fn main() {
    let mut tree = QuadTree::new(QuadRect::new(0, 0, 1024, 1024), 8);

    for i in 0..1024 {
        let mut rng = rand::thread_rng();
        let item = Item {
            id: i as _,
            cx: rng.gen_range(0.0..1023.0),
            cy: rng.gen_range(0.0..1023.0),
            radius: rng.gen_range(0.5..64.0),
        };

        let item_aabb = AABB::new(
            (item.cx - item.radius).ceil() as _,
            (item.cy - item.radius).ceil() as _,
            (item.cx + item.radius).ceil() as _,
            (item.cy + item.radius).ceil() as _,
        );

        tree.insert(QuadTreeElement::new(item.id, item_aabb));
    }

    let candidates = tree.intersect_aabb(&AABB::new(128, 128, 256, 256));
    assert!(candidates.len() > 0);
}
