//! `GCodeProcessor` motion accounting and exported progress-time lookup.

mod commands;
mod custom_time;
mod elapsed;
use commands::{arc_seams, record_motions, synchronizes_planner};
mod export_cache;
pub(super) mod options_z;
mod seam;
use elapsed::elapsed_snapshots;
use seam::SeamDetector;

use super::arc_accounting::arc_internal_g1_lines;
use super::delays::command_delay;
use super::filament_change::FilamentChange;
use super::motion::{IndexedMotion, MotionState};
use super::motion_util::word;
use super::moves::tags::ParsedLine;
use super::moves::{MoveRecorder, RecordedMotion};
use super::schedule::{DelayTarget, FlushEvent, PendingDelay, Schedule, scheduled_times};
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
    pub(super) fn record(
        lines: &[String],
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
        let mut blocks = Vec::new();
        let mut state = MotionState::with_limits(limits);
        let mut prepare_stages = Vec::new();
        let mut events = Vec::new();
        let mut custom_events = Vec::new();
        let mut stop_events = Vec::new();
        let mut prepare_stage = false;
        let mut saw_motion_command = false;
        let mut measure_g29_time = false;
        let mut filaments = FilamentChange::new(config, machine_load_filament_time);
        let mut used_filaments = UsedFilaments::new(config);
        // GCode.cpp's fresh export processor keeps its default XY offset zero.
        let mut printable_area = PrintableAreaTracker::new(config.settings, [0.0; 2]);
        let mut g1_line_id = 0;
        let mut block_line_ids = Vec::new();
        let mut remaining_internal = Vec::new();
        let mut cache_eligible = Vec::new();
        let mut seams = SeamDetector::new(config.detect_layer_based_on_tag);
        for (index, line) in lines.iter().enumerate() {
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
            let code = super::virtual_motion::command(line)
                .unwrap_or_else(|| line.split(';').next().unwrap_or_default().trim());
            let line_start = [
                state.position[0],
                state.position[1],
                state.position[2],
                state.e_position,
            ];
            recorder.machine(&state);
            recorder.options_z.tick(events.len());
            recorder.line(index + 1, parsed, g1_line_id, line_start);
            if let Some(kind) = parsed
                .custom
                .and_then(|tag| tag.time_kind(filaments.current_filament()))
            {
                stop_events.push((g1_line_id, events.len()));
                custom_events.push((kind, events.len()));
                events.push(FlushEvent {
                    block_count: blocks.len(),
                    delay: None,
                    klipper: state.klipper,
                });
            }
            if code.starts_with("M622") && word(code, 'J').unwrap_or(0.0).round() == 1.0 {
                measure_g29_time = true;
            } else if code.starts_with("M623") {
                measure_g29_time = false;
            }
            let is_g29 = code.starts_with("G29") && !code.starts_with("G29.");
            // Orca counts the hardcoded 260s G29 bedding delay on every
            // machine; the M622 J1 gate only applies to BBL printers
            // (`GCodeProcessor.cpp:4859-4869`).
            let bbl_gating_blocks_delay = limits.bbl_printer && !measure_g29_time;
            let delay = match is_g29 {
                true if bbl_gating_blocks_delay => None,
                _ => command_delay(code),
            };
            let measured_g29 = is_g29 && !bbl_gating_blocks_delay;
            if synchronizes_planner(code, measured_g29) {
                events.push(FlushEvent {
                    block_count: blocks.len(),
                    klipper: state.klipper,
                    delay: delay.map(|seconds| PendingDelay {
                        target: DelayTarget::Any,
                        seconds: seconds as f32,
                    }),
                });
            }
            let previous_selection = filaments.selection();
            let initialized = filaments.initialized();
            let change = filaments.command(code);
            if change.is_some() && initialized {
                used_filaments.tool_change(previous_selection.0, previous_selection.1);
            }
            if let Some((filament, extra_time)) = change {
                let vertex = recorder.options_z.next_vertex();
                recorder.tool_change(index + 1, filament, line_start);
                recorder.options_z.block(vertex);
                blocks.push(commands::tool_change_block(state.klipper));
                block_line_ids.push(g1_line_id);
                remaining_internal.push(0);
                cache_eligible.push(false);
                prepare_stages.push(prepare_stage);
                events.push(FlushEvent {
                    block_count: blocks.len(),
                    klipper: state.klipper,
                    delay: Some(PendingDelay {
                        target: DelayTarget::ToolChange,
                        seconds: extra_time,
                    }),
                });
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
                blocks.push(block);
                recorder.options_z.virtual_block();
                block_line_ids.push(g1_line_id);
                prepare_stages.push(prepare_stage);
                cache_eligible.push(false);
                remaining_internal.push(0);
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
                let seam_vertex = seams.associate(extruding, state.position);
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
            record_motions(
                recorder,
                RecordedMotion {
                    source: index + 1,
                    start: line_start,
                    end: line_start,
                    wiping: state.wiping,
                    linear: !arc,
                    seam: None,
                },
                &motion_blocks,
                &seam_vertices,
            );
            remaining_internal.extend(
                motion_blocks
                    .iter()
                    .map(|motion| arc_internal.map_or(0, |internal| internal - motion.index)),
            );
            match command {
                "G0" | "G1" | "G28" => {
                    g1_line_id += 1;
                    let count = motion_blocks.len();
                    blocks.extend(motion_blocks.into_iter().map(|motion| motion.block));
                    block_line_ids.extend(std::iter::repeat_n(g1_line_id, count));
                    prepare_stages.extend(std::iter::repeat_n(prepare_stage, count));
                }
                // `GCodeProcessor::process_G10/G11` decrements the fake
                // G1's line id (`GCodeProcessor.cpp:4873-4893`), so a
                // firmware retract occupies NO g1 line id — the M73 cache
                // alignment skips over it.
                "G10" | "G11" => {
                    let count = motion_blocks.len();
                    blocks.extend(motion_blocks.into_iter().map(|motion| motion.block));
                    block_line_ids.extend(std::iter::repeat_n(g1_line_id, count));
                    prepare_stages.extend(std::iter::repeat_n(prepare_stage, count));
                    // `cache_eligible` was already pushed by the shared
                    // G1-branch above: `e_only && !wiping` keeps the fake
                    // Retract/Unretract out of `g1_times_cache`.
                }
                "G2" | "G3" => {
                    let count = motion_blocks.len();
                    // Every discretized internal `process_G1` consumes its own
                    // g1 line id (`GCodeProcessor.cpp:3868` `++m_g1_line_id`),
                    // including the zero-displacement segments dropped at
                    // :3891 — so a retained block carries
                    // `base + discretization_index` (a hole opens where a
                    // segment dropped), and `g1_line_id` advances by the full
                    // internal-call count `1 + arc_internal` = `segments`.
                    g1_line_id += 1;
                    block_line_ids
                        .extend(motion_blocks.iter().map(|motion| g1_line_id + motion.index));
                    blocks.extend(motion_blocks.into_iter().map(|motion| motion.block));
                    prepare_stages.extend(std::iter::repeat_n(prepare_stage, count));
                    g1_line_id += arc_internal.unwrap_or(0);
                }
                _ => debug_assert!(motion_blocks.is_empty()),
            }
            saw_motion_command |= is_progress_motion(code);
        }
        let Schedule {
            times,
            trapezoids,
            trailing_delay,
            flush_ends,
            flush_starts,
            pushed,
            processed_blocks,
        } = scheduled_times(&blocks, &events, limits.gcode_flavor);
        let mut stop_times = StopTimes::new(&stop_events, &flush_starts, processed_blocks);
        let scheduled = options_z::Scheduled {
            times: &times,
            trapezoids: &trapezoids,
            pushed: &pushed,
            flush_ends: &flush_ends,
        };
        recorder.apply_schedule(&scheduled, &mut cache_eligible);
        debug_assert_eq!(block_line_ids.len(), times.len());
        // Per-block signature dump (id, cumulative, distance, speed) for
        // sequence alignment against the GT oracle's ORCA_DUMP_TIMES — env
        // gated like the cache dump.
        // `machine.time` accumulates `time += double(block_time)`
        // (`GCodeProcessor.cpp:471`) in f64; `g1_times_cache` then snapshots
        // each entry as `float(time)` (:576). Keep the f64 accumulator for
        // `total`, but round each cache entry to f32 — the M73 boundary
        // reads the f32 snapshot, not the running f64 sum.
        let mut cumulative = 0.0;

        let cache = block_line_ids
            .into_iter()
            .zip(&times)
            .zip(cache_eligible)
            .zip(remaining_internal)
            .enumerate()
            .filter_map(|(block, (((id, time), eligible), remaining))| {
                // calculate_time adds every block's time before testing the
                // associated move vertex for Extrude, Travel or Wipe.
                cumulative += time;
                if eligible {
                    stop_times.record(id, block, cumulative as f32);
                }
                // (g1_line_id, f32 elapsed snapshot)
                eligible.then_some(export_cache::Entry {
                    id,
                    remaining,
                    elapsed: cumulative as f32,
                })
            })
            .collect::<Vec<_>>();
        let export_elapsed = export_cache::snapshots(lines, &cache);
        let elapsed = elapsed_snapshots(lines, &cache);
        // `prepare_time += block_time` accumulates in f32
        // (`GCodeProcessor.cpp:475-476`), so the exported trailer matches
        // upstream's float rounding at the sixth decimal.
        let prepare = times
            .iter()
            .zip(prepare_stages)
            .filter_map(|(time, is_prepare)| is_prepare.then_some(time))
            .fold(0.0_f32, |sum, time| sum + *time as f32) as f64;
        let total = cumulative + trailing_delay;
        let custom_gcode_times =
            custom_time::custom_times(&custom_events, &flush_ends, &times, trailing_delay, total);
        Self {
            total,
            custom_gcode_times,
            stop_times,
            prepare,
            print_statistics: used_filaments.finish(filaments.selection().1),
            printable_area,
            elapsed,
            export_elapsed,
        }
    }
}
