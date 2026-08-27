mod delays;
mod motion;
mod motion_util;
mod time;
use crate::options::GCodeFlavor;
use delays::command_delay;
#[cfg(test)]
use motion::planned_times;
use motion::{MotionBlock, MotionKind, MotionState, RollingPlanner};
use motion_util::word;
use std::collections::VecDeque;
use time::{duration, minutes};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ProcessorLimits {
    pub(super) print_acceleration: f64,
    pub(super) retract_acceleration: f64,
    pub(super) travel_acceleration: f64,
    pub(super) gcode_flavor: GCodeFlavor,
    pub(super) bbl_printer: bool,
}

pub(super) fn process(
    mut output: Vec<u8>,
    emit_progress: bool,
    machine_load_filament_time: f64,
    limits: ProcessorLimits,
) -> Vec<u8> {
    let text = String::from_utf8(std::mem::take(&mut output)).expect("generated G-code is UTF-8");
    let lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::from_lines(&lines, machine_load_filament_time, limits);
    let mut result = String::with_capacity(text.len() + text.len() / 100);
    let mut last_progress = None;
    let first_marker = lines
        .iter()
        .position(|line| line == "M73 P0 R0")
        .unwrap_or(0);
    let prepare_time = estimate.prepare;
    let model_time = (estimate.total - prepare_time).max(0.0);

    for (index, line) in lines.iter().enumerate() {
        if line == "M73 P0 R0" {
            if emit_progress {
                let remaining = minutes(estimate.total);
                last_progress = Some((0, remaining));
                result.push_str(&format!("M73 P0 R{remaining}\n"));
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
                result.push_str(&format!(
                    "; model printing time: {}; total estimated time: {}\n",
                    duration(model_time),
                    duration(estimate.total),
                ));
            } else {
                result.push_str(&format!(
                    "; estimated printing time (normal mode) = {}\n",
                    duration(estimate.total),
                ));
            }
            continue;
        }
        if line.starts_with("; estimated first layer printing time") {
            result.push_str(&format!(
                "; estimated first layer printing time (normal mode) = {}\n",
                duration(prepare_time),
            ));
            continue;
        }
        result.push_str(line);
        result.push('\n');
        if index < first_marker || !emit_progress || !is_progress_motion(line) {
            continue;
        }
        let Some(elapsed) = estimate.elapsed_at(index + 1).map(|elapsed| elapsed as f32) else {
            continue;
        };
        let percent = if estimate.total > 0.0 {
            (f64::from(100.0_f32 * elapsed) / estimate.total).clamp(0.0, 99.0) as u64
        } else {
            0
        };
        let remaining = minutes(estimate.total - f64::from(elapsed));
        if last_progress != Some((percent, remaining)) {
            last_progress = Some((percent, remaining));
            result.push_str(&format!("M73 P{percent} R{remaining}\n"));
        }
    }
    output.clear();
    output.extend_from_slice(result.as_bytes());
    output
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

struct Estimate {
    total: f64,
    prepare: f64,
    elapsed: Vec<Option<f64>>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum DelayTarget {
    Any,
    ToolChange,
}

#[derive(Clone, Copy)]
struct PendingDelay {
    target: DelayTarget,
    seconds: f32,
}

struct FlushEvent {
    block_count: usize,
    delay: Option<PendingDelay>,
}

impl Estimate {
    fn from_lines(
        lines: &[String],
        machine_load_filament_time: f64,
        limits: ProcessorLimits,
    ) -> Self {
        let mut blocks = Vec::new();
        let travel_limit = if limits.gcode_flavor.supports_separate_travel_acceleration() {
            limits.travel_acceleration
        } else {
            0.0
        };
        let mut state = MotionState::with_acceleration_limits(
            limits.print_acceleration,
            limits.retract_acceleration,
            travel_limit,
        );
        let mut prepare_stages = Vec::new();
        state.gcode_flavor = limits.gcode_flavor;
        let mut events = Vec::new();
        let mut prepare_stage = false;
        let mut saw_motion_command = false;
        let mut measure_g29_time = false;
        let mut active_tool = None;
        let mut g1_line_id = 0;
        let mut block_line_ids = Vec::new();
        let mut arc_segment_counts = vec![0; lines.len()];
        for (index, line) in lines.iter().enumerate() {
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
            let delay = (!is_g29 || measure_g29_time)
                .then(|| command_delay(code))
                .flatten();
            if synchronizes_planner(code, is_g29 && measure_g29_time) {
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
                    speed: 0.0,
                    acceleration: 0.0,
                    centripetal_acceleration: 0.0,
                    jerk: [0.0; 4],
                    direction: [0.0; 4],
                    kind: MotionKind::ToolChange,
                });
                block_line_ids.push(g1_line_id);
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
            let motion_blocks = state.motions(code);
            match command {
                "G0" | "G1" | "G28" => {
                    g1_line_id += 1;
                    let count = motion_blocks.len();
                    blocks.extend(motion_blocks);
                    block_line_ids.extend(std::iter::repeat_n(g1_line_id, count));
                    prepare_stages.extend(std::iter::repeat_n(prepare_stage, count));
                }
                "G2" | "G3" => {
                    let count = motion_blocks.len();
                    arc_segment_counts[index] = count;
                    blocks.extend(motion_blocks);
                    block_line_ids.extend((0..count).map(|offset| g1_line_id + offset + 1));
                    prepare_stages.extend(std::iter::repeat_n(prepare_stage, count));
                    g1_line_id += count;
                }
                _ => debug_assert!(motion_blocks.is_empty()),
            }
            saw_motion_command |= is_progress_motion(code);
        }
        let (times, trailing_delay) = scheduled_times(&blocks, &events);
        debug_assert_eq!(block_line_ids.len(), times.len());
        let mut cumulative = 0.0;
        let cache = block_line_ids
            .into_iter()
            .zip(&times)
            .map(|(id, time)| {
                cumulative += time;
                (id, cumulative)
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
                    let id = exported_g1_lines;
                    exported_g1_lines += 1;
                    Some(id)
                }
                "G2" | "G3" => {
                    let internal = arc_segment_counts[index].saturating_sub(1);
                    let id = exported_g1_lines + internal;
                    exported_g1_lines += internal + 1;
                    Some(id)
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
        let prepare = times
            .iter()
            .zip(prepare_stages)
            .filter_map(|(time, is_prepare)| is_prepare.then_some(time))
            .sum();
        Self {
            total: cumulative + trailing_delay,
            prepare,
            elapsed,
        }
    }

    fn elapsed_at(&self, line: usize) -> Option<f64> {
        self.elapsed.get(line).copied().flatten()
    }
}

fn scheduled_times(blocks: &[MotionBlock], events: &[FlushEvent]) -> (Vec<f64>, f64) {
    let mut planner = RollingPlanner::new();
    let mut times = vec![0.0; blocks.len()];
    let mut pending = VecDeque::new();
    let mut pushed = 0;
    let mut emitted = 0;

    for event in events {
        while pushed < event.block_count {
            let batch = planner.push(&blocks[pushed]);
            pushed += 1;
            record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);
        }
        if let Some(delay) = event.delay.filter(|delay| delay.seconds > 0.0) {
            if let Some(last) = pending
                .back_mut()
                .filter(|last| last.target == delay.target)
            {
                last.seconds += delay.seconds;
            } else {
                pending.push_back(delay);
            }
        }
        let batch = planner.flush();
        record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);
    }
    while pushed < blocks.len() {
        let batch = planner.push(&blocks[pushed]);
        pushed += 1;
        record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);
    }
    let batch = planner.finish();
    record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);

    let trailing_delay = pending
        .into_iter()
        .fold(0.0_f32, |total, delay| total + delay.seconds);
    (times, f64::from(trailing_delay))
}

fn record_batch(
    blocks: &[MotionBlock],
    times: &mut [f64],
    emitted: &mut usize,
    mut batch: Vec<f64>,
    pending: &mut VecDeque<PendingDelay>,
) {
    for (block, time) in blocks[*emitted..].iter().zip(&mut batch) {
        let Some(delay) = pending.front() else {
            break;
        };
        if delay.target == DelayTarget::Any
            || delay.target == DelayTarget::ToolChange && block.kind == MotionKind::ToolChange
        {
            *time = f64::from(*time as f32 + delay.seconds);
            pending.pop_front();
        }
    }
    let end = *emitted + batch.len();
    times[*emitted..end].copy_from_slice(&batch);
    *emitted = end;
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

#[cfg(test)]
mod tests;

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
