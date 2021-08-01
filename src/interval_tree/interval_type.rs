pub trait IntervalType: Clone + PartialOrd {}

impl IntervalType for i8 {}
impl IntervalType for u8 {}

impl IntervalType for i32 {}
impl IntervalType for u32 {}

impl IntervalType for usize {}
impl IntervalType for isize {}

impl IntervalType for f32 {}
impl IntervalType for f64 {}
