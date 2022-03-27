use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::io::Read;
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
    root: ChildPointer<T, N, M>,
}

/// A pointer to a child entry; can either be a leaf or non-leaf.
#[derive(Debug)]
enum NodeChildEntry<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    /// The child node is a leaf and contains references to objects.
    Leaf(LeafNode<T, N, M>),
    /// The child node is a non-leaf and contains references to other child nodes.
    NonLeaf(NonLeafNode<T, N, M>),
}

/// Type alias for child pointers; introduces a `Box` around the `NodeChildEntry`.
type ChildPointer<T, const N: usize, const M: usize> = Box<NodeChildEntry<T, N, M>>;

/// An entry in a child node.
/// This type stores the minimum bounding box of the object, as well as its ID.
#[derive(Debug)]
struct Entry<T, const N: usize>
where
    T: DimensionType,
{
    /// The minimum bounding box of the object.
    bb: BoundingBox<T, N>,
    /// The ID of the object.
    id: usize,
}

/// A leaf node; this node contains the minimum bounding box of all
/// referenced objects, as well as a vector of entries.
#[derive(Debug)]
struct LeafNode<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    /// The minimum bounding box of the object.
    bb: BoundingBox<T, N>,
    /// The entries of the objects.
    entries: Vec<Entry<T, N>>,
}

#[derive(Debug)]
struct NonLeafNode<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    /// The minimum bounding box of all child nodes.
    bb: BoundingBox<T, N>,
    children: Vec<ChildPointer<T, N, M>>,
}

impl<T, const N: usize, const M: usize> Default for RTree<T, N, M>
where
    T: DimensionType,
{
    fn default() -> Self {
        let root = NodeChildEntry::Leaf(LeafNode {
            entries: vec![],
            bb: BoundingBox::default(),
        });
        Self {
            root: Box::new(root),
        }
    }
}

impl<T, const N: usize, const M: usize> RTree<T, N, M>
where
    T: DimensionType,
{
    /// Inserts an element into the tree.
    pub fn insert(&mut self, id: usize, bb: BoundingBox<T, N>) {
        let child = self.choose_leaf(&bb);

        /*
        let mut stack = vec![&mut self.root];
        'recurse: while let Some(self_node) = stack.pop() {
            // TODO: Should probably keep the root node in here until all adjustments were made.

            let self_node: &NodeChildEntry<T, N, M> = &self_node.into_inner();
            match self_node {
                NodeChildEntry::NonLeaf(node) => {
                    // Descend into the node that fully contains the object's region.
                    'next_child: for c in 0..node.children.len() {
                        // for mut child in node.children.iter_mut() {
                        let child = &node.children[c];
                        // TODO: Pick the node of the smallest area that fully contains the object?
                        if !child.into_inner().contains(&bb) {
                            continue 'next_child;
                        }

                        stack.push(&mut node.children[c]);
                        continue 'recurse;
                    }

                    // At this point, none of the child nodes contained the object's region.
                    // We now need to find pick a node to add the entry to by selecting the
                    // bounding box that grows the least in order to support the addition.
                    debug_assert!(node.children.len() > 0);

                    let child_idx = Self::find_child_of_smallest_size_increase(node, &bb);
                    let mut child = node.children[child_idx].into_inner();
                    child.grow(&bb);

                    // Recurse down into the selected child node.
                    stack.push(&mut node.children[child_idx]);
                    continue 'recurse;
                }
                NodeChildEntry::Leaf(node) => {
                    if node.contains(&bb) {
                        // We simply add the item here; no need to adjust the
                        // bounding box of the node since it already fully contains the item.
                        node.entries.push(Entry { id, bb: bb.clone() });
                    } else {
                        // When the tree is empty, the root node is a leaf with zero elements.
                        if node.is_empty() {
                            node.bb.grow(&bb);
                            node.entries.push(Entry { id, bb: bb.clone() });
                        } else {
                            // Split node (this could change the current node into a non-leaf!)
                            todo!()
                        }

                        // Update parent's BB
                        // propagate adjustment / split upwards
                        todo!()
                    }

                    debug_assert!(!node.is_overfull()); // TODO ...
                }
            }
        }
         */

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

    /// Find the child node that requires the smallest increase in area to contain
    /// the new bounding box.
    ///
    /// ## Returns
    /// The index of the child node that requires the smallest size increase.
    fn find_child_of_smallest_size_increase(
        node: &NonLeafNode<T, N, M>,
        bb: &BoundingBox<T, N>,
    ) -> usize {
        let mut smallest_c = 0;
        let mut smallest_bb = node.children[0].bb().get_grown(bb);

        for c in 1..node.children.len() {
            let adjusted_bb = node.children[c].bb().get_grown(bb);
            if adjusted_bb.area < smallest_bb.area {
                smallest_c = c;
                smallest_bb = adjusted_bb;
            }
        }
        smallest_c
    }

    /// Select a leaf node in which to place a new entry.
    fn choose_leaf(&mut self, bb: &BoundingBox<T, N>) -> &LeafNode<T, N, M> {
        // Citing https://iq.opengenus.org/r-tree/
        //
        // 1. Initialize: Set N to be the root node
        // 2. Leaf Check: If N is a leaf, return N
        // 3. Choose subtree
        //      If N is not a leaf, let F be the entry in N whose rectangle F1 needs
        //      least enlargement to include E1. Resolve ties by choosing the entry with
        //      the rectangle of the smallest area.
        // 4. Descend until a leaf is reached.
        //      Set N to be the child node pointed to by Fp and repeat from step 2.

        let mut parents = vec![];
        let mut current_node_ptr = &mut self.root;
        'recurse: loop {
            let current_node: &mut NodeChildEntry<T, N, M> = &mut current_node_ptr;
            match &mut *current_node {
                NodeChildEntry::NonLeaf(node) => {
                    // Descend into the node that fully contains the object's region.
                    'next_child: for c in 0..node.children.len() {
                        // TODO: If multiple child nodes contain the object fully, pick the one of the smallest area
                        let child = node.children[c];
                        if !child.contains(bb) {
                            continue 'next_child;
                        }

                        current_node_ptr = &mut node.children[c];
                        parents.push(current_node_ptr);
                        continue 'recurse;
                    }

                    // At this point, none of the child nodes contained the object's region.
                    // We now need to find pick a node to add the entry to by selecting the
                    // bounding box that grows the least in order to support the addition.
                    debug_assert!(node.children.len() > 0);
                    let child_idx = Self::find_child_of_smallest_size_increase(node, &bb);

                    // Enlarge the node such that it fully contains the object to be inserted.
                    node.children[child_idx].grow(bb);

                    // Recurse down into the selected child node.
                    current_node_ptr = &mut node.children[child_idx];
                    parents.push(current_node_ptr);
                    continue 'recurse;
                }
                NodeChildEntry::Leaf(node) => {
                    return node;
                }
            }
        }

        unreachable!()
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

    fn split_node(&self, _leaf: &mut LeafNode<T, N, M>) -> &mut LeafNode<T, N, M> {
        // - Exhaustive
        // - Quadratic-Cost
        // - Linear-Cost
        todo!()
    }
}

impl<T, const N: usize, const M: usize> LeafNode<T, N, M>
where
    T: DimensionType,
{
    const MAX_FILL: usize = M;
    const MIN_FILL: usize = (M + 1) / 2;

    /// Returns the number of elements in this leaf node.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether this node has any entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Determines if this is full (including overfull).
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() >= Self::MAX_FILL
    }

    /// Determines if this node has more items than it is allowed to store.
    /// This will not return `true` if the node has exactly the maximum number of
    /// elements.
    #[inline]
    pub fn is_overfull(&self) -> bool {
        self.len() > Self::MAX_FILL
    }

    /// Determines if this node has fewer elements than allowed.
    #[inline]
    pub fn is_underfull(&self) -> bool {
        self.len() < Self::MIN_FILL
    }

    /// Tests whether this node's box fully contains another one.
    #[inline]
    pub fn contains(&self, other: &BoundingBox<T, N>) -> bool {
        self.bb.contains(other)
    }

    /// Updates the bounding box of this node to tightly fit all elements.
    pub fn update_bounding_box(&mut self) {
        let mut new_box = BoundingBox::default();
        for entry in &self.entries {
            new_box.grow(&entry.bb);
        }
        self.bb = new_box;
    }
}

impl<T, const N: usize, const M: usize> NonLeafNode<T, N, M>
where
    T: DimensionType,
{
    /// Returns the number of child nodes of this non-leaf node.
    #[inline]
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns whether this node has any child nodes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Tests whether this node's box fully contains another one.
    #[inline]
    pub fn contains(&self, other: &BoundingBox<T, N>) -> bool {
        self.bb.contains(other)
    }

    /// Updates the bounding box of this node to tightly fit all child nodes.
    pub fn update_bounding_box(&mut self) {
        let mut new_box = BoundingBox::default();
        for entry in &self.children {
            new_box.grow(entry.bb());
        }
        self.bb = new_box;
    }
}

impl<T, const N: usize, const M: usize> NodeChildEntry<T, N, M>
where
    T: DimensionType,
{
    /// Returns the number of child nodes of this child entry..
    #[inline]
    fn len(&self) -> usize {
        match self {
            NodeChildEntry::Leaf(x) => x.len(),
            NodeChildEntry::NonLeaf(x) => x.len(),
        }
    }

    /// Returns whether this entry has any child nodes.
    #[inline]
    fn is_empty(&self) -> bool {
        match self {
            NodeChildEntry::Leaf(x) => x.is_empty(),
            NodeChildEntry::NonLeaf(x) => x.is_empty(),
        }
    }

    /// Tests whether this node's box fully contains another one.
    #[inline]
    pub fn contains(&self, other: &BoundingBox<T, N>) -> bool {
        match self {
            NodeChildEntry::Leaf(x) => x.contains(other),
            NodeChildEntry::NonLeaf(x) => x.contains(other),
        }
    }

    /// Updates the bounding box of this node to tightly fit all elements and/or child nodes.
    #[inline]
    pub fn update_bounding_box(&mut self) {
        match self {
            NodeChildEntry::Leaf(x) => x.update_bounding_box(),
            NodeChildEntry::NonLeaf(x) => x.update_bounding_box(),
        }
    }

    /// Returns a reference to the bounding box of the element.
    #[inline]
    fn bb(&self) -> &BoundingBox<T, N> {
        match self {
            NodeChildEntry::Leaf(x) => &x.bb,
            NodeChildEntry::NonLeaf(x) => &x.bb,
        }
    }

    /// Updates the bounding box of this node to tightly fit all elements and/or child nodes.
    #[inline]
    pub fn grow(&mut self, bb: &BoundingBox<T, N>) {
        match self {
            NodeChildEntry::Leaf(x) => x.bb.grow(bb),
            NodeChildEntry::NonLeaf(x) => x.bb.grow(bb),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_works() {
        let r: RTree<f32, 2, 10> = RTree::default();
        assert!(r.root.is_empty());
        assert_eq!(r.root.len(), 0);
    }

    #[test]
    fn insert_works() {
        let mut tree = RTree::<f32, 2, 2>::default();
        tree.insert(0, BoundingBox::from([1.0..=2.0, 4.0..=17.0]));
        assert!(!tree.root.is_empty());
        assert_eq!(tree.root.len(), 1);
        // TODO: The tree dimensions must now match the object's.
    }
}
