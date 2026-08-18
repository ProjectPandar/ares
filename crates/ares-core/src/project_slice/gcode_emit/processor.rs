use std::f64::consts::PI;

pub(super) fn process(mut output: Vec<u8>) -> Vec<u8> {
    let text = String::from_utf8(std::mem::take(&mut output)).expect("generated G-code is UTF-8");
    let lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::from_lines(&lines);
    let mut result = String::with_capacity(text.len() + text.len() / 100);
    let mut next_percent = 1;
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
            result.push_str(&format!("M73 P0 R{}\n", minutes(estimate.total)));
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

        let elapsed = estimate.elapsed_at(index);
        while next_percent < 100
            && estimate.total > 0.0
            && elapsed >= estimate.total * f64::from(next_percent) / 100.0
        {
            result.push_str(&format!(
                "M73 P{next_percent} R{}\n",
                minutes(estimate.total - elapsed)
            ));
            next_percent += 1;
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
        for (index, line) in lines.iter().enumerate() {
            let code = line.split(';').next().unwrap_or_default().trim();
            if let Some(delay) = command_delay(code) {
                delays[index] += delay;
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

#[derive(Default)]
struct MotionState {
    position: [f64; 3],
    feedrate: f64,
    acceleration: f64,
    relative: bool,
}

struct MotionBlock {
    index: usize,
    distance: f64,
    speed: f64,
    acceleration: f64,
    direction: [f64; 3],
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
        if let Some(value) = word(code, 'F') {
            self.feedrate = value / 60.0;
        }
        if code.starts_with("M204") {
            if let Some(value) = word(code, 'S')
                .or_else(|| word(code, 'P'))
                .or_else(|| word(code, 'T'))
            {
                self.acceleration = value;
            }
            return None;
        }
        let command = code.get(0..2)?;
        if !matches!(command, "G0" | "G1" | "G2" | "G3") || self.feedrate <= 0.0 {
            return None;
        }
        let old = self.position;
        let mut next = old;
        for (axis, letter) in ['X', 'Y', 'Z'].into_iter().enumerate() {
            if let Some(value) = word(code, letter) {
                let offset = match self.relative {
                    true => value,
                    false => value - old[axis],
                };
                next[axis] = old[axis] + offset;
            }
        }
        let mut delta = [next[0] - old[0], next[1] - old[1], next[2] - old[2]];
        let mut distance = norm(delta);
        if matches!(command, "G2" | "G3") {
            let i = word(code, 'I').unwrap_or(0.0);
            let j = word(code, 'J').unwrap_or(0.0);
            let radius = (i * i + j * j).sqrt();
            let start = (-i).atan2(-j);
            let end = (delta[0] - i).atan2(delta[1] - j);
            let mut sweep = end - start;
            if command == "G2" && sweep >= 0.0 {
                sweep -= 2.0 * PI;
            } else if command == "G3" && sweep <= 0.0 {
                sweep += 2.0 * PI;
            }
            distance = (radius * sweep.abs()).hypot(delta[2]);
            delta[0] = sweep.sin() * radius;
            delta[1] = sweep.cos() * radius;
        }
        self.position = next;
        if distance <= f64::EPSILON {
            return None;
        }
        Some(MotionBlock {
            index: 0,
            distance,
            speed: self.feedrate,
            acceleration: self.acceleration.max(1.0),
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
            let dot = block.direction[0] * next.direction[0]
                + block.direction[1] * next.direction[1]
                + block.direction[2] * next.direction[2];
            let sine = ((1.0 - dot.clamp(-1.0, 1.0)) * 0.5).sqrt();
            let junction = (block.acceleration * 0.01 * sine / (1.0 - sine).max(1e-6)).sqrt();
            block.speed.min(next.speed).min(junction)
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

fn command_delay(code: &str) -> Option<f64> {
    if code.starts_with("M400") {
        return Some(word(code, 'S').unwrap_or(0.0) + word(code, 'P').unwrap_or(0.0) * 0.001);
    }
    if code.starts_with("M191") && word(code, 'S').unwrap_or(0.0) > 40.0 {
        return Some(720.0);
    }
    None
}

fn word(code: &str, letter: char) -> Option<f64> {
    let start = code.find(letter)? + letter.len_utf8();
    let value = &code[start..];
    let end = value
        .find(|character: char| character.is_ascii_alphabetic())
        .unwrap_or(value.len());
    value[..end].trim().parse().ok()
}

fn norm(value: [f64; 3]) -> f64 {
    value
        .iter()
        .map(|component| component * component)
        .sum::<f64>()
        .sqrt()
}

fn scale(value: [f64; 3], factor: f64) -> [f64; 3] {
    [value[0] * factor, value[1] * factor, value[2] * factor]
}

#[cfg(test)]
mod tests {
    use super::process;

    #[test]
    fn inserts_progress_and_rewrites_time_fields() {
        let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X1000 F600\nM73 P100 R0\n".to_vec();
        let output = String::from_utf8(process(output)).unwrap();
        assert!(output.contains("total estimated time: 1m 40s"), "{output}");
        assert!(output.contains("M73 P0 R"));
        assert!(output.contains("; model printing time:"));
        assert!(!output.contains("total estimated time: 0s"));
    }
}
