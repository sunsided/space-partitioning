use crate::intersections::IntersectsWith;
use crate::quadtree::aabb::AABB;
use crate::quadtree::centered_aabb::CenteredAABB;

/// A rectangle describing the extents of the QuadTree.
///
/// # Remarks
/// Only the tree node stores its extents. Bounding boxes for sub-nodes are computed on the fly.
#[derive(Debug, Copy, Clone)]
pub struct QuadRect {
    l: i32,
    t: i32,
    hx: i32,
    hy: i32,
}

impl QuadRect {
    pub fn new(left: i32, top: i32, width: i32, height: i32) -> Self {
        Self {
            l: left,
            t: top,
            hx: width,
            hy: height,
        }
    }

    pub fn contains(&self, rect: &AABB) -> bool {
        let mx = (rect.tl.x + rect.br.x) >> 1;
        let my = (rect.tl.y + rect.br.y) >> 1;

        let r = self.l + self.hx;
        let b = self.t + self.hy;
        mx >= self.l && mx <= r && my >= self.t && my <= b
    }
}

impl Default for QuadRect {
    fn default() -> Self {
        QuadRect {
            l: i32::MIN >> 1,
            t: i32::MIN >> 1,
            hx: i32::MAX,
            hy: i32::MAX,
        }
    }
}

impl Into<AABB> for QuadRect {
    fn into(self) -> AABB {
        AABB::new(self.l, self.t, self.l + self.hx, self.t + self.hy)
    }
}

impl Into<CenteredAABB> for QuadRect {
    fn into(self) -> CenteredAABB {
        CenteredAABB::from_ltwh(self.l, self.t, self.hx, self.hy)
    }
}

impl Into<CenteredAABB> for &QuadRect {
    fn into(self) -> CenteredAABB {
        CenteredAABB::from_ltwh(self.l, self.t, self.hx, self.hy)
    }
}
