use crate::intersections::IntersectsWith;
use crate::octree::octants::Octants;
use crate::octree::AABB;
use std::ops::RangeInclusive;

/// A centered axis-aligned bounding box in 3D.
#[derive(Debug, Default, Copy, Clone)]
#[repr(C, align(8))]
pub struct CenteredAABB {
    pub center_x: i32,
    pub center_y: i32,
    pub center_z: i32,
    pub half_width: i32,
    pub half_height: i32,
    pub half_depth: i32,
}

impl CenteredAABB {
    #[inline]
    pub fn from_ltfwhd(
        left: i32,
        top: i32,
        front: i32,
        width: i32,
        height: i32,
        depth: i32,
    ) -> Self {
        let hx = width >> 1;
        let hy = height >> 1;
        let hz = depth >> 1;
        let mx = left + hx;
        let my = top + hy;
        let mz = front + hz;
        Self {
            center_x: mx,
            center_y: my,
            center_z: mz,
            half_width: hx,
            half_height: hy,
            half_depth: hz,
        }
    }

    #[inline]
    pub fn explore_octants_aabb(&self, other: &AABB) -> Octants {
        let explore_left = other.min.x <= self.center_x;
        let explore_right = other.max.x > self.center_x;
        let explore_top = other.min.y <= self.center_y;
        let explore_bottom = other.max.y > self.center_y;
        let explore_front = other.min.z <= self.center_z;
        let explore_back = other.max.z > self.center_z;
        Octants::from_tests(
            explore_left,
            explore_top,
            explore_right,
            explore_bottom,
            explore_front,
            explore_back,
        )
    }

    #[inline]
    pub fn explore_octants_generic<T>(&self, other: &T) -> Octants
    where
        T: IntersectsWith<AABB>,
    {
        let l = self.left_half();
        let r = self.right_half();
        let t = self.top_half();
        let b = self.bottom_half();
        let f = self.front_half();
        let ba = self.back_half();

        let ltf = AABB::from_ranges(l.clone(), t.clone(), f.clone());
        let rtf = AABB::from_ranges(r.clone(), t.clone(), f.clone());
        let lbf = AABB::from_ranges(l.clone(), b.clone(), f.clone());
        let rbf = AABB::from_ranges(r.clone(), b.clone(), f.clone());
        let ltb = AABB::from_ranges(l.clone(), t.clone(), ba.clone());
        let rtb = AABB::from_ranges(r.clone(), t.clone(), ba.clone());
        let lbb = AABB::from_ranges(l.clone(), b.clone(), ba.clone());
        let rbb = AABB::from_ranges(r, b, ba);

        Octants::from_intersections(
            other.intersects_with(&ltf),
            other.intersects_with(&rtf),
            other.intersects_with(&lbf),
            other.intersects_with(&rbf),
            other.intersects_with(&ltb),
            other.intersects_with(&rtb),
            other.intersects_with(&lbb),
            other.intersects_with(&rbb),
        )
    }

    #[inline]
    pub fn get_aabb(&self) -> AABB {
        AABB::new(
            self.left(),
            self.top(),
            self.front(),
            self.right(),
            self.bottom(),
            self.back(),
        )
    }

    #[inline]
    pub fn split_octants(&self) -> [CenteredAABB; 9] {
        let hx = self.half_width >> 1;
        let hy = self.half_height >> 1;
        let hz = self.half_depth >> 1;

        let cx_l = self.center_x - hx;
        let cx_r = self.center_x + hx;
        let cy_t = self.center_y - hy;
        let cy_b = self.center_y + hy;
        let cz_f = self.center_z - hz;
        let cz_b = self.center_z + hz;

        let child = |cx, cy, cz| CenteredAABB {
            center_x: cx,
            center_y: cy,
            center_z: cz,
            half_width: hx,
            half_height: hy,
            half_depth: hz,
        };

        [
            *self,
            child(cx_l, cy_t, cz_f),
            child(cx_r, cy_t, cz_f),
            child(cx_l, cy_b, cz_f),
            child(cx_r, cy_b, cz_f),
            child(cx_l, cy_t, cz_b),
            child(cx_r, cy_t, cz_b),
            child(cx_l, cy_b, cz_b),
            child(cx_r, cy_b, cz_b),
        ]
    }

    #[inline]
    fn left_half(&self) -> RangeInclusive<i32> {
        self.left()..=self.center_x
    }

    #[inline]
    fn right_half(&self) -> RangeInclusive<i32> {
        self.center_x..=self.right()
    }

    #[inline]
    fn top_half(&self) -> RangeInclusive<i32> {
        self.top()..=self.center_y
    }

    #[inline]
    fn bottom_half(&self) -> RangeInclusive<i32> {
        self.center_y..=self.bottom()
    }

    #[inline]
    fn front_half(&self) -> RangeInclusive<i32> {
        self.front()..=self.center_z
    }

    #[inline]
    fn back_half(&self) -> RangeInclusive<i32> {
        self.center_z..=self.back()
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

    #[inline]
    pub fn front(&self) -> i32 {
        self.center_z - self.half_depth
    }

    #[inline]
    pub fn back(&self) -> i32 {
        self.center_z + self.half_depth
    }
}

impl From<CenteredAABB> for AABB {
    #[inline]
    fn from(val: CenteredAABB) -> Self {
        val.get_aabb()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn centered_aabb_is_24_bytes() {
        assert_eq!(std::mem::size_of::<CenteredAABB>(), 24);
    }
}
