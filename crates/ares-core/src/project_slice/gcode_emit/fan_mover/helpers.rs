//! FanMover text helpers — fan command formatting, word parsing, motion
//! parsing, line splitting/rewriting, and the role tag mapping
//! (`FanMover.cpp` mirrors).

use crate::{ExtrusionRole, GCodeFlavor};

use super::BufferData;

/// The position/speed state FanMover needs to track per line — filled by
/// the caller's line walk (upstream reads these off the GCodeReader).
#[derive(Clone, Copy, Default)]
pub(super) struct LineMotion {
    pub(super) has_f: bool,
    pub(super) f_mm_s: f32,
    pub(super) dist: f32,
    pub(super) x: Option<f32>,
    pub(super) y: Option<f32>,
    pub(super) z: Option<f32>,
    pub(super) e: Option<f32>,
    pub(super) dx: f32,
    pub(super) dy: f32,
    pub(super) dz: f32,
    pub(super) de: f32,
}

pub(super) fn set_fan_line(percent: i16) -> String {
    // `GCodeWriter::set_fan` (GCodeWriter.cpp:1114) terminates the command
    // with a newline; a buffered kickstart line then flushes as `raw + "\n"`
    // leaving a blank line after the command, matching upstream.
    format!("M106 S{}\n", fan_pwm(percent))
}

pub(super) fn fan_pwm(percent: i16) -> u32 {
    (255.5 * f32::from(percent) / 100.0) as u32
}

pub(super) fn read_fan_speed(code: &str, flavor: GCodeFlavor) -> i16 {
    let command = code.split_whitespace().next().unwrap_or_default();
    if command == "M106" {
        let p = word(code, 'P');
        if let Some(p) = p {
            if flavor != GCodeFlavor::Mach3 && flavor != GCodeFlavor::Machinekit && p != 1.0 {
                return -1;
            }
        }
        word(code, 'S').map_or(-1, |s| (100.0 * s / 255.0) as i16)
    } else if command == "M127" || command == "M107" {
        0
    } else if command == "M126"
        && (flavor == GCodeFlavor::MakerWare || flavor == GCodeFlavor::Sailfish)
    {
        word(code, 'T').map_or(-1, |t| (100.0 * t / 255.0) as i16)
    } else {
        -1
    }
}

pub(super) fn word(code: &str, letter: char) -> Option<f32> {
    code.split_whitespace().find_map(|token| {
        let mut characters = token.chars();
        let first = characters.next()?;
        (first.eq_ignore_ascii_case(&letter))
            .then(|| characters.as_str().parse::<f32>().ok())
            .flatten()
    })
}

pub(super) fn parse_motion(code: &str) -> LineMotion {
    let mut motion = LineMotion::default();
    let mut x = None;
    let mut y = None;
    let mut z = None;
    let mut e = None;
    for token in code.split_whitespace().skip(1) {
        let mut characters = token.chars();
        let Some(first) = characters.next() else {
            continue;
        };
        let Ok(value) = characters.as_str().parse::<f32>() else {
            continue;
        };
        match first {
            'F' | 'f' => {
                motion.has_f = true;
                motion.f_mm_s = value / 60.0;
            }
            'X' | 'x' => x = Some(value),
            'Y' | 'y' => y = Some(value),
            'Z' | 'z' => z = Some(value),
            'E' | 'e' => e = Some(value),
            _ => {}
        }
    }
    motion.x = x;
    motion.y = y;
    motion.z = z;
    motion.e = e;
    motion
}

/// Splits a buffered G0/G1 line at `percent`, rewriting the axis words of
/// both halves (`change_axis_value`, FanMover.cpp:74-89). Returns
/// `(before, after)`.
pub(super) fn split_line(
    data: &mut BufferData,
    percent: f32,
    relative_e: bool,
) -> (String, String) {
    let mut before = data.raw.clone();
    if data.dx != 0.0 {
        before = replace_word(&before, 'X', data.x + data.dx * percent, 3);
    }
    if data.dy != 0.0 {
        before = replace_word(&before, 'Y', data.y + data.dy * percent, 3);
    }
    if data.dz != 0.0 {
        before = replace_word(&before, 'Z', data.z + data.dz * percent, 3);
    }
    let mut after = data.raw.clone();
    if data.de != 0.0 {
        if relative_e {
            before = replace_word(&before, 'E', data.de * percent, 5);
            after = replace_word(&after, 'E', data.de * (1.0 - percent), 5);
        } else {
            before = replace_word(&before, 'E', data.e + data.de * percent, 5);
        }
    }
    (before, after)
}

pub(super) fn replace_word(line: &str, axis: char, value: f32, digits: usize) -> String {
    let mut result = String::with_capacity(line.len() + 8);
    let code = line.split(';').next().unwrap_or(line);
    let comment = &line[code.len()..];
    let mut replaced = false;
    for token in code.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        if !replaced {
            let mut characters = token.chars();
            if let Some(first) = characters.next() {
                if first.eq_ignore_ascii_case(&axis) && characters.as_str().parse::<f32>().is_ok() {
                    result.push(first);
                    result.push_str(&format!("{value:.digits$}"));
                    replaced = true;
                    continue;
                }
            }
        }
        result.push_str(token);
    }
    result.push_str(comment);
    result
}

pub(super) fn role_from_str(role: &str) -> ExtrusionRole {
    match role.trim() {
        "Outer wall" => ExtrusionRole::ExternalPerimeter,
        "Inner wall" => ExtrusionRole::Perimeter,
        "Overhang wall" => ExtrusionRole::OverhangPerimeter,
        _ => ExtrusionRole::None,
    }
}
