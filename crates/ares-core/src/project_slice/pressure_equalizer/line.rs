//! `GCodeLine` and `LineParser` (`PressureEqualizer.hpp:110-160` and
//! `PressureEqualizer.cpp:272-455`).

use super::{EXTRUDE_END_TAG, EXTRUDE_SET_SPEED_TAG, EXTRUSION_ROLE_TAG};
use crate::ExtrusionRole;

/// Indices into `GCodeLine::pos_start`/`pos_end` (`X,Y,Z,E,F`).
pub(crate) const X: usize = 0;
pub(crate) const Y: usize = 1;
pub(crate) const Z: usize = 2;
pub(crate) const E: usize = 3;
pub(crate) const F: usize = 4;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum GCodeLineType {
    #[default]
    Invalid,
    Noop,
    Other,
    Retract,
    Unretract,
    ToolChange,
    Move,
    Extrude,
}

/// `PressureEqualizer.hpp:110-160 GCodeLine`.
#[derive(Clone, Debug)]
pub(crate) struct GCodeLine {
    pub(crate) line_type: GCodeLineType,
    pub(crate) raw: String,
    pub(crate) modified: bool,
    pub(crate) extruder_id: usize,
    pub(crate) pos_start: [f32; 5],
    pub(crate) pos_end: [f32; 5],
    pub(crate) pos_provided: [bool; 5],
    pub(crate) volumetric_extrusion_rate: f32,
    pub(crate) volumetric_extrusion_rate_start: f32,
    pub(crate) volumetric_extrusion_rate_end: f32,
    pub(crate) max_volumetric_extrusion_rate_slope_positive: f32,
    pub(crate) max_volumetric_extrusion_rate_slope_negative: f32,
    pub(crate) extrusion_role: ExtrusionRole,
    pub(crate) adjustable_flow: bool,
    pub(crate) extrude_set_speed_tag: bool,
    pub(crate) extrude_end_tag: bool,
}

impl Default for GCodeLine {
    fn default() -> Self {
        Self {
            line_type: GCodeLineType::Invalid,
            raw: String::new(),
            modified: false,
            extruder_id: 0,
            pos_start: [0.0; 5],
            pos_end: [0.0; 5],
            pos_provided: [false; 5],
            volumetric_extrusion_rate: 0.0,
            volumetric_extrusion_rate_start: 0.0,
            volumetric_extrusion_rate_end: 0.0,
            max_volumetric_extrusion_rate_slope_positive: 0.0,
            max_volumetric_extrusion_rate_slope_negative: 0.0,
            extrusion_role: ExtrusionRole::None,
            adjustable_flow: false,
            extrude_set_speed_tag: false,
            extrude_end_tag: false,
        }
    }
}

impl GCodeLine {
    pub(crate) fn moving_xy(&self) -> bool {
        (self.pos_end[X] - self.pos_start[X]).abs() > 0.0
            || (self.pos_end[Y] - self.pos_start[Y]).abs() > 0.0
    }

    pub(crate) fn moving_z(&self) -> bool {
        (self.pos_end[Z] - self.pos_start[Z]).abs() > 0.0
    }

    pub(crate) fn extruding(&self) -> bool {
        self.moving_xy() && self.pos_end[E] > self.pos_start[E]
    }

    pub(crate) fn dist_xy2(&self) -> f32 {
        let dx = self.pos_end[X] - self.pos_start[X];
        let dy = self.pos_end[Y] - self.pos_start[Y];
        dx * dx + dy * dy
    }

    pub(crate) fn dist_xyz(&self) -> f32 {
        let dz = self.pos_end[Z] - self.pos_start[Z];
        (self.dist_xy2() + dz * dz).sqrt()
    }

    /// `feedrate()` returns mm/min (`pos_end[4]`).
    pub(crate) fn feedrate(&self) -> f32 {
        self.pos_end[F]
    }

    pub(crate) fn volumetric_correction_avg(&self) -> f32 {
        if self.volumetric_extrusion_rate == 0.0 {
            1.0
        } else {
            0.5 * (self.volumetric_extrusion_rate_start + self.volumetric_extrusion_rate_end)
                / self.volumetric_extrusion_rate
        }
    }
}

/// Parser state — the mutable axes/role/extruder tracking of
/// `PressureEqualizer` (`m_current_pos`, `m_current_extrusion_role`,
/// `m_current_extruder`, `m_retracted`, `opened_extrude_set_speed_block`).
pub(crate) struct LineParser {
    pub(crate) current_pos: [f32; 5],
    pub(crate) current_extruder: usize,
    pub(crate) current_role: ExtrusionRole,
    pub(crate) retracted: bool,
    pub(crate) opened_extrude_set_speed_block: bool,
    use_relative_e_distances: bool,
    filament_crosssections: Vec<f32>,
}

impl LineParser {
    pub(crate) fn new(filament_diameters_mm: &[f64], use_relative_e_distances: bool) -> Self {
        Self {
            current_pos: [0.0; 5],
            current_extruder: 0,
            current_role: ExtrusionRole::None,
            // Upstream expects the first command to fill the nozzle
            // (deretract) — `m_retracted = true`.
            retracted: true,
            opened_extrude_set_speed_block: false,
            use_relative_e_distances,
            filament_crosssections: filament_diameters_mm
                .iter()
                .map(|d| (0.25 * std::f64::consts::PI * d * d) as f32)
                .collect(),
        }
    }

    /// `process_line` (`PressureEqualizer.cpp:272-455`): parse one G-code
    /// line into `buf`. Returns `None` when the line is a marker that must
    /// be filtered out of the output (`;_EXTRUSION_ROLE:` updates the
    /// tracked role) — mirroring upstream's `return false`.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn parse_line(&mut self, raw: &str, buf: &mut GCodeLine) -> Option<()> {
        if let Some(role_text) = raw.strip_prefix(EXTRUSION_ROLE_TAG) {
            let role = role_text.trim().parse::<u32>().unwrap_or(0);
            self.current_role = role_from_u32(role);
            return None;
        }

        buf.raw = raw.to_owned();
        buf.line_type = GCodeLineType::Other;
        buf.modified = false;
        buf.pos_start = self.current_pos;
        buf.pos_end = self.current_pos;
        buf.pos_provided = [false; 5];
        buf.volumetric_extrusion_rate = 0.0;
        buf.volumetric_extrusion_rate_start = 0.0;
        buf.volumetric_extrusion_rate_end = 0.0;
        buf.max_volumetric_extrusion_rate_slope_positive = 0.0;
        buf.max_volumetric_extrusion_rate_slope_negative = 0.0;
        buf.extrusion_role = self.current_role;

        buf.extrude_set_speed_tag = raw.contains(EXTRUDE_SET_SPEED_TAG);
        buf.extrude_end_tag = raw.contains(EXTRUDE_END_TAG);
        debug_assert!(!(buf.extrude_set_speed_tag && buf.extrude_end_tag));
        if buf.extrude_set_speed_tag {
            self.opened_extrude_set_speed_block = true;
        } else if buf.extrude_end_tag {
            self.opened_extrude_set_speed_block = false;
        }

        let mut rest = raw;
        let first = rest.chars().next()?.to_ascii_uppercase();
        rest = &rest[first.len_utf8()..];
        match first {
            'G' => {
                let (code, tail) = split_int(rest)?;
                match code {
                    0 | 1 => {
                        buf.adjustable_flow = self.opened_extrude_set_speed_block;
                        let mut new_pos = self.current_pos;
                        let mut changed = [false; 5];
                        let mut tail = tail;
                        while let Some(c) = tail.chars().next() {
                            if is_eol(c) {
                                break;
                            }
                            let axis = c.to_ascii_uppercase();
                            tail = &tail[1..];
                            let index = match axis {
                                'X' => X,
                                'Y' => Y,
                                'Z' => Z,
                                'E' => E,
                                'F' => F,
                                _ => continue,
                            };
                            let (value, remainder) = split_float(tail)?;
                            tail = remainder;
                            buf.pos_provided[index] = true;
                            new_pos[index] = value;
                            if index == E && self.use_relative_e_distances {
                                new_pos[E] += self.current_pos[E];
                            }
                            changed[index] = new_pos[index] != self.current_pos[index];
                        }
                        if changed[E] {
                            let diff = new_pos[E] - self.current_pos[E];
                            if diff < 0.0 {
                                buf.line_type = GCodeLineType::Retract;
                                self.retracted = true;
                            } else if !changed[X] && !changed[Y] && !changed[Z] {
                                buf.line_type = GCodeLineType::Unretract;
                                self.retracted = false;
                            } else {
                                debug_assert!(changed[X] || changed[Y]);
                                buf.line_type = GCodeLineType::Extrude;
                                let dx = new_pos[X] - self.current_pos[X];
                                let dy = new_pos[Y] - self.current_pos[Y];
                                let dz = new_pos[Z] - self.current_pos[Z];
                                let de = new_pos[E] - self.current_pos[E];
                                // volumetric rate = A_filament * F * L_e / L_xyz
                                let len2 = dx * dx + dy * dy + dz * dz;
                                let rate = self.filament_crosssections[self.current_extruder]
                                    * new_pos[F]
                                    * (de * de / len2).sqrt();
                                buf.volumetric_extrusion_rate = rate;
                                buf.volumetric_extrusion_rate_start = rate;
                                buf.volumetric_extrusion_rate_end = rate;
                            }
                        } else if changed[X] || changed[Y] || changed[Z] {
                            buf.line_type = GCodeLineType::Move;
                        }
                        self.current_pos = new_pos;
                    }
                    92 => {
                        // G92: set a logical coordinate without moving.
                        let mut tail = tail;
                        while let Some(c) = tail.chars().next() {
                            if is_eol(c) {
                                break;
                            }
                            let axis = c.to_ascii_uppercase();
                            tail = &tail[1..];
                            let index = match axis {
                                'X' => X,
                                'Y' => Y,
                                'Z' => Z,
                                'E' => E,
                                _ => continue,
                            };
                            if let Some((value, remainder)) = split_float_opt(tail) {
                                self.current_pos[index] = value;
                                tail = remainder;
                            } else {
                                self.current_pos[index] = 0.0;
                            }
                        }
                    }
                    10 | 22 => {
                        buf.line_type = GCodeLineType::Retract;
                        self.retracted = true;
                    }
                    11 | 23 => {
                        buf.line_type = GCodeLineType::Unretract;
                        self.retracted = false;
                    }
                    _ => {}
                }
                // Upstream ignores `InvalidArgument` from `parse_int` and
                // keeps going (invalid G codes are treated as OTHER).
                let _ = code;
            }
            'M' => {
                // Ignore the rest of the M-codes.
            }
            'T' => {
                if let Some((new_extruder, _)) = split_int(rest) {
                    if new_extruder as usize != self.current_extruder {
                        self.current_extruder = new_extruder as usize;
                        self.retracted = true;
                        buf.line_type = GCodeLineType::ToolChange;
                    } else {
                        buf.line_type = GCodeLineType::Noop;
                    }
                }
            }
            _ => {}
        }
        buf.extruder_id = self.current_extruder;
        buf.pos_end = self.current_pos;
        Some(())
    }
}

/// `is_eol` (`PressureEqualizer.cpp:210`): a comment ends the line too.
fn is_eol(c: char) -> bool {
    c == '\r' || c == '\n' || c == ';'
}

fn skip_ws(text: &str) -> &str {
    text.trim_start_matches([' ', '\t'])
}

/// `parse_int` — advances past the int; `None` mirrors the
/// `InvalidArgument` throw path.
fn split_int(text: &str) -> Option<(i32, &str)> {
    let text = skip_ws(text);
    let digits: usize = text
        .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '+')
        .unwrap_or(text.len());
    if digits == 0 {
        return None;
    }
    let value = text[..digits].parse().ok()?;
    let rest = &text[digits..];
    let next = rest.chars().next();
    match next {
        None => Some((value, rest)),
        Some(c) if c == ' ' || c == '\t' || is_eol(c) => Some((value, rest)),
        _ => None,
    }
}

/// `parse_float` — fast_float semantics with the ws-or-eol terminator check.
fn split_float(text: &str) -> Option<(f32, &str)> {
    let (value, rest) = split_float_opt(text)?;
    match rest.chars().next() {
        None => Some((value, rest)),
        Some(c) if c == ' ' || c == '\t' || is_eol(c) => Some((value, rest)),
        _ => None,
    }
}

fn split_float_opt(text: &str) -> Option<(f32, &str)> {
    let text = skip_ws(text);
    let mut end = 0;
    let bytes = text.as_bytes();
    if end < bytes.len() && (bytes[end] == b'-' || bytes[end] == b'+') {
        end += 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end < bytes.len() && bytes[end] == b'.' {
        end += 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
    }
    if end < bytes.len() && (bytes[end] == b'e' || bytes[end] == b'E') {
        let mut exp = end + 1;
        if exp < bytes.len() && (bytes[exp] == b'-' || bytes[exp] == b'+') {
            exp += 1;
        }
        let exp_start = exp;
        while exp < bytes.len() && bytes[exp].is_ascii_digit() {
            exp += 1;
        }
        if exp > exp_start {
            end = exp;
        }
    }
    if end == 0 {
        return None;
    }
    let value: f32 = text[..end].parse().ok()?;
    Some((value, &text[end..]))
}

/// `;_EXTRUSION_ROLE:<n>` — the upstream enum's discriminants
/// (`ExtrusionRole.hpp`). Ares's `ExtrusionRole` re-declares the same
/// order; map by discriminant to keep the marker byte-compatible.
fn role_from_u32(value: u32) -> ExtrusionRole {
    const ALL: [ExtrusionRole; 20] = [
        ExtrusionRole::None,
        ExtrusionRole::Perimeter,
        ExtrusionRole::ExternalPerimeter,
        ExtrusionRole::OverhangPerimeter,
        ExtrusionRole::InternalInfill,
        ExtrusionRole::SolidInfill,
        ExtrusionRole::TopSolidInfill,
        ExtrusionRole::BottomSurface,
        ExtrusionRole::Ironing,
        ExtrusionRole::BridgeInfill,
        ExtrusionRole::InternalBridgeInfill,
        ExtrusionRole::GapFill,
        ExtrusionRole::Skirt,
        ExtrusionRole::Brim,
        ExtrusionRole::SupportMaterial,
        ExtrusionRole::SupportMaterialInterface,
        ExtrusionRole::SupportTransition,
        ExtrusionRole::WipeTower,
        ExtrusionRole::Custom,
        ExtrusionRole::Mixed,
    ];
    ALL.get(value as usize)
        .copied()
        .unwrap_or(ExtrusionRole::None)
}
