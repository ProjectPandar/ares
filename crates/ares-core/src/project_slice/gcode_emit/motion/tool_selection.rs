//! Writer tool installation and command (`GCodeWriter.cpp:547–585`).

use super::{EmitState, MotionOptions};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(in crate::project_slice::gcode_emit) fn select(
    state: &mut EmitState,
    traversal: &PreparedPostClassicTraversal,
    filament: usize,
) {
    state.extruders.activate(
        filament,
        &mut state.extruder,
        state.options.single_extruder_multi_material,
    );
    state.options = MotionOptions::from_traversal(traversal, filament);
    state.extruder_offset = super::super::offset::extruder(traversal, filament);
    state.offset = (
        state.origin.0 - state.extruder_offset.0,
        state.origin.1 - state.extruder_offset.1,
    );
    state.extruder_selected = true;
    let config = &traversal.resolved.views.full;
    state.current_hotend = if config.printer.remaining.printer_model.0 == "Bambu Lab X2D" {
        -1
    } else {
        config
            .project
            .gcode
            .filament_map
            .0
            .get(filament)
            .map_or(0, |value| value.0 - 1)
    };
}

pub(in crate::project_slice::gcode_emit) fn toolchange(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    traversal: &PreparedPostClassicTraversal,
    filament: usize,
) {
    select(state, traversal, filament);
    let runtime = &traversal.resolved.views.runtime_gcode;
    let is_bbl = super::super::tags::Tags::of(traversal).is_bbl();
    if !state.extruders.multiple() && (runtime.filament_diameter.0.len() <= 1 || is_bbl) {
        return;
    }
    let prefix = if runtime.manual_filament_change.0 {
        "; MANUAL_TOOL_CHANGE T"
    } else if is_bbl {
        "M1020 S"
    } else {
        match runtime.gcode_flavor {
            crate::GCodeFlavor::MakerWare => "M135 T",
            crate::GCodeFlavor::Sailfish => "M108 T",
            _ => "T",
        }
    };
    let comment = if state.options.gcode_comments {
        " ; change extruder"
    } else {
        ""
    };
    output.extend_from_slice(format!("{prefix}{filament}{comment}\n").as_bytes());
    // Forced reset_E clears private and shared positions, but not signed usage
    // or retraction (`Extruder.hpp:37`, `GCodeWriter.cpp:480–503`).
    if runtime.gcode_flavor.resets_absolute_e() {
        state.extruder.e_position = 0.0;
        state.extruder.private_e_position = 0.0;
        if !runtime.use_relative_e_distances.0 {
            let comment = if state.options.gcode_comments {
                " ; reset extrusion distance"
            } else {
                ""
            };
            output.extend_from_slice(format!("G92 E0{comment}\n").as_bytes());
        }
    }
}
