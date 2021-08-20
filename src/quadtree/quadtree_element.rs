use crate::quadtree::AABB;

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
