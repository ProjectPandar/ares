use super::{EmitState, format_axis, format_offset, format_z};

pub(super) fn append(output: &mut Vec<u8>, state: &mut EmitState, raised_z: f64, i: f64, j: f64) {
    // `GCodeWriter.cpp:861`: the linearized spiral interpolates Z from
    // `z_start = m_pos(2)` — the CURRENT writer z (a stacked eager lift
    // continues upward), not the nominal layer z.
    let z_start = state.writer_z.map_or(state.layer_z, f64::from);
    // `_spiral_travel_to_z` (`GCodeWriter.cpp:881`): under
    // `gcode_comments` the comment is a standalone `;spiral lift Z` line
    // before the feedrate set (arc-fitting uses ` ; XY plane for arc`).
    if state.options.enable_arc_fitting {
        if state.options.gcode_comments {
            output.extend_from_slice(b"G17 ; XY plane for arc\n");
        } else {
            output.extend_from_slice(b"G17\n");
        }
        output.extend_from_slice(
            format!(
                "G3 Z{} I{} J{} P1  F{}\n",
                format_z(raised_z),
                format_offset(i),
                format_offset(j),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
        state.record_z(raised_z);
        return;
    }

    if state.options.gcode_comments {
        output.extend_from_slice(b";spiral lift Z\n");
    }
    output.extend_from_slice(format!("G1 F{}\n", format_axis(state.travel_feedrate)).as_bytes());
    let segments = (8.0 * (0.01 / state.options.arc_fitting_tolerance))
        .round()
        .clamp(4.0, 16.0) as usize;
    let center_x = state.x + i;
    let center_y = state.y + j;
    // `_spiral_travel_to_z` derives the circle radius from
    // `ij_offset.norm()` (`GCodeWriter.cpp:876`) — `sqrt(i*i + j*j)` with
    // its double rounding, not `hypot`.
    let radius = (i * i + j * j).sqrt();
    let start_angle = (state.y - center_y).atan2(state.x - center_x);
    for index in 1..segments {
        let progress = index as f64 / segments as f64;
        let angle = start_angle + std::f64::consts::TAU * progress;
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} Z{}\n",
                super::super::super::super::format_processor_float(center_x + radius * angle.cos()),
                super::super::super::super::format_processor_float(center_y + radius * angle.sin()),
                super::super::super::super::format_processor_float(
                    z_start + (raised_z - z_start) * progress
                )
            )
            .as_bytes(),
        );
    }
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} Z{}\n",
            super::super::super::super::format_processor_float(state.x),
            super::super::super::super::format_processor_float(state.y),
            super::super::super::super::format_processor_float(raised_z)
        )
        .as_bytes(),
    );
    state.record_z(raised_z);
}
