//! Option-domain parity sweep against the Ender-3 smoke fixture.

#[path = "arachne_unsupported_wall_last.rs"]
mod arachne_unsupported_wall_last;

#[path = "option_coverage/acceleration_jerk_variants.rs"]
mod acceleration_jerk_variants;

#[path = "option_coverage/bed_temperature_footer.rs"]
mod bed_temperature_footer;
#[path = "option_coverage/filament_is_mixed_echo.rs"]
mod filament_is_mixed_echo;
#[path = "option_coverage/filament_volume_map.rs"]
mod filament_volume_map;
#[path = "option_coverage/filament_volume_map_contrasts.rs"]
mod filament_volume_map_contrasts;
#[path = "option_coverage/hotend_rate_variants.rs"]
mod hotend_rate_variants;
#[path = "option_coverage/initial_bed_temperature.rs"]
mod initial_bed_temperature;
#[path = "option_coverage/multitool_ramming_footer.rs"]
pub(crate) mod multitool_ramming_footer;
#[path = "option_coverage/nozzle_temperature_footer.rs"]
mod nozzle_temperature_footer;
#[path = "option_coverage/nullable_metadata.rs"]
mod nullable_metadata;
#[path = "option_coverage/nullable_vector_placeholder.rs"]
mod nullable_vector_placeholder;
#[path = "option_coverage/pa_variant.rs"]
mod pa_variant;
#[path = "option_coverage/pre_cooling_variant.rs"]
mod pre_cooling_variant;
#[path = "option_coverage/preset_extruder_normalization.rs"]
mod preset_extruder_normalization;
#[path = "option_coverage/project_key_presence.rs"]
mod project_key_presence;
#[path = "option_coverage/remaining_filament_variants.rs"]
mod remaining_filament_variants;
#[path = "option_coverage/variant_retraction.rs"]
mod variant_retraction;

#[path = "classic_narrow_island_guard.rs"]
mod classic_narrow_island_guard;

#[path = "classic_thin_hole_reach.rs"]
mod classic_thin_hole_reach;

#[path = "wipe_inward.rs"]
mod wipe_inward;

#[path = "classic_unsupported_wall_last.rs"]
mod classic_unsupported_wall_last;

#[path = "bambu_gap_fill_interleaving.rs"]
mod bambu_gap_fill_interleaving;

#[path = "spiral_lift_arc_enabled.rs"]
mod spiral_lift_arc_enabled;

#[path = "spiral_lift_resolution.rs"]
mod spiral_lift_resolution;

#[path = "aligned_seam_rotated_object.rs"]
mod aligned_seam_rotated_object;

#[path = "two_wall_inward_move.rs"]
mod two_wall_inward_move;

#[path = "fuzzy_skin_supported_area.rs"]
mod fuzzy_skin_supported_area;

#[path = "arachne_collinear_vertex_tolerance.rs"]
mod arachne_collinear_vertex_tolerance;

#[path = "scarf_seam_neighbor_trim.rs"]
mod scarf_seam_neighbor_trim;

#[path = "scarf_ramp_endpoint_vertex_snap.rs"]
mod scarf_ramp_endpoint_vertex_snap;

#[path = "seam_string_immediate_lower_layer_restart.rs"]
mod seam_string_immediate_lower_layer_restart;

#[path = "arachne_fully_overhanging_exterior_nearest.rs"]
mod arachne_fully_overhanging_exterior_nearest;

#[path = "classic_three_hole_wall_order.rs"]
mod classic_three_hole_wall_order;

#[path = "arachne_first_layer_outer_brim.rs"]
mod arachne_first_layer_outer_brim;

#[path = "arachne_top_layer_requires_top_shell.rs"]
mod arachne_top_layer_requires_top_shell;

#[path = "arachne_first_layer_requires_bottom_shell.rs"]
mod arachne_first_layer_requires_bottom_shell;

#[path = "arachne_center_wall_follows_inner_outer.rs"]
mod arachne_center_wall_follows_inner_outer;

#[path = "top_layer_one_wall_requires_top_shell.rs"]
mod top_layer_one_wall_requires_top_shell;

#[path = "first_layer_one_wall_requires_bottom_shell.rs"]
mod first_layer_one_wall_requires_bottom_shell;

#[path = "classic_fully_overhanging_exterior_nearest.rs"]
mod classic_fully_overhanging_exterior_nearest;

#[path = "option_coverage/printer_technology.rs"]
mod printer_technology;

#[path = "option_coverage/multiline_union.rs"]
pub(crate) mod multiline_union;

#[path = "option_coverage/interface_shells_alignment.rs"]
mod interface_shells_alignment;

#[path = "option_coverage/bbl_toolchange_command.rs"]
mod bbl_toolchange_command;
#[path = "option_coverage/cooling_fan_nozzle_range_variants.rs"]
mod cooling_fan_nozzle_range_variants;
#[path = "option_coverage/filament_override_range.rs"]
mod filament_override_range;
#[path = "option_coverage/first_point_travel.rs"]
mod first_point_travel;
#[path = "option_coverage/hole_to_polyhole.rs"]
mod hole_to_polyhole;
#[path = "option_coverage/ironing_pattern.rs"]
mod ironing_pattern;
#[path = "option_coverage/load_filament_variant_normalize.rs"]
mod load_filament_variant_normalize;
#[path = "option_coverage/max_travel_detour_distance.rs"]
mod max_travel_detour_distance;
#[path = "option_coverage/pressure_advance_variant_layout.rs"]
mod pressure_advance_variant_layout;
#[path = "option_coverage/process_variant_columns.rs"]
mod process_variant_columns;
#[path = "option_coverage/refresh_preset_includes.rs"]
mod refresh_preset_includes;
#[path = "option_coverage/region_speed_emission.rs"]
mod region_speed_emission;
#[path = "option_coverage/single_variant_printer_filament_selection.rs"]
mod single_variant_printer_filament_selection;
#[path = "option_coverage/static_volume_map_variants.rs"]
mod static_volume_map_variants;
#[path = "option_coverage/top_surface_expansion.rs"]
mod top_surface_expansion;
#[path = "option_coverage/variant_footer.rs"]
mod variant_footer;

#[path = "option_coverage/support_ironing_pattern_domain.rs"]
mod support_ironing_pattern_domain;

#[path = "option_coverage/support_ironing_unknown_wire.rs"]
mod support_ironing_unknown_wire;

#[path = "option_coverage/print_extruder_identity.rs"]
mod print_extruder_identity;

#[path = "option_coverage/printer_physical_identity.rs"]
mod printer_physical_identity;

#[path = "option_coverage/semm_ramming.rs"]
mod semm_ramming;

#[path = "option_coverage/seam_position_random.rs"]
mod seam_position_random;

#[path = "option_coverage/staggered_inner_seams.rs"]
mod staggered_inner_seams;

#[path = "option_coverage/wall_generator_arachne.rs"]
mod wall_generator_arachne;

#[path = "option_coverage/nozzle_dataset.rs"]
mod nozzle_dataset;

#[cfg(target_os = "linux")]
#[path = "option_coverage/post_process_matrix.rs"]
mod post_process_matrix;

#[path = "option_coverage/legacy_brim_ears.rs"]
mod legacy_brim_ears;

#[path = "option_coverage/timelapse_coverage.rs"]
mod timelapse_coverage;

#[cfg(test)]
#[cfg(test)]
#[path = "option_coverage/adaptive_volumetric_coverage.rs"]
mod adaptive_volumetric_coverage;
#[cfg(test)]
#[path = "option_coverage/first_layer_sequence.rs"]
mod first_layer_sequence;
#[cfg(test)]
#[path = "option_coverage/tri_hexagon_multiline.rs"]
mod tri_hexagon_multiline;

#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[path = "option_coverage/height_context.rs"]
mod height_context;
#[cfg(test)]
#[path = "option_coverage/safc_coverage.rs"]
mod safc_coverage;
#[cfg(test)]
#[path = "option_coverage/tool_context.rs"]
mod tool_context;

#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[path = "option_coverage/ironing_fan.rs"]
mod ironing_fan;

#[cfg(test)]
#[path = "option_coverage/bookkeeping_export.rs"]
mod bookkeeping_export;

#[cfg(test)]
#[path = "option_coverage/initialized_short_line.rs"]
mod initialized_short_line;

#[cfg(test)]
#[path = "option_coverage/initialized_transition_snap.rs"]
mod initialized_transition_snap;

#[cfg(test)]
#[path = "option_coverage/initialized_stitch_junction.rs"]
mod initialized_stitch_junction;

#[cfg(test)]
#[path = "option_coverage/initialized_bridge_direction.rs"]
mod initialized_bridge_direction;

#[cfg(test)]
#[cfg(test)]
#[path = "option_coverage/filament_stats_vectors.rs"]
mod filament_stats_vectors;

#[cfg(test)]
#[path = "option_coverage/filament_variant_zero_fill.rs"]
mod filament_variant_zero_fill;

#[cfg(test)]
#[path = "option_coverage/single_volume_offset.rs"]
mod single_volume_offset;

#[cfg(test)]
#[path = "option_coverage/initialized_object_id.rs"]
mod initialized_object_id;

#[cfg(test)]
#[path = "option_coverage/seeded_context.rs"]
mod seeded_context;

#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[path = "option_coverage/activation.rs"]
mod activation;

#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[path = "option_coverage/completion_tests.rs"]
mod completion_tests;
#[cfg(test)]
#[cfg(test)]
#[path = "option_coverage/owner_pair_tests.rs"]
mod owner_pair_tests;

#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[path = "option_coverage/fill_grid.rs"]
mod fill_grid;

#[cfg(test)]
#[path = "option_coverage/monotonic_chain.rs"]
mod monotonic_chain;

#[path = "option_coverage/lightning_output.rs"]
mod lightning_output;

#[path = "option_coverage/normal_support_output.rs"]
pub(crate) mod normal_support_output;

#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[path = "option_coverage/filament_map_length.rs"]
mod filament_map_length;

#[path = "option_coverage/filament_map_refresh.rs"]
mod filament_map_refresh;

#[cfg(test)]
#[cfg(test)]
#[path = "option_coverage/preset_identity.rs"]
mod preset_identity;

use crate::{
    self as parity,
    presets::VendorProfiles,
    runner::{self, OrcaRunner},
};
use serde_json::{Map, Value};
use std::path::PathBuf;

#[path = "option_coverage/outcome.rs"]
mod outcome;

use outcome::{profiles_root, strict_comparator_ran};

fn first_cap_layer_object(gcode: &str) -> &str {
    gcode
        .split_once(";LAYER_CHANGE\n;Z:1\n")
        .unwrap()
        .1
        .split_once("; printing object ")
        .unwrap()
        .1
        .split_once('\n')
        .unwrap()
        .1
        .split_once("; stop printing object ")
        .unwrap()
        .0
}

fn first_layer_object(gcode: &str) -> &str {
    gcode
        .split(";LAYER_CHANGE\n")
        .nth(1)
        .unwrap()
        .split_once("; printing object ")
        .unwrap()
        .1
        .split_once('\n')
        .unwrap()
        .1
        .split_once("; stop printing object ")
        .unwrap()
        .0
}
