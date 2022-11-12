use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::node_traits::{HasBoundingBox, Node};
use arrayvec::ArrayVec;
use std::borrow::Borrow;

/// A leaf node; this node contains the minimum bounding box of all
/// referenced objects, as well as a vector of entries.
///
/// ## Type parameters
/// * `T` - The coordinate type.
/// * `N` - The number of dimensions per coordinate.
/// * `M` - The maximum number of elements to store per leaf node.
/// * `TupleIdentifier` - The type used to identify a tuple in application code.
#[derive(Debug)]
pub(crate) struct RTreeLeaf<T, const N: usize, const M: usize, TupleIdentifier>
where
    T: DimensionType,
{
    /// The entries of the object records.
    pub entries: ArrayVec<IndexRecordEntry<T, N, TupleIdentifier>, M>,
}

/// An index record entry that is stored in a leaf node of the tree.
#[derive(Debug, Default)]
pub(crate) struct IndexRecordEntry<T, const N: usize, TupleIdentifier>
where
    T: DimensionType,
{
    /// The minimum bounding box of this entry.
    pub bb: BoundingBox<T, N>,
    /// The identifier of the object.
    pub id: TupleIdentifier,
}

impl<T, const N: usize, TupleIdentifier> Borrow<BoundingBox<T, N>>
    for IndexRecordEntry<T, N, TupleIdentifier>
where
    T: DimensionType,
{
    fn borrow(&self) -> &BoundingBox<T, N> {
        &self.bb
    }
}

impl<T, const N: usize, TupleIdentifier> HasBoundingBox<T, N>
    for IndexRecordEntry<T, N, TupleIdentifier>
where
    T: DimensionType,
{
    fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool {
        self.bb.contains(other)
    }

    fn to_bb(&self) -> BoundingBox<T, N> {
        self.bb.clone()
    }
}

impl<T, const N: usize, TupleIdentifier> IndexRecordEntry<T, N, TupleIdentifier>
where
    T: DimensionType,
{
    #[inline]
    pub fn new<B: Into<BoundingBox<T, N>>>(id: TupleIdentifier, bb: B) -> Self {
        Self { id, bb: bb.into() }
    }
}

impl<T, const N: usize, const M: usize, TupleIdentifier> Default
    for RTreeLeaf<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    fn default() -> Self {
        Self {
            entries: ArrayVec::default(),
        }
    }
}

impl<T, const N: usize, const M: usize, TupleIdentifier> RTreeLeaf<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    const NONE: Option<IndexRecordEntry<T, N, TupleIdentifier>> = None;

    /// Inserts a new entry into this node, growing the bounding box.
    ///
    /// ## Arguments
    /// * `id` - The ID of the element to insert.
    /// * `bb` - The minimum bounding box of the element to insert.
    ///
    /// ## Returns
    /// A `bool` indicating whether the insert was valid (`true`) or whether
    /// the box is now overfull (`false`).
    pub fn insert(&mut self, id: TupleIdentifier, bb: BoundingBox<T, N>) -> bool {
        self.insert_entry(IndexRecordEntry::new(id, bb))
    }

    /// Inserts a new entry into this node, growing the bounding box.
    ///
    /// ## Arguments
    /// * `entry` - The entry to insert.
    ///
    /// ## Returns
    /// A `bool` indicating whether the insert was valid (`true`) or whether
    /// the box is now overfull (`false`).
    pub fn insert_entry(&mut self, entry: IndexRecordEntry<T, N, TupleIdentifier>) -> bool {
        if self.len() == M {
            return false;
        }
        self.entries.push(entry);
        return true;
    }
}

impl<T, const N: usize, const M: usize, TupleIdentifier> HasBoundingBox<T, N>
    for RTreeLeaf<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    /// Tests whether this node's box fully contains another one.
    #[inline]
    fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool {
        let other = other.borrow();
        self.entries.iter().any(|x| x.bb.contains(other))
    }

    /// Builds a bounding box that minimally spans all elements.
    fn to_bb(&self) -> BoundingBox<T, N> {
        self.entries
            .iter()
            .fold(BoundingBox::default(), |mbb, x| mbb.into_grown(&x.bb))
    }
}

impl<T, const N: usize, const M: usize, TupleIdentifier> Node<T, N, M>
    for RTreeLeaf<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    /// Determines if this node is a leaf. For this node type, always returns `true`.
    #[inline]
    fn is_leaf(&self) -> bool {
        true
    }

    /// Returns the number of child nodes of this non-leaf node.
    #[inline]
    fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether this node has any child nodes.
    #[inline]
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
