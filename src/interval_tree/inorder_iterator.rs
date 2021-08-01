use crate::interval_tree::interval::IntervalType;
use crate::interval_tree::node::{ChildNode, Node};

#[derive(Debug)]
enum State<'a, T, D>
where
    T: IntervalType,
{
    Initial,
    EmitLeft(Box<InorderIterator<'a, T, D>>),
    EmitSelf,
    EmitRight(Box<InorderIterator<'a, T, D>>),
    Done,
}

#[derive(Debug)]
pub struct InorderIterator<'a, T, D>
where
    T: IntervalType,
{
    root: Option<&'a Node<T, D>>,
    current_state: State<'a, T, D>,
}

impl<'a, T, D> InorderIterator<'a, T, D>
where
    T: IntervalType,
{
    pub(crate) fn new(root: &'a Node<T, D>) -> Self {
        Self {
            root: Some(root),
            current_state: State::Initial,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            root: None,
            current_state: State::Done,
        }
    }
}

impl<'a, T, D> Iterator for InorderIterator<'a, T, D>
where
    T: IntervalType,
{
    type Item = &'a Node<T, D>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.root.is_none() {
            return None;
        }

        let root = self.root.unwrap();

        loop {
            match &mut self.current_state {
                // The initial state is entered always.
                State::Initial => {
                    if let Some(left) = &root.left {
                        let iter = left.iter_inorder();
                        self.current_state = State::EmitLeft(Box::new(iter))
                    } else {
                        self.current_state = State::EmitSelf;
                    }
                }
                // Only happens when there is a left child,
                // enumerate until it is exhausted.
                State::EmitLeft(iter) => {
                    if let Some(value) = iter.next() {
                        return Some(value);
                    }
                    self.current_state = State::EmitSelf;
                }
                // The "self" state is entered always.
                State::EmitSelf => {
                    if let Some(right) = &root.right {
                        let iter = right.iter_inorder();
                        self.current_state = State::EmitRight(Box::new(iter));
                    } else {
                        self.current_state = State::Done;
                    }
                    return Some(root);
                }
                // Only happens when there is a right child,
                // enumerate until it is exhausted.
                State::EmitRight(iter) => {
                    if let Some(value) = iter.next() {
                        return Some(value);
                    }
                    self.current_state = State::Done;
                }
                // The "Done" state is entered last.
                State::Done => {
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.root.is_none() {
            return (0, None);
        }

        let size = self.root.unwrap().len();
        return (size, Some(size));
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        if let Some(node) = self.root {
            node.len()
        } else {
            0
        }
    }

    fn last(self) -> Option<Self::Item> {
        if self.root.is_none() {
            return None;
        }

        let mut token = self.root.unwrap();

        while token.right.is_some() {
            token = token.right.as_ref().unwrap();
        }

        Some(token)
    }

    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        fn inorder<'a, T, D, F>(node: &'a Node<T, D>, f: &mut F)
        where
            F: FnMut(&'a Node<T, D>),
            T: IntervalType,
        {
            inorder_child(&node.left, f);
            (*f)(node);
            inorder_child(&node.right, f);
        }

        fn inorder_child<'a, T, D, F>(node: &'a ChildNode<T, D>, f: &mut F)
        where
            F: FnMut(&'a Node<T, D>),
            T: IntervalType,
        {
            if node.is_none() {
                return;
            }

            inorder(&node.as_ref().unwrap(), f);
        }

        if let Some(root) = self.root {
            inorder(&root, &mut f);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::interval_tree::node::{test::construct_test_root_node, ChildNode};
    use crate::interval_tree::{InorderIterator, IntervalType, Node};

    #[test]
    fn size_hint_when_empty_works() {
        let iter = InorderIterator::<i32, ()>::empty();
        let (min, max) = iter.size_hint();
        assert_eq!(min, 0);
        assert_eq!(max, None);
    }

    #[test]
    fn size_hint_works() {
        let root = construct_test_root_node();
        let (min, max) = root.iter_inorder().size_hint();
        assert_eq!(min, 6);
        assert_eq!(max, Some(6));
    }

    #[test]
    fn count_when_empty_works() {
        let iter = InorderIterator::<i32, ()>::empty();
        assert_eq!(iter.count(), 0);
    }

    #[test]
    fn count_works() {
        let root = construct_test_root_node();
        let count = root.iter_inorder().count();
        assert_eq!(count, 6);
    }

    #[test]
    fn last_works() {
        let root = construct_test_root_node();
        let last = root.iter_inorder().last();
        assert!(last.is_some());
        let last = last.unwrap();
        assert_eq!(last.entry.interval.start, 30);
        assert_eq!(last.entry.interval.end, 40);
    }

    #[test]
    fn iteration_when_empty_works() {
        let mut iter = InorderIterator::<i32, ()>::empty();
        assert!(iter.next().is_none());
    }

    #[test]
    fn iteration_works() {
        let root = construct_test_root_node();

        // Collect the expected nodes.
        let mut expected = Vec::default();
        collect_inorder(&root, &mut expected);

        // Reverse the collection for easier handling.
        // This allows us to pop elements from the back until
        // the set is empty.
        expected.reverse();

        // Act / Assert
        for node in root.iter_inorder() {
            let expected_node = expected.pop();
            assert!(expected_node.is_some());
            assert_eq!(expected_node.unwrap().entry.interval, node.entry.interval);
        }
    }

    #[test]
    fn for_each_works() {
        let root = construct_test_root_node();

        // Collect the expected nodes.
        let mut expected = Vec::default();
        collect_inorder(&root, &mut expected);

        // Act
        let mut collected = Vec::default();
        root.iter_inorder().for_each(|node| collected.push(node));

        // Assert
        assert_eq!(expected.len(), collected.len());
        for (expected_node, node) in expected.into_iter().zip(collected) {
            assert_eq!(expected_node.entry.interval, node.entry.interval);
        }
    }

    fn collect_inorder<'a, T, D>(node: &'a Node<T, D>, out: &mut Vec<&'a Node<T, D>>)
    where
        T: IntervalType,
    {
        collect_inorder_child(&node.left, out);
        out.push(node);
        collect_inorder_child(&node.right, out);
    }

    fn collect_inorder_child<'a, T, D>(node: &'a ChildNode<T, D>, out: &mut Vec<&'a Node<T, D>>)
    where
        T: IntervalType,
    {
        if node.is_none() {
            return;
        }

        collect_inorder(&node.as_ref().unwrap(), out);
    }
}
