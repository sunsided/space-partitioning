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
                State::Initial => {
                    if self.root.left.is_none() {
                        // If there is no left child we directly jump to emitting
                        // the "root" node.
                        self.current_state = State::EmitSelf;
                    } else {
                        // If there is a left child we go on to emit its own values.
                        let iter = self.root.left.as_ref().unwrap().iter_inorder();
                        self.current_state = State::EmitLeft(Box::new(iter));
                    }
                }
                // Only happens when there is a left child, enumerate until it is exhausted.
                State::EmitLeft(iter) => {
                    if let Some(value) = iter.next() {
                        return Some(value);
                    }
                    self.current_state = State::EmitSelf;
                }
                State::EmitSelf => {
                    if self.root.right.is_none() {
                        // If there is no right node, we emit the "root" node, and proceed to done.
                        self.current_state = State::Done;
                    } else {
                        // If there is a right node, we emit the "root" node, and proceed to
                        // emitting the right side.
                        let iter = self.root.right.as_ref().unwrap().iter_inorder();
                        self.current_state = State::EmitRight(Box::new(iter));
                    }
                    return Some(self.root);
                }
                // Only happens when there is a right child, enumerate until it is exhausted.
                State::EmitRight(iter) => {
                    if let Some(value) = iter.next() {
                        return Some(value);
                    }
                    self.current_state = State::Done;
                }
                State::Done => {
                    return None;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::interval_tree::{ChildNode, Node};
    use std::fmt::Debug;

    #[test]
    fn it_works() {
        let intervals = [15..=20, 10..=30, 17..=19, 5..=20, 12..=15, 30..=40];
        let mut root = Node::new((&intervals[0]).into());
        for interval in intervals.iter().skip(1) {
            root.insert(interval.into());
        }

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
