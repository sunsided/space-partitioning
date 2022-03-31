use crate::rtree::bounding_box::BoundingBox;
use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::{Entry, LeafNode};

pub(crate) trait SplittingStrategy<T, const N: usize, const M: usize>
where
    T: DimensionType,
{
    // TODO: Allow for NonLeafNode as well
    fn split(&self, node: &mut LeafNode<T, N, M>) -> LeafNode<T, N, M>;
}

#[derive(Debug, Default, Clone)]
pub struct LinearCostSplitting {}

impl<T, const N: usize, const M: usize> SplittingStrategy<T, N, M> for LinearCostSplitting
where
    T: DimensionType,
{
    fn split(&self, node: &mut LeafNode<T, N, M>) -> LeafNode<T, N, M> {
        let area = &node.bb;
        let entries = &mut node.entries;

        // Find the best candidates and pop them from the set in reverse order (highest index first).
        let (best_a, best_b) = linear_pick_seeds(&entries, &area);
        let best_b = entries.remove(best_b);
        let best_a = entries.remove(best_a);

        let mut box_a = best_a.bb.clone();
        let mut box_b = best_b.bb.clone();

        let mut group_a = vec![best_a];
        let mut group_b = vec![best_b];

        // TODO: If one group has so few entries that the rest must be assigned for it to have the minimum number of elements, assign the rest and stop.
        while let Some(item) = entries.pop() {
            let a_grown = box_a.get_grown(&item.bb);
            let b_grown = box_b.get_grown(&item.bb);

            // Assign to the box requiring a smaller increase in size.
            if a_grown.area_increase < b_grown.area_increase {
                box_a = a_grown.bb;
                group_a.push(item);
                continue;
            }
            if a_grown.area_increase > b_grown.area_increase {
                box_b = b_grown.bb;
                group_b.push(item);
                continue;
            }

            // In case of a tie, assign to the smaller box.
            if a_grown.area < b_grown.area {
                box_a = a_grown.bb;
                group_a.push(item);
                continue;
            }
            if a_grown.area > b_grown.area {
                box_b = b_grown.bb;
                group_b.push(item);
                continue;
            }

            // In case of a tie, assign to the box with fewer items,
            // or any box.
            if group_a.len() < group_b.len() {
                box_a = a_grown.bb;
                group_a.push(item);
                continue;
            }

            box_b = b_grown.bb;
            group_b.push(item);
        }

        node.bb = box_a;
        node.entries = group_a;

        LeafNode {
            bb: box_b,
            entries: group_b,
        }
    }
}

fn linear_pick_seeds<T, const N: usize>(
    set: &[Entry<T, N>],
    area: &BoundingBox<T, N>,
) -> (usize, usize)
where
    T: DimensionType,
{
    debug_assert!(set.len() > 1);
    let mut highest_lows = vec![(T::min_value(), 0usize); N];
    let mut lowest_highs = vec![(T::max_value(), 0usize); N];

    for item_idx in 0..set.len() {
        let bb = &set[item_idx].bb;
        for dim in 0..N {
            let extent = bb.dims[dim];

            // Find the entry of the highest low dimension,
            // i.e. the start coordinate being the highest.
            if extent.start > highest_lows[dim].0 {
                highest_lows[dim] = (extent.start, item_idx);
            }

            // Find the entry of the lowest high dimension.
            // i.e. the end coordinate being the lowest.
            if extent.end < lowest_highs[dim].0 {
                lowest_highs[dim] = (extent.end, item_idx);
            }
        }
    }

    let mut highest_separation = T::min_value();
    let (mut best_a, mut best_b) = (0usize, 0usize);

    for dim in 0..N {
        let width = area.dims[dim].len();

        let lo_hi = lowest_highs[dim].0;
        let hi_lo = highest_lows[dim].0;

        // Using a makeshift "abs" here to avoid requiring the Ord or Real trait.
        // TODO: Is the "abs" correct here? Issue might be arising from flipped coordinate systems (0 top-left or bottom-left)
        let sep_a = lo_hi - hi_lo;
        let sep_b = hi_lo - lo_hi;
        let separation = if sep_a > sep_b { sep_a } else { sep_b };

        let normalized_separation = separation / width;
        if normalized_separation > highest_separation {
            highest_separation = separation;
            best_a = lowest_highs[dim].1;
            best_b = highest_lows[dim].1;
        }
    }

    debug_assert_ne!(best_a, best_b);
    let low_idx = best_a.min(best_b);
    let high_idx = best_a.max(best_b);
    (low_idx, high_idx)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_works() {
        let mut old_node = LeafNode::<f64, 2, 3>::default();
        old_node.insert(0, [16.0..=68.0, 23.0..=35.0].into());
        old_node.insert(1, [55.0..=68.0, 12.0..=148.0].into());
        old_node.insert(2, [82.0..=94.0, 12.0..=148.0].into());
        old_node.insert(3, [82.0..=145.0, 30.0..=42.0].into());

        let strategy = LinearCostSplitting {};
        let new_node = strategy.split(&mut old_node);

        // Group a contains both horizontal items.
        debug_assert!(old_node.entries.iter().any(|x| x.id == 0));
        debug_assert!(old_node.entries.iter().any(|x| x.id == 3));

        // Group a contains both vertical items.
        debug_assert!(new_node.entries.iter().any(|x| x.id == 1));
        debug_assert!(new_node.entries.iter().any(|x| x.id == 2));
    }
}
