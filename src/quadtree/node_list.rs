use crate::quadtree::node_data::NodeData;

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

    pub fn pop_back(&mut self) -> NodeData {
        debug_assert!(!self.elements.is_empty());
        self.elements.pop().unwrap()
    }
}

impl IntoIterator for NodeList {
    type Item = NodeData;
    type IntoIter = std::vec::IntoIter<NodeData>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}
