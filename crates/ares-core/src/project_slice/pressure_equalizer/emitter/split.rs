//! Rate-transition splitting — the accel-then-decel peak solver and
//! the single-slope fallback (`PressureEqualizer.cpp:570-730`).

use super::{NON_TRIVIAL_RATE_DELTA, OutputBuffer, format_axis, push_line_to_output};
use crate::project_slice::pressure_equalizer::line::{E, F, GCodeLine};

pub(super) fn emit_accel_then_decel(
    buffer: &mut OutputBuffer,
    line: &mut GCodeLine,
    length: f32,
    max_segment_length: f32,
    use_relative_e_distances: bool,
    comment: Option<&str>,
) -> bool {
    let original_feedrate = f64::from(line.pos_end[F] * line.volumetric_extrusion_rate)
        / f64::from(line.volumetric_extrusion_rate_end.max(f32::EPSILON));
    // total extrusion amount of the original line
    let original_extrusion =
        f64::from(length) * f64::from(line.volumetric_extrusion_rate) / original_feedrate;
    let min_steady_extrusion =
        original_extrusion * f64::from(max_segment_length) / f64::from(length);
    let max_sloped_extrusion = original_extrusion - min_steady_extrusion;
    if max_sloped_extrusion <= 0.0 {
        return false;
    }

    let e2 = f64::from(line.volumetric_extrusion_rate).powi(2);
    let e0_2 = f64::from(line.volumetric_extrusion_rate_start).powi(2);
    let e1_2 = f64::from(line.volumetric_extrusion_rate_end).powi(2);
    let sp = f64::from(line.max_volumetric_extrusion_rate_slope_positive);
    let sn = f64::from(line.max_volumetric_extrusion_rate_slope_negative);
    if sp <= 0.0 || sn <= 0.0 {
        return false;
    }
    let sloped_extrusion = (e2 - e0_2) / 2.0 / sp + (e2 - e1_2) / 2.0 / sn;

    let mut target_max_extrusion_rate = f64::from(line.volumetric_extrusion_rate);
    if sloped_extrusion > max_sloped_extrusion {
        target_max_extrusion_rate =
            ((2.0 * max_sloped_extrusion * sp * sn + sn * e0_2 + sp * e1_2) / (sp + sn)).sqrt();
        if target_max_extrusion_rate <= f64::from(line.volumetric_extrusion_rate_start)
            || target_max_extrusion_rate <= f64::from(line.volumetric_extrusion_rate_end)
        {
            return false;
        }
    }

    let delta = target_max_extrusion_rate
        - f64::from(line.volumetric_extrusion_rate_start)
            .min(f64::from(line.volumetric_extrusion_rate_end));
    if delta.round() < f64::from(NON_TRIVIAL_RATE_DELTA) {
        return false;
    }

    let target_max_feedrate =
        original_feedrate * target_max_extrusion_rate / f64::from(line.volumetric_extrusion_rate);
    let t_acc = (target_max_extrusion_rate - f64::from(line.volumetric_extrusion_rate_start)) / sp;
    let l_acc = t_acc * (target_max_feedrate + f64::from(line.pos_start[F])) / 2.0;
    let t_dec = (target_max_extrusion_rate - f64::from(line.volumetric_extrusion_rate_end)) / sn;
    let l_dec = t_dec * (target_max_feedrate + f64::from(line.pos_end[F])) / 2.0;

    let pos_end_backup = line.pos_end;
    let mut pos_start = line.pos_start;

    // Accel slope.
    let mut segments = ((l_acc / f64::from(max_segment_length)).ceil() as usize).max(1);
    let t = l_acc / f64::from(length);
    let mut pos_slope_end = pos_start;
    for index in 0..4 {
        pos_slope_end[index] =
            pos_start[index] + (pos_end_backup[index] - pos_start[index]) * t as f32;
        line.pos_provided[index] = true;
    }
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        for index in 0..4 {
            line.pos_end[index] = pos_start[index] + (pos_slope_end[index] - pos_start[index]) * t;
        }
        let feedrate = pos_start[F]
            + ((target_max_feedrate as f32) - pos_start[F]) * (i as f32 - 0.5) / segments as f32;
        push_line_to_output(buffer, line, use_relative_e_distances, feedrate, comment);
        pos_start = line.pos_end;
        line.pos_start = line.pos_end;
    }

    // Steady segment.
    let t = (f64::from(length) - l_dec) / f64::from(length);
    for index in 0..4 {
        line.pos_end[index] =
            pos_start[index] + (pos_end_backup[index] - pos_start[index]) * t as f32;
    }
    push_line_to_output(
        buffer,
        line,
        use_relative_e_distances,
        target_max_feedrate as f32,
        None,
    );
    pos_start = line.pos_end;
    line.pos_start = line.pos_end;

    // Decel slope.
    line.pos_start[F] = target_max_feedrate as f32;
    pos_start[F] = target_max_feedrate as f32;
    segments = ((l_dec / f64::from(max_segment_length)).ceil() as usize).max(1);
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        for index in 0..4 {
            line.pos_end[index] = pos_start[index] + (pos_end_backup[index] - pos_start[index]) * t;
        }
        let feedrate =
            pos_start[F] + (pos_end_backup[F] - pos_start[F]) * (i as f32 - 0.5) / segments as f32;
        push_line_to_output(buffer, line, use_relative_e_distances, feedrate, None);
        line.pos_start = line.pos_end;
    }

    for index in 0..4 {
        line.pos_end[index] = pos_end_backup[index];
    }
    push_line_to_output(
        buffer,
        line,
        use_relative_e_distances,
        pos_end_backup[F],
        None,
    );
    true
}

/// The single-slope split (`single_slope_fallback`, `PressureEqualizer.cpp:635-728`).
#[allow(clippy::too_many_lines)]
pub(super) fn emit_single_slope(
    buffer: &mut OutputBuffer,
    line: &mut GCodeLine,
    length: f32,
    max_segment_length: f32,
    mut segments: usize,
    use_relative_e_distances: bool,
    comment: Option<&str>,
) {
    let accelerating = line.volumetric_extrusion_rate_start < line.volumetric_extrusion_rate_end;
    let feed_avg = 0.5 * (line.pos_start[F] + line.pos_end[F]);
    let slope = if accelerating {
        line.max_volumetric_extrusion_rate_slope_positive
    } else {
        line.max_volumetric_extrusion_rate_slope_negative
    };
    let t_total = length / feed_avg;
    let t_acc =
        (line.volumetric_extrusion_rate_start - line.volumetric_extrusion_rate_end).abs() / slope;
    let mut l_acc = length;
    let mut l_steady = 0.0_f32;
    if t_acc < t_total {
        l_acc = t_acc * feed_avg;
        l_steady = length - l_acc;
        if l_steady < 0.5 * max_segment_length {
            l_acc = length;
            l_steady = 0.0;
        } else {
            segments = ((l_acc / max_segment_length).ceil() as usize).max(1);
        }
    }

    let pos_start_backup = line.pos_start;
    let pos_end_backup = line.pos_end;
    let mut pos_start = pos_start_backup;
    let mut pos_end = pos_end_backup;
    let mut pos_end2 = [0.0_f32; 4];
    let mut comment = comment;
    if l_steady > 0.0 {
        if accelerating {
            pos_end2[..4].copy_from_slice(&pos_end_backup[..4]);
            let t = l_acc / length;
            for index in 0..4 {
                pos_end[index] = pos_start[index] + (pos_end[index] - pos_start[index]) * t;
                line.pos_provided[index] = true;
            }
        } else {
            // Emit the steady feed rate segment first.
            let t = l_steady / length;
            for index in 0..4 {
                line.pos_end[index] = pos_start[index] + (pos_end[index] - pos_start[index]) * t;
                line.pos_provided[index] = true;
            }
            push_line_to_output(
                buffer,
                line,
                use_relative_e_distances,
                pos_start[F],
                comment,
            );
            comment = None;
            let steady_feedrate = pos_start[F];
            line.pos_start = line.pos_end;
            pos_start = line.pos_end;
            line.pos_start[F] = steady_feedrate;
            pos_start[F] = steady_feedrate;
        }
    }

    for i in 1..segments {
        let t = i as f32 / segments as f32;
        for index in 0..4 {
            line.pos_end[index] = pos_start[index] + (pos_end[index] - pos_start[index]) * t;
            line.pos_provided[index] = true;
        }
        let feedrate =
            pos_start[F] + (pos_end[F] - pos_start[F]) * (i as f32 - 0.5) / segments as f32;
        push_line_to_output(buffer, line, use_relative_e_distances, feedrate, comment);
        comment = None;
        line.pos_start = line.pos_end;
    }
    if l_steady > 0.0 && accelerating {
        for index in 0..4 {
            line.pos_end[index] = pos_end2[index];
            line.pos_provided[index] = true;
        }
        push_line_to_output(buffer, line, use_relative_e_distances, pos_end[F], comment);
    } else {
        for index in 0..4 {
            line.pos_end[index] = pos_end_backup[index];
            line.pos_provided[index] = true;
        }
        push_line_to_output(buffer, line, use_relative_e_distances, pos_end[F], comment);
    }
    let _ = pos_start_backup;
}
