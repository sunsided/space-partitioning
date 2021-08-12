use crate::intersections::Intersects;
use crate::quadtree::aabb::AABB;
use crate::quadtree::free_list;
use crate::quadtree::free_list::{FreeList, IndexType};
use crate::quadtree::node::{Node, NodeElementCountType};
use crate::quadtree::node_data::{NodeData, NodeIndexType};
use crate::quadtree::node_list::NodeList;
use crate::quadtree::quad_rect::QuadRect;
use smallvec::SmallVec;
use std::collections::HashSet;

/// Each node must have less than the maximum allowed number of elements.
const MAX_NUM_ELEMENTS: NodeElementCountType = 1; // TODO: Make parameter of tree

/// We use this value to determine whether a node can be split.
const SMALLEST_CELL_SIZE: i32 = 1; // TODO: Make parameter of tree

/// Represents an element in the QuadTree.
#[derive(Debug, PartialEq, Eq, Default, Copy, Clone)]
pub struct QuadTreeElement<Id = u32>
where
    Id: Default,
{
    // TODO: Split element into two structs: One containing the ID, another one containing the coordinates only. This allows aligning the elements much better. Benchmark!
    /// Stores the ID for the element (can be used to refer to external data).
    pub id: Id,
    /// The axis-aligned bounding box of the element.
    rect: AABB,
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
    Id: Default + std::cmp::Eq + Copy,
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
    fn contains(&self, rect: &AABB) -> bool {
        let r = self.l + self.hx;
        let b = self.t + self.hy;
        rect.x1 >= self.l && rect.x2 <= r && rect.y1 >= self.t && rect.y2 <= b
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

impl<Id> QuadTreeElement<Id>
where
    Id: Default,
{
    pub fn new(id: Id, rect: AABB) -> Self {
        Self { id, rect }
    }

    pub fn new_xy(id: Id, x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        Self {
            id,
            rect: AABB::new(x1, y1, x2, y2),
        }
    }
}

impl<Id> QuadTree<Id>
where
    Id: Default + std::cmp::Eq + std::hash::Hash + Copy,
{
    fn default() -> Self {
        Self::new(QuadRect::default(), 8)
    }
}

impl<Id> QuadTree<Id>
where
    Id: Default + std::cmp::Eq + std::hash::Hash + Copy,
{
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

    pub fn insert(&mut self, element: QuadTreeElement<Id>) {
        let element_coords = &element.rect;
        assert!(self.root_rect.contains(element_coords));

        // Insert the actual element.
        let element_index = self.elements.insert(element);

        let mut to_process: SmallVec<[NodeData; 128]> =
            smallvec::smallvec![self.get_root_node_data()];

        while !to_process.is_empty() {
            let node_data = to_process.pop().unwrap();

            // Find the leaves // TODO: Doesn't seem to work for center rect example
            let mut leaves = self.find_leaves_from_root(node_data, element_coords);

            while !leaves.is_empty() {
                let leaf = leaves.pop_back();

                let (element_count, first_child_or_element) = {
                    let node = &self.nodes[leaf.index as usize];
                    debug_assert!(node.is_leaf());
                    (node.element_count, node.first_child_or_element)
                };

                let can_split = leaf.can_split_further(SMALLEST_CELL_SIZE, self.max_depth);
                let node_is_full = element_count >= MAX_NUM_ELEMENTS;

                let must_store_element = !node_is_full || !can_split;
                if must_store_element {
                    // This leaf takes the element reference without further splitting.
                    let element_node_index = self.element_nodes.insert(QuadTreeElementNode {
                        element: element_index,
                        next: first_child_or_element,
                    });
                    let node = &mut self.nodes[leaf.index as usize];
                    node.first_child_or_element = element_node_index;
                    node.element_count += 1;
                } else {
                    // At this point we have to split the current node.
                    // We push the leaf back onto the stack in order to try to
                    // find a better insertion candidate from there.
                    self.distribute_elements_to_child_nodes(&leaf);
                    to_process.push(leaf);
                }
            }
        }
    }

    /// Splits the specified [`parent`] node into four and distributes its
    /// elements onto the newly created childs.
    fn distribute_elements_to_child_nodes(&mut self, parent: &NodeData) {
        let first_child_index = self.ensure_child_nodes_exist();

        let node = &mut self.nodes[parent.index as usize];
        let mut element_node_index = node.get_first_element_node_index();
        node.make_branch(first_child_index);

        let mx = parent.crect.center_x;
        let my = parent.crect.center_y;

        // For each element in the list ...
        while element_node_index != free_list::SENTINEL {
            let element_node = unsafe { *self.element_nodes.at(element_node_index) };
            let element = unsafe { *self.elements.at(element_node.element) };

            self.assign_element_to_child_nodes(
                mx,
                my,
                first_child_index,
                element_node.element,
                &element,
            );

            // The element was assigned to the child nodes - the former node
            // can be removed (since the former leaf doesn't exist anymore).
            self.element_nodes.erase(element_node_index);

            element_node_index = element_node.next;
        }
    }

    /// Recycles child nodes from the free list or creates
    /// new child nodes if needed.
    fn ensure_child_nodes_exist(&mut self) -> u32 {
        if self.free_node == free_list::SENTINEL {
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
        }
    }

    /// Assigns an element to the child nodes starting at `first_child_index`.
    ///
    /// # Params
    /// * [`mx`] - The center X coordinate of the parent node.
    /// * [`my`] - The center Y coordinate of the parent node.
    /// * [`first_child_index`] - The index of the first child node.
    /// * [`element_index`] - The index of the element.
    /// * [`element`] - The element data.
    fn assign_element_to_child_nodes(
        &mut self,
        mx: i32,
        my: i32,
        first_child_index: free_list::IndexType,
        element_index: free_list::IndexType,
        element: &QuadTreeElement<Id>,
    ) {
        if element.rect.y1 <= my {
            if element.rect.x1 <= mx {
                self.insert_element_in_child_node(first_child_index + 0, element_index);
            }
            if element.rect.x2 > mx {
                self.insert_element_in_child_node(first_child_index + 1, element_index);
            }
        }
        if element.rect.y2 > my {
            if element.rect.x1 <= mx {
                self.insert_element_in_child_node(first_child_index + 2, element_index);
            }
            if element.rect.x2 > mx {
                self.insert_element_in_child_node(first_child_index + 3, element_index);
            }
        }
    }

    fn insert_element_in_child_node(&mut self, child_index: u32, element: free_list::IndexType) {
        let node = &mut self.nodes[child_index as usize];
        let element_node_index = self.element_nodes.insert(QuadTreeElementNode {
            element,
            next: node.first_child_or_element,
        });
        node.first_child_or_element = element_node_index;
        node.element_count += 1;
    }

    fn remove(&mut self, node: NodeData) {
        // TODO: set removed node to free_head
        // TODO: Set free_head to removed node index
        todo!()
    }

    fn find_leaves_from_root(&self, root: NodeData, rect: &AABB) -> NodeList {
        let mut leaves = NodeList::default(); // TODO: extract / pool?
        let mut to_process = NodeList::default(); // TODO: measure max size - back by SmallVec?
        to_process.push_back(root);

        while to_process.len() > 0 {
            let nd = to_process.pop_back();

            // If this node is a leaf, insert it to the list.
            if self.nodes[nd.index as usize].is_leaf() {
                leaves.push_back(nd);
                continue;
            }

            // Otherwise push the children that intersect the rectangle.
            let mx = nd.crect.center_x;
            let my = nd.crect.center_y;
            let hx = nd.crect.width >> 1;
            let hy = nd.crect.height >> 1;
            let fc = self.nodes[nd.index as usize].get_first_child_node_index();
            let l = mx - hx;
            let t = my - hy;
            let r = mx + hx;
            let b = my + hy;

            // TODO: Inserting a very large element over many very small ones could yield a lot of nodes.
            if rect.y1 <= my {
                if rect.x1 <= mx {
                    to_process.push_back(NodeData::new(l, t, hx, hy, fc + 0, nd.depth + 1));
                }
                if rect.x2 > mx {
                    to_process.push_back(NodeData::new(r, t, hx, hy, fc + 1, nd.depth + 1));
                }
            }
            if rect.y2 > my {
                if rect.x1 <= mx {
                    to_process.push_back(NodeData::new(l, b, hx, hy, fc + 2, nd.depth + 1));
                }
                if rect.x2 > mx {
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

    /// Counts the total number of references. This number should be at least
    /// the number of elements inserted; it will be higher if elements
    /// span multiple cells.
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

        debug_assert!(count >= self.elements.debug_len());
        count
    }

    #[inline]
    fn get_root_node_data(&self) -> NodeData {
        NodeData::new_from_root(&self.root_rect)
    }

    pub fn intersect(&self, rect: &AABB) -> HashSet<Id> {
        let root = self.get_root_node_data();
        let mut leaves = self.find_leaves_from_root(root, rect);

        let capacity = leaves.len() * MAX_NUM_ELEMENTS as usize;
        let mut node_set = HashSet::with_capacity(capacity);

        while !leaves.is_empty() {
            let leaf_data = leaves.pop_back();
            let leaf = self.nodes[leaf_data.index as usize];
            debug_assert!(leaf.is_leaf());

            let mut pointer = leaf.first_child_or_element;

            while pointer != free_list::SENTINEL {
                let elem_node = unsafe { self.element_nodes.at(pointer) };
                let elem = unsafe { self.elements.at(elem_node.element) };

                // Depending on the size of the quadrant, the candidate element
                // might still not be covered by the search rectangle.
                if elem.rect.intersects_with(rect) {
                    let _was_known = node_set.insert(elem.id);
                }

                pointer = elem_node.next;
            }
        }

        node_set
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn insert_once_works() {
        let mut tree = QuadTree::default();
        tree.insert(QuadTreeElement {
            id: 0,
            rect: AABB::default(),
        });
        assert_eq!(tree.count_element_references(), 1);
        tree.cleanup();
        assert_eq!(tree.count_element_references(), 1);
    }

    #[test]
    fn insert_twice_works() {
        let mut tree = QuadTree::default();
        let count = 2i32;
        for id in 0..count {
            tree.insert(QuadTreeElement {
                id,
                rect: AABB::new(-id, -id, id + 1, id + 1),
            });
        }
        assert_eq!(tree.count_element_references(), 5);
        tree.cleanup();
        assert_eq!(tree.count_element_references(), 5);
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
                id,
                rect: AABB::new(x, y, x + 1, y + 1),
            });
            x += 1;
            if x == 16 {
                x = -16;
                y += 1;
            }
        }
        assert_eq!(tree.count_element_references(), 1369 as usize);
        let tree = tree;
    }

    #[test]
    fn find_works() {
        let quad_rect = QuadRect {
            l: -20,
            t: -20,
            hx: 40,
            hy: 40,
        };
        let mut tree = QuadTree::new(quad_rect, 1);
        // top-left
        tree.insert(QuadTreeElement::new(1000, AABB::new(-15, -15, -5, -5)));
        tree.insert(QuadTreeElement::new(1001, AABB::new(-20, -20, -18, -18)));
        // top-right
        tree.insert(QuadTreeElement::new(2000, AABB::new(5, -15, 15, -5)));
        // bottom-left
        tree.insert(QuadTreeElement::new(3000, AABB::new(-15, 5, -5, 15)));
        // bottom-right
        tree.insert(QuadTreeElement::new(4000, AABB::new(5, 5, 15, 15)));
        // center
        tree.insert(QuadTreeElement::new(5000, AABB::new(-5, -5, 5, 5)));

        // The depth of 1 limits the tree to four quadrants.
        // Each of the first five elements creates a single reference
        // in each of the quadrants. The "center" element covers
        // all four quadrants, and therefore adds another four references.
        assert_eq!(tree.count_element_references(), 9);

        // Select the top-left quadrant
        let quadrant_tl = AABB::new(-17, -17, 0, 0);
        let results = tree.find_leaves_from_root(tree.get_root_node_data(), &quadrant_tl);

        // Perform the actual intersection.
        let results = tree.intersect(&quadrant_tl);
        let results = Vec::from_iter(results);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1000));
        assert!(!results.contains(&1001));
        assert!(results.contains(&5000));
    }
}
