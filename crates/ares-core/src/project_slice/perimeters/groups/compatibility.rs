use crate::{FloatOrPercent, RegionOptions};

// Layer.cpp::is_perimeter_compatible intentionally excludes density and bridge_flow.
pub(super) fn is_compatible(
    a: &RegionOptions,
    b: &RegionOptions,
    filament_map: &[crate::OrcaInt],
) -> bool {
    // Layer.cpp:168 passes the authored ID unchanged to get_extruder_id;
    // Print.cpp:5008-5015 returns physical0 for an out-of-range argument.
    let slot = filament_map
        .get(a.outer_wall_filament_id.0 as usize)
        .map_or(0, |id| (id.0 - 1) as usize);
    a.outer_wall_filament_id == b.outer_wall_filament_id
        && a.inner_wall_filament_id == b.inner_wall_filament_id
        && a.wall_loops == b.wall_loops
        && a.wall_sequence == b.wall_sequence
        && a.is_infill_first == b.is_infill_first
        && a.inner_wall_speed.get_at(slot) == b.inner_wall_speed.get_at(slot)
        && a.outer_wall_speed.get_at(slot) == b.outer_wall_speed.get_at(slot)
        && float_or_percent_eq(a.small_perimeter_speed, b.small_perimeter_speed)
        && a.gap_infill_speed == b.gap_infill_speed
        && a.filter_out_gap_fill == b.filter_out_gap_fill
        && a.detect_overhang_wall == b.detect_overhang_wall
        && a.overhang_reverse == b.overhang_reverse
        && float_or_percent_eq(a.overhang_reverse_threshold, b.overhang_reverse_threshold)
        && a.wall_direction == b.wall_direction
        && serialized_width(a.inner_wall_line_width) == serialized_width(b.inner_wall_line_width)
        && serialized_width(a.outer_wall_line_width) == serialized_width(b.outer_wall_line_width)
        && a.detect_thin_wall == b.detect_thin_wall
        && a.infill_wall_overlap == b.infill_wall_overlap
        && a.top_bottom_infill_wall_overlap == b.top_bottom_infill_wall_overlap
        && a.only_one_wall_first_layer == b.only_one_wall_first_layer
        && a.only_one_wall_top == b.only_one_wall_top
        && float_or_percent_eq(a.min_width_top_surface, b.min_width_top_surface)
        && a.seam_slope_type == b.seam_slope_type
        && a.seam_slope_conditional == b.seam_slope_conditional
        && a.scarf_angle_threshold == b.scarf_angle_threshold
        && a.scarf_overhang_threshold == b.scarf_overhang_threshold
        && float_or_percent_eq(a.scarf_joint_speed, b.scarf_joint_speed)
        && approx(a.scarf_joint_flow_ratio.0, b.scarf_joint_flow_ratio.0)
        && float_or_percent_eq(a.seam_slope_start_height, b.seam_slope_start_height)
        && a.seam_slope_entire_loop == b.seam_slope_entire_loop
        && approx(a.seam_slope_min_length.0, b.seam_slope_min_length.0)
        && a.seam_slope_steps == b.seam_slope_steps
        && a.seam_slope_inner_walls == b.seam_slope_inner_walls
}

// Config.hpp::ConfigOptionFloat equality uses libslic3r.h::EPSILON.
fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-4
}

fn float_or_percent_eq(a: FloatOrPercent, b: FloatOrPercent) -> bool {
    match (a, b) {
        (FloatOrPercent::Float(a), FloatOrPercent::Float(b)) => approx(a, b),
        (FloatOrPercent::Percent(a), FloatOrPercent::Percent(b)) => approx(a.0, b.0),
        _ => false,
    }
}

fn serialized_width(width: FloatOrPercent) -> String {
    match width {
        FloatOrPercent::Float(value) => crate::options::format_number(value),
        FloatOrPercent::Percent(value) => format!("{}%", crate::options::format_number(value.0)),
    }
}
