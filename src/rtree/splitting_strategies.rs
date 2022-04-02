use crate::rtree::dimension_type::DimensionType;

pub mod linear_cost_split;
use crate::rtree::nodes::prelude::*;
pub use linear_cost_split::LinearCostSplitting;

pub(crate) mod prelude {
    pub(crate) use super::SplittingStrategy;
}

pub(crate) trait SplittingStrategy<T, TNode, const N: usize>
where
    T: DimensionType,
    TNode: Node<T, N>,
{
    fn split(&self, node: &mut TNode) -> TNode;
}
