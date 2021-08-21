use crate::quadtree::{free_list, AABB};

/// Alias for all traits required for an element ID.
pub trait ElementIdType: Default + std::cmp::Eq + std::hash::Hash + Copy {}

/// Helper implementation to automatically derive the [`ElementIdType`] trait
impl<T> ElementIdType for T where T: Default + std::cmp::Eq + std::hash::Hash + Copy {}

/// Represents an element in the QuadTree.
#[derive(Debug, PartialEq, Eq, Default, Copy, Clone)]
pub struct QuadTreeElement<ElementId = u32>
where
    ElementId: ElementIdType,
{
    /// The axis-aligned bounding box of the element.
    pub rect: AABB,
    /// Stores the ID for the element (can be used to refer to external data).
    pub id: ElementId,
}

impl<ElementId> QuadTreeElement<ElementId>
where
    ElementId: ElementIdType,
{
    pub fn new(id: ElementId, rect: AABB) -> Self {
        Self { id, rect }
    }
}

/// Represents an element node in the quadtree.
///
/// # Remarks
/// An element (`QuadTreeElement`) is only inserted once to the quadtree no matter how many
/// cells it occupies. However, for each cell it occupies, an "element node" (`QuadTreeElementNode`)
/// is inserted which indexes that element.
#[derive(Debug, PartialEq, Eq, Default, Copy, Clone)]
pub(crate) struct QuadTreeElementNode {
    /// Points to the next element in the leaf node. A value of `free_list::SENTINEL`
    /// indicates the end of the list.
    pub next: free_list::IndexType,
    /// Stores the element index.
    pub element_idx: free_list::IndexType,
}
