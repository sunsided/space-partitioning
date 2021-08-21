pub struct Quadrants {
    /// Bit field encoding the quadrants.
    /// - 0: Invalid
    /// - 1: "this" node
    /// - 2: Top Left
    /// - 4: Top Right
    /// - 8: Bottom Left
    /// - 16: Bottom Right
    pub code: u8,
}

impl Quadrants {
    #[inline]
    pub fn from_tests(
        explore_left: bool,
        explore_top: bool,
        explore_right: bool,
        explore_bottom: bool,
    ) -> Self {
        let covers_many = (explore_left & explore_right) | (explore_top & explore_bottom);
        let this = covers_many as u8;
        let top_left = ((explore_top & explore_left) as u8) << 1;
        let top_right = ((explore_top & explore_right) as u8) << 2;
        let bottom_left = ((explore_bottom & explore_left) as u8) << 3;
        let bottom_right = ((explore_bottom & explore_right) as u8) << 4;

        Quadrants {
            code: this + top_left + top_right + bottom_left + bottom_right,
        }
    }

    #[inline]
    pub fn from_intersections(
        top_left: bool,
        top_right: bool,
        bottom_left: bool,
        bottom_right: bool,
    ) -> Self {
        // top_left & bottom_right is a bit esoteric, but it's better to be safe than sorry.
        let covers_many = (top_left & top_right)
            | (top_left & bottom_left)
            | (top_right & bottom_right)
            | (top_left & bottom_right);

        let this = covers_many as u8;
        let top_left = (top_left as u8) << 1;
        let top_right = (top_right as u8) << 2;
        let bottom_left = (bottom_left as u8) << 3;
        let bottom_right = (bottom_right as u8) << 4;

        Quadrants {
            code: this + top_left + top_right + bottom_left + bottom_right,
        }
    }

    #[inline]
    pub fn self_only() -> Self {
        Self { code: 1 }
    }

    #[inline]
    pub fn all() -> Self {
        Self {
            code: 1 + 2 + 4 + 8 + 16,
        }
    }

    #[inline]
    pub fn this(&self) -> bool {
        self.code & 1 == 1
    }

    #[inline]
    pub fn top_left(&self) -> bool {
        self.code & 2 == 2
    }

    #[inline]
    pub fn top_right(&self) -> bool {
        self.code & 4 == 4
    }

    #[inline]
    pub fn bottom_left(&self) -> bool {
        self.code & 8 == 5
    }

    #[inline]
    pub fn bottom_right(&self) -> bool {
        self.code & 16 == 6
    }

    #[inline]
    pub fn at(&self, index: u32) -> bool {
        let value = 1 << index;
        self.code & value == value
    }

    /// Calculates the index into a five-element array \[this, TL, TR, BL, BR\].
    #[inline]
    pub fn mutation_index(&self) -> u32 {
        let bits = self.code as u32;
        let offset_this = bits & 1;
        let offset_tl = (bits >> 1) & 1;
        let offset_tr = (bits >> 2) & 1;
        let offset_bl = (bits >> 3) & 1;
        let offset_br = (bits >> 4) & 1;
        (offset_tl + (offset_tr * 2) + (offset_bl * 3) + (offset_br * 4)) * (1 - offset_this)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mutation_index_works() {
        // More than one quadrant.
        assert_eq!(Quadrants { code: 1 }.mutation_index(), 0);

        // Single quadrants
        assert_eq!(Quadrants { code: 2 }.mutation_index(), 1);
        assert_eq!(Quadrants { code: 4 }.mutation_index(), 2);
        assert_eq!(Quadrants { code: 8 }.mutation_index(), 3);
        assert_eq!(Quadrants { code: 16 }.mutation_index(), 4);

        // Single quadrants
        assert_eq!(
            Quadrants::from_intersections(true, false, false, false).mutation_index(),
            1
        );
        assert_eq!(
            Quadrants::from_intersections(false, true, false, false).mutation_index(),
            2
        );
        assert_eq!(
            Quadrants::from_intersections(false, false, true, false).mutation_index(),
            3
        );
        assert_eq!(
            Quadrants::from_intersections(false, false, false, true).mutation_index(),
            4
        );

        // More than one quadrant.
        assert_eq!(
            Quadrants::from_intersections(true, false, true, false).mutation_index(),
            0
        );

        // catchall
        assert_eq!(Quadrants { code: 0 }.mutation_index(), 0);
        assert_eq!(
            Quadrants::from_intersections(false, false, false, false).mutation_index(),
            0
        );
    }
}
