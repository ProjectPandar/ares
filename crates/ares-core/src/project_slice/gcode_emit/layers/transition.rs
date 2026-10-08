//! The sequential by-object instance transition
//! (`GCode.cpp:3300-3372`): everything written between the previous
//! object's last layer flush and the next object's first layer.

use super::super::motion;
use super::super::motion::format::z as format_z;
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

/// The transition inputs beyond the writer state (`GCode.cpp:3309-3342`).
pub(in crate::project_slice::gcode_emit) struct Transition<'a> {
    pub(in crate::project_slice::gcode_emit) traversal: &'a PreparedPostClassicTraversal,
    pub(in crate::project_slice::gcode_emit) finished_objects: usize,
    pub(in crate::project_slice::gcode_emit) initial_extruder: usize,
    pub(in crate::project_slice::gcode_emit) lift_z: f64,
    pub(in crate::project_slice::gcode_emit) origin: [f64; 2],
}

/// `GCode.cpp:3309-3342`: the transition block for the instance about to
/// print (`finished_objects > 0`). `lift_z` is
/// `m_max_layer_z + z_hop[initial_extruder]` (:3326); `origin` is the
/// incoming instance's origin — the upstream travels to `(0, 0)` after
/// `set_origin(shift)` (:3303/:3327), which is the shift point in plate
/// coordinates. Returns whether the incoming extruder was primed here
/// (`prime_extruder`, :3310-3325).
pub(in crate::project_slice::gcode_emit) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    transition: Transition<'_>,
) -> bool {
    let Transition {
        traversal,
        finished_objects,
        initial_extruder,
        lift_z,
        origin,
    } = transition;
    // `GCode.cpp:3310-3325`: `need_toolchange(initial_extruder_id)` primes
    // the incoming extruder before the travels; the same-extruder boundary
    // retracts. Ares defers the prime flush to its toolchange machinery at
    // the instance's first layer — the towerless fixtures keep one
    // filament, so the exercised path is the retract; the primed bytes
    // port lands with the multi-filament by-object fixtures.
    let prime_extruder = state.extruders.active_id() != initial_extruder;
    motion::tower::retract(output, state);
    // `GCode.cpp:3326`: the safety lift above every printed layer.
    motion::tower::writer_z(output, state, lift_z, "", true);
    // `GCode.cpp:3327-3330`: the travel to the incoming instance's origin
    // with the external motion planner once; the writer lifts again for
    // the planar move (the native pair `G1 Z10.4` / `G1 Z10.8` /
    // `G1 X… Y… Z10.8`).
    let hop_z = lift_z + state.options.z_hop;
    motion::tower::writer_z(output, state, hop_z, "", true);
    let feedrate = motion::tower::travel_feedrate(state);
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} Z{} F{}\n",
            format_z(origin[0]),
            format_z(origin[1]),
            format_z(hop_z),
            motion::tower::format_feedrate(feedrate)
        )
        .as_bytes(),
    );
    state.x = origin[0];
    state.y = origin[1];
    state.positioned = true;
    // `GCode.cpp:3333-3339`: the placeholder parser carries
    // `current_object_idx` while the first-layer bed and nozzle
    // temperatures are written without waiting. The
    // `printing_by_object_gcode` custom template and the bed write land
    // with the fixtures that exercise them (both are empty/unchanged in
    // the towerless single-filament baseline).
    let _ = finished_objects;
    let settings = &traversal.resolved.views.full;
    let temperature =
        super::super::machine::initial_layer_nozzle_temperature(settings, initial_extruder);
    if temperature > 0 {
        output.extend_from_slice(
            format!("M104 S{temperature} ; set nozzle temperature\n").as_bytes(),
        );
    }
    prime_extruder
}
