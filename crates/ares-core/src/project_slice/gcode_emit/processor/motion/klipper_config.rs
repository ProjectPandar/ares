//! `GCodeProcessor.cpp:383-387,6870-6916`: live Klipper toolhead limits.
use super::MotionState;
use regex::Regex;
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit::processor) struct KlipperConfig {
    pub(in crate::project_slice::gcode_emit::processor) minimum_cruise_ratio: f32,
    pub(in crate::project_slice::gcode_emit::processor) requested_accel_to_decel: f32,
}

impl Default for KlipperConfig {
    fn default() -> Self {
        Self {
            minimum_cruise_ratio: 0.5,
            requested_accel_to_decel: -1.0,
        }
    }
}

impl KlipperConfig {
    pub(in crate::project_slice::gcode_emit::processor) fn legacy(self) -> bool {
        self.requested_accel_to_decel >= 0.0
    }
}

impl MotionState {
    pub(super) fn klipper_velocity_limit(&mut self, code: &str) {
        static PARAMETERS: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"(?i-u)\s(VELOCITY|ACCEL|ACCEL_TO_DECEL|SQUARE_CORNER_VELOCITY|MINIMUM_CRUISE_RATIO)\s*=\s*([+]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?)")
                .expect("literal Klipper parameter expression")
        });
        let parameters = PARAMETERS
            .captures_iter(code)
            .filter_map(|capture| {
                let end = capture.get(0).unwrap().end();
                if code
                    .as_bytes()
                    .get(end)
                    .is_some_and(|b| !b.is_ascii_whitespace())
                {
                    return None;
                }
                let value = capture[2].parse::<f32>().ok()?;
                value
                    .is_finite()
                    .then(|| (capture[1].to_ascii_uppercase(), value))
            })
            .collect::<Vec<_>>();
        let has_ratio = parameters
            .iter()
            .any(|(name, value)| name == "MINIMUM_CRUISE_RATIO" && (0.0..1.0).contains(value));
        for (name, value) in parameters {
            match name.as_str() {
                "ACCEL" if value > 0.0 => {
                    self.acceleration = f64::from(value);
                    self.travel_acceleration = f64::from(value);
                }
                "MINIMUM_CRUISE_RATIO" if (0.0..1.0).contains(&value) => {
                    self.klipper.minimum_cruise_ratio = value;
                    self.klipper.requested_accel_to_decel = -1.0;
                }
                "ACCEL_TO_DECEL" if value > 0.0 && !has_ratio => {
                    self.klipper.requested_accel_to_decel = value;
                }
                "SQUARE_CORNER_VELOCITY" if value >= 0.0 => {
                    self.jerk[0] = f64::from(value);
                    self.jerk[1] = f64::from(value);
                }
                "VELOCITY" if value > 0.0 => {
                    self.max_feedrate[0] = f64::from(value);
                    self.max_feedrate[1] = f64::from(value);
                }
                _ => {}
            }
        }
    }
}
