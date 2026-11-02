//! Ordinary support rewrite of `Support/SupportMaterial.cpp`.
//! Geometry and toolpath generation; G-code integration remains open.

pub(super) use extrusion_path::{SupportExtrusionEntity, SupportExtrusionPath};
pub(super) use install_layers::InstalledSupportLayer;
pub(super) use necessity::check as check_necessity;
pub(super) use project::generate;

mod adhesion;
mod base_layers;
mod bottom_contacts;
mod bottom_projection;
mod bridging_perimeters;
mod contact_layer;
mod contact_placement;
mod contact_polygons;
mod detect_contacts;
mod extruded_layer;
mod extrusion_path;
mod facet_projection;
mod first_layer;
mod grid_generation;
mod install_layers;
mod interface_counts;
mod interface_grid;
mod interface_loops;
mod intermediate_heights;
mod ironing;
mod layer_index;
mod layer_input;
mod necessity;
mod overhang_clusters;
mod overhangs;
mod overlap_modulation;
mod pattern;
mod project;
mod remove_bridges;
mod sharp_tail_propagation;
mod style;
mod support_projection;
mod toolpath_cache;
mod toolpath_parameters;
mod toolpaths;
mod top_contacts;
mod tree;
mod trim_object;
mod trim_top_contacts;
