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
    #[inline]
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        Self { x1, y1, x2, y2 }
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
}
