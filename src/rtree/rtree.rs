//! Core R-Tree implementation.
//!
//! This module provides the main `RTree` type and its query APIs.
//! It is intentionally low-level; higher-level examples live in `examples/`.

use crate::intersections::IntersectsWith;
use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::node_traits::HasBoundingBox;
use crate::rtree::nodes::prelude::*;
use crate::rtree::nodes::rtree_leaf::IndexRecordEntry;
use crate::rtree::nodes::rtree_node::{ChildPointer, NodeData};
use crate::rtree::splitting_strategies::linear_cost_split::LinearCostSplitting;
use crate::rtree::splitting_strategies::SplittingStrategy;
use arrayvec::ArrayVec;
use std::cmp::Ordering;

/// The R-Tree
///
/// ## Type parameters
/// * `T` - The coordinate type.
/// * `N` - The number of dimensions per coordinate.
/// * `M` - The maximum number of elements to store per leaf node.
/// * `TupleIdentifier` - The type used to identify a tuple in application code.
///
/// # Invariants
/// - `root` always exists and contains all entries.
/// - Each node's bounding box encloses its children or entries.
#[derive(Debug)]
pub struct RTree<T, const N: usize, const M: usize, TupleIdentifier = usize>
where
    T: DimensionType,
{
    /// Root node of the tree. Always present; may be a leaf for small trees.
    root: RTreeNode<T, N, M, TupleIdentifier>,
    /// Split strategy used when a node overflows.
    split_strategy: LinearCostSplitting,
}

#[derive(Debug, Clone, Copy)]
pub enum BulkLoadStrategy {
    /// Sort-Tile-Recursive bulk-loading (fast, lower-quality than insertion for some distributions).
    Str,
    /// Insert entries one-by-one using the standard insertion algorithm.
    Insertion,
}

/// Path to a leaf node, used during insert/adjust operations.
struct LeafPath {
    /// Indices of child pointers from the root to the parent of the leaf-level node.
    node_indices: Vec<usize>,
    /// Index of the leaf child within the leaf-level node.
    leaf_index: Option<usize>,
}

/// Path to a specific entry in a leaf node, used during removal.
struct LeafEntryPath {
    /// Indices of child pointers from the root to the parent of the leaf-level node.
    node_indices: Vec<usize>,
    /// Index of the leaf child within the leaf-level node.
    leaf_child_index: usize,
    /// Index of the entry within the leaf child.
    entry_index: usize,
}

impl<T, const N: usize, const M: usize, TupleIdentifier> RTree<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    /// Inserts an element into the tree.
    ///
    /// The entry's bounding box must satisfy `start <= end` for all dimensions.
    ///
    /// # Example
    /// ```
    /// use space_partitioning::rtree::{BoundingBox, RTree};
    ///
    /// let mut tree: RTree<f32, 2, 4> = RTree::default();
    /// tree.insert(1, BoundingBox::from([0.0..=1.0, 2.0..=3.0]));
    /// assert!(!tree.is_empty());
    /// ```
    pub fn insert(&mut self, id: TupleIdentifier, bb: BoundingBox<T, N>) {
        // Citing https://iq.opengenus.org/r-tree/
        //
        // 1. Find position for new record:
        //      Invoke `choose_leaf` to select leaf node L in which to place the entry.
        let entry = IndexRecordEntry::new(id, bb);
        let leaf_path = self.choose_leaf(&entry.bb);
        let split_strategy = self.split_strategy.clone();

        // 2. Add record to leaf node.
        //      If L has room for another entry then add E, else
        //      invoke `split_node` to obtain L and LL (current leaf and new leaf containing all old entries of L)
        let split_node = {
            let leaf_node = self.node_at_path_mut(&leaf_path.node_indices);
            match &mut leaf_node.node_data {
                NodeData::Leaf(leaves) => Self::insert_into_leaf_node(
                    &split_strategy,
                    leaves,
                    leaf_path.leaf_index,
                    entry,
                ),
                NodeData::NonLeaf(_) => {
                    unreachable!("choose_leaf always descends to a leaf-level node")
                }
            }
        };

        // 3. Propagate changes upward
        //      Invoke `adjust_tree` on L also passing LL if split was performed.
        // 4. Grow the tree taller
        //      If node split propagation caused the root to split, create a new root
        //      whose children are the two resulting nodes.
        if let Some(new_node) = self.adjust_tree(&leaf_path.node_indices, split_node) {
            self.grow_root(new_node);
        }
    }

    /// Finds entries whose bounding boxes intersect the query box.
    ///
    /// # Example
    /// ```
    /// use space_partitioning::rtree::{BoundingBox, RTree};
    ///
    /// let mut tree: RTree<f32, 2, 4> = RTree::default();
    /// tree.insert(1, BoundingBox::from([0.0..=1.0, 0.0..=1.0]));
    /// tree.insert(2, BoundingBox::from([2.0..=3.0, 2.0..=3.0]));
    ///
    /// let hits = tree.query_intersects(&BoundingBox::from([0.5..=2.5, 0.5..=2.5]));
    /// let mut ids: Vec<_> = hits.iter().map(|entry| entry.id).collect();
    /// ids.sort();
    /// assert_eq!(ids, vec![1, 2]);
    /// ```
    pub fn query_intersects<'a>(
        &'a self,
        bb: &BoundingBox<T, N>,
    ) -> Vec<&'a IndexRecordEntry<T, N, TupleIdentifier>> {
        self.query_intersects_generic(bb)
    }

    /// Finds entries whose bounding boxes intersect the query element.
    ///
    /// # Example
    /// ```
    /// use space_partitioning::intersections::IntersectsWith;
    /// use space_partitioning::rtree::{BoundingBox, RTree};
    ///
    /// struct Ray2 {
    ///     origin: [f32; 2],
    ///     inv_dir: [f32; 2],
    /// }
    ///
    /// impl Ray2 {
    ///     fn new(origin: [f32; 2], dir: [f32; 2]) -> Self {
    ///         Self {
    ///             origin,
    ///             inv_dir: [1.0 / dir[0], 1.0 / dir[1]],
    ///         }
    ///     }
    /// }
    ///
    /// impl IntersectsWith<BoundingBox<f32, 2>> for Ray2 {
    ///     fn intersects_with(&self, other: &BoundingBox<f32, 2>) -> bool {
    ///         let mut tmin = f32::NEG_INFINITY;
    ///         let mut tmax = f32::INFINITY;
    ///         for i in 0..2 {
    ///             let start = other.dims[i].start;
    ///             let end = other.dims[i].end;
    ///             let t1 = (start - self.origin[i]) * self.inv_dir[i];
    ///             let t2 = (end - self.origin[i]) * self.inv_dir[i];
    ///             let (t1, t2) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
    ///             tmin = tmin.max(t1);
    ///             tmax = tmax.min(t2);
    ///             if tmin > tmax {
    ///                 return false;
    ///             }
    ///         }
    ///         tmax >= 0.0
    ///     }
    /// }
    ///
    /// let mut tree: RTree<f32, 2, 4> = RTree::default();
    /// tree.insert(1, BoundingBox::from([0.0..=1.0, 0.0..=1.0]));
    /// tree.insert(2, BoundingBox::from([2.0..=3.0, 0.0..=1.0]));
    ///
    /// let ray = Ray2::new([-1.0, 0.5], [1.0, 0.0]);
    /// let hits = tree.query_intersects_generic(&ray);
    /// let mut ids: Vec<_> = hits.iter().map(|entry| entry.id).collect();
    /// ids.sort();
    /// assert_eq!(ids, vec![1, 2]);
    /// ```
    pub fn query_intersects_generic<'a, Q>(
        &'a self,
        element: &Q,
    ) -> Vec<&'a IndexRecordEntry<T, N, TupleIdentifier>>
    where
        Q: IntersectsWith<BoundingBox<T, N>>,
    {
        let mut matches = Vec::new();
        self.query_node_intersects_generic(&self.root, element, &mut matches);
        matches
    }

    /// Calls a function for each entry whose bounding box intersects the query element.
    ///
    /// # Example
    /// ```
    /// use space_partitioning::intersections::IntersectsWith;
    /// use space_partitioning::rtree::{BoundingBox, RTree};
    ///
    /// struct Query;
    ///
    /// impl IntersectsWith<BoundingBox<f32, 2>> for Query {
    ///     fn intersects_with(&self, other: &BoundingBox<f32, 2>) -> bool {
    ///         other.dims[0].start <= 0.5 && 0.5 <= other.dims[0].end
    ///     }
    /// }
    ///
    /// let mut tree: RTree<f32, 2, 4> = RTree::default();
    /// tree.insert(1, BoundingBox::from([0.0..=1.0, 0.0..=1.0]));
    /// tree.insert(2, BoundingBox::from([2.0..=3.0, 0.0..=1.0]));
    ///
    /// let mut ids = Vec::new();
    /// tree.query_intersects_generic_fn(&Query, |entry| ids.push(entry.id));
    /// ids.sort();
    /// assert_eq!(ids, vec![1]);
    /// ```
    pub fn query_intersects_generic_fn<'a, Q, F>(&'a self, element: &Q, mut candidate_fn: F)
    where
        Q: IntersectsWith<BoundingBox<T, N>>,
        F: FnMut(&'a IndexRecordEntry<T, N, TupleIdentifier>),
    {
        self.query_node_intersects_generic_fn(&self.root, element, &mut candidate_fn);
    }

    /// Finds entries whose bounding boxes are fully contained by the query box.
    ///
    /// # Example
    /// ```
    /// use space_partitioning::rtree::{BoundingBox, RTree};
    ///
    /// let mut tree: RTree<f32, 2, 4> = RTree::default();
    /// tree.insert(1, BoundingBox::from([0.0..=1.0, 0.0..=1.0]));
    /// tree.insert(2, BoundingBox::from([2.0..=3.0, 2.0..=3.0]));
    ///
    /// let hits = tree.query_contains(&BoundingBox::from([0.0..=2.5, 0.0..=2.5]));
    /// let mut ids: Vec<_> = hits.iter().map(|entry| entry.id).collect();
    /// ids.sort();
    /// assert_eq!(ids, vec![1]);
    /// ```
    pub fn query_contains<'a>(
        &'a self,
        bb: &BoundingBox<T, N>,
    ) -> Vec<&'a IndexRecordEntry<T, N, TupleIdentifier>> {
        let mut matches = Vec::new();
        self.query_node_contains(&self.root, bb, &mut matches);
        matches
    }

    /// Removes an entry by id and exact bounding box.
    ///
    /// # Example
    /// ```
    /// use space_partitioning::rtree::{BoundingBox, RTree};
    ///
    /// let mut tree: RTree<f32, 2, 4> = RTree::default();
    /// let bb = BoundingBox::from([0.0..=1.0, 0.0..=1.0]);
    /// tree.insert(1, bb.clone());
    /// assert!(tree.remove(&1, &bb));
    /// assert!(tree.is_empty());
    /// ```
    pub fn remove(&mut self, id: &TupleIdentifier, bb: &BoundingBox<T, N>) -> bool
    where
        TupleIdentifier: PartialEq,
    {
        // Removal matches on both identifier and exact bounding box.
        let Some(path) = self.find_leaf_entry_path(id, bb) else {
            return false;
        };

        let leaf_node = self.node_at_path_mut(&path.node_indices);
        match &mut leaf_node.node_data {
            NodeData::Leaf(children) => {
                let leaf_child = &mut children[path.leaf_child_index];
                leaf_child.pointer.entries.remove(path.entry_index);
                if leaf_child.pointer.entries.is_empty() {
                    children.remove(path.leaf_child_index);
                } else {
                    leaf_child.recompute_bb();
                }
            }
            NodeData::NonLeaf(_) => unreachable!("expected leaf-level node for removal"),
        }

        let reinsert = self.condense_tree(&path.node_indices);
        for entry in reinsert {
            self.insert(entry.id, entry.bb);
        }
        true
    }

    /// Bulk-load entries using the default `STR` strategy.
    ///
    /// # Example
    /// ```
    /// use space_partitioning::rtree::{BoundingBox, RTree};
    ///
    /// let items = vec![
    ///     (1, BoundingBox::from([0.0..=1.0, 0.0..=1.0])),
    ///     (2, BoundingBox::from([2.0..=3.0, 2.0..=3.0])),
    /// ];
    /// let tree: RTree<f32, 2, 4> = RTree::bulk_load(items);
    /// assert!(!tree.is_empty());
    /// ```
    pub fn bulk_load<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = (TupleIdentifier, BoundingBox<T, N>)>,
    {
        Self::bulk_load_with_strategy(entries, BulkLoadStrategy::Str)
    }

    /// Bulk-load entries with an explicit strategy.
    pub fn bulk_load_with_strategy<I>(entries: I, strategy: BulkLoadStrategy) -> Self
    where
        I: IntoIterator<Item = (TupleIdentifier, BoundingBox<T, N>)>,
    {
        let items = entries
            .into_iter()
            .map(|(id, bb)| IndexRecordEntry::new(id, bb))
            .collect::<Vec<_>>();
        Self::bulk_load_entries_with_strategy(items, strategy)
    }

    /// Bulk-load pre-built entries using the default `STR` strategy.
    pub fn bulk_load_entries<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = IndexRecordEntry<T, N, TupleIdentifier>>,
    {
        Self::bulk_load_entries_with_strategy(entries, BulkLoadStrategy::Str)
    }

    /// Bulk-load pre-built entries with an explicit strategy.
    pub fn bulk_load_entries_with_strategy<I>(entries: I, strategy: BulkLoadStrategy) -> Self
    where
        I: IntoIterator<Item = IndexRecordEntry<T, N, TupleIdentifier>>,
    {
        // STR constructs a balanced tree in a bottom-up manner.
        let mut entries = entries.into_iter().collect::<Vec<_>>();
        if entries.is_empty() {
            return Self::default();
        }

        if matches!(strategy, BulkLoadStrategy::Insertion) {
            let mut tree = Self::default();
            for entry in entries {
                tree.insert(entry.id, entry.bb);
            }
            return tree;
        }

        let leaf_children = Self::build_leaf_children(&mut entries);
        let mut current_level = Self::build_leaf_level_nodes(leaf_children);

        while current_level.len() > 1 {
            let child_pointers = current_level
                .into_iter()
                .map(|node| ChildPointer {
                    bb: node.to_bb(),
                    pointer: Box::new(node),
                })
                .collect::<Vec<_>>();
            current_level = Self::build_non_leaf_level_nodes(child_pointers);
        }

        let root = current_level.pop().unwrap_or_default();
        Self {
            root,
            split_strategy: LinearCostSplitting::default(),
        }
    }

    /// Finds the nearest entry to the provided point.
    ///
    /// # Example
    /// ```
    /// use space_partitioning::rtree::{BoundingBox, RTree};
    ///
    /// let mut tree: RTree<f32, 2, 4> = RTree::default();
    /// tree.insert(1, BoundingBox::from([0.0..=1.0, 0.0..=1.0]));
    /// tree.insert(2, BoundingBox::from([2.0..=3.0, 2.0..=3.0]));
    ///
    /// let nearest = tree.nearest_neighbor([0.1, 0.2]).expect("nearest");
    /// assert_eq!(nearest.id, 1);
    /// ```
    pub fn nearest_neighbor(
        &self,
        point: [T; N],
    ) -> Option<&IndexRecordEntry<T, N, TupleIdentifier>> {
        // Prunes branches whose bounding boxes are farther than the current best.
        let mut best: Option<(&IndexRecordEntry<T, N, TupleIdentifier>, T)> = None;
        self.nearest_in_node(&self.root, &point, &mut best);
        best.map(|(entry, _)| entry)
    }

    /// Selects a leaf node in which to place a new entry.
    fn choose_leaf(&self, bb: &BoundingBox<T, N>) -> LeafPath {
        // Citing https://iq.opengenus.org/r-tree/
        //
        // 1. Initialize
        //      Set N to be the root node
        let mut current = &self.root;
        let mut node_indices = Vec::new();

        // 2. Leaf check
        //      If N is a leaf, return N
        loop {
            match &current.node_data {
                NodeData::Leaf(children) => {
                    let leaf_index = if children.is_empty() {
                        None
                    } else {
                        Some(Self::choose_child_index(children.as_slice(), bb))
                    };
                    return LeafPath {
                        node_indices,
                        leaf_index,
                    };
                }
                NodeData::NonLeaf(children) => {
                    // 3. Choose subtree
                    //      Let F be the entry in N whose rectangle F1
                    //      needs least enlargement to include E1. Resolve ties by choosing
                    //      the entry with the rectangle of the smallest area.
                    // 4. Descend until leaf is reached
                    //      Set N to be child node pointed to by Fp and repeat from step 2.
                    let idx = Self::choose_child_index(children.as_slice(), bb);
                    node_indices.push(idx);
                    current = &children[idx].pointer;
                }
            }
        }
    }

    /// Finds the path to a specific entry by id and bounding box.
    fn find_leaf_entry_path(
        &self,
        id: &TupleIdentifier,
        bb: &BoundingBox<T, N>,
    ) -> Option<LeafEntryPath>
    where
        TupleIdentifier: PartialEq,
    {
        let mut node_path = Vec::new();
        self.find_leaf_entry_path_in_node(&self.root, id, bb, &mut node_path)
    }

    /// Recursive helper for locating a specific entry in a subtree.
    fn find_leaf_entry_path_in_node(
        &self,
        node: &RTreeNode<T, N, M, TupleIdentifier>,
        id: &TupleIdentifier,
        bb: &BoundingBox<T, N>,
        node_path: &mut Vec<usize>,
    ) -> Option<LeafEntryPath>
    where
        TupleIdentifier: PartialEq,
    {
        match &node.node_data {
            NodeData::Leaf(children) => {
                // Leaf-level scan: match by id and exact bounding box.
                for (leaf_idx, child) in children.iter().enumerate() {
                    if !child.bb.contains(bb) {
                        continue;
                    }
                    for (entry_idx, entry) in child.pointer.entries.iter().enumerate() {
                        if &entry.id == id && &entry.bb == bb {
                            return Some(LeafEntryPath {
                                node_indices: node_path.clone(),
                                leaf_child_index: leaf_idx,
                                entry_index: entry_idx,
                            });
                        }
                    }
                }
                None
            }
            NodeData::NonLeaf(children) => {
                for (idx, child) in children.iter().enumerate() {
                    if !child.bb.contains(bb) {
                        continue;
                    }
                    node_path.push(idx);
                    if let Some(found) =
                        self.find_leaf_entry_path_in_node(&child.pointer, id, bb, node_path)
                    {
                        return Some(found);
                    }
                    node_path.pop();
                }
                None
            }
        }
    }

    /// Adjusts parent bounding boxes and handles split propagation upward.
    fn adjust_tree(
        &mut self,
        node_path: &[usize],
        mut split_node: Option<RTreeNode<T, N, M, TupleIdentifier>>,
    ) -> Option<RTreeNode<T, N, M, TupleIdentifier>> {
        // Citing https://iq.opengenus.org/r-tree/
        //
        // 1. Initialize
        //      Set N=L (L being the leaf node)
        //      If L was split previously, set NN to be the resulting second node.
        // 2. Check if done
        //      If N is the root, stop
        // 3. Adjust covering rectangle in parent entry
        //      Let P be the parent node of N, and let EN be N's entry in P.
        //      Adjust EN so that it tightly encloses all entry rectangles in N.
        // 4. Propagate node split upward
        //      If N has a partner NN resulting from an earlier split,
        //      create a new entry ENN with ENN pointing to NN and ENN enclosing all
        //      rectangles in NN. Add ENN to P if there is room, otherwise invoke `split_node`
        //      to produce P and PP containing ENN and all P's old entries.
        // 5. Move up to the next level
        //      Set N=P and set NN=PP if a split occurred. Repeat from step 2.
        let split_strategy = self.split_strategy.clone();
        let mut depth = node_path.len();

        while depth > 0 {
            let parent_path = &node_path[..depth - 1];
            let child_index = node_path[depth - 1];
            let parent = self.node_at_path_mut(parent_path);

            match &mut parent.node_data {
                NodeData::NonLeaf(children) => {
                    // Update the bounding box of the child we just modified.
                    {
                        let child = &mut children[child_index];
                        child.recompute_bb();
                    }

                    if let Some(new_node) = split_node {
                        let new_child = ChildPointer {
                            bb: new_node.to_bb(),
                            pointer: Box::new(new_node),
                        };

                        if children.len() < M {
                            children.push(new_child);
                            split_node = None;
                        } else {
                            // Parent overflow: split and propagate upward.
                            let parent_bb = children.as_slice().to_bb();
                            let split_result =
                                split_strategy.split(&parent_bb, children, new_child);
                            let second_entries = split_result.second.entries;
                            *children = split_result.first.entries;
                            split_node = Some(RTreeNode {
                                node_data: NodeData::NonLeaf(second_entries),
                            });
                        }
                    }
                }
                NodeData::Leaf(_) => unreachable!("parent nodes must be non-leaf"),
            }

            depth -= 1;
        }

        split_node
    }

    /// Returns `true` if the tree contains no entries.
    ///
    /// # Example
    /// ```
    /// use space_partitioning::rtree::RTree;
    ///
    /// let tree: RTree<f32, 2, 4> = RTree::default();
    /// assert!(tree.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    /// Returns a mutable node reference for a path of child indices from the root.
    fn node_at_path_mut(
        &mut self,
        node_path: &[usize],
    ) -> &mut RTreeNode<T, N, M, TupleIdentifier> {
        let mut current = &mut self.root;
        for &idx in node_path {
            let child = match &mut current.node_data {
                NodeData::NonLeaf(children) => &mut children[idx],
                NodeData::Leaf(_) => panic!("expected non-leaf while descending"),
            };
            current = &mut child.pointer;
        }
        current
    }

    /// Condenses the tree after deletions, returning entries to reinsert.
    fn condense_tree(
        &mut self,
        node_path: &[usize],
    ) -> Vec<IndexRecordEntry<T, N, TupleIdentifier>> {
        let mut reinsert = Vec::new();
        let mut depth = node_path.len();

        while depth > 0 {
            let parent_path = &node_path[..depth - 1];
            let child_index = node_path[depth - 1];
            let parent = self.node_at_path_mut(parent_path);

            match &mut parent.node_data {
                NodeData::NonLeaf(children) => {
                    if children[child_index].pointer.is_underfull() {
                        // Remove underfull subtree and reinsert its entries.
                        let removed = children.remove(child_index);
                        reinsert.extend(Self::collect_entries_from_node(*removed.pointer));
                    } else {
                        children[child_index].recompute_bb();
                    }
                }
                NodeData::Leaf(_) => unreachable!("parent nodes must be non-leaf"),
            }

            depth -= 1;
        }

        self.condense_root();
        reinsert
    }

    /// Collapses the root if it has a single child.
    fn condense_root(&mut self) {
        loop {
            match &mut self.root.node_data {
                NodeData::NonLeaf(children) => {
                    if children.len() == 1 {
                        // Collapse a single-child root to keep the tree shallow.
                        let only = children.remove(0);
                        self.root = *only.pointer;
                        continue;
                    }
                }
                NodeData::Leaf(_) => {}
            }
            break;
        }
    }

    /// Chooses the child that minimizes area enlargement for insertion.
    fn choose_child_index<TNode>(
        children: &[ChildPointer<T, N, TNode>],
        bb: &BoundingBox<T, N>,
    ) -> usize {
        debug_assert!(!children.is_empty());
        // Choose the child that requires the least area enlargement.
        let mut best_idx = 0;
        let mut best = children[0].bb.get_grown(bb);

        for (idx, child) in children.iter().enumerate().skip(1) {
            let grown = child.bb.get_grown(bb);
            if grown.area_increase < best.area_increase
                || (grown.area_increase == best.area_increase && grown.area < best.area)
            {
                best = grown;
                best_idx = idx;
            }
        }

        best_idx
    }

    /// Traverses nodes and collects entries intersecting a generic query element.
    fn query_node_intersects_generic<'a, Q>(
        &'a self,
        node: &'a RTreeNode<T, N, M, TupleIdentifier>,
        element: &Q,
        matches: &mut Vec<&'a IndexRecordEntry<T, N, TupleIdentifier>>,
    ) where
        Q: IntersectsWith<BoundingBox<T, N>>,
    {
        match &node.node_data {
            NodeData::Leaf(children) => {
                for child in children.iter() {
                    if !element.intersects_with(&child.bb) {
                        continue;
                    }
                    for entry in child.pointer.entries.iter() {
                        if element.intersects_with(&entry.bb) {
                            matches.push(entry);
                        }
                    }
                }
            }
            NodeData::NonLeaf(children) => {
                for child in children.iter() {
                    if element.intersects_with(&child.bb) {
                        self.query_node_intersects_generic(&child.pointer, element, matches);
                    }
                }
            }
        }
    }

    /// Traverses nodes and calls back for each entry intersecting a query element.
    fn query_node_intersects_generic_fn<'a, Q, F>(
        &'a self,
        node: &'a RTreeNode<T, N, M, TupleIdentifier>,
        element: &Q,
        candidate_fn: &mut F,
    ) where
        Q: IntersectsWith<BoundingBox<T, N>>,
        F: FnMut(&'a IndexRecordEntry<T, N, TupleIdentifier>),
    {
        match &node.node_data {
            NodeData::Leaf(children) => {
                for child in children.iter() {
                    if !element.intersects_with(&child.bb) {
                        continue;
                    }
                    for entry in child.pointer.entries.iter() {
                        if element.intersects_with(&entry.bb) {
                            candidate_fn(entry);
                        }
                    }
                }
            }
            NodeData::NonLeaf(children) => {
                for child in children.iter() {
                    if element.intersects_with(&child.bb) {
                        self.query_node_intersects_generic_fn(
                            &child.pointer,
                            element,
                            candidate_fn,
                        );
                    }
                }
            }
        }
    }

    /// Traverses nodes and collects entries fully contained in a query box.
    fn query_node_contains<'a>(
        &'a self,
        node: &'a RTreeNode<T, N, M, TupleIdentifier>,
        bb: &BoundingBox<T, N>,
        matches: &mut Vec<&'a IndexRecordEntry<T, N, TupleIdentifier>>,
    ) {
        match &node.node_data {
            NodeData::Leaf(children) => {
                for child in children.iter() {
                    if bb.contains(&child.bb) {
                        matches.extend(child.pointer.entries.iter());
                        continue;
                    }
                    if !child.bb.intersects(bb) {
                        continue;
                    }
                    for entry in child.pointer.entries.iter() {
                        if bb.contains(&entry.bb) {
                            matches.push(entry);
                        }
                    }
                }
            }
            NodeData::NonLeaf(children) => {
                for child in children.iter() {
                    if bb.contains(&child.bb) {
                        self.collect_all_entries(&child.pointer, matches);
                        continue;
                    }
                    if child.bb.intersects(bb) {
                        self.query_node_contains(&child.pointer, bb, matches);
                    }
                }
            }
        }
    }

    /// Collects all entries in a subtree.
    fn collect_all_entries<'a>(
        &'a self,
        node: &'a RTreeNode<T, N, M, TupleIdentifier>,
        matches: &mut Vec<&'a IndexRecordEntry<T, N, TupleIdentifier>>,
    ) {
        match &node.node_data {
            NodeData::Leaf(children) => {
                for child in children.iter() {
                    matches.extend(child.pointer.entries.iter());
                }
            }
            NodeData::NonLeaf(children) => {
                for child in children.iter() {
                    self.collect_all_entries(&child.pointer, matches);
                }
            }
        }
    }

    /// Collects owned entries from a node, consuming the subtree.
    fn collect_entries_from_node(
        node: RTreeNode<T, N, M, TupleIdentifier>,
    ) -> Vec<IndexRecordEntry<T, N, TupleIdentifier>> {
        let mut entries = Vec::new();
        match node.node_data {
            NodeData::Leaf(children) => {
                for child in children.into_iter() {
                    entries.extend(child.pointer.entries);
                }
            }
            NodeData::NonLeaf(children) => {
                for child in children.into_iter() {
                    entries.extend(Self::collect_entries_from_node(*child.pointer));
                }
            }
        }
        entries
    }

    /// Inserts an entry into a leaf-level node, splitting if needed.
    fn insert_into_leaf_node(
        split_strategy: &LinearCostSplitting,
        leaves: &mut ArrayVec<ChildPointer<T, N, RTreeLeaf<T, N, M, TupleIdentifier>>, M>,
        leaf_index: Option<usize>,
        entry: IndexRecordEntry<T, N, TupleIdentifier>,
    ) -> Option<RTreeNode<T, N, M, TupleIdentifier>> {
        // If the leaf-level node has no children yet, create a new leaf child.
        let idx = match leaf_index {
            Some(idx) => idx,
            None => {
                let mut leaf = RTreeLeaf::default();
                leaf.insert_entry(entry);
                leaves.push(ChildPointer {
                    bb: leaf.to_bb(),
                    pointer: Box::new(leaf),
                });
                return None;
            }
        };

        let leaf_pointer = &mut leaves[idx];
        if !leaf_pointer.pointer.is_full() {
            leaf_pointer.pointer.insert_entry(entry);
            leaf_pointer.recompute_bb();
            return None;
        }

        let leaf_bb = leaf_pointer.pointer.to_bb();
        let split = split_strategy.split(&leaf_bb, &mut leaf_pointer.pointer.entries, entry);
        leaf_pointer.pointer.entries = split.first.entries;
        leaf_pointer.bb = split.first.bb;

        let new_leaf = RTreeLeaf {
            entries: split.second.entries,
        };
        let new_child = ChildPointer {
            bb: split.second.bb,
            pointer: Box::new(new_leaf),
        };

        if leaves.len() < M {
            leaves.push(new_child);
            return None;
        }

        // Leaf-level node overflow: split and return a new sibling node.
        let parent_bb = leaves.as_slice().to_bb();
        let split = split_strategy.split(&parent_bb, leaves, new_child);
        let second_entries = split.second.entries;
        *leaves = split.first.entries;
        Some(RTreeNode {
            node_data: NodeData::Leaf(second_entries),
        })
    }

    /// Builds leaf children from entries using STR ordering.
    fn build_leaf_children(
        entries: &mut Vec<IndexRecordEntry<T, N, TupleIdentifier>>,
    ) -> Vec<ChildPointer<T, N, RTreeLeaf<T, N, M, TupleIdentifier>>> {
        let ordered = Self::str_order(std::mem::take(entries), &|entry| &entry.bb);

        let mut iter = ordered.into_iter();
        let mut leaves = Vec::new();
        loop {
            let mut leaf = RTreeLeaf::default();
            for _ in 0..M {
                let Some(entry) = iter.next() else {
                    break;
                };
                leaf.entries.push(entry);
            }
            if leaf.entries.is_empty() {
                break;
            }
            leaves.push(ChildPointer {
                bb: leaf.to_bb(),
                pointer: Box::new(leaf),
            });
        }
        leaves
    }

    /// Groups leaf children into leaf-level nodes.
    fn build_leaf_level_nodes(
        leaves: Vec<ChildPointer<T, N, RTreeLeaf<T, N, M, TupleIdentifier>>>,
    ) -> Vec<RTreeNode<T, N, M, TupleIdentifier>> {
        // Group leaf children into leaf-level nodes after STR ordering.
        let ordered = Self::str_order(leaves, &|child| &child.bb);
        let mut iter = ordered.into_iter();
        let mut nodes = Vec::new();
        loop {
            let mut children: ArrayVec<_, M> = ArrayVec::new();
            for _ in 0..M {
                let Some(child) = iter.next() else {
                    break;
                };
                children.push(child);
            }
            if children.is_empty() {
                break;
            }
            nodes.push(RTreeNode {
                node_data: NodeData::Leaf(children),
            });
        }
        nodes
    }

    /// Groups nodes into non-leaf parent nodes.
    fn build_non_leaf_level_nodes(
        nodes: Vec<ChildPointer<T, N, RTreeNode<T, N, M, TupleIdentifier>>>,
    ) -> Vec<RTreeNode<T, N, M, TupleIdentifier>> {
        // Build parent levels bottom-up using the same STR ordering.
        let ordered = Self::str_order(nodes, &|child| &child.bb);
        let mut iter = ordered.into_iter();
        let mut parents = Vec::new();
        loop {
            let mut children: ArrayVec<_, M> = ArrayVec::new();
            for _ in 0..M {
                let Some(child) = iter.next() else {
                    break;
                };
                children.push(child);
            }
            if children.is_empty() {
                break;
            }
            parents.push(RTreeNode {
                node_data: NodeData::NonLeaf(children),
            });
        }
        parents
    }

    /// Orders entries using Sort-Tile-Recursive (STR).
    fn str_order<TEntry>(
        entries: Vec<TEntry>,
        bb_of: &impl Fn(&TEntry) -> &BoundingBox<T, N>,
    ) -> Vec<TEntry> {
        // Sort-Tile-Recursive ordering: spatially sort to improve bulk-load locality.
        let count = entries.len();
        if count <= 1 {
            return entries;
        }

        let num_groups = count.div_ceil(M);
        let slice_count = (num_groups as f64).powf(1.0 / N as f64).ceil().max(1.0) as usize;
        Self::str_order_axis(entries, 0, slice_count, bb_of)
    }

    /// STR ordering along a specific axis, recursing over dimensions.
    fn str_order_axis<TEntry>(
        mut entries: Vec<TEntry>,
        axis: usize,
        slice_count: usize,
        bb_of: &impl Fn(&TEntry) -> &BoundingBox<T, N>,
    ) -> Vec<TEntry> {
        entries.sort_by(|a, b| {
            let a_center = Self::bb_center_axis(bb_of(a), axis);
            let b_center = Self::bb_center_axis(bb_of(b), axis);
            a_center.partial_cmp(&b_center).unwrap_or(Ordering::Equal)
        });

        if axis + 1 >= N || entries.len() <= M {
            return entries;
        }

        let slice_len = entries.len().div_ceil(slice_count);
        let mut ordered = Vec::with_capacity(entries.len());
        let mut remaining = entries;
        while !remaining.is_empty() {
            let take = slice_len.min(remaining.len());
            let slice = remaining.drain(0..take).collect::<Vec<_>>();
            let mut ordered_slice = Self::str_order_axis(slice, axis + 1, slice_count, bb_of);
            ordered.append(&mut ordered_slice);
        }
        ordered
    }

    /// Returns the center coordinate of a bounding box along an axis.
    fn bb_center_axis(bb: &BoundingBox<T, N>, axis: usize) -> T {
        let two = T::one() + T::one();
        (bb.dims[axis].start + bb.dims[axis].end) / two
    }

    /// Recursive nearest-neighbor search with pruning.
    fn nearest_in_node<'a>(
        &'a self,
        node: &'a RTreeNode<T, N, M, TupleIdentifier>,
        point: &[T; N],
        best: &mut Option<(&'a IndexRecordEntry<T, N, TupleIdentifier>, T)>,
    ) {
        match &node.node_data {
            NodeData::Leaf(children) => {
                for child in children.iter() {
                    let child_dist = child.bb.distance2_to_point(point);
                    if let Some((_, best_dist)) = best {
                        if child_dist > *best_dist {
                            continue;
                        }
                    }
                    for entry in child.pointer.entries.iter() {
                        let dist = entry.bb.distance2_to_point(point);
                        match best {
                            Some((_, best_dist)) if dist >= *best_dist => {}
                            _ => {
                                *best = Some((entry, dist));
                            }
                        }
                    }
                }
            }
            NodeData::NonLeaf(children) => {
                // Explore children in order of increasing distance to prune earlier.
                let mut ordered = children
                    .iter()
                    .map(|child| (child.bb.distance2_to_point(point), child))
                    .collect::<Vec<_>>();
                ordered.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

                for (dist, child) in ordered {
                    if let Some((_, best_dist)) = best {
                        if dist > *best_dist {
                            continue;
                        }
                    }
                    self.nearest_in_node(&child.pointer, point, best);
                }
            }
        }
    }

    /// Replaces the root with a new parent containing the old root and split node.
    fn grow_root(&mut self, split_node: RTreeNode<T, N, M, TupleIdentifier>) {
        // Replace the root with a new parent that references the old and new nodes.
        let old_root = std::mem::take(&mut self.root);
        let mut children: ArrayVec<_, M> = ArrayVec::new();
        children.push(ChildPointer {
            bb: old_root.to_bb(),
            pointer: Box::new(old_root),
        });
        children.push(ChildPointer {
            bb: split_node.to_bb(),
            pointer: Box::new(split_node),
        });
        self.root = RTreeNode {
            node_data: NodeData::NonLeaf(children),
        };
    }
}

impl<T, const N: usize, const M: usize, TupleIdentifier> Default for RTree<T, N, M, TupleIdentifier>
where
    T: DimensionType,
{
    fn default() -> Self {
        Self {
            root: RTreeNode::default(),
            split_strategy: LinearCostSplitting::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::intersections::IntersectsWith;

    #[test]
    fn default_works() {
        let r: RTree<f32, 2, 10> = RTree::default();
        assert!(r.is_empty());
    }

    #[test]
    fn simple_insert_works() {
        let mut tree = RTree::<f32, 2, 2>::default();
        tree.insert(0, BoundingBox::from([1.0..=2.0, 4.0..=17.0]));
        assert!(!tree.is_empty());

        match &tree.root.node_data {
            NodeData::Leaf(children) => {
                assert_eq!(children.len(), 1);
                let leaf = &children[0].pointer;
                assert_eq!(leaf.entries.len(), 1);
                assert_eq!(leaf.to_bb(), [1.0..=2.0, 4.0..=17.0].into());
            }
            NodeData::NonLeaf(_) => panic!("expected root to point to leaves"),
        }
    }

    #[test]
    fn insert_works() {
        let mut tree = RTree::<f32, 2, 3>::default();
        tree.insert(0, [16.0..=68.0, 23.0..=35.0].into());
        tree.insert(1, [55.0..=68.0, 12.0..=148.0].into());
        tree.insert(2, [82.0..=94.0, 12.0..=148.0].into());
        tree.insert(3, [82.0..=145.0, 30.0..=42.0].into());

        assert!(!tree.is_empty());

        match &tree.root.node_data {
            NodeData::Leaf(children) => {
                assert_eq!(children.len(), 2);
                assert_eq!(
                    children.as_slice().to_bb(),
                    [16.0..=145.0, 12.0..=148.0].into()
                );
            }
            NodeData::NonLeaf(_) => panic!("expected root to point to leaves"),
        }
    }

    #[test]
    fn query_intersects_works() {
        let mut tree = RTree::<f32, 2, 2>::default();
        tree.insert(0, [0.0..=1.0, 0.0..=1.0].into());
        tree.insert(1, [2.0..=3.0, 2.0..=3.0].into());
        tree.insert(2, [4.0..=5.0, 4.0..=5.0].into());
        tree.insert(3, [1.0..=2.0, 0.0..=1.0].into());

        let results = tree.query_intersects(&[1.5..=4.5, 1.5..=4.5].into());
        let mut ids: Vec<_> = results.iter().map(|entry| entry.id).collect();
        ids.sort();

        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn query_contains_works() {
        let mut tree = RTree::<f32, 2, 2>::default();
        tree.insert(0, [0.0..=1.0, 0.0..=1.0].into());
        tree.insert(1, [2.0..=3.0, 2.0..=3.0].into());
        tree.insert(2, [4.0..=5.0, 4.0..=5.0].into());
        tree.insert(3, [1.0..=2.0, 0.0..=1.0].into());

        let results = tree.query_contains(&[0.0..=3.0, 0.0..=3.0].into());
        let mut ids: Vec<_> = results.iter().map(|entry| entry.id).collect();
        ids.sort();

        assert_eq!(ids, vec![0, 1, 3]);
    }

    struct Ray2 {
        origin: [f32; 2],
        inv_dir: [f32; 2],
    }

    impl Ray2 {
        fn new(origin: [f32; 2], dir: [f32; 2]) -> Self {
            Self {
                origin,
                inv_dir: [1.0 / dir[0], 1.0 / dir[1]],
            }
        }
    }

    impl IntersectsWith<BoundingBox<f32, 2>> for Ray2 {
        fn intersects_with(&self, other: &BoundingBox<f32, 2>) -> bool {
            let mut tmin = f32::NEG_INFINITY;
            let mut tmax = f32::INFINITY;

            for i in 0..2 {
                let start = other.dims[i].start;
                let end = other.dims[i].end;
                let t1 = (start - self.origin[i]) * self.inv_dir[i];
                let t2 = (end - self.origin[i]) * self.inv_dir[i];
                let (t1, t2) = if t1 < t2 { (t1, t2) } else { (t2, t1) };

                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
                if tmin > tmax {
                    return false;
                }
            }

            tmax >= 0.0
        }
    }

    #[test]
    fn query_intersects_generic_works() {
        let mut tree = RTree::<f32, 2, 2>::default();
        tree.insert(0, [0.0..=1.0, 0.0..=1.0].into());
        tree.insert(1, [2.0..=3.0, 2.0..=3.0].into());
        tree.insert(2, [4.0..=5.0, 4.0..=5.0].into());
        tree.insert(3, [1.0..=2.0, 0.0..=1.0].into());

        let ray = Ray2::new([-1.0, 0.5], [1.0, 0.0]);
        let results = tree.query_intersects_generic(&ray);
        let mut ids: Vec<_> = results.iter().map(|entry| entry.id).collect();
        ids.sort();

        assert_eq!(ids, vec![0, 3]);
    }
}
