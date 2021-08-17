use crate::quadtree::node::NodeElementCountType;
use crate::quadtree::node_data::NodeData;
use crate::quadtree::AABB;

#[derive(Debug)]
pub struct NodeInfo {
    /// The node data.
    pub(crate) nd: NodeData,
    /// Gets the number of elements in this node.
    pub element_count: u32,
}

impl NodeInfo {
    #[inline]
    pub(crate) fn from(nd: NodeData, element_count: NodeElementCountType) -> Self {
        Self { nd, element_count }
    }

    /// Gets the depth of this node.
    pub fn depth(&self) -> u32 {
        self.nd.depth
    }

    pub fn get_aabb(&self) -> AABB {
        self.nd.crect.get_aabb()
    }
}
