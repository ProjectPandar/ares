// Reached all-paths-reversible specialization from OrcaSlicer v2.4.2
// `ShortestPath.cpp`, including reached loop entity chaining, and its KD-tree and
// mutable-priority-queue dependencies.

mod chain;
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "pure O96 seam activates with the real O97 cursor")
)]
mod entity_chain;
mod kd_tree;
mod priority_queue;
#[cfg(test)]
mod tests;

pub(in crate::project_slice) use chain::{
    chain_and_reorder_extrusion_paths, chain_extrusion_loops,
};
#[cfg(test)]
pub(in crate::project_slice) use chain::{chain_extrusion_paths, reorder_extrusion_paths};
pub(in crate::project_slice) use entity_chain::reorder_thick_polylines;
#[cfg(test)]
pub(in crate::project_slice) use entity_chain::{ChainEntity, chain_and_reorder_entities};
