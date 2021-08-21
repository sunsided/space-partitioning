use crate::intersections::IntersectsWith;
use crate::quadtree::quadrants::Quadrants;
use crate::quadtree::AABB;

/// A centered axis-aligned bounding box.
#[derive(Debug, Default)]
#[repr(C, align(8))]
pub struct CenteredAABB {
    /// The center X coordinate.
    pub center_x: i32,
    /// The center Y coordinate.
    pub center_y: i32,
    /// The half-width of the AABB.
    pub half_width: i32,
    /// The half-height of the AABB.
    pub half_height: i32,
}

impl CenteredAABB {
    #[inline]
    pub fn from_ltwh(left: i32, top: i32, width: i32, height: i32) -> Self {
        let hx = width >> 1;
        let hy = height >> 1;
        let mx = left + hx;
        let my = top + hy;
        Self {
            center_x: mx,
            center_y: my,
            half_width: hx,
            half_height: hy,
        }
    }

    // TODO: Prefer specialization, see https://github.com/rust-lang/rust/issues/31844
    #[inline]
    pub fn explore_quadrants_aabb(&self, other: &AABB) -> Quadrants {
        let explore_top = other.tl.y <= self.center_y;
        let explore_bottom = other.br.y > self.center_y;
        let explore_left = other.tl.x <= self.center_x;
        let explore_right = other.br.x > self.center_x;
        Quadrants::from_tests(explore_left, explore_top, explore_right, explore_bottom)
    }

    // TODO: Prefer specialization, see https://github.com/rust-lang/rust/issues/31844
    pub fn explore_quadrants_generic<T>(&self, other: &T) -> Quadrants
    where
        T: IntersectsWith<AABB>,
    {
        let mx = self.center_x;
        let my = self.center_y;
        let hx = self.half_width;
        let hy = self.half_height;

        let l = mx - hx;
        let r = mx + hx;
        let t = my - hy;
        let b = my + hy;

        let top_left = AABB::from_ranges(l..=mx, t..=my);
        let top_right = AABB::from_ranges(mx..=r, t..=my);
        let bottom_left = AABB::from_ranges(l..=mx, my..=b);
        let bottom_right = AABB::from_ranges(mx..=r, my..=b);

        Quadrants::from_intersections(
            other.intersects_with(&top_left),
            other.intersects_with(&top_right),
            other.intersects_with(&bottom_left),
            other.intersects_with(&bottom_right),
        )
    }

    #[inline]
    pub fn get_aabb(&self) -> AABB {
        AABB::new(self.left(), self.top(), self.right(), self.bottom())
    }

    #[inline]
    pub fn left(&self) -> i32 {
        self.center_x - self.half_width
    }

    #[inline]
    pub fn right(&self) -> i32 {
        self.center_x + self.half_width
    }

    #[inline]
    pub fn top(&self) -> i32 {
        self.center_y - self.half_height
    }

    #[inline]
    pub fn bottom(&self) -> i32 {
        self.center_y + self.half_height
    }
}

impl Into<AABB> for CenteredAABB {
    #[inline]
    fn into(self) -> AABB {
        self.get_aabb()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn aabb_i32_is_16_bytes() {
        assert_eq!(std::mem::size_of::<CenteredAABB>(), 16);
    }
}
