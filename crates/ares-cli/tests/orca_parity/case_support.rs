//! Shared construction, outcome, inventory and smoke-option helpers for the parity suite.

use crate::{BTreeMap, Runner, Value};
use case_helpers::ParityOutcome;

#[path = "case_helpers.rs"]
pub(crate) mod case_helpers;
#[path = "outcomes.rs"]
pub(crate) mod outcomes;
#[path = "smoke_overrides.rs"]
pub(crate) mod smoke_overrides;
#[path = "vendor_inventory.rs"]
pub(crate) mod vendor_inventory;

#[path = "startup_fan_initial_tool.rs"]
mod startup_fan_initial_tool;

#[path = "absolute_e_reset_comment.rs"]
mod absolute_e_reset_comment;

#[path = "direct_stl_slice.rs"]
mod direct_stl_slice;

#[path = "export_statistics.rs"]
mod export_statistics;

#[path = "cli_convert_unit.rs"]
mod cli_convert_unit;
#[path = "cli_load_defaultfila.rs"]
mod cli_load_defaultfila;
#[path = "cli_mtcpp.rs"]
mod cli_mtcpp;
#[path = "cli_sparse_filament_slots.rs"]
mod cli_sparse_filament_slots;
#[path = "cli_strict.rs"]
mod cli_strict;

#[path = "option_only_stl_exit_codes.rs"]
mod option_only_stl_exit_codes;

#[path = "mk4_tree_support_base_connect.rs"]
mod mk4_tree_support_base_connect;

#[path = "mk4_tree_support_default_seam_anchor.rs"]
mod mk4_tree_support_default_seam_anchor;

#[path = "spiral_hole_projected_split.rs"]
mod spiral_hole_projected_split;

#[path = "support_base_extension_priority.rs"]
mod support_base_extension_priority;

#[path = "support_interface_top_layers_base_connect.rs"]
mod support_interface_top_layers_base_connect;

#[path = "sequential_origin_label_flush.rs"]
mod sequential_origin_label_flush;

#[path = "auto_lift_query_frame.rs"]
mod auto_lift_query_frame;

#[path = "sequential_template_layer_count.rs"]
mod sequential_template_layer_count;

#[path = "unknown_key_config_block.rs"]
mod unknown_key_config_block;

#[path = "extruded_weight_totals.rs"]
mod extruded_weight_totals;

#[path = "spiral_vase_admission.rs"]
mod spiral_vase_admission;

#[path = "spiral_vase_object_start_z.rs"]
mod spiral_vase_object_start_z;

#[path = "spiral_byobject_transition.rs"]
mod spiral_byobject_transition;
