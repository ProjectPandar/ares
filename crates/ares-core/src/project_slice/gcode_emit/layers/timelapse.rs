//! Traditional-timelapse gating (`GCode.cpp:5455-5461` vs `:5527-5546`).

use crate::project_slice::gcode_emit::motion::EmitState;
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) struct TimelapseGate {
    pub(super) traditional_timelapse: bool,
    pub(super) traditional_interlude: bool,
}

pub(super) fn gate(
    traversal: &PreparedPostClassicTraversal,
    state: &mut EmitState,
) -> TimelapseGate {
    let runtime_gcode = &traversal.resolved.views.runtime_gcode;
    let traditional_timelapse = !runtime_gcode.time_lapse_gcode.0.is_empty()
        && ((runtime_gcode.printer_structure == crate::PrinterStructure::I3
            && !traversal.resolved.views.full.process.print.spiral_mode.0)
            || traversal
                .resolved
                .views
                .full
                .project
                .print
                .nozzle_diameter
                .0
                .len()
                > 1);
    // The mid-layer insert only fires on I3 printers (`GCode.cpp:5455-5461`);
    // corexy/multi-nozzle traditional prints fall through to the layer-end
    // sequence (`GCode.cpp:5527-5546`).
    let traditional_interlude =
        traditional_timelapse && runtime_gcode.printer_structure == crate::PrinterStructure::I3;
    state.traditional_timelapse = traditional_timelapse;
    TimelapseGate {
        traditional_timelapse,
        traditional_interlude,
    }
}
