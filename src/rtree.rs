//! R-Tree implementation for axis-aligned bounding boxes.
//!
//! This module provides an in-memory, fixed-capacity R-Tree parameterized by:
//! - `T`: coordinate type (numeric, bounded).
//! - `N`: number of dimensions.
//! - `M`: maximum number of entries per node (leaf or non-leaf).
//!
//! # Design overview
//! - Nodes store bounding boxes that tightly enclose their children.
//! - Insertions follow the classic R-Tree choose-leaf / adjust-tree algorithm.
//! - Splits use a linear-cost strategy and enforce minimum fill (`ceil(M / 2)`).
//! - Removals condense the tree by pruning underfull nodes and reinserting entries.
//!
//! # Invariants
//! - Every `BoundingBox` has `start <= end` in every dimension.
//! - Every node stores at most `M` children or entries.
//! - Non-root nodes should have at least `ceil(M / 2)` entries after condensing.
//! - Each stored bounding box must enclose its corresponding entry or subtree.
//!
//! # Example
//! ```
//! use space_partitioning::rtree::{BoundingBox, RTree};
//!
//! let mut tree: RTree<f32, 2, 4> = RTree::default();
//! tree.insert(1, BoundingBox::from([0.0..=1.0, 2.0..=3.0]));
//! assert!(!tree.is_empty());
//! ```
//!
//! The tree also supports generic intersection queries using
//! types that implement `IntersectsWith<BoundingBox<_, _>>`.
#[allow(dead_code)]
mod bounding_box;
mod dimension_type;
#[allow(dead_code)]
mod extent;
#[allow(dead_code)]
mod nodes;
#[allow(clippy::module_inception)]
#[allow(dead_code)]
mod rtree;
#[allow(dead_code)]
mod splitting_strategies;

pub use crate::rtree::nodes::rtree_leaf::IndexRecordEntry as RTreeEntry;
pub use crate::rtree::rtree::BulkLoadStrategy;
pub use bounding_box::BoundingBox;
pub use dimension_type::DimensionType;
pub use extent::Extent;
pub use rtree::RTree;

/// Default tuple identifier type used by [`RTree`].
pub type DefaultTupleId = usize;
