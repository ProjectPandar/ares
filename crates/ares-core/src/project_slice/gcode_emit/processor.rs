use std::f64::consts::PI;
mod delays;
use delays::command_delay;

pub(super) fn process(mut output: Vec<u8>) -> Vec<u8> {
    let text = String::from_utf8(std::mem::take(&mut output)).expect("generated G-code is UTF-8");
    let lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::from_lines(&lines);
    let mut result = String::with_capacity(text.len() + text.len() / 100);
    let mut last_progress = None;
    let first_marker = lines
        .iter()
        .position(|line| line == "M73 P0 R0")
        .unwrap_or(0);
    let first_layer_end = lines
        .iter()
        .enumerate()
        .filter(|(index, line)| *index > first_marker && line.starts_with("; CHANGE_LAYER"))
        .nth(1)
        .map(|(index, _)| estimate.elapsed_at(index))
        .unwrap_or(estimate.total);
    let model_time = (estimate.total - first_layer_end).max(0.0);

    for (index, line) in lines.iter().enumerate() {
        if line == "M73 P0 R0" {
            let remaining = minutes(estimate.total);
            last_progress = Some((0, remaining));
            result.push_str(&format!("M73 P0 R{remaining}\n"));
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
        result.push_str(line);
        result.push('\n');
    }
    output.clear();
    output.extend_from_slice(result.as_bytes());
    output
}

fn minutes(seconds: f64) -> u64 {
    (seconds / 60.0).floor() as u64
}

fn duration(seconds: f64) -> String {
    let mut remaining = seconds.round() as u64;
    let hours = remaining / 3600;
    remaining %= 3600;
    let minutes = remaining / 60;
    let seconds = remaining % 60;
    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

struct Estimate {
    total: f64,
    elapsed: Vec<f64>,
}

impl Estimate {
    fn from_lines(lines: &[String]) -> Self {
        let mut blocks = Vec::new();
        let mut state = MotionState::default();
        let mut delays = vec![0.0; lines.len()];
        let mut measure_g29_time = false;
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
            if let Some(block) = state.motion(code) {
                blocks.push(MotionBlock { index, ..block });
            }
        }
        let times = planned_times(&blocks);
        let mut elapsed = vec![0.0; lines.len() + 1];
        let mut block_index = 0;
        for index in 0..lines.len() {
            elapsed[index + 1] = elapsed[index] + delays[index];
            if block_index < blocks.len() && blocks[block_index].index == index {
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

struct MotionState {
    position: [f64; 3],
    e_position: f64,
    feedrate: f64,
    acceleration: f64,
    retract_acceleration: f64,
    travel_acceleration: f64,
    jerk: [f64; 4],
    relative: bool,
    e_relative: bool,
}

impl Default for MotionState {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            e_position: 0.0,
            feedrate: 0.0,
            acceleration: 0.0,
            retract_acceleration: 0.0,
            travel_acceleration: 0.0,
            jerk: [9.0, 9.0, 3.0, 2.5],
            relative: false,
            e_relative: false,
        }
    }
}

struct MotionBlock {
    index: usize,
    distance: f64,
    speed: f64,
    acceleration: f64,
    jerk: [f64; 4],
    direction: [f64; 4],
}

impl MotionState {
    fn motion(&mut self, code: &str) -> Option<MotionBlock> {
        if code == "G90" {
            self.relative = false;
            return None;
        }
        if code == "G91" {
            self.relative = true;
            return None;
        }
        if code == "M82" {
            self.e_relative = false;
            return None;
        }
        if code == "M83" {
            self.e_relative = true;
            return None;
        }
        if code.starts_with("G92") {
            if let Some(value) = word(code, 'E') {
                self.e_position = value;
            }
            for (axis, letter) in ['X', 'Y', 'Z'].into_iter().enumerate() {
                self.position[axis] = word(code, letter).unwrap_or(self.position[axis]);
            }
            return None;
        }
        if let Some(value) = word(code, 'F') {
            self.feedrate = value / 60.0;
        }
        if code.starts_with("M204") {
            if let Some(value) = word(code, 'S') {
                self.acceleration = value;
                self.travel_acceleration = value;
            }
            self.acceleration = word(code, 'P').unwrap_or(self.acceleration);
            self.retract_acceleration = word(code, 'R').unwrap_or(self.retract_acceleration);
            self.travel_acceleration = word(code, 'T').unwrap_or(self.travel_acceleration);
            return None;
        }
        if code.starts_with("M205") {
            for (axis, letter) in ['X', 'Y', 'Z', 'E'].into_iter().enumerate() {
                self.jerk[axis] = word(code, letter).unwrap_or(self.jerk[axis]);
            }
            return None;
        }
        let command = code.split_whitespace().next()?;
        if !matches!(command, "G0" | "G1" | "G2" | "G3") || self.feedrate <= 0.0 {
            return None;
        }
        let old = self.position;
        let mut next = old;
        for (axis, letter) in ['X', 'Y', 'Z'].into_iter().enumerate() {
            let Some(value) = word(code, letter) else {
                continue;
            };
            let offset = match self.relative {
                true => value,
                false => value - old[axis],
            };
            next[axis] = old[axis] + offset;
        }
        let old_e = self.e_position;
        let e_delta = word(code, 'E').map_or(0.0, |value| {
            if self.e_relative {
                value
            } else {
                value - old_e
            }
        });
        self.e_position = old_e + e_delta;
        let mut delta = [
            next[0] - old[0],
            next[1] - old[1],
            next[2] - old[2],
            e_delta,
        ];
        let xyz_distance = norm([delta[0], delta[1], delta[2], 0.0]);
        let e_only = xyz_distance <= f64::EPSILON;
        if !e_only {
            delta[3] = 0.0;
        }
        let mut distance = if e_only { e_delta.abs() } else { xyz_distance };
        if matches!(command, "G2" | "G3") {
            let i = word(code, 'I').unwrap_or(0.0);
            let j = word(code, 'J').unwrap_or(0.0);
            let radius = (i * i + j * j).sqrt();
            let same_xy = delta[0].abs() <= f64::EPSILON && delta[1].abs() <= f64::EPSILON;
            let mut sweep = if same_xy {
                0.0
            } else {
                let start = (-j).atan2(-i);
                let end = (delta[1] - j).atan2(delta[0] - i);
                let mut sweep = end - start;
                sweep = match (command, sweep >= 0.0, sweep <= 0.0) {
                    ("G2", true, _) => sweep - 2.0 * PI,
                    ("G3", _, true) => sweep + 2.0 * PI,
                    _ => sweep,
                };
                sweep
            };
            let turns = word(code, 'P').unwrap_or(0.0);
            let turns = if same_xy && turns == 0.0 { 1.0 } else { turns } * 2.0 * PI;
            sweep += if command == "G2" { -turns } else { turns };
            distance = (radius * sweep.abs()).hypot(delta[2]).hypot(delta[3]);
            delta[0] = sweep.cos() * radius;
            delta[1] = sweep.sin() * radius;
        }
        self.position = next;
        if distance <= f64::EPSILON {
            return None;
        }
        let acceleration = if e_delta < 0.0 {
            self.retract_acceleration
        } else if e_delta > 0.0 {
            self.acceleration
        } else {
            self.travel_acceleration
        };
        Some(MotionBlock {
            index: 0,
            distance,
            speed: self.feedrate,
            acceleration: acceleration.max(1.0),
            jerk: self.jerk,
            direction: scale(delta, 1.0 / distance),
        })
    }
}

fn planned_times(blocks: &[MotionBlock]) -> Vec<f64> {
    if blocks.is_empty() {
        return Vec::new();
    }
    let mut entry = vec![0.0; blocks.len()];
    let mut max_entry = vec![0.0; blocks.len()];
    for (index, block) in blocks.iter().enumerate() {
        max_entry[index] = if let Some(next) = blocks.get(index + 1) {
            let junction = block.speed.min(next.speed);
            (0..4).fold(junction, |limit, axis| {
                let delta =
                    (next.speed * next.direction[axis] - block.speed * block.direction[axis]).abs();
                let factor = (block.jerk[axis] / delta.max(block.jerk[axis])).min(1.0);
                limit.min(junction * factor)
            })
        } else {
            0.0
        };
    }
    for index in 1..blocks.len() {
        let previous = &blocks[index - 1];
        entry[index] = max_entry[index].min(
            (entry[index - 1] * entry[index - 1] + 2.0 * previous.acceleration * previous.distance)
                .sqrt(),
        );
    }
    for index in (0..blocks.len() - 1).rev() {
        let block = &blocks[index];
        entry[index] = entry[index].min(
            (entry[index + 1] * entry[index + 1] + 2.0 * block.acceleration * block.distance)
                .sqrt(),
        );
    }
    blocks
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let start = entry[index];
            let end = entry.get(index + 1).copied().unwrap_or(0.0);
            trapezoid_time(block.distance, start, block.speed, end, block.acceleration)
        })
        .collect()
}

fn trapezoid_time(distance: f64, start: f64, cruise: f64, end: f64, acceleration: f64) -> f64 {
    let accelerate = ((cruise * cruise - start * start) / (2.0 * acceleration)).max(0.0);
    let decelerate = ((cruise * cruise - end * end) / (2.0 * acceleration)).max(0.0);
    let cruise_distance = distance - accelerate - decelerate;
    if cruise_distance >= 0.0 {
        (cruise - start) / acceleration
            + cruise_distance / cruise.max(f64::MIN_POSITIVE)
            + (cruise - end) / acceleration
    } else {
        let peak = ((2.0 * acceleration * distance + start * start + end * end) * 0.5).sqrt();
        (peak - start) / acceleration + (peak - end) / acceleration
    }
}

fn word(code: &str, letter: char) -> Option<f64> {
    let start = code.find(letter)? + letter.len_utf8();
    let value = &code[start..];
    let end = value
        .find(|character: char| character.is_ascii_alphabetic())
        .unwrap_or(value.len());
    value[..end].trim().parse().ok()
}

fn norm(value: [f64; 4]) -> f64 {
    value
        .iter()
        .map(|component| component * component)
        .sum::<f64>()
        .sqrt()
}

fn scale(value: [f64; 4], factor: f64) -> [f64; 4] {
    [
        value[0] * factor,
        value[1] * factor,
        value[2] * factor,
        value[3] * factor,
    ]
}
#[cfg(test)]
mod tests;
