// Reached all-paths-reversible specialization from OrcaSlicer v2.4.2
// `ShortestPath.cpp`, including reached loop entity chaining, and its KD-tree and
// mutable-priority-queue dependencies.

mod chain;
mod entity_chain;
mod kd_tree;
mod polyline_chain;
mod priority_queue;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(in crate::project_slice) use chain::reorder_extrusion_paths;
pub(in crate::project_slice) use chain::{
    chain_and_reorder_extrusion_paths, chain_extrusion_entities, chain_loops_by_first_points,
    chain_segments_constrained_with_optional_start,
};
#[cfg(test)]
pub(in crate::project_slice) use chain::{chain_extrusion_loops, chain_extrusion_paths};
#[cfg(test)]
pub(in crate::project_slice) use entity_chain::chain_and_reorder_entities;
pub(in crate::project_slice) use entity_chain::{
    ChainEntity, chain_and_reorder_entities_flips, chain_and_reorder_entities_with_optional_start,
    reorder_thick_polylines,
};
pub(crate) use polyline_chain::chain_polylines;
