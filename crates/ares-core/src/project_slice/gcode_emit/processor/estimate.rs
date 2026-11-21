//! `GCodeProcessor` motion accounting and exported progress-time lookup.

mod commands;
mod custom_time;
mod elapsed;
use commands::{arc_seams, synchronizes_planner};
mod export_cache;
pub(super) mod options_z;
mod seam;
mod timing;
use elapsed::elapsed_snapshots;
use seam::SeamDetector;
use timing::{Metadata, Timing};

use super::arc_accounting::arc_internal_g1_lines;
use super::delays::command_delay;
use super::elegoo;
use super::filament_change::FilamentChange;
use super::motion::{IndexedMotion, MotionState};
use super::motion_util::word;
use super::moves::tags::ParsedLine;
use super::moves::{MoveRecorder, RecordedMotion};
use super::schedule::{DelayTarget, PendingDelay};
use super::stop_times::StopTimes;
use super::used_filaments::UsedFilaments;
use super::{ProcessorConfig, is_progress_motion};
use crate::gcode_processor::printable_area::PrintableAreaTracker;

pub(super) struct Estimate {
    pub(super) total: f64,
    pub(super) prepare: f64,
    pub(super) custom_gcode_times: Vec<crate::gcode_processor::CustomGCodeTime>,
    pub(super) stop_times: StopTimes,
    pub(super) print_statistics: crate::PrintEstimatedStatistics,
    pub(super) printable_area: PrintableAreaTracker,
    elapsed: Vec<Option<elapsed::Snapshot>>,
    export_elapsed: Vec<Option<f32>>,
}
impl Estimate {
    /// Runs the time estimate and, with a `recorder`, records the
    /// `GCodeProcessorResult::moves` of the same pass. The progress lines
    /// read the vertex types, so a pass without one records them unused.
    pub(super) fn record<'a>(
        lines: impl IntoIterator<Item = &'a str> + Clone,
        line_count: usize,
        machine_load_filament_time: f64,
        config: &ProcessorConfig<'_>,
        recorder: Option<&mut MoveRecorder>,
    ) -> Self {
        let mut owned = None;
        let recorder = match recorder {
            Some(recorder) => recorder,
            None => owned.insert(MoveRecorder::new(config)),
        };
        let limits = config.limits;
        let cache_lines = lines.clone();
        let elapsed_lines = lines.clone();
        let mut timing = Timing::new(limits.gcode_flavor);
        let mut state = MotionState::with_limits(limits);
        let mut prepare_stage = false;
        let mut skip_end_gcode_delays = false;
        let mut saw_motion_command = false;
        let mut measure_g29_time = false;
        let mut g29_time_added = false;
        let mut filaments = FilamentChange::new(config, machine_load_filament_time);
        let mut used_filaments = UsedFilaments::new(config);
        let elegoo =
            elegoo::Model::from_printer_model(&config.settings.printer.remaining.printer_model.0);
        let filament_count = config.settings.filament.gcode.filament_diameter.0.len();
        // GCode.cpp's fresh export processor keeps its default XY offset zero.
        let mut printable_area = PrintableAreaTracker::new(config.settings, [0.0; 2]);
        let mut g1_line_id = 0;
        let mut seams = SeamDetector::new(config.detect_layer_based_on_tag);
        for (index, line) in lines.into_iter().enumerate() {
            seams.role(line);
            used_filaments.tags(line, filaments.selection().1);
            printable_area.tag(line, limits.bbl_printer);
            let parsed = ParsedLine::new(line, limits.bbl_printer);
            if let Some(wiping) = parsed.wiping {
                state.set_wiping(wiping);
            }
            match line.trim() {
                "; FEATURE: Custom" | ";TYPE:Custom" => prepare_stage = !saw_motion_command,
                line if line.starts_with("; FEATURE:") || line.starts_with(";TYPE:") => {
                    prepare_stage = false
                }
                _ => {}
            }
            if line.trim() == "; MACHINE_END_GCODE_START" {
                skip_end_gcode_delays = true;
            }
            let code = super::virtual_motion::command(line)
                .unwrap_or_else(|| line.split(';').next().unwrap_or_default().trim());
            let line_start = [
                state.position[0],
                state.position[1],
                state.position[2],
                state.e_position,
            ];
            recorder.machine(&state);
            recorder.line(index + 1, parsed, g1_line_id, line_start);
            if let Some(kind) = parsed
                .custom
                .and_then(|tag| tag.time_kind(filaments.current_filament()))
            {
                timing.custom(g1_line_id, kind, state.klipper, recorder);
            }
            if code.starts_with("M622") && word(code, 'J').unwrap_or(0.0).round() == 1.0 {
                measure_g29_time = true;
            } else if code.starts_with("M623") {
                measure_g29_time = false;
            }
            let is_g29 = code.starts_with("G29") && !code.starts_with("G29.");
            // `GCodeProcessor::process_G29` (`GCodeProcessor.cpp:6367-6375`): only the first G29
            // passing `!s_IsBBLPrinter || m_measure_g29_time` adds time and synchronizes; the
            // `m_g29_time_added` latch is reset per processor run (`:3808`).
            let g29_adds_time =
                is_g29 && (!limits.bbl_printer || measure_g29_time) && !g29_time_added;
            g29_time_added |= g29_adds_time;
            let delay = match is_g29 {
                true if !g29_adds_time => None,
                _ => command_delay(code),
            };
            if synchronizes_planner(
                code,
                g29_adds_time,
                limits.gcode_flavor,
                skip_end_gcode_delays,
            ) {
                timing.flush(
                    state.klipper,
                    delay.map(|seconds| PendingDelay {
                        target: DelayTarget::Any,
                        seconds: seconds as f32,
                    }),
                    recorder,
                );
            }
            let previous_selection = filaments.selection();
            let initialized = filaments.initialized();
            let purge = elegoo.and_then(|model| {
                model.purge(code, filament_count, [line_start[0], line_start[1]])
            });
            let change = if let Some(purge) = &purge {
                recorder.elegoo_purge_temperature(
                    filaments.physical_for(purge.filament),
                    purge.temperature,
                );
                filaments.elegoo_purge(purge.filament, purge.seconds)
            } else {
                filaments.command(code)
            };
            if change.is_some() && initialized {
                used_filaments.tool_change(previous_selection.0, previous_selection.1);
            }
            if let Some(purge) = &purge {
                used_filaments.elegoo_purge(
                    filaments.physical_for(purge.filament),
                    initialized.then_some(previous_selection.1),
                    purge,
                );
            }
            if let Some((filament, extra_time)) = change {
                let vertex = recorder.next_vertex();
                recorder.tool_change(index + 1, filament, line_start);
                recorder.queue_block(vertex);
                timing.push(
                    &commands::tool_change_block(state.klipper),
                    Metadata {
                        id: g1_line_id,
                        remaining: 0,
                        prepare: prepare_stage,
                        eligible: false,
                    },
                    recorder,
                );
                timing.flush(
                    state.klipper,
                    Some(PendingDelay {
                        target: DelayTarget::ToolChange,
                        seconds: extra_time,
                    }),
                    recorder,
                );
            }
            let command = code.split_whitespace().next().unwrap_or_default();
            if command == "VG1" {
                // `process_VG1` advances G1 identity even for no displacement;
                // it queues a default move_id=0 block and never stores a vertex.
                g1_line_id += 1;
                saw_motion_command = true;
            }
            let virtual_block = (command == "VG1")
                .then(|| state.virtual_motion(code))
                .flatten();
            if command == "VG1" && word(code, 'F').is_some() {
                recorder.set_feedrate(state.feedrate, false);
            }
            if command == "VG1" && virtual_block.is_none() {
                continue;
            }
            if let Some(block) = virtual_block {
                used_filaments.motion(block.delta, true, state.wiping, &filaments);
                recorder.queue_virtual();
                timing.push(
                    &block,
                    Metadata {
                        id: g1_line_id,
                        remaining: 0,
                        prepare: prepare_stage,
                        eligible: false,
                    },
                    recorder,
                );
                continue;
            }
            let arc_internal = matches!(command, "G2" | "G3")
                .then(|| arc_internal_g1_lines(code, command, &state));
            let arc_start = state.position;
            let motion_blocks = state.motions_indexed(code);
            for motion in &motion_blocks {
                used_filaments.motion(motion.block.delta, false, state.wiping, &filaments);
                printable_area.motion(
                    motion.block.delta,
                    motion.end,
                    state.wiping,
                    filaments.selection().1,
                );
            }
            let mut seam_vertices = Vec::new();
            let mut cache_eligible = Vec::new();
            // Linear commands produce at most one block; only arcs subdivide.
            // `GCodeProcessor::process_G10/G11` calls `process_G1` after
            // `--m_g1_line_id` (`GCodeProcessor.cpp:4873-4893`), so the
            // fake firmware retract/unretract delta runs the same seam
            // lifecycle (`:4192-4234`) and can close a pending Outer wall
            // seam before the following travel.
            if matches!(command, "G0" | "G1" | "G28" | "G10" | "G11")
                && let Some(IndexedMotion { block, .. }) = motion_blocks.first()
            {
                let extruding = !state.wiping
                    && block.delta[3] > 0.0
                    && (block.delta[0] != 0.0 || block.delta[1] != 0.0);
                let seam_vertex =
                    commands::associate_linear_seam(&mut seams, extruding, &mut state.position);
                // `g1_times_cache` is only populated for blocks whose
                // `result.moves[block.move_id]` vertex is Extrude/Travel/
                // Wipe — the Normal-mode vertex filter ahead of the push
                // (`GCodeProcessor.cpp:656-662`). The seam lifecycle feeds
                // that filter: when a loop closes, `:5702-5704` stores an
                // EXTRA `EMoveType::Seam` vertex AFTER this block's enqueue
                // stamped `move_id` and BEFORE the move's own vertex, so the
                // closing move's block reads the Seam vertex (`:658`:
                // Seam ∉ {Extrude,Travel,Wipe}) and is cache-excluded —
                // exactly one block per closed seam. Upstream's
                // `move_type` (:4259-4277): `m_wiping` forces Wipe for ANY
                // delta (an E-only retract inside a wipe IS cached); without
                // wiping an E-only block is Retract/Unretract — never
                // cached. `wiping || !e_only` plus the seam-closing
                // exclusion reproduce the eligibility set; the `G1 E4`/
                // `G1 E-2.8` standalone retracts and the G10/G11 fake
                // blocks stay out.
                cache_eligible.push((!block.e_only || state.wiping) && seam_vertex.is_none());
                seam_vertices.push(seam_vertex);
            } else if matches!(command, "G2" | "G3") && !motion_blocks.is_empty() {
                // `G2`/`G3` arc: every discretized internal segment is a
                // `process_G1` → `store_move_vertex`, so the seam lifecycle
                // runs PER SEGMENT (`GCodeProcessor.cpp:4194-4225`), not once
                // at the arc level. A segment whose start returns to
                // `first_vertex` while terminating the loop associates a
                // `Seam` vertex with its block → cache-excluded: the segment
                // stores the extra `EMoveType::Seam` vertex (`:5704`) after
                // its block's enqueue, so the vertex filter (`:658`) drops
                // it, collapsing the `remaining` chain `ExportLines::update`
                // walks (:790-800). `arc_start` is the position before
                // `motions` advanced `state.position` to the arc end.
                seam_vertices = arc_seams(
                    &mut seams,
                    &motion_blocks,
                    arc_start,
                    state.wiping,
                    &mut cache_eligible,
                );
            }
            let arc = matches!(command, "G2" | "G3");
            if matches!(command, "G10" | "G11") || word(code, 'F').is_some() {
                recorder.set_feedrate(state.feedrate, arc);
            }
            let base_id =
                g1_line_id + usize::from(matches!(command, "G0" | "G1" | "G28" | "G2" | "G3"));
            let mut start = line_start;
            for (segment, motion) in motion_blocks.iter().enumerate() {
                let vertex = recorder.next_vertex();
                recorder.motion(RecordedMotion {
                    source: index + 1,
                    start,
                    end: motion.end,
                    wiping: state.wiping,
                    linear: !arc,
                    seam: seam_vertices.get(segment).copied().flatten(),
                });
                recorder.queue_block(vertex);
                let motion_id = motion.index * usize::from(arc);
                timing.push(
                    &motion.block,
                    Metadata {
                        id: base_id + motion_id,
                        remaining: arc_internal.map_or(0, |internal| internal - motion.index),
                        prepare: prepare_stage,
                        eligible: cache_eligible[segment],
                    },
                    recorder,
                );
                start = motion.end;
            }
            g1_line_id = base_id + if arc { arc_internal.unwrap_or(0) } else { 0 };
            saw_motion_command |= is_progress_motion(code);
        }
        timing.finish(recorder);
        let export_elapsed = export_cache::snapshots(cache_lines, line_count, &timing.cache);
        let elapsed = elapsed_snapshots(elapsed_lines, line_count, &timing.cache);
        Self {
            total: timing.total,
            custom_gcode_times: timing.custom_gcode_times,
            stop_times: timing.stop_times,
            prepare: f64::from(timing.prepare),
            print_statistics: used_filaments.finish(filaments.selection().1),
            printable_area,
            elapsed,
            export_elapsed,
        }
    }
}
