use crate::rtree::dimension_type::DimensionType;
use crate::rtree::extent::{Contains, Extent};
pub use num_traits::Num;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::ops::RangeInclusive;

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

impl<T, const N: usize> BoundingBox<T, N>
where
    T: DimensionType,
{
    /// Initializes a new box from the specified dimensions.
    pub fn new(dims: [Extent<T>; N]) -> Self {
        debug_assert_ne!(N, 0);
        Self { dims }
    }

    /// Initializes a new box from the specified ranges.
    pub fn new_from_ranges<R: Borrow<[RangeInclusive<T>; N]>>(dims: R) -> Self {
        debug_assert_ne!(N, 0);
        let dims: &[RangeInclusive<T>; N] = dims.borrow();

        let mut data: [MaybeUninit<Extent<T>>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..N {
            data[i].write(Extent::from(&dims[i]));
        }
        // mem::transmute() doesn't work due to the generic T.
        let data = unsafe { data.as_ptr().cast::<[Extent<T>; N]>().read() };

        return BoundingBox::new(data);
    }

    /// Gets the number of dimensions of the bounding box.
    ///
    /// This value is a compile-time constant determined
    /// by the generic parameter `N`.
    #[inline]
    pub fn len(&self) -> usize {
        return N;
    }

    /// Tests whether this box fully contains another one.
    pub fn contains<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> bool {
        let other = other.borrow();
        for i in 0..N {
            if !self.dims[i].contains(other.dims[i]) {
                return false;
            }
        }
        true
    }

    /// Grows this bounding box of this node to tightly fit all elements.
    pub fn grow<B: Borrow<BoundingBox<T, N>>>(&mut self, other: B) {
        let other = other.borrow();
        for d in 0..N {
            self.dims[d].grow(&other.dims[d]);
        }
    }

    /// Grows this bounding box of this node to tightly fit all elements.
    /// Consumes self and returns an updated version.
    pub fn into_grown<B: Borrow<BoundingBox<T, N>>>(mut self, other: B) -> BoundingBox<T, N> {
        let other = other.borrow();
        for d in 0..N {
            self.dims[d].grow(&other.dims[d]);
        }
        self
    }

    /// Calculates the area of the box.
    pub fn area(&self) -> T {
        let mut area = T::one();
        for d in 0..N {
            area = area * self.dims[d].len()
        }
        area
    }

    /// Grows this bounding box of this node to tightly fit all elements.
    pub fn get_grown<B: Borrow<BoundingBox<T, N>>>(&self, other: B) -> BoxAndArea<T, N> {
        let other = other.borrow();
        if self.contains(other) {
            return BoxAndArea {
                bb: self.clone(),
                area: self.area(),
                area_increase: T::zero(),
            };
        }

        let mut new = self.clone();
        let mut area = T::one();
        for d in 0..N {
            new.dims[d].grow(&other.dims[d]);
            area = area * new.dims[d].len()
        }
        BoxAndArea {
            bb: new,
            area,
            area_increase: area - self.area(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoxAndArea<T, const N: usize>
where
    T: DimensionType,
{
    pub(crate) bb: BoundingBox<T, N>,
    pub area: T,
    pub area_increase: T,
}

impl<T, const N: usize> Default for BoundingBox<T, N>
where
    T: DimensionType,
{
    fn default() -> Self {
        debug_assert_ne!(N, 0);
        BoundingBox::new([Extent::default(); N])
    }
}

impl<T, R, const N: usize> From<R> for BoundingBox<T, N>
where
    T: DimensionType,
    R: Borrow<[RangeInclusive<T>; N]>,
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
            dims: [Extent::from(0.0..=1.0), Extent::from(1.0..=2.0)],
        };
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn default_is_empty() {
        let b: BoundingBox<f64, 3> = BoundingBox::default();
        for dim in b.dims {
            assert_eq!(dim.start, 0.0);
            assert_eq!(dim.end, 0.0);
        }
    }

    #[test]
    fn new_works() {
        let b = BoundingBox::new([Extent::from(0.0..=1.0), Extent::from(0.1..=2.0)]);
        assert_eq!(b.len(), 2);
        assert_eq!(b.dims[0], (0.0..=1.0).into());
        assert_eq!(b.dims[1], (0.1..=2.0).into());
    }

    #[test]
    fn new_from_ranges_works() {
        let a = BoundingBox::from([0.0..=1.0, 0.1..=2.0]);
        let b = BoundingBox::from(&[0.0..=1.0, 0.1..=2.0]);
        let c = BoundingBox::new([Extent::from(0.0..=1.0), Extent::from(0.1..=2.0)]);
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

    #[test]
    fn contains_works() {
        let a = BoundingBox::from([0.0..=1.0, 0.0..=1.0]);
        let b = BoundingBox::from([0.25..=0.75, 0.0..=1.0]);
        let c = BoundingBox::from([0.25..=0.75, 0.0..=1.5]);
        let d = BoundingBox::from([-1.0..=1.0, 0.0..=1.0]);
        assert!(a.contains(&b));
        assert!(!a.contains(&c));
        assert!(!a.contains(&d));
    }

    #[test]
    fn grow_works() {
        let a = BoundingBox::from([0.0..=1.0, 0.0..=1.0]);
        let b = BoundingBox::from([0.25..=0.75, 0.0..=1.0]);
        let c = BoundingBox::from([0.25..=0.75, 0.0..=1.5]);
        let d = BoundingBox::from([-1.0..=1.0, 0.0..=1.0]);

        let mut x = BoundingBox::default();
        x.grow(a);
        x.grow(b);
        x.grow(c);
        x.grow(d);

        assert_eq!(x.dims[0].start, -1.0);
        assert_eq!(x.dims[0].end, 1.0);
        assert_eq!(x.dims[1].start, 0.0);
        assert_eq!(x.dims[1].end, 1.5);
    }

    #[test]
    fn get_grown_works() {
        let a = BoundingBox::from([0.0..=1.0, 0.0..=1.0]);
        let b = BoundingBox::from([0.5..=1.5, 0.5..=1.5]);

        let x = a.get_grown(b);

        assert_eq!(x.bb.dims[0].start, 0.0);
        assert_eq!(x.bb.dims[0].end, 1.5);
        assert_eq!(x.bb.dims[1].start, 0.0);
        assert_eq!(x.bb.dims[1].end, 1.5);
        assert_eq!(x.area, 1.5 * 1.5);
        assert_eq!(x.area_increase, (1.5 * 1.5) - (1.0 * 1.0));
        assert_eq!(x.bb.area(), x.area);
    }
}
