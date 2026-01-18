extern crate core;

pub mod intersections;
pub mod interval_tree;
pub mod quadtree;
pub mod rtree;

pub use interval_tree::IntervalTree;
pub use quadtree::QuadTree;
pub use rtree::{BoundingBox, DefaultTupleId, DimensionType, Extent, RTree, RTreeEntry};
