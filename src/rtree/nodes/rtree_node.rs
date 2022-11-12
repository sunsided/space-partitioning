use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::node_traits::HasBoundingBox;
use crate::rtree::nodes::prelude::{Node, RTreeLeaf};
use arrayvec::ArrayVec;
use std::borrow::Borrow;

/// A node type.
///
/// A node can either be a non-leaf, containing other child nodes, or
/// a leaf, containing data tuples.
///
/// ## Type parameters
/// * `T` - The coordinate type.
/// * `N` - The number of dimensions per coordinate.
/// * `M` - The maximum number of elements to store per leaf node.
/// * `TupleIdentifier` - The type used to identify a tuple in application code.
#[derive(Debug)]
pub(crate) struct RTreeNode<T, const N: usize, const M: usize, TupleIdentifier>
where
    T: DimensionType,
{
    /// The node data.
    pub node_data: NodeData<T, N, M, TupleIdentifier>,
}

/// ## Type parameters
/// * `T` - The coordinate type.
/// * `N` - The number of dimensions per coordinate.
/// * `M` - The maximum number of elements to store per leaf node.
/// * `TupleIdentifier` - The type used to identify a tuple in application code.
#[derive(Debug)]
pub(crate) enum NodeData<T, const N: usize, const M: usize, TupleIdentifier>
where
    T: DimensionType,
{
    /// A non-leaf node contains other nodes, either non-leaf or leaf.
    NonLeaf(ArrayVec<ChildPointer<T, N, RTreeNode<T, N, M, TupleIdentifier>>, M>),
    /// A leaf node is terminal and contains data.
    Leaf(ArrayVec<ChildPointer<T, N, RTreeLeaf<T, N, M, TupleIdentifier>>, M>),
}

/// A pointer to a child node.
///
/// ## Type parameters
/// * `T` - The coordinate type.
/// * `N` - The number of dimensions per coordinate.
#[derive(Debug)]
pub(crate) struct ChildPointer<T, const N: usize, TNode>
where
    T: DimensionType,
{
    /// The minimum bounding box of the child node.
    pub bb: BoundingBox<T, N>,
    /// The pointer to the child node.
    pub pointer: Box<TNode>,
}

impl<T, const N: usize, TupleIdentifier> ChildPointer<T, N, TupleIdentifier>
where
    T: DimensionType,
{
    /// Tests whether this node's box fully contains another one.
    #[inline]
    pub fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool {
        self.bb.contains(other)
    }
}

impl<T, const N: usize, const M: usize, TupleIdentifier> Default
    for RTreeNode<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    fn default() -> Self {
        Self {
            node_data: NodeData::Leaf(ArrayVec::new()),
        }
    }
}

impl<T, const N: usize, const M: usize, TupleIdentifier> HasBoundingBox<T, N>
    for RTreeNode<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    #[inline]
    fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool {
        self.node_data.contains(other)
    }

    #[inline]
    fn to_bb(&self) -> BoundingBox<T, N> {
        self.node_data.to_bb()
    }
}

impl<T, const N: usize, const M: usize, TupleIdentifier> Node<T, N, M>
    for RTreeNode<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    #[inline]
    fn is_leaf(&self) -> bool {
        self.node_data.is_leaf()
    }

    #[inline]
    fn len(&self) -> usize {
        self.node_data.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.node_data.is_empty()
    }
}

impl<T, const N: usize, const M: usize, TupleIdentifier> NodeData<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Leaf(leaf) => leaf.len(),
            Self::NonLeaf(non_leaf) => non_leaf.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Leaf(leaf) => leaf.is_empty(),
            Self::NonLeaf(non_leaf) => non_leaf.is_empty(),
        }
    }

    pub fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool {
        match self {
            Self::Leaf(leaf) => leaf.iter().any(|cp| cp.contains(other.borrow())),
            Self::NonLeaf(non_leaf) => non_leaf.iter().any(|cp| cp.contains(other.borrow())),
        }
    }

    pub fn to_bb(&self) -> BoundingBox<T, N> {
        match self {
            Self::Leaf(leaf) => leaf.iter().fold(BoundingBox::default(), |bb, cp| {
                bb.into_grown(cp.bb.borrow())
            }),
            Self::NonLeaf(non_leaf) => non_leaf.iter().fold(BoundingBox::default(), |bb, cp| {
                bb.into_grown(cp.bb.borrow())
            }),
        }
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        match self {
            Self::Leaf(_) => true,
            Self::NonLeaf(_) => false,
        }
    }
}
