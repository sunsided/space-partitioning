///! `Interval<T>` for capturing intervals.
pub use crate::interval_tree::interval_type::IntervalType;
use std::fmt::{Debug, Display, Formatter};
use std::ops::RangeInclusive;

/// Structure to represent an interval.
#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Interval<T>
where
    T: IntervalType,
{
    pub start: T,
    pub end: T,
}

impl<T> Interval<T>
where
    T: IntervalType,
{
    /// Constructs a new interval.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::interval_tree::Interval;
    /// let interval = Interval::new(-2.0, 10.0);
    /// assert_eq!(interval.start, -2.0);
    /// assert_eq!(interval.end, 10.0);
    /// ```
    pub fn new(low: T, high: T) -> Self {
        Self {
            start: low,
            end: high,
        }
    }

    /// Checks whether the current interval overlaps with another one.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::interval_tree::Interval;
    /// let interval = Interval::from(-2.0..=10.0);
    /// assert!(interval.overlaps_with(&(0.0..=2.0).into()));
    /// assert!(!interval.overlaps_with(&(20.0..=30.0).into()));
    /// ```
    pub fn overlaps_with(&self, other: &Interval<T>) -> bool {
        (self.start <= other.end) && (other.start <= self.end)
    }
}

impl<T> Debug for Interval<T>
where
    T: Debug + IntervalType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}, {:?}]", self.start, self.end)
    }
}

impl<T> Display for Interval<T>
where
    T: Display + IntervalType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.start, self.end)
    }
}

impl<T> From<(T, T)> for Interval<T>
where
    T: IntervalType,
{
    /// Constructs an interval from a tuple.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::interval_tree::Interval;
    /// let interval: Interval<_> = (-2.0, 10.0).into();
    /// assert_eq!(interval.start, -2.0);
    /// assert_eq!(interval.end, 10.0);
    /// assert_eq!(interval, Interval::from((-2.0, 10.0)));
    /// ```
    fn from(interval: (T, T)) -> Self {
        Self {
            start: interval.0,
            end: interval.1,
        }
    }
}

impl<T> From<std::ops::RangeInclusive<T>> for Interval<T>
where
    T: IntervalType,
{
    /// Constructs an interval from a `RangeInclusive<T>``.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::interval_tree::Interval;
    /// let interval: Interval<_> = (-2.0..=10.0).into();
    /// assert_eq!(interval.start, -2.0);
    /// assert_eq!(interval.end, 10.0);
    /// assert_eq!(interval, Interval::from(-2.0..=10.0));
    /// ```
    fn from(range: RangeInclusive<T>) -> Self {
        Self {
            start: range.start().clone(),
            end: range.end().clone(),
        }
    }
}

impl<T> From<&std::ops::RangeInclusive<T>> for Interval<T>
where
    T: IntervalType,
{
    /// Constructs an interval from a `&RangeInclusive<T>``.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::interval_tree::Interval;
    /// let range = -2.0..=10.0;
    /// let interval: Interval<_> = (&range).into();
    /// assert_eq!(interval.start, -2.0);
    /// assert_eq!(interval.end, 10.0);
    /// ```
    fn from(range: &RangeInclusive<T>) -> Self {
        Self {
            start: range.start().clone(),
            end: range.end().clone(),
        }
    }
}
