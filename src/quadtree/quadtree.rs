use crate::quadtree::free_list;
use crate::quadtree::free_list::FreeList;
use smallvec::SmallVec;
use crate::quadtree::node_list::NodeList;
use crate::quadtree::node_data::{NodeData, NodeIndexType};
use crate::quadtree::node::Node;
use crate::quadtree::quad_rect::QuadRect;

/// Represents an element in the QuadTree.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct QuadTreeElement {
    /// Stores the ID for the element (can be used to refer to external data).
    pub id: u32,
    /// Left X coordinate of the rectangle of the element.
    pub x1: u32,
    /// Top Y coordinate of the rectangle of the element.
    pub y1: u32,
    /// Right X coordinate of the rectangle of the element.
    pub x2: u32,
    /// Bottom Y coordinate of the rectangle of the element.
    pub y2: u32
}

/// Represents an element node in the quadtree.
///
/// # Remarks
/// An element (`QuadTreeElement`) is only inserted once to the quadtree no matter how many
/// cells it occupies. However, for each cell it occupies, an "element node" (`QuadTreeElementNode`)
/// is inserted which indexes that element.
#[derive(Debug, PartialEq, Eq, Default)]
struct QuadTreeElementNode {
    /// Points to the next element in the leaf node. A value of `free_list::SENTINEL`
    /// indicates the end of the list.
    next: free_list::IndexType,
    /// Stores the element index.
    element: free_list::IndexType
}

pub struct QuadTree {
    /// Stores all the elements in the quadtree.
    /// An element is only inserted once to the quadtree no matter how many cells it occupies.
    elements: FreeList<QuadTreeElement>,
    /// Stores all the element nodes in the quadtree.
    /// For each cell occupied by a `QuadTreeElement`, we store
    /// a `QuadTreeElementNode`.
    element_nodes: FreeList<QuadTreeElementNode>,
    /// Stores all the nodes in the quadtree. The first node in this
    /// sequence is always the root.
    nodes: Vec<Node>,
    /// Stores the quadtree extents.
    root_rect: QuadRect,
    /// Stores the first free node in the quadtree to be reclaimed as 4
    /// contiguous nodes at once. A value of `free_list::SENTINEL` indicates that the free
    /// list is empty, at which point we simply insert 4 nodes to the
    /// back of the nodes array.
    free_node: free_list::IndexType,
    /// Stores the maximum depth allowed for the quadtree.
    max_depth: u32
}

impl Default for QuadTree {
    fn default() -> Self {
        Self::new(8)
    }
}

impl QuadTree {
    pub fn new(max_depth: u32) -> Self {
        Self {
            elements: FreeList::default(),
            element_nodes: FreeList::default(),
            nodes: Vec::default(),
            root_rect: QuadRect::default(),
            free_node: free_list::SENTINEL,
            max_depth
        }
    }

    fn insert(&mut self, element: QuadTreeElement) {
        // let element_index = self.elements.insert(element);
        // self.element_nodes.insert(QuadTreeElementNode { element: ei, next: ?? })

        if self.free_node == free_list::SENTINEL {
            // TODO: push back to node array
        }
        else {
            // TODO: overwrite node at free head
            // TODO: set free head to node[free_head]
        }
    }

    fn remove(&mut self, node: NodeData) {
        // TODO: set removed node to free_head
        // TODO: Set free_head to removed node index
    }

    fn find_leaves(&self, rect: [i32; 4]) -> NodeList {
        let root_data = NodeData::new_from_root(&self.root_rect);
        self.find_leaves_from_root(root_data, rect)
    }

    fn find_leaves_from_root(&self, root: NodeData, rect: [i32; 4]) -> NodeList {
        let mut leaves = NodeList::default();
        let mut to_process = NodeList::default();
        to_process.push_back(root);

        while to_process.len() > 0 {
            let nd = to_process.pop_back();

            // If this node is a leaf, insert it to the list.
            if self.nodes[nd.index as usize].is_leaf() {
                leaves.push_back(nd);
                continue;
            }

            // Otherwise push the children that intersect the rectangle.
            let mx = nd.crect[0];
            let my = nd.crect[1];
            let hx = nd.crect[2] >> 1;
            let hy = nd.crect[3] >> 1;
            let fc = self.nodes[nd.index as usize].get_first_child_node_index();
            let l = mx - hx;
            let t = my - hy;
            let r = mx + hx;
            let b = my + hy;

            if rect[1] <= my {
                if rect[0] <= mx {
                    to_process.push_back(NodeData::new(l, t, hx, hy, fc + 0, nd.depth + 1));
                } else {
                    to_process.push_back(NodeData::new(r, t, hx, hy, fc + 1, nd.depth + 1));
                }
            } else {
                if rect[0] <= mx {
                    to_process.push_back(NodeData::new(l, b, hx, hy, fc + 2, nd.depth + 1));
                } else {
                    to_process.push_back(NodeData::new(r, b, hx, hy, fc + 3, nd.depth + 1));
                }
            }
        }

        leaves
    }

    pub fn cleanup(&mut self) {
        // Only process the root if it is not a leaf.
        if self.nodes[0].is_leaf() {
            return;
        }

        // Initialize the stack of nodes to be processed with the index of the root node.
        // TODO: revisit the small list size, check element count
        let mut to_process: SmallVec<[NodeIndexType; 128]> = smallvec::smallvec![0];

        while !to_process.is_empty() {
            let node_index = to_process.pop().unwrap();
            let first_child_index = self.nodes[node_index as usize].get_first_child_node_index();

            // Loop through the children.
            let mut num_empty_leaves = 0usize;
            for j in 0..4 {
                let child_index = first_child_index + j;
                let child = &self.nodes[child_index as usize];

                // Increment empty leaf count if the child is an empty
                // leaf. Otherwise if the child is a branch, add it to
                // the stack to be processed in the next iteration.
                if child.is_empty() {
                    num_empty_leaves += 1; // TODO: Reverse, compare to zero
                } else if child.is_branch() {
                    to_process.push(child_index);
                }
            }

            // If all the children were empty leaves, remove them and
            // make this node the new empty leaf.
            if num_empty_leaves == 4 { // TODO: Reverse, compare to zero
                // Push all 4 children to the free list.
                // (We don't change the indexes of the 2nd to 4th child because
                // child nodes are always processed together.)
                self.nodes[first_child_index as usize].first_child = self.free_node;
                self.free_node = first_child_index;

                // Make this node the new empty leaf.
                let node = &mut self.nodes[node_index as usize];
                node.make_empty_leaf();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_works() {
        let tree = QuadTree::default();
    }
}
