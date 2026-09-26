//! `GCodeProcessor` motion accounting and exported progress-time lookup.

mod seam;
use seam::SeamDetector;

use super::arc_accounting::arc_internal_g1_lines;
use super::delays::command_delay;
use super::motion::{MotionBlock, MotionKind, MotionState};
use super::motion_util::word;
use super::schedule::{DelayTarget, FlushEvent, PendingDelay, scheduled_times};
use super::{ProcessorLimits, is_progress_motion};

pub(super) struct Estimate {
    pub(super) total: f64,
    pub(super) prepare: f64,
    elapsed: Vec<Option<f64>>,
    #[cfg(test)]
    pub(super) block_line_ids: Vec<usize>,
}
impl Estimate {
    pub(super) fn from_lines(
        lines: &[String],
        machine_load_filament_time: f64,
        limits: ProcessorLimits,
    ) -> Self {
        let mut blocks = Vec::new();
        let mut state = MotionState::with_limits(limits);
        let mut prepare_stages = Vec::new();
        let mut events = Vec::new();
        let mut prepare_stage = false;
        let mut saw_motion_command = false;
        let mut measure_g29_time = false;
        let mut active_tool = None;
        let mut g1_line_id = 0;
        let mut block_line_ids = Vec::new();
        let mut cache_eligible = Vec::new();
        let mut seams = SeamDetector::default();
        // Per-source-line arc bookkeeping for the M73 cache lookup: the
        // geometric internal-segment count and whether the arc's first block
        // was seam-excluded (`ExportLines::update` → `internal` = 0).
        let mut arc_segment_counts = vec![0usize; lines.len()];
        let mut arc_seam_excluded = vec![false; lines.len()];
        for (index, line) in lines.iter().enumerate() {
            seams.role(line);
            match line.trim() {
                "; WIPE_START" | ";WIPE_START" => state.set_wiping(true),
                "; WIPE_END" | ";WIPE_END" => state.set_wiping(false),
                "; FEATURE: Custom" | ";TYPE:Custom" => prepare_stage = !saw_motion_command,
                line if line.starts_with("; FEATURE:") || line.starts_with(";TYPE:") => {
                    prepare_stage = false
                }
                _ => {}
            }
            let code = line.split(';').next().unwrap_or_default().trim();
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
                    delay: delay.map(|seconds| PendingDelay {
                        target: DelayTarget::Any,
                        seconds: seconds as f32,
                    }),
                });
            }
            if selects_initial_tool(code, &mut active_tool) {
                blocks.push(MotionBlock {
                    distance: 0.0,
                    delta: [0.0; 4],
                    speed: 0.0,
                    acceleration: 0.0,
                    centripetal_acceleration: 0.0,
                    max_feedrate: [0.0; 4],
                    max_acceleration: [0.0; 4],
                    jerk: [0.0; 4],
                    extrude_factor: 1.0,
                    // The tool-change block lands no `g1_times_cache` entry
                    // (verified against the `ORCA_DUMP_TIMES` dump: the
                    // initial `T0`'s load delay reaches the machine total via
                    // blocks that the M73 lookup never sees), so M73 emission
                    // never attaches to the tool-change line.
                    e_only: true,
                    kind: MotionKind::ToolChange,
                });
                block_line_ids.push(g1_line_id);
                cache_eligible.push(false);
                prepare_stages.push(prepare_stage);
                events.push(FlushEvent {
                    block_count: blocks.len(),
                    delay: Some(PendingDelay {
                        target: DelayTarget::ToolChange,
                        seconds: machine_load_filament_time as f32,
                    }),
                });
            }
            let command = code.split_whitespace().next().unwrap_or_default();
            let arc_internal = matches!(command, "G2" | "G3")
                .then(|| arc_internal_g1_lines(code, command, &state));
            let arc_start = state.position;
            let mut seam_excluded = false;
            let motion_blocks = state.motions_indexed(code);
            // Linear commands produce at most one block; only arcs subdivide.
            if matches!(command, "G0" | "G1" | "G28" | "G10" | "G11")
                && let Some((_, block)) = motion_blocks.first()
            {
                let extruding = !state.wiping
                    && block.delta[3] > 0.0
                    && (block.delta[0] != 0.0 || block.delta[1] != 0.0);
                let seam_vertex = seams.associate(extruding, state.position);
                // `GCodeProcessor.cpp:574` pushes a `g1_times_cache` entry
                // for EVERY block — E-only retract/deretract moves included
                // — so the `it->id == g1_lines_counter` probe can land on an
                // E-only line and emit M73 immediately after it. Excluding
                // `e_only` blocks deferred the marker to the next XY move
                // (observed: oracle `M73` follows `G1 E-2.8`).
                // `process_G10/G11` reuses the already-assigned g1 line id
                // (`--m_g1_line_id`, `GCodeProcessor.cpp:4879/4889`) and the
                // resulting Retract/Unretract vertex is not a
                // `g1_times_cache` type, so the phantom entry must stay
                // out — otherwise it collides with the next real G1's id.
                cache_eligible.push(matches!(command, "G0" | "G1" | "G28") && !seam_vertex);
            } else if !motion_blocks.is_empty() {
                // `G2`/`G3` arc: every discretized internal segment is a
                // `process_G1` → `store_move_vertex`, so the seam lifecycle
                // runs PER SEGMENT (`GCodeProcessor.cpp:4194-4225`), not once
                // at the arc level. A segment whose start returns to
                // `first_vertex` while terminating the loop associates a
                // `Seam` vertex with its block → cache-excluded (:484-488),
                // collapsing the `remaining` chain `ExportLines::update`
                // walks (:790-800). `arc_start` is the position before
                // `motions` advanced `state.position` to the arc end.
                let mut position = arc_start;
                for (segment_index, (_, block)) in motion_blocks.iter().enumerate() {
                    let end = std::array::from_fn::<_, 3, _>(|i| position[i] + block.delta[i]);
                    let extruding = !state.wiping
                        && block.delta[3] > 0.0
                        && (block.delta[0] != 0.0 || block.delta[1] != 0.0);
                    let seam_vertex = seams.associate(extruding, end);
                    seam_excluded |= seam_vertex && segment_index == 0;
                    cache_eligible.push(!seam_vertex);
                    position = end;
                }
            }
            match command {
                "G0" | "G1" | "G28" => {
                    g1_line_id += 1;
                    let count = motion_blocks.len();
                    blocks.extend(motion_blocks.into_iter().map(|(_, block)| block));
                    block_line_ids.extend(std::iter::repeat_n(g1_line_id, count));
                    prepare_stages.extend(std::iter::repeat_n(prepare_stage, count));
                }
                // `GCodeProcessor::process_G10/G11` decrements the fake
                // G1's line id (`GCodeProcessor.cpp:4873-4893`), so a
                // firmware retract occupies NO g1 line id — the M73 cache
                // alignment skips over it.
                "G10" | "G11" => {
                    let count = motion_blocks.len();
                    blocks.extend(motion_blocks.into_iter().map(|(_, block)| block));
                    block_line_ids.extend(std::iter::repeat_n(g1_line_id, count));
                    prepare_stages.extend(std::iter::repeat_n(prepare_stage, count));
                }
                "G2" | "G3" => {
                    let count = motion_blocks.len();
                    arc_segment_counts[index] = arc_internal.unwrap_or(0);
                    arc_seam_excluded[index] = seam_excluded;
                    // Every discretized internal `process_G1` consumes its own
                    // g1 line id (`GCodeProcessor.cpp:3868` `++m_g1_line_id`),
                    // including the zero-displacement segments dropped at
                    // :3891 — so a retained block carries
                    // `base + discretization_index` (a hole opens where a
                    // segment dropped), and `g1_line_id` advances by the full
                    // internal-call count `1 + arc_internal` = `segments`.
                    g1_line_id += 1;
                    block_line_ids
                        .extend(motion_blocks.iter().map(|(index, _)| g1_line_id + index));
                    blocks.extend(motion_blocks.into_iter().map(|(_, block)| block));
                    prepare_stages.extend(std::iter::repeat_n(prepare_stage, count));
                    g1_line_id += arc_internal.unwrap_or(0);
                }
                _ => debug_assert!(motion_blocks.is_empty()),
            }
            saw_motion_command |= is_progress_motion(code);
        }
        let (times, trailing_delay) = scheduled_times(&blocks, &events);
        debug_assert_eq!(block_line_ids.len(), times.len());
        // Per-block signature dump (id, cumulative, distance, speed) for
        // sequence alignment against the GT oracle's ORCA_DUMP_TIMES — env
        // gated like the cache dump.
        if let Ok(path) = std::env::var("ARES_DUMP_BLOCKS") {
            use std::io::Write;
            let mut out = std::fs::File::create(&path).unwrap();
            let mut running = 0.0;
            for ((block, id), time) in blocks.iter().zip(&block_line_ids).zip(&times) {
                running += time;
                writeln!(
                    out,
                    "{id} {running:.6} {:.6} {:.3} {} {:.4} {:.4} {:.4}",
                    block.distance,
                    block.speed,
                    match block.kind {
                        MotionKind::Regular => "reg",
                        MotionKind::Unretract => "unretract",
                        MotionKind::ToolChange => "tool",
                    },
                    block.delta[0],
                    block.delta[1],
                    block.delta[2]
                )
                .unwrap();
            }
        }
        // `machine.time` accumulates `time += double(block_time)`
        // (`GCodeProcessor.cpp:471`) in f64; `g1_times_cache` then snapshots
        // each entry as `float(time)` (:576). Keep the f64 accumulator for
        // `total`, but round each cache entry to f32 — the M73 boundary
        // reads the f32 snapshot, not the running f64 sum.
        let mut cumulative = 0.0;
        #[cfg(test)]
        let block_line_ids_snapshot = block_line_ids.clone();
        let cache = block_line_ids
            .into_iter()
            .zip(&times)
            .zip(cache_eligible)
            .filter_map(|((id, time), eligible)| {
                // calculate_time adds every block's time before testing the
                // associated move vertex for Extrude, Travel or Wipe.
                cumulative += time;
                // (g1_line_id, f32 elapsed snapshot)
                eligible.then_some((id, f64::from(cumulative as f32)))
            })
            .collect::<Vec<_>>();
        let mut elapsed = vec![None; lines.len() + 1];
        let mut cache_index = 0;
        let mut exported_g1_lines = 0;
        for (index, line) in lines.iter().enumerate() {
            let command = line
                .split(';')
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .next()
                .unwrap_or_default();
            let lookup_id = match command {
                "G0" | "G1" => {
                    // `process_line_move(g1_lines_counter++)` (:1460) probes
                    // `g1_times_cache_it->id == g1_lines_counter` at the
                    // PRE-increment caller counter (:1246-1248) — the line
                    // reads the cumulative through the previous eligible
                    // cache entry.
                    let id = exported_g1_lines;
                    exported_g1_lines += 1;
                    Some(id)
                }
                "G2" | "G3" => {
                    // `ret` = `internal_g1_lines_counter` = the
                    // `remaining_internal_g1_lines > 0` walk count in
                    // `ExportLines::update` (:793-799): `arc_internal - 1`
                    // eligible internal entries for a fully-cached arc
                    // (the `it->id == counter` probe lands on the
                    // second-to-last internal segment), 0 when the first
                    // block was seam-excluded (`it->id > counter`).
                    // `process_line_move(g1_lines_counter + ret)` (:1466),
                    // `g1_lines_counter += 1 + ret` (:1467). The `it->id`
                    // probe is `exported + 1 + ret` (`++` at :778).
                    let internal = if arc_seam_excluded[index] {
                        0
                    } else {
                        arc_segment_counts[index]
                    };
                    // `process_line_move(g1_lines_counter + internal)` (:1466)
                    // probes `g1_times_cache_it->id == g1_lines_counter +
                    // internal` (:1246-1248) — `internal` is `update`'s
                    // `ret` = the `remaining>0` walk count = `arc_internal`
                    // for a fully-cached arc, 0 when the first block was
                    // seam-excluded. `g1_lines_counter += 1 + internal` (:1467).
                    let lookup = exported_g1_lines + internal;
                    exported_g1_lines += 1 + internal;
                    Some(lookup)
                }
                "G28" => {
                    exported_g1_lines += 1;
                    None
                }
                _ => None,
            };
            let Some(lookup_id) = lookup_id else {
                continue;
            };
            while cache_index < cache.len() && cache[cache_index].0 < lookup_id {
                cache_index += 1;
            }
            if cache
                .get(cache_index)
                .is_some_and(|(id, _)| *id == lookup_id)
            {
                elapsed[index + 1] = Some(cache[cache_index].1);
            }
        }
        // Diagnostic backing for the committed `m73_profile` parity harness
        // (`ares-cli/tests/orca_parity/m73_profile.rs`): dumps the
        // `g1_times_cache` pairs `<id> <cumulative-seconds>`. No effect on
        // slicing unless `ARES_DUMP_ELAPSED` names an output path.
        if let Ok(path) = std::env::var("ARES_DUMP_ELAPSED") {
            use std::io::Write;
            let mut out = std::fs::File::create(&path).unwrap();
            for (id, cumulative) in &cache {
                writeln!(out, "{id} {cumulative:.6}").unwrap();
            }
        }
        // `prepare_time += block_time` accumulates in f32
        // (`GCodeProcessor.cpp:475-476`), so the exported trailer matches
        // upstream's float rounding at the sixth decimal.
        let prepare = times
            .iter()
            .zip(prepare_stages)
            .filter_map(|(time, is_prepare)| is_prepare.then_some(time))
            .fold(0.0_f32, |sum, time| sum + *time as f32) as f64;
        if std::env::var("ARES_DUMP_TOTAL").is_ok() {
            eprintln!(
                "ARES_TOTAL cumulative={cumulative:.9} trailing_delay={trailing_delay:.9} total={:.9} prepare={prepare}",
                cumulative + trailing_delay
            );
        }
        Self {
            total: cumulative + trailing_delay,
            prepare,
            elapsed,
            #[cfg(test)]
            block_line_ids: block_line_ids_snapshot,
        }
    }

    pub(super) fn elapsed_at(&self, line: usize) -> Option<f64> {
        self.elapsed.get(line).copied().flatten()
    }
}

fn synchronizes_planner(code: &str, measured_g29: bool) -> bool {
    let command = code.split_whitespace().next().unwrap_or_default();
    matches!(command, "M0" | "M1")
        || matches!(command, "G4" | "M400")
            && (word(code, 'S').is_some() || word(code, 'P').is_some())
        || measured_g29
        || command == "M191" && word(code, 'S').unwrap_or(0.0) > 40.0
        || command == "G92" && word(code, 'E').is_none()
        || command == "M702" && code.contains('C')
        || command == "SYNC" && word(code, 'T').is_some()
}

fn tool_id(code: &str) -> Option<u8> {
    let command = code.split_whitespace().next()?;
    command.strip_prefix('T')?.parse().ok()
}

fn selects_initial_tool(code: &str, active_tool: &mut Option<u8>) -> bool {
    let Some(tool) = tool_id(code) else {
        return false;
    };
    active_tool.replace(tool).is_none()
}
