//! `ShortestPath.cpp:2015-2041`: instance centers are reversible point segments.

use crate::project_slice::perimeters::classic::shortest_path::chain_segments_constrained_with_optional_start;

pub(super) fn chain_instance_order(centers: &[(i64, i64)], start_near: (i64, i64)) -> Vec<usize> {
    chain(centers, Some(start_near))
}

/// `GCode.cpp:2392-2430::sort_object_instances_by_model_order` reduce.
///
/// With `init_order` the function assigns `arrange_order` from the model object
/// index (`:2409-2410`) and then renumbers the sorted result to a unique `1..N`
/// (`:2415-2418`), so every key is unique and the `lower_bound` collect loop at
/// `:2422-2427` plus the re-sort at `:2429` collapse to "order by model object
/// index". Instances of one object share that key; libstdc++ `std::sort` is an
/// insertion sort for `n <= 16` and therefore keeps their iteration order there.
/// For `n > 16` the upstream relative order of same-object instances is *not*
/// reproduced here, and no rule is invented to stand in for it.
pub(super) fn model_order(model_indices: &[usize]) -> Vec<usize> {
    let mut order = (0..model_indices.len()).collect::<Vec<_>>();
    order.sort_by_key(|&index| model_indices[index]);
    order
}

pub(in crate::project_slice::gcode_emit) fn chain_without_start(
    centers: &[(i64, i64)],
) -> Vec<usize> {
    chain(centers, None)
}

fn chain(centers: &[(i64, i64)], start_near: Option<(i64, i64)>) -> Vec<usize> {
    let endpoints = centers
        .iter()
        .map(|&(x, y)| [[x, y]; 2])
        .collect::<Vec<_>>();
    chain_segments_constrained_with_optional_start(
        &endpoints,
        start_near.map(|(x, y)| [x, y]),
        &vec![true; centers.len()],
    )
    .into_iter()
    .map(|(index, _)| index)
    .collect()
}

#[cfg(test)]
mod tests;
