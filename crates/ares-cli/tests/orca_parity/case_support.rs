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

#[path = "cli_slice_plates.rs"]
mod cli_slice_plates;
#[path = "cli_uptodate_printable_size.rs"]
mod cli_uptodate_printable_size;
#[path = "cli_uptodate_refresh.rs"]
mod cli_uptodate_refresh;
#[path = "export_3mf.rs"]
mod export_3mf;

#[path = "startup_fan_initial_tool.rs"]
mod startup_fan_initial_tool;

#[path = "absolute_e_reset_comment.rs"]
mod absolute_e_reset_comment;

#[path = "mk4_tree_support_base_connect.rs"]
mod mk4_tree_support_base_connect;

#[path = "mk4_tree_support_default_seam_anchor.rs"]
mod mk4_tree_support_default_seam_anchor;

#[path = "spiral_hole_projected_split.rs"]
mod spiral_hole_projected_split;

#[path = "support_base_extension_priority.rs"]
mod support_base_extension_priority;

#[path = "support_contact_overlap.rs"]
mod support_contact_overlap;

#[path = "support_interface_top_layers_base_connect.rs"]
mod support_interface_top_layers_base_connect;

#[path = "organic_vertical_enforcer.rs"]
mod organic_vertical_enforcer;

#[path = "sequential_origin_label_flush.rs"]
mod sequential_origin_label_flush;

#[path = "non_tower_flush_precision.rs"]
mod non_tower_flush_precision;
#[path = "plate_loaded_label_first_entry.rs"]
mod plate_loaded_label_first_entry;

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

#[path = "brim_auto_group_clamp.rs"]
mod brim_auto_group_clamp;

#[path = "outer_brim_width_quantization.rs"]
mod outer_brim_width_quantization;
#[path = "type1_compacted_exit.rs"]
mod type1_compacted_exit;
#[path = "unclear_travel_lift_cancellation.rs"]
mod unclear_travel_lift_cancellation;
