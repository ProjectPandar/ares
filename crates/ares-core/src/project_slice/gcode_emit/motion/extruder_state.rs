//! Active extrusion counters from `Extruder.hpp` / `Extruder.cpp`.
//! Position, motion, wipe paths and writer caches remain in `EmitState`.

#[derive(Default)]
pub(in crate::project_slice::gcode_emit) struct ExtruderState {
    pub(in crate::project_slice::gcode_emit) e_position: f64,
    /// SEMM templates access private `m_E` and `m_retracted`, while motion
    /// accesses the physical extruder's shared slots (`Extruder.hpp:36-58`).
    pub(in crate::project_slice::gcode_emit) private_e_position: f64,
    pub(in crate::project_slice::gcode_emit) private_retracted: f64,
    /// Active-writer adapter flag; numeric retraction stays authoritative.
    pub(in crate::project_slice::gcode_emit) retracted: bool,
    /// Signed `m_absolute_E`; templates do not advance this tachometer.
    pub(in crate::project_slice::gcode_emit) filament_used: f64,
    pub(in crate::project_slice::gcode_emit) retracted_amount: f64,
    /// `m_restart_extra` belongs to the filament even with shared E state.
    pub(in crate::project_slice::gcode_emit) restart_extra: f64,
}
