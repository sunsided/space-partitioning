use num_traits::Num;

/// Alias trait for numerical types that can be used
/// to define values along a bounding box dimension.
pub trait DimensionType: Num + std::cmp::PartialOrd + Copy {}

impl<T> DimensionType for T where T: Num + std::cmp::PartialOrd + Copy {}
