use super::{EmitState, format};

pub(super) fn set(output: &mut Vec<u8>, state: &mut EmitState, jerk: f64) {
    if jerk < 0.01
        || state
            .last_jerk
            .is_some_and(|last| (last - jerk).abs() < 1.0e-6)
    {
        return;
    }
    state.last_jerk = Some(jerk);
    let limit = |value: f64, maximum: f64| {
        if maximum > 0.0 {
            value.min(maximum)
        } else {
            value
        }
    };
    let x = limit(jerk, state.options.max_jerk_x);
    let y = limit(jerk, state.options.max_jerk_y);
    match state.options.gcode_flavor {
        crate::GCodeFlavor::Klipper => output.extend_from_slice(
            format!(
                "SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY={}\n",
                format::axis(x)
            )
            .as_bytes(),
        ),
        crate::GCodeFlavor::Repetier => {
            output.extend_from_slice(format!("M207 X{}\n", format::axis(x)).as_bytes());
        }
        _ => {
            let mut command = format!("M205 X{} Y{}", format::axis(x), format::axis(y));
            if state.tags.is_bbl() {
                command.push_str(&format!(
                    " Z{} E{}",
                    format::axis(state.options.max_jerk_z),
                    format::axis(state.options.max_jerk_e)
                ));
            }
            command.push('\n');
            output.extend_from_slice(command.as_bytes());
        }
    }
}

/// Klipper clamps the jerk by both the X and the Y machine limits
/// (`GCodeWriter.cpp:332-335`).
pub(super) fn clamp_xy(state: &EmitState, jerk: f64) -> f64 {
    let limit = |value: f64, maximum: f64| {
        if maximum > 0.0 {
            value.min(maximum)
        } else {
            value
        }
    };
    limit(
        limit(jerk, state.options.max_jerk_x),
        state.options.max_jerk_y,
    )
}
