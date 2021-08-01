use crate::interval_tree::{Interval, IntervalType};
use std::fmt::{Debug, Formatter};

pub struct IntervalTreeEntry<T, D>
where
    T: IntervalType,
{
    pub interval: Interval<T>,
    pub data: D,
}

impl<T, D> Debug for IntervalTreeEntry<T, D>
where
    T: Debug + IntervalType,
    D: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} data = {:?}", self.interval, self.data)
    }
}

impl<T, D> IntervalTreeEntry<T, D>
where
    T: IntervalType,
{
    pub fn new<I>(interval: I, data: D) -> Self
    where
        I: Into<Interval<T>>,
    {
        Self {
            interval: interval.into(),
            data,
        }
    }
}

impl<I, T, D> From<(I, D)> for IntervalTreeEntry<T, D>
where
    I: Into<Interval<T>>,
    T: IntervalType,
{
    fn from(pair: (I, D)) -> Self {
        Self {
            interval: pair.0.into(),
            data: pair.1,
        }
    }
}

impl<I, T> From<I> for IntervalTreeEntry<T, ()>
where
    I: Into<Interval<T>>,
    T: IntervalType,
{
    fn from(value: I) -> Self {
        Self {
            interval: value.into(),
            data: (),
        }
    }
}
