use num_traits::Num;
use std::cmp::PartialOrd;

/// Alias trait for numerical types that can be used
/// to define values along a bounding box dimension.
pub trait DimensionType: Num + PartialOrd + Copy {}

impl<T> DimensionType for T where T: Num + PartialOrd + Copy {}
