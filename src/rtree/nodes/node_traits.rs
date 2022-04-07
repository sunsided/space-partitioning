use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use std::borrow::Borrow;

pub(crate) trait AsBoundingBox<T, const N: usize>
where
    T: DimensionType,
{
    fn as_bb(&self) -> &BoundingBox<T, N>;
}

pub(crate) trait ToBoundingBox<T, const N: usize>
where
    T: DimensionType,
{
    fn to_bb(&self) -> BoundingBox<T, N>;
}

pub(crate) trait Node<T, const N: usize, const M: usize>: AsBoundingBox<T, N>
where
    T: DimensionType,
{
    const MAX_FILL: usize = M;
    const MIN_FILL: usize = (M + 1) / 2;

    /// Determines if this is full (including overfull).
    #[inline]
    fn is_full(&self) -> bool {
        self.len() >= Self::MAX_FILL
    }

    /// Determines if this node has more items than it is allowed to store.
    /// This will not return `true` if the node has exactly the maximum number of
    /// elements.
    #[inline]
    fn is_overfull(&self) -> bool {
        self.len() > Self::MAX_FILL
    }

    /// Determines if this node has fewer items than it is required to store.
    /// This will not return `true` if the node has exactly the minimum number of
    /// elements.
    #[inline]
    fn is_underfull(&self) -> bool {
        self.len() < Self::MIN_FILL
    }

    /// Returns the number of elements in this leaf node.
    fn len(&self) -> usize;

    /// Returns whether this node has any entries.
    fn is_empty(&self) -> bool;

    /// Tests whether this node's box fully contains another one.
    fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool;
}
