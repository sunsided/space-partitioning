use crate::rtree::dimension_type::DimensionType;
use crate::rtree::extent::Extent;
pub use num_traits::Num;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Range;

/// An N-dimensional bounding box.
///
/// The struct is parameterized by `T`, the data type of a
/// dimension, and `N`, the number of dimensions.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox<T, const N: usize>
where
    T: DimensionType,
{
    /// The dimensions of the box.
    ///
    /// Each range entry represents the extent of the box
    /// along the particular dimension.
    pub dims: [Extent<T>; N],
}

impl<T, const N: usize> Default for BoundingBox<T, N>
where
    T: DimensionType,
{
    fn default() -> Self {
        BoundingBox::new([Extent::default(); N])
    }
}

impl<T, const N: usize> BoundingBox<T, N>
where
    T: DimensionType,
{
    pub fn new(dims: [Extent<T>; N]) -> Self {
        Self { dims }
    }

    pub fn new_from_ranges<R: Borrow<[Range<T>; N]>>(dims: R) -> Self {
        let dims = dims.borrow();
        let mut data: [Extent<T>; N] = [Extent::default(); N];

        for i in 0..N {
            data[i] = Extent::from(&dims[i]);
        }

        Self { dims: data }
    }

    pub fn len(self: &Self) -> usize {
        return N;
    }
}

impl<const N: usize, T, R: Borrow<[Range<T>; N]>> From<R> for BoundingBox<T, N>
where
    T: DimensionType,
{
    fn from(dims: R) -> Self {
        Self::new_from_ranges(dims)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn construction_works() {
        let b = BoundingBox {
            dims: [Extent::from(0.0..1.0), Extent::from(1.0..2.0)],
        };
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn new_works() {
        let b = BoundingBox::new([Extent::from(0.0..1.0), Extent::from(0.1..2.0)]);
        assert_eq!(b.len(), 2);
        assert_eq!(b.dims[0], (0.0..1.0).into());
        assert_eq!(b.dims[1], (0.1..2.0).into());
    }

    #[test]
    fn new_from_ranges_works() {
        let a = BoundingBox::from([0.0..1.0, 0.1..2.0]);
        let b = BoundingBox::from(&[0.0..1.0, 0.1..2.0]);
        let c = BoundingBox::new([Extent::from(0.0..1.0), Extent::from(0.1..2.0)]);
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn default_works() {
        let b = BoundingBox::<f64, 3>::default();
        assert_eq!(b.len(), 3);
        assert_eq!(b.dims[0], Extent::default());
        assert_eq!(b.dims[1], Extent::default());
        assert_eq!(b.dims[2], Extent::default());
    }
}
