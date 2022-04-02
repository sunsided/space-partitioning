use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::node_traits::{GetBoundingBox, Node};
use crate::rtree::nodes::ChildNodes;
use std::borrow::Borrow;

#[derive(Debug)]
pub(crate) struct NonLeafNode<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    /// The minimum bounding box of all child nodes.
    pub bb: BoundingBox<T, N>,
    pub children: ChildNodes<T, N, M>,
}

impl<T, const N: usize, const M: usize> Node<T, N> for NonLeafNode<T, N, M>
where
    T: DimensionType,
{
    /// Returns the number of child nodes of this non-leaf node.
    #[inline]
    fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns whether this node has any child nodes.
    #[inline]
    fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Tests whether this node's box fully contains another one.
    #[inline]
    fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool {
        self.bb.contains(other)
    }
}

impl<T, const N: usize, const M: usize> GetBoundingBox<T, N> for NonLeafNode<T, N, M>
where
    T: DimensionType,
{
    fn bb_ref(&self) -> &BoundingBox<T, N> {
        &self.bb
    }
}
