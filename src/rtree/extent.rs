use crate::rtree::dimension_type::DimensionType;
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Range, RangeInclusive};

/// Extents along a dimension.
#[derive(Copy, Clone, PartialEq)]
pub struct Extent<T>
where
    T: DimensionType,
{
    /// The start coordinate of the extent.
    pub start: T,
    /// The end coordinate of the extent.
    pub end: T,
}

impl<T> Extent<T>
where
    T: DimensionType,
{
    /// Initializes a new box from coordinates.
    ///
    /// ## Arguments
    /// * `start` - The start coordinate along the dimension.
    /// * `end` - The end coordinate along the dimension.
    pub fn new(start: T, end: T) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }

    /// Initializes a new box from coordinates.
    ///
    /// ## Arguments
    /// * `start` - The start coordinate along the dimension.
    /// * `end` - The end coordinate along the dimension.
    pub fn new_from_range<R: Borrow<RangeInclusive<T>>>(range: R) -> Self {
        let range = range.borrow();
        debug_assert!(range.start() <= range.end());
        Self {
            start: range.start().clone(),
            end: range.end().clone(),
        }
    }
}

impl<T> Default for Extent<T>
where
    T: DimensionType,
{
    fn default() -> Self {
        Self {
            start: T::zero(),
            end: T::one(),
        }
    }
}

impl<T, R: Borrow<RangeInclusive<T>>> From<R> for Extent<T>
where
    T: DimensionType,
{
    fn from(range: R) -> Self {
        Self::new_from_range(range)
    }
}

impl<T> Into<RangeInclusive<T>> for Extent<T>
where
    T: DimensionType,
{
    fn into(self) -> RangeInclusive<T> {
        self.start..=self.end
    }
}

pub trait Contains<T> {
    fn contains(self, value: T) -> bool;
}

impl<T> Contains<T> for Extent<T>
where
    T: DimensionType,
{
    fn contains(self, value: T) -> bool {
        self.start <= value && value <= self.end
    }
}

impl<T> Contains<Extent<T>> for Extent<T>
where
    T: DimensionType,
{
    fn contains(self, value: Extent<T>) -> bool {
        // Since we require that start <= end, we only need to test the boundaries.
        self.start <= value.start && value.end <= self.end
    }
}

impl<T> Contains<Range<T>> for Extent<T>
where
    T: DimensionType,
{
    fn contains(self, value: Range<T>) -> bool {
        self.start <= value.start
            && value.start <= self.end
            && self.start <= value.end
            && value.end <= self.end
    }
}

impl<T> Contains<RangeInclusive<T>> for Extent<T>
where
    T: DimensionType,
{
    fn contains(self, value: RangeInclusive<T>) -> bool {
        let start = value.start();
        let end = value.end();
        &self.start <= start && start <= &self.end && &self.start <= end && end <= &self.end
    }
}

impl<T> Debug for Extent<T>
where
    T: DimensionType + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

impl<T> Display for Extent<T>
where
    T: DimensionType + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn new_works() {
        let e = Extent::new(1.0, 2.0);
        assert_eq!(e.start, 1.0);
        assert_eq!(e.end, 2.0);
    }

    #[test]
    fn default_works() {
        let e: Extent<f64> = Extent::default();
        assert_eq!(e.start, 0.0);
        assert_eq!(e.end, 1.0);
    }

    #[test]
    fn new_from_range_works() {
        let e = Extent::new_from_range(2.0..=5.0);
        assert_eq!(e.start, 2.0);
        assert_eq!(e.end, 5.0);
    }

    #[test]
    fn from_range_works() {
        let e = Extent::from(2.0..=5.0);
        assert_eq!(e.start, 2.0);
        assert_eq!(e.end, 5.0);
    }

    #[test]
    fn from_range_ref_works() {
        let r = 2.0..=5.0;
        let e = Extent::from(&r);
        assert_eq!(e.start, 2.0);
        assert_eq!(e.end, 5.0);
    }

    #[test]
    fn into_range_works() {
        let e = Extent::from(2.0..=5.0);
        let r: RangeInclusive<f64> = e.into();
        assert_eq!(r.start(), &2.0);
        assert_eq!(r.end(), &5.0);
    }

    #[test]
    fn extent_contains_start() {
        let e = Extent::from(2.0..=5.0);
        assert!(e.contains(e.start));
    }

    #[test]
    fn extent_contains_end() {
        let e = Extent::from(2.0..=5.0);
        assert!(e.contains(e.end));
    }

    #[test]
    fn contains_value_works() {
        let e = Extent::from(2.0..=5.0);
        assert!(e.contains(2.5));
        assert!(!e.contains(0.0));
        assert!(!e.contains(5.1));
    }

    #[test]
    fn extent_contains_itself() {
        let e = Extent::from(2.0..=5.0);
        assert!(e.contains(e));
    }

    #[test]
    fn extent_contains_itself_as_range() {
        let e = Extent::from(2.0..=5.0);
        assert!(e.contains(2.0..5.0));
    }

    #[test]
    fn extent_contains_smaller_extent() {
        let e = Extent::from(2.0..=5.0);
        assert!(e.contains(Extent::from(2.1..=4.9)));
    }

    #[test]
    fn extent_contains_smaller_range() {
        let e = Extent::from(2.0..=5.0);
        assert!(e.contains(2.1..=4.9));
    }

    #[test]
    fn extent_does_not_contain_overlaps() {
        let e = Extent::from(2.0..=5.0);
        assert!(!e.contains(Extent::from(2.1..=5.1)));
        assert!(!e.contains(Extent::from(1.9..=4.9)));
        assert!(!e.contains(Extent::from(1.9..=5.1)));
    }

    #[test]
    fn extent_does_not_contain_overlapping_rangers() {
        let e = Extent::from(2.0..=5.0);
        assert!(!e.contains(2.1..=5.1));
        assert!(!e.contains(1.9..=4.9));
        assert!(!e.contains(1.9..=5.1));
    }

    #[test]
    fn display_works() {
        assert_eq!(format!("{}", Extent::from(0.0..=1.2)), "0..1.2");
        assert_eq!(format!("{}", Extent::from(0..=12)), "0..12");
    }

    #[test]
    fn debug_works() {
        assert_eq!(format!("{:?}", Extent::from(0.0..=1.2)), "0.0..1.2");
        assert_eq!(format!("{:?}", Extent::from(0..=12)), "0..12");
    }
}
