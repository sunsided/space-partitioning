use crate::quadtree::node_data::NodeData;
use smallvec::SmallVec;
use std::ops::Index;

#[derive(Default)]
pub struct NodeList {
    elements: SmallVec<[NodeData; 64]>,
}

impl NodeList {
    #[inline]
    pub fn push_back(&mut self, nd: NodeData) {
        self.elements.push(nd)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    #[inline]
    pub fn pop_back(&mut self) -> NodeData {
        debug_assert!(!self.elements.is_empty());
        self.elements.pop().unwrap()
    }
}

impl Index<usize> for NodeList {
    type Output = NodeData;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index]
    }
}
