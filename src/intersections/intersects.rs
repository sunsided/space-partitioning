pub trait Intersects<T = Self> {
    /// Tests whether this element intersects with the [`other`].
    ///
    /// # Returns
    /// - `true` if the two elements intersect.
    fn intersects_with(&self, other: &T) -> bool;
}
