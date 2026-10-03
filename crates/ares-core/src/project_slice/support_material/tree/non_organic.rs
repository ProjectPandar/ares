//! Enabled non-Organic owners from `TreeSupport.cpp` (`USE_SUPPORT_3D=0`).

mod contacts;
pub(in crate::project_slice::support_material) use contacts::VerticalEnforcer;
mod data;
mod drawing;
mod drop_nodes;
mod drop_settings;
mod dropping;
pub(super) mod generation;
mod grouping;
mod heights;
mod line_cut;
mod merging;
mod movement;
mod neighbor_movement;
mod nodes;
mod position_map;
mod pruning;
mod raft_layers;
mod sampling;
mod smoothing;
mod spanning_tree;
mod toolpaths;
