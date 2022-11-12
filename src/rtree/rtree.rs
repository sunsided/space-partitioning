use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::prelude::*;
use crate::rtree::splitting_strategies::linear_cost_split::LinearCostSplitting;

/// The R-Tree
///
/// ## Type parameters
/// * `T` - The coordinate type.
/// * `N` - The number of dimensions per coordinate.
/// * `M` - The maximum number of elements to store per leaf node.
/// * `TupleIdentifier` - The type used to identify a tuple in application code.
#[derive(Debug)]
pub struct RTree<T, const N: usize, const M: usize, TupleIdentifier = usize>
where
    T: DimensionType,
{
    root: RTreeNode<T, N, M, TupleIdentifier>,
    split_strategy: LinearCostSplitting,
}

impl<T, const N: usize, const M: usize> RTree<T, N, M>
where
    T: DimensionType,
{
    /// Inserts an element into the tree.
    pub fn insert(&mut self, id: usize, bb: BoundingBox<T, N>) {
        // Citing https://iq.opengenus.org/r-tree/
        //
        // 1. Find position for new record:
        //      Invoke `choose_leaf` to select leaf node L in which to place the entry.
        let trail = self.choose_leaf(&bb);
        // 2. Add record to leaf node.
        //      If L has room for another entry then add E, else
        //      invoke `split_node` to obtain L and LL (current leaf and new leaf containing all old entries of L)
        if !node.is_full() {
            todo!("Add item to this leaf")
        } else {
            todo!("Split the node")
        }

        // 3. Propagate changes upward
        //      Invoke `adjust_tree` on L also passing LL if split was performed.
        // 4. Grow the tree taller
        //      If node split propagation caused the root to split, create a new root
        //      whose children are the two resulting nodes.

        todo!()
    }

    /// Select a leaf node in which to place a new entry.
    fn choose_leaf(
        &mut self,
        bb: &BoundingBox<T, N>,
    ) -> Vec<&mut RTreeNode<T, N, M, TupleIdentifier>> {
        // Citing https://iq.opengenus.org/r-tree/
        //
        // 1. Initialize
        //      Set N to be the root node
        let mut trail = vec![&mut self.root]; // no element = root node

        // 2. Leaf check
        //      If N is a leaf, return N
        if self.root.is_leaf() {
            return trail;
        }

        // 3. Choose subtree
        //      If N is a leaf, let F be the entry in N whose rectangle F1
        //      needs least enlargement to include E1. Resolve ties by choosing
        //      the entry with the rectangle of the smallest area.
        // 4. Descend until leaf is reached
        //      Set N to be child node pointed to by Fp and repeat from step 2.

        todo!("Descend into child nodes")
    }

    fn adjust_tree(&mut self) {
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
        //      If N has a partner NN resulting from an earlier split,
        //      create a new entry ENN with ENN pointing to NN and ENN enclosing all
        //      rectangles in NN. Add ENN to P if there is room, otherwise invoke `split_node`
        //      to produce P and PP containing ENN and all P's old entries.
        // 5. Move up to the next level
        //      Set N=P and set NN=PP if a split occurred. Repeat from step 2.
        todo!()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }
}

impl<T, const N: usize, const M: usize, TupleIdentifier> Default for RTree<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    fn default() -> Self {
        Self {
            root: RTreeNode::default(),
            split_strategy: LinearCostSplitting::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rtree::nodes::node_traits::Node;

    #[test]
    fn default_works() {
        let r: RTree<f32, 2, 10> = RTree::default();
        assert!(r.is_empty());
    }

    #[test]
    fn simple_insert_works() {
        let mut tree = RTree::<f32, 2, 2>::default();
        tree.insert(0, BoundingBox::from([1.0..=2.0, 4.0..=17.0]));
        //let root = tree.leaf_nodes[tree.root_id.get()].as_ref().unwrap();
        //assert!(!root.is_empty());
        //assert_eq!(root.len(), 1);
        //assert_eq!(root.to_bb(), [1.0..=2.0, 4.0..=17.0].into());
        todo!();
    }

    #[test]
    fn insert_works() {
        let mut tree = RTree::<f32, 2, 3>::default();
        tree.insert(0, [16.0..=68.0, 23.0..=35.0].into());
        tree.insert(1, [55.0..=68.0, 12.0..=148.0].into());
        tree.insert(2, [82.0..=94.0, 12.0..=148.0].into());
        tree.insert(3, [82.0..=145.0, 30.0..=42.0].into());

        //let root = tree.leaf_nodes[tree.root_id.get()].as_ref().unwrap();
        //assert!(!root.is_empty());
        //assert_eq!(root.len(), 2); // two "top-level" leaf nodes
        //assert_eq!(root.to_bb(), [16.0..=145.0, 12.0..=148.0].into());
        todo!()
    }
}
