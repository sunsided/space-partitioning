use crate::intersections::IntersectsWith;
use crate::octree::aabb::AABB;
use crate::octree::centered_aabb::CenteredAABB;
use crate::octree::error::InsertError;
use crate::octree::free_list::{self, FreeList, IndexType};
use crate::octree::node::Node;
use crate::octree::node_data::{NodeData, NodeIndexType};
use crate::octree::node_info::NodeInfo;
use crate::octree::node_list::NodeList;
use crate::octree::oct_rect::OctRect;
use crate::octree::octants::Octants;
use crate::octree::octree_element::OctTreeElementNode;
pub use crate::octree::octree_element::{ElementIdType, OctTreeElement};
use smallvec::SmallVec;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum FindLeafHint {
    Query,
    Mutate,
}

/// A 3D Octree implementation following the same layout as the quadtree variant.
pub struct OctTree<ElementId = u32>
where
    ElementId: ElementIdType,
{
    element_ids: FreeList<ElementId>,
    element_rects: FreeList<AABB>,
    element_nodes: FreeList<OctTreeElementNode>,
    nodes: Vec<Node>,
    root_rect: OctRect,
    free_node: free_list::IndexType,
    max_num_elements: u32,
    smallest_cell_size: u32,
    max_depth: u8,
}

impl<ElementId> OctTree<ElementId>
where
    ElementId: ElementIdType,
{
    pub fn new(
        root_rect: OctRect,
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

    pub fn insert(&mut self, element: OctTreeElement<ElementId>) -> Result<(), InsertError> {
        let element_coords = &element.rect;
        if !self.root_rect.contains(element_coords) {
            return Err(InsertError::OutOfBounds);
        }

        let max_num_elements = self.max_num_elements;

        let element_idx = self.element_ids.insert(element.id);
        let element_rect_idx = self.element_rects.insert(element.rect);
        debug_assert_eq!(element_idx, element_rect_idx);

        let mut to_process: SmallVec<[NodeData; 128]> =
            smallvec::smallvec![self.get_root_node_data()];

        while let Some(node_data) = to_process.pop() {
            let mut leaves = NodeList::default();
            self.find_leaves_aabb_fn(
                node_data,
                element_coords,
                FindLeafHint::Mutate,
                |_rect, nd| {
                    leaves.push_back(nd);
                },
            );

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
                    let element_node_idx = self.element_nodes.insert(OctTreeElementNode {
                        element_idx,
                        next: first_child_or_element,
                    });
                    let node = &mut self.nodes[leaf.index as usize];
                    node.first_child_or_element = element_node_idx;
                    node.element_count += 1;
                } else {
                    self.distribute_elements_to_child_nodes(&leaf);
                    to_process.push(leaf);
                }
            }
        }

        Ok(())
    }

    fn distribute_elements_to_child_nodes(&mut self, parent: &NodeData) {
        let first_child_index = self.ensure_child_nodes_exist();

        let node = &mut self.nodes[parent.index as usize];
        let mut element_node_index = node.get_first_element_node_index();
        node.make_branch(first_child_index);

        let mx = parent.crect.center_x;
        let my = parent.crect.center_y;
        let mz = parent.crect.center_z;

        while element_node_index != free_list::SENTINEL {
            let element_node = unsafe { *self.element_nodes.at(element_node_index) };
            let element = unsafe { *self.element_rects.at(element_node.element_idx) };

            self.assign_element_to_child_nodes(
                mx,
                my,
                mz,
                first_child_index,
                element_node.element_idx,
                &element,
            );

            self.element_nodes.erase(element_node_index);

            element_node_index = element_node.next;
        }
    }

    fn ensure_child_nodes_exist(&mut self) -> u32 {
        if self.free_node != free_list::SENTINEL {
            let node_index = self.free_node;
            let next_free_node = self.nodes[node_index as usize].first_child_or_element;
            self.nodes[node_index as usize] = Node::default();
            self.free_node = next_free_node;
            node_index
        } else {
            let node_index = self.nodes.len() as IndexType;
            self.nodes.push(Node::default());
            for _ in 0..8 {
                self.nodes.push(Node::default());
            }
            node_index
        }
    }

    fn assign_element_to_child_nodes(
        &mut self,
        mx: i32,
        my: i32,
        mz: i32,
        first_child_index: free_list::IndexType,
        element_index: free_list::IndexType,
        element_rect: &AABB,
    ) {
        let insert_left = element_rect.min.x <= mx;
        let insert_right = element_rect.max.x > mx;
        let insert_top = element_rect.min.y <= my;
        let insert_bottom = element_rect.max.y > my;
        let insert_front = element_rect.min.z <= mz;
        let insert_back = element_rect.max.z > mz;

        let covers_many = (insert_top & insert_bottom)
            | (insert_left & insert_right)
            | (insert_front & insert_back);
        if covers_many {
            self.insert_element_in_child_node(first_child_index, element_index);
            return;
        }

        let mut offset = 1;
        if insert_right {
            offset += 1;
        }
        if insert_bottom {
            offset += 2;
        }
        if insert_back {
            offset += 4;
        }
        self.insert_element_in_child_node(first_child_index + offset, element_index);
    }

    fn insert_element_in_child_node(&mut self, child_index: u32, element: free_list::IndexType) {
        let node = &mut self.nodes[child_index as usize];
        let element_node_index = self.element_nodes.insert(OctTreeElementNode {
            element_idx: element,
            next: node.first_child_or_element,
        });
        node.first_child_or_element = element_node_index;
        node.element_count += 1;
    }

    pub fn remove(&mut self, element: &OctTreeElement<ElementId>) -> bool {
        let element_coords = &element.rect;
        let root = self.get_root_node_data();

        let mut found_element_idx = free_list::SENTINEL;

        let mut leaves = NodeList::default();
        self.find_leaves_aabb_fn(root, element_coords, FindLeafHint::Mutate, |_rect, nd| {
            leaves.push_back(nd);
        });

        while !leaves.is_empty() {
            let leaf = leaves.pop_back();
            let leaf_node_data = self.nodes[leaf.index as usize];

            if leaf_node_data.element_count == 0 {
                continue;
            }

            let mut element_found = false;

            let mut element_node_idx = leaf_node_data.first_child_or_element;
            let mut prev_element_node_idx = element_node_idx;
            let mut new_first_child_or_element = element_node_idx;

            while element_node_idx != free_list::SENTINEL {
                let elem_node = *unsafe { self.element_nodes.at(element_node_idx) };
                let elem_id = unsafe { self.element_ids.at(elem_node.element_idx) };

                if *elem_id == element.id {
                    debug_assert!(!element_found);
                    element_found = true;

                    if leaf_node_data.first_child_or_element == element_node_idx {
                        new_first_child_or_element = elem_node.next;
                    }

                    if element_node_idx != prev_element_node_idx {
                        unsafe { self.element_nodes.at_mut(prev_element_node_idx) }.next =
                            elem_node.next;
                    }

                    self.element_nodes.erase(element_node_idx);
                    debug_assert!(
                        found_element_idx == free_list::SENTINEL
                            || found_element_idx == elem_node.element_idx
                    );
                    found_element_idx = elem_node.element_idx;
                }

                prev_element_node_idx = element_node_idx;
                element_node_idx = elem_node.next;

                #[cfg(not(debug_assertions))]
                if element_found {
                    break;
                }
            }

            let node = &mut self.nodes[leaf.index as usize];
            node.first_child_or_element = new_first_child_or_element;

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

    fn find_leaves_aabb_fn<F>(
        &self,
        root: NodeData,
        rect: &AABB,
        hint: FindLeafHint,
        mut callback: F,
    ) where
        F: FnMut(&AABB, NodeData),
    {
        let mut to_process = NodeList::default();
        to_process.push_back(root);

        while !to_process.is_empty() {
            let nd = to_process.pop_back();

            if self.nodes[nd.index as usize].is_leaf() {
                callback(rect, nd);
                continue;
            }

            let fc = self.nodes[nd.index as usize].get_first_child_node_index();

            let octants = nd.crect.explore_octants_aabb(rect);
            Self::collect_relevant_octants(&mut to_process, &nd, fc, octants, hint)
        }
    }

    fn find_leaves_generic_fn<T, F>(&self, root: NodeData, element: &T, mut callback: F)
    where
        T: IntersectsWith<AABB>,
        F: FnMut(NodeData),
    {
        let mut to_process = NodeList::default();
        to_process.push_back(root);

        while !to_process.is_empty() {
            let nd = to_process.pop_back();

            if self.nodes[nd.index as usize].is_leaf() {
                callback(nd);
                continue;
            }

            let fc = self.nodes[nd.index as usize].get_first_child_node_index();

            let octants = nd.crect.explore_octants_generic(element);
            Self::collect_relevant_octants(&mut to_process, &nd, fc, octants, FindLeafHint::Query)
        }
    }

    pub fn visit_leaves<F>(&self, mut visit: F)
    where
        F: FnMut(NodeInfo),
    {
        let mut to_process = NodeList::default();
        to_process.push_back(self.get_root_node_data());

        while !to_process.is_empty() {
            let nd = to_process.pop_back();

            let is_this_node = nd.index > 0 && ((nd.index - 1) % 9) == 0;
            if is_this_node {
                debug_assert!(self.nodes[nd.index as usize].is_leaf());
                continue;
            }

            let node = &self.nodes[nd.index as usize];
            if node.is_leaf() {
                visit(NodeInfo::from(nd, node.element_count));
                continue;
            }

            let fc = self.nodes[nd.index as usize].get_first_child_node_index();
            Self::collect_relevant_octants(
                &mut to_process,
                &nd,
                fc,
                Octants::all(),
                FindLeafHint::Query,
            )
        }
    }

    #[inline]
    fn collect_relevant_octants(
        to_process: &mut NodeList,
        nd: &NodeData,
        first_child_id: u32,
        octants: Octants,
        hint: FindLeafHint,
    ) {
        let split_octants = nd.crect.split_octants();

        match hint {
            FindLeafHint::Query => Self::collect_relevant_octants_for_query(
                to_process,
                nd.depth,
                first_child_id,
                octants,
                &split_octants,
            ),
            FindLeafHint::Mutate => Self::collect_relevant_octants_for_mutation(
                to_process,
                nd.depth,
                first_child_id,
                octants,
                &split_octants,
            ),
        }
    }

    fn collect_relevant_octants_for_mutation(
        to_process: &mut NodeList,
        depth: u8,
        first_child_id: u32,
        octants: Octants,
        split_octants: &[CenteredAABB; 9],
    ) {
        let offset = octants.mutation_index();
        debug_assert!(offset <= 8);

        let can_split = offset > 0;

        let child_depth = depth + (1 - octants.this() as u8);

        to_process.push_back(NodeData::new(
            split_octants[offset as usize],
            first_child_id + offset,
            child_depth,
            can_split,
        ));
    }

    fn collect_relevant_octants_for_query(
        to_process: &mut NodeList,
        depth: u8,
        first_child_id: u32,
        octants: Octants,
        split_octants: &[CenteredAABB; 9],
    ) {
        let child_depth = depth + 1;

        for offset in (1..=8).rev() {
            if octants.at(offset as u32) {
                to_process.push_back(NodeData::new(
                    split_octants[offset as usize],
                    first_child_id + offset,
                    child_depth,
                    true,
                ));
            }
        }

        to_process.push_back(NodeData::new(
            split_octants[0],
            first_child_id,
            depth,
            false,
        ));
    }

    pub fn cleanup(&mut self) -> bool {
        if self.nodes[0].is_leaf() {
            return false;
        }

        let mut tree_compacted = false;
        let mut to_process: SmallVec<[NodeIndexType; 128]> = smallvec::smallvec![0];

        while let Some(node_index) = to_process.pop() {
            let first_child_index = self.nodes[node_index as usize].get_first_child_node_index();

            let mut num_empty_leaves = 0usize;
            for j in 0..9 {
                let child_index = first_child_index + j;
                let child = &self.nodes[child_index as usize];

                if child.is_empty() {
                    num_empty_leaves += 1;
                } else if child.is_branch() {
                    to_process.push(child_index);
                }
            }

            if num_empty_leaves == 9 {
                self.nodes[first_child_index as usize].first_child_or_element = self.free_node;
                self.free_node = first_child_index;

                self.nodes[node_index as usize].make_empty_leaf();

                tree_compacted = true;
            }
        }

        tree_compacted
    }

    #[allow(dead_code)]
    pub(crate) fn count_element_references(&self) -> usize {
        let mut to_process: SmallVec<[usize; 128]> = smallvec::smallvec![0];
        let mut count = 0usize;
        while let Some(index) = to_process.pop() {
            let node = &self.nodes[index];
            if node.is_branch() {
                for j in 0..9 {
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

    #[inline]
    pub fn intersect_aabb(&self, rect: &AABB) -> Vec<ElementId> {
        let root = self.get_root_node_data();
        let mut node_set = Vec::with_capacity(128);

        self.find_leaves_aabb_fn(root, rect, FindLeafHint::Query, |rect, nd| {
            self.intersect_from_leaf(rect, nd, &mut |id| {
                node_set.push(id);
            });
        });

        node_set
    }

    #[inline]
    pub fn intersect_aabb_fn<F>(&self, rect: &AABB, mut candidate_fn: F)
    where
        F: FnMut(ElementId),
    {
        let root = self.get_root_node_data();
        self.find_leaves_aabb_fn(root, rect, FindLeafHint::Query, move |rect, nd| {
            self.intersect_from_leaf(rect, nd, &mut |id| {
                candidate_fn(id);
            });
        });
    }

    #[inline]
    pub fn intersect_generic<T>(&self, element: &T) -> Vec<ElementId>
    where
        T: IntersectsWith<AABB>,
    {
        let root = self.get_root_node_data();
        let mut node_set = Vec::with_capacity(128);

        self.find_leaves_generic_fn(root, element, |nd| {
            self.intersect_from_leaf(element, nd, &mut |id| {
                node_set.push(id);
            });
        });

        node_set
    }

    #[inline]
    pub fn intersect_generic_fn<T, F>(&self, element: &T, mut candidate_fn: F)
    where
        T: IntersectsWith<AABB>,
        F: FnMut(ElementId),
    {
        let root = self.get_root_node_data();
        self.find_leaves_generic_fn(root, element, move |nd| {
            self.intersect_from_leaf(element, nd, &mut |id| {
                candidate_fn(id);
            });
        });
    }

    #[inline]
    fn intersect_from_leaf<T, F>(&self, element: &T, leaf_data: NodeData, mut candidate_fn: F)
    where
        T: IntersectsWith<AABB>,
        F: FnMut(ElementId),
    {
        let leaf = self.nodes[leaf_data.index as usize];
        debug_assert!(leaf.is_leaf());

        let mut elem_node_idx = leaf.first_child_or_element;
        while elem_node_idx != free_list::SENTINEL {
            let elem_node = unsafe { self.element_nodes.at(elem_node_idx) };
            let elem_rect = unsafe { self.element_rects.at(elem_node.element_idx) };

            if element.intersects_with(elem_rect) {
                let elem_id = *unsafe { self.element_ids.at(elem_node.element_idx) };
                candidate_fn(elem_id);
            }

            elem_node_idx = elem_node.next;
        }
    }

    #[allow(dead_code)]
    pub(crate) fn collect_ids(&self) -> Vec<ElementId> {
        let aabb: AABB = self.root_rect.into();
        self.intersect_aabb(&aabb)
    }
}

impl<ElementId> Default for OctTree<ElementId>
where
    ElementId: ElementIdType,
{
    fn default() -> Self {
        Self::new(OctRect::default(), 8, 16, 1)
    }
}

#[cfg(test)]
pub(crate) fn build_test_tree() -> OctTree {
    let oct_rect = OctRect::new(-20, -20, -20, 40, 40, 40);
    let mut tree = OctTree::new(oct_rect, 1, 1, 1);

    tree.insert(OctTreeElement::new(
        1000,
        AABB::new(-15, -15, -15, -5, -5, -5),
    ))
    .expect("insert should work");
    tree.insert(OctTreeElement::new(
        1001,
        AABB::new(-20, -20, -20, -18, -18, -18),
    ))
    .expect("insert should work");

    tree.insert(OctTreeElement::new(
        2000,
        AABB::new(5, -15, -15, 15, -5, -5),
    ))
    .expect("insert should work");

    tree.insert(OctTreeElement::new(
        3000,
        AABB::new(-15, 5, -15, -5, 15, -5),
    ))
    .expect("insert should work");

    tree.insert(OctTreeElement::new(4000, AABB::new(5, 5, -15, 15, 15, -5)))
        .expect("insert should work");

    tree.insert(OctTreeElement::new(5000, AABB::new(-5, -5, -5, 5, 5, 5)))
        .expect("insert should work");

    assert_eq!(tree.count_element_references(), 6);

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
        let oct_rect = OctRect::new(-20, -20, -20, 40, 40, 40);
        let mut tree = OctTree::new(oct_rect, 1, 1, 1);

        tree.insert(OctTreeElement::new(
            1000,
            AABB::new(-15, -15, -15, -5, -5, -5),
        ))
        .expect("insert should work");
        tree.insert(OctTreeElement::new(
            1001,
            AABB::new(-20, -20, -20, -18, -18, -18),
        ))
        .expect("insert should work");
        tree.insert(OctTreeElement::new(
            2000,
            AABB::new(5, -15, -15, 15, -5, -5),
        ))
        .expect("insert should work");
        tree.insert(OctTreeElement::new(
            3000,
            AABB::new(-15, 5, -15, -5, 15, -5),
        ))
        .expect("insert should work");
        tree.insert(OctTreeElement::new(4000, AABB::new(5, 5, -15, 15, 15, -5)))
            .expect("insert should work");

        assert_eq!(tree.collect_ids().len(), 5);

        tree.insert(OctTreeElement::new(5000, AABB::new(-5, -5, -5, 5, 5, 5)))
            .expect("insert should work");

        assert_eq!(tree.collect_ids().len(), 6);
        assert_eq!(tree.count_element_references(), 6);

        assert!(tree.remove(&OctTreeElement::new(
            1000,
            AABB::new(-15, -15, -15, -5, -5, -5)
        )));
        assert!(tree.remove(&OctTreeElement::new(
            1001,
            AABB::new(-20, -20, -20, -18, -18, -18)
        )));
        assert!(tree.remove(&OctTreeElement::new(
            2000,
            AABB::new(5, -15, -15, 15, -5, -5)
        )));
        assert!(tree.remove(&OctTreeElement::new(
            3000,
            AABB::new(-15, 5, -15, -5, 15, -5)
        )));
        assert!(tree.remove(&OctTreeElement::new(4000, AABB::new(5, 5, -15, 15, 15, -5))));
        assert!(tree.remove(&OctTreeElement::new(5000, AABB::new(-5, -5, -5, 5, 5, 5))));
        assert_eq!(tree.collect_ids().len(), 0);
        assert_eq!(tree.count_element_references(), 0);

        assert!(tree.nodes[0].is_branch());
        assert_eq!(tree.nodes[0].first_child_or_element, 1);

        assert!(tree.cleanup());

        assert!(tree.nodes[0].is_leaf());
        assert_eq!(tree.nodes[0].element_count, 0);
    }
}
