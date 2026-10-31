mod smoothing;

use crate::Point2;
use smoothing::{SmoothRequest, smooth_line};

const SOURCE_EPSILON_MM: f64 = 1e-4;
const PART_FAN_MARKER: &str = ";__ARES_PART_FAN_STATE__";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProjectSpiralVaseConfig {
    pub(crate) enabled: bool,
    pub(crate) smooth_xy: bool,
    pub(crate) max_xy_smoothing: f64,
    pub(crate) starting_flow_ratio: f64,
    pub(crate) finishing_flow_ratio: f64,
    pub(crate) resolution: f64,
    pub(crate) relative_e: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProjectSpiralVaseLayer {
    pub(crate) start: usize,
    pub(crate) enabled: bool,
    pub(crate) final_layer: bool,
    pub(crate) z: f64,
    pub(crate) height: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ReaderState {
    x: f64,
    y: f64,
    e: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProjectSpiralVaseRunState {
    config: ProjectSpiralVaseConfig,
    reader: ReaderState,
    previous_layer: Vec<Point2>,
    vase_enabled: bool,
}

impl ProjectSpiralVaseRunState {
    pub(crate) fn new(config: ProjectSpiralVaseConfig) -> Self {
        Self {
            config,
            reader: ReaderState::default(),
            previous_layer: Vec::new(),
            vase_enabled: false,
        }
    }

    /// Ports the serial vase filter boundary at `GCode.cpp:3693-3705`, before
    /// cooling, and `GCode/SpiralVase.cpp:66-216`'s full-layer transform.
    pub(crate) fn process_layer(&mut self, output: &mut Vec<u8>, layer: ProjectSpiralVaseLayer) {
        let input = std::str::from_utf8(&output[layer.start..])
            .expect("generated G-code is UTF-8")
            .to_owned();
        let enabled = self.config.enabled && layer.enabled;
        if !enabled {
            self.vase_enabled = false;
            self.observe(&input);
            return;
        }

        let transition_in = !self.vase_enabled && self.config.relative_e;
        self.vase_enabled = true;
        let base_z = f64::from(layer.z as f32 - layer.height as f32);
        let total_length = total_extrusion_length(&input, &self.reader, self.config.relative_e);
        let minimum_segment_length = SOURCE_EPSILON_MM.max(2.0 * self.config.resolution);
        let transition_out = layer.final_layer && self.config.relative_e;
        let smooth = self.config.smooth_xy && self.config.max_xy_smoothing > 0.0;
        let mut length = 0.0_f32;
        let mut current_layer = Vec::new();
        let mut last_emitted = self.previous_layer.last().copied();
        let mut transformed = String::new();
        let mut transition = String::new();

        for line in input.lines() {
            let parsed = ParsedLine::parse(line);
            let movement = Movement::from(&self.reader, &parsed, self.config.relative_e);
            self.reader.apply(&parsed, self.config.relative_e);

            if parsed.command != Some(Command::G1) {
                push_passthrough(&mut transformed, &mut transition, line, transition_out);
                continue;
            }
            if movement.retracting
                || (movement.extruding && movement.xy_distance < minimum_segment_length)
            {
                continue;
            }
            if parsed.z.is_some() && !parsed.has_xy() {
                // `SpiralVase.cpp:136-141`: every remaining Z-only G1 is kept
                // and rewritten to the layer base z (`line.set(Z, z)`).
                push_line(&mut transformed, &set_word(line, 'Z', format_z(base_z)));
                continue;
            }
            if !parsed.has_xy() {
                push_passthrough(&mut transformed, &mut transition, line, transition_out);
                continue;
            }
            // `SpiralVase.cpp:192-200`: omit XY travels so consecutive loops
            // blend at their seam.
            if !movement.extruding || movement.xy_distance <= 0.0 {
                continue;
            }

            length += movement.xy_distance as f32;
            let progress = if total_length > f32::EPSILON {
                f64::from((length / total_length).clamp(0.0, 1.0))
            } else {
                1.0
            };
            let mut normal = line.to_owned();
            if transition_in && let Some(e) = parsed.e {
                let starting = self.config.starting_flow_ratio as f32;
                let factor = starting + progress as f32 * (1.0 - starting);
                normal = set_word(&normal, 'E', format_e(f64::from(e as f32 * factor)));
            } else if transition_out {
                let finishing = self.config.finishing_flow_ratio as f32;
                let factor = finishing + (1.0 - progress as f32) * (1.0 - finishing);
                let tapered = parsed.e.map_or_else(
                    || line.to_owned(),
                    |e| set_word(line, 'E', format_e(f64::from(e as f32 * factor))),
                );
                push_line(&mut transition, &tapered);
            }
            let target_z = f64::from(base_z as f32 + progress as f32 * layer.height as f32);
            normal = set_word(&normal, 'Z', format_z(target_z));
            if smooth {
                normal = match smooth_line(
                    SmoothRequest {
                        normal,
                        movement,
                        progress,
                        previous_layer: &self.previous_layer,
                        maximum_distance: self.config.max_xy_smoothing,
                        minimum_segment_length,
                    },
                    &mut last_emitted,
                    &mut current_layer,
                ) {
                    Some(normal) => normal,
                    None => continue,
                };
            }
            push_line(&mut transformed, &normal);
        }

        self.previous_layer = current_layer;
        output.truncate(layer.start);
        output.extend_from_slice(transformed.as_bytes());
        output.extend_from_slice(transition.as_bytes());
    }

    fn observe(&mut self, input: &str) {
        for line in input.lines() {
            self.reader
                .apply(&ParsedLine::parse(line), self.config.relative_e);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Command {
    G1,
    G92,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ParsedLine {
    command: Option<Command>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    e: Option<f64>,
}

impl ParsedLine {
    fn parse(line: &str) -> Self {
        let mut parsed = Self::default();
        for (index, word) in line
            .split(';')
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .enumerate()
        {
            if index == 0 {
                parsed.command = match word {
                    "G1" | "G01" => Some(Command::G1),
                    "G92" => Some(Command::G92),
                    _ => None,
                };
                continue;
            }
            let mut chars = word.chars();
            let Some(letter) = chars.next() else {
                continue;
            };
            match letter {
                'X' => parsed.x = chars.as_str().parse().ok().or(parsed.x),
                'Y' => parsed.y = chars.as_str().parse().ok().or(parsed.y),
                'Z' => parsed.z = chars.as_str().parse().ok().or(parsed.z),
                'E' => {
                    parsed.e = chars
                        .as_str()
                        .parse::<f32>()
                        .ok()
                        .map(f64::from)
                        .or(parsed.e);
                }
                _ => {}
            }
        }
        parsed
    }

    fn has_xy(self) -> bool {
        self.x.is_some() || self.y.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Movement {
    target_x: f64,
    target_y: f64,
    xy_distance: f64,
    extruding: bool,
    retracting: bool,
}

impl Movement {
    fn from(reader: &ReaderState, line: &ParsedLine, relative_e: bool) -> Self {
        let target_x = line.x.unwrap_or(reader.x);
        let target_y = line.y.unwrap_or(reader.y);
        let dx = target_x as f32 - reader.x as f32;
        let dy = target_y as f32 - reader.y as f32;
        let e_delta = line
            .e
            .map_or(0.0, |e| if relative_e { e } else { e - reader.e });
        Self {
            target_x,
            target_y,
            xy_distance: f64::from((dx * dx + dy * dy).sqrt()),
            extruding: e_delta > f64::EPSILON,
            retracting: e_delta < -f64::EPSILON,
        }
    }
}

impl ReaderState {
    fn apply(&mut self, line: &ParsedLine, relative_e: bool) {
        if !matches!(line.command, Some(Command::G1 | Command::G92)) {
            return;
        }
        if let Some(x) = line.x {
            self.x = x;
        }
        if let Some(y) = line.y {
            self.y = y;
        }
        if let Some(e) = line.e {
            self.e = if relative_e && line.command == Some(Command::G1) {
                self.e + e
            } else {
                e
            };
        }
    }
}

fn total_extrusion_length(input: &str, initial: &ReaderState, relative_e: bool) -> f32 {
    let mut reader = initial.clone();
    let mut total = 0.0_f32;
    for line in input.lines() {
        let parsed = ParsedLine::parse(line);
        let movement = Movement::from(&reader, &parsed, relative_e);
        if parsed.command == Some(Command::G1) && movement.extruding {
            total += movement.xy_distance as f32;
        }
        reader.apply(&parsed, relative_e);
    }
    total
}

fn set_word(line: &str, letter: char, value: String) -> String {
    // Port of `GCodeReader::GCodeLine::set` (`GCodeReader.cpp:323-357`):
    // an existing ` L` word's numeric text is replaced in place; a missing
    // word is inserted directly after the command (the first space).
    let needle = format!(" {letter}");
    let mut raw = line.to_owned();
    if let Some(pos) = raw.find(&needle) {
        let start = pos + needle.len();
        let end = raw[start..]
            .find(' ')
            .map_or(raw.len(), |offset| start + offset);
        raw.replace_range(start..end, &value);
    } else {
        let insertion = format!("{needle}{value}");
        match raw.find(' ') {
            Some(pos) => raw.insert_str(pos, &insertion),
            None => raw.push_str(&insertion),
        }
    }
    raw
}

fn word_value(line: &str, letter: char) -> Option<f64> {
    line.split(';')
        .next()?
        .split_whitespace()
        .find_map(|word| word.strip_prefix(letter)?.parse().ok())
}

fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}

fn push_passthrough(
    transformed: &mut String,
    transition: &mut String,
    line: &str,
    transition_out: bool,
) {
    push_line(transformed, line);
    if transition_out && line != PART_FAN_MARKER {
        push_line(transition, line);
    }
}

fn format_axis(value: f64) -> String {
    format!("{:.3}", value as f32)
}
fn format_z(value: f64) -> String {
    // `GCodeLine::set` default `decimal_digits = 3` with `std::fixed`.
    format!("{value:.3}")
}
fn format_e(value: f64) -> String {
    // `line.set(E, .., 5)` — fixed 5 decimals, leading zero kept.
    format!("{value:.5}")
}
