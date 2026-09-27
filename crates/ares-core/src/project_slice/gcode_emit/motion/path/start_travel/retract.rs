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
    destination_feature: &str,
    replan: &mut dyn FnMut(&mut EmitState),
) {
    // `GCode::_retract` skips `m_writer.retract()` when the first layer
    // runs a Hilbert bottom pattern, or when the destination is
    // TopSolidInfill under a Hilbert top pattern
    // (`GCode.cpp:7667-7672`). No `Extruder::retract` call means no
    // `m_restart_extra` assignment.
    let hilbert_suppressed = (state.layer_index == 0
        && state.options.bottom_surface_pattern_hilbert)
        || (destination_feature == "Top surface" && state.options.top_surface_pattern_hilbert);
    // `needs_retraction` (`GCode.cpp:7528-7647`) does not check the
    // retracted flag — a long non-exempt travel invokes `retract()` even
    // when already retracted, and `Extruder::retract` assigns
    // `m_restart_extra` unconditionally (`Extruder.cpp:77`).
    let upstream_would_retract = routed_length >= state.options.retraction_minimum_travel
        && !skip_retraction
        && !hilbert_suppressed;
    // Lift eligibility is a separate, narrower condition
    // (`GCodeWriter.cpp:637` gates on m_lifted == 0 && m_to_lift == 0).
    let defer_lift_for_retracted_travel = upstream_would_retract
        && state.retracted
        && state.lifted_amount == 0.0
        && state.pending_lift.is_none();
    if retract && !hilbert_suppressed {
        let head_before = (state.x, state.y);
        travel::retract_and_lift(output, state);
        if (state.x, state.y) != head_before {
            // The wipe moved the head; re-plan the route from here
            // (`GCode.cpp:7436-7443`).
            replan(state);
        }
    } else if upstream_would_retract {
        state.restart_extra = state.options.retract_restart_extra;
        if defer_lift_for_retracted_travel
            && state.options.z_hop > 0.0
            && state.options.retraction_length > 0.0
            && travel::lift_is_allowed_at(state, state.layer_z)
        {
            state.pending_lift = Some(travel::lift_mode_for(state, false));
        }
    }
}
