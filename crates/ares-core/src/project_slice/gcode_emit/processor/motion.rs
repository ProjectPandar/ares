pub(super) mod arc;
#[cfg(test)]
mod axis_acceleration_tests;
mod cached_feedrate;
mod config_commands;
mod firmware;
mod homing;
mod segment;
#[cfg(test)]
mod test_hooks;
#[cfg(test)]
pub(in crate::project_slice) use test_hooks::{planned_times, planned_trapezoid_time};
mod limits;
use super::motion_util::{MMMIN_TO_MMSEC, norm, word};
use crate::options::GCodeFlavor;
mod planner;
pub(super) use planner::{RollingPlanner, Trapezoid};
use std::f64::consts::PI;

pub(super) struct MotionState {
    pub(super) position: [f64; 3],
    pub(super) e_position: f64,
    pub(super) units: f32,
    pub(super) origin: [f64; 4],
    pub(super) feedrate: f64,
    pub(super) cached_feedrate: f32,
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
    pub(super) min_extruding_rate: f64,
    pub(super) min_travel_rate: f64,
    /// `M221` extrude factor override (`TimeMachine::
    /// extrude_factor_override_percentage`), applied to the E-axis
    /// feedrate before the safe/jerk limits (`GCodeProcessor.cpp:4040`).
    pub(super) extrude_factor: f64,
    pub(super) relative: bool,
    pub(super) e_relative: bool,
    pub(super) wiping: bool,
    pub(super) gcode_flavor: GCodeFlavor,
    /// Firmware-retraction emulation values for G10/G11
    /// (`GCodeProcessor.cpp:4873-4893`).
    pub(super) retraction_length: f64,
    pub(super) retract_restart_extra: f64,
    pub(super) retraction_feedrate: f64,
    pub(super) deretraction_feedrate: f64,
}

impl Default for MotionState {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            e_position: 0.0,
            units: 1.0,
            origin: [0.0; 4],
            feedrate: 0.0,
            // GCodeProcessor.cpp:233-237/2506: reset the cached-speed sentinel.
            cached_feedrate: f32::MAX,
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
            min_extruding_rate: 0.0,
            min_travel_rate: 0.0,
            extrude_factor: 1.0,
            relative: false,
            e_relative: false,
            gcode_flavor: GCodeFlavor::MarlinLegacy,
            wiping: false,
            retraction_length: 0.0,
            retract_restart_extra: 0.0,
            retraction_feedrate: 0.0,
            deretraction_feedrate: 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MotionKind {
    Regular,
    Unretract,
    ToolChange,
}

pub(super) struct IndexedMotion {
    pub(super) index: usize,
    pub(super) block: MotionBlock,
    /// Absolute XYZE after the move (upstream `m_end_position`).
    pub(super) end: [f64; 4],
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

    #[cfg(test)]
    pub(super) fn motions(&mut self, code: &str) -> Vec<MotionBlock> {
        self.motions_indexed(code)
            .into_iter()
            .map(|motion| motion.block)
            .collect()
    }

    /// `index` is `0` for linear moves and the segment's position in
    /// `arc::targets` for `G2`/`G3` — needed to preserve the
    /// `m_g1_line_id` hole of dropped zero-displacement segments
    /// (`GCodeProcessor.cpp:3868` fires before the :3891 drop).
    pub(super) fn motions_indexed(&mut self, code: &str) -> Vec<IndexedMotion> {
        let start = self.position;
        let start_e = self.e_position;
        let Some(block) = self.motion(code) else {
            return Vec::new();
        };
        let linear = |state: &Self, block| {
            let [x, y, z] = state.position;
            vec![IndexedMotion {
                index: 0,
                block,
                end: [x, y, z, state.e_position],
            }]
        };
        let command = code.split_whitespace().next().unwrap_or_default();
        if !matches!(command, "G2" | "G3") {
            return linear(self, block);
        }
        let Some(targets) = arc::targets(
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
            return linear(self, block);
        };
        // `internal_only_g1_line` → `process_G1` → `++m_g1_line_id` fires for
        // EVERY discretized segment (`GCodeProcessor.cpp:3868`), including
        // zero-displacement segments dropped at :3891 — so a retained
        // block's `g1_line_id` is its discretization position `base + index`,
        // not a compact rank over the survivors. Carry the `targets` index.
        let mut previous = [start[0], start[1], start[2], start_e];
        targets
            .into_iter()
            .enumerate()
            .filter_map(|(index, end)| {
                let delta = std::array::from_fn(|axis| end[axis] - previous[axis]);
                previous = end;
                self.segment_block(delta)
                    .map(|block| IndexedMotion { index, block, end })
            })
            .collect()
    }

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
            let value = f64::from(value as f32) * f64::from(self.units);
            next[axis] = match self.relative {
                true => old[axis] + value,
                false => self.origin[axis] + value,
            };
        }
        let old_e = self.e_position;
        let next_e =
            match word(code, 'E').map(|value| f64::from(value as f32) * f64::from(self.units)) {
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
