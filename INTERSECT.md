# Quadtree Line/Ray Intersection (Generic)

## Feature summary
The quadtree does not expose a dedicated line-search API. Instead, it provides a
generic intersection query that accepts any shape implementing
`IntersectsWith<AABB>`. A line or ray query is supported by defining a type
(e.g., `Ray`) that implements the ray/AABB slab test and then invoking
`QuadTree::intersect_generic` or `QuadTree::intersect_generic_fn`. The traversal
prunes by leaf nodes first and then checks candidate elements for intersection
using the provided `IntersectsWith` implementation. Degenerate AABBs (lines or
points) are handled by the AABB intersection logic, which keeps edge cases
consistent.

Relevant code paths:
- `QuadTree::intersect_generic` / `QuadTree::intersect_generic_fn` in
  `src/quadtree/quadtree.rs`.
- Ray/AABB example in tests (`src/quadtree.rs`, module `ray_box`).
- Degenerate AABB intersection handling in `src/quadtree/aabb.rs`.

## Tasks
1) Document the generic intersection mechanism in the quadtree docs
   - Add a short section to `src/quadtree.rs` or `README.md` explaining
     `intersect_generic` usage for rays/lines.

2) Provide a public example type and helper
   - Add a minimal `Ray` (or `LineSegment`) example in `examples/` with a
     `IntersectsWith<AABB>` implementation and a short query demo.

3) Optional: add dedicated convenience APIs
   - Add `intersect_ray`/`intersect_segment` wrappers that internally call
     `intersect_generic`, keeping the core traversal unchanged.

4) Extend tests
   - Promote the existing ray tests to a public-facing example or add additional
     coverage for line segments, boundary hits, and degenerate cases.
