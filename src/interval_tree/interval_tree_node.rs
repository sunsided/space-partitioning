use crate::interval_tree::{InorderIterator, Interval, IntervalTreeEntry, IntervalType};

/// A child node in the tree.
pub type ChildNode<T, D> = Option<Box<IntervalTreeNode<T, D>>>;

/// Structure to represent a node in Interval Search Tree.
pub struct IntervalTreeNode<T, D>
where
    T: IntervalType,
{
    pub entry: IntervalTreeEntry<T, D>,
    max: T,
    pub(crate) left: ChildNode<T, D>,
    pub(crate) right: ChildNode<T, D>,
}

/// Wrapper to help with `FromIterator<T>` implementations
/// that may have to deal with empty sequences.
#[derive(Debug)]
pub(crate) enum IntervalTreeNodeOption<T, D>
where
    T: IntervalType,
{
    None,
    Some(IntervalTreeNode<T, D>),
}

impl<T, D> IntervalTreeNodeOption<T, D>
where
    T: IntervalType,
{
    /// Unwraps the value of this option if it exists or panics
    /// if it was `None`.
    #[allow(dead_code)]
    fn unwrap(self) -> IntervalTreeNode<T, D> {
        match self {
            Self::Some(value) => value,
            Self::None => panic!(),
        }
    }
}

impl<T, D> IntervalTreeNode<T, D>
where
    T: IntervalType,
{
    /// A utility function to create a new Interval Search Tree Node.
    pub(crate) fn new(entry: IntervalTreeEntry<T, D>) -> Self {
        let max = entry.interval.end.clone();
        Self {
            entry,
            max,
            left: None,
            right: None,
        }
    }

    /// A utility function to create a new Interval Search Tree Node.
    pub(crate) fn new_from_pair<I>(interval: I, data: D) -> Self
    where
        I: Into<Interval<T>>,
    {
        Self::new(IntervalTreeEntry::new(interval, data))
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

    /// A utility function to insert a new Interval Search Tree Node
    pub(crate) fn insert(&mut self, node: IntervalTreeNode<T, D>) -> &Self {
        // This is similar to BST Insert.  Here the low value of interval
        // is used to maintain BST property

        // Get low/high value of interval at root.
        let low = self.entry.interval.start.clone();
        let high = self.entry.interval.end.clone();

        // If root's low value is smaller, then new interval goes to
        // left subtree, otherwise it goes to the right subtree.
        if node.entry.interval.start < low {
            match &mut self.left {
                Some(left) => {
                    left.insert(node);
                }
                None => {
                    self.left = Some(Box::new(node));
                }
            };
        } else {
            match &mut self.right {
                Some(right) => {
                    right.insert(node);
                }
                None => {
                    self.right = Some(Box::new(node));
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
        if self.entry.interval.overlaps_with(&interval) {
            return Some(self.entry.interval.clone());
        }

        // If left child of root is present and max of left child is
        // greater than or equal to given interval, then the interval may
        // overlap with an interval of left subtree.
        if self.left.is_some() && self.left.as_ref().unwrap().max >= interval.start {
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
    pub(crate) fn iter_inorder(&self) -> InorderIterator<T, D> {
        InorderIterator::new(&self)
    }
}

impl<T, D> From<IntervalTreeEntry<T, D>> for IntervalTreeNode<T, D>
where
    T: IntervalType,
{
    fn from(value: IntervalTreeEntry<T, D>) -> Self {
        IntervalTreeNode::new(value)
    }
}

impl<I, T, D> std::iter::FromIterator<I> for IntervalTreeNodeOption<T, D>
where
    I: Into<IntervalTreeEntry<T, D>>,
    T: IntervalType,
{
    fn from_iter<Iter>(iter: Iter) -> Self
    where
        Iter: IntoIterator<Item = I>,
    {
        let mut root: Option<IntervalTreeNode<T, D>> = None;
        for into_entry in iter.into_iter() {
            let entry: IntervalTreeEntry<T, D> = into_entry.into();

            let new_node = IntervalTreeNode::from(entry);
            if root.is_some() {
                root.as_mut().unwrap().insert(new_node);
            } else {
                root = Some(new_node)
            }
        }

        if root.is_some() {
            IntervalTreeNodeOption::Some(root.unwrap())
        } else {
            IntervalTreeNodeOption::None
        }
    }
}

impl<T: std::fmt::Debug, D> std::fmt::Debug for IntervalTreeNode<T, D>
where
    T: IntervalType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} max = {:?}", self.entry.interval, self.max)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use std::iter::FromIterator;

    /// Constructs a test tree.
    pub fn construct_test_root_node() -> IntervalTreeNode<i32, ()> {
        IntervalTreeNodeOption::from_iter([15..=20, 10..=30, 17..=19, 5..=20, 12..=15, 30..=40])
            .unwrap()
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
