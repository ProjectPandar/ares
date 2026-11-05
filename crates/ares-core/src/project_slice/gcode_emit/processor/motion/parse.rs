//! `GCodeProcessor::process_G1` argument admission and linear/arc motion.
use super::{MotionBlock, MotionKind, MotionState, arc, firmware, homing};
use crate::GCodeFlavor;
use crate::project_slice::gcode_emit::processor::motion_util::{
    MMMIN_TO_MMSEC, norm, precise_word, word,
};
use std::f64::consts::PI;

impl MotionState {
    pub(super) fn motion(&mut self, code: &str) -> Option<MotionBlock> {
        if code == "G20" {
            self.units = 25.4;
            return None;
        }
        if code == "G21" {
            self.units = 1.0;
            return None;
        }
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
        if code.split_whitespace().next() == Some("G92") {
            self.set_position(code);
            return None;
        }
        if self.handle_config_command(code) || self.handle_cached_feedrate(code) {
            return None;
        }
        let command = code.split_whitespace().next()?;
        if command == "G10" || command == "G11" {
            // `GCodeProcessor::process_G10/G11` (`GCodeProcessor.cpp:4873-
            // 4893`): emulate a G1 with the retraction E delta and the
            // retraction/deretraction feedrate — the fake G1's line id was
            // already decremented upstream so the M73 cache alignment is
            // unchanged.
            return firmware::emulate_firmware_retraction(self, command);
        }
        if command == "G28" {
            return self.motion(&homing::homing_g1(code));
        }
        if !matches!(command, "G0" | "G1" | "G2" | "G3") {
            return None;
        }
        let precise = self.gcode_flavor == GCodeFlavor::Klipper && matches!(command, "G0" | "G1");
        let read = |letter| {
            if precise {
                precise_word(code, letter)
            } else {
                word(code, letter)
            }
        };
        if let Some(value) = read('F') {
            self.feedrate = if precise {
                f64::from((value * f64::from(MMMIN_TO_MMSEC)) as f32)
            } else {
                f64::from(value as f32 * MMMIN_TO_MMSEC)
            };
        }
        let old = self.position;
        let mut next = old;
        for (axis, letter) in ['X', 'Y', 'Z'].into_iter().enumerate() {
            let Some(value) = read(letter) else {
                continue;
            };
            // Klipper linear G1 retains precise_value (cpp:5195-5202).
            // Other paths keep the GCodeReader float-word quantization.
            let value = value * f64::from(self.units);
            next[axis] = match self.relative {
                true => old[axis] + value,
                false => self.origin[axis] + value,
            };
        }
        let old_e = self.e_position;
        let next_e = match read('E').map(|value| value * f64::from(self.units)) {
            Some(value) if self.relative || self.e_relative => old_e + value,
            Some(value) => self.origin[3] + value,
            None => old_e,
        };
        let e_delta = next_e - old_e;
        self.e_position = next_e;
        self.position = next;
        // Upstream always tracks the position; a non-positive feedrate only
        // Upstream always tracks the position and creates the TimeBlock for
        // any displaced move; `Trapezoid::cruise_time()` returns 0 when
        // `cruise_feedrate == 0` (`GCodeProcessor.hpp:439`), so blocks with
        // zero feedrate contribute zero time but still occupy a
        // g1_times_cache entry — the M73 lookup depends on it.
        // The block's `speed` field carries the (possibly zero) feedrate;
        // the schedule treats zero-speed blocks as zero-time.
        let mut delta = [
            next[0] - old[0],
            next[1] - old[1],
            next[2] - old[2],
            e_delta,
        ];
        let xyz_distance = norm([delta[0], delta[1], delta[2], 0.0]);
        let e_only = xyz_distance <= f64::EPSILON;
        // `move_length` (`GCodeProcessor.cpp:3963-3968`): the squared
        // components sum in f64 but the SUM STORES TO FLOAT before the
        // sqrt — `float sq_xyz_length = sqr(dx)+sqr(dy)+sqr(dz)`.
        // Skipping the intermediate f32 rounding shifts single-axis Z
        // moves by ~1 f32 ulp, which flips the following junction's
        // per-axis jerk comparison (the wipe→travel junction 3.5355 vs
        // the oracle's 2.0).
        let sum_squared = delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2];
        let mut distance = if sum_squared > 0.0 {
            f64::from((sum_squared as f32).sqrt())
        } else {
            f64::from(delta[3].abs() as f32)
        };
        if matches!(command, "G2" | "G3") {
            // R fitting takes precedence (`GCodeProcessor.cpp:4557-4592`):
            // the center/radius come from `ArcWelder::arc_center`.
            let (i, j, radius) = match word(code, 'R').filter(|r| *r != 0.0) {
                Some(r) => {
                    match arc::center_from_radius(
                        [old[0], old[1]],
                        [next[0], next[1]],
                        r,
                        command != "G2",
                    ) {
                        Some(center) => (
                            center[0] - old[0],
                            center[1] - old[1],
                            ((old[0] - center[0]).powi(2) + (old[1] - center[1]).powi(2)).sqrt(),
                        ),
                        None => (0.0, 0.0, 0.0),
                    }
                }
                None => {
                    let i = word(code, 'I').unwrap_or(0.0);
                    let j = word(code, 'J').unwrap_or(0.0);
                    (i, j, (i * i + j * j).sqrt())
                }
            };
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
            distance = (radius * sweep.abs()).hypot(delta[2]);
            delta[0] = sweep.cos() * radius;
            delta[1] = sweep.sin() * radius;
        }
        self.position = next;
        if distance <= f64::EPSILON {
            return None;
        }
        let has_xy = delta[0] != 0.0 || delta[1] != 0.0;
        let acceleration = if e_only {
            self.retract_acceleration
        } else if self.wiping || (e_delta > 0.0 && has_xy) {
            self.acceleration
        } else {
            self.travel_acceleration
        };
        // The axis factor chain and centripetal limit run in the
        // planner's `prepare` in upstream order
        // (`GCodeProcessor.cpp:3993-4080`); the minimum-rate-selected F travels here.
        Some(MotionBlock {
            klipper: self.klipper,
            junction_deviation: if self.gcode_flavor == GCodeFlavor::MarlinFirmware {
                self.junction_deviation
            } else {
                0.0
            },
            virtual_motion: false,
            distance,
            delta,
            speed: self.block_feedrate(e_delta),
            acceleration,
            centripetal_acceleration: self.acceleration,
            max_feedrate: self.max_feedrate,
            max_acceleration: self.max_acceleration,
            jerk: self.effective_jerk(),
            extrude_factor: self.extrude_factor,
            kind: if !self.wiping && e_only && e_delta > 0.0 {
                MotionKind::Unretract
            } else {
                MotionKind::Regular
            },
            e_only,
        })
    }
}
