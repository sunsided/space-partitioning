use crate::quadtree::quad_rect::QuadRect;

pub type NodeIndexType = u32;

pub struct NodeData {
    /// The index of the `Node` described by this `NodeData` instance.
    pub index: NodeIndexType,
    /// The rectangle of the node, center x, center y, width and height.
    pub crect: [i32; 4],
    /// The depth of the node.
    pub depth: u32,
}

impl NodeData {
    pub fn new(l: i32, t: i32, hx: i32, hy: i32, index: u32, depth: u32) -> Self {
        let mx = l + (hx >> 1);
        let my = t + (hy >> 1);
        Self {
            index,
            crect: [mx, my, hx, hy],
            depth,
        }
    }

    pub fn new_from_root(root_rect: &QuadRect) -> Self {
        Self::new(root_rect.l, root_rect.t, root_rect.hx, root_rect.hy, 0, 0)
    }

    /// Determines if a node is at least `smallest_size` in width or height,
    /// guaranteeing that after subdivision, each cell would be of a usable size.
    /// This is mostly relevant with integral sizes.
    pub fn can_split(&self, smallest_size: i32) -> bool {
        let can_split_width = self.crect[2] >= (smallest_size * 2);
        let can_split_height = self.crect[3] >= (smallest_size * 2);
        can_split_width || can_split_height
    }
}
