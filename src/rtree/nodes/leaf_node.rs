use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::node_traits::{AsBoundingBox, Node, ToBoundingBox};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

/// An entry in a child node.
/// This type stores the minimum bounding box of the object, as well as its ID.
#[derive(Debug, Clone)]
pub(crate) struct Entry<T, const N: usize>
where
    T: DimensionType,
{
    /// The minimum bounding box of the object.
    pub bb: BoundingBox<T, N>,
    /// The ID of the object.
    pub id: usize,
}

/// A leaf node; this node contains the minimum bounding box of all
/// referenced objects, as well as a vector of entries.
#[derive(Debug)]
pub(crate) struct LeafNode<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    /// The minimum bounding box of the object.
    pub bb: BoundingBox<T, N>,
    /// The entries of the objects.
    pub entries: Vec<Entry<T, N>>,
}

impl<T, const N: usize> Entry<T, N>
where
    T: DimensionType,
{
    #[inline]
    pub fn new(id: usize, bb: BoundingBox<T, N>) -> Self {
        Self { id, bb }
    }
}

impl<T, const N: usize, const M: usize> LeafNode<T, N, M>
where
    T: DimensionType,
{
    pub const MAX_FILL: usize = M;
    pub const MIN_FILL: usize = (M + 1) / 2;

    /// Determines if this is full (including overfull).
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() >= Self::MAX_FILL
    }

    /// Determines if this node has more items than it is allowed to store.
    /// This will not return `true` if the node has exactly the maximum number of
    /// elements.
    #[inline]
    pub fn is_overfull(&self) -> bool {
        self.len() > Self::MAX_FILL
    }

    /// Determines if this node has fewer elements than allowed.
    #[inline]
    pub fn is_underfull(&self) -> bool {
        self.len() < Self::MIN_FILL
    }

    /// Updates the bounding box of this node to tightly fit all elements.
    pub fn update_bounding_box(&mut self) {
        let mut new_box = BoundingBox::default();
        for entry in &self.entries {
            new_box.grow(&entry.bb);
        }
        self.bb = new_box;
    }

    /// Inserts a new entry into this node, growing the bounding box.
    ///
    /// ## Arguments
    /// * `id` - The ID of the element to insert.
    /// * `bb` - The minimum bounding box of the element to insert.
    pub fn insert(&mut self, id: usize, bb: BoundingBox<T, N>) {
        debug_assert!(!self.is_overfull());
        self.bb.grow(bb.clone());
        self.entries.push(Entry::new(id, bb));
    }

    /// Inserts a new entry into this node, growing the bounding box.
    ///
    /// ## Arguments
    /// * `id` - The ID of the element to insert.
    /// * `bb` - The minimum bounding box of the element to insert.
    ///
    /// ## Returns
    /// A `bool` indicating whether the insert was valid (`true`) or whether
    /// the box is now overfull (`false`).
    pub fn insert_unchecked(&mut self, id: usize, bb: BoundingBox<T, N>) -> bool {
        self.bb.grow(bb.clone());
        self.entries.push(Entry::new(id, bb));
        !self.is_overfull()
    }
}

impl<T, const N: usize, const M: usize> Node<T, N> for LeafNode<T, N, M>
where
    T: DimensionType,
{
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

    /// Tests whether this node's box fully contains another one.
    #[inline]
    fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool {
        self.bb.contains(other)
    }
}

impl<T, const N: usize, const M: usize> AsBoundingBox<T, N> for LeafNode<T, N, M>
where
    T: DimensionType,
{
    fn as_bb(&self) -> &BoundingBox<T, N> {
        &self.bb
    }
}

impl<T, const N: usize> AsBoundingBox<T, N> for Entry<T, N>
where
    T: DimensionType,
{
    fn as_bb(&self) -> &BoundingBox<T, N> {
        &self.bb
    }
}

impl<T, const N: usize> ToBoundingBox<T, N> for Entry<T, N>
where
    T: DimensionType,
{
    fn to_bb(&self) -> BoundingBox<T, N> {
        self.bb.clone()
    }
}

impl<T, const N: usize, const M: usize> ToBoundingBox<T, N> for Rc<RefCell<LeafNode<T, N, M>>>
where
    T: DimensionType,
{
    fn to_bb(&self) -> BoundingBox<T, N> {
        self.deref().borrow().bb.clone()
    }
}

impl<T, const N: usize, const M: usize> Default for LeafNode<T, N, M>
where
    T: DimensionType,
{
    fn default() -> Self {
        Self {
            bb: BoundingBox::default(),
            entries: vec![],
        }
    }
}
