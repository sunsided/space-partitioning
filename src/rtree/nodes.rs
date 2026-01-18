//! Internal node types and traits used by the R-Tree implementation.

pub(crate) mod node_traits;
pub(crate) mod rtree_leaf;
pub(crate) mod rtree_node;

pub(crate) mod prelude {
    //! Internal prelude for node-related types.
    pub(crate) use crate::rtree::nodes::node_traits::Node;
    pub(crate) use crate::rtree::nodes::rtree_leaf::RTreeLeaf;
    pub(crate) use crate::rtree::nodes::rtree_node::RTreeNode;
}
