//! Region-resolved speeds and accelerations (`MotionOptions::
//! from_traversal` mid-section).

pub(in crate::project_slice::gcode_emit) struct RegionSpeeds {
    pub(in crate::project_slice::gcode_emit) bridge_speed: f64,
    pub(in crate::project_slice::gcode_emit) outer_wall_speed: f64,
    pub(in crate::project_slice::gcode_emit) small_perimeter_speed: f64,
    pub(in crate::project_slice::gcode_emit) outer_wall_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) default_acceleration: u32,
}

use super::super::{
    MotionOptions,
    helpers::{absolute, acceleration},
};

pub(in crate::project_slice::gcode_emit) fn region_speeds(
    full: &crate::options::ProjectSettings,
    object: Option<&crate::project::effective_config::types::ResolvedProjectObject>,
    region: Option<&crate::RegionOptions>,
) -> RegionSpeeds {
    let bridge_speed = region.map_or(full.process.region.bridge_speed.0, |value| {
        value.bridge_speed.0
    });
    let outer_wall_speed = region.map_or(full.process.region.outer_wall_speed.0, |value| {
        value.outer_wall_speed.0
    });
    let small_perimeter = region.map_or(full.process.region.small_perimeter_speed, |region| {
        region.small_perimeter_speed
    });
    let small_perimeter_speed = small_perimeter_speed(small_perimeter, outer_wall_speed);
    let outer_wall_acceleration = acceleration(
        object.map(|value| &value.object),
        full.process.object.outer_wall_acceleration.0,
        |value| value.outer_wall_acceleration.0,
    );
    let default_acceleration = acceleration(
        object.map(|value| &value.object),
        full.process.object.default_acceleration.0,
        |value| value.default_acceleration.0,
    );
    RegionSpeeds {
        bridge_speed,
        outer_wall_speed,
        small_perimeter_speed,
        outer_wall_acceleration,
        default_acceleration,
    }
}

// GCode.cpp:6136/6163 applies the current region before entity emission;
// :6523 and :5811-5813 consume its outer speed and relative small-loop speed.
pub(super) fn apply_outer_wall_speed(options: &mut MotionOptions, region: &crate::RegionOptions) {
    options.outer_wall_speed = region.outer_wall_speed.0;
    options.small_perimeter_speed =
        small_perimeter_speed(region.small_perimeter_speed, options.outer_wall_speed);
}

fn small_perimeter_speed(configured: crate::FloatOrPercent, outer_wall_speed: f64) -> f64 {
    let speed = absolute(configured, outer_wall_speed);
    if speed > 0.0 {
        speed
    } else {
        0.5 * outer_wall_speed
    }
}
