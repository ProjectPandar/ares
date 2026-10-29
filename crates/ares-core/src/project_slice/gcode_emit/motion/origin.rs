//! Local cursor and wipe translation from `GCode.cpp::set_origin` (5659-5669).

use super::EmitState;

pub(in crate::project_slice::gcode_emit) fn set_origin(state: &mut EmitState, origin: (f64, f64)) {
    // Point3's double constructor rounds the scaled translation (`Point.hpp:280`).
    let translation = (
        ((state.origin.0 - origin.0) / state.scale_factor).round() as i64,
        ((state.origin.1 - origin.1) / state.scale_factor).round() as i64,
    );
    state.last_scaled_position.0 += translation.0;
    state.last_scaled_position.1 += translation.1;
    for point in &mut state.wipe_path {
        // Wipe points represent upstream scaled integers, not writer coordinates.
        point.x = ((point.x / state.scale_factor).round() as i64 + translation.0) as f64
            * state.scale_factor;
        point.y = ((point.y / state.scale_factor).round() as i64 + translation.1) as f64
            * state.scale_factor;
    }
    state.origin = origin;
    state.offset = (
        origin.0 - state.extruder_offset.0,
        origin.1 - state.extruder_offset.1,
    );
}
