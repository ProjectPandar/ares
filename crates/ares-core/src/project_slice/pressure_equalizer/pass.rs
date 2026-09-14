//! The per-layer pass driver: `process_layer(gcode)` +
//! `advance_segment_beyond_small_gap` (`PressureEqualizer.cpp:96-200`).

use super::emitter::{OutputBuffer, output_gcode_line};
use super::limiter::{RateSlope, adjust_volumetric_rate};
use super::line::{GCodeLine, LineParser};
use super::{MAX_IGNORED_GAP_BETWEEN_EXTRUDING_SEGMENTS, MAX_LOOK_BACK_LIMIT};
use crate::ExtrusionRole;

const ROLE_COUNT: usize = 20;

/// The pass state — one instance per print, buffering the previous
/// layer's G-code (`process_layer(LayerResult&&)` semantics,
/// `PressureEqualizer.cpp:176-200`).
pub(crate) struct PressureEqualizerPass {
    parser: LineParser,
    slopes: [RateSlope; ROLE_COUNT],
    smoothing_external_perimeter_only: bool,
    max_segment_length: f32,
    use_relative_e_distances: bool,
    lines: Vec<GCodeLine>,
    buffered: Option<String>,
}

impl PressureEqualizerPass {
    /// The constructor gate (`PressureEqualizer.cpp:56-80`): the pass is
    /// created only when `max_volumetric_extrusion_rate_slope > 0`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        slope: f64,
        segment_length: f64,
        smoothing_external_perimeter_only: bool,
        filament_diameters_mm: &[f64],
        use_relative_e_distances: bool,
    ) -> Self {
        let per_role = RateSlope {
            positive: (slope * 60.0 * 60.0) as f32,
            negative: (slope * 60.0 * 60.0) as f32,
        };
        let mut slopes = [per_role; ROLE_COUNT];
        // Don't regulate the pressure before and after ironing.
        slopes[super::limiter::role_index(ExtrusionRole::Ironing)] = RateSlope {
            positive: 0.0,
            negative: 0.0,
        };
        Self {
            parser: LineParser::new(filament_diameters_mm, use_relative_e_distances),
            slopes,
            smoothing_external_perimeter_only,
            max_segment_length: segment_length as f32,
            use_relative_e_distances,
            lines: Vec::new(),
            buffered: None,
        }
    }

    /// `process_layer(LayerResult&&)`: process this layer's text and
    /// return the PREVIOUS layer's rewritten G-code (the first call
    /// returns `None` — a NOP layer result upstream).
    pub(crate) fn process_layer(&mut self, gcode: &str) -> Option<String> {
        let previous = self.buffered.take();
        if !gcode.is_empty() {
            self.process_layer_lines(gcode);
        }
        self.buffered = Some(self.emit_all());
        previous
    }

    /// Flush the final buffered layer (`DoExport` ends with a NOP layer
    /// result upstream; ares calls this once after the last layer).
    pub(crate) fn flush(&mut self) -> Option<String> {
        self.buffered.take().map(|gcode| {
            self.buffered = Some(gcode);
            let out = self.buffered.take().unwrap();
            out
        })
    }

    fn process_layer_lines(&mut self, gcode: &str) {
        for raw in gcode.split('\n') {
            if raw.is_empty() {
                continue;
            }
            let mut line = GCodeLine::default();
            if self.parser.parse_line(raw, &mut line).is_some() {
                self.lines.push(line);
            }
            // Marker lines (`;_EXTRUSION_ROLE:`) are filtered out —
            // they only update the parser's role state.
        }

        let mut idx_end_current_extrusion = 0;
        while idx_end_current_extrusion < self.lines.len() {
            let idx_begin_current_extrusion = self.lines[idx_end_current_extrusion..]
                .iter()
                .position(|line| line.extruding())
                .map(|offset| idx_end_current_extrusion + offset)
                .unwrap_or(self.lines.len());
            idx_end_current_extrusion = idx_begin_current_extrusion;

            // Extend the extrusion segment over small travel moves.
            while idx_end_current_extrusion < self.lines.len() {
                let just_after_end = self.lines[idx_end_current_extrusion..]
                    .iter()
                    .position(|line| !line.extruding())
                    .map(|offset| idx_end_current_extrusion + offset)
                    .unwrap_or(self.lines.len());
                idx_end_current_extrusion = just_after_end.saturating_sub(1);
                let continuation = self.advance_segment_beyond_small_gap(idx_end_current_extrusion);
                if continuation > idx_end_current_extrusion {
                    idx_end_current_extrusion = continuation;
                } else {
                    break;
                }
            }

            // Steamroller: sliding-window rate limiting across the segment.
            for i in idx_begin_current_extrusion..idx_end_current_extrusion {
                let start_idx =
                    idx_begin_current_extrusion.max(i.saturating_sub(MAX_LOOK_BACK_LIMIT));
                adjust_volumetric_rate(
                    &mut self.lines,
                    &self.slopes,
                    self.smoothing_external_perimeter_only,
                    start_idx,
                    i,
                );
            }
            idx_end_current_extrusion += 1;
        }
    }

    /// `advance_segment_beyond_small_gap` (`PressureEqualizer.cpp:136-152`).
    fn advance_segment_beyond_small_gap(&self, idx_orig: usize) -> usize {
        debug_assert!(self.lines[idx_orig].extruding());
        let mut distance_traveled = 0.0_f32;
        for idx_cur_pos in (idx_orig + 1)..self.lines.len() {
            if self.lines[idx_cur_pos].extruding() {
                return idx_cur_pos;
            }
            distance_traveled += self.lines[idx_cur_pos].dist_xy2().sqrt();
            if distance_traveled as f64 > MAX_IGNORED_GAP_BETWEEN_EXTRUDING_SEGMENTS {
                return idx_orig;
            }
        }
        idx_orig
    }

    fn emit_all(&mut self) -> String {
        let mut buffer = OutputBuffer::default();
        for line in &mut self.lines {
            output_gcode_line(
                &mut buffer,
                line,
                self.max_segment_length,
                self.use_relative_e_distances,
            );
        }
        self.lines.clear();
        buffer.finish()
    }
}

mod tests;
