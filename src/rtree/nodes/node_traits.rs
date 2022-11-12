use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use std::borrow::Borrow;

/// Common functionalities of node types.
///
/// ## Type parameters
/// * `T` - The coordinate type.
/// * `N` - The number of dimensions per coordinate.
/// * `M` - The maximum number of elements to store per leaf node.
/// * `TupleIdentifier` - The type used to identify a tuple in application code
pub(crate) trait Node<T, const N: usize, const M: usize>: HasBoundingBox<T, N>
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

    /// Determines if this node has fewer items than it is required to store.
    /// This will not return `true` if the node has exactly the minimum number of
    /// elements.
    #[inline]
    fn is_underfull(&self) -> bool {
        self.len() < Self::MIN_FILL
    }

    /// Determines if this node is a leaf.
    #[inline]
    fn is_leaf(&self) -> bool;

    /// Returns the number of elements in this leaf node.
    fn len(&self) -> usize;

    /// Returns whether this node has any entries.
    fn is_empty(&self) -> bool;
}

/// Common functionalities of node types.
///
/// ## Type parameters
/// * `T` - The coordinate type.
/// * `N` - The number of dimensions per coordinate.
pub(crate) trait HasBoundingBox<T, const N: usize>
where
    T: DimensionType,
{
    /// Tests whether this node's box fully contains another one.
    fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool;

    /// Builds a bounding box that minimally spans all child nodes or elements.
    fn to_bb(&self) -> BoundingBox<T, N>;
}

impl<Target, T, const N: usize> HasBoundingBox<T, N> for [Target]
where
    //Target: HasBoundingBox<T, N>,
    Target: Borrow<BoundingBox<T, N>>,
    T: DimensionType,
{
    fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool {
        self.iter().any(|bb| bb.borrow().contains(other.borrow()))
    }

    fn to_bb(&self) -> BoundingBox<T, N> {
        self.iter().fold(BoundingBox::default(), |bb, other| {
            bb.into_grown(other.borrow())
        })
    }
}
