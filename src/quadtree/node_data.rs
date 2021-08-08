use crate::quadtree::quad_rect::QuadRect;

pub type NodeIndexType = u32;

pub struct NodeData {
    pub index: NodeIndexType,
    pub crect: [i32; 4],
    pub depth: u32,
}

impl NodeData {
    pub fn new(l: i32, t: i32, hx: i32, hy: i32, index: u32, depth: u32) -> Self {
        Self {
            index,
            crect: [l, t, hx, hy],
            depth
        }
    }

    pub fn new_from_root(root_rect: &QuadRect) -> Self {
        Self {
            index: 0,
            crect: [root_rect.l, root_rect.t, root_rect.hx, root_rect.hy],
            depth: 0
        }
    }
}