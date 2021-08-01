//! Marker trait for Interval Types.

/// A marker trait for interval types. Default implemented for standard integral and floating-point types.
///
/// # Example
/// ```rust
/// use space_partitioning::interval_tree::IntervalType;
///
/// #[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
/// struct Vec2d {
///     pub x: f64,
///     pub y: f64,
/// }
///
/// impl IntervalType for Vec2d {}
/// ```
pub trait IntervalType: Clone + PartialOrd + PartialEq {}

impl IntervalType for i8 {}
impl IntervalType for u8 {}

impl IntervalType for i32 {}
impl IntervalType for u32 {}

impl IntervalType for usize {}
impl IntervalType for isize {}

impl IntervalType for f32 {}
impl IntervalType for f64 {}
