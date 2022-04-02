use crate::rtree::dimension_type::DimensionType;

pub mod linear_cost_split;
use crate::rtree::nodes::prelude::*;
pub use linear_cost_split::LinearCostSplitting;

pub(crate) mod prelude {
    pub(crate) use super::SplittingStrategy;
}

pub(crate) trait SplittingStrategy<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    // TODO: Allow for NonLeafNode as well
    fn split(&self, node: &mut LeafNode<T, N, M>) -> LeafNode<T, N, M>;
}
