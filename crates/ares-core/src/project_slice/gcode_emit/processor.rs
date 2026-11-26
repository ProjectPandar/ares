mod arc_accounting;
mod delays;
mod elegoo;
mod envelope;
mod estimate;
mod estimated_stats;
mod export_lines;
mod export_stream;
pub(super) mod filament_change;
mod material_statistics;
mod motion;
mod motion_util;
mod moves;
mod pre_heating;
mod schedule;
mod stats;
mod stop_times;
mod time;
mod used_filaments;
mod virtual_motion;
use crate::options::GCodeFlavor;
use estimate::Estimate;
use moves::{MoveRecorder, RecordMode};
pub(super) use stats::DEFERRED_PLACEHOLDERS;

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
    mass_locator: Option<super::object_mass::Locator<'a>>,
    /// `GCodeProcessorResult::extruder_colors`
    /// (`GCodeProcessor.cpp:2321-2343`).
    extruder_colors: Vec<String>,
    /// `m_writer.extruders()` ids, sorted (`GCode.cpp:1904-1940`).
    writer_extruders: Vec<usize>,
    tower_statistics: (f64, f64),
    /// `m_enable_pre_heating` of `finalize(true)` after the export (`GCodeProcessor.cpp:4000-4005`).
    pre_heating: bool,
    /// `nozzle_group_result.get_nozzle_from_id` consumed by `GCodeProcessor.cpp:2217-2223`.
    nozzle_group_result: &'a [crate::project_slice::gcode_emit::tool_ordering::NozzleGroup],
    nozzle_counts: &'a [i64],
}

impl ProcessorLimits {
    pub(super) fn with_settings(self, settings: &crate::ProjectSettings) -> ProcessorConfig<'_> {
        ProcessorConfig {
            limits: self,
            settings,
            detect_layer_based_on_tag: settings.process.print.spiral_mode.0
                || settings.project.gcode.has_scarf_joint_seam.0,
            record_moves: false,
            mass_locator: None,
            extruder_colors: Vec::new(),
            writer_extruders: Vec::new(),
            tower_statistics: (0.0, 0.0),
            pre_heating: false,
            nozzle_group_result: &[],
            nozzle_counts: &[1],
        }
    }
}

impl<'a> ProcessorConfig<'a> {
    pub(super) fn mass_locator(mut self, locator: Option<super::object_mass::Locator<'a>>) -> Self {
        self.mass_locator = locator;
        self
    }

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

    pub(super) fn tower_statistics(self, tower_statistics: (f64, f64)) -> Self {
        Self {
            tower_statistics,
            ..self
        }
    }

    pub(super) fn pre_heating(self, pre_heating: bool) -> Self {
        Self {
            pre_heating,
            ..self
        }
    }

    pub(super) fn nozzle_counts(self, nozzle_counts: &'a [i64]) -> Self {
        Self {
            nozzle_counts,
            ..self
        }
    }

    pub(super) fn nozzle_groups(
        self,
        nozzle_group_result: &'a [crate::project_slice::gcode_emit::tool_ordering::NozzleGroup],
    ) -> Self {
        Self {
            nozzle_group_result,
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
    let mut out = Vec::new();
    let streamed = process_to_sink(
        gcode.as_bytes().to_vec(),
        (
            post_process,
            // `GCodeProcessor::apply_config` sets load time independently of
            // `finalize(bool)` (`GCodeProcessor.cpp:3217,3558-3560`).
            settings.printer.gcode.machine_load_filament_time.0,
        ),
        limits.with_settings(settings).recording_moves(true),
        None,
        &mut out,
    )
    .expect("in-memory Vec<u8> GCodeSink cannot fail");
    streamed.into_processed(out)
}

pub(super) fn process_to_sink(
    output: Vec<u8>,
    (emit_progress, machine_load_filament_time): (bool, f64),
    config: ProcessorConfig<'_>,
    printable_area_check: Option<&super::PreparedPostClassicTraversal>,
    sink: &mut dyn crate::GCodeSink,
) -> Result<crate::StreamedGCode, crate::SliceError> {
    let limits = config.limits;
    // The injector reads the moves and the line map.
    let record_moves = config.record_moves || config.pre_heating;
    let (bytes_written, mut processor_result, estimate) = {
        let text = std::str::from_utf8(&output).expect("generated G-code is UTF-8");
        let mut line_count = 0usize;
        let mut first_marker = None;
        let mut has_custom_tag = false;
        for line in text.lines() {
            if first_marker.is_none() && line == "M73 P0 R0" {
                first_marker = Some(line_count);
            }
            if !record_moves && !has_custom_tag && line.starts_with(';') {
                has_custom_tag = moves::tags::has_custom_tag(line, limits.bbl_printer);
            }
            line_count += 1;
        }
        let first_marker = first_marker.unwrap_or(0);
        let record_mode = if record_moves {
            RecordMode::Full
        } else if has_custom_tag {
            RecordMode::CacheOnly
        } else {
            RecordMode::StatsOnly
        };
        let mut recorder = MoveRecorder::new(&config, record_mode);
        let estimate = Estimate::record(
            text.lines(),
            machine_load_filament_time,
            &config,
            &mut recorder,
        );
        let materials =
            material_statistics::MaterialStatistics::new(&estimate.print_statistics, &config);
        let prepare_time = estimate.prepare;
        let model_time = (estimate.total - prepare_time).max(0.0);
        let deferred = stats::values(
            estimate.total,
            &materials,
            config.settings.printer.gcode.time_cost.0,
            config.tower_statistics,
        );
        let pass = export_stream::StreamPass {
            text,
            emit_progress,
            first_marker,
            line_count,
            record_moves,
            estimate: &estimate,
            materials: &materials,
            deferred: &deferred,
            prepare_time,
            model_time,
        };
        if let Some(traversal) = printable_area_check {
            let error_code = estimate
                .printable_area
                .check(&traversal.resolved.views.full)?;
            if error_code != 0 {
                let mut gcode = Vec::new();
                run_pass_two(&pass, &config, recorder, &mut gcode)?;
                return Err(super::retained_output::outside_printable_area(
                    error_code, gcode, traversal,
                ));
            }
        }
        let (bytes_written, processor_result) = run_pass_two(&pass, &config, recorder, sink)?;
        (bytes_written, processor_result, estimate)
    };
    drop(output);
    if config.record_moves {
        processor_result.apply_filament_config(&config.settings.filament.gcode);
    }
    let mut statistics = estimate.print_statistics;
    // `GCodeProcessor::get_time` preserves the double machine time.
    statistics.normal_time = estimate.total;
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
    processor_result.initial_layer_time = estimate.initial_layer_time;
    processor_result.extruder_colors = config.extruder_colors;
    processor_result.custom_gcode_times = estimate.custom_gcode_times;
    Ok(crate::StreamedGCode {
        bytes_written,
        warnings: Vec::new(),
        result: processor_result,
        print_statistics,
        archive: None,
    })
}

fn run_pass_two(
    pass: &export_stream::StreamPass<'_>,
    config: &ProcessorConfig<'_>,
    recorder: MoveRecorder<'_>,
    sink: &mut dyn crate::GCodeSink,
) -> Result<(usize, crate::GCodeProcessorResult), crate::SliceError> {
    if !config.pre_heating {
        sink.reserve_hint(pass.text.len());
        let (bytes_written, line_map) = export_stream::emit(pass, config, sink)?;
        let processor_result = line_map
            .map(|line_map| recorder.finish(&line_map))
            .filter(|_| config.record_moves)
            .unwrap_or_default();
        return Ok((bytes_written, processor_result));
    }
    let mut intermediate = Vec::with_capacity(pass.text.len());
    let (_, line_map) = export_stream::emit(pass, config, &mut intermediate)?;
    let line_map = line_map.expect("pre_heating tracks line map");
    let mut full_result = recorder.finish(&line_map);
    let bytes_written = pre_heating::inject_to_sink(
        (pass.text.lines(), &line_map),
        config.settings,
        (config.nozzle_group_result, config.nozzle_counts),
        (&intermediate, &mut full_result.moves),
        sink,
    )?;
    let processor_result = if config.record_moves {
        full_result
    } else {
        Default::default()
    };
    Ok((bytes_written, processor_result))
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
