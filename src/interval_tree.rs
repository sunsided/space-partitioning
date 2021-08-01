// https://www.geeksforgeeks.org/interval-tree/

mod inorder_iterator;
mod interval;
mod interval_tree_entry;
mod interval_tree_node;
mod interval_type;

pub use inorder_iterator::InorderIterator;
pub use interval::{Interval, IntervalType};
pub use interval_tree_entry::IntervalTreeEntry;

use crate::interval_tree::interval_tree_node::{IntervalTreeNode, IntervalTreeNodeOption};
use std::fmt::{Debug, Formatter};

pub struct IntervalTree<T, D>
where
    T: IntervalType,
{
    root: Option<IntervalTreeNode<T, D>>,
}

impl<T, D> Default for IntervalTree<T, D>
where
    T: IntervalType,
{
    fn default() -> Self {
        Self { root: None }
    }
}

impl<T, D> IntervalTree<T, D>
where
    T: IntervalType,
{
    pub fn new_from_entry<I>(entry: I) -> Self
    where
        I: Into<IntervalTreeEntry<T, D>>,
    {
        Self {
            root: Some(IntervalTreeNode::new(entry.into())),
        }
    }

    fn new_from_node(root: IntervalTreeNode<T, D>) -> Self {
        Self { root: Some(root) }
    }

    /// A utility function to insert a new Interval Search Tree Node
    pub fn insert<I>(&mut self, entry: I) -> &Self
    where
        I: Into<IntervalTreeEntry<T, D>>,
    {
        let node = IntervalTreeNode::new(entry.into());
        if self.root.is_none() {
            self.root = Some(node);
        } else {
            self.root.as_mut().unwrap().insert(node);
        }
        self
    }

    /// Returns the number of elements in the tree.
    pub fn len(&self) -> usize {
        return if let Some(node) = &self.root {
            node.len()
        } else {
            0
        };
    }

    /// The main function that searches a given interval i in a given
    /// Interval Tree.
    pub fn overlap_search(&self, interval: Interval<T>) -> Option<Interval<T>> {
        if let Some(node) = &self.root {
            node.overlap_search(interval)
        } else {
            None
        }
    }

    /// Iterates the tree in-order, i.e. earlier-starting intervals first.
    pub fn iter_inorder(&self) -> impl Iterator<Item = &IntervalTreeNode<T, D>> {
        if let Some(node) = &self.root {
            InorderIterator::new(node)
        } else {
            InorderIterator::empty()
        }
    }
}

impl<T, D> Debug for IntervalTree<T, D>
where
    T: Debug + IntervalType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "len = {}", self.len())
    }
}

impl<T> From<Interval<T>> for IntervalTree<T, ()>
where
    T: IntervalType,
{
    fn from(interval: Interval<T>) -> Self {
        Self::new_from_node(IntervalTreeNode::new_from_pair(interval, ()))
    }
}

impl<T, D> From<(Interval<T>, D)> for IntervalTree<T, D>
where
    T: IntervalType,
{
    fn from(value: (Interval<T>, D)) -> Self {
        Self::new_from_node(IntervalTreeNode::new_from_pair(value.0, value.1))
    }
}

impl<I, T, D> std::iter::FromIterator<I> for IntervalTree<T, D>
where
    I: Into<IntervalTreeEntry<T, D>>,
    T: IntervalType,
{
    fn from_iter<Iter>(iter: Iter) -> Self
    where
        Iter: IntoIterator<Item = I>,
    {
        match IntervalTreeNodeOption::from_iter(iter) {
            IntervalTreeNodeOption::Some(node) => IntervalTree::new_from_node(node),
            IntervalTreeNodeOption::None => IntervalTree::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::FromIterator;

    mod construction {
        use super::*;

        #[test]
        fn insert_only_works() {
            let mut tree = IntervalTree::default();
            assert_eq!(tree.len(), 0);
            tree.insert((15..=20, 4.2));
            assert_eq!(tree.len(), 1);
            tree.insert((10..=30, 13.37));
            assert_eq!(tree.len(), 2);
        }

        #[test]
        fn from_constructor_works() {
            let mut tree = IntervalTree::new_from_entry(15..=20);
            assert_eq!(tree.len(), 1);
            tree.insert(15..=20);
            assert_eq!(tree.len(), 2);
        }
    }

    mod from_iter {
        use super::*;

        #[test]
        fn range_without_data_works() {
            let tree =
                IntervalTree::from_iter([15..=20, 10..=30, 17..=19, 5..=20, 12..=15, 30..=40]);
            assert_eq!(tree.len(), 6);
        }

        #[test]
        fn interval_without_data_works() {
            let tree = IntervalTree::from_iter([
                Interval::from(15..=20),
                (10..=30).into(),
                (17..=19).into(),
                (5..=20).into(),
                (12..=15).into(),
                (30..=40).into(),
            ]);
            assert_eq!(tree.len(), 6);
        }

        #[test]
        fn range_with_data_works() {
            let tree = IntervalTree::from_iter([
                (15..=20, 1),
                (10..=30, 2),
                (17..=19, 3),
                (5..=20, 4),
                (12..=15, 5),
                (30..=40, 6),
            ]);
            assert_eq!(tree.len(), 6);
        }

        #[test]
        fn interval_with_data_works() {
            let tree = IntervalTree::from_iter([
                (Interval::from(15..=20), 1),
                ((10..=30).into(), 2),
                ((17..=19).into(), 3),
                ((5..=20).into(), 4),
                ((12..=15).into(), 5),
                ((30..=40).into(), 6),
            ]);
            assert_eq!(tree.len(), 6);
        }
    }

    mod iter {
        use super::*;

        #[test]
        fn last_works() {
            let tree =
                IntervalTree::from_iter([15..=20, 10..=30, 17..=19, 5..=20, 12..=15, 30..=40]);
            let last = tree.iter_inorder().last();
            assert!(last.is_some());
            let last = last.unwrap();
            assert_eq!(last.entry.interval.start, 30);
            assert_eq!(last.entry.interval.end, 40);
        }
    }

    mod search {
        use super::*;

        #[test]
        fn overlap_search_works() {
            let tree =
                IntervalTree::from_iter([15..=20, 10..=30, 17..=19, 5..=20, 12..=15, 30..=40]);
            let overlap = tree.overlap_search(Interval::from(6..=7));
            assert_eq!(overlap, Some(Interval::from(5..=20)));
        }
    }

    mod utility {
        use super::*;
        use std::ops::RangeInclusive;

        #[test]
        fn len_works() {
            let tree =
                IntervalTree::from_iter([15..=20, 10..=30, 17..=19, 5..=20, 12..=15, 30..=40]);
            assert_eq!(tree.len(), 6);
        }

        #[test]
        fn len_when_empty_works() {
            let tree = IntervalTree::from_iter([] as [RangeInclusive<i32>; 0]);
            assert_eq!(tree.len(), 0);
        }
    }
}
