//! `GCodeProcessor` axis limits, junction deviation and logical position.

use super::super::motion_util::MMMIN_TO_MMSEC;
use super::{GCodeFlavor, MotionState, word};

impl MotionState {
    pub(in crate::project_slice::gcode_emit::processor) fn with_limits(
        limits: crate::project_slice::gcode_emit::processor::ProcessorLimits,
    ) -> Self {
        let travel = if limits.gcode_flavor.supports_separate_travel_acceleration() {
            limits.travel_acceleration
        } else {
            0.0
        };
        Self {
            gcode_flavor: limits.gcode_flavor,
            junction_deviation: limits.junction_deviation,
            min_extruding_rate: limits.min_extruding_rate,
            min_travel_rate: limits.min_travel_rate,
            max_feedrate: limits.max_feedrate,
            max_acceleration: limits.max_acceleration,
            jerk: limits.jerk,
            retraction_length: limits.retraction_length,
            retract_restart_extra: limits.retract_restart_extra,
            retraction_feedrate: limits.retraction_feedrate,
            deretraction_feedrate: limits.deretraction_feedrate,
            ..Self::with_acceleration_limits(
                limits.print_acceleration,
                limits.retract_acceleration,
                travel,
            )
        }
    }

    /// `GCodeProcessor.cpp:3987–3989,4347–4349,5726–5740`: zero E delta
    /// selects the travel floor; retracts and E-only moves use extrusion.
    pub(super) fn block_feedrate(&self, e_delta: f64) -> f64 {
        self.feedrate.max(if e_delta == 0.0 {
            self.min_travel_rate
        } else {
            self.min_extruding_rate
        })
    }

    /// `get_axis_max_jerk` (`GCodeProcessor.cpp:5798-5804`): for Marlin
    /// firmware with a positive `machine_max_junction_deviation`, the f32
    /// `get_axis_max_jerk_with_jd(mode, axis, get_acceleration(mode))`
    /// (`:5771-5791`); classic M205 jerks apply otherwise.
    pub(super) fn effective_jerk(&self) -> [f64; 4] {
        let jd = self.junction_deviation as f32;
        if self.gcode_flavor != GCodeFlavor::MarlinFirmware || jd <= 0.0 {
            return self.jerk;
        }
        // The `<= 0` fallback to `get_acceleration(mode)` re-reads the same
        // machine acceleration on this path.
        let acceleration = self.acceleration as f32;
        self.max_acceleration.map(|axis_max| {
            let axis_max = axis_max as f32;
            let effective = match (axis_max > 0.0, acceleration > 0.0) {
                (true, true) => acceleration.min(axis_max),
                (true, false) => axis_max,
                (false, _) => acceleration,
            };
            if effective <= 0.0 {
                0.0
            } else {
                f64::from((jd * effective * 2.5).sqrt())
            }
        })
    }

    /// `GCodeProcessor.cpp:5160-5201` (M201) and `:5179-5201` (M203):
    /// acceleration values apply raw; feedrate values apply raw only for
    /// Marlin/Smoothie/Klipper and are converted from mm/min otherwise
    /// (M203 is ignored entirely for Repetier).
    pub(super) fn update_axis_limits(&mut self, code: &str, acceleration: bool) {
        if !acceleration && self.gcode_flavor == GCodeFlavor::Repetier {
            return;
        }
        let factor = if acceleration
            || matches!(
                self.gcode_flavor,
                GCodeFlavor::MarlinLegacy
                    | GCodeFlavor::MarlinFirmware
                    | GCodeFlavor::Klipper
                    | GCodeFlavor::Smoothie
            ) {
            1.0
        } else {
            MMMIN_TO_MMSEC
        };
        let limits = if acceleration {
            &mut self.max_acceleration
        } else {
            &mut self.max_feedrate
        };
        for (axis, letter) in ['X', 'Y', 'Z', 'E'].into_iter().enumerate() {
            limits[axis] =
                word(code, letter).map_or(limits[axis], |value| (value as f32 * factor) as f64);
        }
    }

    /// `GCodeProcessor.cpp:5400-5418`: M566 jerk limits are always
    /// specified in mm/min and converted to mm/s.
    pub(super) fn update_jerk_limits(&mut self, code: &str) {
        for (axis, letter) in ['X', 'Y', 'Z', 'E'].into_iter().enumerate() {
            self.jerk[axis] = word(code, letter).map_or(self.jerk[axis], |value| {
                (value as f32 * MMMIN_TO_MMSEC) as f64
            });
        }
    }

    pub(super) fn set_position(&mut self, code: &str) {
        let e = word(code, 'E');
        let position = ['X', 'Y', 'Z'].map(|letter| word(code, letter));
        if e.is_none() && position.iter().all(Option::is_none) {
            self.origin = [
                self.position[0],
                self.position[1],
                self.position[2],
                self.e_position,
            ];
            return;
        }
        if let Some(value) = e {
            self.e_position = f64::from(value as f32 * self.units);
        }
        for (axis, value) in position.into_iter().enumerate() {
            if let Some(value) = value {
                self.origin[axis] = self.position[axis] - f64::from(value as f32 * self.units);
            }
        }
    }
}
