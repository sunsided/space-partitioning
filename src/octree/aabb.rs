use crate::intersections::IntersectsWith;
use crate::octree::Point;
use std::ops::{Add, RangeInclusive};

/// An axis-aligned bounding box in 3D defined by its minimum and maximum corners.
#[derive(Debug, PartialEq, Eq, Default, Copy, Clone)]
pub struct AABB {
    /// Minimum (left, top, front) corner.
    pub min: Point,
    /// Maximum (right, bottom, back) corner.
    pub max: Point,
}

impl AABB {
    /// Constructs a new [`AABB`] from the coordinates of its edges.
    #[inline]
    pub fn new(x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32) -> Self {
        Self {
            min: Point::new(x1, y1, z1),
            max: Point::new(x2, y2, z2),
        }
    }

    #[inline]
    pub fn from_ranges(
        x: RangeInclusive<i32>,
        y: RangeInclusive<i32>,
        z: RangeInclusive<i32>,
    ) -> Self {
        Self {
            min: Point::new(*x.start(), *y.start(), *z.start()),
            max: Point::new(*x.end(), *y.end(), *z.end()),
        }
    }
}

impl IntersectsWith<AABB> for AABB {
    /// Tests whether this [`AABB`] intersects with another one.
    #[inline]
    fn intersects_with(&self, other: &AABB) -> bool {
        let x1_max = self.min.x.max(other.min.x);
        let x2_min = self.max.x.min(other.max.x);
        let y1_max = self.min.y.max(other.min.y);
        let y2_min = self.max.y.min(other.max.y);
        let z1_max = self.min.z.max(other.min.z);
        let z2_min = self.max.z.min(other.max.z);

        let intersects = (x1_max < x2_min) & (y1_max < y2_min) & (z1_max < z2_min);

        let d_a = x1_max <= x2_min;
        let d_b = y1_max <= y2_min;
        let d_c = z1_max <= z2_min;

        let degenerate_x = (other.min.x == other.max.x) | (self.min.x == self.max.x);
        let degenerate_y = (other.min.y == other.max.y) | (self.min.y == self.max.y);
        let degenerate_z = (other.min.z == other.max.z) | (self.min.z == self.max.z);
        let is_degenerate = degenerate_x | degenerate_y | degenerate_z;
        let d_intersects = is_degenerate & d_a & d_b & d_c;

        intersects | d_intersects
    }
}

impl Add for AABB {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let min_x = self.min.x.min(rhs.min.x);
        let min_y = self.min.y.min(rhs.min.y);
        let min_z = self.min.z.min(rhs.min.z);
        let max_x = self.max.x.max(rhs.max.x);
        let max_y = self.max.y.max(rhs.max.y);
        let max_z = self.max.z.max(rhs.max.z);
        AABB::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }
}

impl From<[i32; 6]> for AABB {
    #[inline]
    fn from(rect: [i32; 6]) -> Self {
        Self::from(&rect)
    }
}

impl From<&[i32; 6]> for AABB {
    #[inline]
    fn from(rect: &[i32; 6]) -> Self {
        Self::new(rect[0], rect[1], rect[2], rect[3], rect[4], rect[5])
    }
}

impl From<AABB> for [i32; 6] {
    fn from(val: AABB) -> Self {
        [
            val.min.x, val.min.y, val.min.z, val.max.x, val.max.y, val.max.z,
        ]
    }
}

impl AsRef<[i32; 6]> for AABB {
    fn as_ref(&self) -> &[i32; 6] {
        let ptr = self as *const _ as *const [i32; 6];
        unsafe { ptr.as_ref() }.unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn aabb_is_24_bytes() {
        assert_eq!(std::mem::size_of::<AABB>(), 24);
    }

    #[test]
    fn from_works() {
        let aabb = AABB::from([1, 2, 3, 4, 5, 6]);
        assert_eq!(aabb.min.x, 1);
        assert_eq!(aabb.min.y, 2);
        assert_eq!(aabb.min.z, 3);
        assert_eq!(aabb.max.x, 4);
        assert_eq!(aabb.max.y, 5);
        assert_eq!(aabb.max.z, 6);
    }

    #[test]
    fn from_ref_works() {
        let aabb = AABB::from(&[1, 2, 3, 4, 5, 6]);
        assert_eq!(aabb.min.x, 1);
        assert_eq!(aabb.min.y, 2);
        assert_eq!(aabb.min.z, 3);
        assert_eq!(aabb.max.x, 4);
        assert_eq!(aabb.max.y, 5);
        assert_eq!(aabb.max.z, 6);
    }

    #[test]
    fn as_ref_works() {
        let aabb = AABB::new(1, 2, 3, 4, 5, 6);
        let array: &[i32; 6] = aabb.as_ref();
        for i in 1..=6 {
            assert_eq!(array[i as usize - 1], i);
        }
    }

    #[test]
    fn intersects_with_self_works() {
        let a = AABB::new(0, 0, 0, 1, 1, 1);
        assert!(a.intersects_with(&a));
    }
}
