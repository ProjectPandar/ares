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

pub(in crate::project_slice) use chain::{
    chain_and_reorder_extrusion_paths, chain_extrusion_loops,
};
#[cfg(test)]
pub(in crate::project_slice) use chain::{chain_extrusion_paths, reorder_extrusion_paths};
pub(in crate::project_slice) use entity_chain::{
    ChainEntity, chain_and_reorder_entities, reorder_thick_polylines,
};
pub(crate) use polyline_chain::chain_polylines;
