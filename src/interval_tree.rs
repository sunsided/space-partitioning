// https://www.geeksforgeeks.org/interval-tree/

mod inorder_iterator;
mod interval;

pub use interval::Interval;

use crate::interval_tree::inorder_iterator::InorderIterator;
use std::fmt::{Debug, Formatter};

/// A child node in the tree.
pub type ChildNode<T> = Option<Box<Node<T>>>;

/// Structure to represent a node in Interval Search Tree.
pub struct Node<T> {
    interval: Interval<T>,
    max: T,
    pub left: ChildNode<T>,
    pub right: ChildNode<T>,
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} max = {:?}", self.interval, self.max)
    }
}

impl<T: Copy + PartialOrd<T>> Node<T> {
    /// A utility function to create a new Interval Search Tree Node.
    pub fn new(interval: Interval<T>) -> Self {
        let max = interval.high.clone();
        Self {
            interval,
            max,
            left: None,
            right: None,
        }
    }

    /// A utility function to insert a new Interval Search Tree Node
    pub fn insert(&mut self, interval: Interval<T>) -> &Self {
        // This is similar to BST Insert.  Here the low value of interval
        // is used to maintain BST property

        // Get low value of interval at root.
        let low = self.interval.low;

        // If root's low value is smaller, then new interval goes to
        // left subtree, otherwise it goes to the right subtree.
        if interval.low < low {
            match &mut self.left {
                Some(left) => {
                    left.insert(interval);
                }
                None => {
                    self.left = Some(Box::new(Self::new(interval)));
                }
            };
        } else {
            match &mut self.right {
                Some(right) => {
                    right.insert(interval);
                }
                None => {
                    self.right = Some(Box::new(Self::new(interval)));
                }
            };
        }

        // Update the max value of this ancestor if needed
        if self.max < interval.high {
            self.max = interval.high;
        }

        self
    }

    /// The main function that searches a given interval i in a given
    /// Interval Tree.
    pub fn overlap_search(&self, interval: Interval<T>) -> Option<Interval<T>> {
        // Check for overlap with root.
        if self.interval.overlaps_with(interval) {
            return Some(self.interval);
        }

        // If left child of root is present and max of left child is
        // greater than or equal to given interval, then the interval may
        // overlap with an interval of left subtree.
        if self.left.is_some() && self.left.as_ref().unwrap().max >= interval.low {
            return self.left.as_ref().unwrap().overlap_search(interval);
        }

        // Else interval can only overlap with right subtree, or not at all.
        if self.right.is_some() {
            return self.right.as_ref().unwrap().overlap_search(interval);
        }

        None
    }

    pub fn iter_inorder(&self) -> InorderIterator<T> {
        InorderIterator::new(&self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let intervals = [15..=20, 10..=30, 17..=19, 5..=20, 12..=15, 30..=40];
        let mut root = Node::new((&intervals[0]).into());
        for interval in intervals.iter().skip(1) {
            root.insert(interval.into());
        }

        println!("Inorder traversal of constructed Interval Tree:");
        for node in root.iter_inorder() {
            println!("{:?}", node);
        }

        let overlap = root.overlap_search(Interval::from(6..=7));
        assert_eq!(overlap, Some(Interval::from(5..=20)));
    }
}
