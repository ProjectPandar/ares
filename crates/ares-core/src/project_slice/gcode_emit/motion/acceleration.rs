//! Acceleration/jerk emission per flavor (`GCodeWriter.cpp:216-256,
//! 324-348`).

use super::{EmitState, format, jerk};

fn set_acceleration(output: &mut Vec<u8>, state: &mut EmitState, acceleration: u32, travel: bool) {
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
    let line = match state.options.gcode_flavor {
        crate::GCodeFlavor::Repetier => {
            let code = if separate_travel { "M202" } else { "M201" };
            format!("{code} X{acceleration} Y{acceleration}\n")
        }
        crate::GCodeFlavor::RepRapFirmware | crate::GCodeFlavor::MarlinFirmware => {
            let code = if separate_travel { "M204 T" } else { "M204 P" };
            format!("{code}{acceleration}\n")
        }
        _ => format!("M204 S{acceleration}\n"),
    };
    output.extend_from_slice(line.as_bytes());
    *last = Some(acceleration);
}

/// Klipper merges acceleration and jerk into one `SET_VELOCITY_LIMIT` line
/// (`GCodeWriter.cpp:324-348`, `GCode.cpp:7409-7412`); other flavors emit
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
    let mut line = String::from("SET_VELOCITY_LIMIT");
    let mut empty = true;
    if acceleration != 0 && state.last_acceleration != Some(acceleration) {
        line.push_str(&format!(" ACCEL={acceleration}"));
        if state.options.accel_to_decel_enable {
            // streams as a double with ostream's default 6-significant digits
            let decel = acceleration as f64 * state.options.accel_to_decel_factor / 100.0;
            line.push_str(&format!(" ACCEL_TO_DECEL={}", format::axis(decel)));
        }
        state.last_acceleration = Some(acceleration);
        empty = false;
    }
    if jerk > 0.01
        && !state
            .last_jerk
            .is_some_and(|last| (last - jerk).abs() < 1.0e-6)
    {
        line.push_str(&format!(" SQUARE_CORNER_VELOCITY={}", format::axis(jerk)));
        state.last_jerk = Some(jerk);
        empty = false;
    }
    if !empty {
        line.push('\n');
        output.extend_from_slice(line.as_bytes());
    }
}
