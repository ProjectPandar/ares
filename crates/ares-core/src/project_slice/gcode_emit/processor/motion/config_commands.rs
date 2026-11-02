//! Non-motion machine config commands (`GCodeProcessor.cpp:5317+`,
//! `5269-5304`, M201-M221/M566) split from `motion.rs` to keep each source
//! file under the LOC cap. Each handler mutates `MotionState` and reports
//! whether the line was consumed (no motion block produced).
use super::MotionState;
use crate::project_slice::gcode_emit::processor::motion_util::{
    assignment, clamp, clamped_word, word,
};

impl MotionState {
    /// Returns true when `code` was a consumed config command (M201, M203,
    /// M204, M205, M221, M566, SET_VELOCITY_LIMIT) producing no motion block.
    pub(super) fn handle_config_command(&mut self, code: &str) -> bool {
        if code.starts_with("M201") {
            self.update_axis_limits(code, true);
            return true;
        }
        if code.starts_with("M203") {
            self.update_axis_limits(code, false);
            return true;
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
            return true;
        }
        if code.starts_with("M205") {
            // `GCodeProcessor.cpp:5241–5249`: X writes both XY axes,
            // then an explicit Y overrides only Y.
            if let Some(value) = word(code, 'X') {
                self.jerk[0] = value;
                self.jerk[1] = value;
            }
            for (axis, letter) in [(1, 'Y'), (2, 'Z'), (3, 'E')] {
                self.jerk[axis] = word(code, letter).unwrap_or(self.jerk[axis]);
            }
            for (limit, letter) in [
                (&mut self.min_extruding_rate, 'S'),
                (&mut self.min_travel_rate, 'T'),
                (&mut self.junction_deviation, 'J'),
            ] {
                *limit = word(code, letter).unwrap_or(*limit);
            }
            return true;
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
            return true;
        }
        if code.starts_with("M566") {
            self.update_jerk_limits(code);
            return true;
        }
        if code.split_whitespace().next() == Some("SET_VELOCITY_LIMIT") {
            if self.gcode_flavor == crate::GCodeFlavor::Klipper {
                self.klipper_velocity_limit(code);
                return true;
            }
            // Klipper: ACCEL applies to print and travel moves alike; each
            // setter clamps the raw value to its own limit
            // (`GCodeProcessor.cpp:5287-5298`), and Klipper's travel limit is
            // 0 (`supports_separate_travel_acceleration`), i.e. unclamped.
            if let Some(value) = assignment(code, "ACCEL=") {
                self.acceleration = clamp(value, self.max_print_acceleration);
                self.travel_acceleration = clamp(value, self.max_travel_acceleration);
            }
            if let Some(value) = assignment(code, "SQUARE_CORNER_VELOCITY=") {
                self.jerk[0] = value;
                self.jerk[1] = value;
            }
            // Upstream also updates machine_max_speed_x/y from `VELOCITY=`
            // (`GCodeProcessor.cpp:5301-5313`); Ares' planner has no
            // per-axis speed limit yet, so the field is not tracked.
            return true;
        }
        false
    }
}
