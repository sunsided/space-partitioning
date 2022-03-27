use num_traits::{Bounded, Num};
use std::cmp::PartialOrd;
use std::fmt::Debug;

/// Alias trait for numerical types that can be used
/// to define values along a bounding box dimension.
pub trait DimensionType: Num + Bounded + PartialOrd + Copy + Debug {}

impl<T> DimensionType for T where T: Num + Bounded + PartialOrd + Copy + Debug {}
