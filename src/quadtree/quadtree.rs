use crate::quadtree::free_list;
use crate::quadtree::free_list::FreeList;
use smallvec::SmallVec;

type NodeIndexType = u32;

/// This value encodes that a node is a branch, i.e., not a leaf.
const NODE_INDEX_IS_BRANCH: u32 = NodeIndexType::MAX;

/// Represents a node in the quadtree.
struct QuadNode {
    /// Points to the first child if this node is a branch or the first
    /// element if this node is a leaf.
    first_child: free_list::IndexType,

    /// Stores the number of elements in the leaf or `NODE_INDEX_IS_BRANCH if it this node is
    /// a branch (i.e., not a leaf).
    count: NodeIndexType,
}

struct QuadNodeData {
    index: u32,
    crect: [i32; 4],
    depth: u32,
}

/// A rectangle describing the extents of a QudTree cell.
///
/// # Remarks
/// Only the tree node stores its extents. Bounding boxes for sub-nodes are computed on the fly.
#[derive(Default, Debug)]
struct QuadRect {
    // TODO: Might want to use a centered AABB instead, storing center and half-width/height?
    l: i32,
    t: i32,
    hx: i32,
    hy: i32
}

pub struct QuadTree {
    /// Stores all the elements in the quadtree.
    elements: FreeList<QuadTreeElement>,
    /// Stores all the element nodes in the quadtree.
    element_nodes: FreeList<QuadTreeElementNode>,
    /// Stores all the nodes in the quadtree. The first node in this
    /// sequence is always the root.
    nodes: Vec<QuadNode>,
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

impl QuadNode {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline]
    pub fn is_branch(&self) -> bool {
        self.count == NODE_INDEX_IS_BRANCH
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        !self.is_branch()
    }
}


impl QuadNodeData {
    fn new(l: i32, t: i32, hx: i32, hy: i32, index: u32, depth: u32) -> Self {
        Self {
            index,
            crect: [l, t, hx, hy],
            depth
        }
    }

    fn new_from_root(root_rect: &QuadRect) -> Self {
        Self {
            index: 0,
            crect: [root_rect.l, root_rect.t, root_rect.hx, root_rect.hy],
            depth: 0
        }
    }
}

#[derive(Default)]
struct QuadNodeList {
    elements: Vec<QuadNodeData>
}

impl QuadNodeList {
    pub fn push_back(&mut self, nd: QuadNodeData) {
        self.elements.push(nd)
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn pop_back(&mut self) -> QuadNodeData {
        debug_assert!(!self.elements.is_empty());
        self.elements.pop().unwrap()
    }
}

/// Represents an element in the quadtree.
///
/// # Remarks
/// An element (`QuadTreeElement`) is only inserted once to the quadtree no matter how many
/// cells it occupies. However, for each cell it occupies, an "element node" (`QuadTreeElementNode`)
/// is inserted which indexes that element.
#[derive(Debug, PartialEq, Eq, Default)]
struct QuadTreeElement {
    /// Stores the ID for the element (can be used to refer to external data).
    id: u32,
    /// Left X coordinate of the rectangle of the element.
    x1: u32,
    /// Top Y coordinate of the rectangle of the element.
    y1: u32,
    /// Right X coordinate of the rectangle of the element.
    x2: u32,
    /// Bottom Y coordinate of the rectangle of the element.
    y2: u32
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

    fn insert(&mut self, node: QuadNodeData) {

    }

    fn find_leaves(&self, rect: [i32; 4]) -> QuadNodeList {
        let root_data = QuadNodeData::new_from_root(&self.root_rect);
        self.find_leaves_from_root(root_data, rect)
    }

    fn find_leaves_from_root(&self, root: QuadNodeData, rect: [i32; 4]) -> QuadNodeList {
        let mut leaves = QuadNodeList::default();
        let mut to_process = QuadNodeList::default();

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
            let fc = &self.nodes[nd.index as usize].first_child;
            let l = mx - hx;
            let t = my - hy;
            let r = mx + hx;
            let b = my + hy;

            if rect[1] <= my {
                if rect[0] <= mx {
                    to_process.push_back(QuadNodeData::new(l, t, hx, hy, fc + 0, nd.depth + 1));
                } else {
                    to_process.push_back(QuadNodeData::new(r, t, hx, hy, fc + 1, nd.depth + 1));
                }
            } else {
                if rect[0] <= mx {
                    to_process.push_back(QuadNodeData::new(l, b, hx, hy, fc + 2, nd.depth + 1));
                } else {
                    to_process.push_back(QuadNodeData::new(r, b, hx, hy, fc + 3, nd.depth + 1));
                }
            }
        }

        leaves
    }

    pub fn cleanup(&mut self) {
        // Only process the root if it is not a leaf.
        let mut to_process: SmallVec<[NodeIndexType; 128]> = smallvec::smallvec![]; // TODO: revisit the small list size, check element count
        if self.nodes[0].is_branch() {
            to_process.push(0);
        }

        while !to_process.is_empty() {
            let node_index = to_process.pop().unwrap();
            let first_child_index = self.nodes[node_index as usize].first_child;

            // Loop through the children.
            let mut num_empty_leaves = 0usize;
            for j in 0..4 {
                let child_index = first_child_index + j;
                let child = &self.nodes[child_index as usize];

                // Increment empty leaf count if the child is an empty
                // leaf. Otherwise if the child is a branch, add it to
                // the stack to be processed in the next iteration.
                if child.is_empty() {
                    num_empty_leaves += 1;
                } else if child.is_branch() {
                    to_process.push(child_index);
                }
            }

            // If all the children were empty leaves, remove them and
            // make this node the new empty leaf.
            if num_empty_leaves == 4 {
                // Push all 4 children to the free list.
                self.nodes[first_child_index as usize].first_child = self.free_node;
                self.free_node = first_child_index;

                // Make this node the new empty leaf.
                let node = &mut self.nodes[node_index as usize];
                node.first_child = free_list::SENTINEL;
                node.count = 0;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn quad_node_is_eight_bytes() {
        assert_eq!(std::mem::size_of::<QuadNode>(), 8);
    }

    #[test]
    fn insert_works() {
        let tree = QuadTree::default();
    }
}
