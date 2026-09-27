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
//! writer-visible assignments are applied back after, and a render
//! failure records the template name so export aborts at the end
//! (`GCode.cpp:3637-3645`).

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

/// `PlaceholderParserIntegration::update_from_gcodewriter`
/// (`GCode.cpp:1663-1697`): refresh writer-visible placeholders from the
/// emit state before each render.
fn refresh_writer_state(
    config: &mut crate::project_slice::gcode_emit::value::Config,
    state: &EmitState,
) {
    config.insert(
        "position",
        Value::List(vec![
            Value::number(state.x),
            Value::number(state.y),
            Value::number(state.writer_z.unwrap_or(state.source_layer_z)),
        ]),
    );
    config.insert("zhop", Value::number(state.lifted_amount));
    // Single-extruder slices expose the current extruder's counters.
    config.insert(
        "e_retracted",
        Value::List(vec![Value::number(state.retracted_amount)]),
    );
    config.insert("e_restart_extra", Value::List(vec![Value::number(0.0)]));
    config.insert(
        "extruded_volume",
        Value::List(vec![Value::number(state.filament_used)]),
    );
    config.insert("extruded_volume_total", Value::number(state.filament_used));
    if !state.options.use_relative_e_distances {
        config.insert(
            "e_position",
            Value::List(vec![Value::number(state.e_position)]),
        );
    }
}

/// Writer write-back (`GCode.cpp:3903-3915`): assigned `position` and
/// E counters propagate back into the writer so subsequent output uses
/// the overridden values.
fn apply_writer_state(
    config: &crate::project_slice::gcode_emit::value::Config,
    state: &mut EmitState,
) {
    if let Some(Value::List(position)) = config.get("position") {
        let numeric = |v: &Value| match v {
            Value::Number(n) => Some(*n),
            Value::Integer(n) => Some(*n as f64),
            _ => None,
        };
        if let [x, y, z] = position.as_slice()
            && let (Some(x), Some(y), Some(z)) = (numeric(x), numeric(y), numeric(z))
        {
            if (x - state.x).abs() > f64::EPSILON || (y - state.y).abs() > f64::EPSILON {
                // `m_writer.set_position` + `set_last_pos(gcode_to_point)`
                // (`GCode.cpp:3904-3907`).
                state.x = x;
                state.y = y;
                state.last_scaled_position = Some((
                    ((x - state.offset.0) / state.scale_factor) as i64,
                    ((y - state.offset.1) / state.scale_factor) as i64,
                ));
            }
            state.writer_z = Some(z);
        }
    }
    if !state.options.use_relative_e_distances
        && let Some(Value::List(list)) = config.get("e_position")
        && let Some(value) = list.first()
    {
        let position = match value {
            Value::Number(n) => *n,
            Value::Integer(n) => *n as f64,
            _ => return,
        };
        state.e_position = position;
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
    config.insert("layer_num", Value::Integer((state.layer_index + 1) as i64));
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
                apply_writer_state(&config, state);
                output.extend_from_slice(rendered.as_bytes());
                output.push(b'\n');
            }
            // Upstream inserts the failure report inline and aborts the
            // export at the end (`GCode.cpp:3920-3933`, `:3637-3645`).
            Err(error) => {
                state.record_template_failure(name, &error.to_string());
                output.extend_from_slice(
                    format!(
                        "\n!!!!! Failed to process the custom G-code template {name}\n{error}\n!!!!! End of an error report for the custom G-code template {name}\n\n"
                    )
                    .as_bytes(),
                );
            }
        }
    }
}
