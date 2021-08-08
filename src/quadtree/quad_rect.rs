/// A rectangle describing the extents of a QudTree cell.
///
/// # Remarks
/// Only the tree node stores its extents. Bounding boxes for sub-nodes are computed on the fly.
#[derive(Default, Debug)]
pub struct QuadRect {
    // TODO: Might want to use a centered AABB instead, storing center and half-width/height?
    pub l: i32,
    pub t: i32,
    pub hx: i32,
    pub hy: i32
}