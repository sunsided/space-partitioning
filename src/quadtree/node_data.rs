use crate::quadtree::centered_aabb::CenteredAABB;
use crate::quadtree::quad_rect::QuadRect;

pub type NodeIndexType = u32;

#[derive(Debug)]
pub struct NodeData {
    /// The index of the `Node` described by this `NodeData` instance.
    pub index: NodeIndexType,
    /// The centered AABB of the the node: center x, center y, width and height.
    pub crect: CenteredAABB,
    /// The depth of the node.
    pub depth: u32,
}

impl NodeData {
    #[inline]
    pub fn new(l: i32, t: i32, hx: i32, hy: i32, index: u32, depth: u32) -> Self {
        Self::new_from_centered_aabb(index, depth, CenteredAABB::from_ltwh(l, t, hx, hy))
    }

    #[inline]
    pub fn new_from_root(root_rect: &QuadRect) -> Self {
        Self::new_from_centered_aabb(0, 0, root_rect.into())
    }

    #[inline]
    fn new_from_centered_aabb(index: u32, depth: u32, crect: CenteredAABB) -> Self {
        Self {
            index,
            crect,
            depth,
        }
    }

    /// Determines if a node is at least `smallest_size` in width or height,
    /// guaranteeing that after subdivision, each cell would be of a usable size.
    /// This is mostly relevant with integral sizes.
    ///
    /// Additionally, ensures that the node has not reached its maximum depth.
    pub fn can_split_further(&self, smallest_size: i32, max_depth: u32) -> bool {
        let split_allowed = self.can_subdivide(smallest_size);
        let can_go_deeper = self.depth < max_depth;
        split_allowed && can_go_deeper
    }

    /// Determines if a node is at least `smallest_size` in width or height,
    /// guaranteeing that after subdivision, each cell would be of a usable size.
    /// This is mostly relevant with integral sizes.
    fn can_subdivide(&self, smallest_size: i32) -> bool {
        let can_split_width = self.crect.width >= (smallest_size * 2);
        let can_split_height = self.crect.height >= (smallest_size * 2);
        can_split_width || can_split_height
    }
}
