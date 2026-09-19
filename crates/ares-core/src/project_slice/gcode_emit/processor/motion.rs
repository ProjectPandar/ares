pub(super) mod arc;
#[cfg(test)]
mod axis_acceleration_tests;
mod segment;
mod test_hooks;
#[cfg(test)]
pub(in crate::project_slice) use test_hooks::{planned_times, planned_trapezoid_time};
mod limits;
use super::motion_util::{MMMIN_TO_MMSEC, assignment, clamp, clamped_word, norm, scale, word};
use crate::options::GCodeFlavor;
mod planner;
pub(super) use planner::RollingPlanner;
use std::f64::consts::PI;

pub(super) struct MotionState {
    pub(super) position: [f64; 3],
    pub(super) e_position: f64,
    pub(super) feedrate: f64,
    pub(super) acceleration: f64,
    pub(super) retract_acceleration: f64,
    pub(super) travel_acceleration: f64,
    pub(super) max_print_acceleration: f64,
    pub(super) max_retract_acceleration: f64,
    pub(super) max_travel_acceleration: f64,
    pub(super) max_acceleration: [f64; 4],
    pub(super) max_feedrate: [f64; 4],
    pub(super) jerk: [f64; 4],
    pub(super) junction_deviation: f64,
    /// `M221` extrude factor override (`TimeMachine::
    /// extrude_factor_override_percentage`), applied to the E-axis
    /// feedrate before the safe/jerk limits (`GCodeProcessor.cpp:4040`).
    pub(super) extrude_factor: f64,
    pub(super) relative: bool,
    pub(super) e_relative: bool,
    pub(super) wiping: bool,
    pub(super) gcode_flavor: GCodeFlavor,
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
            max_print_acceleration: 0.0,
            max_retract_acceleration: 0.0,
            max_travel_acceleration: 0.0,
            max_acceleration: [0.0; 4],
            max_feedrate: [0.0; 4],
            jerk: [9.0, 9.0, 3.0, 2.5],
            junction_deviation: 0.0,
            extrude_factor: 1.0,
            relative: false,
            e_relative: false,
            gcode_flavor: GCodeFlavor::MarlinLegacy,
            wiping: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MotionKind {
    Regular,
    Unretract,
    ToolChange,
}

pub(super) struct MotionBlock {
    pub(super) distance: f64,
    /// Raw axis deltas (upstream `AxisCoords delta_pos` doubles,
    /// `GCodeProcessor.hpp:392`).
    pub(super) delta: [f64; 4],
    pub(super) speed: f64,
    pub(super) acceleration: f64,
    pub(super) centripetal_acceleration: f64,
    pub(super) max_feedrate: [f64; 4],
    pub(super) max_acceleration: [f64; 4],
    pub(super) jerk: [f64; 4],
    /// `M221` extrude factor override carried per block so the planner
    /// can scale the E-axis feedrate like `GCodeProcessor.cpp:4040`.
    pub(super) extrude_factor: f64,
    /// Whether XYZ displacement is zero; cache eligibility also depends on
    /// the upstream move classification and inserted seam vertex.
    pub(super) e_only: bool,
    pub(super) kind: MotionKind,
}

impl MotionState {
    /// `GCodeProcessor.cpp:2110-2122`: the current accelerations are
    /// seeded from the machine limits at configure time (falling back to
    /// the upstream Prusa defaults when a limit is unset), not zero.
    pub(super) fn with_acceleration_limits(print: f64, retract: f64, travel: f64) -> Self {
        Self {
            acceleration: if print > 0.0 { print } else { 1500.0 },
            retract_acceleration: if retract > 0.0 { retract } else { 1500.0 },
            travel_acceleration: if travel > 0.0 { travel } else { 1250.0 },
            max_print_acceleration: print,
            max_retract_acceleration: retract,
            max_travel_acceleration: travel,
            ..Self::default()
        }
    }

    pub(super) fn set_wiping(&mut self, wiping: bool) {
        self.wiping = wiping;
    }

    pub(super) fn motions(&mut self, code: &str) -> Vec<MotionBlock> {
        let start = self.position;
        let start_e = self.e_position;
        let Some(block) = self.motion(code) else {
            return Vec::new();
        };
        let command = code.split_whitespace().next().unwrap_or_default();
        if !matches!(command, "G2" | "G3") {
            return vec![block];
        }
        let Some(deltas) = arc::deltas(
            command,
            code,
            arc::ArcMotion {
                start,
                end: self.position,
                start_e,
                e_delta: self.e_position - start_e,
                feedrate: self.feedrate,
                gcode_flavor: self.gcode_flavor,
            },
        ) else {
            return vec![block];
        };
        deltas
            .into_iter()
            .filter_map(|delta| self.segment_block(delta))
            .collect()
    }

    pub(super) fn motion(&mut self, code: &str) -> Option<MotionBlock> {
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
        if code.starts_with("M201") {
            self.update_axis_limits(code, true);
            return None;
        }
        if code.starts_with("M203") {
            self.update_axis_limits(code, false);
            return None;
        }
        if code.starts_with("M204") {
            if let Some(value) = word(code, 'S') {
                self.acceleration = clamp(value, self.max_print_acceleration);
                self.travel_acceleration = clamp(value, self.max_travel_acceleration);
                self.retract_acceleration = clamped_word(
                    code,
                    'T',
                    self.retract_acceleration,
                    self.max_retract_acceleration,
                );
            } else {
                self.acceleration =
                    clamped_word(code, 'P', self.acceleration, self.max_print_acceleration);
                self.retract_acceleration = clamped_word(
                    code,
                    'R',
                    self.retract_acceleration,
                    self.max_retract_acceleration,
                );
                self.travel_acceleration = clamped_word(
                    code,
                    'T',
                    self.travel_acceleration,
                    self.max_travel_acceleration,
                );
            }
            return None;
        }
        if code.starts_with("M205") {
            for (axis, letter) in ['X', 'Y', 'Z', 'E'].into_iter().enumerate() {
                self.jerk[axis] = word(code, letter).unwrap_or(self.jerk[axis]);
            }
            return None;
        }
        if code.starts_with("M221") {
            // `GCodeProcessor::process_M221` (`GCodeProcessor.cpp:5317-
            // 5326`): an S word without a T word rescales the E-axis
            // feedrate. Upstream's parser reads a BARE `S` word as 0
            // (empirically verified: BBL start gcode `M221 S;` zeroes the
            // factor for the rest of the stream).
            if code.contains('S') && !code.contains('T') {
                self.extrude_factor = word(code, 'S').unwrap_or(0.0) * 0.01;
            }
            return None;
        }
        if code.starts_with("M566") {
            self.update_jerk_limits(code);
            return None;
        }
        if code.split_whitespace().next() == Some("SET_VELOCITY_LIMIT") {
            // Klipper: ACCEL applies to print and travel moves alike
            // (`GCodeProcessor.cpp:5269-5304`).
            if let Some(value) = assignment(code, "ACCEL=") {
                let value = clamp(value, self.max_print_acceleration);
                self.acceleration = value;
                self.travel_acceleration = clamp(value, self.max_travel_acceleration);
            }
            if let Some(value) = assignment(code, "SQUARE_CORNER_VELOCITY=") {
                self.jerk[0] = value;
                self.jerk[1] = value;
            }
            // Upstream also updates machine_max_speed_x/y from `VELOCITY=`
            // (`GCodeProcessor.cpp:5301-5313`); Ares' planner has no
            // per-axis speed limit yet, so the field is not tracked.
            return None;
        }
        let command = code.split_whitespace().next()?;
        if command == "G28" {
            let has_axis = ['X', 'Y', 'Z'].iter().any(|&axis| code.contains(axis));
            let mut homing = String::from("G1");
            let axes = ['X', 'Y', 'Z']
                .into_iter()
                .filter(|axis| !has_axis || code.contains(*axis));
            for axis in axes {
                homing.push(' ');
                homing.push(axis);
                homing.push('0');
            }
            return self.motion(&homing);
        }
        if !matches!(command, "G0" | "G1" | "G2" | "G3") {
            return None;
        }
        if let Some(value) = word(code, 'F') {
            self.feedrate = f64::from(value as f32 * MMMIN_TO_MMSEC);
        }
        let old = self.position;
        let mut next = old;
        for (axis, letter) in ['X', 'Y', 'Z'].into_iter().enumerate() {
            let Some(value) = word(code, letter) else {
                continue;
            };
            // `GCodeReader::GCodeLine::x() const { return m_axis[X]; }`
            // returns FLOAT (`GCodeReader.hpp:70`): every axis word is
            // f32-quantized before the position/delta math — the text
            // 152.193 becomes 152.1929931640625, and the whole estimator
            // chain (distances, axis feedrates, cruises) follows.
            let value = f64::from(value as f32);
            next[axis] = match self.relative {
                true => old[axis] + value,
                false => value,
            };
        }
        let old_e = self.e_position;
        let (e_delta, next_e) = match word(code, 'E').map(|value| f64::from(value as f32)) {
            Some(value) if self.e_relative => (value, old_e + value),
            Some(value) => (value - old_e, value),
            None => (0.0, old_e),
        };
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
        let mut distance = if e_only { e_delta.abs() } else { xyz_distance };
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
        // (`GCodeProcessor.cpp:3993-4080`); the raw F travels here.
        Some(MotionBlock {
            distance,
            delta,
            speed: self.feedrate,
            acceleration,
            centripetal_acceleration: self.acceleration.max(1.0),
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
