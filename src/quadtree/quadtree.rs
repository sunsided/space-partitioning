use crate::quadtree::free_list;
use crate::quadtree::free_list::{FreeList, IndexType};
use crate::quadtree::node::{Node, NodeElementCountType};
use crate::quadtree::node_data::{NodeData, NodeIndexType};
use crate::quadtree::node_list::NodeList;
use crate::quadtree::quad_rect::QuadRect;
use smallvec::SmallVec;

/// Represents an element in the QuadTree.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct QuadTreeElement<Id = u32>
where
    Id: Default,
{
    /// Stores the ID for the element (can be used to refer to external data).
    pub id: Id,
    /// Left X coordinate of the rectangle of the element.
    pub x1: i32,
    /// Top Y coordinate of the rectangle of the element.
    pub y1: i32,
    /// Right X coordinate of the rectangle of the element.
    pub x2: i32,
    /// Bottom Y coordinate of the rectangle of the element.
    pub y2: i32,
}

/// Represents an element node in the quadtree.
///
/// # Remarks
/// An element (`QuadTreeElement`) is only inserted once to the quadtree no matter how many
/// cells it occupies. However, for each cell it occupies, an "element node" (`QuadTreeElementNode`)
/// is inserted which indexes that element.
#[derive(Debug, PartialEq, Eq, Default, Copy, Clone)]
struct QuadTreeElementNode {
    /// Points to the next element in the leaf node. A value of `free_list::SENTINEL`
    /// indicates the end of the list.
    next: free_list::IndexType,
    /// Stores the element index.
    element: free_list::IndexType,
}

pub struct QuadTree<Id = u32>
where
    Id: Default,
{
    /// Stores all the elements in the quadtree.
    /// An element is only inserted once to the quadtree no matter how many cells it occupies.
    elements: FreeList<QuadTreeElement<Id>>,
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
    max_depth: u32,
}

impl QuadRect {
    fn contains(&self, element: &QuadTreeElement) -> bool {
        let r = self.l + self.hx;
        let b = self.t + self.hy;
        element.x1 >= self.l && element.x2 <= r && element.y1 >= self.t && element.y2 <= b
    }
}

impl Default for QuadRect {
    fn default() -> Self {
        QuadRect {
            l: i32::MIN >> 1,
            t: i32::MIN >> 1,
            hx: i32::MAX,
            hy: i32::MAX,
        }
    }
}

impl QuadTreeElement {
    fn to_array(&self) -> [i32; 4] {
        let hx = self.x2 - self.x1;
        let hy = self.y2 - self.y1;
        [self.x1, self.y1, hx, hy]
    }
}

impl Default for QuadTree {
    fn default() -> Self {
        Self::new(QuadRect::default(), 8)
    }
}

impl QuadTree {
    pub fn new(root_rect: QuadRect, max_depth: u32) -> Self {
        Self {
            elements: FreeList::default(),
            element_nodes: FreeList::default(),
            nodes: vec![Node::default()],
            root_rect,
            free_node: free_list::SENTINEL,
            max_depth,
        }
    }

    fn count_element_references(&self) -> usize {
        let mut to_process: SmallVec<[usize; 128]> = smallvec::smallvec![0];
        let mut count = 0usize;
        while !to_process.is_empty() {
            let index = to_process.pop().unwrap();
            let node = &self.nodes[index];
            if node.is_branch() {
                for j in 0..4 {
                    to_process.push((node.first_child_or_element + j) as usize);
                }
            } else {
                count += node.element_count as usize;
            }
        }
        count
    }

    fn insert(&mut self, element: QuadTreeElement) {
        assert!(self.root_rect.contains(&element));

        // Each node must have less than the maximum allowed number of elements.
        const MAX_NUM_ELEMENTS: NodeElementCountType = 1;

        // We use this value to determine whether a node can be split.
        const SMALLEST_CELL_SIZE: i32 = 1;

        let element_coords = element.to_array();

        // Insert the actual element.
        let element_index = self.elements.insert(element);

        let mut to_process: SmallVec<[NodeData; 128]> =
            smallvec::smallvec![NodeData::new_from_root(&self.root_rect)];

        while !to_process.is_empty() {
            let node_data = to_process.pop().unwrap();
            let leaves = self.find_leaves_from_root(node_data, element_coords.clone());

            for leaf in leaves.into_iter() {
                // Only here to assist the IDE in determining the type.
                let leaf: NodeData = leaf;

                let (element_count, first_child_or_element) = {
                    let node = self.nodes[leaf.index as usize];
                    debug_assert!(node.is_leaf());
                    (node.element_count, node.first_child_or_element)
                };

                // Do not subdivide node if after subdivision cells would be less than the smallest size.
                let can_split = leaf.can_split(SMALLEST_CELL_SIZE);

                // Determine if the node must store an element (or should subdivide).
                let node_is_full = element_count >= MAX_NUM_ELEMENTS;
                let must_store_element = leaf.depth == self.max_depth;
                let store_element = !node_is_full || must_store_element || !can_split;

                if store_element {
                    let element_node_index = self.element_nodes.insert(QuadTreeElementNode {
                        element: element_index,
                        next: first_child_or_element,
                    });
                    let node = &mut self.nodes[leaf.index as usize];
                    node.first_child_or_element = element_node_index;
                    node.element_count += 1;
                    return;
                }

                // Split the node
                debug_assert_eq!(MAX_NUM_ELEMENTS, 1);

                self.distribute_elements_to_child_nodes(&leaf);

                // At this point we have only split the current node but must still
                // insert the new item. To do this we use the former leaf node as a
                // starting point and restart a search from there.
                to_process.push(leaf);
            }
        }
    }

    fn distribute_elements_to_child_nodes(&mut self, leaf: &NodeData) {
        // Create or recycle child nodes.
        let first_child_index = if self.free_node == free_list::SENTINEL {
            let node_index = self.nodes.len() as IndexType;
            for _ in 0..4 {
                self.nodes.push(Node::default());
            }
            node_index
        } else {
            let node_index = self.free_node;
            let next_free_node = self.nodes[node_index as usize].first_child_or_element;
            self.nodes[node_index as usize] = Node::default();
            self.free_node = next_free_node;
            node_index
        };

        // Mutably get the node we need to convert to a branch.
        let node = &mut self.nodes[leaf.index as usize];

        // Get the head of the list pointing to the elements.
        let mut element_ptr = node.get_first_element_node_index();

        // Convert this node to a branch.
        node.make_branch(first_child_index);
        debug_assert!(node.is_branch());

        // Get the boundaries of the leaf.
        let mx = leaf.crect[0];
        let my = leaf.crect[1];

        // For each element in the list ...
        while element_ptr != free_list::SENTINEL {
            let current_element_node = unsafe { *self.element_nodes.at(element_ptr) };
            let current_element = unsafe { self.elements.at(current_element_node.element) };

            // If the top of the element is north of the center, we must register it there.
            if current_element.y1 <= my {
                // If the left of the element is west of the center, we must register it there.
                if current_element.x1 <= mx {
                    let top_left = &mut self.nodes[(first_child_index + 0) as usize];
                    let element_node_index = self.element_nodes.insert(QuadTreeElementNode {
                        element: current_element_node.element,
                        next: top_left.first_child_or_element,
                    });
                    top_left.first_child_or_element = element_node_index;
                    top_left.element_count += 1;
                }

                // If the right of the element is east of the center, we must also register it there.
                if current_element.x2 > mx {
                    let top_right = &mut self.nodes[(first_child_index + 1) as usize];
                    let element_node_index = self.element_nodes.insert(QuadTreeElementNode {
                        element: current_element_node.element,
                        next: top_right.first_child_or_element,
                    });
                    top_right.first_child_or_element = element_node_index;
                    top_right.element_count += 1;
                }
            }

            // If the bottom of the element is south of the center, we must also register it there.
            if current_element.y2 > my {
                // If the left of the element is west of the center, we must register it there.
                if current_element.x1 <= mx {
                    let bottom_left = &mut self.nodes[(first_child_index + 2) as usize];
                    let element_node_index = self.element_nodes.insert(QuadTreeElementNode {
                        element: current_element_node.element,
                        next: bottom_left.first_child_or_element,
                    });
                    bottom_left.first_child_or_element = element_node_index;
                    bottom_left.element_count += 1;
                }

                // If the right of the element is east of the center, we must also register it there.
                if current_element.x2 > mx {
                    let bottom_right = &mut self.nodes[(first_child_index + 3) as usize];
                    let element_node_index = self.element_nodes.insert(QuadTreeElementNode {
                        element: current_element_node.element,
                        next: bottom_right.first_child_or_element,
                    });
                    bottom_right.first_child_or_element = element_node_index;
                    bottom_right.element_count += 1;
                }
            }

            // The element was assigned to the child nodes - it can be removed from the
            // former branch.
            self.element_nodes.erase(element_ptr);

            // Move to the next element in the list.
            element_ptr = current_element_node.next;
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
            if num_empty_leaves == 4 {
                // TODO: Reverse, compare to zero
                // Push all 4 children to the free list.
                // (We don't change the indexes of the 2nd to 4th child because
                // child nodes are always processed together.)
                self.nodes[first_child_index as usize].first_child_or_element = self.free_node;
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
    fn insert_once_works() {
        let mut tree = QuadTree::default();
        tree.insert(QuadTreeElement {
            id: 0,
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
        });
        assert_eq!(tree.count_element_references(), 1);
    }

    #[test]
    fn insert_twice_works() {
        let mut tree = QuadTree::default();
        let count = 2i32;
        for id in 0..count {
            tree.insert(QuadTreeElement {
                id: id as _,
                x1: -id,
                y1: -id,
                x2: id + 1,
                y2: id + 1,
            });
        }
        assert_eq!(tree.count_element_references(), count as usize);
    }

    #[test]
    fn insert_a_lot_works() {
        let mut tree = QuadTree::new(
            QuadRect {
                l: -16,
                t: -16,
                hx: 32,
                hy: 32,
            },
            8,
        );
        let count = 1024i32;
        let mut x = -16;
        let mut y = -16;
        for id in 0..count {
            tree.insert(QuadTreeElement {
                id: id as _,
                x1: x,
                y1: y,
                x2: x + 1,
                y2: y + 1,
            });
            x += 1;
            if x == 16 {
                x = -16;
                y += 1;
            }
        }
        assert_eq!(tree.count_element_references(), count as usize);
        let tree = tree;
    }
}
