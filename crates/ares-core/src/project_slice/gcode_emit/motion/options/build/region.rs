//! Region-resolved speeds and accelerations (`MotionOptions::
//! from_traversal` mid-section).

pub(in crate::project_slice::gcode_emit) struct RegionSpeeds {
    pub(in crate::project_slice::gcode_emit) bridge_speed: f64,
    pub(in crate::project_slice::gcode_emit) outer_wall_speed: f64,
    pub(in crate::project_slice::gcode_emit) small_perimeter_speed: f64,
    pub(in crate::project_slice::gcode_emit) outer_wall_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) default_acceleration: u32,
}

use super::super::helpers::{absolute, acceleration};

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
    let configured_small_perimeter_speed = absolute(small_perimeter, outer_wall_speed);
    let small_perimeter_speed = if configured_small_perimeter_speed > 0.0 {
        configured_small_perimeter_speed
    } else {
        0.5 * outer_wall_speed
    };
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
