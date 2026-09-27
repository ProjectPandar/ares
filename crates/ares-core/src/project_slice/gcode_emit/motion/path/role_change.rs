//! Custom extrusion-role-change gcode (`GCode.cpp:6745-6770`): at every
//! `path.role()` transition, render `change_extrusion_role_gcode`,
//! `filament_change_extrusion_role_gcode`, then
//! `process_change_extrusion_role_gcode` with `extrusion_role`,
//! `last_extrusion_role`, `layer_num = m_layer_index + 1`, and
//! `layer_z = print_z` (or `m_last_height` when no layer is active).

use super::super::EmitState;
use crate::project_slice::gcode_emit::{template, value::Value};

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
    config.insert("layer_num", Value::number((state.layer_index + 1) as f64));
    config.insert("layer_z", Value::number(state.source_layer_z));
    for template_source in [
        &state.options.role_change_machine,
        &state.options.role_change_filament,
        &state.options.role_change_process,
    ] {
        if template_source.is_empty() {
            continue;
        }
        if let Ok(rendered) = template::render(template_source, &mut config) {
            output.extend_from_slice(rendered.as_bytes());
            output.push(b'\n');
        }
    }
}
