use std::fmt::{Debug, Display, Formatter};

/// Structure to represent an interval.
#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Interval<T> {
    pub low: T,
    pub high: T,
}

impl<T> Interval<T> {
    pub fn new(low: T, high: T) -> Self {
        Self { low, high }
    }
}

impl<T: Debug> Debug for Interval<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}, {:?}]", self.low, self.high)
    }
}

impl<T: Display> Display for Interval<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.low, self.high)
    }
}

impl<T> From<(T, T)> for Interval<T> {
    fn from(interval: (T, T)) -> Self {
        Self {
            low: interval.0,
            high: interval.1,
        }
    }
}

impl<T: Copy> From<std::ops::RangeInclusive<T>> for Interval<T> {
    fn from(range: std::ops::RangeInclusive<T>) -> Self {
        Self {
            low: *range.start(),
            high: *range.end(),
        }
    }
}

impl<T: Clone> From<&std::ops::RangeInclusive<T>> for Interval<T> {
    fn from(range: &std::ops::RangeInclusive<T>) -> Self {
        Self {
            low: range.start().clone(),
            high: range.end().clone(),
        }
    }
}

impl<T: PartialOrd> Interval<T> {
    /// A utility function to check if given two intervals overlap.
    pub fn overlaps_with(&self, other: &Interval<T>) -> bool {
        (self.low <= other.high) && (other.low <= self.high)
    }
}
