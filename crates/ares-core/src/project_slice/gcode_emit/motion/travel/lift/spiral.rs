//! `GCodeWriter.cpp::_spiral_travel_to_z:1200-1275` (345efec64c7b).

use super::{EmitState, format_axis, format_offset, format_z, z_feedrate};
use crate::geometry::arc_welder::arc_discretization_steps;

pub(super) fn append(output: &mut Vec<u8>, state: &mut EmitState, raised_z: f64, i: f64, j: f64) {
    let feedrate = z_feedrate(state);
    let comment = if state.options.gcode_comments {
        " ; spiral lift Z"
    } else {
        ""
    };
    if state.options.enable_arc_fitting {
        if state.options.gcode_comments {
            output.extend_from_slice(b"G17 ; XY plane for arc\n");
        } else {
            output.extend_from_slice(b"G17\n");
        }
        output.extend_from_slice(
            format!(
                "G3 Z{} I{} J{} P1  F{}{comment}\n",
                format_z(raised_z),
                format_offset(i),
                format_offset(j),
                format_axis(feedrate)
            )
            .as_bytes(),
        );
    } else {
        let z_start = state.writer_z.unwrap_or(state.layer_z);
        let center_x = state.x + i;
        let center_y = state.y + j;
        let radius = (i * i + j * j).sqrt();
        let resolution = state.options.arc_fitting_tolerance;
        let segments = if resolution > 0.0 {
            arc_discretization_steps(radius, std::f64::consts::TAU, resolution).clamp(8, 128)
        } else {
            128
        };
        let start_angle = (state.y - center_y).atan2(state.x - center_x);
        output.reserve(segments * 40);
        output.extend_from_slice(format!("G1 F{}{comment}\n", format_axis(feedrate)).as_bytes());
        for index in 1..segments {
            let progress = index as f64 / segments as f64;
            let angle = start_angle + std::f64::consts::TAU * progress;
            append_point(
                output,
                center_x + radius * angle.cos(),
                center_y + radius * angle.sin(),
                z_start + (raised_z - z_start) * progress,
            );
        }
        append_point(output, state.x, state.y, raised_z);
    }
    state.current_feedrate = feedrate;
    state.record_z(raised_z);
}

fn append_point(output: &mut Vec<u8>, x: f64, y: f64, z: f64) {
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} Z{}\n",
            format_axis(x),
            format_axis(y),
            format_z(z)
        )
        .as_bytes(),
    );
}
