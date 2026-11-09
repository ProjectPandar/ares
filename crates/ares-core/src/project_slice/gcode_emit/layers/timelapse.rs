//! Traditional-timelapse gating (`GCode.cpp:5455-5461` vs `:5527-5546`).

use crate::project_slice::{
    gcode_emit::{motion::EmitState, tags::Tags},
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(super) struct TimelapseGate {
    pub(super) traditional_timelapse: bool,
    pub(super) traditional_interlude: bool,
}

pub(super) fn gate(
    traversal: &PreparedPostClassicTraversal,
    state: &mut EmitState,
) -> TimelapseGate {
    let runtime_gcode = &traversal.resolved.views.runtime_gcode;
    // `!m_wipe_tower || !enable_timelapse_print()` (`GCode.cpp:6690`): a tower printing a smooth
    // timelapse keeps the layer-change paths.
    let smooth_tower = state.wipe_tower.is_some()
        && traversal.resolved.views.full.process.print.timelapse_type
            == crate::ProcessTimelapseType::Smooth;
    // `is_BBL_Printer() || !time_lapse_gcode.empty()` (`GCode.cpp:4649`):
    // BBL printers take the traditional path with an EMPTY timelapse
    // template too, so their object-end labels flush in the previous
    // layer's tail (`GCode.cpp:5544`) instead of the next layer's
    // change_layer block.
    let traditional_timelapse = !smooth_tower
        && (Tags::of(traversal).is_bbl() || !runtime_gcode.time_lapse_gcode.0.is_empty())
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
