use crate::intersections::IntersectsWith;
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
    element_idx: free_list::IndexType,
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
    pub fn default() -> Self {
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
        let element_idx = self.elements.insert(element);

        let mut to_process: SmallVec<[NodeData; 128]> =
            smallvec::smallvec![self.get_root_node_data()];

        while !to_process.is_empty() {
            let node_data = to_process.pop().unwrap();

            // Find the leaves
            let mut leaves = self.find_leaves_aabb(node_data, element_coords);

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
                    let element_node_idx = self.element_nodes.insert(QuadTreeElementNode {
                        element_idx,
                        next: first_child_or_element,
                    });
                    let node = &mut self.nodes[leaf.index as usize];
                    node.first_child_or_element = element_node_idx;
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
            let element = unsafe { *self.elements.at(element_node.element_idx) };

            self.assign_element_to_child_nodes(
                mx,
                my,
                first_child_index,
                element_node.element_idx,
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
        let insert_top = element.rect.y1 <= my;
        let insert_bottom = element.rect.y2 > my;
        let insert_left = element.rect.x1 <= mx;
        let insert_right = element.rect.x2 > mx;

        if insert_top & insert_left {
            self.insert_element_in_child_node(first_child_index + 0, element_index);
        }
        if insert_top & insert_right {
            self.insert_element_in_child_node(first_child_index + 1, element_index);
        }
        if insert_bottom & insert_left {
            self.insert_element_in_child_node(first_child_index + 2, element_index);
        }
        if insert_bottom & insert_right {
            self.insert_element_in_child_node(first_child_index + 3, element_index);
        }
    }

    fn insert_element_in_child_node(&mut self, child_index: u32, element: free_list::IndexType) {
        let node = &mut self.nodes[child_index as usize];
        let element_node_index = self.element_nodes.insert(QuadTreeElementNode {
            element_idx: element,
            next: node.first_child_or_element,
        });
        node.first_child_or_element = element_node_index;
        node.element_count += 1;
    }

    pub fn remove(&mut self, element: &QuadTreeElement<Id>) -> bool {
        // Find the leaves containing the node.
        let element_coords = &element.rect;
        let root = self.get_root_node_data();

        // The index of the element (if it was found).
        let mut found_element_idx = free_list::SENTINEL;

        let mut leaves = self.find_leaves_aabb(root, element_coords);
        while !leaves.is_empty() {
            let leaf = leaves.pop_back();
            let leaf_node_data = self.nodes[leaf.index as usize];
            debug_assert!(leaf_node_data.element_count >= 1);

            // Used for debug assertion.
            let mut element_found = false;

            // Find the element in question.
            let mut element_node_idx = leaf_node_data.first_child_or_element;
            let mut prev_element_node_idx = element_node_idx;
            let mut new_first_child_or_element = element_node_idx;

            while element_node_idx != free_list::SENTINEL {
                let elem_node = *unsafe { self.element_nodes.at(element_node_idx) };
                let elem = unsafe { self.elements.at(elem_node.element_idx) };

                if elem.id == element.id {
                    debug_assert!(!element_found);
                    element_found = true;

                    // If the element to be deleted is the first element,
                    // we need to update the leaf.
                    if leaf_node_data.first_child_or_element == element_node_idx {
                        new_first_child_or_element = elem_node.next;
                    }

                    // Update the previous node if it exists.
                    if element_node_idx != prev_element_node_idx {
                        unsafe { self.element_nodes.at_mut(prev_element_node_idx) }.next =
                            elem_node.next;
                    }

                    // Remove the reference from this leaf and
                    // keep track of the element index in the list.
                    self.element_nodes.erase(element_node_idx);
                    debug_assert!(
                        found_element_idx == free_list::SENTINEL
                            || found_element_idx == elem_node.element_idx
                    );
                    found_element_idx = elem_node.element_idx;
                }

                prev_element_node_idx = element_node_idx;
                element_node_idx = elem_node.next;

                // We assume that a user never inserts the same element
                // twice, therefore there is no need to visit the other
                // elements of this node if we found the correct one.
                //
                // To assert that elements are only inserted once (per node),
                // we allow further iteration during debugging.
                #[cfg(not(debug_assertions))]
                if element_found {
                    break;
                }
            }

            // Update the leaf node itself.
            let node = &mut self.nodes[leaf.index as usize];
            node.first_child_or_element = new_first_child_or_element;

            debug_assert!(element_found);
            debug_assert!(node.element_count > 0);
            node.element_count -= 1;
        }

        if found_element_idx != free_list::SENTINEL {
            self.elements.erase(found_element_idx);
            true
        } else {
            false
        }
    }

    fn find_leaves_aabb(&self, root: NodeData, rect: &AABB) -> NodeList {
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

            let fc = self.nodes[nd.index as usize].get_first_child_node_index();

            // Otherwise push the children that intersect the rectangle.
            let quadrants = nd.crect.explore_quadrants_aabb(rect);

            let mx = nd.crect.center_x;
            let my = nd.crect.center_y;
            let hx = nd.crect.width >> 1;
            let hy = nd.crect.height >> 1;

            let l = mx - hx;
            let t = my - hy;
            let r = mx + hx;
            let b = my + hy;

            if quadrants.top_left {
                to_process.push_back(NodeData::new(l, t, hx, hy, fc + 0, nd.depth + 1));
            }
            if quadrants.top_right {
                to_process.push_back(NodeData::new(r, t, hx, hy, fc + 1, nd.depth + 1));
            }
            if quadrants.bottom_left {
                to_process.push_back(NodeData::new(l, b, hx, hy, fc + 2, nd.depth + 1));
            }
            if quadrants.bottom_right {
                to_process.push_back(NodeData::new(r, b, hx, hy, fc + 3, nd.depth + 1));
            }
        }

        leaves
    }

    fn find_leaves_generic<T>(&self, root: NodeData, element: &T) -> NodeList
    where
        T: IntersectsWith<AABB>,
    {
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

            let fc = self.nodes[nd.index as usize].get_first_child_node_index();

            // Otherwise push the children that intersect the rectangle.
            let quadrants = nd.crect.explore_quadrants_generic(element);

            let mx = nd.crect.center_x;
            let my = nd.crect.center_y;
            let hx = nd.crect.width >> 1;
            let hy = nd.crect.height >> 1;

            let l = mx - hx;
            let t = my - hy;
            let r = mx + hx;
            let b = my + hy;

            if quadrants.top_left {
                to_process.push_back(NodeData::new(l, t, hx, hy, fc + 0, nd.depth + 1));
            }
            if quadrants.top_right {
                to_process.push_back(NodeData::new(r, t, hx, hy, fc + 1, nd.depth + 1));
            }
            if quadrants.bottom_left {
                to_process.push_back(NodeData::new(l, b, hx, hy, fc + 2, nd.depth + 1));
            }
            if quadrants.bottom_right {
                to_process.push_back(NodeData::new(r, b, hx, hy, fc + 3, nd.depth + 1));
            }
        }

        leaves
    }

    pub fn cleanup(&mut self) -> bool {
        // Only process the root if it is not a leaf.
        if self.nodes[0].is_leaf() {
            return false;
        }

        let mut tree_compacted = false;

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
                // TODO: Reverse, compare to zero?
                // Push all 4 children to the free list.
                // (We don't change the indexes of the 2nd to 4th child because
                // child nodes are always processed together.)
                self.nodes[first_child_index as usize].first_child_or_element = self.free_node;
                self.free_node = first_child_index;

                // Make this node the new empty leaf.
                let node = &mut self.nodes[node_index as usize];
                node.make_empty_leaf();

                tree_compacted = true;
            }
        }

        tree_compacted
    }

    /// Counts the total number of references. This number should be at least
    /// the number of elements inserted; it will be higher if elements
    /// span multiple cells.
    pub(crate) fn count_element_references(&self) -> usize {
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

    /// Returns the set of IDs that occupy space within the
    /// specified bounding box.
    ///
    /// # Arguments
    /// * [`rect`] - The rectangle to test for.
    pub fn intersect_aabb(&self, rect: &AABB) -> HashSet<Id> {
        let root = self.get_root_node_data();
        let leaves = self.find_leaves_aabb(root, rect);
        self.intersect_from_leaves(rect, leaves)
    }

    /// Returns the set of IDs that occupy space within the
    /// specified bounding box.
    ///
    /// # Arguments
    /// * [`rect`] - The rectangle to test for.
    pub fn intersect_generic<T>(&self, element: &T) -> HashSet<Id>
    where
        T: IntersectsWith<AABB>,
    {
        let root = self.get_root_node_data();
        let leaves = self.find_leaves_generic(root, element);
        self.intersect_from_leaves(element, leaves)
    }

    fn intersect_from_leaves<T>(&self, rect: &T, mut leaves: NodeList) -> HashSet<Id>
    where
        T: IntersectsWith<AABB>,
    {
        let capacity = leaves.len() * MAX_NUM_ELEMENTS as usize;
        let mut node_set = HashSet::with_capacity(capacity);

        while !leaves.is_empty() {
            let leaf_data = leaves.pop_back();
            let leaf = self.nodes[leaf_data.index as usize];
            debug_assert!(leaf.is_leaf());

            let mut elem_node_idx = leaf.first_child_or_element;
            while elem_node_idx != free_list::SENTINEL {
                let elem_node = unsafe { self.element_nodes.at(elem_node_idx) };
                let elem = unsafe { self.elements.at(elem_node.element_idx) };

                // Depending on the size of the quadrant, the candidate element
                // might still not be covered by the search rectangle.
                if rect.intersects_with(&elem.rect) {
                    let _was_known = node_set.insert(elem.id);
                }

                elem_node_idx = elem_node.next;
            }
        }

        node_set
    }

    /// Collects all element IDs stored in the tree by visiting all cells.
    pub(crate) fn collect_ids(&self) -> HashSet<Id> {
        let aabb: AABB = self.root_rect.into();
        self.intersect_aabb(&aabb)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cleanup_works() {
        let quad_rect = QuadRect::new(-20, -20, 40, 40);
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

        // Similar to index test.
        assert_eq!(tree.collect_ids().len(), 6);
        assert_eq!(tree.count_element_references(), 9);

        // Erase the all elements.
        assert!(tree.remove(&QuadTreeElement::new(1000, AABB::new(-15, -15, -5, -5))));
        assert!(tree.remove(&QuadTreeElement::new(1001, AABB::new(-20, -20, -18, -18))));
        assert!(tree.remove(&QuadTreeElement::new(2000, AABB::new(5, -15, 15, -5))));
        assert!(tree.remove(&QuadTreeElement::new(3000, AABB::new(-15, 5, -5, 15))));
        assert!(tree.remove(&QuadTreeElement::new(4000, AABB::new(5, 5, 15, 15))));
        assert!(tree.remove(&QuadTreeElement::new(5000, AABB::new(-5, -5, 5, 5))));
        assert_eq!(tree.collect_ids().len(), 0);
        assert_eq!(tree.count_element_references(), 0);

        // Since cleanup wasn't called yet, the root is still considered a branch
        // with four child nodes.
        assert!(tree.nodes[0].is_branch());
        assert_eq!(tree.nodes[0].first_child_or_element, 1);

        // Cleanup does something now.
        assert!(tree.cleanup());

        // Since all four child nodes of the root were empty,
        // cleanup has removed them. The root node is now a leaf
        // with zero elements.
        assert!(tree.nodes[0].is_leaf());
        assert_eq!(tree.nodes[0].element_count, 0);
    }
}
