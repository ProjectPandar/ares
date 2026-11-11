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
    state.current_hotend = if config
        .printer
        .remaining
        .printer_model
        .0
        .starts_with("Bambu Lab")
    {
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
    custom: &str,
) {
    select(state, traversal, filament);
    let runtime = &traversal.resolved.views.runtime_gcode;
    let is_bbl = super::super::tags::Tags::of(traversal).is_bbl();
    if !state.extruders.multiple() && (runtime.filament_diameter.0.len() <= 1 || is_bbl) {
        return;
    }
    let prefix = prefix(traversal);
    let comment = if state.options.gcode_comments {
        " ; change extruder"
    } else {
        ""
    };
    let supplied = custom_changes_tool(custom, prefix, filament);
    if !supplied {
        let command = if is_bbl && !runtime.manual_filament_change.0 {
            format!(
                "M1020 S{filament} H{}",
                nozzle(&traversal.resolved.views.full, filament)
            )
        } else {
            format!("{prefix}{filament}")
        };
        output.extend_from_slice(format!("{command}{comment}\n").as_bytes());
    }
    // Forced reset_E clears private and shared positions, but not signed usage
    // or retraction (`Extruder.hpp:37`, `GCodeWriter.cpp:480–503`).
    if runtime.gcode_flavor.resets_absolute_e() {
        state.extruder.e_position = 0.0;
        state.extruder.private_e_position = 0.0;
        if !runtime.use_relative_e_distances.0 && !supplied {
            let comment = if state.options.gcode_comments {
                " ; reset extrusion distance"
            } else {
                ""
            };
            output.extend_from_slice(format!("G92 E0{comment}\n").as_bytes());
        }
    }
}

/// `GCodeWriter::toolchange_prefix` (`GCodeWriter.cpp:793-800`), also the cooling buffer's
/// tool-change prefix (`CoolingBuffer.cpp:40`).
pub(in crate::project_slice::gcode_emit) fn prefix(
    traversal: &PreparedPostClassicTraversal,
) -> &'static str {
    let runtime = &traversal.resolved.views.runtime_gcode;
    if runtime.manual_filament_change.0 {
        "; MANUAL_TOOL_CHANGE T"
    } else {
        match runtime.gcode_flavor {
            crate::GCodeFlavor::MakerWare => "M135 T",
            crate::GCodeFlavor::Sailfish => "M108 T",
            _ => "T",
        }
    }
}

/// `nozzle_id_for_gcode_placeholder` (`GCode.cpp:241-247`) over the static
/// grouping result, published as global physical IDs in `filament_nozzle_map`.
pub(in crate::project_slice::gcode_emit) fn nozzle(
    settings: &crate::ProjectSettings,
    filament: usize,
) -> i64 {
    settings
        .project
        .gcode
        .filament_nozzle_map
        .0
        .get(filament)
        .map_or(-1, |id| i64::from(id.0))
}

/// Source scanner keeps the last parseable, line-leading prefix result
/// (`GCode.cpp:241–264`); parsing begins one byte after the prefix's start.
fn custom_changes_tool(source: &str, prefix: &str, filament: usize) -> bool {
    let mut selected = false;
    for line in source.lines() {
        let line = line.trim_start_matches(|c: char| c.is_ascii_whitespace());
        if !line.starts_with(prefix) {
            continue;
        }
        let number = line[1..].trim_start_matches(|c: char| c.is_ascii_whitespace());
        let digits = number.strip_prefix(['+', '-']).unwrap_or(number);
        let end = digits
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(digits.len());
        let Ok(tool) = digits[..end].parse::<u32>() else {
            continue;
        };
        let tool = if number.starts_with('-') {
            0_u32.wrapping_sub(tool)
        } else {
            tool
        };
        selected = tool as usize == filament;
    }
    selected
}
