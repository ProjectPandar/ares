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

/// Upstream E-counter vectors are `MAXIMUM_EXTRUDER_NUMBER` wide
/// (`libslic3r.h:65`, `GCode.cpp:1633-1636`).
const MAXIMUM_EXTRUDER_NUMBER: usize = 64;

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

fn list64(index0: f64) -> Value {
    let mut values = vec![0.0f64; MAXIMUM_EXTRUDER_NUMBER];
    values[0] = index0;
    Value::List(values.into_iter().map(Value::number).collect())
}

fn numeric(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => Some(*n),
        Value::Integer(n) => Some(*n as f64),
        _ => None,
    }
}

/// `PlaceholderParserIntegration::update_from_gcodewriter`
/// (`GCode.cpp:1663-1697`): refresh writer-visible placeholders from the
/// emit state before each render. Vector widths match upstream
/// (`e_retracted`/`e_restart_extra`/`e_position` are 64-wide so reads of
/// unused slots yield 0; `extruded_volume`/`extruded_weight` are
/// num_extruders-wide).
pub(in crate::project_slice::gcode_emit) fn refresh_writer_state(
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
    config.insert("e_retracted", list64(state.retracted_amount));
    config.insert("e_restart_extra", list64(state.restart_extra));
    // `Extruder::extruded_volume` = `used_filament() * filament_crossection`
    // (`Extruder.cpp:125-145`); extruded_weight = volume * density * 1e-3.
    let volume = state.extrusion_totals().used_filament * state.options.filament_area;
    config.insert("extruded_volume", Value::List(vec![Value::number(volume)]));
    config.insert(
        "extruded_weight",
        Value::List(vec![Value::number(
            volume * state.options.filament_density * 0.001,
        )]),
    );
    config.insert("extruded_volume_total", Value::number(volume));
    config.insert(
        "extruded_weight_total",
        Value::number(volume * state.options.filament_density * 0.001),
    );
    if !state.options.use_relative_e_distances {
        config.insert("e_position", list64(state.e_position));
    }
}

/// `validate_output_vector_variables` (`GCode.cpp:1702-1713`): a mutable
/// output vector assigned to the wrong size aborts the export.
fn validate_vector_size(
    name: &str,
    config: &crate::project_slice::gcode_emit::value::Config,
    key: &str,
    expected: usize,
    state: &mut EmitState,
) -> bool {
    let Some(Value::List(list)) = config.get(key) else {
        return true;
    };
    if list.len() != expected {
        state.record_template_failure(
            name,
            &format!("\"{key}\" output variable must not be resized by the script."),
        );
        return false;
    }
    true
}

/// Writer write-back (`GCode.cpp:3903-3915`): assigned `position` and
/// E counters propagate back into the writer so subsequent output uses
/// the overridden values.
/// Returns false when a write-back raised a template error — the caller
/// emits the upstream inline failure block and skips the remaining
/// templates (`GCode.cpp:3920-3933`).
pub(in crate::project_slice::gcode_emit) fn apply_writer_state(
    name: &str,
    config: &crate::project_slice::gcode_emit::value::Config,
    state: &mut EmitState,
) -> bool {
    // Vector-size validation first (`GCode.cpp:3898` calls
    // `validate_output_vector_variables`, `GCode.cpp:1702-1713`): only
    // `position` (exactly 3) and the 64-wide E vectors are checked.
    let valid = validate_vector_size(name, config, "position", 3, state)
        && ["e_retracted", "e_restart_extra", "e_position"]
            .iter()
            .all(|key| validate_vector_size(name, config, key, MAXIMUM_EXTRUDER_NUMBER, state));
    if !valid {
        return false;
    }
    if let Some(Value::List(position)) = config.get("position")
        && let [x, y, z] = position.as_slice()
        && let (Some(x), Some(y), Some(z)) = (numeric(x), numeric(y), numeric(z))
    {
        // `ppi.position != pos` is an exact component compare — any
        // assigned difference writes back (`GCode.cpp:3901-3907`).
        if x != state.x || y != state.y {
            // `set_position` + `set_last_pos(gcode_to_point)`;
            // `gcode_to_point` is `(p - m_origin) + extruder_offset` then
            // `scale_` TRUNCATES the new coordinate (`Point.hpp:665`).
            state.x = x;
            state.y = y;
            state.last_scaled_position = Some((
                ((x - state.origin.0 + state.extruder_offset.0) / state.scale_factor) as i64,
                ((y - state.origin.1 + state.extruder_offset.1) / state.scale_factor) as i64,
            ));
        }
        state.writer_z = Some(z);
    }
    if !state.options.use_relative_e_distances
        && let Some(Value::List(list)) = config.get("e_position")
        && let Some(position) = list.first().and_then(numeric)
        && !is_approx(position, state.e_position)
    {
        state.e_position = position;
    }
    // `e.set_retracted(retracted, restart_extra)` runs when EITHER output
    // differs (`is_approx` = 1e-4, `GCode.cpp:3912-3914`); set_retracted
    // then validates/clamps as a PAIR (`Extruder.cpp:108-122`): negative
    // values fail the template; retracted <= EPSILON clears BOTH.
    let new_retracted = config
        .get("e_retracted")
        .and_then(|value| value.index(0))
        .and_then(|value| value.as_number());
    let new_extra = config
        .get("e_restart_extra")
        .and_then(|value| value.index(0))
        .and_then(|value| value.as_number());
    if let (Some(retracted), Some(extra)) = (new_retracted, new_extra)
        && (!is_approx(retracted, state.retracted_amount) || !is_approx(extra, state.restart_extra))
    {
        if retracted < -1e-4 {
            state.record_template_failure(name, "Custom G-code reports negative z_retracted.");
            return false;
        }
        if extra < -1e-4 {
            state.record_template_failure(name, "Custom G-code reports negative z_restart_extra.");
            return false;
        }
        if retracted > 1e-4 {
            state.retracted = true;
            state.retracted_amount = retracted;
            state.restart_extra = if extra < 1e-4 { 0.0 } else { extra };
        } else {
            state.retracted = false;
            state.retracted_amount = 0.0;
            state.restart_extra = 0.0;
        }
    }
    true
}

/// `is_approx` (`libslic3r.h:307-309`): `fabs(a - b) < EPSILON` with
/// upstream EPSILON = 1e-4 (`libslic3r.h:52`).
fn is_approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-4
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
                if !apply_writer_state(name, &config, state) {
                    let error = state
                        .failed_templates
                        .last()
                        .map(|(_, error)| error.clone())
                        .unwrap_or_default();
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
