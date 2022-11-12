use crate::rtree::bounding_box::{BoundingBox, BoxAndArea};
use crate::rtree::dimension_type::DimensionType;
use crate::rtree::nodes::node_traits::HasBoundingBox;
use crate::rtree::nodes::prelude::Node;
use crate::rtree::splitting_strategies::{SplitGroup, SplitResult, SplittingStrategy};
use arrayvec::ArrayVec;

#[derive(Debug, Default, Clone)]
pub struct LinearCostSplitting {}

impl<T, TEntry, const N: usize, const M: usize> SplittingStrategy<T, TEntry, N, M>
    for LinearCostSplitting
where
    T: DimensionType,
    TEntry: HasBoundingBox<T, N>,
{
    fn split(
        &self,
        area: &BoundingBox<T, N>,
        existing_entries: &mut ArrayVec<TEntry, M>,
        new_entry: TEntry,
    ) -> SplitResult<T, TEntry, N, M> {
        // Ensure the area contains the new element as well.
        let area = area.clone().into_grown(new_entry.to_bb());

        // Find the best candidates and remove them from the set.
        let (best_a, best_b) = linear_pick_seeds(&existing_entries, &new_entry, &area);

        let (best_a, best_b) = match (best_a, best_b) {
            (None, None) => unreachable!(),
            (Some(best_a), None) => (existing_entries.remove(best_a), new_entry),
            (None, Some(best_b)) => (new_entry, existing_entries.remove(best_b)),
            (Some(best_a), Some(best_b)) => {
                // Pop them from the set in reverse order (highest index first).
                let best_b = existing_entries.remove(best_b);
                let best_a = existing_entries.remove(best_a);

                // In this case, the new entry was not accounted for. The removal of at least
                // one element here however leaves enough space to add it to the list for further
                // processing, as if it had been added before.
                existing_entries.push(new_entry);

                (best_a, best_b)
            }
        };

        let mut box_a = best_a.to_bb();
        let mut box_b = best_b.to_bb();

        let mut group_a: ArrayVec<_, M> = ArrayVec::new();
        let mut group_b: ArrayVec<_, M> = ArrayVec::new();
        group_a.push(best_a);
        group_b.push(best_b);

        // TODO: If one group has so few entries that the rest must be assigned for it to have the minimum number of elements, assign the rest and stop.
        while let Some(item) = existing_entries.pop() {
            let a_grown = box_a.get_grown(item.to_bb());
            let b_grown = box_b.get_grown(item.to_bb());

            match decide_group(&a_grown, &b_grown, group_a.len(), group_b.len()) {
                Decision::Left => {
                    box_a = a_grown.bb;
                    group_a.push(item);
                }
                Decision::Right => {
                    box_b = b_grown.bb;
                    group_b.push(item);
                }
            }
        }

        SplitResult {
            first: SplitGroup {
                bb: box_a,
                entries: group_a,
            },
            second: SplitGroup {
                bb: box_b,
                entries: group_b,
            },
        }
    }
}

/// Picks two seed nodes from the provided entries and returns their indices.
///
/// ## Arguments
/// * `entries` - The entries to choose from.
/// * `new_entry` - The new entry to add.
/// * `area` - The minimal bounding box of all entries.
///
/// ## Returns
/// A tuple of two distinct indexes.
/// The entries are sorted in ascending order such that elements can be removed from
/// a vector back to front.
/// A value of [`Option<usize>::None`] indicates the new item to be added.
fn linear_pick_seeds<T, TEntry, const N: usize>(
    entries: &[TEntry],
    new_entry: &TEntry,
    area: &BoundingBox<T, N>,
) -> (Option<usize>, Option<usize>)
where
    T: DimensionType,
    TEntry: HasBoundingBox<T, N>,
{
    debug_assert!(entries.len() > 1);
    let mut highest_lows = [(T::min_value(), None); N];
    let mut lowest_highs = [(T::max_value(), None); N];

    for item_idx in 0..entries.len() {
        let bb = entries[item_idx].to_bb();
        for dim in 0..N {
            let extent = bb.dims[dim];

            // Find the entry of the highest low dimension,
            // i.e. the start coordinate being the highest.
            if extent.start > highest_lows[dim].0 {
                highest_lows[dim] = (extent.start, Some(item_idx));
            }

            // Find the entry of the lowest high dimension.
            // i.e. the end coordinate being the lowest.
            if extent.end < lowest_highs[dim].0 {
                lowest_highs[dim] = (extent.end, Some(item_idx));
            }
        }
    }

    // Repeat with the new entry.
    let bb = new_entry.to_bb();
    for dim in 0..N {
        let extent = bb.dims[dim];

        // Find the entry of the highest low dimension,
        // i.e. the start coordinate being the highest.
        if extent.start > highest_lows[dim].0 {
            highest_lows[dim] = (extent.start, None);
        }

        // Find the entry of the lowest high dimension.
        // i.e. the end coordinate being the lowest.
        if extent.end < lowest_highs[dim].0 {
            lowest_highs[dim] = (extent.end, None);
        }
    }

    let mut highest_separation = T::min_value();
    let (mut best_a, mut best_b) = (None, None);

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

enum Decision {
    Left,
    Right,
}

fn decide_group<T: DimensionType, const N: usize>(
    a: &BoxAndArea<T, N>,
    b: &BoxAndArea<T, N>,
    a_count: usize,
    b_count: usize,
) -> Decision {
    // Assign to the box requiring a smaller increase in size.
    if a.area_increase < b.area_increase {
        return Decision::Left;
    }

    if a.area_increase > b.area_increase {
        return Decision::Right;
    }

    // In case of a tie, assign to the smaller box.
    if a.area < b.area {
        return Decision::Left;
    }

    if a.area > b.area {
        return Decision::Right;
    }

    // In case of a tie, assign to the box with fewer items,
    // or any box.
    if a_count < b_count {
        return Decision::Left;
    }

    return Decision::Right;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rtree::nodes::rtree_leaf::IndexRecordEntry;

    #[test]
    fn split_works() {
        let mut existing_entries = ArrayVec::from([
            IndexRecordEntry::new(0, [16.0..=68.0, 23.0..=35.0]),
            IndexRecordEntry::new(1, [55.0..=68.0, 12.0..=148.0]),
            IndexRecordEntry::new(2, [82.0..=94.0, 12.0..=148.0]),
        ]);

        let new_entry = IndexRecordEntry::new(3, [82.0..=145.0, 30.0..=42.0]);

        let strategy = LinearCostSplitting {};
        let result: SplitResult<_, _, 2, 3> = strategy.split(
            &existing_entries.as_slice().to_bb(),
            &mut existing_entries,
            new_entry,
        );

        // Group a contains both horizontal items.
        debug_assert!(result.first.entries.iter().any(|x| x.id == 0));
        debug_assert!(result.first.entries.iter().any(|x| x.id == 3));

        // Group a contains both vertical items.
        debug_assert!(result.second.entries.iter().any(|x| x.id == 1));
        debug_assert!(result.second.entries.iter().any(|x| x.id == 2));
    }
}
