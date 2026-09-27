mod approach;
mod dispatch;
mod restate;
mod retract;
mod route;
#[cfg(test)]
mod tests;

use super::PathProperties;
use crate::project_slice::gcode_emit::motion::{
    EmitState, LayerGeometry, append_object_start,
    format::{axis as format_axis, z as format_z},
    travel,
};

pub(super) struct Request<'a> {
    pub(super) first_scaled: (i64, i64),
    pub(super) first_x: f64,
    pub(super) first_y: f64,
    pub(super) properties: PathProperties<'a>,
    pub(super) geometry: LayerGeometry<'a>,
}

pub(super) fn emit(output: &mut Vec<u8>, state: &mut EmitState, request: Request<'_>) {
    let Request {
        first_scaled,
        first_x,
        first_y,
        properties,
        geometry,
    } = request;
    // `positioned` mirrors `GCodeWriter::m_is_current_pos_clear`: cleared by
    // custom g-code (timelapse/toolchange), set by every travel. The
    // `never_positioned` flag mirrors `GCode::m_last_pos.is_none()` — true
    // only before the first generated travel — and drives the extra
    // `G1 Z{nominal_z}` re-statement that `_last_pos_undefined` triggers
    // (`GCode.cpp:6381-6386`), which a merely-uncleared position does not.
    let first_position = !state.positioned;
    let never_positioned = state.last_scaled_position.is_none();
    let layer_change_travel = state.layer_change_travel_pending && !first_position;
    let slope_start_z = properties
        .slope
        .filter(|slope| (slope.z_begin - slope.z_end).abs() > super::SOURCE_EPSILON_MM)
        .map(|slope| {
            state.layer_z - f64::from(properties.height)
                + f64::from(properties.height) * slope.z_begin
        });
    let slope_needs_z_travel = slope_start_z.is_some_and(|target| {
        (state.scarf_z.unwrap_or(state.layer_z) - target).abs() >= super::SOURCE_EPSILON_MM
    });
    let target_z = slope_start_z.unwrap_or(state.layer_z);
    let needs_travel =
        first_position || state.last_scaled_position != Some(first_scaled) || slope_needs_z_travel;
    let travel_distance = (first_x - state.x).hypot(first_y - state.y);
    let (mut travel_set_layer_z, eager_lifted_travel, unclear_position_travel) = if needs_travel {
        approach::emit_needs_travel(
            output,
            state,
            approach::Approach {
                first_scaled,
                first_x,
                first_y,
                properties,
                geometry,
                first_position,
                layer_change_travel,
                slope_start_z,
                target_z,
                travel_distance,
            },
        )
    } else {
        (false, false, false)
    };
    if !needs_travel && layer_change_travel {
        let z_feedrate = travel::lift_z_feedrate(state);
        output.extend_from_slice(
            format!("G1 Z{} F{}\n", format_z(target_z), format_axis(z_feedrate)).as_bytes(),
        );
        travel_set_layer_z = true;
        state.current_feedrate = z_feedrate;
        state.record_z(target_z);
    }
    if let Some(z) = slope_start_z {
        state.scarf_z = Some(z);
    }
    state.layer_change_travel_pending = false;
    append_object_start(output, state);
    // Orca `_extrude` (`GCode.cpp:6378-6385`): the print's first extrusion
    // re-states Z to sync the writer with the planned layer height
    // (`_last_pos_undefined`). It applies whenever the travel emitted the
    // Z as a separate unclear-position descend — the eager-lift branch or
    // the unknown-source first travel. The lazy-lift branch's descend above
    // already models it.
    // Upstream re-states `G1 Z{nominal_z}` after the approach when
    // `_last_pos_undefined` (`GCode.cpp:6381-6386`) — even when the split
    // already emitted `G1 Z{target_z}` (the double `G1 Z.2` first travel).
    // A merely-uncleared position (post-timelapse/toolchange) does NOT get
    // the re-statement. The retracted path below descends an eager-lifted
    // nozzle itself (`state.lifted && !travel_set_layer_z`).
    restate::restate(
        output,
        state,
        restate::RestateContext {
            slope_start_z,
            never_positioned,
            travel_set_layer_z,
            eager_lifted_travel,
            unclear_position_travel,
        },
    );
}
