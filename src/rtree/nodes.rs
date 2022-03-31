use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use std::cell::RefCell;
use std::rc::Rc;

/// Type alias for child pointers; introduces a `Box` around the `NodeChildEntry`.
#[derive(Debug)]
pub(crate) enum ChildNodes<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    Leaves(Vec<Rc<RefCell<LeafNode<T, N, M>>>>),
    NonLeaves(Vec<Rc<RefCell<NonLeafNode<T, N, M>>>>),
}

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

#[derive(Debug)]
pub(crate) struct NonLeafNode<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    /// The minimum bounding box of all child nodes.
    pub bb: BoundingBox<T, N>,
    pub children: ChildNodes<T, N, M>,
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

    /// Returns the number of elements in this leaf node.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether this node has any entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

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

    /// Tests whether this node's box fully contains another one.
    #[inline]
    pub fn contains(&self, other: &BoundingBox<T, N>) -> bool {
        self.bb.contains(other)
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
    pub fn insert(&mut self, id: usize, bb: BoundingBox<T, N>) {
        debug_assert!(!self.is_overfull());
        self.bb.grow(bb.clone());
        self.entries.push(Entry::new(id, bb));
    }
}

impl<T, const N: usize, const M: usize> NonLeafNode<T, N, M>
where
    T: DimensionType,
{
    /// Returns the number of child nodes of this non-leaf node.
    #[inline]
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns whether this node has any child nodes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Tests whether this node's box fully contains another one.
    #[inline]
    pub fn contains(&self, other: &BoundingBox<T, N>) -> bool {
        self.bb.contains(other)
    }
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

pub(crate) trait GetBoundingBox<T, const N: usize>
where
    T: DimensionType,
{
    fn bb(&self) -> &BoundingBox<T, N>;
}

impl<T, const N: usize, const M: usize> GetBoundingBox<T, N> for LeafNode<T, N, M>
where
    T: DimensionType,
{
    fn bb(&self) -> &BoundingBox<T, N> {
        &self.bb
    }
}

impl<T, const N: usize, const M: usize> GetBoundingBox<T, N> for NonLeafNode<T, N, M>
where
    T: DimensionType,
{
    fn bb(&self) -> &BoundingBox<T, N> {
        &self.bb
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
