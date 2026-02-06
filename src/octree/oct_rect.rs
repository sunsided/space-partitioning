use crate::octree::centered_aabb::CenteredAABB;
use crate::octree::AABB;

/// A rectangular prism describing the extents of the OctTree.
/// Only the tree node stores its extents; bounding boxes for sub-nodes are computed on the fly.
#[derive(Debug, Copy, Clone)]
pub struct OctRect {
    l: i32,
    t: i32,
    f: i32,
    hx: i32,
    hy: i32,
    hz: i32,
}

impl OctRect {
    pub fn new(left: i32, top: i32, front: i32, width: i32, height: i32, depth: i32) -> Self {
        Self {
            l: left,
            t: top,
            f: front,
            hx: width,
            hy: height,
            hz: depth,
        }
    }

    #[inline]
    pub fn contains(&self, rect: &AABB) -> bool {
        let mx = (rect.min.x + rect.max.x) >> 1;
        let my = (rect.min.y + rect.max.y) >> 1;
        let mz = (rect.min.z + rect.max.z) >> 1;

        let r = self.l + self.hx;
        let b = self.t + self.hy;
        let ba = self.f + self.hz;
        (mx >= self.l) & (mx <= r) & (my >= self.t) & (my <= b) & (mz >= self.f) & (mz <= ba)
    }
}

impl Default for OctRect {
    fn default() -> Self {
        OctRect {
            l: i32::MIN >> 1,
            t: i32::MIN >> 1,
            f: i32::MIN >> 1,
            hx: i32::MAX,
            hy: i32::MAX,
            hz: i32::MAX,
        }
    }
}

impl From<OctRect> for AABB {
    #[inline]
    fn from(val: OctRect) -> Self {
        AABB::new(
            val.l,
            val.t,
            val.f,
            val.l + val.hx,
            val.t + val.hy,
            val.f + val.hz,
        )
    }
}

impl From<OctRect> for CenteredAABB {
    #[inline]
    fn from(val: OctRect) -> Self {
        CenteredAABB::from_ltfwhd(val.l, val.t, val.f, val.hx, val.hy, val.hz)
    }
}

impl From<&OctRect> for CenteredAABB {
    #[inline]
    fn from(val: &OctRect) -> Self {
        CenteredAABB::from_ltfwhd(val.l, val.t, val.f, val.hx, val.hy, val.hz)
    }
}
