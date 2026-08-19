mod delays;
mod motion;
mod time;
use delays::command_delay;
use motion::{MotionBlock, MotionState, planned_times, word};
use time::{duration, minutes};

pub(super) fn process(
    mut output: Vec<u8>,
    emit_progress: bool,
    machine_load_filament_time: f64,
) -> Vec<u8> {
    let text = String::from_utf8(std::mem::take(&mut output)).expect("generated G-code is UTF-8");
    let lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::from_lines(&lines, machine_load_filament_time);
    let mut result = String::with_capacity(text.len() + text.len() / 100);
    let mut last_progress = None;
    let first_marker = lines
        .iter()
        .position(|line| line == "M73 P0 R0")
        .unwrap_or(0);
    let first_layer_end = lines
        .iter()
        .enumerate()
        .find(|(index, line)| *index > first_marker && line.starts_with("; CHANGE_LAYER"))
        .map(|(index, _)| estimate.elapsed_at(index))
        .unwrap_or(estimate.total);
    let model_time = (estimate.total - first_layer_end).max(0.0);

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
        if line.starts_with("; model printing time:") {
            result.push_str(&format!(
                "; model printing time: {}; total estimated time: {}\n",
                duration(model_time),
                duration(estimate.total),
            ));
            continue;
        }
        if line.starts_with("; estimated first layer printing time") {
            result.push_str(&format!(
                "; estimated first layer printing time (normal mode) = {}\n",
                duration(first_layer_end),
            ));
            continue;
        }
        if index < first_marker {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        if emit_progress {
            let elapsed = estimate.elapsed_at(index);
            let percent = if estimate.total > 0.0 {
                ((elapsed / estimate.total) * 100.0)
                    .floor()
                    .clamp(0.0, 99.0) as u64
            } else {
                0
            };
            let remaining = minutes(estimate.total - elapsed);
            if last_progress != Some((percent, remaining)) {
                last_progress = Some((percent, remaining));
                result.push_str(&format!("M73 P{percent} R{remaining}\n"));
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    output.clear();
    output.extend_from_slice(result.as_bytes());
    output
}

struct Estimate {
    total: f64,
    elapsed: Vec<f64>,
}

impl Estimate {
    fn from_lines(lines: &[String], machine_load_filament_time: f64) -> Self {
        let mut blocks = Vec::new();
        let mut state = MotionState::default();
        let mut delays = vec![0.0; lines.len()];
        let mut flushes = Vec::new();
        let mut measure_g29_time = false;
        let mut active_tool = None;
        for (index, line) in lines.iter().enumerate() {
            let code = line.split(';').next().unwrap_or_default().trim();
            if code.starts_with("M622") && word(code, 'J').unwrap_or(0.0).round() == 1.0 {
                measure_g29_time = true;
            } else if code.starts_with("M623") {
                measure_g29_time = false;
            }
            let is_g29 = code.starts_with("G29") && !code.starts_with("G29.");
            if !is_g29 || measure_g29_time {
                delays[index] += command_delay(code).unwrap_or(0.0);
            }
            if synchronizes_planner(code, is_g29 && measure_g29_time) {
                flushes.push(index);
            }
            if selects_initial_tool(code, &mut active_tool) {
                delays[index] += machine_load_filament_time;
                flushes.push(index);
            }
            blocks.extend(
                state
                    .motions(code)
                    .into_iter()
                    .map(|block| MotionBlock { index, ..block }),
            );
        }
        let mut times = Vec::with_capacity(blocks.len());
        let mut start = 0;
        for line in flushes {
            let end = start + blocks[start..].partition_point(|block| block.index < line);
            times.extend(planned_times(&blocks[start..end]));
            start = end;
        }
        times.extend(planned_times(&blocks[start..]));
        let mut elapsed = vec![0.0; lines.len() + 1];
        let mut block_index = 0;
        for index in 0..lines.len() {
            elapsed[index + 1] = elapsed[index] + delays[index];
            while block_index < blocks.len() && blocks[block_index].index == index {
                elapsed[index + 1] += times[block_index];
                block_index += 1;
            }
        }
        Self {
            total: elapsed[lines.len()],
            elapsed,
        }
    }

    fn elapsed_at(&self, line: usize) -> f64 {
        self.elapsed.get(line).copied().unwrap_or(self.total)
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
