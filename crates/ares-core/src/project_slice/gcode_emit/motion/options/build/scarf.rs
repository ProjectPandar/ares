use super::super::{ScarfOptions, helpers::absolute};
use crate::{ProcessRegionSourceOptions, RegionOptions};

/// `GCode.cpp:6136/6163` `m_config.apply(region.config())` for the
/// scarf block — the single resolution vocabulary shared by the
/// initial `assemble` (`region = first resolved region`) and the
/// per-phase `region::apply`. `default` is `full.process.region`;
/// `nozzle` is the physical-extruder diameter resolving the
/// `FloatOrPercent` line-width fields (the same diameter
/// `MotionOptions.nozzle_diameter` carries).
pub(super) fn apply(
    options: &mut ScarfOptions,
    default: &ProcessRegionSourceOptions,
    region: Option<&RegionOptions>,
    nozzle: f64,
) {
    options.angle_threshold = f64::from(region.map_or(default.scarf_angle_threshold.0, |value| {
        value.scarf_angle_threshold.0
    }));
    options.overhang_threshold = region.map_or(default.scarf_overhang_threshold.0, |value| {
        value.scarf_overhang_threshold.0
    });
    options.outer_line_width = absolute(
        region.map_or(default.outer_wall_line_width, |value| {
            value.outer_wall_line_width
        }),
        nozzle,
    );
    options.inner_line_width = absolute(
        region.map_or(default.inner_wall_line_width, |value| {
            value.inner_wall_line_width
        }),
        nozzle,
    );
    options.seam_slope_type = region.map_or(default.seam_slope_type, |value| value.seam_slope_type);
    options.conditional = region.map_or(default.seam_slope_conditional.0, |value| {
        value.seam_slope_conditional.0
    });
    options.start_height = Some(region.map_or(default.seam_slope_start_height, |value| {
        value.seam_slope_start_height
    }));
    options.entire_loop = region.map_or(default.seam_slope_entire_loop.0, |value| {
        value.seam_slope_entire_loop.0
    });
    options.min_length = region.map_or(default.seam_slope_min_length.0, |value| {
        value.seam_slope_min_length.0
    });
    options.steps = region
        .map_or(default.seam_slope_steps.0, |value| value.seam_slope_steps.0)
        .max(0) as usize;
    options.inner_walls = region.map_or(default.seam_slope_inner_walls.0, |value| {
        value.seam_slope_inner_walls.0
    });
    options.speed = Some(region.map_or(default.scarf_joint_speed, |value| value.scarf_joint_speed));
    options.flow_ratio = region.map_or(default.scarf_joint_flow_ratio.0, |value| {
        value.scarf_joint_flow_ratio.0
    });
}
