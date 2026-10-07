//! ORCA preview attributes of `store_move_vertex` (`GCodeProcessor.cpp:
//! 5630-5664`, getters `:5771-5885`) and the pressure-advance commands
//! (`:5031-5060`). Values are the Normal-mode `float` machine state.

use super::super::motion::MotionState;
use super::super::motion_util::word;
use crate::gcode_processor::EMoveType;
use crate::options::GCodeFlavor;

/// Normal-mode `TimeMachine` accelerations and `machine_limits` jerk /
/// junction-deviation values at the start of a processor line.
#[derive(Clone, Copy, Default)]
pub(in crate::project_slice::gcode_emit::processor) struct PreviewMachine {
    acceleration: f32,
    retract_acceleration: f32,
    travel_acceleration: f32,
    max_axis_acceleration: [f32; 4],
    jerk: [f32; 4],
    /// `machine_max_junction_deviation` when `m_flavor == gcfMarlinFirmware`
    /// and positive; `0` selects the classic jerks.
    junction_deviation: f32,
}

impl PreviewMachine {
    pub(in crate::project_slice::gcode_emit::processor) fn new(state: &MotionState) -> Self {
        let junction_deviation = state.junction_deviation as f32;
        Self {
            acceleration: state.acceleration as f32,
            retract_acceleration: state.retract_acceleration as f32,
            travel_acceleration: state.travel_acceleration as f32,
            max_axis_acceleration: state.max_acceleration.map(|value| value as f32),
            jerk: state.jerk.map(|value| value as f32),
            junction_deviation: if state.gcode_flavor == GCodeFlavor::MarlinFirmware
                && junction_deviation > 0.0
            {
                junction_deviation
            } else {
                0.0
            },
        }
    }

    /// `move_acceleration` and `move_jerk` for a vertex whose machine
    /// position moves from `start` to `end`.
    pub(super) fn sample(&self, move_type: EMoveType, start: [f64; 4], end: [f64; 4]) -> [f32; 2] {
        let acceleration = match move_type {
            EMoveType::Travel => self.travel_acceleration,
            EMoveType::Retract | EMoveType::Unretract => self.retract_acceleration,
            _ => self.acceleration,
        };
        let jerk: [f32; 4] = std::array::from_fn(|axis| self.axis_jerk(axis, acceleration));
        let [x, y, z, e] = std::array::from_fn(|axis| (end[axis] - start[axis]) as f32 != 0.0);
        let [jerk_x, jerk_y, jerk_z, jerk_e] = jerk;
        let move_jerk = match (x, y, z, e) {
            (false, false, false, true) => jerk_e,
            (false, false, true, _) => jerk_z,
            (true, true, ..) => jerk_x.min(jerk_y),
            (true, false, ..) => jerk_x,
            (false, true, ..) => jerk_y,
            (false, false, false, false) => jerk_x.min(jerk_y),
        };
        [acceleration, move_jerk]
    }

    /// `get_axis_max_jerk_with_jd(mode, axis, acceleration)` or the classic
    /// `get_axis_max_jerk`.
    fn axis_jerk(&self, axis: usize, acceleration: f32) -> f32 {
        if self.junction_deviation <= 0.0 {
            return self.jerk[axis];
        }
        let max_acceleration = self.max_axis_acceleration[axis];
        let mut effective = if acceleration <= 0.0 {
            self.acceleration
        } else {
            acceleration
        };
        if max_acceleration > 0.0 {
            effective = if effective > 0.0 {
                effective.min(max_acceleration)
            } else {
                max_acceleration
            };
        }
        if effective <= 0.0 {
            return 0.0;
        }
        (self.junction_deviation * effective * 2.5).sqrt()
    }
}

/// `process_M900` / `process_M572` (the `K` / `S` word of `code`) and the
/// Klipper `process_SET_PRESSURE_ADVANCE`; returns the new
/// `m_pressure_advance`, or `None` when the line leaves it unchanged.
pub(super) fn pressure_advance(
    command: &str,
    code: &str,
    raw: &str,
    flavor: GCodeFlavor,
) -> Option<f32> {
    let value = match command {
        "M900" => word(code, 'K')? as f32,
        "M572" => word(code, 'S')? as f32,
        _ if flavor == GCodeFlavor::Klipper
            && command.eq_ignore_ascii_case("SET_PRESSURE_ADVANCE") =>
        {
            set_pressure_advance(raw)?
        }
        _ => return None,
    };
    Some(value.max(0.0))
}

/// `std::regex_search(raw, R"(SET_PRESSURE_ADVANCE\s+(?:.*\s+)?ADVANCE\s*=\s*([\d.]+))")`
/// followed by `std::stof` (failure → `0`).
fn set_pressure_advance(raw: &str) -> Option<f32> {
    const COMMAND: &str = "SET_PRESSURE_ADVANCE";
    raw.match_indices(COMMAND).find_map(|(start, _)| {
        let after = &raw[start + COMMAND.len()..];
        let rest = after.trim_start_matches(is_space);
        if rest.len() == after.len() {
            return None;
        }
        // Greedy `.*`: the last `ADVANCE` that is preceded by whitespace (or
        // starts `rest`) and is followed by `\s*=\s*[\d.]+`.
        rest.rmatch_indices("ADVANCE")
            .filter(|&(at, _)| at == 0 || rest[..at].ends_with(is_space))
            .filter_map(|(at, key)| {
                let value = rest[at + key.len()..].trim_start_matches(is_space);
                let value = value.strip_prefix('=')?.trim_start_matches(is_space);
                let end = value
                    .find(|c: char| !c.is_ascii_digit() && c != '.')
                    .unwrap_or(value.len());
                (end > 0).then(|| &value[..end])
            })
            .next()
            .map(stof)
    })
}

fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0b' | '\x0c')
}

/// `std::stof` on a `[\d.]+` match: the longest decimal prefix; no digit or
/// an `ERANGE` overflow/underflow throws, which the caller turns into `0`.
fn stof(text: &str) -> f32 {
    let integer = text.find('.').unwrap_or(text.len());
    let fraction = text[integer..]
        .char_indices()
        .skip(1)
        .find(|&(_, c)| c == '.')
        .map_or(text.len(), |(at, _)| integer + at);
    text[..fraction]
        .parse::<f32>()
        .ok()
        .filter(|value| !value.is_infinite() && !value.is_subnormal())
        .unwrap_or(0.0)
}
