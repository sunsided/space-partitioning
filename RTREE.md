# R-Tree Feature Plan

This branch introduces an R-Tree implementation to index axis-aligned bounding boxes (AABBs)
across N dimensions. The goal is a usable, tested API that supports insertion, querying,
and future extensions (deletion, bulk load, nearest-neighbor).

## High-level goals
- Provide a public `RTree` API with a minimal, stable surface.
- Support insertion and intersection/containment queries.
- Implement node splitting with linear-cost strategy (already started).
- Maintain correctness (MBBs, min/max fill) and test coverage.

## Step-by-step plan

### 1) Public API surface
- [x] Expose `RTree`, `BoundingBox`, and relevant types from `src/rtree.rs` and `src/lib.rs`.
- [x] Decide on user-facing types for ids and entries (e.g., `TupleIdentifier`).
- [x] Add minimal docs and examples in `src/rtree/README.md` or crate docs.

Notes (implemented so far)
- `RTree::insert` now uses the generic `TupleIdentifier` instead of forcing `usize`.
- `IndexRecordEntry` is public and re-exported as `RTreeEntry`.
- Crate-level re-exports added for `RTree`, `BoundingBox`, `Extent`, `DimensionType`, and `DefaultTupleId`.
- Added module-level docs and a quick-start example.

Suggestions (next improvements)
- Consider renaming `DefaultTupleId`/`RTreeEntry` to match existing crate naming patterns.
- Add a small `examples/rtree.rs` once insertion compiles and basic queries exist.
- Expose `BoundingBox::new_from_ranges` (or a helper) in docs to show idiomatic construction.
- Clarify whether `Extent` and `DimensionType` should be public API or stay internal.

### 2) Core insertion path
- [x] Implement `choose_leaf` traversal logic in `src/rtree/rtree.rs`.
- [x] Implement `insert` to add records, handle overflow, and return result/errors if needed.
- [x] Implement `adjust_tree` to propagate MBB changes and splits up to the root.

Notes
- `choose_leaf` now selects children by least area enlargement (tie-breaker: smallest area).
- `insert` now handles leaf insertion, leaf splits, split propagation, and root growth.
- `adjust_tree` updates parent MBBs and propagates splits up to the root.
- Added `ChildPointer` bounding box support to enable splitting at internal levels.
- `rtree::rtree::test::simple_insert_works` and `rtree::rtree::test::insert_works` now assert real structure/MBBs.

### 3) Node storage and MBB correctness
- [x] Fix MBB accumulation so empty nodes don’t include the origin by default.
- [x] Ensure MBBs update correctly on insert and split for both leaf and non-leaf nodes.
- [x] Add helpers for recomputing MBBs from child entries.

### 4) Splitting strategy correctness
- [x] Enforce min-fill during linear-cost split (assign remaining entries when forced).
- [x] Validate separation calculation and normalization logic.
- [x] Add tests that verify split groups and min-fill guarantees.

### 5) Query support (initial)
- [x] Define intersection/containment query API.
- [x] Implement tree traversal for queries using MBB pruning.
- [x] Add tests for query correctness.

### 6) Testing & examples
- [x] Replace `todo!()` tests with real assertions. (none found)
- [x] Add basic integration tests for insert/query behavior.
- [x] Add a small example in `examples/rtree.rs` (or similar).

Suggestions
- Add tests for choose-leaf tie-breakers (equal enlargement, smaller area).
- Add tests that validate internal-node split propagation and root growth (multi-level trees).
- Add tests covering min-fill edge cases (odd/even M, forced assignment order).
- Add tests for MBB recomputation when inserting into previously empty nodes.
- Add property-style tests for splitting (all entries preserved, MBBs are tight, min-fill honored).

### 7) Follow-up extensions (optional)
- [x] Deletion with underflow handling and reinsertion.
- [x] Bulk-loading variants (STR, Hilbert).
- [x] Nearest-neighbor search.
- [x] Benchmarks for insert/query/delete/bulk-load/nearest-neighbor.
