use crate::interval_tree::Node;

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
