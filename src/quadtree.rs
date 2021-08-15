mod aabb;
mod centered_aabb;
mod free_list;
mod node;
mod node_data;
mod node_list;
mod quad_rect;
mod quadtree;

pub use aabb::AABB;
pub use quad_rect::QuadRect;
pub use quadtree::{QuadTree, QuadTreeElement};

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn insert_once_works() {
        let mut tree = QuadTree::default();
        tree.insert(QuadTreeElement::new(0, AABB::default()));
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
            ));
        }
        assert_eq!(tree.count_element_references(), 5);

        let inserted_ids = tree.collect_ids();
        assert_eq!(inserted_ids.len(), 2);
        assert!(inserted_ids.contains(&0));
        assert!(inserted_ids.contains(&1));
    }

    #[test]
    fn insert_a_lot_works() {
        let mut tree = QuadTree::new(QuadRect::new(-16, -16, 32, 32), 8);
        let count = 1024i32;
        let mut x = -16;
        let mut y = -16;
        for id in 0..count {
            tree.insert(QuadTreeElement::new(id, AABB::new(x, y, x + 1, y + 1)));
            x += 1;
            if x == 16 {
                x = -16;
                y += 1;
            }
        }
        assert_eq!(tree.count_element_references(), 1369 as usize);
        let tree = tree;

        let inserted_ids = tree.collect_ids();
        assert_eq!(inserted_ids.len(), count as usize);
    }

    #[test]
    fn find_works() {
        let quad_rect = QuadRect::new(-20, -20, 40, 40);
        let mut tree = QuadTree::new(quad_rect, 1);
        // top-left
        tree.insert(QuadTreeElement::new(1000, AABB::new(-15, -15, -5, -5)));
        tree.insert(QuadTreeElement::new(1001, AABB::new(-20, -20, -18, -18)));
        // top-right
        tree.insert(QuadTreeElement::new(2000, AABB::new(5, -15, 15, -5)));
        // bottom-left
        tree.insert(QuadTreeElement::new(3000, AABB::new(-15, 5, -5, 15)));
        // bottom-right
        tree.insert(QuadTreeElement::new(4000, AABB::new(5, 5, 15, 15)));
        // center
        tree.insert(QuadTreeElement::new(5000, AABB::new(-5, -5, 5, 5)));

        // The depth of 1 limits the tree to four quadrants.
        // Each of the first five elements creates a single reference
        // in each of the quadrants. The "center" element covers
        // all four quadrants, and therefore adds another four references.
        assert_eq!(tree.count_element_references(), 9);

        // Ensure we have the exact elements inserted.
        let inserted_ids = tree.collect_ids();
        assert_eq!(inserted_ids.len(), 6);
        assert!(inserted_ids.contains(&1000));
        assert!(inserted_ids.contains(&1001));
        assert!(inserted_ids.contains(&2000));
        assert!(inserted_ids.contains(&3000));
        assert!(inserted_ids.contains(&4000));
        assert!(inserted_ids.contains(&5000));

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
    fn erase_last_works() {
        let quad_rect = QuadRect::new(-20, -20, 40, 40);
        let mut tree = QuadTree::new(quad_rect, 1);
        // top-left
        tree.insert(QuadTreeElement::new(1000, AABB::new(-15, -15, -5, -5)));
        tree.insert(QuadTreeElement::new(1001, AABB::new(-20, -20, -18, -18)));
        // top-right
        tree.insert(QuadTreeElement::new(2000, AABB::new(5, -15, 15, -5)));
        // bottom-left
        tree.insert(QuadTreeElement::new(3000, AABB::new(-15, 5, -5, 15)));
        // bottom-right
        tree.insert(QuadTreeElement::new(4000, AABB::new(5, 5, 15, 15)));
        // center
        tree.insert(QuadTreeElement::new(5000, AABB::new(-5, -5, 5, 5)));

        // Similar to index test.
        assert_eq!(tree.collect_ids().len(), 6);
        assert_eq!(tree.count_element_references(), 9);

        // Erase the last-inserted node.
        assert!(tree.remove(&QuadTreeElement::new(5000, AABB::new(-5, -5, 5, 5))));
        assert_eq!(tree.collect_ids().len(), 5);
        assert_eq!(tree.count_element_references(), 5);

        // Since there are still populated child nodes, cleanup doesn't do anything.
        assert!(!tree.cleanup());
    }

    #[test]
    fn erase_first_works() {
        let quad_rect = QuadRect::new(-20, -20, 40, 40);
        let mut tree = QuadTree::new(quad_rect, 1);
        // top-left
        tree.insert(QuadTreeElement::new(1000, AABB::new(-15, -15, -5, -5)));
        tree.insert(QuadTreeElement::new(1001, AABB::new(-20, -20, -18, -18)));
        // top-right
        tree.insert(QuadTreeElement::new(2000, AABB::new(5, -15, 15, -5)));
        // bottom-left
        tree.insert(QuadTreeElement::new(3000, AABB::new(-15, 5, -5, 15)));
        // bottom-right
        tree.insert(QuadTreeElement::new(4000, AABB::new(5, 5, 15, 15)));
        // center
        tree.insert(QuadTreeElement::new(5000, AABB::new(-5, -5, 5, 5)));

        // Similar to index test.
        assert_eq!(tree.collect_ids().len(), 6);
        assert_eq!(tree.count_element_references(), 9);

        // Erase the first-inserted node.
        assert!(tree.remove(&QuadTreeElement::new(1000, AABB::new(-15, -15, -5, -5))));
        assert_eq!(tree.collect_ids().len(), 5);
        assert_eq!(tree.count_element_references(), 8);

        // Since there are still populated child nodes, cleanup doesn't do anything.
        assert!(!tree.cleanup());
    }
}
