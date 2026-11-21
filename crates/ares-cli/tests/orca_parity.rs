//! OrcaSlicer parity suite: slices vendor printer profiles with both the
//! OrcaSlicer 2.4.2 CLI and Ares using the shared generator-only strict byte
//! comparator. Equality of supplied streams does not establish inventory coverage.
//!
//! Environment-gated: runs only when `ARES_ORCA_BIN` (or the repository
//! wrapper `scripts/orca-parity.sh`) names a working OrcaSlicer CLI.

#[path = "orca_parity/gcode_regression_cases.rs"]
mod gcode_regression_cases;
#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;
#[path = "orca_parity/standalone_thin_gap_fill.rs"]
mod standalone_thin_gap_fill;

#[path = "orca_parity/object_emission_cases.rs"]
mod object_emission_cases;

#[path = "orca_parity/chamber_used_extruder.rs"]
mod chamber_used_extruder;
#[path = "orca_parity/endpoint_contact.rs"]
mod endpoint_contact;
#[path = "orca_parity/exhaust_used_extruder.rs"]
mod exhaust_used_extruder;
#[path = "orca_parity/fan_mixed_timing.rs"]
mod fan_mixed_timing;
#[path = "orca_parity/interlocking_base.rs"]
mod interlocking_base;
#[path = "orca_parity/inward_displacement.rs"]
mod inward_displacement;
#[path = "orca_parity/paint_color.rs"]
mod paint_color;
#[path = "orca_parity/paint_color/activity.rs"]
mod paint_color_activity;
#[path = "orca_parity/painted_beam.rs"]
mod painted_beam;
#[path = "orca_parity/painted_cases.rs"]
mod painted_cases;
#[path = "orca_parity/painted_side_cut.rs"]
mod painted_side_cut;
#[path = "orca_parity/processor_sloped_seam.rs"]
mod processor_sloped_seam;
#[path = "orca_parity/thin_safezone.rs"]
mod thin_safezone;
#[path = "orca_parity/used_extruder_case.rs"]
mod used_extruder_case;
#[path = "orca_parity/vg1_export_arc_cache.rs"]
mod vg1_export_arc_cache;

#[path = "orca_parity/m400_priority.rs"]
mod m400_priority;

#[path = "orca_parity/classic_seam_overhang.rs"]
mod classic_seam_overhang;

#[path = "orca_parity/pre_external_wipe.rs"]
mod pre_external_wipe;

#[path = "orca_parity/closest_lines_runtime.rs"]
mod closest_lines_runtime;

#[path = "orca_parity/per_object_draft_triple.rs"]
mod per_object_draft_triple;

#[path = "orca_parity/per_object_draft_compositions.rs"]
mod per_object_draft_compositions;

#[path = "orca_parity/per_object_draft_shield.rs"]
mod per_object_draft_shield;

#[path = "orca_parity/per_object_skirt_min_brim.rs"]
mod per_object_skirt_min_brim;

#[path = "orca_parity/per_object_skirt_min_length.rs"]
mod per_object_skirt_min_length;

#[path = "orca_parity/spiral_overlap_transition.rs"]
mod spiral_overlap_transition;

#[path = "orca_parity/spiral_smooth_fixed_xy.rs"]
mod spiral_smooth_fixed_xy;

#[path = "orca_parity/spiral_reader_float_distance.rs"]
mod spiral_reader_float_distance;

#[path = "orca_parity/firmware_absolute_literal_e.rs"]
mod firmware_absolute_literal_e;

#[path = "orca_parity/lightning_initial_overhang_join.rs"]
mod lightning_initial_overhang_join;

#[path = "orca_parity/raft_contact_first_flow.rs"]
mod raft_contact_first_flow;

#[path = "orca_parity/placeholder_regex_full_match.rs"]
mod placeholder_regex_full_match;

#[path = "orca_parity/placeholder_regex_byte_semantics.rs"]
mod placeholder_regex_byte_semantics;

#[path = "orca_parity/spiral_placement_gate.rs"]
mod spiral_placement_gate;

#[path = "orca_parity/arachne_extra_overhang_call.rs"]
mod arachne_extra_overhang_call;

#[path = "orca_parity/scarf_remaining_flat_role.rs"]
mod scarf_remaining_flat_role;

#[path = "orca_parity/random_scarf_normalized_edge.rs"]
mod random_scarf_normalized_edge;

#[path = "orca_parity/staggered_seam_interpolation.rs"]
mod staggered_seam_interpolation;

#[path = "orca_parity/project_settings_substitution.rs"]
mod project_settings_substitution;

#[path = "orca_parity/combined_contact_components.rs"]
mod combined_contact_components;

#[path = "orca_parity/ordered_brim_owner.rs"]
mod ordered_brim_owner;

#[path = "orca_parity/print_global_brim_flow.rs"]
mod print_global_brim_flow;

#[path = "orca_parity/auto_brim_combined.rs"]
mod auto_brim_combined;
#[path = "orca_parity/per_object_skirt_brim.rs"]
mod per_object_skirt_brim;
#[path = "orca_parity/perobject_auto_admission.rs"]
mod perobject_auto_admission;

#[path = "orca_parity/config_floats_header_serialization.rs"]
mod config_floats_header_serialization;

#[path = "orca_parity/auto_brim_consumes_efc_clipped_groups.rs"]
mod auto_brim_consumes_efc_clipped_groups;
#[path = "orca_parity/auto_brim_exact_group_assignment.rs"]
mod auto_brim_exact_group_assignment;
#[path = "orca_parity/auto_brim_object_adhesion.rs"]
mod auto_brim_object_adhesion;
#[path = "orca_parity/auto_brim_object_speed.rs"]
mod auto_brim_object_speed;
#[path = "orca_parity/auto_brim_small_diagonal.rs"]
mod auto_brim_small_diagonal;
#[path = "orca_parity/auto_brim_tables.rs"]
mod auto_brim_tables;
#[path = "orca_parity/brim_ears_consumes_efc_outline.rs"]
mod brim_ears_consumes_efc_outline;
#[path = "orca_parity/config_float_header_serialization.rs"]
mod config_float_header_serialization;
#[path = "orca_parity/crosshatch_ordering.rs"]
mod crosshatch_ordering;
#[path = "orca_parity/efc_outline_witness_case.rs"]
mod efc_outline_witness_case;
#[path = "orca_parity/junction_deviation.rs"]
mod junction_deviation;
#[path = "orca_parity/per_object_skirt_brim_ears.rs"]
mod per_object_skirt_brim_ears;
#[path = "orca_parity/per_object_skirt_brim_types.rs"]
mod per_object_skirt_brim_types;
#[path = "orca_parity/per_object_skirt_groups.rs"]
mod per_object_skirt_groups;
#[path = "orca_parity/per_object_skirt_painted.rs"]
mod per_object_skirt_painted;
#[path = "orca_parity/per_object_skirt_support_raft.rs"]
mod per_object_skirt_support_raft;
#[path = "orca_parity/power_loss_recovery.rs"]
mod power_loss_recovery;
#[path = "orca_parity/skirt_per_object_flush_into_infill_wiping_pass.rs"]
mod skirt_per_object_flush_into_infill_wiping_pass;
#[path = "orca_parity/skirt_per_object_flush_into_objects.rs"]
mod skirt_per_object_flush_into_objects;
#[path = "orca_parity/skirt_per_object_flush_into_support.rs"]
mod skirt_per_object_flush_into_support;
#[path = "orca_parity/skirt_per_object_groups_with_tower_obstacle.rs"]
mod skirt_per_object_groups_with_tower_obstacle;
#[path = "orca_parity/skirt_per_object_tower_control.rs"]
mod skirt_per_object_tower_control;
#[path = "orca_parity/toolchange_layer_boundary_retract_wipe_timing.rs"]
mod toolchange_layer_boundary_retract_wipe_timing;
#[path = "orca_parity/toolchange_wipe_control.rs"]
mod toolchange_wipe_control;

#[path = "orca_parity/adaptive_octree_spacing.rs"]
mod adaptive_octree_spacing;
#[path = "orca_parity/adaptive_pressure_advance.rs"]
mod adaptive_pressure_advance;
#[path = "orca_parity/arachne_alternate_extra_wall.rs"]
mod arachne_alternate_extra_wall;
#[path = "orca_parity/arachne_fuzzy_f1.rs"]
mod arachne_fuzzy_f1;
#[path = "orca_parity/arachne_fuzzy_f2.rs"]
mod arachne_fuzzy_f2;
#[path = "orca_parity/artifacts.rs"]
mod artifacts;
#[path = "orca_parity/bridge_flow_admission.rs"]
mod bridge_flow_admission;
#[path = "orca_parity/bridge_over_infill_filament.rs"]
mod bridge_over_infill_filament;
#[path = "orca_parity/bridge_over_infill_zero_density.rs"]
mod bridge_over_infill_zero_density;
#[path = "orca_parity/brim_bed_region.rs"]
mod brim_bed_region;
#[path = "orca_parity/brim_bed_region_controls.rs"]
mod brim_bed_region_controls;
#[path = "orca_parity/brim_consumes_efc_region_polygons.rs"]
mod brim_consumes_efc_region_polygons;
#[path = "orca_parity/brim_ears_explicit_width_support_no_brim.rs"]
mod brim_ears_explicit_width_support_no_brim;
#[path = "orca_parity/brim_inner_only_explicit_width.rs"]
mod brim_inner_only_explicit_width;
#[path = "orca_parity/brim_outer_and_inner_support_no_brim_type_independent.rs"]
mod brim_outer_and_inner_support_no_brim_type_independent;
#[path = "orca_parity/brim_outer_only_explicit_width.rs"]
mod brim_outer_only_explicit_width;
#[path = "orca_parity/brim_painted_control.rs"]
mod brim_painted_control;
#[path = "orca_parity/brim_painted_ears_from_ear_points.rs"]
mod brim_painted_ears_from_ear_points;
#[path = "orca_parity/brim_painted_efc_control_no_snap.rs"]
mod brim_painted_efc_control_no_snap;
#[path = "orca_parity/brim_painted_efc_snaps_ear_centre.rs"]
mod brim_painted_efc_snaps_ear_centre;
#[path = "orca_parity/brim_painted_inner_control.rs"]
mod brim_painted_inner_control;
#[path = "orca_parity/brim_painted_inner_hole_point.rs"]
mod brim_painted_inner_hole_point;
#[path = "orca_parity/brim_physical_layer.rs"]
mod brim_physical_layer;
#[path = "orca_parity/byobject_owner_brim.rs"]
mod byobject_owner_brim;
#[path = "orca_parity/byobject_perobject_multi_copy_brim.rs"]
mod byobject_perobject_multi_copy_brim;
#[path = "orca_parity/byobject_perobject_skirt.rs"]
mod byobject_perobject_skirt;
#[path = "orca_parity/byobject_transition_comments.rs"]
mod byobject_transition_comments;
#[path = "orca_parity/change_filament_flush_blocks.rs"]
mod change_filament_flush_blocks;
#[path = "orca_parity/clearance_admission.rs"]
mod clearance_admission;
#[path = "orca_parity/cli_allow_newer_file.rs"]
mod cli_allow_newer_file;
#[path = "orca_parity/cli_custom_gcode_exit.rs"]
mod cli_custom_gcode_exit;
#[path = "orca_parity/cli_exit_channels.rs"]
mod cli_exit_channels;
#[path = "orca_parity/cli_exit_compat17.rs"]
mod cli_exit_compat17;
#[path = "orca_parity/cli_retained_exit.rs"]
mod cli_retained_exit;
#[path = "orca_parity/cli_retained_input.rs"]
mod cli_retained_input;
#[path = "orca_parity/cli_sequential_clearance.rs"]
mod cli_sequential_clearance;
#[path = "orca_parity/cli_upward_machine_result.rs"]
mod cli_upward_machine_result;
#[path = "orca_parity/gated_role_flow.rs"]
mod gated_role_flow;

#[path = "orca_parity/filament_flow_admission.rs"]
mod filament_flow_admission;
#[path = "orca_parity/firmware_zero_layer_lift.rs"]
mod firmware_zero_layer_lift;
#[path = "orca_parity/gcode_flavor_admission.rs"]
mod gcode_flavor_admission;
#[path = "orca_parity/global_combined_brim.rs"]
mod global_combined_brim;
#[path = "orca_parity/load_settings_3mf.rs"]
mod load_settings_3mf;
#[path = "orca_parity/load_settings_3mf_inherits.rs"]
mod load_settings_3mf_inherits;
#[path = "orca_parity/load_settings_3mf_partial.rs"]
mod load_settings_3mf_partial;
#[path = "orca_parity/load_settings_cli.rs"]
mod load_settings_cli;
#[path = "orca_parity/multi_instance_brim.rs"]
mod multi_instance_brim;
#[path = "orca_parity/native_wall_clock.rs"]
mod native_wall_clock;
#[path = "orca_parity/object_archimedean_ironing.rs"]
mod object_archimedean_ironing;
#[path = "orca_parity/object_hilbert_ironing.rs"]
mod object_hilbert_ironing;
#[path = "orca_parity/object_octagram_ironing.rs"]
mod object_octagram_ironing;
#[path = "orca_parity/object_zag_ironing.rs"]
mod object_zag_ironing;
#[path = "orca_parity/perobject_multi_copy_skirt.rs"]
mod perobject_multi_copy_skirt;
#[path = "orca_parity/perobject_outer_inner_brim.rs"]
mod perobject_outer_inner_brim;
#[path = "orca_parity/plate_origin_cli.rs"]
mod plate_origin_cli;
#[path = "orca_parity/option_coverage/post_process.rs"]
mod post_process_cli;
#[path = "orca_parity/residual_stock_cli.rs"]
mod residual_stock_cli;
#[path = "orca_parity/stock_early_cli.rs"]
mod stock_early_cli;
#[path = "orca_parity/wipe_before_override.rs"]
mod wipe_before_override;
#[path = "orca_parity/wipe_retraction_epsilon.rs"]
mod wipe_retraction_epsilon;
#[path = "orca_parity/wipe_zero_retraction_comment.rs"]
mod wipe_zero_retraction_comment;

#[path = "orca_parity/curled_perimeter_slowdown.rs"]
mod curled_perimeter_slowdown;
#[path = "orca_parity/empty_selected_plate.rs"]
mod empty_selected_plate;
#[path = "orca_parity/empty_selected_plate_controls.rs"]
mod empty_selected_plate_controls;
#[path = "orca_parity/filament_usage_stats.rs"]
mod filament_usage_stats;
#[path = "orca_parity/filament_volume_oracle.rs"]
mod filament_volume_oracle;
#[path = "orca_parity/flow_errors.rs"]
mod flow_errors;
#[path = "orca_parity/gcode_printable_area.rs"]
mod gcode_printable_area;
#[path = "orca_parity/honeycomb_multiline.rs"]
mod honeycomb_multiline;
#[path = "orca_parity/infill_anchor_dont_connect.rs"]
mod infill_anchor_dont_connect;
#[path = "orca_parity/infill_anchor_negative.rs"]
mod infill_anchor_negative;
#[path = "orca_parity/infill_rotate_template.rs"]
mod infill_rotate_template;
#[path = "orca_parity/klipper_velocity_limit.rs"]
mod klipper_velocity_limit;
#[path = "orca_parity/m205.rs"]
mod m205;
#[path = "orca_parity/narrowed_bridge_anchor_whole_file_control.rs"]
mod narrowed_bridge_anchor_whole_file_control;
#[path = "orca_parity/option_coverage.rs"]
mod option_coverage;
#[path = "orca_parity/plate_custom_gcodes.rs"]
mod plate_custom_gcodes;
#[path = "orca_parity/presets.rs"]
mod presets;
#[path = "orca_parity/print_estimated_stats.rs"]
mod print_estimated_stats;
#[path = "orca_parity/print_object_centering.rs"]
mod print_object_centering;
#[path = "orca_parity/print_validation.rs"]
mod print_validation;
#[path = "orca_parity/replay.rs"]
mod replay;
#[path = "orca_parity/replay_tests.rs"]
mod replay_tests;
#[path = "orca_parity/runner.rs"]
mod runner;
#[path = "orca_parity/sermoon_v1_whole_file.rs"]
mod sermoon_v1_whole_file;
#[path = "orca_parity/shared_mesh_centering.rs"]
mod shared_mesh_centering;
#[path = "orca_parity/shared_mesh_import_contract.rs"]
mod shared_mesh_import_contract;
#[path = "orca_parity/smoke.rs"]
mod smoke;
#[path = "orca_parity/smooth_type2_tower.rs"]
mod smooth_type2_tower;
#[path = "orca_parity/stock_presets.rs"]
mod stock_presets;
#[path = "orca_parity/sync_time.rs"]
mod sync_time;
#[path = "orca_parity/vg1_flush_state.rs"]
mod vg1_flush_state;

#[path = "stock_project_whole_file/golden.rs"]
mod golden;
// Diagnostic-only semantic behavior tests; never a whole-output acceptance gate.
#[path = "stock_project_whole_file/semantic.rs"]
mod semantic;

#[path = "orca_parity/case_support.rs"]
mod case_support;

use std::collections::BTreeMap;

pub(crate) use case_support::case_helpers::*;
use case_support::smoke_overrides;
use presets::VendorProfiles;
use runner::{CaseInputs, OrcaRunner};
use serde_json::Value;
pub(crate) use smoke_overrides::{
    normalize_filament_defaults, normalize_process_defaults, smoke_case_overrides, smoke_overrides,
};

use crate::runner::{OrcaRunner as Runner, ParityCase};

pub(crate) use case_support::outcomes::{divergence, pass};
pub(crate) use case_support::vendor_inventory::vendors;

#[path = "orca_parity/support_ironing_raft_offset.rs"]
mod support_ironing_raft_offset;
#[path = "orca_parity/support_touching_grid.rs"]
mod support_touching_grid;
