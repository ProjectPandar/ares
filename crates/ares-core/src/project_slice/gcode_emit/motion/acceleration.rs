//! Acceleration/jerk emission per flavor (`GCodeWriter.cpp:428-481,
//! 535-583`).

use std::io::Write;

use crate::project_slice::gcode_emit::klipper_acceleration::append_klipper_accel_to_decel;

use super::{EmitState, format, jerk, travel::EPSILON};

fn set_acceleration(output: &mut Vec<u8>, state: &mut EmitState, acceleration: u32, travel: bool) {
    let comment = if state.options.gcode_comments {
        " ; adjust acceleration"
    } else {
        ""
    };
    let separate_travel = travel
        && matches!(
            state.options.gcode_flavor,
            crate::GCodeFlavor::Repetier
                | crate::GCodeFlavor::MarlinFirmware
                | crate::GCodeFlavor::RepRapFirmware
        );
    let limit = if travel {
        state.options.max_travel_acceleration
    } else {
        state.options.max_acceleration
    };
    // `GCodeWriter.cpp:218-221`: clamp by the machine limit, then skip zero
    // or unchanged values without touching the cached one.
    let acceleration = if limit > 0 && acceleration > limit {
        limit
    } else {
        acceleration
    };
    let last = if separate_travel {
        &mut state.last_travel_acceleration
    } else {
        &mut state.last_acceleration
    };
    if acceleration == 0 || *last == Some(acceleration) {
        return;
    }
    match state.options.gcode_flavor {
        crate::GCodeFlavor::Repetier => {
            let code = if separate_travel { "M202" } else { "M201" };
            let _ = writeln!(output, "{code} X{acceleration} Y{acceleration}{comment}");
        }
        crate::GCodeFlavor::RepRapFirmware | crate::GCodeFlavor::MarlinFirmware => {
            let code = if separate_travel { "M204 T" } else { "M204 P" };
            let _ = writeln!(output, "{code}{acceleration}{comment}");
        }
        crate::GCodeFlavor::Klipper => {
            let mut line = format!("SET_VELOCITY_LIMIT ACCEL={acceleration}");
            if state.options.accel_to_decel_enable {
                append_klipper_accel_to_decel(
                    &mut line,
                    acceleration,
                    state.options.accel_to_decel_factor,
                );
                if state.options.gcode_comments {
                    line.push_str(" ; adjust ACCEL_TO_DECEL");
                }
            }
            line.push_str(comment);
            line.push('\n');
            output.extend_from_slice(line.as_bytes());
        }
        _ => {
            let _ = writeln!(output, "M204 S{acceleration}{comment}");
        }
    }
    *last = Some(acceleration);
}

pub(in crate::project_slice::gcode_emit) fn set_layer_acceleration_and_jerk(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    acceleration: u32,
    jerk: f64,
) {
    set_acceleration(output, state, acceleration, false);
    jerk::set(output, state, jerk);
}

/// Klipper merges acceleration and jerk into one `SET_VELOCITY_LIMIT` line
/// (`GCodeWriter.cpp:535-583`, `GCode.cpp:7409-7412`); other flavors emit
/// separate acceleration and jerk commands.
pub(in crate::project_slice::gcode_emit) fn set_accel_and_jerk(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    acceleration: u32,
    jerk: f64,
    travel: bool,
) {
    if state.options.gcode_flavor != crate::GCodeFlavor::Klipper {
        set_acceleration(output, state, acceleration, travel);
        jerk::set(output, state, jerk);
        return;
    }
    let acceleration =
        if state.options.max_acceleration > 0 && acceleration > state.options.max_acceleration {
            state.options.max_acceleration
        } else {
            acceleration
        };
    let jerk = jerk::clamp_xy(state, jerk);
    let emit_accel = acceleration != 0 && state.last_acceleration != Some(acceleration);
    let emit_jerk = jerk > 0.01
        && !state
            .last_jerk
            .is_some_and(|last| (last - jerk).abs() < EPSILON);
    if !emit_accel && !emit_jerk {
        return;
    }
    output.extend_from_slice(b"SET_VELOCITY_LIMIT");
    if emit_accel {
        let _ = write!(output, " ACCEL={acceleration}");
        if state.options.accel_to_decel_enable {
            let mut extra = String::new();
            append_klipper_accel_to_decel(
                &mut extra,
                acceleration,
                state.options.accel_to_decel_factor,
            );
            output.extend_from_slice(extra.as_bytes());
        }
        state.last_acceleration = Some(acceleration);
    }
    if emit_jerk {
        let _ = write!(output, " SQUARE_CORNER_VELOCITY={}", format::ostream(jerk));
        state.last_jerk = Some(jerk);
    }
    if state.options.gcode_comments {
        output.extend_from_slice(b" ; adjust VELOCITY_LIMIT(accel/jerk)");
    }
    output.push(b'\n');
}
