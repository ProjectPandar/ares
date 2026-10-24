use super::{EmitState, format, travel::EPSILON};

pub(super) fn set(output: &mut Vec<u8>, state: &mut EmitState, jerk: f64) {
    if jerk < 0.01
        || state
            .last_jerk
            .is_some_and(|last| (last - jerk).abs() < EPSILON)
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
                format::ostream(clamp_xy(state, jerk))
            )
            .as_bytes(),
        ),
        crate::GCodeFlavor::Repetier => {
            output.extend_from_slice(format!("M207 X{}\n", format::ostream(x)).as_bytes());
        }
        _ => {
            // `GCodeWriter::set_jerk_xy` (`GCodeWriter.cpp:295`) writes
            // `M205 X.. Y..` with the stream-default 6-significant-digit
            // format, not the 3-decimal axis quantizer (`1.9624`, not `1.962`).
            let mut command = format!("M205 X{} Y{}", format::ostream(x), format::ostream(y));
            if state.tags.is_bbl() {
                command.push_str(&format!(
                    " Z{} E{}",
                    format::ostream_with_precision(state.options.max_jerk_z, 2),
                    format::ostream_with_precision(state.options.max_jerk_e, 2)
                ));
            }
            if state.options.gcode_comments {
                command.push_str(" ; adjust jerk");
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
