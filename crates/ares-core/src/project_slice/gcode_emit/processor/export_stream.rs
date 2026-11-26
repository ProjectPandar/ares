//! Pass-2 post-processed G-code export loop streaming into [`crate::GCodeSink`]
//! (`GCodeProcessor.cpp:1169-1488`).

use super::estimate::{
    Estimate, elapsed::ProgressElapsedCursor, export_cache::ExportElapsedCursor,
};
use super::export_lines::ExportLines;
use super::material_statistics::MaterialStatistics;
use super::stats::{self, DEFERRED_PLACEHOLDERS};
use super::time::{duration, minutes};
use super::{ProcessorConfig, is_progress_motion};
use crate::{GCodeSink, SliceError};

pub(super) struct StreamPass<'a> {
    pub(super) text: &'a str,
    pub(super) emit_progress: bool,
    pub(super) first_marker: usize,
    pub(super) line_count: usize,
    pub(super) record_moves: bool,
    pub(super) estimate: &'a Estimate,
    pub(super) materials: &'a MaterialStatistics,
    pub(super) deferred: &'a [String; DEFERRED_PLACEHOLDERS.len()],
    pub(super) prepare_time: f64,
    pub(super) model_time: f64,
}

pub(super) fn emit(
    pass: &StreamPass<'_>,
    config: &ProcessorConfig<'_>,
    sink: &mut dyn GCodeSink,
) -> Result<(usize, Option<Vec<u32>>), SliceError> {
    let limits = config.limits;
    let mut result = ExportLines::new(
        config.settings,
        limits.gcode_flavor,
        limits.bbl_printer,
        pass.record_moves.then_some(pass.line_count),
        sink,
    );
    let mut last_progress = None;
    let mut stop_export = pass.estimate.stop_times.exporter(pass.estimate.total);
    let mut export_cursor = ExportElapsedCursor::default();
    let mut progress_cursor = ProgressElapsedCursor::default();

    for (index, original) in pass.text.split_inclusive('\n').enumerate() {
        // GCodeProcessor.cpp:1794-1803 retains CR/LF for every ordinary line;
        // ExportLines (:1027-1040,1105-1133) and GCodeFileWriter (:867-881) keep them.
        let (raw_line, ending) = if let Some(line) = original.strip_suffix("\r\n") {
            (line, "\r\n")
        } else if let Some(line) = original.strip_suffix('\n') {
            (line, "\n")
        } else {
            (original, "\n")
        };
        let export_elapsed = result
            .has_backtrace()
            .then(|| export_cursor.step(raw_line, &pass.estimate.cache))
            .flatten();
        let progress_snapshot = pass
            .emit_progress
            .then(|| progress_cursor.step(raw_line, &pass.estimate.cache))
            .flatten();
        let material = pass.materials.process(raw_line);
        let line = material.as_deref().unwrap_or(raw_line);
        let expanded = stats::expand(line, pass.deferred);
        let line = expanded.as_ref();
        result.begin(line, export_elapsed)?;
        if line == "; EXECUTABLE_BLOCK_START" {
            result.flush_section()?;
        }
        if line == "M73 P0 R0" {
            if pass.emit_progress {
                let remaining = minutes(pass.estimate.total);
                last_progress = Some((0, remaining));
                result.append(format!("M73 P0 R{remaining}\n"))?;
            }
            if pass.emit_progress
                && let Some(line) = stop_export.initial()
            {
                result.append(line)?;
            }
            continue;
        }
        if !pass.emit_progress && line == "M73 P100 R0" {
            continue;
        }
        if line.starts_with("; model printing time:")
            || line == "; estimated printing time (normal mode) = 0s"
        {
            if limits.bbl_printer {
                result.append(format!(
                    "; model printing time: {}; total estimated time: {}\n",
                    duration(pass.model_time),
                    duration(pass.estimate.total),
                ))?;
            } else {
                result.append(format!(
                    "; estimated printing time (normal mode) = {}\n",
                    duration(pass.estimate.total),
                ))?;
            }
            continue;
        }
        if line.starts_with("; estimated first layer printing time") {
            result.append(format!(
                "; estimated first layer printing time (normal mode) = {}\n",
                duration(pass.prepare_time),
            ))?;
            continue;
        }
        result.append_parts(line, ending, index + 1)?;
        if line == "; EXECUTABLE_BLOCK_END" {
            result.flush_section()?;
        }
        if index < pass.first_marker || !pass.emit_progress || !is_progress_motion(line) {
            continue;
        }
        let Some(snapshot) = progress_snapshot else {
            continue;
        };
        let elapsed = snapshot.elapsed;
        // Upstream computes `int(100.0f * elapsed_time / machine.time)`
        // (`GCodeProcessor.cpp:1251-1252`): the 100× product stays f32, then
        // the division promotes to f64 against `machine.time` — a `double`
        // (`GCodeProcessor.hpp:544`), matching `estimate.total` in f64.
        // `time_in_minutes(machine.time - elapsed_time)` subtracts the
        // f32 `elapsed_time` from the f64 `machine.time` in f64, then
        // `time_in_minutes` narrows to float (:1013).
        let percent = if pass.estimate.total > 0.0 {
            (f64::from(100.0_f32 * elapsed) / pass.estimate.total) as u64
        } else {
            0
        };
        let remaining = minutes(pass.estimate.total - f64::from(elapsed));
        if last_progress != Some((percent, remaining)) {
            last_progress = Some((percent, remaining));
            result.append(format!("M73 P{percent} R{remaining}\n"))?;
        }
        if let Some(line) = stop_export.progress(elapsed, snapshot.next) {
            result.append(line)?;
        }
    }
    result.finish()
}
