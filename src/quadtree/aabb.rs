use crate::intersections::IntersectsWith;

/// An axis-aligned bounding box defined by its edge coordinates.
#[derive(Debug, PartialEq, Eq, Default, Copy, Clone)]
pub struct AABB {
    /// Left X coordinate of the rectangle of the element.
    pub x1: i32,
    /// Top Y coordinate of the rectangle of the element.
    pub y1: i32,
    /// Right X coordinate of the rectangle of the element.
    pub x2: i32,
    /// Bottom Y coordinate of the rectangle of the element.
    pub y2: i32,
}

impl AABB {
    /// Constructs a new [`AABB`] from the coordinates of its edges.
    ///
    /// # Arguments
    /// * [`x1`] - The left-most X coordinate.
    /// * [`y1`] - The top-most Y coordinate.
    /// * [`x2`] - The right-most X coordinate.
    /// * [`y2`] - The bottom-most Y coordinate.
    #[inline]
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

impl IntersectsWith<AABB> for AABB {
    /// Tests whether this [`AABB`] intersects with another one.
    ///
    /// # Remarks
    /// It is assumed that none of the AABBs is degenerate,
    /// i.e., neither a line nor a point.
    ///
    /// # Arguments
    /// * [`other`] - The AABB to test for intersection.
    #[inline]
    fn intersects_with(&self, other: &AABB) -> bool {
        // TODO: We might want to have tree specifically for storing point data rather than rects
        //       as this would simplify the tests below.

        let x1_max = self.x1.max(other.x1);
        let x2_min = self.x2.min(other.x2);
        let y1_max = self.y1.max(other.y1);
        let y2_min = self.y2.min(other.y2);

        // In the non-degenerate case (rect/rect), this covers the intersection.
        let a = x1_max < x2_min;
        let b = y1_max < y2_min;
        let intersects = a & b;

        // If intersects is true, we could skip the entire following
        // block. With instruction pipelining, this could incur penalties from
        // branch mis-predictions however, so it might be better to just calculate
        // the test for degenerate cases anyway.

        // In the degenerate case we need a more relaxed test.
        let d_a = x1_max <= x2_min;
        let d_b = y1_max <= y2_min;

        // Only use the above values in degenerate cases.
        let degenerate_x = (other.x1 == other.x2) | (self.x1 == self.x2);
        let degenerate_y = (other.y1 == other.y2) | (self.y1 == self.y2);
        let is_degenerate = degenerate_x | degenerate_y;
        let d_intersects = is_degenerate & d_a & d_b;

        intersects | d_intersects
    }
}

impl From<[i32; 4]> for AABB {
    #[inline]
    fn from(rect: [i32; 4]) -> Self {
        Self::from(&rect)
    }
}

impl From<&[i32; 4]> for AABB {
    #[inline]
    fn from(rect: &[i32; 4]) -> Self {
        Self::new(rect[0], rect[1], rect[2], rect[3])
    }
}

impl Into<[i32; 4]> for AABB {
    fn into(self) -> [i32; 4] {
        [self.x1, self.y1, self.x2, self.y2]
    }
}

impl AsRef<[i32; 4]> for AABB {
    fn as_ref(&self) -> &[i32; 4] {
        let ptr = self as *const _ as *const [i32; 4];
        unsafe { ptr.as_ref() }.unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn aabb_is_16_bytes() {
        assert_eq!(std::mem::size_of::<AABB>(), 16);
    }

    #[test]
    fn from_works() {
        let aabb = AABB::from([1, 2, 3, 4]);
        assert_eq!(aabb.x1, 1);
        assert_eq!(aabb.y1, 2);
        assert_eq!(aabb.x2, 3);
        assert_eq!(aabb.y2, 4);
    }

    #[test]
    fn from_ref_works() {
        let aabb = AABB::from(&[1, 2, 3, 4]);
        assert_eq!(aabb.x1, 1);
        assert_eq!(aabb.y1, 2);
        assert_eq!(aabb.x2, 3);
        assert_eq!(aabb.y2, 4);
    }

    #[test]
    fn as_ref_works() {
        let aabb = AABB::new(1, 2, 3, 4);
        let array: &[i32; 4] = aabb.as_ref();
        for i in 1..=4 {
            assert_eq!(array[i as usize - 1], i);
        }
    }

    #[test]
    fn intersects_with_self_works() {
        let a = AABB::new(0, 0, 1, 1);
        assert!(a.intersects_with(&a));
    }

    #[test]
    fn intersects_when_partial_overlap_works() {
        let a = AABB::new(0, 0, 2, 2);
        let b = AABB::new(1, 1, 3, 3);
        assert!(a.intersects_with(&b));
        assert!(b.intersects_with(&a));

        let a = AABB::new(0, 0, 2, 2);
        let b = AABB::new(-1, -1, 1, 1);
        assert!(a.intersects_with(&b));
        assert!(b.intersects_with(&a));

        let a = AABB::new(0, 0, 2, 2);
        let b = AABB::new(-1, 1, 1, 2);
        assert!(a.intersects_with(&b));
        assert!(b.intersects_with(&a));
    }

    #[test]
    fn intersects_when_not_overlapping_works() {
        let a = AABB::new(0, 0, 2, 2);
        let b = AABB::new(2, 0, 3, 3);
        assert!(!a.intersects_with(&b));
        assert!(!b.intersects_with(&a));

        let c = AABB::new(0, 0, 2, 2);
        let d = AABB::new(10, 10, 12, 12);
        assert!(!c.intersects_with(&d));
        assert!(!d.intersects_with(&c));
    }

    #[test]
    fn intersects_when_degenerate_works() {
        // With a line
        let a = AABB::new(-1, 0, 0, -1);
        let b = AABB::new(1, 1, 0, 1);
        assert!(!a.intersects_with(&b));
        assert!(!a.intersects_with(&a));
        assert!(!b.intersects_with(&b));
    }

    #[test]
    fn intersects_rect_point_works() {
        let point = AABB::new(3, 3, 3, 3);

        // Point lies inside the rectangle.
        let covering_rect = AABB::new(-10, -10, 10, 10);
        assert!(covering_rect.intersects_with(&point));

        // Point lies outside the rectangle.
        let other_rect = AABB::new(-10, -10, 0, 0);
        assert!(!other_rect.intersects_with(&point));
    }

    #[test]
    fn intersects_line_point_works() {
        let line = AABB::new(-10, 0, 10, 0);
        let point = AABB::new(1, 0, 1, 0);
        assert!(line.intersects_with(&point));

        let boundary_point = AABB::new(10, 0, 10, 0);
        assert!(line.intersects_with(&boundary_point));
    }

    #[test]
    fn intersects_point_point_works() {
        let point = AABB::new(-1, -1, -1, -1);
        assert!(point.intersects_with(&point));
    }
}
