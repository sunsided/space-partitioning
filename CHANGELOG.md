# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.5.0 - 2021-08-22

### Changed

- The `intersect_X()` methods now return a vector, removing the need for the
  [hashbrown](https://crates.io/crates/hashbrown) crate. This is a breaking change.

### Internal

- The `QuadTree` now stores elements that intersect multiple quadrants
  in a separate node, reducing the total number of insertions and intersection tests.

## 0.4.0 - 2021-08-21

### Added

- Added opt-out support for [hashbrown](https://crates.io/crates/hashbrown).
- Added `QuadTree::intersect_aabb_fn()` and `QuadTree::intersect_generic_fn()` functions
  that execute a user-defined callback function for every intersected (candidate) item.

## 0.3.0 - 2021-08-21

### Changed

- For the `QuadTree`, the maximum number of elements per node and the smallest cell size
  are now parameters of the tree rather than constants.

### Internal

- The `QuadTree` now stores ID and AABB information for the nodes in two separate lists.
- The `QuadTree` now stores up to sixteen elements per node before it splits.

## 0.2.0 - 2021-08-18

### Added

- QuadTrees in `quadtree::QuadTree<T>`.

## 0.1.0 - 2021-08-01

### Added

- Added Interval Trees with the `interval_tree::IntervalTree` type. Only
  insertion to the tree is currently supported and no re-balancing is performed.
- Added in-order iterator for the Interval Tree.
