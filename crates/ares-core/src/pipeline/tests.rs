use super::*;
use crate::{
    InputFormat, Point2, PrintPathRole, Segment2, SliceError, SliceOptions,
    pipeline::test_support::rectangular_pipeline,
};
use serde_json::json;

mod alternate_extra_wall;
mod auxiliary_fan_first_x_gcode;
mod basic;
mod bridge_angle;
mod bridge_density;
mod bridge_no_support;
mod brim_ears;
mod brim_efc_outline;
mod calib_flowrate_topinfill_special_order;
mod combine_brims;
mod detect_thin_wall;
mod different_extruders_guard;
mod enable_extra_bridge_layer;
mod extra_perimeters_on_overhangs;
mod extra_solid_infills;
mod fan_kickstart;
mod filament_cost_gcode;
mod filament_density_header;
mod filament_flow_ratio;
mod filament_ironing_inset;
mod filament_max_volumetric_speed;
mod filament_shrink_xy;
mod fill_multiline;
mod first_layer_flow_ratio;
mod gap_fill_role_gcode;
mod infill_anchor;
mod infill_combination;
mod infill_wall_overlap;
mod initial_layer_line_width;
mod initial_layer_speeds;
mod internal_bridge;
mod internal_bridge_angle;
mod internal_bridge_density;
mod internal_bridge_fan_gcode;
mod internal_solid_infill;
mod internal_solid_numeric;
mod ironing_angle;
mod ironing_flow;
mod ironing_inset;
mod ironing_pattern;
mod ironing_spacing;
mod ironing_speed;
mod ironing_type_paths;
mod machine_min_rate_time_gcode;
mod make_overhang_printable;
mod min_feature_bead_width;
mod min_length_factor;
mod notes_header;
mod only_one_wall_first_layer;
mod only_one_wall_top;
mod overhang_bridge_fan_gcode;
mod overhang_reverse;
mod overhang_speed;
mod per_object_skirt;
mod precise_outer_wall;
mod print_flow_ratio;
mod printable_height;
mod resonance_avoidance;
mod role_fan_gcode_support;
mod role_filament_extrusion;
mod set_other_flow_ratios;
mod shell_thickness;
mod skirt_height;
mod slow_down_layers;
mod small_area_infill_flow;
mod small_perimeter_speed;
mod solid_surface_patterns;
mod sparse_density_shell_surfaces;
mod sparse_infill_flow_ratio;
mod sparse_infill_pattern;
mod spiral_finishing_flow_ratio_gcode;
mod spiral_mode_normalization;
mod spiral_mode_xy_smoothing_gcode;
mod spiral_starting_flow_ratio_gcode;
mod staggered_inner_seams;
mod support_angle;
mod support_base_pattern;
mod support_base_pattern_spacing;
mod support_bottom_interface_spacing;
mod support_critical_regions_only_proxy;
mod support_critical_regions_only_proxy_gcode;
mod support_enable;
mod support_expansion;
mod support_interface_loop_pattern;
mod support_interface_not_for_body;
mod support_interface_pattern;
mod support_interface_pattern_concentric;
mod support_interface_pattern_gcode;
mod support_interface_pattern_interlaced;
mod support_interface_spacing;
mod support_interface_speed_flow;
mod support_interface_top_layers_runtime;
mod support_ironing_paths;
mod support_ironing_pattern;
mod support_ironing_role_fan_gcode;
mod support_ironing_spacing;
mod support_object_first_layer_gap_proxy;
mod support_object_first_layer_gap_proxy_gcode;
mod support_object_xy_distance_proxy;
mod support_object_xy_distance_proxy_gcode;
mod support_on_build_plate_only_proxy;
mod support_on_build_plate_only_proxy_gcode;
mod support_placement;
mod support_raft_expansion;
mod support_raft_expansion_gcode;
mod support_raft_first_layer_density;
mod support_raft_first_layer_density_gcode;
mod support_raft_first_layer_expansion;
mod support_raft_first_layer_expansion_gcode;
mod support_remove_small_overhang_proxy;
mod support_remove_small_overhang_proxy_gcode;
mod support_speed_flow;
mod support_style;
mod support_style_snug_proxy;
mod support_style_snug_proxy_gcode;
mod support_threshold;
mod support_threshold_contact_proxy;
mod support_threshold_contact_proxy_gcode;
mod support_type;
mod support_z_distance;
mod surface_density;
mod symmetric_infill_y_axis;
mod thick_bridge_extrusion_gcode;
mod top_bottom_solid_surface;
mod tree_support_brim;
mod tree_support_options;
mod tree_support_wall_sheath;
mod tree_support_wall_sheath_gcode;
mod wall_direction;
mod wall_flow_ratios;
mod wall_infill_order;
mod wall_maximum_resolution_deviation;
mod wall_sequence;
mod wipe_before_external_loop;
mod wipe_on_loops;

fn square_pyramid_ascii_stl() -> Vec<u8> {
    [
        "solid pyramid",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 1 0 0.4",
        "vertex 0 1 0.4",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 0 -1 0.4",
        "vertex 1 0 0.4",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex -1 0 0.4",
        "vertex 0 -1 0.4",
        "endloop",
        "endfacet",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 0 1 0.4",
        "vertex -1 0 0.4",
        "endloop",
        "endfacet",
        "endsolid pyramid",
    ]
    .join("\n")
    .into_bytes()
}

fn single_sloped_triangle_ascii_stl() -> Vec<u8> {
    [
        "solid open",
        "facet normal 0 0 1",
        "outer loop",
        "vertex 0 0 0",
        "vertex 1 0 0.4",
        "vertex 0 1 0.4",
        "endloop",
        "endfacet",
        "endsolid open",
    ]
    .join("\n")
    .into_bytes()
}
