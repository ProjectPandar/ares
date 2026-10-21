//! `ShortestPath.cpp:2015-2041`: instance centers are reversible point segments.

use crate::project_slice::perimeters::classic::shortest_path::chain_segments_constrained_with_optional_start;

pub(super) fn chain_instance_order(centers: &[(i64, i64)], start_near: (i64, i64)) -> Vec<usize> {
    chain(centers, Some(start_near))
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
