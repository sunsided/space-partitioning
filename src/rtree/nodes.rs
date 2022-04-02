use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::leaf_node::LeafNode;

use crate::rtree::nodes::non_leaf_node::NonLeafNode;

use std::cell::RefCell;
use std::rc::Rc;

pub(crate) mod leaf_node;
pub(crate) mod node_traits;
pub(crate) mod non_leaf_node;

pub(crate) mod prelude {
    pub(crate) use super::ChildNodes;
    pub(crate) use crate::rtree::nodes::leaf_node::LeafNode;
    pub(crate) use crate::rtree::nodes::node_traits::{AsBoundingBox, Node};
    pub(crate) use crate::rtree::nodes::non_leaf_node::NonLeafNode;
}

/// Type alias for child pointers; introduces a `Box` around the `NodeChildEntry`.
#[derive(Debug)]
pub(crate) enum ChildNodes<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    Leaves(Vec<Rc<RefCell<LeafNode<T, N, M>>>>),
    NonLeaves(Vec<Rc<RefCell<NonLeafNode<T, N, M>>>>),
}

impl<T, const N: usize, const M: usize> ChildNodes<T, N, M>
where
    T: DimensionType,
{
    /// Returns the value as a vector of leaf nodes.
    ///
    /// ## Panics
    /// Panics if the value is not a vector of leaf nodes.
    ///
    /// ## Returns
    /// A mutable reference to the embedded vector, cast to
    /// leaf nodes.
    pub fn to_leaves_mut(&mut self) -> &mut Vec<Rc<RefCell<LeafNode<T, N, M>>>> {
        match self {
            ChildNodes::Leaves(children) => children,
            _ => panic!("children are not leaves"),
        }
    }

    /// Returns the value as a vector of non-leaf nodes.
    ///
    /// ## Panics
    /// Panics if the value is not a vector of non-leaf nodes.
    ///
    /// ## Returns
    /// A mutable reference to the embedded vector, cast to
    /// non-leaf nodes.
    pub fn to_non_leaves_mut(&mut self) -> &mut Vec<Rc<RefCell<NonLeafNode<T, N, M>>>> {
        match self {
            ChildNodes::NonLeaves(children) => children,
            _ => panic!("children are not non-leaves"),
        }
    }

    /// Returns the number of child nodes of this child entry.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            ChildNodes::Leaves(x) => x.len(),
            ChildNodes::NonLeaves(x) => x.len(),
        }
    }

    /// Returns whether this node has any child nodes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        match &self {
            ChildNodes::Leaves(x) => x.is_empty(),
            ChildNodes::NonLeaves(x) => x.is_empty(),
        }
    }
}
