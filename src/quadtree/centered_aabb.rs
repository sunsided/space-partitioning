use crate::intersections::IntersectsWith;
use crate::quadtree::AABB;
use std::ops::Index;

/// A centered axis-aligned bounding box.
#[derive(Debug, Default)]
pub struct CenteredAABB {
    /// The center X coordinate.
    pub center_x: i32,
    /// The center Y coordinate.
    pub center_y: i32,
    /// The width.
    pub width: i32,
    /// The height.
    pub height: i32,
}

impl Index<usize> for CenteredAABB {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 4);
        let ptr = self as *const _ as *const [i32; 4];
        unsafe { &(*ptr)[index] }
    }
}

impl CenteredAABB {
    pub fn from_center_xy_wh(center_x: i32, center_y: i32, width: i32, height: i32) -> Self {
        Self {
            center_x,
            center_y,
            width,
            height,
        }
    }

    #[inline]
    pub fn from_ltwh(left: i32, top: i32, width: i32, height: i32) -> Self {
        let mx = left + (width >> 1);
        let my = top + (height >> 1);
        Self::from_center_xy_wh(mx, my, width, height)
    }

    // TODO: Prefer specialization, see https://github.com/rust-lang/rust/issues/31844
    pub fn explore_quadrants_aabb(&self, other: &AABB) -> Quadrants {
        let mx = self.center_x;
        let my = self.center_y;

        let explore_top = other.tl.y <= my;
        let explore_bottom = other.br.y > my;
        let explore_left = other.tl.x <= mx;
        let explore_right = other.br.x > mx;

        Quadrants {
            top_left: explore_top & explore_left,
            top_right: explore_top & explore_right,
            bottom_left: explore_bottom & explore_left,
            bottom_right: explore_bottom & explore_right,
        }
    }

    // TODO: Prefer specialization, see https://github.com/rust-lang/rust/issues/31844
    pub fn explore_quadrants_generic<T>(&self, other: &T) -> Quadrants
    where
        T: IntersectsWith<AABB>,
    {
        let mx = self.center_x;
        let my = self.center_y;
        let hx = self.width >> 1;
        let hy = self.height >> 1;

        let l = mx - hx;
        let t = my - hy;
        let r = mx + hx;
        let b = my + hy;

        let top_left = AABB::new(l, t, l + hx, t + hy);
        let top_right = AABB::new(r, t, r + hx, t + hy);
        let bottom_left = AABB::new(l, b, l + hx, b + hy);
        let bottom_right = AABB::new(r, b, r + hx, b + hy);

        Quadrants {
            top_left: other.intersects_with(&top_left),
            top_right: other.intersects_with(&top_right),
            bottom_left: other.intersects_with(&bottom_left),
            bottom_right: other.intersects_with(&bottom_right),
        }
    }
}

pub struct Quadrants {
    pub top_left: bool,
    pub top_right: bool,
    pub bottom_left: bool,
    pub bottom_right: bool,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn aabb_i32_is_16_bytes() {
        assert_eq!(std::mem::size_of::<CenteredAABB>(), 16);
    }
    #[test]
    fn from_ltwh_index_i32_works() {
        let aabb = CenteredAABB::from_ltwh(-5, 0, 30, 40);
        assert_eq!(aabb[0], 10);
        assert_eq!(aabb[1], 20);
        assert_eq!(aabb[2], 30);
        assert_eq!(aabb[3], 40);
    }

    #[test]
    fn from_center_xy_wh_index_i32_works() {
        let aabb = CenteredAABB::from_center_xy_wh(10, 20, 30, 40);
        assert_eq!(aabb[0], 10);
        assert_eq!(aabb[1], 20);
        assert_eq!(aabb[2], 30);
        assert_eq!(aabb[3], 40);
    }
}
