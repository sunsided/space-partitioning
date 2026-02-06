mod aabb;
mod centered_aabb;
mod error;
mod free_list;
mod node;
mod node_data;
mod node_info;
mod node_list;
mod oct_rect;
mod octants;
#[allow(clippy::module_inception)]
mod octree;
mod octree_element;
mod point;

pub use aabb::AABB;
pub use node_info::NodeInfo;
pub use oct_rect::OctRect;
pub use octree::{OctTree, OctTreeElement};
pub use point::Point;

#[cfg(test)]
mod test {
    use super::*;
    use crate::octree::octree::build_test_tree;

    #[test]
    fn insert_once_works() {
        let mut tree = OctTree::default();
        tree.insert(OctTreeElement::new(0, AABB::new(0, 0, 0, 1, 1, 1)))
            .expect("insert should work");
        assert_eq!(tree.count_element_references(), 1);

        let inserted_ids = tree.collect_ids();
        assert_eq!(inserted_ids.len(), 1);
        assert!(inserted_ids.contains(&0));
    }

    #[test]
    fn insert_twice_works() {
        let mut tree = OctTree::default();
        for id in 0..2i32 {
            tree.insert(OctTreeElement::new(
                id,
                AABB::new(-id, -id, -id, id + 1, id + 1, id + 1),
            ))
            .expect("insert should work");
        }
        assert_eq!(tree.count_element_references(), 2);

        let inserted_ids = tree.collect_ids();
        assert_eq!(inserted_ids.len(), 2);
        assert!(inserted_ids.contains(&0));
        assert!(inserted_ids.contains(&1));
    }

    #[test]
    fn intersect_aabb_works() {
        let tree = build_test_tree();

        let octant = AABB::new(-17, -17, -17, 0, 0, 0);
        let results = tree.intersect_aabb(&octant);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1000));
        assert!(!results.contains(&1001));
        assert!(results.contains(&5000));
    }

    #[test]
    fn intersect_generic_works() {
        let tree = build_test_tree();

        let octant = AABB::new(-17, -17, -17, 0, 0, 0);
        let results = tree.intersect_generic(&octant);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1000));
        assert!(!results.contains(&1001));
        assert!(results.contains(&5000));
    }

    mod ray_box {
        use super::*;
        use crate::intersections::IntersectsWith;

        struct Ray3 {
            origin: [f32; 3],
            inv_dir: [f32; 3],
        }

        impl Ray3 {
            fn new(origin: [f32; 3], dir: [f32; 3]) -> Ray3 {
                Ray3 {
                    origin,
                    inv_dir: [1.0 / dir[0], 1.0 / dir[1], 1.0 / dir[2]],
                }
            }
        }

        impl IntersectsWith<AABB> for Ray3 {
            fn intersects_with(&self, other: &AABB) -> bool {
                let mut tmin = f32::NEG_INFINITY;
                let mut tmax = f32::INFINITY;

                let bounds = [
                    (other.min.x as f32, other.max.x as f32),
                    (other.min.y as f32, other.max.y as f32),
                    (other.min.z as f32, other.max.z as f32),
                ];

                for axis in 0..3 {
                    let t1 = (bounds[axis].0 - self.origin[axis]) * self.inv_dir[axis];
                    let t2 = (bounds[axis].1 - self.origin[axis]) * self.inv_dir[axis];
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

        #[test]
        fn ray_box_intersection_works() {
            let r#box = AABB::new(-5, -5, -5, 5, 5, 5);
            let ray_inside = Ray3::new([0., 0., 0.], [0.1, 0.1, 1.]);
            let ray_outside = Ray3::new([20., 0., 0.], [1., 0.1, 0.1]);
            assert!(ray_inside.intersects_with(&r#box));
            assert!(!ray_outside.intersects_with(&r#box));
        }

        #[test]
        fn tree_ray_does_intersect() {
            let tree = build_test_tree();
            let ray = Ray3::new([1., 8., -10.], [1., 0.1, 1.]);
            let results = tree.intersect_generic(&ray);
            assert_eq!(results.len(), 1);
            assert!(results.contains(&4000));
        }
    }
}
