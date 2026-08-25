use super::{
    CoolingLine, State, TYPE_ADJUSTABLE, TYPE_EXTERNAL_PERIMETER, TYPE_EXTRUDE_END, TYPE_G0,
    TYPE_G1, TYPE_G2, TYPE_G3, TYPE_G4, TYPE_G92, TYPE_HAS_F, TYPE_WIPE,
};

const SET_SPEED: &[u8] = b";_EXTRUDE_SET_SPEED";
const EXTERNAL_PERIMETER: &[u8] = b";_EXTERNAL_PERIMETER";
const WIPE: &[u8] = b";_WIPE";

pub(super) fn layer(gcode: &[u8], state: &mut State) -> Vec<CoolingLine> {
    let mut lines: Vec<CoolingLine> = Vec::new();
    let mut active_speed_modifier = None;
    let mut layer_had_extrusion = false;
    let mut start = 0;

    while start < gcode.len() {
        let end = gcode[start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(gcode.len(), |offset| start + offset + 1);
        let mut text = &gcode[start..end];
        if text.ends_with(b"\n") {
            text = &text[..text.len() - 1];
        }
        if text.ends_with(b"\r") {
            text = &text[..text.len() - 1];
        }

        let mut line = CoolingLine::new(command_type(text), start, end);
        if line.kind != 0 {
            let mut new_position = state.position;
            parse_position(text, state.position, &mut new_position, &mut line.kind);

            let external_perimeter = contains(text, EXTERNAL_PERIMETER);
            let wipe = contains(text, WIPE);
            if external_perimeter {
                line.kind |= TYPE_EXTERNAL_PERIMETER;
            }
            if wipe {
                line.kind |= TYPE_WIPE;
            }

            let set_speed = contains(text, SET_SPEED);
            if set_speed {
                layer_had_extrusion = true;
            }
            if set_speed && !wipe && !(state.config.keep_outer_wall_speed && external_perimeter) {
                line.kind |= TYPE_ADJUSTABLE;
                active_speed_modifier = Some(lines.len());
            }

            measure_and_aggregate(
                &mut line,
                state,
                new_position,
                active_speed_modifier,
                &mut lines,
            );
            state.position = new_position;
        } else if text.starts_with(b";_EXTRUDE_END") {
            line.kind = TYPE_EXTRUDE_END;
            active_speed_modifier = None;
        } else if text.starts_with(b"G4 ") {
            line.kind = TYPE_G4;
            line.time = word_value(text, b'S')
                .or_else(|| word_value(text, b'P').map(|value| value * 0.001))
                .unwrap_or(0.0);
            line.maximum_time = line.time;
        }

        if !layer_had_extrusion {
            line.time = 0.0;
            line.maximum_time = 0.0;
        }
        if line.kind != 0 {
            lines.push(line);
        }
        start = end;
    }

    lines
}

fn command_type(line: &[u8]) -> u32 {
    if line.starts_with(b"G0 ") {
        TYPE_G0
    } else if line.starts_with(b"G1 ") {
        TYPE_G1
    } else if line.starts_with(b"G92 ") {
        TYPE_G92
    } else if line.starts_with(b"G2 ") {
        TYPE_G2
    } else if line.starts_with(b"G3 ") {
        TYPE_G3
    } else {
        0
    }
}

fn parse_position(
    line: &[u8],
    current_position: [f32; 7],
    position: &mut [f32; 7],
    kind: &mut u32,
) {
    let command = line.split(|byte| *byte == b';').next().unwrap_or(line);
    for token in command[3.min(command.len())..]
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|token| !token.is_empty())
    {
        let Some((&axis, value)) = token.split_first() else {
            continue;
        };
        let Ok(value) = std::str::from_utf8(value) else {
            continue;
        };
        let Ok(value) = value.parse::<f64>() else {
            continue;
        };
        let value = value as f32;
        match axis {
            b'X' => position[0] = value,
            b'Y' => position[1] = value,
            b'Z' => position[2] = value,
            b'E' => position[3] = value,
            b'F' => {
                position[4] = value / 60.0;
                if *kind & TYPE_G92 == 0 {
                    *kind |= TYPE_HAS_F;
                }
            }
            b'I' => position[5] = value + current_position[0],
            b'J' => position[6] = value + current_position[1],
            _ => {}
        }
    }
}

fn measure_and_aggregate(
    line: &mut CoolingLine,
    state: &mut State,
    new_position: [f32; 7],
    active_speed_modifier: Option<usize>,
    lines: &mut [CoolingLine],
) {
    if line.kind & TYPE_G92 != 0 {
        return;
    }
    if state.config.relative_e {
        state.position[3] = 0.0;
    }
    let adjustable_block = line.kind & TYPE_ADJUSTABLE != 0 || active_speed_modifier.is_some();
    measure_movement(
        line,
        &state.position,
        new_position,
        state.config.minimum_speed,
        adjustable_block,
    );

    let Some(index) = active_speed_modifier else {
        return;
    };
    if index >= lines.len() || line.kind & (TYPE_G1 | TYPE_G2 | TYPE_G3) == 0 {
        return;
    }
    let speed_modifier = &mut lines[index];
    speed_modifier.length += line.length;
    speed_modifier.time += line.time;
    if speed_modifier.maximum_time != f32::MAX {
        if line.maximum_time == f32::MAX {
            speed_modifier.maximum_time = f32::MAX;
        } else {
            speed_modifier.maximum_time += line.maximum_time;
        }
    }
    line.kind = 0;
}

fn measure_movement(
    line: &mut CoolingLine,
    current_position: &[f32; 7],
    new_position: [f32; 7],
    minimum_speed: f32,
    adjustable_block: bool,
) {
    if line.kind & TYPE_G92 != 0 {
        return;
    }
    let dx = new_position[0] - current_position[0];
    let dy = new_position[1] - current_position[1];
    let dz = new_position[2] - current_position[2];
    let xy_squared = if line.kind & (TYPE_G2 | TYPE_G3) != 0 {
        let length = arc_length(
            [current_position[0], current_position[1]],
            [new_position[0], new_position[1]],
            [new_position[5], new_position[6]],
            line.kind & TYPE_G3 != 0,
        );
        length * length
    } else {
        dx * dx + dy * dy
    };
    let distance_squared = xy_squared + dz * dz;
    if distance_squared > 0.0 {
        line.length = distance_squared.sqrt();
    } else {
        line.length = (new_position[3] - current_position[3]).abs();
    }

    line.feedrate = new_position[4];
    if line.length > 0.0 {
        line.time = line.length / line.feedrate;
    }
    line.maximum_time = line.time;
    if adjustable_block {
        line.maximum_time = if minimum_speed == 0.0 {
            f32::MAX
        } else {
            line.time.max(line.length / minimum_speed)
        };
    }
}

fn arc_length(start: [f32; 2], end: [f32; 2], center: [f32; 2], ccw: bool) -> f32 {
    let first = [center[0] - start[0], center[1] - start[1]];
    let second = [center[0] - end[0], center[1] - end[1]];
    let radius = (first[0] * first[0] + first[1] * first[1]).sqrt();
    let difference = [first[0] - second[0], first[1] - second[1]];
    let radians = if (difference[0] * difference[0] + difference[1] * difference[1]).sqrt() < 1e-6 {
        std::f64::consts::TAU as f32
    } else {
        let dot = first[0] * second[0] + first[1] * second[1];
        let cross =
            f64::from(first[0]) * f64::from(second[1]) - f64::from(first[1]) * f64::from(second[0]);
        let radians = cross.atan2(f64::from(dot)) as f32;
        if ccw {
            if radians < 0.0 {
                (std::f64::consts::TAU + f64::from(radians)) as f32
            } else {
                radians
            }
        } else if radians < 0.0 {
            radians.abs()
        } else {
            (std::f64::consts::TAU - f64::from(radians)) as f32
        }
    };
    radius * radians
}

fn word_value(line: &[u8], word: u8) -> Option<f32> {
    let command = line.split(|byte| *byte == b';').next().unwrap_or(line);
    command
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|token| !token.is_empty())
        .find_map(|token| {
            let (&first, value) = token.split_first()?;
            (first == word)
                .then(|| {
                    std::str::from_utf8(value)
                        .ok()?
                        .parse::<f64>()
                        .ok()
                        .map(|value| value as f32)
                })
                .flatten()
        })
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
