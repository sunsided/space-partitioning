use crate::quadtree::free_list;
use std::fmt::{Debug, Formatter};

pub type NodeElementCountType = u32;

/// This value encodes that a node is a branch, i.e., not a leaf.
const NODE_IS_BRANCH: u32 = NodeElementCountType::MAX;

/// Represents a node in the quadtree.
#[derive(Copy, Clone)]
pub struct Node {
    /// Contains
    /// - the index of the first child if this node is a branch or
    /// - the index of the first element if this node is a leaf or
    /// - `free_list::SENTINEL` if neither of which exists.
    ///
    /// If this node is the first child node and pointed to by
    /// the free node pointer, all subsequent nodes of the same parent (i.e., the next three nodes).
    pub first_child_or_element: free_list::IndexType,
    /// Stores the number of elements in the leaf or `NODE_INDEX_IS_BRANCH if it this node is
    /// a branch (i.e., not a leaf).
    pub element_count: NodeElementCountType,
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.element_count, self.first_child_or_element) {
            (NODE_IS_BRANCH, free_list::SENTINEL) => {
                write!(f, "Branch, no child nodes")
            }
            (NODE_IS_BRANCH, child_index) => write!(
                f,
                "Branch, child nodes at {}..={}",
                child_index,
                child_index + 3
            ),
            (0, free_list::SENTINEL) => write!(f, "Leaf, no elements"),
            (count, free_list::SENTINEL) => {
                panic!("Got {} child elements but no data pointer", count)
            }
            (count, element_index) => write!(
                f,
                "Leaf, elements {}..={}",
                element_index,
                element_index + count
            ),
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            first_child_or_element: free_list::SENTINEL,
            element_count: 0,
        }
    }
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
        self.first_child_or_element
    }

    /// If the node is leaf, gets the index of the element node.
    #[inline]
    pub fn get_first_element_node_index(&self) -> free_list::IndexType {
        debug_assert!(self.is_leaf());
        debug_assert!(self.element_count > 0 || self.first_child_or_element == free_list::SENTINEL);
        self.first_child_or_element
    }

    /// Make this node an empty leaf.
    pub fn make_empty_leaf(&mut self) {
        self.first_child_or_element = free_list::SENTINEL;
        self.element_count = 0;
    }

    /// Make this node a branch.
    pub fn make_branch(&mut self, first_child: free_list::IndexType) {
        self.first_child_or_element = first_child;
        self.element_count = NODE_IS_BRANCH;
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
