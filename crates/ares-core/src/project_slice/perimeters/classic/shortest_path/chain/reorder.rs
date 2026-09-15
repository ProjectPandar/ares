//! Extrusion-path reorder entry points (chain_and_reorder wrappers).

use super::{chain_extrusion_paths, chain_multiple, chain_points};
use crate::project_slice::perimeters::classic::materialize::ExtrusionPath;

pub(in crate::project_slice) fn reorder_extrusion_paths(
    paths: &mut Vec<ExtrusionPath>,
    chain: &[(usize, bool)],
) {
    debug_assert_eq!(paths.len(), chain.len());
    if paths.is_empty() {
        return;
    }
    let mut source: Vec<_> = std::mem::take(paths).into_iter().map(Some).collect();
    paths.reserve(chain.len());
    for &(index, reverse) in chain {
        let mut path = source[index].take().expect("chain indices are unique");
        if reverse {
            path.reverse();
        }
        paths.push(path);
    }
}

pub(in crate::project_slice) fn chain_and_reorder_extrusion_paths(
    paths: &mut Vec<ExtrusionPath>,
    start_near: [crate::geometry::Coord; 2],
) {
    let chain = chain_extrusion_paths(paths, Some(start_near));
    reorder_extrusion_paths(paths, &chain);
}
