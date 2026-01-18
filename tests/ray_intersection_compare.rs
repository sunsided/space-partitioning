use space_partitioning::intersections::IntersectsWith;
use space_partitioning::quadtree::{AABB, QuadRect, QuadTreeElement};
use space_partitioning::rtree::{BoundingBox, RTree};
use space_partitioning::QuadTree;

#[derive(Clone, Copy)]
struct Ray2 {
    origin: [f32; 2],
    inv_dir: [f32; 2],
}

impl Ray2 {
    fn new(origin: [f32; 2], dir: [f32; 2]) -> Self {
        Self {
            origin,
            inv_dir: [1.0 / dir[0], 1.0 / dir[1]],
        }
    }
}

impl IntersectsWith<AABB> for Ray2 {
    fn intersects_with(&self, other: &AABB) -> bool {
        let min_x = other.tl.x as f32;
        let max_x = other.br.x as f32;
        let min_y = other.tl.y as f32;
        let max_y = other.br.y as f32;

        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;

        let t1 = (min_x - self.origin[0]) * self.inv_dir[0];
        let t2 = (max_x - self.origin[0]) * self.inv_dir[0];
        let (t1, t2) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
        tmin = tmin.max(t1);
        tmax = tmax.min(t2);

        let t3 = (min_y - self.origin[1]) * self.inv_dir[1];
        let t4 = (max_y - self.origin[1]) * self.inv_dir[1];
        let (t3, t4) = if t3 < t4 { (t3, t4) } else { (t4, t3) };
        tmin = tmin.max(t3);
        tmax = tmax.min(t4);

        if tmin > tmax {
            return false;
        }
        tmax >= 0.0
    }
}

impl IntersectsWith<BoundingBox<f32, 2>> for Ray2 {
    fn intersects_with(&self, other: &BoundingBox<f32, 2>) -> bool {
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;

        for i in 0..2 {
            let start = other.dims[i].start;
            let end = other.dims[i].end;
            let t1 = (start - self.origin[i]) * self.inv_dir[i];
            let t2 = (end - self.origin[i]) * self.inv_dir[i];
            let (t1, t2) = if t1 < t2 { (t1, t2) } else { (t2, t1) };

            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return false;
            }
        }

        tmax >= 0.0
    }
}

fn dedup_sorted(mut ids: Vec<u32>) -> Vec<u32> {
    ids.sort();
    ids.dedup();
    ids
}

#[test]
fn quadtree_rtree_ray_intersections_match() {
    let rects = [
        (10u32, (-20, -10, -5, -2)),
        (20u32, (-12, 4, -2, 12)),
        (30u32, (5, -15, 12, -3)),
        (40u32, (8, 1, 18, 9)),
        (50u32, (-2, -2, 2, 2)),
    ];

    let quad_bounds = QuadRect::new(-50, -50, 100, 100);
    let mut quadtree = QuadTree::new(quad_bounds, 6, 8, 1);
    let mut rtree: RTree<f32, 2, 8, u32> = RTree::default();

    for (id, (x1, y1, x2, y2)) in rects {
        quadtree
            .insert(QuadTreeElement::new(id, AABB::new(x1, y1, x2, y2)))
            .expect("insert should work");
        rtree.insert(
            id,
            BoundingBox::from([x1 as f32..=x2 as f32, y1 as f32..=y2 as f32]),
        );
    }

    let rays = [
        Ray2::new([-30.0, -5.0], [1.0, 0.0]),
        Ray2::new([-10.0, 6.0], [1.0, 0.2]),
        Ray2::new([0.0, -20.0], [0.0, 1.0]),
        Ray2::new([25.0, 5.0], [-1.0, -0.5]),
    ];

    for ray in rays {
        let qt_hits = dedup_sorted(quadtree.intersect_generic(&ray));
        let rt_hits = dedup_sorted(
            rtree
                .query_intersects_generic(&ray)
                .iter()
                .map(|entry| entry.id)
                .collect(),
        );
        assert_eq!(qt_hits, rt_hits);
    }
}
