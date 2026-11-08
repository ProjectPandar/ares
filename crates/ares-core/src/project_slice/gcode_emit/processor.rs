mod arc_accounting;
mod delays;
mod envelope;
mod estimate;
mod estimated_stats;
mod export_lines;
pub(super) mod filament_change;
mod material_statistics;
mod motion;
mod motion_util;
mod moves;
mod schedule;
mod stats;
mod stop_times;
mod time;
mod used_filaments;
mod virtual_motion;
use crate::options::GCodeFlavor;
use estimate::Estimate;
use export_lines::ExportLines;
use moves::MoveRecorder;
pub(super) use stats::{PRINT_TIME_SEC_PLACEHOLDER, USED_FILAMENT_LENGTH_PLACEHOLDER};
use time::{duration, minutes};

pub(super) fn extruder_colors(
    filament: &crate::options::FilamentGCodeSourceOptions,
) -> Vec<String> {
    // `GCodeProcessorResult::extruder_colors`: `filament_colour` padded
    // to `filaments_count`, missing/empty slots → `#FF8000`
    // (`GCodeProcessor::apply_config`, `GCodeProcessor.cpp:2321-2343`).
    (0..filament.filament_diameter.0.len())
        .map(|id| {
            filament
                .filament_colour
                .0
                .get(id)
                .filter(|value| !value.is_empty())
                .cloned()
                .unwrap_or_else(|| "#FF8000".to_string())
        })
        .collect::<Vec<_>>()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ProcessorLimits {
    pub(super) print_acceleration: f64,
    pub(super) retract_acceleration: f64,
    pub(super) travel_acceleration: f64,
    pub(super) gcode_flavor: GCodeFlavor,
    pub(super) bbl_printer: bool,
    pub(super) junction_deviation: f64,
    pub(super) min_extruding_rate: f64,
    pub(super) min_travel_rate: f64,
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
    detect_layer_based_on_tag: bool,
    /// Also record `GCodeProcessorResult::moves` (`GCodeProcessor::process_file`).
    record_moves: bool,
    /// `GCodeProcessorResult::extruder_colors`
    /// (`GCodeProcessor.cpp:2321-2343`).
    extruder_colors: Vec<String>,
    /// `m_writer.extruders()` ids, sorted (`GCode.cpp:1904-1940`).
    writer_extruders: Vec<usize>,
}

impl ProcessorLimits {
    pub(super) fn with_settings(self, settings: &crate::ProjectSettings) -> ProcessorConfig<'_> {
        ProcessorConfig {
            limits: self,
            settings,
            detect_layer_based_on_tag: settings.process.print.spiral_mode.0
                || settings.project.gcode.has_scarf_joint_seam.0,
            record_moves: false,
            extruder_colors: Vec::new(),
            writer_extruders: Vec::new(),
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

    pub(super) fn extruder_colors(self, extruder_colors: Vec<String>) -> Self {
        Self {
            extruder_colors,
            ..self
        }
    }

    pub(super) fn writer_extruders(self, writer_extruders: Vec<usize>) -> Self {
        Self {
            writer_extruders,
            ..self
        }
    }
}

/// `GCodeProcessor::apply_config` + `process_file`/`process_buffer` and
/// `finalize` (`GCodeProcessor.hpp:887,905,914-915`; cpp:2025,2677-2709).
/// `post_process` selects the existing progress pass (`hpp:915::finalize(bool)`).
pub fn process_gcode(
    gcode: &str,
    settings: &crate::ProjectSettings,
    bbl_printer: bool,
    post_process: bool,
) -> crate::ProcessedGCode {
    let limits = ProcessorLimits::from_config(
        &settings.printer.machine,
        settings.printer.gcode.gcode_flavor,
        bbl_printer,
    );
    let (mut output, _) = process(
        gcode.as_bytes().to_vec(),
        post_process,
        if post_process {
            settings.printer.gcode.machine_load_filament_time.0
        } else {
            0.0
        },
        limits.with_settings(settings).recording_moves(true),
    );
    output
        .result
        .apply_filament_config(&settings.filament.gcode);
    output
}

pub(super) fn process(
    output: Vec<u8>,
    emit_progress: bool,
    machine_load_filament_time: f64,
    config: ProcessorConfig<'_>,
) -> (
    crate::ProcessedGCode,
    crate::gcode_processor::printable_area::PrintableAreaTracker,
) {
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
    let materials =
        material_statistics::MaterialStatistics::new(&estimate.print_statistics, &config);
    let mut result = ExportLines::new(
        config.settings,
        limits.gcode_flavor,
        limits.bbl_printer,
        text.len(),
        record_moves.then_some(lines.len()),
    );
    let mut last_progress = None;
    let mut stop_export = estimate.stop_times.exporter(estimate.total);
    let first_marker = lines
        .iter()
        .position(|line| line == "M73 P0 R0")
        .unwrap_or(0);
    let prepare_time = estimate.prepare;
    let model_time = (estimate.total - prepare_time).max(0.0);
    let print_time_sec = format!("{:.2}", estimate.total);
    let used_filament_length = format!(
        "{:.2}",
        estimate.print_statistics.total_used_filament / 1_000.0
    );

    for (index, line) in lines.iter().enumerate() {
        let material = materials.process(line);
        let line = material.as_deref().unwrap_or(line);
        let expanded = stats::expand(line, &print_time_sec, &used_filament_length);
        let line = expanded.as_ref();
        result.begin(line, estimate.export_elapsed_at(index + 1));
        if line == "M73 P0 R0" {
            if emit_progress {
                let remaining = minutes(estimate.total);
                last_progress = Some((0, remaining));
                result.append(format!("M73 P0 R{remaining}\n"));
            }
            if emit_progress && let Some(line) = stop_export.initial() {
                result.append(line);
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
        if let Some(line) = stop_export.progress(elapsed, estimate.next_elapsed_at(index + 1)) {
            result.append(line);
        }
    }
    let (gcode, line_map) = result.finish();
    let mut processor_result = recorder
        .zip(line_map)
        .map(|(recorder, line_map)| recorder.finish(&line_map))
        .unwrap_or_default();
    let mut statistics = estimate.print_statistics;
    // `GCodeProcessor::get_time` returns the float machine time.
    statistics.normal_time = estimate.total as f32;
    // `estimate.prepare` is the f32 prepare-stage sum widened to f64.
    statistics.normal_prepare_time = estimate.prepare as f32;
    let print_statistics = estimated_stats::update_print_estimated_stats(
        &statistics,
        &config.writer_extruders,
        config.settings,
    );
    statistics.volumes_per_color_change =
        std::mem::take(&mut processor_result.print_statistics.volumes_per_color_change);
    processor_result.print_statistics = statistics;
    processor_result.extruder_colors = config.extruder_colors;
    processor_result.custom_gcode_times = estimate.custom_gcode_times;
    (
        crate::ProcessedGCode {
            warnings: Vec::new(),
            gcode,
            result: processor_result,
            print_statistics,
        },
        estimate.printable_area,
    )
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
