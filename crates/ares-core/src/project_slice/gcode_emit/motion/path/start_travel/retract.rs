//! The travel-retraction decision — the defer flag and the wiping
//! retract with route re-planning (`GCode.cpp:7424-7443`,
//! `GCodeWriter.cpp:637`).

use crate::project_slice::gcode_emit::motion::state::EmitState;
use crate::project_slice::gcode_emit::motion::travel;

/// Runs the wiping retract (re-planning the route when the wipe moved the
/// head) or defers a layer-start lift for an already-retracted nozzle.
/// Returns whether the route was re-planned.
pub(super) fn decide(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    retract: bool,
    routed_length: f64,
    skip_retraction: bool,
    replan: &mut dyn FnMut(&mut EmitState),
) {
    // Upstream `retract()` also runs for a long travel with the
    // extruder already retracted — dE is a no-op but `maybe_zlift`
    // defers, gated on m_lifted == 0 && m_to_lift == 0
    // (GCodeWriter.cpp:637). The lifted_amount model mirrors
    // m_lifted's distance semantics.
    let defer_lift_for_retracted_travel = state.retracted
        && state.lifted_amount == 0.0
        && state.pending_lift.is_none()
        && routed_length >= state.options.retraction_minimum_travel
        && !skip_retraction;
    if retract {
        let head_before = (state.x, state.y);
        travel::retract_and_lift(output, state);
        if (state.x, state.y) != head_before {
            // The wipe moved the head; re-plan the route from here
            // (`GCode.cpp:7436-7443`).
            replan(state);
        }
    } else if defer_lift_for_retracted_travel
        && state.options.z_hop > 0.0
        && state.options.retraction_length > 0.0
        && travel::lift_is_allowed_at(state, state.layer_z)
    {
        state.pending_lift = Some(travel::lift_mode_for(state, false));
    }
}
