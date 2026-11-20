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

#[path = "seam_paint.rs"]
mod seam_paint;

#[path = "seam_paint_aligned_back.rs"]
mod seam_paint_aligned_back;

#[path = "seam_paint_extended_palette.rs"]
mod seam_paint_extended_palette;

#[path = "seam_paint_nearest_rear.rs"]
mod seam_paint_nearest_rear;
#[path = "seam_paint_random.rs"]
mod seam_paint_random;

#[path = "seam_paint_split.rs"]
mod seam_paint_split;

#[path = "seam_paint_volumes.rs"]
mod seam_paint_volumes;

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
