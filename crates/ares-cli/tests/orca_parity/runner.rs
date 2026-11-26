//! Drives the OrcaSlicer CLI for the parity harness: builds a 3MF from
//! flattened presets and the shared cube model, slices it for the reference
//! G-code, and caches reference output keyed by the input digest.
#[path = "runner/bbl_eager_enforce.rs"]
mod bbl_eager_enforce;
#[path = "brim_flow_ratio_state.rs"]
mod brim_flow_ratio_state;
#[path = "runner/fan_speedup_sign.rs"]
mod fan_speedup_sign;
#[path = "runner/loaded_filament_dev_scalars.rs"]
mod loaded_filament_dev_scalars;
#[path = "runner/loaded_system_inherits_group.rs"]
mod loaded_system_inherits_group;
#[path = "runner/modifier_feature_filament.rs"]
mod modifier_feature_filament;
#[path = "runner/native_stream_registry.rs"]
mod native_stream_registry;
#[path = "runner/type2_tower_template_serialization.rs"]
mod type2_tower_template_serialization;
pub(crate) use native_stream_registry::native_stream_case;
#[path = "runner/processor_time.rs"]
mod processor_time;
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
#[path = "skirt_brim_seam_state.rs"]
mod skirt_brim_seam_state;
#[path = "skirt_minlength_multitool.rs"]
mod skirt_minlength_multitool;
#[path = "skirt_seam_scope.rs"]
mod skirt_seam_scope;
#[path = "skirt_support_speed_state.rs"]
mod skirt_support_speed_state;

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

#[path = "runner/stock_project_native_stream.rs"]
mod stock_project_native_stream;

#[path = "runner/repetier_cached_feedrate.rs"]
mod repetier_cached_feedrate;

use serde_json::{Map, Value};

#[path = "runner/first_travel_acceleration_ordering.rs"]
pub(crate) mod first_travel_acceleration_ordering;
#[path = "runner/grid_multiline_mode0.rs"]
mod grid_multiline_mode0;
#[path = "runner/lateral_multiline_overlap.rs"]
mod lateral_multiline_overlap;
#[path = "runner/object_layer_ranges.rs"]
mod object_layer_ranges;
#[path = "runner/rectilinear_multiline.rs"]
mod rectilinear_multiline;
#[path = "runner/triangles_multiline_mode1.rs"]
mod triangles_multiline_mode1;

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

#[path = "runner/application.rs"]
pub(crate) mod application;
#[path = "runner/filament_map.rs"]
pub(crate) mod filament_map;
#[path = "runner/object_copy_second_print_instance.rs"]
pub(crate) mod object_copy_second_print_instance;
#[cfg(test)]
#[path = "runner/profile_entry.rs"]
mod profile_entry;
#[path = "runner/vg1_two_filament_inputs.rs"]
pub(crate) mod vg1_two_filament_inputs;
#[cfg(target_os = "linux")]
pub(crate) use profile_entry::placed_cube;
#[path = "runner/command.rs"]
mod command;
#[path = "runner/command_cache.rs"]
mod command_cache;
#[cfg(test)]
#[path = "runner/native_api_cli.rs"]
pub(crate) mod native_api_cli;
#[path = "runner/stages.rs"]
pub(crate) mod stages;
#[path = "runner/user_overrides.rs"]
mod user_overrides;

include!("runner/test_modules.rs");

use stages::{FailureKind, Stage, StageError};
#[path = "runner/environment.rs"]
mod environment;
#[path = "runner/initialization.rs"]
mod initialization;
pub(crate) use environment::{
    official_reference_enabled, reference_available, seeded_reference_enabled, sweep_enabled,
};
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

#[path = "runner/fixture_support.rs"]
mod fixture_support;
pub(super) use fixture_support::preset_helpers::{
    PresetKind, apply_override, digest_slug, join_paths, repo_root, write_preset,
};
pub(super) use fixture_support::volume_fixture;

#[path = "option_coverage/safc_reference.rs"]
pub(super) mod safc_reference;
#[path = "runner/spiral_transition_e_precision.rs"]
mod spiral_transition_e_precision;
