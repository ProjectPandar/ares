pub(super) mod arc;

mod cached_feedrate;
mod config_commands;
mod firmware;
mod homing;
mod klipper_config;
mod parse;
pub(super) use klipper_config::KlipperConfig;
mod segment;

mod limits;
use super::motion_util::word;
use crate::options::GCodeFlavor;
mod planner;
pub(super) use planner::{Planner, Trapezoid};

pub(super) struct MotionState {
    pub(super) klipper: KlipperConfig,
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
            klipper: KlipperConfig::default(),
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
    pub(super) klipper: KlipperConfig,
    pub(super) virtual_motion: bool,
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
}
