use std::ops::Index;

pub struct Quadrants {
    quadrants: [bool; 5],
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
        Quadrants {
            quadrants: [
                covers_many,
                explore_top & explore_left,
                explore_top & explore_right,
                explore_bottom & explore_left,
                explore_bottom & explore_right,
            ],
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
        let covers_many = top_left & top_right
            | top_left & bottom_left
            | top_right & bottom_right
            | top_left & bottom_right;
        Quadrants {
            quadrants: [covers_many, top_left, top_right, bottom_left, bottom_right],
        }
    }

    #[inline]
    pub fn self_only() -> Self {
        Self {
            quadrants: [true, false, false, false, false],
        }
    }

    #[inline]
    pub fn all() -> Self {
        Self {
            quadrants: [true; 5],
        }
    }

    #[inline]
    pub fn this(&self) -> bool {
        self.quadrants[0]
    }

    #[inline]
    pub fn top_left(&self) -> bool {
        self.quadrants[1]
    }

    #[inline]
    pub fn top_right(&self) -> bool {
        self.quadrants[2]
    }

    #[inline]
    pub fn bottom_left(&self) -> bool {
        self.quadrants[3]
    }

    #[inline]
    pub fn bottom_right(&self) -> bool {
        self.quadrants[4]
    }
}

impl Index<usize> for Quadrants {
    type Output = bool;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.quadrants[index]
    }
}

impl Index<u32> for Quadrants {
    type Output = bool;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        &self.quadrants[index as usize]
    }
}
