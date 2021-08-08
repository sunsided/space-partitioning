use crate::quadtree::free_list;

type NodeElementCountType = u32;

/// This value encodes that a node is a branch, i.e., not a leaf.
const NODE_IS_BRANCH: u32 = NodeElementCountType::MAX;

/// Represents a node in the quadtree.
pub struct Node {
    /// Points to
    /// - the first child if this node is a branch or
    /// - the first element if this node is a leaf.
    ///
    /// If this node is the first child node and pointed to by
    /// the free node pointer, all subsequent nodes of the same parent (i.e., the next three nodes).
    pub first_child: free_list::IndexType,
    /// Stores the number of elements in the leaf or `NODE_INDEX_IS_BRANCH if it this node is
    /// a branch (i.e., not a leaf).
    pub element_count: NodeElementCountType,
}

impl Node {
    /// Returns whether the node stores elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.element_count == 0
    }

    /// Determines whether this node is a branch.
    #[inline]
    pub fn is_branch(&self) -> bool {
        self.element_count == NODE_IS_BRANCH
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        !self.is_branch()
    }

    /// If the node is branch, gets the index of the first child node.
    #[inline]
    pub fn get_first_child_node_index(&self) -> free_list::IndexType {
        debug_assert!(self.is_branch());
        self.first_child
    }

    /// If the node is leaf, gets the index of the element node.
    #[inline]
    pub fn get_element_index(&self) -> free_list::IndexType {
        debug_assert!(self.is_leaf());
        self.first_child
    }

    /// Make this node an empty leaf.
    pub fn make_empty_leaf(&mut self) {
        self.first_child = free_list::SENTINEL;
        self.element_count = 0;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn quad_node_is_eight_bytes() {
        assert_eq!(std::mem::size_of::<Node>(), 8);
    }
}
