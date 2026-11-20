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

#[path = "adaptive_cubic_bodies.rs"]
mod adaptive_cubic_bodies;
#[path = "arachne_top_wall_split.rs"]
mod arachne_top_wall_split;

#[path = "cli_slice_plates.rs"]
mod cli_slice_plates;
#[path = "cli_uptodate_printable_size.rs"]
mod cli_uptodate_printable_size;
#[path = "cli_uptodate_refresh.rs"]
mod cli_uptodate_refresh;
#[path = "export_3mf.rs"]
mod export_3mf;

#[path = "export_stl.rs"]
mod export_stl;
#[path = "export_stls_multi_input.rs"]
mod export_stls_multi_input;
#[path = "grid_bridge_anchor_direction.rs"]
mod grid_bridge_anchor_direction;
#[path = "sparse_anchor_object_layer.rs"]
mod sparse_anchor_object_layer;
#[path = "trapezoidal_grid_raft.rs"]
mod trapezoidal_grid_raft;

#[path = "support_cubic_bodies.rs"]
mod support_cubic_bodies;

#[path = "startup_fan_initial_tool.rs"]
mod startup_fan_initial_tool;

#[path = "cli_assemble.rs"]
mod cli_assemble;

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
#[path = "concentric_thick_polyline_rotation.rs"]
mod concentric_thick_polyline_rotation;
#[path = "direct_stl_auto_arrange.rs"]
mod direct_stl_auto_arrange;
#[path = "direct_stl_slice.rs"]
mod direct_stl_slice;
#[path = "export_statistics.rs"]
mod export_statistics;
#[path = "option_only_stl_exit_codes.rs"]
mod option_only_stl_exit_codes;

#[path = "mk4_tree_support_base_connect.rs"]
mod mk4_tree_support_base_connect;

#[path = "mk4_tree_support_default_seam_anchor.rs"]
mod mk4_tree_support_default_seam_anchor;

#[path = "tree_raft_without_support.rs"]
mod tree_raft_without_support;

#[path = "organic_support_first_layer.rs"]
mod organic_support_first_layer;

#[path = "spiral_hole_projected_split.rs"]
mod spiral_hole_projected_split;

#[path = "support_base_extension_priority.rs"]
mod support_base_extension_priority;

#[path = "support_contact_overlap.rs"]
mod support_contact_overlap;

#[path = "support_interface_top_layers_base_connect.rs"]
mod support_interface_top_layers_base_connect;

#[path = "normal_raft_interface_arcs.rs"]
mod normal_raft_interface_arcs;
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
#[path = "type2_tower_entry_z.rs"]
mod type2_tower_entry_z;

#[path = "unclear_travel_lift_cancellation.rs"]
mod unclear_travel_lift_cancellation;

#[path = "arachne_disconnected_loop.rs"]
mod arachne_disconnected_loop;

#[path = "arachne_ioi_proximity.rs"]
mod arachne_ioi_proximity;
