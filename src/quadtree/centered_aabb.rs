use std::ops::Index;

/// A centered axis-aligned bounding box.
#[derive(Debug, Default)]
pub struct CenteredAABB<T = i32> {
    /// The center X coordinate.
    pub center_x: T,
    /// The center Y coordinate.
    pub center_y: T,
    /// The width.
    pub width: T,
    /// The height.
    pub height: T,
}

impl<T> Index<usize> for CenteredAABB<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 4);
        let ptr = self as *const _ as *const [T; 4];
        unsafe { &(*ptr)[index] }
    }
}

impl<T> CenteredAABB<T>
where
    T: Copy,
{
    pub fn from_center_xy_wh(center_x: T, center_y: T, width: T, height: T) -> Self {
        Self {
            center_x,
            center_y,
            width,
            height,
        }
    }

    pub fn from_array(xywh: [T; 4]) -> Self {
        Self {
            center_x: xywh[0],
            center_y: xywh[1],
            width: xywh[2],
            height: xywh[3],
        }
    }
}

pub trait FromLeftTopWidthHeight<T> {
    fn from_ltwh(left: T, top: T, width: T, height: T) -> CenteredAABB<T>;
}

impl FromLeftTopWidthHeight<i32> for CenteredAABB<i32> {
    #[inline]
    fn from_ltwh(left: i32, top: i32, width: i32, height: i32) -> Self {
        let mx = left + (width >> 1);
        let my = top + (height >> 1);
        Self::from_center_xy_wh(mx, my, width, height)
    }
}

impl FromLeftTopWidthHeight<f32> for CenteredAABB<f32> {
    #[inline]
    fn from_ltwh(left: f32, top: f32, width: f32, height: f32) -> Self {
        let mx = left + (width * 0.5f32);
        let my = top + (height * 0.5f32);
        Self::from_center_xy_wh(mx, my, width, height)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn aabb_u8_is_4_bytes() {
        assert_eq!(std::mem::size_of::<CenteredAABB<u8>>(), 4);
    }

    #[test]
    fn aabb_i16_is_8_bytes() {
        assert_eq!(std::mem::size_of::<CenteredAABB<i16>>(), 8);
    }

    #[test]
    fn aabb_i32_is_16_bytes() {
        assert_eq!(std::mem::size_of::<CenteredAABB<i32>>(), 16);
    }

    #[test]
    fn aabb_f32_is_16_bytes() {
        assert_eq!(std::mem::size_of::<CenteredAABB<f32>>(), 16);
    }

    #[test]
    fn aabb_f64_is_32_bytes() {
        assert_eq!(std::mem::size_of::<CenteredAABB<f64>>(), 32);
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

    #[test]
    fn index_f64_works() {
        let aabb = CenteredAABB::from_center_xy_wh(10.0, 20.0, 30.0, 40.0);
        assert_eq!(aabb[0], 10.0);
        assert_eq!(aabb[1], 20.0);
        assert_eq!(aabb[2], 30.0);
        assert_eq!(aabb[3], 40.0);
    }
}
