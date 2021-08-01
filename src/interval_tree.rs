///! According to Wikipedia:
///! > An interval tree is a tree data structure to hold intervals.
///! > Specifically, it allows one to efficiently find all intervals that overlap with any given interval or point.
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

/// An Interval Tree.
pub struct IntervalTree<T, D>
where
    T: IntervalType,
{
    /// The root node. May not exist if the tree is default constructed
    /// or initialized from an empty iterator.
    root: Option<IntervalTreeNode<T, D>>,
}

impl<T, D> Default for IntervalTree<T, D>
where
    T: IntervalType,
{
    /// Returns an empty `IntervalTree<T, D>`.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::IntervalTree;
    /// let tree = IntervalTree::<i32, ()>::default();
    /// assert_eq!(tree.len(), 0);
    /// ```
    fn default() -> Self {
        Self { root: None }
    }
}

impl<T, D> IntervalTree<T, D>
where
    T: IntervalType,
{
    /// Creates a new `IntervalTree` from a root entry.
    ///
    /// # Parameters
    /// * `entry` - The first entry.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::IntervalTree;
    /// let tree = IntervalTree::new_from_entry((15..=20, "data"));
    /// assert_eq!(tree.len(), 1);
    /// ```
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

    /// Inserts a new entry to the `IntervalTree`.
    ///
    /// # Parameters
    /// * `entry` - The entry to insert.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::IntervalTree;
    /// let mut tree = IntervalTree::default();
    /// tree.insert((15..=20, "data"));
    /// assert_eq!(tree.len(), 1);
    /// ```
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

    /// Returns the number of elements in the `IntervalTree`.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::IntervalTree;
    /// let tree = IntervalTree::new_from_entry((15..=20, "data"));
    /// assert_eq!(tree.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        return if let Some(node) = &self.root {
            node.len()
        } else {
            0
        };
    }

    /// Returns whether the tree is empty, i.e., whether it has no elements.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::IntervalTree;
    /// let tree = IntervalTree::<i32, ()>::default();
    /// assert!(tree.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Queries the tree for overlaps with the specified `interval`.
    ///
    /// /// # Parameters
    /// * `interval` - The interval to query for.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::IntervalTree;
    /// use space_partitioning::interval_tree::Interval;
    ///
    /// let mut tree = IntervalTree::new_from_entry((15..=20, "A"));
    /// tree.insert((100..=101, "B"));
    ///
    /// let matched_a = tree.overlap_search(&(18..=25).into()).unwrap();
    /// assert_eq!(matched_a.interval.start, 15);
    /// assert_eq!(matched_a.interval.end, 20);
    /// assert_eq!(matched_a.data, "A");
    ///
    /// let matched_b = tree.overlap_search(&(100..=100).into()).unwrap();
    /// assert_eq!(matched_b.interval.start, 100);
    /// assert_eq!(matched_b.interval.end, 101);
    /// assert_eq!(matched_b.data, "B");
    ///
    /// let no_match = tree.overlap_search(0..=5);
    /// assert!(no_match.is_none());
    /// ```
    pub fn overlap_search<I>(&self, interval: I) -> Option<&IntervalTreeEntry<T, D>>
    where
        I: Into<Interval<T>>,
    {
        if let Some(node) = &self.root {
            let interval = interval.into();
            node.overlap_search(&interval)
        } else {
            None
        }
    }

    /// Returns an `InorderIterator<T, D>` that iterates the tree elements in order
    /// of their interval starts.
    ///
    /// # Example
    /// ```rust
    /// use space_partitioning::IntervalTree;
    /// use std::iter::FromIterator;
    ///
    /// let tree = IntervalTree::from_iter([(18..=25, "abc"), (0..=20, "xyz")]);
    /// let mut iter = tree.iter_inorder();
    ///
    /// let first = iter.next().unwrap();
    /// assert_eq!(first.interval.start, 0);
    /// assert_eq!(first.interval.end, 20);
    /// assert_eq!(first.data, "xyz");
    ///
    /// let second = iter.next().unwrap();
    /// assert_eq!(second.interval.start, 18);
    /// assert_eq!(second.interval.end, 25);
    /// assert_eq!(second.data, "abc");
    ///
    /// assert!(iter.next().is_none());
    /// ```
    pub fn iter_inorder(&self) -> InorderIterator<T, D> {
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
            assert_eq!(last.interval.start, 30);
            assert_eq!(last.interval.end, 40);
        }
    }

    mod search {
        use super::*;

        #[test]
        fn overlap_search_works() {
            let tree =
                IntervalTree::from_iter([15..=20, 10..=30, 17..=19, 5..=20, 12..=15, 30..=40]);
            let overlap = tree.overlap_search(Interval::from(6..=7));
            assert_eq!(overlap.unwrap().interval, Interval::from(5..=20));
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

    mod multi_dimensional {
        use super::*;

        #[derive(Debug, PartialOrd, PartialEq, Copy, Clone, Default)]
        struct Vec2d {
            pub x: f64,
            pub y: f64,
        }

        impl IntervalType for Vec2d {}

        #[test]
        fn it_works() {
            let tree = IntervalTree::from_iter([
                (
                    Interval::new(Vec2d { x: 1., y: 2. }, Vec2d { x: 10., y: 10. }),
                    "A",
                ),
                (
                    Interval::new(Vec2d { x: -5., y: -5. }, Vec2d { x: 5., y: 5. }),
                    "B",
                ),
                (
                    Interval::new(Vec2d { x: -10., y: -10. }, Vec2d { x: -7., y: -7. }),
                    "C",
                ),
            ]);

            let test = Interval::new(Vec2d::default(), Vec2d::default());
            let result = tree.overlap_search(test).unwrap();

            assert_eq!(result.data, "B")
        }
    }
}
