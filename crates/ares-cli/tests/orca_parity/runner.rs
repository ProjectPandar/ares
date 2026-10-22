//! Drives the OrcaSlicer CLI for the parity harness: builds a 3MF from
//! flattened presets and the shared cube model, slices it for the reference
//! G-code, and caches reference output keyed by the input digest.
#[path = "runner/bbl_eager_enforce.rs"]
mod bbl_eager_enforce;
#[path = "runner/bbl_type1_categories.rs"]
mod bbl_type1_categories;
#[path = "runner/bbl_type1_tower.rs"]
mod bbl_type1_tower;
#[path = "runner/crosshatch_dont_connect.rs"]
mod crosshatch_dont_connect;
#[path = "runner/fan_layer_tag.rs"]
mod fan_layer_tag;
#[path = "runner/fan_reader_xyz.rs"]
mod fan_reader_xyz;
#[path = "runner/fan_speedup_sign.rs"]
mod fan_speedup_sign;
#[path = "runner/loaded_system_inherits_group.rs"]
mod loaded_system_inherits_group;
#[path = "runner/modifier_feature_filament.rs"]
mod modifier_feature_filament;
#[path = "runner/native_stream_case.rs"]
pub(crate) mod native_stream_case;
#[path = "runner/placeholder.rs"]
mod placeholder;
#[path = "runner/resource_item_ignore.rs"]
mod resource_item_ignore;
#[path = "runner/resource_part_build.rs"]
mod resource_part_build;
#[path = "runner/resource_triangle_defaults.rs"]
mod resource_triangle_defaults;
#[path = "runner/resource_vertex_defaults.rs"]
mod resource_vertex_defaults;
#[path = "runner/scarf_conditional.rs"]
mod scarf_conditional;
#[path = "runner/seam_physical_plane.rs"]
mod seam_physical_plane;
#[path = "runner/seam_rear.rs"]
mod seam_rear;
#[path = "skirt_seam_scope.rs"]
mod skirt_seam_scope;

#[path = "runner/concentric_sparse_multiline.rs"]
mod concentric_sparse_multiline;

#[cfg(test)]
#[path = "runner/anchoring_zaa/mod.rs"]
mod anchoring_zaa;

#[path = "runner/modifier_sparse_loss.rs"]
mod modifier_sparse_loss;

use std::path::{Path, PathBuf};

#[path = "runner/tpms_d_rotation_precision.rs"]
mod tpms_d_rotation_precision;

#[path = "runner/adaptive_cubic_precision.rs"]
mod adaptive_cubic_precision;

#[path = "runner/aligned_rectilinear_multiline.rs"]
mod aligned_rectilinear_multiline;
#[path = "runner/aligned_template_identity.rs"]
mod aligned_template_identity;

#[path = "runner/native_ksr_reference_migration.rs"]
mod native_ksr_reference_migration;

#[path = "runner/repetier_cached_feedrate.rs"]
mod repetier_cached_feedrate;

use serde_json::{Map, Value};

#[path = "runner/grid_multiline_mode0.rs"]
mod grid_multiline_mode0;
#[path = "runner/lateral_multiline_overlap.rs"]
mod lateral_multiline_overlap;
#[path = "runner/rectilinear_multiline.rs"]
mod rectilinear_multiline;
#[path = "runner/triangles_multiline_mode1.rs"]
mod triangles_multiline_mode1;

#[path = "runner/object_layer_ranges.rs"]
mod object_layer_ranges;

#[path = "runner/object_layer_profiles.rs"]
mod object_layer_profiles;

#[path = "runner/config_block_roundtrip.rs"]
mod config_block_roundtrip;
#[path = "runner/native_plate_origin_migration.rs"]
mod native_plate_origin_migration;

#[path = "runner/native_travel_reference_migration.rs"]
mod native_travel_reference_migration;

#[path = "runner/native_arachne_prisms_migration.rs"]
mod native_arachne_prisms_migration;

#[path = "runner/native_exclude_outline_migration.rs"]
mod native_exclude_outline_migration;

#[path = "runner/native_bbl_boundary_migration.rs"]
mod native_bbl_boundary_migration;

#[path = "runner/native_jerk_semm_migration.rs"]
mod native_jerk_semm_migration;

#[path = "runner/mesh_edge_pairing.rs"]
mod mesh_edge_pairing;
#[path = "runner/mesh_odd_order_admission.rs"]
mod mesh_odd_order_admission;
#[path = "runner/modifier_bottom_shell.rs"]
mod modifier_bottom_shell;
#[path = "runner/modifier_top_shell.rs"]
mod modifier_top_shell;
#[path = "runner/modifier_wall_loops.rs"]
mod modifier_wall_loops;
#[path = "runner/object_range_epsilon.rs"]
mod object_range_epsilon;
#[path = "runner/stock_validation.rs"]
mod stock_validation;
#[path = "runner/vendor_profile_compatibility.rs"]
mod vendor_profile_compatibility;

#[path = "runner/adjacent_color.rs"]
mod adjacent_color;
#[path = "runner/adjacent_motion.rs"]
mod adjacent_motion;

#[cfg(test)]
#[path = "runner/tiq2_footer_templates.rs"]
mod tiq2_footer_templates;
#[cfg(test)]
#[path = "runner/tiq2_footer_whole.rs"]
mod tiq2_footer_whole;
#[cfg(test)]
#[path = "runner/tiq2_retained_fan.rs"]
mod tiq2_retained_fan;
#[path = "runner/type2_copies.rs"]
pub(crate) mod type2_copies;
#[path = "runner/vg1_inputs.rs"]
pub(crate) mod vg1_inputs;

#[cfg(test)]
#[path = "runner/apa_m_active.rs"]
mod apa_m_active;
#[cfg(test)]
#[path = "runner/apa_m_pa_off.rs"]
mod apa_m_pa_off;
#[path = "runner/application.rs"]
pub(crate) mod application;
#[cfg(test)]
#[path = "runner/application_tests.rs"]
mod application_tests;
#[cfg(test)]
#[path = "runner/arachne_counterbore.rs"]
mod arachne_counterbore;
#[cfg(test)]
#[path = "runner/arachne_counterbore_none.rs"]
mod arachne_counterbore_none;
#[cfg(test)]
#[path = "runner/arachne_open_wall_order.rs"]
mod arachne_open_wall_order;
#[cfg(test)]
#[path = "runner/cache_contract_tests.rs"]
mod cache_contract_tests;
#[cfg(test)]
#[path = "runner/classic_layer_id.rs"]
mod classic_layer_id;
#[cfg(test)]
#[path = "runner/clipped_loop_motion.rs"]
mod clipped_loop_motion;
#[cfg(test)]
#[path = "runner/compatible_regions.rs"]
mod compatible_regions;
#[cfg(test)]
#[path = "runner/costrings.rs"]
mod costrings;
#[cfg(test)]
#[path = "runner/counterbore_filled.rs"]
mod counterbore_filled;
#[cfg(test)]
#[path = "runner/counterbore_n_split.rs"]
mod counterbore_n_split;
#[cfg(test)]
#[path = "runner/counterbore_partiallybridge.rs"]
mod counterbore_partiallybridge;

#[cfg(test)]
#[path = "runner/direct_stl_override_bookkeeping.rs"]
mod direct_stl_override_bookkeeping;
#[cfg(test)]
#[path = "runner/embedded_script_whole_file.rs"]
mod embedded_script_whole_file;
#[cfg(test)]
#[path = "runner/empty_tools.rs"]
mod empty_tools;
#[path = "runner/filament_map.rs"]
pub(crate) mod filament_map;
#[cfg(test)]
#[path = "runner/filament_map_ramming.rs"]
mod filament_map_ramming;
#[cfg(test)]
#[path = "runner/filament_map_tests.rs"]
mod filament_map_tests;
#[cfg(test)]
#[path = "runner/filament_self_index.rs"]
mod filament_self_index;
#[cfg(test)]
#[path = "runner/generated_printer_ids.rs"]
mod generated_printer_ids;
#[cfg(test)]
#[path = "runner/infill_first_ironing.rs"]
mod infill_first_ironing;
#[cfg(test)]
#[path = "runner/initialization_tests.rs"]
mod initialization_tests;
#[cfg(test)]
#[path = "runner/initialized_loop_chain.rs"]
mod initialized_loop_chain;
#[cfg(test)]
#[path = "runner/initialized_overhang_intersections.rs"]
mod initialized_overhang_intersections;
#[cfg(test)]
#[path = "runner/j1_runtime_used_mask.rs"]
mod j1_runtime_used_mask;
#[cfg(test)]
#[path = "runner/j1_top_surface_lifecycle.rs"]
mod j1_top_surface_lifecycle;
#[cfg(test)]
#[path = "runner/lifecycle_tests.rs"]
mod lifecycle_tests;
#[cfg(test)]
#[path = "runner/multi_object_skirt.rs"]
mod multi_object_skirt;
#[cfg(test)]
#[path = "runner/native_availability_tests.rs"]
mod native_availability_tests;
#[cfg(test)]
#[path = "runner/nearest_seam_gaussian.rs"]
mod nearest_seam_gaussian;
#[cfg(test)]
#[path = "runner/object_layer_height_ranges.rs"]
mod object_layer_height_ranges;
#[cfg(test)]
#[path = "runner/object_range_regions.rs"]
mod object_range_regions;
#[cfg(test)]
#[path = "runner/overhang_reverse.rs"]
mod overhang_reverse;
#[cfg(test)]
#[path = "runner/path_tests.rs"]
mod path_tests;
#[cfg(test)]
#[path = "runner/plate_custom_gcode_emission.rs"]
mod plate_custom_gcode_emission;
#[path = "runner/print_sequence_by_object.rs"]
mod print_sequence_by_object;
#[cfg(test)]
#[path = "runner/processor_migration.rs"]
mod processor_migration;
#[cfg(test)]
#[path = "runner/profile_entry.rs"]
mod profile_entry;
pub(crate) use profile_entry::placed_cube;
#[cfg(test)]
#[path = "runner/native_api_cli.rs"]
pub(crate) mod native_api_cli;

#[path = "runner/concentric_ironing_order.rs"]
mod concentric_ironing_order;
#[cfg(test)]
#[path = "runner/profile_entry_3mf.rs"]
mod profile_entry_3mf;
#[cfg(test)]
#[path = "runner/profile_inherits_group.rs"]
mod profile_inherits_group;
#[cfg(test)]
#[path = "runner/profile_legacy_overlay.rs"]
mod profile_legacy_overlay;
#[cfg(test)]
#[path = "runner/profile_overlay_default_gcodes.rs"]
mod profile_overlay_default_gcodes;
#[cfg(test)]
#[path = "runner/project_export_metadata.rs"]
mod project_export_metadata;
#[cfg(test)]
#[path = "runner/scarf_default_threshold.rs"]
mod scarf_default_threshold;
#[cfg(test)]
#[path = "runner/seam_layer_height.rs"]
mod seam_layer_height;
#[cfg(test)]
#[path = "runner/seam_object_bias.rs"]
mod seam_object_bias;
#[cfg(test)]
#[path = "runner/seam_visibility_eigen_distance.rs"]
mod seam_visibility_eigen_distance;
#[cfg(test)]
#[path = "runner/seam_visibility_search_radius.rs"]
mod seam_visibility_search_radius;
#[path = "runner/simple_rotation_list.rs"]
mod simple_rotation_list;
#[cfg(test)]
#[path = "runner/small_loop_speed.rs"]
mod small_loop_speed;
#[cfg(test)]
#[path = "runner/stage_tests.rs"]
mod stage_tests;
#[path = "runner/stages.rs"]
pub(crate) mod stages;
#[cfg(test)]
#[path = "runner/stl_model_label.rs"]
mod stl_model_label;
#[cfg(test)]
#[path = "runner/stl_variant_whole_file.rs"]
mod stl_variant_whole_file;
#[cfg(test)]
#[path = "runner/support_annotations.rs"]
mod support_annotations;
#[cfg(test)]
#[path = "runner/support_modifier.rs"]
mod support_modifier;
#[cfg(test)]
#[path = "runner/template_rng.rs"]
mod template_rng;
#[cfg(test)]
#[path = "runner/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "runner/tree_support.rs"]
mod tree_support;

#[path = "runner/command.rs"]
mod command;
#[cfg(test)]
#[path = "runner/strict_reference_tests.rs"]
mod strict_reference_tests;

#[path = "runner/user_overrides.rs"]
mod user_overrides;

#[cfg(test)]
#[path = "runner/wrapper_tests.rs"]
mod wrapper_tests;
#[cfg(test)]
#[path = "runner/x2d_flush_matrix.rs"]
mod x2d_flush_matrix;

#[cfg(test)]
#[path = "runner/wipe_tags.rs"]
mod wipe_tags;

use stages::{FailureKind, Stage, StageError};
#[path = "runner/environment.rs"]
mod environment;
#[path = "runner/initialization.rs"]
mod initialization;
pub(crate) use environment::reference_enabled;
#[path = "runner/cache.rs"]
mod cache;
pub(crate) use cache::EarlyExit;
#[path = "runner/cache_identity.rs"]
mod cache_identity;
#[path = "runner/native_replay_reference_migration.rs"]
pub(super) mod native_replay_reference_migration;

#[path = "runner/cases.rs"]
mod cases;
pub(super) use cases::{CaseInputs, ExportedCase, ParityCase};

pub(super) struct OrcaRunner {
    bin: PathBuf,
    work: PathBuf,
    /// Content-keyed export and reference outputs; the private work
    /// directory unless ARES_ORCA_CACHE_DIR shares them across runs.
    cache: PathBuf,
    producer: std::sync::OnceLock<cache_identity::Producer>,
}

impl OrcaRunner {
    /// Path to the executable invoked for reference production — used by
    /// the sweep preflight to verify the wrapper actually runs.
    pub(super) fn bin(&self) -> &PathBuf {
        &self.bin
    }
}

#[path = "runner/fixture_support.rs"]
mod fixture_support;
pub(super) use fixture_support::preset_helpers::{
    PresetKind, apply_override, digest_slug, join_paths, repo_root, write_preset,
};
pub(super) use fixture_support::volume_fixture;
pub(crate) use fixture_support::wrapping_input;

#[path = "option_coverage/safc_reference.rs"]
pub(super) mod safc_reference;
#[path = "runner/spiral_transition_e_precision.rs"]
mod spiral_transition_e_precision;
