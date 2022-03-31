use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::{ChildNodes, GetBoundingBox, LeafNode, NonLeafNode};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

/// The R-Tree
///
/// ## Type parameters
/// * `T` - The coordinate type.
/// * `N` - The number of dimensions per coordinate.
/// * `M` - The maximum number of elements to store per leaf node.
#[derive(Debug)]
pub struct RTree<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    /// The root node. Default trees always start
    /// out with a leaf node that has zero elements.
    root: Rc<RefCell<NonLeafNode<T, N, M>>>,
}

impl<T, const N: usize, const M: usize> Default for RTree<T, N, M>
where
    T: DimensionType,
{
    fn default() -> Self {
        let root = NonLeafNode {
            bb: BoundingBox::default(),
            children: ChildNodes::Leaves(vec![]),
        };
        Self {
            root: Rc::new(RefCell::new(root)),
        }
    }
}

impl<T, const N: usize, const M: usize> RTree<T, N, M>
where
    T: DimensionType,
{
    /// Inserts an element into the tree.
    pub fn insert(&mut self, id: usize, bb: BoundingBox<T, N>) {
        let MatchingLeaf { leaf, parents } = self.choose_leaf(&bb);

        // Particularly if the tree is vanilla (e.g. default constructed)
        // there is no leaf node to choose. In this case, we attach a new leaf to the
        // deepest (and empty) non-leaf node; in case of the empty tree, this is the root.
        debug_assert_ne!(parents.len(), 0);
        let leaf = match leaf {
            Some(leaf) => leaf,
            None => Self::add_leaf_to_empty_node(parents.last().unwrap()),
        };

        let mut leaf = leaf.deref().borrow_mut();
        if !leaf.is_full() {
            leaf.insert(id, bb);
        } else {
            // Need to split the node here.
            let _new_leaf = Self::split_node(&mut leaf);

            // TODO: Insert the new leaf into the parent node.
            // TODO: Might need to propagate the split upwards if the parent becomes overfull.
            todo!();
        }

        // Citing https://iq.opengenus.org/r-tree/
        //
        // 1. Find position for new record:
        //      Invoke `choose_leaf` to select leaf node L in which to place the entry.
        // 2. Add record to leaf node.
        //      If L has room for another entry then add E, else
        //      invoke `split_node` to obtain L and LL (current leaf and new leaf containing all old entries of L)
        // 3. Propagate changes upward
        //      Invoke `adjust_tree` on L also passing LL if split was performed.
        // 4. Grow the tree taller
        //      If node split propagation caused the root to split, create a new root
        //      whose children are the two resulting nodes.
    }

    /// Select a leaf node in which to place a new entry.
    fn choose_leaf(&mut self, bb: &BoundingBox<T, N>) -> MatchingLeaf<T, N, M> {
        let mut parents = vec![];
        let mut current_node: Rc<RefCell<NonLeafNode<T, N, M>>> = self.root.clone();
        loop {
            parents.push(current_node.clone());

            let next_node = match &current_node.deref().borrow().children {
                ChildNodes::NonLeaves(non_leaves) => {
                    let smallest_idx =
                        Self::find_best_fitting_child_of_smallest_area(bb, non_leaves);
                    let next_node = non_leaves[smallest_idx].clone();

                    // Update this box's BB to fully contain the new object.
                    next_node.deref().borrow_mut().bb.grow(bb);

                    // Return next node for recursion.
                    next_node
                }
                ChildNodes::Leaves(leaves) => {
                    if leaves.is_empty() {
                        return MatchingLeaf {
                            leaf: None,
                            parents,
                        };
                    }

                    let smallest_idx = Self::find_best_fitting_child_of_smallest_area(bb, leaves);
                    return MatchingLeaf {
                        leaf: Some(leaves[smallest_idx].clone()),
                        parents,
                    };
                }
            };

            // Recurse.
            current_node = next_node;
        }
    }

    /// Adds a new leaf node to an empty non-leaf node.
    ///
    /// ## Arguments
    /// * `parent` - The parent node. Must be empty.
    ///
    /// ## Returns
    /// The new leaf node. This node is already registered as a child of the `parent`.
    fn add_leaf_to_empty_node(
        parent: &Rc<RefCell<NonLeafNode<T, N, M>>>,
    ) -> Rc<RefCell<LeafNode<T, N, M>>> {
        let mut parent = parent.deref().borrow_mut();
        assert!(parent.children.is_empty());

        let new_leaf = Rc::new(RefCell::new(LeafNode::default()));
        parent.children.to_leaves_mut().push(new_leaf.clone());
        new_leaf
    }

    /// Determines the child that either fully accepts the provided bounding
    /// box or requires the least increase in size; if multiple options exist, picks
    /// the first one of the smallest area.
    ///
    /// ## Arguments
    /// * `bb` - The bounding box of the object to add.
    /// * `leaves` - The vector of leaf node. Must not be empty.
    ///
    /// ## Returns
    /// Returns the index of the optimal fit.
    fn find_best_fitting_child_of_smallest_area<B>(
        bb: &BoundingBox<T, N>,
        leaves: &Vec<Rc<RefCell<B>>>,
    ) -> usize
    where
        B: GetBoundingBox<T, N>,
    {
        debug_assert!(!leaves.is_empty());
        let mut smallest_idx = usize::MAX;
        let mut smallest_area = T::max_value();
        let mut smallest_area_increase = T::max_value();

        for c in 0..leaves.len() {
            let leaf = leaves[c].deref().borrow();

            // If the bb is already fully contained by the leaf,
            // we ensure that we still pick the smallest leaf node.
            let grown = leaf.bb().get_grown(bb);
            let is_smaller_increase = grown.area_increase < smallest_area_increase;
            let is_same_increase = grown.area_increase == smallest_area_increase;
            let is_smaller_area = grown.area < smallest_area;

            // We keep the leaf that results in a smaller area increase
            // or, in case of a tie, with the smaller area.
            if is_smaller_increase || (is_same_increase && is_smaller_area) {
                smallest_idx = c;
                smallest_area = grown.area;
                smallest_area_increase = grown.area_increase;
            }
        }

        debug_assert_ne!(smallest_idx, usize::MAX);
        debug_assert_ne!(smallest_area, T::max_value());
        smallest_idx
    }

    fn adjust_tree(&self) {
        // Citing https://iq.opengenus.org/r-tree/
        //
        // 1. Initialize
        //      Set N=L (L being the leaf node)
        //      If L was split previously, set NN to be the resulting second node.
        // 2. Check if done
        //      If N is the root, stop
        // 3. Adjust covering rectangle in parent entry
        //      Let P be the parent node of N, and let EN be N's entry in P.
        //      Adjust EN so that it tightly encloses all entry rectangles in N.
        // 4. Propagate node split upward
        //      If N as a partner NN resulting from an earlier split,
        //      create a new entry ENN with ENN pointing to NN and ENN enclosing all
        //      rectangles in NN. Add ENN to P if there is room, otherwise invoke `split_node`
        //      to produce P and PP continuing ENN and all P's old entries.
        // 5. Move up to the next level
        //      Set N=P and set NN=PP if a split occurred. Repeat from step 2.
        todo!()
    }

    fn split_node(_leaf: &mut LeafNode<T, N, M>) -> LeafNode<T, N, M> {
        // - Exhaustive
        // - Quadratic-Cost
        // - Linear-Cost
        todo!()
    }
}

#[derive(Debug)]
struct MatchingLeaf<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    pub leaf: Option<Rc<RefCell<LeafNode<T, N, M>>>>,
    pub parents: Vec<Rc<RefCell<NonLeafNode<T, N, M>>>>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rtree::extent::Extent;
    use crate::rtree::nodes::Entry;
    use std::ops::Deref;

    #[test]
    fn default_works() {
        let r: RTree<f32, 2, 10> = RTree::default();
        assert!(r.root.deref().borrow().is_empty());
        assert_eq!(r.root.deref().borrow().len(), 0);
    }

    #[test]
    fn insert_works() {
        let mut tree = RTree::<f32, 2, 2>::default();
        tree.insert(0, BoundingBox::from([1.0..=2.0, 4.0..=17.0]));
        assert!(!tree.root.deref().borrow().is_empty());
        assert_eq!(tree.root.deref().borrow().len(), 1);
        // TODO: The tree dimensions must now match the object's.
    }

    #[test]
    fn split_works() {
        let test_a = Entry::new(0, [16.0..=68.0, 23.0..=35.0].into());
        let test_b = Entry::new(1, [55.0..=68.0, 12.0..=148.0].into());
        let test_c = Entry::new(2, [82.0..=94.0, 12.0..=148.0].into());
        let test_d = Entry::new(3, [82.0..=145.0, 30.0..=42.0].into());
        let mut entries = vec![test_a, test_b, test_c, test_d];

        // Build the entire area as we go; not needed if we already know it from the tree.
        let mut area = entries[0].bb.clone();
        for i in 1..entries.len() {
            area.grow(&entries[i].bb);
        }

        fn linear_pick_seeds<T: DimensionType, const N: usize>(
            set: &[Entry<T, N>],
            area: &BoundingBox<T, N>,
        ) -> (usize, usize) {
            debug_assert!(set.len() > 1);
            let mut highest_lows = vec![(T::min_value(), 0usize); N];
            let mut lowest_highs = vec![(T::max_value(), 0usize); N];

            for item_idx in 0..set.len() {
                let bb = &set[item_idx].bb;
                for dim in 0..N {
                    let extent = bb.dims[dim];

                    // Find the entry of the highest low dimension,
                    // i.e. the start coordinate being the highest.
                    if extent.start > highest_lows[dim].0 {
                        highest_lows[dim] = (extent.start, item_idx);
                    }

                    // Find the entry of the lowest high dimension.
                    // i.e. the end coordinate being the lowest.
                    if extent.end < lowest_highs[dim].0 {
                        lowest_highs[dim] = (extent.end, item_idx);
                    }
                }
            }

            let mut highest_separation = T::min_value();
            let (mut best_a, mut best_b) = (0usize, 0usize);

            for dim in 0..N {
                let width = area.dims[dim].len();

                let lo_hi = lowest_highs[dim].0;
                let hi_lo = highest_lows[dim].0;

                // Using a makeshift "abs" here to avoid requiring the Ord or Real trait.
                // TODO: Is the "abs" correct here? Issue might be arising from flipped coordinate systems (0 top-left or bottom-left)
                let sep_a = lo_hi - hi_lo;
                let sep_b = hi_lo - lo_hi;
                let separation = if sep_a > sep_b { sep_a } else { sep_b };

                let normalized_separation = separation / width;
                if normalized_separation > highest_separation {
                    highest_separation = separation;
                    best_a = lowest_highs[dim].1;
                    best_b = highest_lows[dim].1;
                }
            }

            debug_assert_ne!(best_a, best_b);
            let low_idx = best_a.min(best_b);
            let high_idx = best_a.max(best_b);
            (low_idx, high_idx)
        }

        // Find the best candidates and pop them from the set in reverse order (highest index first).
        let (best_a, best_b) = linear_pick_seeds(&entries, &area);
        let best_b = entries.remove(best_b);
        let best_a = entries.remove(best_a);

        let mut box_a = best_a.bb.clone();
        let mut box_b = best_b.bb.clone();

        let mut group_a = vec![best_a];
        let mut group_b = vec![best_b];

        // TODO: If one group has so few entries that the rest must be assigned for it to have the minimum number of elements, assign the rest and stop.
        for item in entries {
            let a_grown = box_a.get_grown(&item.bb);
            let b_grown = box_b.get_grown(&item.bb);

            // Assign to the box requiring a smaller increase in size.
            if a_grown.area_increase < b_grown.area_increase {
                box_a = a_grown.bb;
                group_a.push(item);
                continue;
            }
            if a_grown.area_increase > b_grown.area_increase {
                box_b = b_grown.bb;
                group_b.push(item);
                continue;
            }

            // In case of a tie, assign to the smaller box.
            if a_grown.area < b_grown.area {
                box_a = a_grown.bb;
                group_a.push(item);
                continue;
            }
            if a_grown.area > b_grown.area {
                box_b = b_grown.bb;
                group_b.push(item);
                continue;
            }

            // In case of a tie, assign to the box with fewer items,
            // or any box.
            if group_a.len() < group_b.len() {
                box_a = a_grown.bb;
                group_a.push(item);
                continue;
            }

            box_b = b_grown.bb;
            group_b.push(item);
        }

        // Group a contains both horizontal items.
        debug_assert!(group_a.iter().any(|x| x.id == 0));
        debug_assert!(group_a.iter().any(|x| x.id == 3));

        // Group a contains both vertical items.
        debug_assert!(group_b.iter().any(|x| x.id == 1));
        debug_assert!(group_b.iter().any(|x| x.id == 2));
    }
}
