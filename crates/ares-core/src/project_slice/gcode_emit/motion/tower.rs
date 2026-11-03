//! `GCode.cpp::WipeTowerIntegration::append_tcr2` live writer operations.
use super::format::{axis, z as format_z};
use super::{EmitState, LayerGeometry, arc, path, travel};
use crate::geometry::CoordinateScale;

pub(in crate::project_slice::gcode_emit) fn unlift(output: &mut Vec<u8>, state: &mut EmitState) {
    travel::unlift(output, state);
}

pub(in crate::project_slice::gcode_emit) fn retract(output: &mut Vec<u8>, state: &mut EmitState) {
    travel::retract_and_wipe(output, state);
    travel::schedule_normal(state);
    state.extruder.retracted = true;
}

/// The travel feedrate for the transition block's manual moves (the
/// same z/travel feedrate the writer uses, `travel/lift.rs:136`).
pub(in crate::project_slice::gcode_emit) fn travel_feedrate(state: &EmitState) -> f64 {
    travel::lift_z_feedrate(state)
}

/// The feedrate formatting used by the writer's axis moves
/// (`format::axis`, `motion.rs:1`).
pub(in crate::project_slice::gcode_emit) fn format_feedrate(value: f64) -> String {
    super::format::axis(value)
}

/// The transition-boundary retract with the cooling markers off
/// (`GCode.cpp:3308`, `:3325` `retract()`).
pub(in crate::project_slice::gcode_emit) fn transition_retract(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    super::travel::transition_retract(output, state)
}

/// `GCode.cpp:1728-1733`: the travel to the tower, through the grid waypoints `route` plans
/// from the position the retract's wipe leaves (`travel_to_tower_gap`).
pub(in crate::project_slice::gcode_emit) fn travel_to(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    point: [f32; 2],
    route: impl FnOnce(&EmitState) -> Result<Vec<[i64; 2]>, crate::SliceError>,
    geometry: LayerGeometry<'_>,
) -> Result<(), crate::SliceError> {
    retract(output, state);
    state.use_external_mp_once = true;
    for [x, y] in route(state)? {
        path::tower_travel(output, state, (x, y), geometry)?;
    }
    let point = object_point(point, state, geometry.scale);
    path::tower_travel(output, state, point, geometry)?;
    travel::unlift(output, state);
    travel::unretract(output, state);
    Ok(())
}

pub(in crate::project_slice::gcode_emit) fn change_z(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    z: f64,
    comment: &str,
) {
    travel::plain_retract(output, state);
    writer_z(output, state, z, comment, false);
    travel::unretract(output, state);
}

pub(in crate::project_slice::gcode_emit) fn restore_z(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    z: f64,
) {
    writer_z(output, state, z, "Force restore layer Z", true);
    state.record_z(z);
    travel::unlift(output, state);
    travel::unretract(output, state);
}

pub(in crate::project_slice::gcode_emit) fn writer_z(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    z: f64,
    comment: &str,
    force: bool,
) {
    let current = state.writer_z.unwrap_or(0.0);
    let will_move = if state.lifted_amount > 0.0 {
        z < current - state.lifted_amount || z > current
    } else {
        (z - current).abs() >= 1e-4
    };
    if !force && !will_move {
        state.lifted_amount = current - z;
        return;
    }
    let feedrate = travel::lift_z_feedrate(state);
    // `GCodeWriter.hpp:266-267`: empty descriptions never emit a comment.
    let comment = if state.options.gcode_comments && !comment.is_empty() {
        format!(" ; {comment}")
    } else {
        String::new()
    };
    output.extend_from_slice(
        format!("G1 Z{} F{}{comment}\n", format_z(z), axis(feedrate)).as_bytes(),
    );
    state.record_z(z);
    state.current_feedrate = feedrate;
    state.lifted_amount = 0.0;
    state.lifted = false;
    state.template_lifted = false;
}

pub(in crate::project_slice::gcode_emit) fn finish(
    state: &mut EmitState,
    scale: CoordinateScale,
    end: [f32; 2],
    wipe: impl IntoIterator<Item = [f32; 2]>,
) {
    // The discarded writer XY move does not subtract nozzle offsets; it adds the plate origin
    // (`GCode.cpp:1933-1934`).
    let [origin_x, origin_y] = state.xy_offset_f32();
    state.x = f64::from(end[0] + origin_x);
    state.y = f64::from(end[1] + origin_y);
    state.last_scaled_position = object_point(end, state, scale);
    state.last_position_defined = true;
    state.positioned = true;
    state.wipe_path = wipe
        .into_iter()
        .map(|point| {
            let point = object_point(point, state, scale);
            arc::Point {
                x: scale.unscale(point.0),
                y: scale.unscale(point.1),
            }
        })
        .collect();
    state.wipe_start = Some(arc::Point {
        x: state.x,
        y: state.y,
    });
    state.use_external_mp_once = true;
}

pub(in crate::project_slice::gcode_emit) fn change_layer(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    print_z: f64,
) {
    let z = print_z + state.options.z_offset;
    let current = state.writer_z.unwrap_or(0.0);
    let will_move = if state.lifted_amount > 0.0 {
        z < current - state.lifted_amount || z > current
    } else {
        (z - current).abs() >= 1e-4
    };
    state.pending_layer_retract = false;
    if state.options.retract_when_changing_layer && will_move {
        travel::retract_and_wipe(output, state);
        travel::defer_layer_change_lift(state);
        state.extruder.retracted = true;
    }
    super::append_exclude_end(output, state);
    state.layer_change_travel_pending = true;
    state.layer_chunk_index += 1;
    state.layer_z = z;
    state.record_z(z);
}

pub(in crate::project_slice::gcode_emit) fn type1_retract(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    toolchange: bool,
) {
    travel::retract_for_type1(output, state, toolchange);
}

/// `GCode.cpp:824`: nozzle-change start move — grid of the transformed
/// `Vec2f` via `wipe_tower_point_to_object_point`.
pub(in crate::project_slice::gcode_emit) fn type1_travel(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    point: [f32; 2],
    geometry: LayerGeometry<'_>,
) -> Result<(), crate::SliceError> {
    path::tower_travel(
        output,
        state,
        object_point(point, state, geometry.scale),
        geometry,
    )
}

/// `wipe_tower_point_to_object_point` of a plate-frame tower point plus `plate_origin_2d`.
pub(in crate::project_slice::gcode_emit) fn object_point_grid(
    point: [f32; 2],
    state: &EmitState,
    scale: CoordinateScale,
) -> [i64; 2] {
    let (x, y) = object_point(point, state, scale);
    [x, y]
}

/// `GCode.cpp:1036-1043`: the `prime_tower_skip_points` polyline points are
/// grid `Point`s fed straight into `travel_to(p)` (`point_to_gcode`,
/// GCode.cpp:8124-8127) — rescaling them through `f32` shifts boundary
/// cases across the 3-decimal rounding of the next slope-lift top
/// (`GCodeWriter.cpp:685-770`).
pub(in crate::project_slice::gcode_emit) fn type1_travel_scaled(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    point: [i64; 2],
    geometry: LayerGeometry<'_>,
) -> Result<(), crate::SliceError> {
    path::tower_travel(output, state, (point[0], point[1]), geometry)
}

pub(in crate::project_slice::gcode_emit) fn type1_unretract(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    travel::unlift(output, state);
    travel::unretract(output, state);
}

/// `wipe_tower_point_to_object_point` (`GCode.cpp:752-755`) of a plate-frame tower point plus
/// `plate_origin_2d`, the `Vec2f` every tower caller adds (`GCode.cpp:1146-1945`).
fn object_point(point: [f32; 2], state: &EmitState, scale: CoordinateScale) -> (i64, i64) {
    let [origin_x, origin_y] = state.xy_offset_f32();
    let point = [point[0] + origin_x, point[1] + origin_y];
    (
        ((f64::from(point[0]) - state.origin.0) / scale.factor()).round() as i64,
        ((f64::from(point[1]) - state.origin.1) / scale.factor()).round() as i64,
    )
}
