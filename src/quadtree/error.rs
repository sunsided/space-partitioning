use std::{error, fmt};

#[derive(Debug)]
pub enum InsertError {
    /// The element that was about to be inserted was outside of the bounds of the QuadTree.
    OutOfBounds,
}

impl fmt::Display for InsertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::OutOfBounds => write!(f, "the element was outside of the tree bounds"),
        }
    }
}

impl error::Error for InsertError {}
