pub struct Octants {
    /// Bit field encoding the octants.
    /// - 0: Invalid
    /// - 1: "this" node (covers multiple octants)
    /// - 1<<1..=1<<8: the eight spatial octants
    pub code: u16,
}

impl Octants {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn from_tests(
        explore_left: bool,
        explore_top: bool,
        explore_right: bool,
        explore_bottom: bool,
        explore_front: bool,
        explore_back: bool,
    ) -> Self {
        let covers_many = (explore_left & explore_right)
            | (explore_top & explore_bottom)
            | (explore_front & explore_back);

        let ltf = ((explore_left & explore_top & explore_front) as u16) << 1;
        let rtf = ((explore_right & explore_top & explore_front) as u16) << 2;
        let lbf = ((explore_left & explore_bottom & explore_front) as u16) << 3;
        let rbf = ((explore_right & explore_bottom & explore_front) as u16) << 4;
        let ltb = ((explore_left & explore_top & explore_back) as u16) << 5;
        let rtb = ((explore_right & explore_top & explore_back) as u16) << 6;
        let lbb = ((explore_left & explore_bottom & explore_back) as u16) << 7;
        let rbb = ((explore_right & explore_bottom & explore_back) as u16) << 8;

        Octants {
            code: (covers_many as u16) + ltf + rtf + lbf + rbf + ltb + rtb + lbb + rbb,
        }
    }

    #[inline]
    pub fn from_intersections(
        ltf: bool,
        rtf: bool,
        lbf: bool,
        rbf: bool,
        ltb: bool,
        rtb: bool,
        lbb: bool,
        rbb: bool,
    ) -> Self {
        let hits = ltf as u16
            + rtf as u16
            + lbf as u16
            + rbf as u16
            + ltb as u16
            + rtb as u16
            + lbb as u16
            + rbb as u16;
        let covers_many = hits > 1;
        let this = covers_many as u16;

        Octants {
            code: this
                + ((ltf as u16) << 1)
                + ((rtf as u16) << 2)
                + ((lbf as u16) << 3)
                + ((rbf as u16) << 4)
                + ((ltb as u16) << 5)
                + ((rtb as u16) << 6)
                + ((lbb as u16) << 7)
                + ((rbb as u16) << 8),
        }
    }

    #[inline]
    pub fn self_only() -> Self {
        Self { code: 1 }
    }

    #[inline]
    pub fn all() -> Self {
        Self {
            code: 1 + 2 + 4 + 8 + 16 + 32 + 64 + 128 + 256,
        }
    }

    #[inline]
    pub fn this(&self) -> bool {
        self.code & 1 == 1
    }

    #[inline]
    pub fn at(&self, index: u32) -> bool {
        let value = 1u16 << index;
        self.code & value == value
    }

    /// Calculates the index into a nine-element array [this, LTF, RTF, LBF, RBF, LTB, RTB, LBB, RBB].
    #[inline]
    pub fn mutation_index(&self) -> u32 {
        let octant_bits = self.code & !1;
        if octant_bits.count_ones() != 1 {
            return 0;
        }
        octant_bits.trailing_zeros()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mutation_index_matches_octant() {
        for i in 1..=8 {
            let code = 1u16 << i;
            let oct = Octants { code };
            assert_eq!(oct.mutation_index(), i as u32);
        }
    }

    #[test]
    fn mutation_index_returns_zero_for_multi_hits() {
        let oct = Octants {
            code: (1 << 2) + (1 << 5),
        };
        assert_eq!(oct.mutation_index(), 0);
    }

    #[test]
    fn self_bit_for_multi_axis_spans() {
        let oct = Octants::from_tests(true, true, true, false, false, false);
        assert!(oct.this());
        assert_eq!(oct.mutation_index(), 0);
    }
}
