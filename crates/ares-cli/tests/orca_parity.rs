//! OrcaSlicer parity suite: slices vendor printer profiles with both the
//! OrcaSlicer 2.4.2 CLI and Ares using the shared generator-only strict byte
//! comparator. Equality of supplied streams does not establish inventory coverage.
//!
//! Environment-gated: runs only when `ARES_ORCA_BIN` (or the repository
//! wrapper `scripts/orca-parity.sh`) names a working OrcaSlicer CLI.

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

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

#[path = "orca_parity/per_object_skirt_brim.rs"]
mod per_object_skirt_brim;

#[path = "orca_parity/per_object_skirt_groups.rs"]
mod per_object_skirt_groups;
#[path = "orca_parity/per_object_skirt_support_raft.rs"]
mod per_object_skirt_support_raft;
#[path = "orca_parity/skirt_flush_infill_w5.rs"]
mod skirt_flush_infill_w5;
#[path = "orca_parity/skirt_flush_objects_w5.rs"]
mod skirt_flush_objects_w5;
#[path = "orca_parity/skirt_flush_support_w5.rs"]
mod skirt_flush_support_w5;
#[path = "orca_parity/skirt_per_object_tower_ctl.rs"]
mod skirt_per_object_tower_ctl;
#[path = "orca_parity/skirt_per_object_tower_w5.rs"]
mod skirt_per_object_tower_w5;
#[path = "orca_parity/toolchange_wipe_ctl.rs"]
mod toolchange_wipe_ctl;
#[path = "orca_parity/toolchange_wipe_flush_w5.rs"]
mod toolchange_wipe_flush_w5;

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
#[path = "orca_parity/brim_ears_w5.rs"]
mod brim_ears_w5;
#[path = "orca_parity/brim_efc_outline.rs"]
mod brim_efc_outline;
#[path = "orca_parity/brim_inner_only_w5.rs"]
mod brim_inner_only_w5;
#[path = "orca_parity/brim_outer_inner_w8.rs"]
mod brim_outer_inner_w8;
#[path = "orca_parity/brim_outeronly_w5.rs"]
mod brim_outeronly_w5;
#[path = "orca_parity/brim_painted_ctl.rs"]
mod brim_painted_ctl;
#[path = "orca_parity/brim_painted_efc_ctl.rs"]
mod brim_painted_efc_ctl;
#[path = "orca_parity/brim_painted_efc_w5.rs"]
mod brim_painted_efc_w5;
#[path = "orca_parity/brim_painted_inner_ctl.rs"]
mod brim_painted_inner_ctl;
#[path = "orca_parity/brim_painted_inner_w5.rs"]
mod brim_painted_inner_w5;
#[path = "orca_parity/brim_painted_w5.rs"]
mod brim_painted_w5;
#[path = "orca_parity/brim_physical_layer.rs"]
mod brim_physical_layer;
#[path = "orca_parity/byobject_perobject_skirt.rs"]
mod byobject_perobject_skirt;
#[path = "orca_parity/byobject_transition_comments.rs"]
mod byobject_transition_comments;
#[path = "orca_parity/change_filament_flush_blocks.rs"]
mod change_filament_flush_blocks;
#[path = "orca_parity/clearance_admission.rs"]
mod clearance_admission;
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
#[path = "orca_parity/filament_flow_admission.rs"]
mod filament_flow_admission;
#[path = "orca_parity/gcode_flavor_admission.rs"]
mod gcode_flavor_admission;
#[path = "orca_parity/load_settings_3mf.rs"]
mod load_settings_3mf;
#[path = "orca_parity/load_settings_3mf_inherits.rs"]
mod load_settings_3mf_inherits;
#[path = "orca_parity/load_settings_3mf_partial.rs"]
mod load_settings_3mf_partial;
#[path = "orca_parity/load_settings_cli.rs"]
mod load_settings_cli;
#[path = "orca_parity/native_wall_clock.rs"]
mod native_wall_clock;
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
#[path = "orca_parity/narrowed_x2d_whole_file.rs"]
mod narrowed_x2d_whole_file;
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
#[path = "orca_parity/smoke_overrides.rs"]
mod smoke_overrides;
#[path = "orca_parity/smooth_type2_tower.rs"]
mod smooth_type2_tower;
#[path = "orca_parity/stock_presets.rs"]
mod stock_presets;
#[path = "orca_parity/sync_time.rs"]
mod sync_time;
#[path = "orca_parity/vg1.rs"]
mod vg1;

#[path = "ksr_fdmtest_v4/golden.rs"]
mod golden;
// Diagnostic-only semantic behavior tests; never a whole-output acceptance gate.
#[path = "ksr_fdmtest_v4/semantic.rs"]
mod semantic;

#[path = "orca_parity/case_helpers.rs"]
mod case_helpers;

use std::collections::BTreeMap;

pub(crate) use case_helpers::*;
use presets::VendorProfiles;
use runner::{CaseInputs, OrcaRunner};
use serde_json::Value;
pub(crate) use smoke_overrides::{
    normalize_filament_defaults, normalize_process_defaults, smoke_case_overrides, smoke_overrides,
};

use crate::runner::{OrcaRunner as Runner, ParityCase};

#[path = "orca_parity/outcomes.rs"]
mod outcomes;
pub(crate) use outcomes::{divergence, pass};

#[path = "orca_parity/vendor_inventory.rs"]
mod vendor_inventory;
pub(crate) use vendor_inventory::vendors;

#[path = "orca_parity/support_touching_grid.rs"]
mod support_touching_grid;
