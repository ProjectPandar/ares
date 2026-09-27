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
    let upstream_would_retract =
        routed_length >= state.options.retraction_minimum_travel && !skip_retraction;
    // Lift eligibility is a separate, narrower condition
    // (`GCodeWriter.cpp:637` gates on m_lifted == 0 && m_to_lift == 0).
    let defer_lift_for_retracted_travel = upstream_would_retract
        && state.retracted
        && state.lifted_amount == 0.0
        && state.pending_lift.is_none();
    if retract {
        if hilbert_wipe_only {
            let head_before = (state.x, state.y);
            travel::retract_and_wipe_suppressed_tail(output, state);
            state.retracted = true;
            if (state.x, state.y) != head_before {
                replan(state);
            }
            // The suppressed tail still runs `reset_e` and the lazy lift
            // (`GCode.cpp:7674-7681`).
            if !state.options.use_relative_e_distances
                && !matches!(
                    state.options.gcode_flavor,
                    crate::GCodeFlavor::Mach3
                        | crate::GCodeFlavor::MakerWare
                        | crate::GCodeFlavor::Sailfish
                )
                && state.e_position.abs() >= 1e-6
            {
                let comment = state
                    .options
                    .gcode_comments
                    .then_some(" ; reset extrusion distance")
                    .unwrap_or("");
                output.extend_from_slice(format!("G92 E0{comment}\n").as_bytes());
                state.e_position = 0.0;
            }
            if state.options.z_hop > 0.0
                && (state.options.retraction_length > 0.0 || state.options.use_firmware_retraction)
                && state.lifted_amount == 0.0
                && state.pending_lift.is_none()
                && travel::lift_is_allowed_at(state, state.writer_z.unwrap_or(state.layer_z))
            {
                state.pending_lift = Some(travel::lift_mode_for(state, false));
            }
        } else if hilbert_suppressed {
            // `GCode.cpp:7667-7672` suppresses ONLY `m_writer.retract()`;
            // `reset_e` (`GCode.cpp:7674`) and the lazy lift still run.
            // `reset_e` emits nothing for a zero E counter (force=false)
            // or Mach3/MakerWare/Sailfish flavors
            // (`GCodeWriter.cpp:480-499`).
            if !state.options.use_relative_e_distances
                && !matches!(
                    state.options.gcode_flavor,
                    crate::GCodeFlavor::Mach3
                        | crate::GCodeFlavor::MakerWare
                        | crate::GCodeFlavor::Sailfish
                )
                && state.e_position.abs() >= 1e-6
            {
                let comment = state
                    .options
                    .gcode_comments
                    .then_some(" ; reset extrusion distance")
                    .unwrap_or("");
                output.extend_from_slice(format!("G92 E0{comment}\n").as_bytes());
                state.e_position = 0.0;
            }
            // `needs_lift` (`GCode.cpp:7678-7681`): configured length OR
            // firmware retraction allows the deferred hop; upstream
            // `lazy_lift` reads the CURRENT writer Z.
            if state.options.z_hop > 0.0
                && (state.options.retraction_length > 0.0 || state.options.use_firmware_retraction)
                && state.lifted_amount == 0.0
                && state.pending_lift.is_none()
                && travel::lift_is_allowed_at(state, state.writer_z.unwrap_or(state.layer_z))
            {
                state.pending_lift = Some(travel::lift_mode_for(state, false));
            }
        } else {
            let head_before = (state.x, state.y);
            travel::retract_and_lift(output, state);
            if (state.x, state.y) != head_before {
                // The wipe moved the head; re-plan the route from here
                // (`GCode.cpp:7436-7443`).
                replan(state);
            }
        }
    } else if upstream_would_retract {
        if !hilbert_suppressed {
            state.restart_extra = state.options.retract_restart_extra;
        } else if !state.options.use_relative_e_distances
            && !matches!(
                state.options.gcode_flavor,
                crate::GCodeFlavor::Mach3
                    | crate::GCodeFlavor::MakerWare
                    | crate::GCodeFlavor::Sailfish
            )
            && state.e_position.abs() >= 1e-6
        {
            let comment = state
                .options
                .gcode_comments
                .then_some(" ; reset extrusion distance")
                .unwrap_or("");
            output.extend_from_slice(format!("G92 E0{comment}\n").as_bytes());
            state.e_position = 0.0;
        }
        if defer_lift_for_retracted_travel
            && state.options.z_hop > 0.0
            && (state.options.retraction_length > 0.0 || state.options.use_firmware_retraction)
            && travel::lift_is_allowed_at(state, state.writer_z.unwrap_or(state.layer_z))
        {
            state.pending_lift = Some(travel::lift_mode_for(state, false));
        }
    }
}
