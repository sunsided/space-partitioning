mod aabb;
mod centered_aabb;
mod error;
mod free_list;
mod node;
mod node_data;
mod node_info;
mod node_list;
mod point;
mod quad_rect;
mod quadrants;
mod quadtree;
mod quadtree_element;

pub use aabb::AABB;
pub use node_info::NodeInfo;
pub use point::Point;
pub use quad_rect::QuadRect;
pub use quadtree::{QuadTree, QuadTreeElement};

#[cfg(test)]
mod test {
    use super::*;
    use crate::quadtree::quadtree::build_test_tree;
    use std::iter::FromIterator;

    #[test]
    fn insert_once_works() {
        let mut tree = QuadTree::default();
        tree.insert(QuadTreeElement::new(0, AABB::default()))
            .expect("insert should work");
        assert_eq!(tree.count_element_references(), 1);

        let inserted_ids = tree.collect_ids();
        assert_eq!(inserted_ids.len(), 1);
        assert!(inserted_ids.contains(&0));
    }

    #[test]
    fn insert_twice_works() {
        let mut tree = QuadTree::default();
        let count = 2i32;
        for id in 0..count {
            tree.insert(QuadTreeElement::new(
                id,
                AABB::new(-id, -id, id + 1, id + 1),
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
    fn insert_a_lot_works() {
        let mut tree = QuadTree::new(QuadRect::new(-16, -16, 32, 32), 8, 1, 1);
        let count = 1024i32;
        let mut x = -16;
        let mut y = -16;
        for id in 0..count {
            tree.insert(QuadTreeElement::new(id, AABB::new(x, y, x + 1, y + 1)))
                .expect("insert should work");
            x += 1;
            if x == 16 {
                x = -16;
                y += 1;
            }
        }
        let inserted_ids = tree.collect_ids();
        assert_eq!(inserted_ids.len(), count as usize);
    }

    #[test]
    fn intersect_aabb_works() {
        let tree = build_test_tree();

        // Select the top-left quadrant
        let quadrant_tl = AABB::new(-17, -17, 0, 0);

        // Perform the actual intersection.
        let results = tree.intersect_aabb(&quadrant_tl);
        let results = Vec::from_iter(results);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1000));
        assert!(!results.contains(&1001));
        assert!(results.contains(&5000));
    }

    #[test]
    fn intersect_generic_works() {
        let tree = build_test_tree();

        // Select the top-left quadrant
        let quadrant_tl = AABB::new(-17, -17, 0, 0);

        // Perform the actual intersection.
        let results = tree.intersect_generic(&quadrant_tl);
        let results = Vec::from_iter(results);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1000));
        assert!(!results.contains(&1001));
        assert!(results.contains(&5000));
    }

    #[test]
    fn erase_last_works() {
        let mut tree = build_test_tree();

        // Erase the last-inserted node.
        assert!(tree.remove(&QuadTreeElement::new(5000, AABB::new(-5, -5, 5, 5))));
        assert_eq!(tree.collect_ids().len(), 5);
        assert_eq!(tree.count_element_references(), 5);

        // Since there are still populated child nodes, cleanup doesn't do anything.
        assert!(!tree.cleanup());
    }

    #[test]
    fn erase_first_works() {
        let mut tree = build_test_tree();

        // Erase the first-inserted node.
        assert!(tree.remove(&QuadTreeElement::new(1000, AABB::new(-15, -15, -5, -5))));
        assert_eq!(tree.collect_ids().len(), 5);
        assert_eq!(tree.count_element_references(), 8);

        // Since there are still populated child nodes, cleanup doesn't do anything.
        assert!(!tree.cleanup());
    }

    mod ray_box {
        use super::*;
        use crate::intersections::IntersectsWith;

        struct Ray {
            x: f32,
            y: f32,
            inv_dx: f32,
            inv_dy: f32,
        }

        impl Ray {
            fn new(x: f32, y: f32, dx: f32, dy: f32) -> Ray {
                Ray {
                    x,
                    y,
                    inv_dx: 1.0 / dx,
                    inv_dy: 1.0 / dy,
                }
            }
        }

        impl IntersectsWith<AABB> for Ray {
            fn intersects_with(&self, other: &AABB) -> bool {
                // https://gamedev.stackexchange.com/a/18459/10433

                let t1 = (other.tl.x as f32 - self.x) * self.inv_dx;
                let t2 = (other.br.x as f32 - self.x) * self.inv_dx;
                let t3 = (other.br.y as f32 - self.y) * self.inv_dy;
                let t4 = (other.tl.y as f32 - self.y) * self.inv_dy;

                let tmin = t1.min(t2).max(t3.min(t4));
                let tmax = t1.max(t2).min(t3.max(t4));

                // if tmax < 0, ray (line) is intersecting AABB, but the whole AABB is behind us
                // if tmin > tmax, ray doesn't intersect AABB
                if (tmax < 0.) | (tmin > tmax) {
                    return false;
                }

                return true;
            }
        }

        #[test]
        fn ray_box_intersection_works() {
            let r#box = AABB::new(-5, -5, 5, 5);
            let ray_in_box_pointing_up = Ray::new(0., 0., 0., 1.);
            let ray_on_left_pointing_right = Ray::new(-10., 0., 1., 0.);
            let ray_on_right_pointing_right = Ray::new(10., 0., 1., 0.);
            let ray_on_right_pointing_left = Ray::new(10., 0., -1., 0.);
            let ray_on_top_pointing_right = Ray::new(10., -10., 1., 0.);
            let ray_on_bottom_pointing_right = Ray::new(10., 10., 1., 0.);
            assert!(ray_in_box_pointing_up.intersects_with(&r#box));
            assert!(ray_on_left_pointing_right.intersects_with(&r#box));
            assert!(ray_on_right_pointing_left.intersects_with(&r#box));
            assert!(!ray_on_right_pointing_right.intersects_with(&r#box));
            assert!(!ray_on_top_pointing_right.intersects_with(&r#box));
            assert!(!ray_on_bottom_pointing_right.intersects_with(&r#box));

            let diagonal_ray = Ray::new(-10., 0., 1., 0.1);
            assert!(diagonal_ray.intersects_with(&r#box));
        }

        #[test]
        fn tree_ray_between_elements_does_not_intersect() {
            let tree = build_test_tree();
            let ray = Ray::new(1., 5., 1., 0.);
            let results = tree.intersect_generic(&ray);
            assert_eq!(results.len(), 0);
        }

        #[test]
        fn tree_ray_does_intersect() {
            let tree = build_test_tree();
            let ray = Ray::new(1., 8., 1., 0.);
            let results = tree.intersect_generic(&ray);
            assert_eq!(results.len(), 1);
            assert!(results.contains(&4000));
        }
    }
}
