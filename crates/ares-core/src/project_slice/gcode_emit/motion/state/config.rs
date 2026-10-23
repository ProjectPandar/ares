use super::EmitState;

impl EmitState {
    /// GCode.cpp:4594/5342 applies the object-scoped part of current m_config.
    pub(in crate::project_slice::gcode_emit) fn apply_object_config(
        &mut self,
        object: &crate::ObjectOptions,
    ) {
        self.current_brim_flow_ratio = object.brim_flow_ratio.0;
        self.current_support_speed = object.support_speed.0;
    }

    /// GCode.cpp:5796 resolves the retained region value at loop emission.
    pub(in crate::project_slice::gcode_emit) fn current_seam_gap_mm(&self) -> f64 {
        match self.current_seam_gap {
            crate::FloatOrPercent::Float(value) => value,
            crate::FloatOrPercent::Percent(value) => self.options.nozzle_diameter * value.0 / 100.0,
        }
    }
}
