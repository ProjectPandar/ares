//! Custom extrusion-role-change gcode (`GCode.cpp:6745-6770`): at every
//! `path.role()` transition, render `change_extrusion_role_gcode`,
//! `filament_change_extrusion_role_gcode`, then
//! `process_change_extrusion_role_gcode` with `extrusion_role`,
//! `last_extrusion_role`, `layer_num = m_layer_index + 1`, and
//! `layer_z = print_z` (or `m_last_height` when no layer is active).
//!
//! Like every upstream custom-gcode hook, the render runs through
//! `placeholder_parser_process` (`GCode.cpp:3894-3935`): live writer
//! state is refreshed before each template (position/zhop/E counters),
//! writer-visible assignments are applied back after
//! (`GCode.cpp:3903-3916`), and a render failure records the template
//! name so export aborts at the end (`GCode.cpp:3637-3645`).

use super::super::EmitState;
use crate::project_slice::gcode_emit::{template, value::Value};

use super::super::writer_state::{apply_writer_state, refresh_writer_state};

/// `extrusion_role_to_string_for_parser` (`GCode.cpp:7264-7290`) — keyed by
/// the `ExtrusionEntity` role id used for `;_EXTRUSION_ROLE:` markers.
fn role_name(role: u32) -> &'static str {
    match role {
        1 => "Perimeter",
        2 => "ExternalPerimeter",
        3 => "OverhangPerimeter",
        4 => "InternalInfill",
        5 => "SolidInfill",
        6 => "TopSolidInfill",
        7 => "BottomSurface",
        9 | 10 => "BridgeInfill",
        11 => "GapFill",
        8 => "Ironing",
        12 => "Skirt",
        13 => "Brim",
        14 => "SupportMaterial",
        15 => "SupportMaterialInterface",
        16 => "SupportTransition",
        17 => "WipeTower",
        _ => "Mixed",
    }
}

pub(super) fn emit(output: &mut Vec<u8>, role: u32, state: &mut EmitState) {
    let mut config = match state.role_change_config.clone() {
        Some(config) => config,
        None => return,
    };
    config.insert("extrusion_role", Value::String(role_name(role).to_owned()));
    config.insert(
        "last_extrusion_role",
        Value::String(
            state
                .last_extrusion_role
                .map_or("Mixed", role_name)
                .to_owned(),
        ),
    );
    // `ConfigOptionInt` upstream (`GCode.cpp:6758`) — integer semantics so
    // `{layer_num / 2}` divides as an integer.
    config.insert(
        "layer_num",
        Value::Integer((state.layer_chunk_index + 1) as i64),
    );
    config.insert("layer_z", Value::number(state.source_layer_z));
    let sources = [
        (
            "change_extrusion_role_gcode",
            state.options.role_change_machine.clone(),
        ),
        (
            "filament_change_extrusion_role_gcode",
            state.options.role_change_filament.clone(),
        ),
        (
            "process_change_extrusion_role_gcode",
            state.options.role_change_process.clone(),
        ),
    ];
    for (name, template_source) in sources {
        if template_source.is_empty() {
            continue;
        }
        refresh_writer_state(&mut config, state);
        match template::render(&template_source, &mut config) {
            Ok(rendered) => {
                if let Err(error) = apply_writer_state(name, &config, &rendered, state) {
                    // `GCode.cpp:3929-3932` verbatim: `err.what()` is
                    // concatenated with NO separating newline, and the
                    // caller's role-gcode loop adds a trailing `\n`.
                    output.extend_from_slice(
                        format!(
                            "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
                        )
                        .as_bytes(),
                    );
                    break;
                }
                output.extend_from_slice(rendered.as_bytes());
                output.push(b'\n');
            }
            // Upstream inserts the failure report inline and aborts the
            // export at the end (`GCode.cpp:3920-3933`, `:3637-3645`).
            Err(error) => {
                state.record_template_failure(name, &error.to_string());
                output.extend_from_slice(
                    format!(
                        "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
                    )
                    .as_bytes(),
                );
            }
        }
    }
}
