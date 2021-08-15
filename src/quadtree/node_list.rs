use crate::quadtree::node_data::NodeData;
use std::ops::Index;

#[derive(Default)]
pub struct NodeList {
    elements: Vec<NodeData>,
}

impl NodeList {
    pub fn push_back(&mut self, nd: NodeData) {
        self.elements.push(nd)
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn pop_back(&mut self) -> NodeData {
        debug_assert!(!self.elements.is_empty());
        self.elements.pop().unwrap()
    }
}

impl Index<usize> for NodeList {
    type Output = NodeData;

    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index]
    }
}

impl IntoIterator for NodeList {
    type Item = NodeData;
    type IntoIter = std::vec::IntoIter<NodeData>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}
