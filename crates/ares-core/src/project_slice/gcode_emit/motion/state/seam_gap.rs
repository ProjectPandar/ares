use super::EmitState;

impl EmitState {
    /// GCode.cpp:5796 resolves the retained region value at loop emission.
    pub(in crate::project_slice::gcode_emit) fn current_seam_gap_mm(&self) -> f64 {
        match self.current_seam_gap {
            crate::FloatOrPercent::Float(value) => value,
            crate::FloatOrPercent::Percent(value) => self.options.nozzle_diameter * value.0 / 100.0,
        }
    }
}
