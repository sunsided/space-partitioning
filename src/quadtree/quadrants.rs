pub struct Quadrants {
    pub this: bool,
    pub top_left: bool,
    pub top_right: bool,
    pub bottom_left: bool,
    pub bottom_right: bool,
}

impl Quadrants {
    pub fn from_tests(
        explore_left: bool,
        explore_top: bool,
        explore_right: bool,
        explore_bottom: bool,
    ) -> Self {
        let covers_many = (explore_left & explore_right) | (explore_top & explore_bottom);
        Quadrants {
            this: covers_many,
            top_left: explore_top & explore_left,
            top_right: explore_top & explore_right,
            bottom_left: explore_bottom & explore_left,
            bottom_right: explore_bottom & explore_right,
        }
    }

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
            this: covers_many,
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    pub fn self_only() -> Self {
        Self {
            this: true,
            top_left: false,
            top_right: false,
            bottom_left: false,
            bottom_right: false,
        }
    }

    pub fn all() -> Self {
        Self {
            this: true,
            top_left: true,
            top_right: true,
            bottom_left: true,
            bottom_right: true,
        }
    }
}
