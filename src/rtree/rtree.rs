use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::node_traits::HasBoundingBox;
use crate::rtree::nodes::prelude::*;
use crate::rtree::nodes::rtree_leaf::IndexRecordEntry;
use crate::rtree::nodes::rtree_node::{ChildPointer, NodeData};
use crate::rtree::splitting_strategies::linear_cost_split::LinearCostSplitting;
use crate::rtree::splitting_strategies::SplittingStrategy;
use arrayvec::ArrayVec;

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

struct LeafPath {
    node_indices: Vec<usize>,
    leaf_index: Option<usize>,
}

impl<T, const N: usize, const M: usize, TupleIdentifier> RTree<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    /// Inserts an element into the tree.
    pub fn insert(&mut self, id: TupleIdentifier, bb: BoundingBox<T, N>) {
        // Citing https://iq.opengenus.org/r-tree/
        //
        // 1. Find position for new record:
        //      Invoke `choose_leaf` to select leaf node L in which to place the entry.
        let entry = IndexRecordEntry::new(id, bb);
        let leaf_path = self.choose_leaf(&entry.bb);
        let split_strategy = self.split_strategy.clone();

        // 2. Add record to leaf node.
        //      If L has room for another entry then add E, else
        //      invoke `split_node` to obtain L and LL (current leaf and new leaf containing all old entries of L)
        let split_node = {
            let leaf_node = self.node_at_path_mut(&leaf_path.node_indices);
            match &mut leaf_node.node_data {
                NodeData::Leaf(leaves) => {
                    Self::insert_into_leaf_node(&split_strategy, leaves, leaf_path.leaf_index, entry)
                }
                NodeData::NonLeaf(_) => {
                    unreachable!("choose_leaf always descends to a leaf-level node")
                }
            }
        };

        // 3. Propagate changes upward
        //      Invoke `adjust_tree` on L also passing LL if split was performed.
        // 4. Grow the tree taller
        //      If node split propagation caused the root to split, create a new root
        //      whose children are the two resulting nodes.
        if let Some(new_node) = self.adjust_tree(&leaf_path.node_indices, split_node) {
            self.grow_root(new_node);
        }
    }

    /// Select a leaf node in which to place a new entry.
    fn choose_leaf(&self, bb: &BoundingBox<T, N>) -> LeafPath {
        // Citing https://iq.opengenus.org/r-tree/
        //
        // 1. Initialize
        //      Set N to be the root node
        let mut current = &self.root;
        let mut node_indices = Vec::new();

        // 2. Leaf check
        //      If N is a leaf, return N
        loop {
            match &current.node_data {
                NodeData::Leaf(children) => {
                    let leaf_index = if children.is_empty() {
                        None
                    } else {
                        Some(Self::choose_child_index(children.as_slice(), bb))
                    };
                    return LeafPath {
                        node_indices,
                        leaf_index,
                    };
                }
                NodeData::NonLeaf(children) => {
                    // 3. Choose subtree
                    //      Let F be the entry in N whose rectangle F1
                    //      needs least enlargement to include E1. Resolve ties by choosing
                    //      the entry with the rectangle of the smallest area.
                    // 4. Descend until leaf is reached
                    //      Set N to be child node pointed to by Fp and repeat from step 2.
                    let idx = Self::choose_child_index(children.as_slice(), bb);
                    node_indices.push(idx);
                    current = &children[idx].pointer;
                }
            }
        }
    }

    fn adjust_tree(
        &mut self,
        node_path: &[usize],
        mut split_node: Option<RTreeNode<T, N, M, TupleIdentifier>>,
    ) -> Option<RTreeNode<T, N, M, TupleIdentifier>> {
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
        let split_strategy = self.split_strategy.clone();
        let mut depth = node_path.len();

        while depth > 0 {
            let parent_path = &node_path[..depth - 1];
            let child_index = node_path[depth - 1];
            let parent = self.node_at_path_mut(parent_path);

            match &mut parent.node_data {
                NodeData::NonLeaf(children) => {
                    {
                        let child = &mut children[child_index];
                        child.bb = child.pointer.to_bb();
                    }

                    if let Some(new_node) = split_node {
                        let new_child = ChildPointer {
                            bb: new_node.to_bb(),
                            pointer: Box::new(new_node),
                        };

                        if children.len() < M {
                            children.push(new_child);
                            split_node = None;
                        } else {
                            let parent_bb = children.as_slice().to_bb();
                            let split_result = split_strategy.split(&parent_bb, children, new_child);
                            let second_entries = split_result.second.entries;
                            *children = split_result.first.entries;
                            split_node = Some(RTreeNode {
                                node_data: NodeData::NonLeaf(second_entries),
                            });
                        }
                    }
                }
                NodeData::Leaf(_) => unreachable!("parent nodes must be non-leaf"),
            }

            depth -= 1;
        }

        split_node
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    fn node_at_path_mut(
        &mut self,
        node_path: &[usize],
    ) -> &mut RTreeNode<T, N, M, TupleIdentifier> {
        let mut current = &mut self.root;
        for &idx in node_path {
            let child = match &mut current.node_data {
                NodeData::NonLeaf(children) => &mut children[idx],
                NodeData::Leaf(_) => panic!("expected non-leaf while descending"),
            };
            current = &mut child.pointer;
        }
        current
    }

    fn choose_child_index<TNode>(
        children: &[ChildPointer<T, N, TNode>],
        bb: &BoundingBox<T, N>,
    ) -> usize {
        debug_assert!(!children.is_empty());
        let mut best_idx = 0;
        let mut best = children[0].bb.get_grown(bb);

        for (idx, child) in children.iter().enumerate().skip(1) {
            let grown = child.bb.get_grown(bb);
            if grown.area_increase < best.area_increase
                || (grown.area_increase == best.area_increase && grown.area < best.area)
            {
                best = grown;
                best_idx = idx;
            }
        }

        best_idx
    }

    fn insert_into_leaf_node(
        split_strategy: &LinearCostSplitting,
        leaves: &mut ArrayVec<
            ChildPointer<T, N, RTreeLeaf<T, N, M, TupleIdentifier>>,
            M,
        >,
        leaf_index: Option<usize>,
        entry: IndexRecordEntry<T, N, TupleIdentifier>,
    ) -> Option<RTreeNode<T, N, M, TupleIdentifier>> {
        let idx = match leaf_index {
            Some(idx) => idx,
            None => {
                let mut leaf = RTreeLeaf::default();
                leaf.insert_entry(entry);
                leaves.push(ChildPointer {
                    bb: leaf.to_bb(),
                    pointer: Box::new(leaf),
                });
                return None;
            }
        };

        let leaf_pointer = &mut leaves[idx];
        if !leaf_pointer.pointer.is_full() {
            leaf_pointer.pointer.insert_entry(entry);
            leaf_pointer.bb = leaf_pointer.pointer.to_bb();
            return None;
        }

        let leaf_bb = leaf_pointer.pointer.to_bb();
        let split = split_strategy.split(&leaf_bb, &mut leaf_pointer.pointer.entries, entry);
        leaf_pointer.pointer.entries = split.first.entries;
        leaf_pointer.bb = split.first.bb;

        let new_leaf = RTreeLeaf {
            entries: split.second.entries,
        };
        let new_child = ChildPointer {
            bb: split.second.bb,
            pointer: Box::new(new_leaf),
        };

        if leaves.len() < M {
            leaves.push(new_child);
            return None;
        }

        let parent_bb = leaves.as_slice().to_bb();
        let split = split_strategy.split(&parent_bb, leaves, new_child);
        let second_entries = split.second.entries;
        *leaves = split.first.entries;
        Some(RTreeNode {
            node_data: NodeData::Leaf(second_entries),
        })
    }

    fn grow_root(&mut self, split_node: RTreeNode<T, N, M, TupleIdentifier>) {
        let old_root = std::mem::replace(&mut self.root, RTreeNode::default());
        let mut children: ArrayVec<_, M> = ArrayVec::new();
        children.push(ChildPointer {
            bb: old_root.to_bb(),
            pointer: Box::new(old_root),
        });
        children.push(ChildPointer {
            bb: split_node.to_bb(),
            pointer: Box::new(split_node),
        });
        self.root = RTreeNode {
            node_data: NodeData::NonLeaf(children),
        };
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

    #[test]
    fn default_works() {
        let r: RTree<f32, 2, 10> = RTree::default();
        assert!(r.is_empty());
    }

    #[test]
    fn simple_insert_works() {
        let mut tree = RTree::<f32, 2, 2>::default();
        tree.insert(0, BoundingBox::from([1.0..=2.0, 4.0..=17.0]));
        assert!(!tree.is_empty());

        match &tree.root.node_data {
            NodeData::Leaf(children) => {
                assert_eq!(children.len(), 1);
                let leaf = &children[0].pointer;
                assert_eq!(leaf.entries.len(), 1);
                assert_eq!(leaf.to_bb(), [1.0..=2.0, 4.0..=17.0].into());
            }
            NodeData::NonLeaf(_) => panic!("expected root to point to leaves"),
        }
    }

    #[test]
    fn insert_works() {
        let mut tree = RTree::<f32, 2, 3>::default();
        tree.insert(0, [16.0..=68.0, 23.0..=35.0].into());
        tree.insert(1, [55.0..=68.0, 12.0..=148.0].into());
        tree.insert(2, [82.0..=94.0, 12.0..=148.0].into());
        tree.insert(3, [82.0..=145.0, 30.0..=42.0].into());

        assert!(!tree.is_empty());

        match &tree.root.node_data {
            NodeData::Leaf(children) => {
                assert_eq!(children.len(), 2);
                assert_eq!(
                    children.as_slice().to_bb(),
                    [16.0..=145.0, 12.0..=148.0].into()
                );
            }
            NodeData::NonLeaf(_) => panic!("expected root to point to leaves"),
        }
    }
}
