//! `GCodeWriter.cpp:1056–1094`: writer-only E restoration and unlift.
use super::super::{
    EmitState, extrusion,
    format::{axis, extrusion as format_extrusion, z},
};

pub(in crate::project_slice::gcode_emit::motion) fn unlift(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if state.lifted_amount > 0.0 {
        let target = state.writer_z.expect("lift retains writer Z") - state.lifted_amount;
        let feedrate = super::lift_z_feedrate(state);
        let comment = if state.options.gcode_comments {
            " ; restore layer Z"
        } else {
            ""
        };
        output.extend_from_slice(
            format!("G1 Z{} F{}{comment}\n", z(target), axis(feedrate)).as_bytes(),
        );
        state.record_z(target);
        state.current_feedrate = feedrate;
    }
    state.lifted_amount = 0.0;
    state.lifted = false;
    state.template_lifted = false;
    state.pending_lift = None;
}

pub(in crate::project_slice::gcode_emit::motion) fn unretract(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if state.options.gcode_flavor == crate::GCodeFlavor::MakerWare {
        output.extend_from_slice(b"M101 ; extruder on\n");
    }
    let delta = state.extruder.retracted_amount + state.extruder.restart_extra;
    let coordinate = extrusion::coordinate(state, delta);
    state.extruder.retracted_amount = 0.0;
    state.extruder.restart_extra = 0.0;
    state.extruder.retracted = false;
    if delta.abs() < 1e-6 {
        return;
    }
    if state.options.use_firmware_retraction {
        output.extend_from_slice(
            if state.options.gcode_flavor == crate::GCodeFlavor::Machinekit {
                b"G23 ; unretract\n"
            } else {
                b"G11 ; unretract\n"
            },
        );
        super::reset_e(output, state, false);
    } else {
        let comment = if state.options.gcode_comments {
            " ;  ; unretract"
        } else {
            ""
        };
        output.extend_from_slice(
            format!(
                "G1 E{} F{}{comment}\n",
                format_extrusion(coordinate),
                axis(state.options.deretraction_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = state.options.deretraction_feedrate;
    }
}
