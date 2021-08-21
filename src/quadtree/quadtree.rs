use crate::intersections::IntersectsWith;
use crate::quadtree::aabb::AABB;
use crate::quadtree::centered_aabb::CenteredAABB;
use crate::quadtree::error::InsertError;
use crate::quadtree::free_list::{self, FreeList, IndexType};
use crate::quadtree::node::Node;
use crate::quadtree::node_data::{NodeData, NodeIndexType};
use crate::quadtree::node_info::NodeInfo;
use crate::quadtree::node_list::NodeList;
use crate::quadtree::quad_rect::QuadRect;
use crate::quadtree::quadrants::Quadrants;
use crate::quadtree::quadtree_element::QuadTreeElementNode;
pub use crate::quadtree::quadtree_element::{ElementIdType, QuadTreeElement};
use crate::types::HashSet;
use smallvec::SmallVec;

// TODO: Add range query: Query using intersect_aabb() or intersect_generic()

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum FindLeafHint {
    /// A tree query, e.g. an intersection test.
    /// These queries need to visit all leaves.
    Query,
    /// A tree mutation, i.e. insert or remove.
    /// These queries skip exploration of child nodes whenever possible.
    Mutate,
}

/// A QuadTree implementation as described in [Efficient Quadtrees](https://stackoverflow.com/a/48330314/195651).
///
/// # Remarks
/// This tree uses integral coordinates only in order to speed up box-box intersection tests.
pub struct QuadTree<ElementId = u32>
where
    ElementId: ElementIdType,
{
    /// Stores all the IDs fo the elements in the quadtree.
    /// An element is only inserted once to the quadtree no matter how many cells it occupies.
    element_ids: FreeList<ElementId>,
    /// Stores all the rectangles of the elements in the quadtree.
    /// An element is only inserted once to the quadtree no matter how many cells it occupies.
    element_rects: FreeList<AABB>,
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
    /// Stores the maximum number of elements allowed before a node splits.
    max_num_elements: u32,
    /// We use this value to determine whether a node can be split.
    smallest_cell_size: u32,
    /// Stores the maximum depth allowed for the quadtree.
    max_depth: u8,
}

impl<ElementId> QuadTree<ElementId>
where
    ElementId: ElementIdType,
{
    pub fn default() -> Self {
        Self::new(QuadRect::default(), 8, 16, 1)
    }

    pub fn new(
        root_rect: QuadRect,
        max_depth: u8,
        max_num_elements: u32,
        smallest_cell_size: u32,
    ) -> Self {
        assert!(max_num_elements > 0);
        assert!(smallest_cell_size > 0);
        Self {
            element_ids: FreeList::default(),
            element_rects: FreeList::default(),
            element_nodes: FreeList::default(),
            nodes: vec![Node::default()],
            root_rect,
            free_node: free_list::SENTINEL,
            max_depth,
            max_num_elements,
            smallest_cell_size,
        }
    }

    pub fn insert(&mut self, element: QuadTreeElement<ElementId>) -> Result<(), InsertError> {
        let element_coords = &element.rect;
        if !self.root_rect.contains(element_coords) {
            return Err(InsertError::OutOfBounds);
        }

        let max_num_elements = self.max_num_elements;

        // Insert the actual element.
        let element_idx = self.element_ids.insert(element.id);
        let element_rect_idx = self.element_rects.insert(element.rect);
        debug_assert_eq!(element_idx, element_rect_idx);

        let mut to_process: SmallVec<[NodeData; 128]> =
            smallvec::smallvec![self.get_root_node_data()];

        while !to_process.is_empty() {
            let node_data = to_process.pop().unwrap();

            // Find the leaves
            let mut leaves = self.find_leaves_aabb(node_data, element_coords, FindLeafHint::Mutate);

            while !leaves.is_empty() {
                let leaf = leaves.pop_back();

                let (element_count, first_child_or_element) = {
                    let node = &self.nodes[leaf.index as usize];
                    debug_assert!(node.is_leaf());
                    (node.element_count, node.first_child_or_element)
                };

                let can_split = leaf.can_split_further(self.smallest_cell_size, self.max_depth);
                let node_is_full = element_count >= max_num_elements;

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

        Ok(())
    }

    /// Splits the specified [`parent`] node into four and distributes its
    /// elements onto the newly created children.
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
            let element = unsafe { *self.element_rects.at(element_node.element_idx) };

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
        if self.free_node != free_list::SENTINEL {
            let node_index = self.free_node;
            let next_free_node = self.nodes[node_index as usize].first_child_or_element;
            self.nodes[node_index as usize] = Node::default();
            self.free_node = next_free_node;
            node_index
        } else {
            let node_index = self.nodes.len() as IndexType;
            // The first node captures all elements spanning more than one child.
            self.nodes.push(Node::default());
            // The four childs.
            for _ in 0..4 {
                self.nodes.push(Node::default());
            }
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
        element_rect: &AABB,
    ) {
        let insert_left = element_rect.tl.x <= mx;
        let insert_right = element_rect.br.x > mx;
        let insert_top = element_rect.tl.y <= my;
        let insert_bottom = element_rect.br.y > my;

        // If an element covers more than one child node, we store it separately.
        let covers_many = (insert_top & insert_bottom) | (insert_left & insert_right);
        if covers_many {
            self.insert_element_in_child_node(first_child_index + 0, element_index);
            return;
        }

        // At this point, exactly one of the quadrants is selected.
        debug_assert!(
            (insert_top & insert_left)
                || (insert_top & insert_right)
                || (insert_bottom & insert_left)
                || (insert_bottom && insert_right)
        );
        if insert_top & insert_left {
            self.insert_element_in_child_node(first_child_index + 1, element_index);
        } else if insert_top & insert_right {
            self.insert_element_in_child_node(first_child_index + 2, element_index);
        } else if insert_bottom & insert_left {
            self.insert_element_in_child_node(first_child_index + 3, element_index);
        } else if insert_bottom & insert_right {
            self.insert_element_in_child_node(first_child_index + 4, element_index);
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

    /// Removes the specified element.
    ///
    /// # Remarks
    /// The element is located using its bounding box and identified using the ID.
    /// Because of that, the bounding box of the element must not change until is was
    /// removed from the tree.
    ///
    /// # Arguments
    /// * [`element`] - The element to remove.
    pub fn remove(&mut self, element: &QuadTreeElement<ElementId>) -> bool {
        // Find the leaves containing the node.
        let element_coords = &element.rect;
        let root = self.get_root_node_data();

        // The index of the element (if it was found).
        let mut found_element_idx = free_list::SENTINEL;

        let mut leaves = self.find_leaves_aabb(root, element_coords, FindLeafHint::Mutate);
        while !leaves.is_empty() {
            let leaf = leaves.pop_back();
            let leaf_node_data = self.nodes[leaf.index as usize];

            // The user may try to remove an element that was not in the tree (anymore).
            if leaf_node_data.element_count == 0 {
                continue;
            }

            // Used for debug assertion.
            let mut element_found = false;

            // Find the element in question.
            let mut element_node_idx = leaf_node_data.first_child_or_element;
            let mut prev_element_node_idx = element_node_idx;
            let mut new_first_child_or_element = element_node_idx;

            while element_node_idx != free_list::SENTINEL {
                let elem_node = *unsafe { self.element_nodes.at(element_node_idx) };
                let elem_id = unsafe { self.element_ids.at(elem_node.element_idx) };

                if *elem_id == element.id {
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

            // The user may try to remove an element that was not in the tree (anymore).
            if element_found {
                debug_assert!(node.element_count > 0);
                node.element_count -= 1;
            }
        }

        if found_element_idx != free_list::SENTINEL {
            self.element_ids.erase(found_element_idx);
            self.element_rects.erase(found_element_idx);
            true
        } else {
            false
        }
    }

    // TODO: Prefer specialization, see https://github.com/rust-lang/rust/issues/31844
    fn find_leaves_aabb(&self, root: NodeData, rect: &AABB, hint: FindLeafHint) -> NodeList {
        let mut leaves = NodeList::default(); // TODO: extract / pool?
        let mut to_process = NodeList::default();
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
            Self::collect_relevant_quadrants(&mut to_process, &nd, fc, quadrants, hint)
        }

        leaves
    }

    // TODO: Prefer specialization, see https://github.com/rust-lang/rust/issues/31844
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
            Self::collect_relevant_quadrants(
                &mut to_process,
                &nd,
                fc,
                quadrants,
                FindLeafHint::Query,
            )
        }

        leaves
    }

    pub fn visit_leaves<F>(&self, mut visit: F)
    where
        F: FnMut(NodeInfo),
    {
        let mut to_process = NodeList::default();
        to_process.push_back(self.get_root_node_data());

        while to_process.len() > 0 {
            let nd = to_process.pop_back();

            let node = &self.nodes[nd.index as usize];
            if node.is_leaf() {
                visit(NodeInfo::from(nd, node.element_count));
                continue;
            }

            let fc = self.nodes[nd.index as usize].get_first_child_node_index();
            Self::collect_relevant_quadrants(
                &mut to_process,
                &nd,
                fc,
                Quadrants::all(),
                FindLeafHint::Query,
            )
        }
    }

    #[inline]
    fn collect_relevant_quadrants(
        to_process: &mut NodeList,
        nd: &NodeData,
        first_child_id: u32,
        quadrants: Quadrants,
        hint: FindLeafHint,
    ) {
        // Opportunistically calculate the new child rects.
        // With inlining in place the compiler should be able to simplify some calculations.
        let split_quadrants = nd.crect.split_quadrants();

        match hint {
            FindLeafHint::Query => Self::collect_relevant_quadrants_for_query(
                to_process,
                nd.depth,
                first_child_id,
                quadrants,
                &split_quadrants,
            ),
            FindLeafHint::Mutate => Self::collect_relevant_quadrants_for_mutation(
                to_process,
                nd.depth,
                first_child_id,
                quadrants,
                &split_quadrants,
            ),
        }
    }

    fn collect_relevant_quadrants_for_mutation(
        to_process: &mut NodeList,
        depth: u8,
        first_child_id: u32,
        quadrants: Quadrants,
        split_quadrants: &[CenteredAABB; 5],
    ) {
        debug_assert!(
            quadrants.this()
                ^ quadrants.top_left()
                ^ quadrants.top_right()
                ^ quadrants.bottom_left()
                ^ quadrants.bottom_right()
        );

        let offset = if quadrants.this() {
            0
        } else if quadrants.top_left() {
            1
        } else if quadrants.top_right() {
            2
        } else if quadrants.bottom_left() {
            3
        } else {
            4
        };

        // The "this" node at offset 0 cannot be split.
        let can_split = offset > 0;

        // The child depth only increases for the non-"this" node.
        let mut child_depth = depth + 1;
        if offset == 0 {
            child_depth = 0;
        }

        to_process.push_back(NodeData::new(
            split_quadrants[offset as usize],
            first_child_id + offset,
            child_depth,
            can_split,
        ));
    }

    fn collect_relevant_quadrants_for_query(
        to_process: &mut NodeList,
        depth: u8,
        first_child_id: u32,
        quadrants: Quadrants,
        split_quadrants: &[CenteredAABB; 5],
    ) {
        let child_depth = depth + 1;

        for offset in (1..=4).rev() {
            if quadrants[offset] {
                to_process.push_back(NodeData::new(
                    split_quadrants[offset],
                    first_child_id + offset as u32,
                    child_depth,
                    true,
                ));
            }
        }

        // In intersection tests we always need to explore the self node.
        to_process.push_back(NodeData::new(
            split_quadrants[0],
            first_child_id + 0,
            // The "this" node is at the same depth and cannot split.
            depth,
            false,
        ));
    }

    /// Prunes unused child nodes from the tree.
    ///
    /// # Remarks
    /// The tree is never pruned automatically for performance reasons. Call
    /// this method after all elements were removed or updated.
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
            for j in 0..5 {
                let child_index = first_child_index + j;
                let child = &self.nodes[child_index as usize];

                // TODO: Compact nodes when the number of elements in child is less than allowed maximum.

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
            if num_empty_leaves == 5 {
                // Push all 5 children to the free list.
                // (We don't change the indexes of the 2nd to 4th child because
                // child nodes are always processed together.)
                self.nodes[first_child_index as usize].first_child_or_element = self.free_node;
                self.free_node = first_child_index;

                // Make this node the new empty leaf.
                self.nodes[node_index as usize].make_empty_leaf();

                tree_compacted = true;
            }
        }

        tree_compacted
    }

    /// Counts the total number of references. This number should be at least
    /// the number of elements inserted; it will be higher if elements
    /// span multiple cells.
    #[allow(dead_code)]
    pub(crate) fn count_element_references(&self) -> usize {
        let mut to_process: SmallVec<[usize; 128]> = smallvec::smallvec![0];
        let mut count = 0usize;
        while !to_process.is_empty() {
            let index = to_process.pop().unwrap();
            let node = &self.nodes[index];
            if node.is_branch() {
                for j in 0..5 {
                    to_process.push((node.first_child_or_element + j) as usize);
                }
            } else {
                count += node.element_count as usize;
            }
        }

        debug_assert!(count >= self.element_ids.debug_len());
        debug_assert!(count >= self.element_rects.debug_len());
        count
    }

    #[inline]
    fn get_root_node_data(&self) -> NodeData {
        NodeData::new_from_root(&self.root_rect, true)
    }

    /// Returns the set of IDs that occupy space within the
    /// specified bounding box.
    ///
    /// # Arguments
    /// * [`rect`] - The rectangle to test for.
    #[inline]
    pub fn intersect_aabb(&self, rect: &AABB) -> HashSet<ElementId> {
        let root = self.get_root_node_data();
        let leaves = self.find_leaves_aabb(root, rect, FindLeafHint::Query);
        let capacity = leaves.len() * self.max_num_elements as usize;
        let mut node_set = HashSet::with_capacity(capacity);
        self.intersect_from_leaves(rect, leaves, |id| {
            node_set.insert(id);
        });
        node_set
    }

    /// Calls a function for each ID that occupies space within the
    /// specified bounding box. The function may be called multiple
    /// times for the same ID.
    ///
    /// # Arguments
    /// * [`rect`] - The rectangle to test for.
    /// * [`candidate_fn`] - The function called for each candidate element's ID.
    #[inline]
    pub fn intersect_aabb_fn<F>(&self, rect: &AABB, candidate_fn: F)
    where
        F: FnMut(ElementId),
    {
        let root = self.get_root_node_data();
        let leaves = self.find_leaves_aabb(root, rect, FindLeafHint::Query);
        self.intersect_from_leaves(rect, leaves, candidate_fn);
    }

    /// Returns the set of IDs that occupy space within the
    /// specified bounding box.
    ///
    /// # Arguments
    /// * [`element`] - The element to test for.
    #[inline]
    pub fn intersect_generic<T>(&self, element: &T) -> HashSet<ElementId>
    where
        T: IntersectsWith<AABB>,
    {
        let root = self.get_root_node_data();
        let leaves = self.find_leaves_generic(root, element);
        let capacity = leaves.len() * self.max_num_elements as usize;
        let mut node_set = HashSet::with_capacity(capacity);
        self.intersect_from_leaves(element, leaves, |id| {
            node_set.insert(id);
        });
        node_set
    }

    /// Calls a function for each ID that occupies space within the
    /// specified bounding box. The function may be called multiple
    /// times for the same ID.
    ///
    /// # Arguments
    /// * [`element`] - The element to test for.
    /// * [`candidate_fn`] - The function called for each candidate element's ID.
    #[inline]
    pub fn intersect_generic_fn<T, F>(&self, element: &T, candidate_fn: F)
    where
        T: IntersectsWith<AABB>,
        F: FnMut(ElementId),
    {
        let root = self.get_root_node_data();
        let leaves = self.find_leaves_generic(root, element);
        self.intersect_from_leaves(element, leaves, candidate_fn);
    }

    fn intersect_from_leaves<T, F>(&self, rect: &T, mut leaves: NodeList, mut candidate_fn: F)
    where
        T: IntersectsWith<AABB>,
        F: FnMut(ElementId),
    {
        while !leaves.is_empty() {
            let leaf_data = leaves.pop_back();
            let leaf = self.nodes[leaf_data.index as usize];
            debug_assert!(leaf.is_leaf());

            let mut elem_node_idx = leaf.first_child_or_element;
            while elem_node_idx != free_list::SENTINEL {
                let elem_node = unsafe { self.element_nodes.at(elem_node_idx) };
                let elem_rect = unsafe { self.element_rects.at(elem_node.element_idx) };

                // Depending on the size of the quadrant, the candidate element
                // might still not be covered by the search rectangle.
                if rect.intersects_with(&elem_rect) {
                    let elem_id = *unsafe { self.element_ids.at(elem_node.element_idx) };
                    candidate_fn(elem_id);
                }

                elem_node_idx = elem_node.next;
            }
        }
    }

    /// Collects all element IDs stored in the tree by visiting all cells.
    #[allow(dead_code)]
    pub(crate) fn collect_ids(&self) -> HashSet<ElementId> {
        let aabb: AABB = self.root_rect.into();
        self.intersect_aabb(&aabb)
    }
}

#[cfg(test)]
pub(crate) fn build_test_tree() -> QuadTree {
    let quad_rect = QuadRect::new(-20, -20, 40, 40);
    let mut tree = QuadTree::new(quad_rect, 1, 1, 1);
    // top-left
    tree.insert(QuadTreeElement::new(1000, AABB::new(-15, -15, -5, -5)))
        .expect("insert should work");
    tree.insert(QuadTreeElement::new(1001, AABB::new(-20, -20, -18, -18)))
        .expect("insert should work");
    // top-right
    tree.insert(QuadTreeElement::new(2000, AABB::new(5, -15, 15, -5)))
        .expect("insert should work");
    // bottom-left
    tree.insert(QuadTreeElement::new(3000, AABB::new(-15, 5, -5, 15)))
        .expect("insert should work");
    // bottom-right
    tree.insert(QuadTreeElement::new(4000, AABB::new(5, 5, 15, 15)))
        .expect("insert should work");
    // center
    tree.insert(QuadTreeElement::new(5000, AABB::new(-5, -5, 5, 5)))
        .expect("insert should work");

    // The depth of 1 limits the tree to four quadrants.
    // Each of the first five elements creates a single reference
    // in each of the quadrants. The "center" element covers
    // all four quadrants, and therefore adds another four references.
    assert_eq!(tree.count_element_references(), 6);

    // Ensure we have the exact elements inserted.
    let inserted_ids = tree.collect_ids();
    assert_eq!(inserted_ids.len(), 6);
    assert!(inserted_ids.contains(&1000));
    assert!(inserted_ids.contains(&1001));
    assert!(inserted_ids.contains(&2000));
    assert!(inserted_ids.contains(&3000));
    assert!(inserted_ids.contains(&4000));
    assert!(inserted_ids.contains(&5000));

    tree
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cleanup_works() {
        let quad_rect = QuadRect::new(-20, -20, 40, 40);
        let mut tree = QuadTree::new(quad_rect, 1, 1, 1);
        // top-left
        tree.insert(QuadTreeElement::new(1000, AABB::new(-15, -15, -5, -5)))
            .expect("insert should work");
        tree.insert(QuadTreeElement::new(1001, AABB::new(-20, -20, -18, -18)))
            .expect("insert should work");
        // top-right
        tree.insert(QuadTreeElement::new(2000, AABB::new(5, -15, 15, -5)))
            .expect("insert should work");
        // bottom-left
        tree.insert(QuadTreeElement::new(3000, AABB::new(-15, 5, -5, 15)))
            .expect("insert should work");
        // bottom-right
        tree.insert(QuadTreeElement::new(4000, AABB::new(5, 5, 15, 15)))
            .expect("insert should work");

        // Similar to index test.
        assert_eq!(tree.collect_ids().len(), 5);

        // center
        tree.insert(QuadTreeElement::new(5000, AABB::new(-5, -5, 5, 5)))
            .expect("insert should work");

        // Similar to index test.
        assert_eq!(tree.collect_ids().len(), 6);
        assert_eq!(tree.count_element_references(), 6);

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
