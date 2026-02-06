use crate::octree::node::NodeElementCountType;
use crate::octree::node_data::NodeData;
use crate::octree::AABB;

#[derive(Debug)]
pub struct NodeInfo {
    pub(crate) nd: NodeData,
    pub element_count: u32,
}

impl NodeInfo {
    #[inline]
    pub(crate) fn from(nd: NodeData, element_count: NodeElementCountType) -> Self {
        Self { nd, element_count }
    }

    pub fn depth(&self) -> u8 {
        self.nd.depth
    }

    #[inline]
    pub fn get_aabb(&self) -> AABB {
        self.nd.crect.get_aabb()
    }
}
