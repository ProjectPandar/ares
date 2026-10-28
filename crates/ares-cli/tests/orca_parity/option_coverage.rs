//! Option-domain parity sweep against the Ender-3 smoke fixture.

#[path = "option_coverage/filament_self_index.rs"]
mod filament_self_index;

#[path = "option_coverage/master_extruder_id.rs"]
mod master_extruder_id;

#[path = "option_coverage/printer_technology.rs"]
mod printer_technology;

#[path = "option_coverage/multiline_union.rs"]
pub(crate) mod multiline_union;

#[path = "option_coverage/interface_shells_alignment.rs"]
mod interface_shells_alignment;

#[path = "option_coverage/filament_override_range.rs"]
mod filament_override_range;
#[path = "option_coverage/hole_to_polyhole.rs"]
mod hole_to_polyhole;
#[path = "option_coverage/ironing_pattern.rs"]
mod ironing_pattern;
#[path = "option_coverage/max_travel_detour_distance.rs"]
mod max_travel_detour_distance;
#[path = "option_coverage/region_speed_emission.rs"]
mod region_speed_emission;

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
#[path = "option_coverage/wrapping_exclude_coverage.rs"]
mod wrapping_exclude_coverage;

#[cfg(test)]
#[path = "option_coverage/filament_flow_coverage.rs"]
mod filament_flow_coverage;

#[path = "option_coverage/adaptive_volumetric_coverage.rs"]
mod adaptive_volumetric_coverage;
#[cfg(test)]
#[path = "option_coverage/bridge_flow_coverage.rs"]
mod bridge_flow_coverage;
#[path = "option_coverage/first_layer_sequence.rs"]
mod first_layer_sequence;
#[cfg(test)]
#[path = "option_coverage/tri_hexagon_multiline.rs"]
mod tri_hexagon_multiline;

#[cfg(test)]
#[path = "option_coverage/safc_coverage.rs"]
mod safc_coverage;
#[cfg(test)]
#[path = "option_coverage/tool_context.rs"]
mod tool_context;
#[cfg(test)]
#[path = "option_coverage/tool_mapping.rs"]
mod tool_mapping;

#[cfg(test)]
#[path = "option_coverage/tool_sequences.rs"]
mod tool_sequences;

#[cfg(test)]
#[path = "option_coverage/layer_heights.rs"]
mod layer_heights;

#[path = "option_coverage/height_context.rs"]
mod height_context;

#[cfg(test)]
#[path = "option_coverage/print_compatible_printers.rs"]
mod print_compatible_printers;

#[path = "option_coverage/adaptive_pa_bridges.rs"]
mod adaptive_pa_bridges;

#[cfg(test)]
#[path = "option_coverage/locked_width_context.rs"]
mod locked_width_context;

#[cfg(test)]
#[path = "option_coverage/extruder_printable_area.rs"]
mod extruder_printable_area;

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
#[path = "option_coverage/filament_ironing_speed.rs"]
mod filament_ironing_speed;

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
#[path = "option_coverage/elephant_foot_count.rs"]
mod elephant_foot_count;

#[cfg(test)]
#[path = "option_coverage/top_shell_count.rs"]
mod top_shell_count;

#[cfg(test)]
#[path = "option_coverage/bottom_shell_count.rs"]
mod bottom_shell_count;

#[cfg(test)]
#[path = "option_coverage/slow_layer_count.rs"]
mod slow_layer_count;

#[cfg(test)]
#[path = "option_coverage/filename_format.rs"]
mod filename_format;

#[cfg(test)]
#[path = "option_coverage/range_completion.rs"]
mod range_completion;

#[cfg(test)]
#[path = "option_coverage/filament_printable.rs"]
mod filament_printable;
#[cfg(test)]
#[path = "option_coverage/vitrification.rs"]
mod vitrification;

#[path = "option_coverage/activation.rs"]
mod activation;
use activation::{inject_case, support_geometry_option};

#[path = "option_coverage/domains.rs"]
mod domains;
#[path = "option_coverage/summary.rs"]
mod summary;
use summary::write_summary;
#[cfg(test)]
#[path = "option_coverage/float_percent_tests.rs"]
mod float_percent_tests;
#[cfg(test)]
#[path = "option_coverage/owner_pair_tests.rs"]
mod owner_pair_tests;
#[cfg(test)]
#[path = "option_coverage/sweep.rs"]
mod sweep;
#[cfg(test)]
#[path = "option_coverage/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "option_coverage/completion_tests.rs"]
mod completion_tests;

#[cfg(test)]
#[path = "option_coverage/support_report_tests.rs"]
mod support_report_tests;

#[cfg(test)]
#[path = "option_coverage/tree_activation.rs"]
mod tree_activation;

#[cfg(test)]
#[path = "option_coverage/pressure_advance.rs"]
mod pressure_advance;

#[cfg(test)]
#[path = "option_coverage/skirt_height.rs"]
mod skirt_height;

#[cfg(test)]
#[path = "option_coverage/hot_plate.rs"]
mod hot_plate;

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
#[path = "option_coverage/locked_widths.rs"]
mod locked_widths;

#[cfg(test)]
#[path = "option_coverage/cut_retraction.rs"]
mod cut_retraction;

#[cfg(test)]
#[path = "option_coverage/cut_retraction_values.rs"]
mod cut_retraction_values;

#[cfg(test)]
#[path = "option_coverage/air_filtration.rs"]
mod air_filtration;

#[cfg(test)]
#[path = "option_coverage/chamber_control.rs"]
mod chamber_control;

#[cfg(test)]
#[path = "option_coverage/scalar_points.rs"]
mod scalar_points;

#[cfg(test)]
#[path = "option_coverage/obsolete_silent_mode.rs"]
mod obsolete_silent_mode;

#[cfg(test)]
#[path = "option_coverage/resolution_floor.rs"]
mod resolution_floor;

#[cfg(test)]
#[path = "option_coverage/filament_map_mode.rs"]
mod filament_map_mode;

#[path = "option_coverage/filament_map_length.rs"]
mod filament_map_length;

#[cfg(test)]
#[path = "option_coverage/thumbnails.rs"]
mod thumbnails;

#[cfg(test)]
#[path = "option_coverage/preset_identity_tests.rs"]
mod preset_identity_tests;

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

use outcome::{OptionOutcome, profiles_root, strict_comparator_ran};
