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

    /// `volumetric_correction_avg` (`PressureEqualizer.hpp:132-137`):
    /// capped to `[0.05, 1.00000001]` (upstream clamps; a bare division
    /// can exceed 1 when the limited start/end rates average above the
    /// original — the skirt tie case).
    pub(crate) fn volumetric_correction_avg(&self) -> f32 {
        if self.volumetric_extrusion_rate == 0.0 {
            1.0
        } else {
            (0.5 * (self.volumetric_extrusion_rate_start + self.volumetric_extrusion_rate_end)
                / self.volumetric_extrusion_rate)
                .clamp(0.05, 1.000_001)
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
                self.parse_g_command(code, tail, buf)?;
                // Upstream ignores `InvalidArgument` from `parse_int` and
                // keeps going (invalid G codes are treated as OTHER).
                let _ = code;
            }
            'M' => {
                // Ignore the rest of the M-codes.
            }
            'T' => self.parse_tool_change(rest, buf),
            _ => {}
        }
        buf.extruder_id = self.current_extruder;
        buf.pos_end = self.current_pos;
        Some(())
    }
}

/// `is_eol` (`PressureEqualizer.cpp:210`): a comment ends the line too.
mod lex;

use lex::{role_from_u32, split_int};

mod commands;
