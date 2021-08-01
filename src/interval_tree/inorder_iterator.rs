use crate::interval_tree::{ChildNode, Node};

#[derive(Debug)]
enum State<'a, T> {
    Initial,
    EmitLeft(Box<InorderIterator<'a, T>>),
    EmitSelf,
    EmitRight(Box<InorderIterator<'a, T>>),
    Done,
}

#[derive(Debug)]
pub struct InorderIterator<'a, T> {
    root: &'a Node<T>,
    current_state: State<'a, T>,
}

impl<'a, T> InorderIterator<'a, T> {
    pub(crate) fn new(root: &'a Node<T>) -> Self {
        Self {
            root,
            current_state: State::Initial,
        }
    }
}

impl<'a, T: Copy + PartialOrd> Iterator for InorderIterator<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.current_state {
                // The initial state is entered always.
                State::Initial => {
                    if let Some(left) = &self.root.left {
                        let iter = left.iter_inorder();
                        self.current_state = State::EmitLeft(Box::new(iter));
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
                    if let Some(right) = &self.root.right {
                        let iter = right.iter_inorder();
                        self.current_state = State::EmitRight(Box::new(iter));
                    } else {
                        self.current_state = State::Done;
                    }
                    return Some(self.root);
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
        let size = self.root.len();
        return (size, Some(size));
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.root.len()
    }

    fn last(self) -> Option<Self::Item> {
        let mut token = self.root;
        while token.right.is_some() {
            token = token.right.as_ref().unwrap();
        }

        Some(token)
    }

    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        fn inorder<'a, T, F>(node: &'a Node<T>, f: &mut F)
        where
            F: FnMut(&'a Node<T>),
        {
            inorder_child(&node.left, f);
            (*f)(node);
            inorder_child(&node.right, f);
        }

        fn inorder_child<'a, T, F>(node: &'a ChildNode<T>, f: &mut F)
        where
            F: FnMut(&'a Node<T>),
        {
            if node.is_none() {
                return;
            }

            inorder(&node.as_ref().unwrap(), f);
        }

        inorder(self.root, &mut f);
    }
}

#[cfg(test)]
mod test {
    use crate::interval_tree::{test::construct_test_tree, ChildNode, Node};
    use std::fmt::Debug;

    #[test]
    fn size_hint_works() {
        let root = construct_test_tree();
        let (min, max) = root.iter_inorder().size_hint();
        assert_eq!(min, 6);
        assert_eq!(max, Some(6));
    }

    #[test]
    fn count_works() {
        let root = construct_test_tree();
        let count = root.iter_inorder().count();
        assert_eq!(count, 6);
    }

    #[test]
    fn last_works() {
        let root = construct_test_tree();
        let last = root.iter_inorder().last();
        assert!(last.is_some());
        let last = last.unwrap();
        assert_eq!(last.interval.low, 30);
        assert_eq!(last.interval.high, 40);
    }

    #[test]
    fn iteration_works() {
        let root = construct_test_tree();

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
            assert_eq!(expected_node.unwrap().interval, node.interval);
        }
    }

    #[test]
    fn for_each_works() {
        let root = construct_test_tree();

        // Collect the expected nodes.
        let mut expected = Vec::default();
        collect_inorder(&root, &mut expected);

        // Act
        let mut collected = Vec::default();
        root.iter_inorder().for_each(|node| collected.push(node));

        // Assert
        assert_eq!(expected.len(), collected.len());
        for (expected_node, node) in expected.into_iter().zip(collected) {
            assert_eq!(expected_node.interval, node.interval);
        }
    }

    fn collect_inorder<'a, T: Debug>(node: &'a Node<T>, out: &mut Vec<&'a Node<T>>) {
        collect_inorder_child(&node.left, out);
        out.push(node);
        collect_inorder_child(&node.right, out);
    }

    fn collect_inorder_child<'a, T: Debug>(node: &'a ChildNode<T>, out: &mut Vec<&'a Node<T>>) {
        if node.is_none() {
            return;
        }

        collect_inorder(&node.as_ref().unwrap(), out);
    }
}
