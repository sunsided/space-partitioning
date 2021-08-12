pub trait Intersects<T = Self> {
    fn intersects(&self, other: &T) -> bool;
}
