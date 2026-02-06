use crate::octree::{free_list, AABB};

/// Alias for all traits required for an element ID.
pub trait ElementIdType: Default + std::cmp::Eq + std::hash::Hash + Copy {}

impl<T> ElementIdType for T where T: Default + std::cmp::Eq + std::hash::Hash + Copy {}

/// Represents an element in the OctTree.
#[derive(Debug, PartialEq, Eq, Default, Copy, Clone)]
pub struct OctTreeElement<ElementId = u32>
where
    ElementId: ElementIdType,
{
    pub rect: AABB,
    pub id: ElementId,
}

impl<ElementId> OctTreeElement<ElementId>
where
    ElementId: ElementIdType,
{
    pub fn new(id: ElementId, rect: AABB) -> Self {
        Self { id, rect }
    }
}

/// Represents an element node in the octree.
#[derive(Debug, PartialEq, Eq, Default, Copy, Clone)]
pub(crate) struct OctTreeElementNode {
    pub next: free_list::IndexType,
    pub element_idx: free_list::IndexType,
}
