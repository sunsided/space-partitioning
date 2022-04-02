use crate::rtree::dimension_type::DimensionType;

pub mod linear_cost_split;
use crate::rtree::bounding_box::BoundingBox;
pub use linear_cost_split::LinearCostSplitting;

pub(crate) mod prelude {
    pub(crate) use super::SplittingStrategy;
}

/// Trait for strategies used to split overfull nodes.
///
/// Some well-known approaches are:
///
/// - Exhaustive
/// - Quadratic-Cost
/// - Linear-Cost
pub(crate) trait SplittingStrategy<T, TEntry, const N: usize>
where
    T: DimensionType,
{
    fn split(
        &self,
        area: &BoundingBox<T, N>,
        entries: &mut Vec<TEntry>,
    ) -> SplitResult<T, TEntry, N>;
}

/// A single group that was created while splitting results.
pub(crate) struct SplitGroup<T, TEntry, const N: usize>
where
    T: DimensionType,
{
    /// The minimum bounding box of the entries.
    pub bb: BoundingBox<T, N>,

    /// The vector of entries.
    pub entries: Vec<TEntry>,
}

/// Tuple for capturing the two groups that result from the splitting operation.
pub(crate) struct SplitResult<T, TEntry, const N: usize>
where
    T: DimensionType,
{
    /// The first of two groups.
    pub first: SplitGroup<T, TEntry, N>,

    /// The second of two groups.
    pub second: SplitGroup<T, TEntry, N>,
}
