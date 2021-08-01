// https://www.geeksforgeeks.org/interval-tree/

mod inorder_iterator;
mod interval;
mod interval_type;
mod node;

pub use inorder_iterator::InorderIterator;
pub use interval::{Interval, IntervalType};

use crate::interval_tree::node::Node;
use std::fmt::{Debug, Formatter};
use std::ops::RangeInclusive;

#[derive(Default)]
pub struct IntervalTree<T, D>
where
    T: Clone + IntervalType,
{
    root: Option<Node<T, D>>,
}

impl<T, D> IntervalTree<T, D>
where
    T: Clone + IntervalType,
{
    fn new(root: Node<T, D>) -> Self {
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
    pub fn iter_inorder(&self) -> impl Iterator<Item = &Node<T, D>> {
        if let Some(node) = &self.root {
            InorderIterator::new(node)
        } else {
            InorderIterator::empty()
        }
    }
}

impl<T: Debug + Clone + PartialOrd, D> Debug for IntervalTree<T, D>
where
    T: IntervalType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "len = {}", self.len())
    }
}

impl<T> From<Interval<T>> for IntervalTree<T, ()>
where
    T: Clone + IntervalType,
{
    fn from(interval: Interval<T>) -> Self {
        Self::new(Node::new_pair(interval, ()))
    }
}

impl<T, const N: usize> From<[RangeInclusive<T>; N]> for IntervalTree<T, ()>
where
    T: Clone + IntervalType,
{
    fn from(intervals: [RangeInclusive<T>; N]) -> Self {
        IntervalTree::new(Node::from_ranges_empty(intervals))
    }
}

impl<T, D, const N: usize> From<[(Interval<T>, D); N]> for IntervalTree<T, D>
where
    T: Clone + IntervalType,
{
    fn from(intervals: [(Interval<T>, D); N]) -> Self {
        use std::iter::FromIterator;
        IntervalTree::new(Node::from_iter(intervals))
    }
}

impl<T, D, const N: usize> From<[(RangeInclusive<T>, D); N]> for IntervalTree<T, D>
where
    T: Clone + IntervalType,
{
    fn from(intervals: [(RangeInclusive<T>, D); N]) -> Self {
        use std::iter::FromIterator;
        IntervalTree::new(Node::from_iter(intervals))
    }
}
