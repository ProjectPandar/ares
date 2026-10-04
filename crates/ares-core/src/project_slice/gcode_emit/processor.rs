mod arc_accounting;
mod delays;
mod envelope;
mod estimate;
mod export_lines;
pub(super) mod filament_change;
mod motion;
mod motion_util;
mod moves;
mod schedule;
mod stats;
mod time;
use crate::options::GCodeFlavor;
use estimate::Estimate;
use export_lines::ExportLines;
use moves::MoveRecorder;
pub(super) use stats::{PRINT_TIME_SEC_PLACEHOLDER, USED_FILAMENT_LENGTH_PLACEHOLDER};
use time::{duration, minutes};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ProcessorLimits {
    pub(super) print_acceleration: f64,
    pub(super) retract_acceleration: f64,
    pub(super) travel_acceleration: f64,
    pub(super) gcode_flavor: GCodeFlavor,
    pub(super) bbl_printer: bool,
    pub(super) junction_deviation: f64,
    pub(super) max_feedrate: [f64; 4],
    pub(super) max_acceleration: [f64; 4],
    pub(super) jerk: [f64; 4],
    /// Firmware-retraction emulation inputs (`GCodeProcessor.cpp:4873-
    /// 4893`: G10/G11 expand to fake G1s with these E/F words).
    pub(super) retraction_length: f64,
    pub(super) retract_restart_extra: f64,
    pub(super) retraction_feedrate: f64,
    pub(super) deretraction_feedrate: f64,
}

pub(super) struct ProcessorConfig<'a> {
    limits: ProcessorLimits,
    settings: &'a crate::ProjectSettings,
    /// Also record `GCodeProcessorResult::moves` (`GCodeProcessor::process_file`).
    record_moves: bool,
}

impl ProcessorLimits {
    pub(super) fn with_settings(self, settings: &crate::ProjectSettings) -> ProcessorConfig<'_> {
        ProcessorConfig {
            limits: self,
            settings,
            record_moves: false,
        }
    }
}

impl ProcessorConfig<'_> {
    pub(super) fn recording_moves(self, record_moves: bool) -> Self {
        Self {
            record_moves,
            ..self
        }
    }
}

#[cfg(test)]
impl Default for ProcessorLimits {
    fn default() -> Self {
        Self {
            print_acceleration: 0.0,
            retract_acceleration: 0.0,
            travel_acceleration: 0.0,
            gcode_flavor: GCodeFlavor::default(),
            bbl_printer: false,
            junction_deviation: 0.0,
            max_feedrate: [0.0; 4],
            max_acceleration: [0.0; 4],
            jerk: [9.0, 9.0, 3.0, 2.5],
            retraction_length: 0.0,
            retract_restart_extra: 0.0,
            retraction_feedrate: 0.0,
            deretraction_feedrate: 0.0,
        }
    }
}

pub(super) fn process(
    output: Vec<u8>,
    emit_progress: bool,
    machine_load_filament_time: f64,
    used_filament: f64,
    config: ProcessorConfig<'_>,
) -> crate::ProcessedGCode {
    let limits = config.limits;
    let text = String::from_utf8(output).expect("generated G-code is UTF-8");
    let lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let record_moves = config.record_moves;
    let mut recorder = record_moves.then(|| MoveRecorder::new(&config));
    let estimate = Estimate::record(
        &lines,
        machine_load_filament_time,
        &config,
        recorder.as_mut(),
    );
    let mut result = ExportLines::new(
        config.settings,
        limits.gcode_flavor,
        limits.bbl_printer,
        text.len(),
        record_moves.then_some(lines.len()),
    );
    let mut last_progress = None;
    let first_marker = lines
        .iter()
        .position(|line| line == "M73 P0 R0")
        .unwrap_or(0);
    let prepare_time = estimate.prepare;
    let model_time = (estimate.total - prepare_time).max(0.0);
    let print_time_sec = format!("{:.2}", estimate.total);
    let used_filament_length = format!("{:.2}", used_filament / 1_000.0);

    for (index, line) in lines.iter().enumerate() {
        let expanded = stats::expand(line, &print_time_sec, &used_filament_length);
        let line = expanded.as_ref();
        result.begin(line, estimate.export_elapsed_at(index + 1));
        if line == "M73 P0 R0" {
            if emit_progress {
                let remaining = minutes(estimate.total);
                last_progress = Some((0, remaining));
                result.append(format!("M73 P0 R{remaining}\n"));
            }
            continue;
        }
        if !emit_progress && line == "M73 P100 R0" {
            continue;
        }
        if line.starts_with("; model printing time:")
            || line == "; estimated printing time (normal mode) = 0s"
        {
            if limits.bbl_printer {
                result.append(format!(
                    "; model printing time: {}; total estimated time: {}\n",
                    duration(model_time),
                    duration(estimate.total),
                ));
            } else {
                result.append(format!(
                    "; estimated printing time (normal mode) = {}\n",
                    duration(estimate.total),
                ));
            }
            continue;
        }
        if line.starts_with("; estimated first layer printing time") {
            result.append(format!(
                "; estimated first layer printing time (normal mode) = {}\n",
                duration(prepare_time),
            ));
            continue;
        }
        result.append_from(format!("{line}\n"), index + 1);
        if index < first_marker || !emit_progress || !is_progress_motion(line) {
            continue;
        }
        let Some(elapsed) = estimate.elapsed_at(index + 1).map(|elapsed| elapsed as f32) else {
            continue;
        };
        // Upstream computes `int(100.0f * elapsed_time / machine.time)`
        // (`GCodeProcessor.cpp:1251-1252`): the 100× product stays f32, then
        // the division promotes to f64 against `machine.time` — a `double`
        // (`GCodeProcessor.hpp:544`), matching `estimate.total` in f64.
        // `time_in_minutes(machine.time - elapsed_time)` subtracts the
        // f32 `elapsed_time` from the f64 `machine.time` in f64, then
        // `time_in_minutes` narrows to float (:1013).
        let percent = if estimate.total > 0.0 {
            (f64::from(100.0_f32 * elapsed) / estimate.total) as u64
        } else {
            0
        };
        let remaining = minutes(estimate.total - f64::from(elapsed));
        if last_progress != Some((percent, remaining)) {
            last_progress = Some((percent, remaining));
            result.append(format!("M73 P{percent} R{remaining}\n"));
        }
    }
    let (gcode, line_map) = result.finish();
    crate::ProcessedGCode {
        gcode,
        result: recorder
            .zip(line_map)
            .map(|(recorder, line_map)| recorder.finish(&line_map))
            .unwrap_or_default(),
    }
}

fn is_progress_motion(line: &str) -> bool {
    matches!(
        line.split(';')
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .next(),
        Some("G0" | "G1" | "G2" | "G3")
    )
}

#[cfg(test)]
mod arc_cache_tests;
#[cfg(test)]
mod envelope_replay_tests;
#[cfg(test)]
mod estimate_tests;
#[cfg(test)]
mod gt_oracle_tests;
#[cfg(test)]
mod motion_parse_tests;
#[cfg(test)]
mod seam_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod trapezoid_tests;
#[cfg(test)]
mod zero_acceleration_tests;
