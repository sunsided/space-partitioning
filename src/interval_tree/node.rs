use crate::interval_tree::{InorderIterator, Interval};
use std::ops::RangeInclusive;

/// A child node in the tree.
pub type ChildNode<T> = Option<Box<Node<T>>>;

/// Structure to represent a node in Interval Search Tree.
pub struct Node<T> {
    pub interval: Interval<T>,
    max: T,
    pub(crate) left: ChildNode<T>,
    pub(crate) right: ChildNode<T>,
}

impl<T, const N: usize> From<[RangeInclusive<T>; N]> for Node<T>
where
    T: Clone + PartialOrd,
{
    fn from(intervals: [RangeInclusive<T>; N]) -> Self {
        let first_interval = Interval::from(&intervals[0]);
        let mut root = Node::new(first_interval);
        for range in intervals.iter().skip(1) {
            root.insert(Interval::from(range));
        }
        root
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} max = {:?}", self.interval, self.max)
    }
}

impl<T: Clone> Node<T> {
    /// A utility function to create a new Interval Search Tree Node.
    pub(crate) fn new(interval: Interval<T>) -> Self {
        let max = interval.high.clone();
        Self {
            interval,
            max,
            left: None,
            right: None,
        }
    }

    /// Gets the size of the tree, i.e., the number of intervals stored.
    pub(crate) fn len(&self) -> usize {
        let mut size = 1;
        if let Some(left) = &self.left {
            size += left.len();
        }
        if let Some(right) = &self.right {
            size += right.len();
        }
        size
    }
}

impl<T: Clone + PartialOrd<T>> Node<T> {
    /// A utility function to insert a new Interval Search Tree Node
    pub(crate) fn insert(&mut self, interval: Interval<T>) -> &Self {
        // This is similar to BST Insert.  Here the low value of interval
        // is used to maintain BST property

        // Get low/high value of interval at root.
        let low = self.interval.low.clone();
        let high = self.interval.high.clone();

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
        if self.max < high {
            self.max = high;
        }

        self
    }

    /// The main function that searches a given interval i in a given
    /// Interval Tree.
    pub(crate) fn overlap_search(&self, interval: Interval<T>) -> Option<Interval<T>> {
        // Check for overlap with root.
        if self.interval.overlaps_with(&interval) {
            return Some(self.interval.clone());
        }

        // If left child of root is present and max of left child is
        // greater than or equal to given interval, then the interval may
        // overlap with an interval of left subtree.
        if self.left.is_some() && self.left.as_ref().unwrap().max >= interval.low {
            return self.left.as_ref().unwrap().overlap_search(interval.clone());
        }

        // Else interval can only overlap with right subtree, or not at all.
        if self.right.is_some() {
            return self
                .right
                .as_ref()
                .unwrap()
                .overlap_search(interval.clone());
        }

        None
    }

    /// Iterates the tree in-order, i.e. earlier-starting intervals first.
    pub(crate) fn iter_inorder(&self) -> InorderIterator<T> {
        InorderIterator::new(&self)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    /// Constructs a test tree.
    pub fn construct_test_root_node() -> Node<i32> {
        Node::from([15..=20, 10..=30, 17..=19, 5..=20, 12..=15, 30..=40])
    }

    #[test]
    fn overlap_search_works() {
        let root = construct_test_root_node();
        let overlap = root.overlap_search(Interval::from(6..=7));
        assert_eq!(overlap, Some(Interval::from(5..=20)));
    }

    #[test]
    fn len_works() {
        let root = construct_test_root_node();
        assert_eq!(root.len(), 6);
    }
}
