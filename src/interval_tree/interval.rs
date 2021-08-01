pub use crate::interval_tree::interval_type::IntervalType;
use std::fmt::{Debug, Display, Formatter};
use std::ops::RangeInclusive;

/// Structure to represent an interval.
#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Interval<T>
where
    T: IntervalType,
{
    pub low: T,
    pub high: T,
}

impl<T> Interval<T>
where
    T: IntervalType,
{
    pub fn new(low: T, high: T) -> Self {
        Self { low, high }
    }
}

impl<T> Debug for Interval<T>
where
    T: Debug + IntervalType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}, {:?}]", self.low, self.high)
    }
}

impl<T> Display for Interval<T>
where
    T: Display + IntervalType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.low, self.high)
    }
}

impl<T> From<(T, T)> for Interval<T>
where
    T: IntervalType,
{
    fn from(interval: (T, T)) -> Self {
        Self {
            low: interval.0,
            high: interval.1,
        }
    }
}

impl<T> From<std::ops::RangeInclusive<T>> for Interval<T>
where
    T: IntervalType,
{
    fn from(range: RangeInclusive<T>) -> Self {
        Self {
            low: range.start().clone(),
            high: range.end().clone(),
        }
    }
}

impl<T> From<&std::ops::RangeInclusive<T>> for Interval<T>
where
    T: IntervalType,
{
    fn from(range: &RangeInclusive<T>) -> Self {
        Self {
            low: range.start().clone(),
            high: range.end().clone(),
        }
    }
}

impl<T> Interval<T>
where
    T: IntervalType,
{
    /// A utility function to check if given two intervals overlap.
    pub fn overlaps_with(&self, other: &Interval<T>) -> bool {
        (self.low <= other.high) && (other.low <= self.high)
    }
}
