//! `output_gcode_line` + `push_line_to_output` (`PressureEqualizer.cpp:482-730`
//! and `:868-930`): re-emission of modified extrusion lines as
//! slope-smoothed segment chains.

use super::line::{E, F, GCodeLine, X, Y, Z};
use super::{EXTERNAL_PERIMETER_TAG, EXTRUDE_END_TAG, EXTRUDE_SET_SPEED_TAG};
use crate::ExtrusionRole;

/// `NON_TRIVIAL_RATE_DELTA` (`PressureEqualizer.cpp:503`).
pub(super) const NON_TRIVIAL_RATE_DELTA: i32 = 10;

/// Output buffer semantics of `push_to_output` + the
/// `output_buffer_prev_length` rollback (`PressureEqualizer.cpp:905-917`).
#[derive(Default)]
pub(crate) struct OutputBuffer {
    out: String,
    prev_len: usize,
}

impl OutputBuffer {
    fn push(&mut self, text: &str, add_eol: bool) {
        if !text.is_empty() {
            self.prev_len = self.out.len();
            self.out.push_str(text);
        }
        if add_eol {
            self.out.push('\n');
        }
    }

    /// Roll the last pushed line back (the bare set-speed line removal,
    /// `PressureEqualizer.cpp:909-912`).
    fn rollback_last_line(&mut self) {
        self.out.truncate(self.prev_len);
    }

    pub(crate) fn finish(self) -> String {
        self.out
    }
}

/// `is_just_line_with_extrude_set_speed_tag` (`PressureEqualizer.cpp:862-884`).
fn is_just_line_with_extrude_set_speed_tag(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("G1 ") else {
        return false;
    };
    let Some((value, tail)) = rest.split_once('F') else {
        return false;
    };
    if !value.trim().is_empty() {
        return false;
    }
    let after_value = skip_number(tail);
    after_value.map_or(false, |tail| tail == EXTRUDE_SET_SPEED_TAG)
}

fn skip_number(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut end = 0;
    if end < bytes.len() && (bytes[end] == b'-' || bytes[end] == b'+') {
        end += 1;
    }
    while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
        end += 1;
    }
    (end > 0).then(|| &text[end..])
}

/// `GCodeFormatter::quantize` + trailing-zero trimming (`GCodeWriter.hpp`).
pub(super) fn format_axis(value: f64, digits: usize) -> String {
    let factor = 10f64.powi(digits as i32);
    let quantized = (value * factor).round() / factor;
    let mut text = format!("{quantized:.digits$}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    // Upstream `GCodeFormatter::emit_axis` (`GCodeWriter.cpp:1248-1261`)
    // writes the scaled integer and inserts the decimal point before the
    // last `digits` digits, so 0 < |v| < 1 has NO leading zero (".58733",
    // "-.37351"); only a bare zero keeps it.
    if let Some(rest) = text.strip_prefix("0.") {
        text = format!(".{rest}");
    } else if let Some(rest) = text.strip_prefix("-0.") {
        text = format!("-.{rest}");
    }
    text
}

/// `push_line_to_output` (`PressureEqualizer.cpp:899-930`).
#[allow(clippy::too_many_arguments)]
pub(super) fn push_line_to_output(
    buffer: &mut OutputBuffer,
    line: &GCodeLine,
    use_relative_e_distances: bool,
    new_feedrate: f32,
    comment: Option<&str>,
) {
    // Orca: sanity check, 1 mm/s is the minimum feedrate.
    let mut new_feedrate = new_feedrate;
    if new_feedrate < 60.0 {
        new_feedrate = 60.0;
    }
    // Quantize speed changes to a minimum of 1mm/s.
    new_feedrate = (new_feedrate / 60.0).round() * 60.0;

    let feedrate_line = format!(
        "G1 F{}{}",
        format_axis(f64::from(new_feedrate), 3),
        EXTRUDE_SET_SPEED_TAG
    );
    if buffer.out.is_empty() {
        buffer.push(EXTRUDE_END_TAG, true);
    } else if let Some(last_line) = buffer.out.lines().next_back() {
        if is_just_line_with_extrude_set_speed_tag(last_line) {
            // Remove the useless bare speed-setting line.
            buffer.rollback_last_line();
        } else {
            buffer.push(EXTRUDE_END_TAG, true);
        }
    }

    let mut feedrate_line = feedrate_line;
    if line.extrusion_role == ExtrusionRole::ExternalPerimeter {
        feedrate_line.push_str(EXTERNAL_PERIMETER_TAG);
    }
    buffer.push(&feedrate_line, true);

    let mut extrusion = String::from("G1");
    for (index, axis) in [X, Y, Z].into_iter().enumerate() {
        if line.pos_provided[index] {
            extrusion.push(' ');
            extrusion.push((b'X' + axis as u8) as char);
            extrusion.push_str(&format_axis(f64::from(line.pos_end[axis]), 3));
        }
    }
    let e_value = if use_relative_e_distances {
        line.pos_end[E] - line.pos_start[E]
    } else {
        line.pos_end[E]
    };
    extrusion.push_str(" E");
    extrusion.push_str(&format_axis(f64::from(e_value), 5));
    if let Some(comment) = comment {
        extrusion.push_str(comment);
    }
    buffer.push(&extrusion, true);
}

/// `output_gcode_line` (`PressureEqualizer.cpp:482-730`): re-emit `line`
/// (already slope-limited). `lines` is the whole layer window — the
/// accel-then-decel path mutates the line's positions as it splits, so
/// the caller passes it as `&mut` via the split state below.
pub(crate) fn output_gcode_line(
    buffer: &mut OutputBuffer,
    line: &mut GCodeLine,
    max_segment_length: f32,
    use_relative_e_distances: bool,
) {
    if !line.modified {
        buffer.push(&line.raw, true);
        return;
    }

    let comment: Option<String> = line.raw.find(';').map(|index| line.raw[index..].to_owned());

    let length = line.dist_xyz();
    let mut segments = (length / max_segment_length).ceil() as usize;

    let delta_volumetric_rate = ([
        (line.volumetric_extrusion_rate_end - line.volumetric_extrusion_rate_start).abs(),
        (line.volumetric_extrusion_rate - line.volumetric_extrusion_rate_start).abs(),
        (line.volumetric_extrusion_rate - line.volumetric_extrusion_rate_end).abs(),
    ])
    .into_iter()
    .fold(f32::MIN, f32::max)
    .round() as i32;

    if segments == 1 || delta_volumetric_rate < NON_TRIVIAL_RATE_DELTA {
        let feedrate = line.feedrate() * line.volumetric_correction_avg();
        push_line_to_output(
            buffer,
            line,
            use_relative_e_distances,
            feedrate,
            comment.as_deref(),
        );
        return;
    }

    let original_feedrate = line.feedrate();
    // Update the initial and final feed rates.
    line.pos_start[F] =
        line.volumetric_extrusion_rate_start * line.pos_end[F] / line.volumetric_extrusion_rate;
    line.pos_end[F] =
        line.volumetric_extrusion_rate_end * line.pos_end[F] / line.volumetric_extrusion_rate;

    if line.volumetric_extrusion_rate > line.volumetric_extrusion_rate_start
        && line.volumetric_extrusion_rate > line.volumetric_extrusion_rate_end
    {
        if emit_accel_then_decel(
            buffer,
            line,
            length,
            max_segment_length,
            use_relative_e_distances,
            comment.as_deref(),
        ) {
            return;
        }
    }

    emit_single_slope(
        buffer,
        line,
        length,
        max_segment_length,
        segments,
        use_relative_e_distances,
        comment.as_deref(),
    );
    let _ = original_feedrate;
}

/// The accel-then-decel fast path (`PressureEqualizer.cpp:531-632`).
/// Returns `true` when the path fully emitted the line.
#[allow(clippy::too_many_lines)]
mod split;

use split::{emit_accel_then_decel, emit_single_slope};

mod tests;
