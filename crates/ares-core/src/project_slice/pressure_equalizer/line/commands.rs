//! Command bodies from PressureEqualizer.cpp::process_line.

use super::lex::{is_eol, split_float, split_float_opt, split_int};
use super::{E, F, GCodeLine, GCodeLineType, LineParser, X, Y, Z};

impl LineParser {
    pub(super) fn parse_g_command(
        &mut self,
        code: i32,
        tail: &str,
        buf: &mut GCodeLine,
    ) -> Option<()> {
        match code {
            0 | 1 => self.parse_move(tail, buf)?,
            92 => self.parse_set_position(tail),
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
        Some(())
    }

    fn parse_move(&mut self, tail: &str, buf: &mut GCodeLine) -> Option<()> {
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
        Some(())
    }

    fn parse_set_position(&mut self, tail: &str) {
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

    pub(super) fn parse_tool_change(&mut self, rest: &str, buf: &mut GCodeLine) {
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
}
