//! Repetier cached speed (`GCodeProcessor.cpp:5357-5398`).
use super::MotionState;
use crate::options::GCodeFlavor;

impl MotionState {
    pub(super) fn handle_cached_feedrate(&mut self, code: &str) -> bool {
        if self.gcode_flavor != GCodeFlavor::Repetier {
            return false;
        }
        match code.split_whitespace().next() {
            Some("M401") => self.cached_feedrate = self.feedrate as f32,
            Some("M402") => self.restore_cached_feedrate(),
            _ => return false,
        }
        true
    }

    fn restore_cached_feedrate(&mut self) {
        // :5393 uses has_value(char4), not 'F' (GCodeReader.hpp:35/cpp:292-315).
        // :5384/:5390 only change start; :2888 overwrites it before the next line.
        if self.cached_feedrate != f32::MAX {
            self.feedrate = f64::from(self.cached_feedrate);
        }
    }
}
