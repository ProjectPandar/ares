//! Organic configuration and algorithms owned by `TreeSupportCommon.hpp`.

pub(super) use mesh_settings::MeshGroupSettings;
pub(super) use settings::TreeSettings;

mod area_growth;
mod branch_mesh;
mod drawing;
pub(in super::super) mod generation;
mod infill_lines;
mod initial_areas;
mod interface_placer;
mod interfaces;
mod layer_growth;
mod layer_index;
mod merge_areas;
mod mesh_settings;
mod move_inside;
mod nodes;
mod output;
mod pair_merge;
mod pathing;
mod raft;
mod rich_interface_placer;
mod roof_sampler;
mod settings;
mod single_area;
mod smoothing;
mod support_element;
mod tip_lines;
mod tip_sampling;
