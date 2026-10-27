//! The travel-retraction decision — the defer flag and the wiping
//! retract with route re-planning (`GCode.cpp:7424-7443`,
//! `GCodeWriter.cpp:637`).

use crate::project_slice::gcode_emit::motion::state::EmitState;
use crate::project_slice::gcode_emit::motion::travel;

/// The `_retract` call context (`GCode.cpp:7424-7443`): whether the
/// route qualified for retraction, whether its coord_t routed length
/// reaches `scale_(retraction_minimum_travel)`, the exemption
/// flag, and the destination feature name used by Hilbert suppression.
pub(super) struct RetractContext<'a> {
    pub(super) retract: bool,
    pub(super) route_reaches_minimum: bool,
    pub(super) skip_retraction: bool,
    pub(super) destination_feature: &'a str,
}

/// Runs the wiping retract (re-planning the route when the wipe moved the
/// head) or defers a layer-start lift for an already-retracted nozzle.
/// Returns whether the route was re-planned.
pub(super) fn decide(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    context: RetractContext<'_>,
    replan: &mut dyn FnMut(&mut EmitState) -> Result<(), crate::SliceError>,
) -> Result<(), crate::SliceError> {
    let RetractContext {
        retract,
        route_reaches_minimum,
        skip_retraction,
        destination_feature,
    } = context;
    // `GCode::_retract` skips `m_writer.retract()` when the first layer
    // runs a Hilbert bottom pattern, or when the destination is
    // TopSolidInfill under a Hilbert top pattern
    // (`GCode.cpp:7667-7672`). No `Extruder::retract` call means no
    // `m_restart_extra` assignment.
    let hilbert_suppressed = ((state.layer_index == 0
        && state.options.bottom_surface_pattern_hilbert)
        || (destination_feature == "Top surface" && state.options.top_surface_pattern_hilbert))
        // With a stored wipe path, upstream still runs
        // `retract(true, beforeWipe)` + the wipe — which assigns
        // `m_restart_extra` and retracts — before the suppressed second
        // call (`GCode.cpp:7659-7672`). Only a wipe-less travel actually
        // suppresses.
        && !(state.options.wipe
            && !state.wipe_path.is_empty()
            // `scale_(wipe_distance) > SCALED_EPSILON` (`GCode.cpp:7659`).
            && state.options.wipe_distance > 1e-4);
    // With an eligible wipe the before-wipe retraction and the wipe still
    // run upstream even under Hilbert suppression — only the final
    // `retract()` is skipped (`GCode.cpp:7659-7672`).
    let hilbert_wipe_only = ((state.layer_index == 0
        && state.options.bottom_surface_pattern_hilbert)
        || (destination_feature == "Top surface" && state.options.top_surface_pattern_hilbert))
        && !hilbert_suppressed;
    // `needs_retraction` (`GCode.cpp:7528-7647`) does not check the
    // retracted flag — a long non-exempt travel invokes `retract()` even
    // when already retracted, and `Extruder::retract` assigns
    // `m_restart_extra` unconditionally (`Extruder.cpp:77`).
    let upstream_would_retract = route_reaches_minimum && !skip_retraction;
    // Lift eligibility is a separate, narrower condition
    // (`GCodeWriter.cpp:637` gates on m_lifted == 0 && m_to_lift == 0).
    let defer_lift_for_retracted_travel = upstream_would_retract
        && state.extruder.retracted
        && state.lifted_amount == 0.0
        && state.pending_lift.is_none();
    // A wipe that moved the head re-plans the route from the wipe's end
    // (`GCode.cpp:7436-7443`).
    let mut wipe_then_replan = |output: &mut Vec<u8>,
                                state: &mut EmitState,
                                emit: fn(&mut Vec<u8>, &mut EmitState)|
     -> Result<(), crate::SliceError> {
        let head_before = (state.x, state.y);
        emit(output, state);
        if (state.x, state.y) != head_before {
            replan(state)?;
        }
        Ok(())
    };
    if retract {
        if hilbert_wipe_only {
            wipe_then_replan(output, state, travel::retract_and_wipe_suppressed_tail)?;
            state.extruder.retracted = true;
            // The suppressed tail still runs `reset_e` and the lazy lift
            // (`GCode.cpp:7674-7681`).
            travel::reset_e(output, state, false);
            travel::schedule_retract_tail(output, state);
        } else if hilbert_suppressed {
            // `GCode.cpp:7667-7672` suppresses ONLY `m_writer.retract()`;
            // `reset_e` (`GCode.cpp:7674`) and the lazy lift still run.
            // `reset_e` emits nothing for a zero E counter (force=false)
            // or Mach3/MakerWare/Sailfish flavors
            // (`GCodeWriter.cpp:480-499`).
            travel::reset_e(output, state, false);
            // `needs_lift` (`GCode.cpp:7678-7681`): configured length OR
            // firmware retraction allows the deferred hop; upstream
            // `lazy_lift` reads the CURRENT writer Z.
            travel::schedule_retract_tail(output, state);
        } else {
            wipe_then_replan(output, state, travel::retract_and_lift)?;
        }
    } else if upstream_would_retract {
        // A fully-retracted nozzle still runs upstream's wipe +
        // `reset_e` side effects (`GCode.cpp:7659-7674`) — the
        // `retract()` E move is a no-op, not the sequence. Hilbert
        // suppression keeps skipping the FINAL retract and preserves the
        // scaled restart extra (`GCode.cpp:7667-7672`).
        // The wipe + `reset_e` run whenever upstream's `retract()`
        // executes — even when its E delta is zero (`retraction_length =
        // 0` or a fully-retracted nozzle; `GCode.cpp:7659-7674`). Hilbert
        // suppression keeps skipping only the final `retract()` and
        // preserves the scaled restart extra (`GCode.cpp:7667-7672`).
        let wipe_eligible =
            !state.wipe_path.is_empty() && state.options.wipe && state.options.wipe_distance > 1e-4;
        if hilbert_wipe_only {
            if wipe_eligible {
                wipe_then_replan(output, state, travel::retract_and_wipe_suppressed_tail)?;
            }
        } else if !hilbert_suppressed {
            state.extruder.restart_extra = state.options.retract_restart_extra;
            if wipe_eligible {
                wipe_then_replan(output, state, travel::wipe_only_retract)?;
            }
        }
        travel::reset_e(output, state, false);
        // The deferred hop is additionally gated on the nozzle already
        // being retracted (`defer_lift_for_retracted_travel`);
        // `schedule_retract_tail` re-checks the shared needs_lift band
        // (`GCode.cpp:7678-7681`).
        if defer_lift_for_retracted_travel {
            travel::schedule_retract_tail(output, state);
        }
    }
    Ok(())
}
