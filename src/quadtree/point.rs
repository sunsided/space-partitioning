#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn point_is_8_bytes() {
        assert_eq!(std::mem::size_of::<Point>(), 8);
    }
}
