// https://www.geeksforgeeks.org/interval-tree/

mod inorder_iterator;
mod interval;
mod interval_tree_entry;
mod interval_tree_node;
mod interval_type;

pub use inorder_iterator::InorderIterator;
pub use interval::{Interval, IntervalType};
pub use interval_tree_entry::IntervalTreeEntry;

use crate::interval_tree::interval_tree_node::IntervalTreeNode;
use std::fmt::{Debug, Formatter};

#[derive(Default)]
pub struct IntervalTree<T, D>
where
    T: IntervalType,
{
    root: Option<IntervalTreeNode<T, D>>,
}

impl<T, D> IntervalTree<T, D>
where
    T: IntervalType,
{
    fn new(root: IntervalTreeNode<T, D>) -> Self {
        Self { root: Some(root) }
    }

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
        Self::new(IntervalTreeNode::new_from_pair(interval, ()))
    }
}

impl<T, D> From<(Interval<T>, D)> for IntervalTree<T, D>
where
    T: IntervalType,
{
    fn from(value: (Interval<T>, D)) -> Self {
        Self::new(IntervalTreeNode::new_from_pair(value.0, value.1))
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
        IntervalTree::new(IntervalTreeNode::from_iter(iter))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::FromIterator;

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
