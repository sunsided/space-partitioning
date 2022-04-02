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

pub(crate) trait Node<T, const N: usize>: AsBoundingBox<T, N>
where
    T: DimensionType,
{
    /// Returns the number of elements in this leaf node.
    fn len(&self) -> usize;

    /// Returns whether this node has any entries.
    fn is_empty(&self) -> bool;

    /// Tests whether this node's box fully contains another one.
    fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool;
}
