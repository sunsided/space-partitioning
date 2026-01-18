//! R-Tree implementation for axis-aligned bounding boxes.
//!
//! # Example
//! ```
//! use space_partitioning::rtree::{BoundingBox, RTree};
//!
//! let mut tree: RTree<f32, 2, 4> = RTree::default();
//! tree.insert(1, BoundingBox::from([0.0..=1.0, 2.0..=3.0]));
//! assert!(!tree.is_empty());
//! ```
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

pub use bounding_box::BoundingBox;
pub use dimension_type::DimensionType;
pub use extent::Extent;
pub use rtree::RTree;
pub use crate::rtree::rtree::BulkLoadStrategy;
pub use crate::rtree::nodes::rtree_leaf::IndexRecordEntry as RTreeEntry;

/// Default tuple identifier type used by [`RTree`].
pub type DefaultTupleId = usize;
