// https://www.geeksforgeeks.org/interval-tree/

mod inorder_iterator;
mod interval;
mod node;

pub use inorder_iterator::InorderIterator;
pub use interval::Interval;

use crate::interval_tree::node::Node;
use std::fmt::{Debug, Formatter};
use std::ops::RangeInclusive;

#[derive(Default)]
pub struct IntervalTree<T>
where
    T: Clone + PartialOrd,
{
    root: Option<Node<T>>,
}

impl<T> IntervalTree<T>
where
    T: Clone + PartialOrd,
{
    fn new(root: Node<T>) -> Self {
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
    pub fn iter_inorder(&self) -> impl Iterator<Item = &Node<T>> {
        if let Some(node) = &self.root {
            InorderIterator::new(node)
        } else {
            InorderIterator::empty()
        }
    }
}

impl<T: Debug + Clone + PartialOrd> Debug for IntervalTree<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "len = {}", self.len())
    }
}

impl<T> From<Interval<T>> for IntervalTree<T>
where
    T: Clone + PartialOrd,
{
    fn from(interval: Interval<T>) -> Self {
        Self::new(Node::new(interval))
    }
}

impl<T, const N: usize> From<[RangeInclusive<T>; N]> for IntervalTree<T>
where
    T: Clone + PartialOrd,
{
    fn from(intervals: [RangeInclusive<T>; N]) -> Self {
        IntervalTree::new(Node::from(intervals))
    }
}
